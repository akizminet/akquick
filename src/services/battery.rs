use zbus::proxy;
use std::fs;

#[proxy(
    default_service = "org.freedesktop.UPower",
    default_path = "/org/freedesktop/UPower",
    interface = "org.freedesktop.UPower"
)]
trait UPower {
    fn enumerate_devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

#[proxy(
    default_service = "org.freedesktop.UPower",
    interface = "org.freedesktop.UPower.Device"
)]
trait UPowerDevice {
    #[zbus(property)]
    fn percentage(&self) -> zbus::Result<f64>;

    #[zbus(property)]
    fn time_to_empty(&self) -> zbus::Result<i64>;

    #[zbus(property)]
    fn time_to_full(&self) -> zbus::Result<i64>;

    #[zbus(property)]
    fn state(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn is_present(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn type_(&self) -> zbus::Result<u32>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatteryState {
    pub is_present: bool,
    pub percentage: u32,
    pub time_remaining: String,
    pub is_charging: bool,
}

impl Default for BatteryState {
    fn default() -> Self {
        Self {
            is_present: false,
            percentage: 0,
            time_remaining: String::new(),
            is_charging: false,
        }
    }
}

impl BatteryState {
    #[allow(dead_code)]
    pub fn icon_name(&self) -> &'static str {
        if !self.is_present {
            return "ac-adapter-symbolic";
        }
        if self.is_charging {
            if self.percentage >= 95 {
                return "battery-level-100-charged-symbolic";
            }
            match self.percentage {
                0..=15 => "battery-level-10-charging-symbolic",
                16..=25 => "battery-level-20-charging-symbolic",
                26..=35 => "battery-level-30-charging-symbolic",
                36..=45 => "battery-level-40-charging-symbolic",
                46..=55 => "battery-level-50-charging-symbolic",
                56..=65 => "battery-level-60-charging-symbolic",
                66..=75 => "battery-level-70-charging-symbolic",
                76..=85 => "battery-level-80-charging-symbolic",
                86..=95 => "battery-level-90-charging-symbolic",
                _ => "battery-level-100-charging-symbolic",
            }
        } else {
            match self.percentage {
                0..=15 => "battery-level-10-symbolic",
                16..=25 => "battery-level-20-symbolic",
                26..=35 => "battery-level-30-symbolic",
                36..=45 => "battery-level-40-symbolic",
                46..=55 => "battery-level-50-symbolic",
                56..=65 => "battery-level-60-symbolic",
                66..=75 => "battery-level-70-symbolic",
                76..=85 => "battery-level-80-symbolic",
                86..=95 => "battery-level-90-symbolic",
                _ => "battery-level-100-symbolic",
            }
        }
    }
}

pub struct BatteryService;

impl BatteryService {
    pub async fn fetch() -> BatteryState {
        // 1. Try UPower over D-Bus
        if let Ok(conn) = zbus::Connection::system().await {
            if let Ok(upower) = UPowerProxy::new(&conn).await {
                if let Ok(devices) = upower.enumerate_devices().await {
                    for dev_path in devices {
                        let path_str = dev_path.as_str();
                        if path_str.contains("battery") {
                            if let Ok(builder) = UPowerDeviceProxy::builder(&conn).path(dev_path) {
                                if let Ok(dev) = builder.build().await {
                                    let dev_type = dev.type_().await.unwrap_or(0);
                                    if dev_type != 2 {
                                        // 2 = UP_DEVICE_KIND_BATTERY. Skip peripheral batteries (mouse, keyboard, etc.)
                                        continue;
                                    }

                                    let is_present = dev.is_present().await.unwrap_or(true);
                                    if !is_present {
                                        continue;
                                    }

                                    let pct = dev.percentage().await.unwrap_or(0.0).clamp(0.0, 100.0).round() as u32;
                                    let st = dev.state().await.unwrap_or(0);
                                    let is_charging = st == 1; // 1 = Charging
                                    let is_full = st == 4; // 4 = Fully charged

                                    let mut time_remaining = String::new();
                                    if is_charging {
                                        if let Ok(secs) = dev.time_to_full().await {
                                            if secs > 0 {
                                                let hours = secs / 3600;
                                                let mins = (secs % 3600) / 60;
                                                time_remaining = format!("{}h {}m to full", hours, mins);
                                            }
                                        }
                                        if time_remaining.is_empty() {
                                            time_remaining = "Charging".to_string();
                                        }
                                    } else if is_full {
                                        time_remaining = "Full".to_string();
                                    } else if let Ok(secs) = dev.time_to_empty().await {
                                        if secs > 0 {
                                            let hours = secs / 3600;
                                            let mins = (secs % 3600) / 60;
                                            time_remaining = format!("{}h {}m rem.", hours, mins);
                                        }
                                    }

                                    return BatteryState {
                                        is_present: true,
                                        percentage: pct,
                                        time_remaining,
                                        is_charging,
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Sysfs fallback (/sys/class/power_supply)
        if let Some(sysfs_state) = Self::fetch_sysfs_battery() {
            return sysfs_state;
        }

        // 3. No battery present (Desktop or AC power)
        BatteryState::default()
    }

    fn fetch_sysfs_battery() -> Option<BatteryState> {
        let entries = fs::read_dir("/sys/class/power_supply").ok()?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("BAT") {
                let path = entry.path();
                // Check if present
                if let Ok(pres) = fs::read_to_string(path.join("present")) {
                    if pres.trim() == "0" {
                        continue;
                    }
                }

                let cap_str = fs::read_to_string(path.join("capacity")).ok()?;
                let pct = cap_str.trim().parse::<u32>().ok()?;

                let status = fs::read_to_string(path.join("status"))
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                let is_charging = status.eq_ignore_ascii_case("Charging");
                let is_full = status.eq_ignore_ascii_case("Full");

                let time_remaining = if is_charging {
                    "Charging".to_string()
                } else if is_full {
                    "Full".to_string()
                } else {
                    String::new()
                };

                return Some(BatteryState {
                    is_present: true,
                    percentage: pct,
                    time_remaining,
                    is_charging,
                });
            }
        }
        None
    }
}
