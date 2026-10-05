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
    fn state(&self) -> zbus::Result<u32>;
}

#[derive(Debug, Clone)]
pub struct BatteryState {
    pub percentage: u32,
    pub time_remaining: String,
    pub is_charging: bool,
}

impl Default for BatteryState {
    fn default() -> Self {
        Self {
            percentage: 88,
            time_remaining: "5h 20m rem.".to_string(),
            is_charging: false,
        }
    }
}

pub struct BatteryService;

impl BatteryService {
    pub async fn fetch() -> BatteryState {
        let mut state = BatteryState::default();

        if let Ok(conn) = zbus::Connection::system().await {
            if let Ok(upower) = UPowerProxy::new(&conn).await {
                if let Ok(devices) = upower.enumerate_devices().await {
                    for dev_path in devices {
                        let path_str = dev_path.as_str();
                        if path_str.contains("battery") {
                            if let Ok(builder) = UPowerDeviceProxy::builder(&conn).path(dev_path) {
                                if let Ok(dev) = builder.build().await {
                                    if let Ok(pct) = dev.percentage().await {
                                        state.percentage = pct.round() as u32;
                                    }

                                    if let Ok(st) = dev.state().await {
                                        state.is_charging = st == 1; // 1 = Charging
                                    }

                                    if let Ok(secs) = dev.time_to_empty().await {
                                        if secs > 0 {
                                            let hours = secs / 3600;
                                            let mins = (secs % 3600) / 60;
                                            state.time_remaining = format!("{}h {}m rem.", hours, mins);
                                        }
                                    }
                                    return state;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sysfs fallback
        state.percentage = Self::fetch_sysfs_battery();
        state
    }

    fn fetch_sysfs_battery() -> u32 {
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
}
