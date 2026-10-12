use crate::services::{AudioService, BluetoothService, BluetoothState, NetworkService, NetworkStatus};
use std::io::Write;

pub struct FullStatus {
    pub wifi_icon: &'static str,
    pub wifi_name: String,
    pub down_speed: String,
    pub up_speed: String,
    pub bt_icon: &'static str,
    pub bt_enabled: bool,
    pub bt_connected: bool,
    pub audio_icon: &'static str,
    pub audio_vol: u32,
    pub audio_muted: bool,
    pub mic_icon: &'static str,
    pub mic_vol: u32,
    pub mic_muted: bool,
    pub has_mic: bool,
}

impl FullStatus {
    pub fn from_components(
        network: &NetworkStatus,
        audio: &crate::services::AudioState,
        bluetooth: &BluetoothState,
    ) -> Self {
        let (wifi_icon, raw_wifi_name) = if network.primary_is_ethernet || network.ethernet_connected {
            ("󰈀", if network.ethernet_name.is_empty() { "Ethernet".to_string() } else { network.ethernet_name.clone() })
        } else if !network.wifi_enabled {
            ("󰤭", "Wi-Fi Off".to_string())
        } else if network.wifi_ssid.is_empty() || network.wifi_ssid == "Disconnected" {
            ("󰤮", "Disconnected".to_string())
        } else {
            ("󰤨", network.wifi_ssid.clone())
        };

        let wifi_name = if raw_wifi_name.chars().count() > 18 {
            let truncated: String = raw_wifi_name.chars().take(16).collect();
            format!("{}…", truncated)
        } else {
            raw_wifi_name
        };

        let (bt_icon, bt_connected) = if !bluetooth.enabled {
            ("󰂲", false)
        } else if bluetooth.connected_count > 0 {
            ("󰂱", true)
        } else {
            ("󰂯", false)
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

        let (down_speed, up_speed) = crate::translators::sysfs::get_network_speed();

        Self {
            wifi_icon,
            wifi_name,
            down_speed,
            up_speed,
            bt_icon,
            bt_enabled: bluetooth.enabled,
            bt_connected,
            audio_icon,
            audio_vol: audio.volume_percent,
            audio_muted: audio.volume_muted,
            mic_icon,
            mic_vol: audio.mic_percent,
            mic_muted: audio.mic_muted,
            has_mic: audio.has_input_device,
        }
    }

    pub async fn fetch() -> Self {
        let (network, audio, bluetooth) = tokio::join!(
            NetworkService::fetch_status(),
            async { AudioService::fetch() },
            BluetoothService::fetch(),
        );
        Self::from_components(&network, &audio, &bluetooth)
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
            "{} {}  ⬇{} ⬆{}  {}  {}  {}",
            self.wifi_icon, self.wifi_name, self.down_speed, self.up_speed, self.bt_icon, audio_str, mic_str
        )
    }

    pub fn format_icons_only(&self) -> String {
        format!("{}  {}  {}  {}", self.wifi_icon, self.bt_icon, self.audio_icon, self.mic_icon)
    }
}

pub fn handle_cli(args: &[String]) -> Option<glib::ExitCode> {
    let is_full_status = args.iter().any(|a| a == "--status" || a == "-s" || a == "--status-full");
    let is_icons = args.iter().any(|a| a == "--status-icons");
    let is_wifi = args.iter().any(|a| a == "--status-wifi");
    let is_bt = args.iter().any(|a| a == "--status-bt" || a == "--status-bluetooth");
    let is_net = args.iter().any(|a| a == "--status-net" || a == "--status-speed");
    let is_audio = args.iter().any(|a| a == "--status-audio");
    let is_mic = args.iter().any(|a| a == "--status-mic");
    let is_json = args.iter().any(|a| a == "--status-json");
    let is_watch = args.iter().any(|a| a == "--watch" || a == "-w");
    let is_watch_icons = args.iter().any(|a| a == "--watch-icons");

    if !(is_full_status || is_icons || is_wifi || is_bt || is_net || is_audio || is_mic || is_json || is_watch || is_watch_icons) {
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
                r#"{{"wifi_icon":"{}","wifi_name":"{}","down_speed":"{}","up_speed":"{}","bt_icon":"{}","bt_enabled":{},"bt_connected":{},"audio_icon":"{}","audio_volume":{},"audio_muted":{},"mic_icon":"{}","mic_volume":{},"mic_muted":{}}}"#,
                status.wifi_icon,
                status.wifi_name,
                status.down_speed,
                status.up_speed,
                status.bt_icon,
                status.bt_enabled,
                status.bt_connected,
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
        } else if is_bt {
            println!("{}", status.bt_icon);
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
            // Default: full status with wifi name, speed, bluetooth, output volume, mic volume
            println!("{}", status.format_full());
        }
    });

    Some(glib::ExitCode::SUCCESS)
}
