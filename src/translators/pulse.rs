use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

pub struct PulseAudioClient {
    stream: UnixStream,
    tag: u32,
}

unsafe extern "C" {
    fn getuid() -> u32;
}

impl PulseAudioClient {
    pub fn connect() -> Result<Self, String> {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| format!("/run/user/{}", unsafe { getuid() }));
        let socket_path = PathBuf::from(runtime_dir).join("pulse/native");

        let stream = UnixStream::connect(&socket_path)
            .map_err(|e| format!("Failed to connect to pulse socket: {}", e))?;

        stream
            .set_read_timeout(Some(Duration::from_millis(500)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_millis(500)))
            .map_err(|e| e.to_string())?;

        let mut client = Self { stream, tag: 0 };
        client.authenticate()?;
        client.set_client_name("akquick")?;

        Ok(client)
    }

    fn next_tag(&mut self) -> u32 {
        let t = self.tag;
        self.tag = self.tag.wrapping_add(1);
        t
    }

    fn send_command(&mut self, cmd: u32, payload: &[u8]) -> Result<Vec<u8>, String> {
        let tag = self.next_tag();
        let mut buf = Vec::with_capacity(32 + payload.len());

        // Tagstruct Command (Tag 'L' + u32 big endian)
        buf.push(b'L');
        buf.extend_from_slice(&cmd.to_be_bytes());

        // Tagstruct Tag (Tag 'L' + u32 big endian)
        buf.push(b'L');
        buf.extend_from_slice(&tag.to_be_bytes());

        // Payload
        buf.extend_from_slice(payload);

        // Packet Header (20 bytes: length, channel 0xFFFFFFFF, offset_hi, offset_lo, flags)
        let mut hdr = [0u8; 20];
        hdr[0..4].copy_from_slice(&(buf.len() as u32).to_be_bytes());
        hdr[4..8].copy_from_slice(&0xFFFFFFFFu32.to_be_bytes());

        self.stream
            .write_all(&hdr)
            .map_err(|e| format!("Pulse socket write error: {}", e))?;
        self.stream
            .write_all(&buf)
            .map_err(|e| format!("Pulse socket write payload error: {}", e))?;
        self.stream.flush().map_err(|e| e.to_string())?;

        // Read response header
        let mut resp_hdr = [0u8; 20];
        self.stream
            .read_exact(&mut resp_hdr)
            .map_err(|e| format!("Pulse socket read header error: {}", e))?;

        let resp_len = u32::from_be_bytes([resp_hdr[0], resp_hdr[1], resp_hdr[2], resp_hdr[3]]) as usize;
        let mut resp_payload = vec![0u8; resp_len];
        self.stream
            .read_exact(&mut resp_payload)
            .map_err(|e| format!("Pulse socket read payload error: {}", e))?;

        // Check if response is error (cmd == 0)
        if resp_payload.len() >= 5 && resp_payload[0] == b'L' {
            let resp_cmd = u32::from_be_bytes([
                resp_payload[1],
                resp_payload[2],
                resp_payload[3],
                resp_payload[4],
            ]);
            if resp_cmd == 0 {
                return Err("PulseAudio server returned error command".to_string());
            }
        }

        Ok(resp_payload)
    }

    fn authenticate(&mut self) -> Result<(), String> {
        let cookie = Self::read_cookie();
        let mut payload = Vec::with_capacity(300);

        // Protocol version 35 with flags 0xc0000000
        payload.push(b'L');
        payload.extend_from_slice(&(0xC0000000u32 | 35).to_be_bytes());

        // Arbitrary data cookie tag 'x' + len 256 + 256 bytes
        payload.push(b'x');
        payload.extend_from_slice(&(cookie.len() as u32).to_be_bytes());
        payload.extend_from_slice(&cookie);

        self.send_command(8, &payload)?; // PA_COMMAND_AUTH = 8
        Ok(())
    }

    fn set_client_name(&mut self, name: &str) -> Result<(), String> {
        let mut payload = Vec::new();
        payload.push(b'P'); // proplist tag

        Self::write_prop(&mut payload, "application.name", name.as_bytes());
        Self::write_prop(&mut payload, "application.process.binary", b"akquick");
        payload.push(b'N'); // end of proplist

        self.send_command(9, &payload)?; // PA_COMMAND_SET_CLIENT_NAME = 9
        Ok(())
    }

    fn write_prop(buf: &mut Vec<u8>, key: &str, val: &[u8]) {
        buf.push(b't');
        buf.extend_from_slice(key.as_bytes());
        buf.push(0);

        let len = (val.len() + 1) as u32;
        buf.push(b'L');
        buf.extend_from_slice(&len.to_be_bytes());

        buf.push(b'x');
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(val);
        buf.push(0);
    }

    fn read_cookie() -> Vec<u8> {
        if let Ok(home) = std::env::var("HOME") {
            let p = PathBuf::from(home).join(".config/pulse/cookie");
            if let Ok(c) = std::fs::read(p) {
                if c.len() == 256 {
                    return c;
                }
            }
        }
        vec![0u8; 256]
    }

    pub fn get_sink_info(&mut self) -> Result<(String, String, u32, bool), String> {
        let mut payload = Vec::new();
        // PA_INVALID_INDEX = 0xFFFFFFFF
        payload.push(b'L');
        payload.extend_from_slice(&0xFFFFFFFFu32.to_be_bytes());
        // String tag 't' + @DEFAULT_SINK@ + null
        payload.push(b't');
        payload.extend_from_slice(b"@DEFAULT_SINK@\0");

        let rep = self.send_command(21, &payload)?; // PA_COMMAND_GET_SINK_INFO = 21
        Self::parse_device_info(&rep)
    }

    pub fn get_source_info(&mut self) -> Result<(String, String, u32, bool), String> {
        let mut payload = Vec::new();
        payload.push(b'L');
        payload.extend_from_slice(&0xFFFFFFFFu32.to_be_bytes());
        payload.push(b't');
        payload.extend_from_slice(b"@DEFAULT_SOURCE@\0");

        let rep = self.send_command(23, &payload)?; // PA_COMMAND_GET_SOURCE_INFO = 23
        Self::parse_device_info(&rep)
    }

    fn parse_device_info(rep: &[u8]) -> Result<(String, String, u32, bool), String> {
        if rep.len() < 10 || rep[0] != b'L' {
            return Err("Invalid tagstruct reply".to_string());
        }

        let mut offset = 10;
        let mut name = String::new();
        let mut desc = String::new();
        let mut volume = 0u32;
        let mut muted = false;

        // Field 0: Index: Tag 'L' (5 bytes)
        if offset < rep.len() && rep[offset] == b'L' {
            offset += 5;
        }

        // Field 1: Name: Tag 't' (string) or 'N' (null)
        if offset < rep.len() && rep[offset] == b't' {
            offset += 1;
            let start = offset;
            while offset < rep.len() && rep[offset] != 0 {
                offset += 1;
            }
            if let Ok(s) = std::str::from_utf8(&rep[start..offset]) {
                name = s.to_string();
            }
            if offset < rep.len() && rep[offset] == 0 {
                offset += 1;
            }
        } else if offset < rep.len() && rep[offset] == b'N' {
            offset += 1;
        }

        // Field 2: Description: Tag 't' (string) or 'N' (null)
        if offset < rep.len() && rep[offset] == b't' {
            offset += 1;
            let start = offset;
            while offset < rep.len() && rep[offset] != 0 {
                offset += 1;
            }
            if let Ok(s) = std::str::from_utf8(&rep[start..offset]) {
                desc = s.to_string();
            }
            if offset < rep.len() && rep[offset] == 0 {
                offset += 1;
            }
        } else if offset < rep.len() && rep[offset] == b'N' {
            offset += 1;
        }

        // Field 3: Sample spec: Tag 'a' (1 + 1 + 1 + 4 = 7 bytes)
        if offset < rep.len() && rep[offset] == b'a' {
            offset += 7;
        }

        // Field 4: Channel map: Tag 'm' (1 + 1 + channels bytes)
        if offset + 1 < rep.len() && rep[offset] == b'm' {
            let channels = rep[offset + 1] as usize;
            offset += 2 + channels;
        }

        // Field 5: Module index: Tag 'L' (5 bytes)
        if offset < rep.len() && rep[offset] == b'L' {
            offset += 5;
        }

        // Field 6: Volume: Tag 'v' (1 + 1 + channels * 4 bytes)
        if offset + 1 < rep.len() && rep[offset] == b'v' {
            let channels = rep[offset + 1] as usize;
            let vol_start = offset + 2;
            if vol_start + 4 <= rep.len() {
                let v = u32::from_be_bytes([
                    rep[vol_start],
                    rep[vol_start + 1],
                    rep[vol_start + 2],
                    rep[vol_start + 3],
                ]);
                volume = ((v as f64 * 100.0) / 65536.0).round() as u32;
            }
            offset += 2 + channels * 4;
        }

        // Field 7: Muted: Tag '0' (false) or '1' (true)
        if offset < rep.len() {
            if rep[offset] == b'0' {
                muted = false;
                #[allow(unused_assignments)]
                {
                    offset += 1;
                }
            } else if rep[offset] == b'1' {
                muted = true;
                #[allow(unused_assignments)]
                {
                    offset += 1;
                }
            }
        }

        // Proplist check: if "device.description" exists in proplist, prefer it if cleaner
        if let Some(pos) = rep.windows(19).position(|w| w == b"device.description\0") {
            let val_start = pos + 19;
            if val_start + 10 < rep.len() && rep[val_start] == b'L' && rep[val_start + 5] == b'x' {
                let prop_len = u32::from_be_bytes([
                    rep[val_start + 1],
                    rep[val_start + 2],
                    rep[val_start + 3],
                    rep[val_start + 4],
                ]) as usize;
                let s_start = val_start + 10;
                let s_end = (s_start + prop_len).min(rep.len());
                if let Ok(pretty) = std::str::from_utf8(&rep[s_start..s_end]) {
                    let clean = pretty.trim_matches('\0').trim();
                    if !clean.is_empty() && clean != "PipeWire" {
                        desc = clean.to_string();
                    }
                }
            }
        }

        Ok((name, desc, volume, muted))
    }

    pub fn set_sink_volume(&mut self, percent: u32) -> Result<(), String> {
        let vol = (((percent.min(150) as f64) * 65536.0) / 100.0).round() as u32;
        let mut payload = Vec::new();
        payload.push(b'L');
        payload.extend_from_slice(&0xFFFFFFFFu32.to_be_bytes());
        payload.push(b't');
        payload.extend_from_slice(b"@DEFAULT_SINK@\0");

        // Tag 'v' + 2 channels + 2 volumes
        payload.push(b'v');
        payload.push(2);
        payload.extend_from_slice(&vol.to_be_bytes());
        payload.extend_from_slice(&vol.to_be_bytes());

        self.send_command(36, &payload)?; // PA_COMMAND_SET_SINK_VOLUME = 36
        Ok(())
    }

    pub fn set_sink_mute(&mut self, mute: bool) -> Result<(), String> {
        let mut payload = Vec::new();
        payload.push(b'L');
        payload.extend_from_slice(&0xFFFFFFFFu32.to_be_bytes());
        payload.push(b't');
        payload.extend_from_slice(b"@DEFAULT_SINK@\0");
        payload.push(if mute { b'1' } else { b'0' });

        self.send_command(39, &payload)?; // PA_COMMAND_SET_SINK_MUTE = 39
        Ok(())
    }

    pub fn set_source_volume(&mut self, percent: u32) -> Result<(), String> {
        let vol = (((percent.min(100) as f64) * 65536.0) / 100.0).round() as u32;
        let mut payload = Vec::new();
        payload.push(b'L');
        payload.extend_from_slice(&0xFFFFFFFFu32.to_be_bytes());
        payload.push(b't');
        payload.extend_from_slice(b"@DEFAULT_SOURCE@\0");

        payload.push(b'v');
        payload.push(2);
        payload.extend_from_slice(&vol.to_be_bytes());
        payload.extend_from_slice(&vol.to_be_bytes());

        self.send_command(38, &payload)?; // PA_COMMAND_SET_SOURCE_VOLUME = 38
        Ok(())
    }

    pub fn set_source_mute(&mut self, mute: bool) -> Result<(), String> {
        let mut payload = Vec::new();
        payload.push(b'L');
        payload.extend_from_slice(&0xFFFFFFFFu32.to_be_bytes());
        payload.push(b't');
        payload.extend_from_slice(b"@DEFAULT_SOURCE@\0");
        payload.push(if mute { b'1' } else { b'0' });

        self.send_command(40, &payload)?; // PA_COMMAND_SET_SOURCE_MUTE = 40
        Ok(())
    }
}
