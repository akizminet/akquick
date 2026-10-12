use std::path::PathBuf;

pub fn socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("akquick.sock")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcCommand {
    StatusFull,
    StatusJson,
    StatusIcons,
    StatusWifi,
    StatusBt,
    StatusNet,
    StatusAudio,
    StatusMic,
    Toggle,
    OpenPage(String),
    Volume(String),
    ToggleVolumeMute,
    ToggleMicMute,
    Unknown(String),
}

impl IpcCommand {
    pub fn parse(s: &str) -> Self {
        let trimmed = s.trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return IpcCommand::Unknown(String::new());
        }

        match parts[0].to_lowercase().as_str() {
            "status" | "status_full" | "status-full" => IpcCommand::StatusFull,
            "status_json" | "status-json" => IpcCommand::StatusJson,
            "status_icons" | "status-icons" => IpcCommand::StatusIcons,
            "status_wifi" | "status-wifi" => IpcCommand::StatusWifi,
            "status_bt" | "status-bt" | "status_bluetooth" | "status-bluetooth" => IpcCommand::StatusBt,
            "status_net" | "status-net" => IpcCommand::StatusNet,
            "status_audio" | "status-audio" => IpcCommand::StatusAudio,
            "status_mic" | "status-mic" => IpcCommand::StatusMic,
            "toggle" => IpcCommand::Toggle,
            "open" => {
                let page = parts.get(1).copied().unwrap_or("main").to_string();
                IpcCommand::OpenPage(page)
            }
            "volume" => {
                let arg = parts.get(1).copied().unwrap_or("toggle").to_string();
                IpcCommand::Volume(arg)
            }
            "mute" | "toggle-mute" => IpcCommand::ToggleVolumeMute,
            "mic-mute" | "toggle-mic-mute" => IpcCommand::ToggleMicMute,
            _ => IpcCommand::Unknown(trimmed.to_string()),
        }
    }

    pub fn to_wire_string(&self) -> String {
        match self {
            IpcCommand::StatusFull => "status\n".to_string(),
            IpcCommand::StatusJson => "status_json\n".to_string(),
            IpcCommand::StatusIcons => "status_icons\n".to_string(),
            IpcCommand::StatusWifi => "status_wifi\n".to_string(),
            IpcCommand::StatusBt => "status_bt\n".to_string(),
            IpcCommand::StatusNet => "status_net\n".to_string(),
            IpcCommand::StatusAudio => "status_audio\n".to_string(),
            IpcCommand::StatusMic => "status_mic\n".to_string(),
            IpcCommand::Toggle => "toggle\n".to_string(),
            IpcCommand::OpenPage(p) => format!("open {}\n", p),
            IpcCommand::Volume(v) => format!("volume {}\n", v),
            IpcCommand::ToggleVolumeMute => "mute\n".to_string(),
            IpcCommand::ToggleMicMute => "mic-mute\n".to_string(),
            IpcCommand::Unknown(s) => format!("{}\n", s),
        }
    }
}
