use crate::services::{AudioService, NetworkService};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct FullStatus {
    pub wifi_icon: &'static str,
    pub wifi_name: String,
    pub down_speed: String,
    pub up_speed: String,
    pub audio_icon: &'static str,
    pub audio_vol: u32,
    pub audio_muted: bool,
    pub mic_icon: &'static str,
    pub mic_vol: u32,
    pub mic_muted: bool,
    pub has_mic: bool,
}

impl FullStatus {
    pub async fn fetch() -> Self {
        let (network, audio) = tokio::join!(
            NetworkService::fetch_status(),
            async { AudioService::fetch() },
        );

        let (wifi_icon, raw_wifi_name) = if !network.wifi_enabled {
            ("󰤭", "Wi-Fi Off".to_string())
        } else if network.wifi_ssid.is_empty() || network.wifi_ssid == "Disconnected" {
            ("󰤮", "Disconnected".to_string())
        } else {
            ("󰤨", network.wifi_ssid)
        };

        let wifi_name = if raw_wifi_name.chars().count() > 18 {
            let truncated: String = raw_wifi_name.chars().take(16).collect();
            format!("{}…", truncated)
        } else {
            raw_wifi_name
        };

        let audio_icon = if audio.volume_muted || audio.volume_percent == 0 {
            "󰝟"
        } else if audio.volume_percent > 66 {
            "󰕾"
        } else if audio.volume_percent > 33 {
            "󰖀"
        } else {
            "󰕿"
        };

        let mic_icon = if !audio.has_input_device || audio.mic_muted || audio.mic_percent == 0 {
            "󰍭"
        } else {
            "󰍬"
        };

        let (down_speed, up_speed) = get_network_speed();

        Self {
            wifi_icon,
            wifi_name,
            down_speed,
            up_speed,
            audio_icon,
            audio_vol: audio.volume_percent,
            audio_muted: audio.volume_muted,
            mic_icon,
            mic_vol: audio.mic_percent,
            mic_muted: audio.mic_muted,
            has_mic: audio.has_input_device,
        }
    }

    pub fn format_full(&self) -> String {
        let audio_str = if self.audio_muted {
            format!("󰝟 {}%", self.audio_vol)
        } else {
            format!("{} {}%", self.audio_icon, self.audio_vol)
        };

        let mic_str = if !self.has_mic {
            "󰍭 0%".to_string()
        } else if self.mic_muted {
            format!("󰍭 {}%", self.mic_vol)
        } else {
            format!("{} {}%", self.mic_icon, self.mic_vol)
        };

        format!(
            "{} {}  ⬇{} ⬆{}  {}  {}",
            self.wifi_icon, self.wifi_name, self.down_speed, self.up_speed, audio_str, mic_str
        )
    }

    pub fn format_icons_only(&self) -> String {
        format!("{}  {}  {}", self.wifi_icon, self.audio_icon, self.mic_icon)
    }
}

fn get_total_rx_tx() -> (u64, u64) {
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;

    if let Ok(content) = std::fs::read_to_string("/proc/net/dev") {
        for line in content.lines() {
            if let Some(pos) = line.find(':') {
                let name = line[..pos].trim();
                if name != "lo" {
                    let parts: Vec<&str> = line[pos + 1..].split_whitespace().collect();
                    if parts.len() >= 9 {
                        let rx = parts[0].parse::<u64>().unwrap_or(0);
                        let tx = parts[8].parse::<u64>().unwrap_or(0);
                        total_rx += rx;
                        total_tx += tx;
                    }
                }
            }
        }
    }
    (total_rx, total_tx)
}

fn format_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec >= 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1}G", bytes_per_sec / (1024.0 * 1024.0 * 1024.0))
    } else if bytes_per_sec >= 1024.0 * 1024.0 {
        format!("{:.1}M", bytes_per_sec / (1024.0 * 1024.0))
    } else if bytes_per_sec >= 1024.0 {
        format!("{:.0}K", bytes_per_sec / 1024.0)
    } else {
        "0K".to_string()
    }
}

fn get_network_speed() -> (String, String) {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let (curr_rx, curr_tx) = get_total_rx_tx();
    let stat_file = "/dev/shm/akquick_net_stat";

    let mut down_speed = 0.0;
    let mut up_speed = 0.0;

    if let Ok(content) = std::fs::read_to_string(stat_file) {
        let parts: Vec<&str> = content.trim().split_whitespace().collect();
        if parts.len() == 3 {
            if let (Ok(prev_time), Ok(prev_rx), Ok(prev_tx)) = (
                parts[0].parse::<u64>(),
                parts[1].parse::<u64>(),
                parts[2].parse::<u64>(),
            ) {
                if now_ms > prev_time && curr_rx >= prev_rx && curr_tx >= prev_tx {
                    let delta_sec = (now_ms - prev_time) as f64 / 1000.0;
                    if delta_sec > 0.05 && delta_sec < 10.0 {
                        down_speed = (curr_rx - prev_rx) as f64 / delta_sec;
                        up_speed = (curr_tx - prev_tx) as f64 / delta_sec;
                    }
                }
            }
        }
    }

    let _ = std::fs::write(stat_file, format!("{} {} {}", now_ms, curr_rx, curr_tx));

    (format_speed(down_speed), format_speed(up_speed))
}

pub fn handle_cli(args: &[String]) -> Option<glib::ExitCode> {
    let is_full_status = args.iter().any(|a| a == "--status" || a == "-s" || a == "--status-full");
    let is_icons = args.iter().any(|a| a == "--status-icons");
    let is_wifi = args.iter().any(|a| a == "--status-wifi");
    let is_net = args.iter().any(|a| a == "--status-net" || a == "--status-speed");
    let is_audio = args.iter().any(|a| a == "--status-audio");
    let is_mic = args.iter().any(|a| a == "--status-mic");
    let is_json = args.iter().any(|a| a == "--status-json");
    let is_watch = args.iter().any(|a| a == "--watch" || a == "-w");
    let is_watch_icons = args.iter().any(|a| a == "--watch-icons");

    if !(is_full_status || is_icons || is_wifi || is_net || is_audio || is_mic || is_json || is_watch || is_watch_icons) {
        return None;
    }

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build status tokio runtime");

    rt.block_on(async {
        if is_watch || is_watch_icons {
            let mut last_output = String::new();
            loop {
                let status = FullStatus::fetch().await;
                let current = if is_watch_icons {
                    status.format_icons_only()
                } else {
                    status.format_full()
                };
                if current != last_output {
                    println!("{}", current);
                    let _ = std::io::stdout().flush();
                    last_output = current;
                }
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            }
        }

        let status = FullStatus::fetch().await;

        if is_json {
            println!(
                r#"{{"wifi_icon":"{}","wifi_name":"{}","down_speed":"{}","up_speed":"{}","audio_icon":"{}","audio_volume":{},"audio_muted":{},"mic_icon":"{}","mic_volume":{},"mic_muted":{}}}"#,
                status.wifi_icon,
                status.wifi_name,
                status.down_speed,
                status.up_speed,
                status.audio_icon,
                status.audio_vol,
                status.audio_muted,
                status.mic_icon,
                status.mic_vol,
                status.mic_muted
            );
        } else if is_icons {
            println!("{}", status.format_icons_only());
        } else if is_wifi {
            println!("{} {}", status.wifi_icon, status.wifi_name);
        } else if is_net {
            println!("⬇{} ⬆{}", status.down_speed, status.up_speed);
        } else if is_audio {
            if status.audio_muted {
                println!("󰝟 {}%", status.audio_vol);
            } else {
                println!("{} {}%", status.audio_icon, status.audio_vol);
            }
        } else if is_mic {
            if !status.has_mic {
                println!("󰍭 0%");
            } else if status.mic_muted {
                println!("󰍭 {}%", status.mic_vol);
            } else {
                println!("{} {}%", status.mic_icon, status.mic_vol);
            }
        } else {
            // Default: full status with wifi name, speed, output volume, mic volume
            println!("{}", status.format_full());
        }
    });

    Some(glib::ExitCode::SUCCESS)
}
