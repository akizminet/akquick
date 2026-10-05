use std::process::Command;
use std::fs;

#[derive(Debug, Clone, Default)]
pub struct SystemState {
    pub volume_percent: u32,
    pub volume_muted: bool,
    pub mic_percent: u32,
    pub mic_muted: bool,
    pub brightness_percent: u32,
    pub wifi_enabled: bool,
    pub wifi_ssid: String,
    pub bluetooth_enabled: bool,
    pub bluetooth_connected: u32,
    pub dnd_active: bool,
    pub night_light_active: bool,
    pub wireguard_active: bool,
    pub power_mode: String,
    pub dark_active: bool,
    pub battery_percent: u32,
    pub battery_time: String,
    pub mpris_title: String,
    pub mpris_artist: String,
    pub mpris_player: String,
    pub mpris_playing: bool,
}

impl SystemState {
    pub fn fetch() -> Self {
        let mut state = SystemState::default();

        // 1. Audio volume via wpctl
        if let Ok(out) = Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SINK@"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            // Example: "Volume: 0.72 [MUTED]"
            let mut parts = s.split_whitespace();
            if let Some(vol_str) = parts.nth(1) {
                if let Ok(vol_f) = vol_str.parse::<f32>() {
                    state.volume_percent = (vol_f * 100.0).round() as u32;
                }
            }
            state.volume_muted = s.contains("[MUTED]");
        } else {
            state.volume_percent = 72;
        }

        // 2. Mic volume via wpctl
        if let Ok(out) = Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SOURCE@"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            let mut parts = s.split_whitespace();
            if let Some(vol_str) = parts.nth(1) {
                if let Ok(vol_f) = vol_str.parse::<f32>() {
                    state.mic_percent = (vol_f * 100.0).round() as u32;
                }
            }
            state.mic_muted = s.contains("[MUTED]");
        } else {
            state.mic_percent = 65;
        }

        // 3. Brightness via brightnessctl or sysfs
        state.brightness_percent = Self::fetch_brightness();

        // 4. Wi-Fi SSID & Status
        if let Ok(out) = Command::new("nmcli").args(["-t", "-f", "ACTIVE,SSID", "dev", "wifi"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            let mut found = false;
            for line in s.lines() {
                if line.starts_with("yes:") {
                    state.wifi_enabled = true;
                    state.wifi_ssid = line.trim_start_matches("yes:").to_string();
                    found = true;
                    break;
                }
            }
            if !found {
                state.wifi_enabled = false;
                state.wifi_ssid = "Disconnected".to_string();
            }
        } else {
            state.wifi_enabled = true;
            state.wifi_ssid = "Wi-Fi Active".to_string();
        }

        // 5. Bluetooth
        if let Ok(out) = Command::new("bluetoothctl").args(["devices", "Connected"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            let count = s.lines().filter(|l| l.contains("Device")).count() as u32;
            state.bluetooth_connected = count;
            state.bluetooth_enabled = true;
        } else {
            state.bluetooth_enabled = true;
            state.bluetooth_connected = 0;
        }

        // 6. Battery from /sys/class/power_supply
        state.battery_percent = Self::fetch_battery();
        state.battery_time = "5h 20m rem.".to_string();

        // 7. Night light check (e.g. hyprsunset or gammastep running)
        state.night_light_active = Self::is_process_running("hyprsunset") || Self::is_process_running("gammastep");

        // 8. WireGuard / VPN check
        state.wireguard_active = Self::is_interface_up("wg") || Self::is_interface_up("tun");

        // 9. Power mode & Dark style
        state.power_mode = "Balanced".to_string();
        state.dark_active = true;

        // 10. MPRIS Media Player
        if let Ok(out) = Command::new("playerctl").args(["metadata", "--format", "{{artist}} - {{title}} ({{status}}) [{{playerName}}]"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            if !s.trim().is_empty() {
                // Parse simple string
                state.mpris_artist = Self::playerctl_get("artist").unwrap_or_else(|| "Solaris".into());
                state.mpris_title = Self::playerctl_get("title").unwrap_or_else(|| "Carbon Based Lifeforms".into());
                state.mpris_player = Self::playerctl_get("playerName").unwrap_or_else(|| "Spotify Wayland".into());
                state.mpris_playing = s.contains("Playing");
            } else {
                Self::fill_fallback_mpris(&mut state);
            }
        } else {
            Self::fill_fallback_mpris(&mut state);
        }

        state
    }

    fn fill_fallback_mpris(state: &mut SystemState) {
        state.mpris_title = "Solaris".to_string();
        state.mpris_artist = "Carbon Based Lifeforms".to_string();
        state.mpris_player = "SPOTIFY WAYLAND MPRIS".to_string();
        state.mpris_playing = true;
    }

    fn playerctl_get(field: &str) -> Option<String> {
        let arg = format!("{{{{{}}}}}", field);
        if let Ok(out) = Command::new("playerctl").args(["metadata", "--format", &arg]).output() {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !val.is_empty() {
                return Some(val);
            }
        }
        None
    }

    fn fetch_brightness() -> u32 {
        if let Ok(out) = Command::new("brightnessctl").args(["get"]).output() {
            let cur = String::from_utf8_lossy(&out.stdout).trim().parse::<f32>().unwrap_or(85.0);
            if let Ok(max_out) = Command::new("brightnessctl").args(["max"]).output() {
                let max = String::from_utf8_lossy(&max_out.stdout).trim().parse::<f32>().unwrap_or(100.0);
                return ((cur / max) * 100.0).round() as u32;
            }
        }
        85
    }

    fn fetch_battery() -> u32 {
        if let Ok(entries) = fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("BAT") {
                    let cap_path = entry.path().join("capacity");
                    if let Ok(cap_str) = fs::read_to_string(cap_path) {
                        if let Ok(pct) = cap_str.trim().parse::<u32>() {
                            return pct;
                        }
                    }
                }
            }
        }
        88
    }

    fn is_process_running(proc_name: &str) -> bool {
        Command::new("pgrep")
            .arg("-x")
            .arg(proc_name)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn is_interface_up(prefix: &str) -> bool {
        if let Ok(entries) = fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(prefix) {
                    return true;
                }
            }
        }
        false
    }

    // Actions
    pub fn set_volume(percent: u32) {
        let vol_arg = format!("{}%", percent.min(150));
        let _ = Command::new("wpctl")
            .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &vol_arg])
            .spawn();
    }

    pub fn toggle_volume_mute() {
        let _ = Command::new("wpctl")
            .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
            .spawn();
    }

    pub fn set_mic_volume(percent: u32) {
        let vol_arg = format!("{}%", percent.min(100));
        let _ = Command::new("wpctl")
            .args(["set-volume", "@DEFAULT_AUDIO_SOURCE@", &vol_arg])
            .spawn();
    }

    pub fn toggle_mic_mute() {
        let _ = Command::new("wpctl")
            .args(["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"])
            .spawn();
    }

    pub fn set_brightness(percent: u32) {
        let arg = format!("{}%", percent.min(100));
        let _ = Command::new("brightnessctl")
            .args(["set", &arg])
            .spawn();
    }

    pub fn mpris_play_pause() {
        let _ = Command::new("playerctl").arg("play-pause").spawn();
    }

    pub fn mpris_previous() {
        let _ = Command::new("playerctl").arg("previous").spawn();
    }

    pub fn mpris_next() {
        let _ = Command::new("playerctl").arg("next").spawn();
    }

    pub fn take_screenshot() {
        let _ = Command::new("satty")
            .args(["--filename", "/tmp/screenshot.png"])
            .spawn()
            .or_else(|_| {
                Command::new("sh")
                    .arg("-c")
                    .arg("grim -g \"$(slurp)\" /tmp/screenshot.png && satty -f /tmp/screenshot.png")
                    .spawn()
            });
    }

    pub fn open_settings() {
        let _ = Command::new("gnome-control-center")
            .spawn()
            .or_else(|_| Command::new("nmrs-gui").spawn());
    }

    pub fn lock_session() {
        let _ = Command::new("hyprlock").spawn();
    }

    pub fn power_menu() {
        let _ = Command::new("/var/home/phamnv/.config/rofi/powermenu.sh")
            .spawn()
            .or_else(|_| Command::new("wlogout").spawn());
    }
}
