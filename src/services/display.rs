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
        let monitors = crate::translators::hyprland::get_monitors();
        let mon_data: Vec<(String, String)> = if !monitors.is_empty() {
            monitors
                .into_iter()
                .map(|m| {
                    let label = if m.width >= 2560 && m.height >= 1440 {
                        format!("LG 2K QHD ({})", m.name)
                    } else if let Some(ref desc) = m.description {
                        format!("{} ({})", desc, m.name)
                    } else {
                        format!("Display ({})", m.name)
                    };
                    (m.name, label)
                })
                .collect()
        } else {
            // Fallback to user's dual 2K setup
            vec![
                ("DP-1".to_string(), "LG 2K QHD (DP-1)".to_string()),
                ("HDMI-A-2".to_string(), "LG 2K QHD (HDMI-A-2)".to_string()),
            ]
        };

        // Query brightness concurrently in parallel threads to minimize latency
        std::thread::scope(|s| {
            let mut handles = Vec::new();
            for (name, label) in mon_data {
                let handle = s.spawn(move || {
                    let b = crate::translators::ddc::get_monitor_brightness(&name).unwrap_or(85);
                    MonitorDisplay {
                        name,
                        label,
                        brightness_percent: b,
                    }
                });
                handles.push(handle);
            }
            handles.into_iter().filter_map(|h| h.join().ok()).collect()
        })
    }

    pub fn set_brightness(name: &str, percent: u32) {
        crate::translators::ddc::set_monitor_brightness(name, percent);
    }
}
