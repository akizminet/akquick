use std::process::Command;
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct HyprMonitor {
    pub id: Option<i64>,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub model: String,
    pub width: u32,
    pub height: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MonitorDisplay {
    pub name: String,
    pub label: String,
    pub brightness_percent: u32,
}

pub struct DisplayService;

impl DisplayService {
    pub fn fetch() -> Vec<MonitorDisplay> {
        if let Ok(out) = Command::new("hyprctl").args(["monitors", "-j"]).output() {
            if let Ok(monitors) = serde_json::from_slice::<Vec<HyprMonitor>>(&out.stdout) {
                if !monitors.is_empty() {
                    return monitors
                        .into_iter()
                        .map(|m| {
                            let label = if m.width >= 2560 && m.height >= 1440 {
                                format!("LG 2K QHD ({})", m.name)
                            } else {
                                format!("{} ({})", m.model, m.name)
                            };
                            MonitorDisplay {
                                name: m.name,
                                label,
                                brightness_percent: 85,
                            }
                        })
                        .collect();
                }
            }
        }

        // Fallback to user's dual 2K setup
        vec![
            MonitorDisplay {
                name: "DP-1".to_string(),
                label: "LG 2K QHD (DP-1)".to_string(),
                brightness_percent: 85,
            },
            MonitorDisplay {
                name: "HDMI-A-2".to_string(),
                label: "LG 2K QHD (HDMI-A-2)".to_string(),
                brightness_percent: 85,
            },
        ]
    }
}
