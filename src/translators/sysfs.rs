use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Read backlight brightness as a percentage (0..100) directly from Linux sysfs
#[allow(dead_code)]
pub fn read_backlight_percent() -> Option<u32> {
    let base = Path::new("/sys/class/backlight");
    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let p = entry.path();
            let cur_p = p.join("brightness");
            let max_p = p.join("max_brightness");

            if let (Ok(cur_s), Ok(max_s)) = (fs::read_to_string(cur_p), fs::read_to_string(max_p)) {
                if let (Ok(cur), Ok(max)) = (cur_s.trim().parse::<f64>(), max_s.trim().parse::<f64>()) {
                    if max > 0.0 {
                        return Some(((cur / max) * 100.0).round() as u32);
                    }
                }
            }
        }
    }
    None
}

#[zbus::proxy(
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1/session/auto",
    interface = "org.freedesktop.login1.Session"
)]
trait LogindSession {
    fn set_brightness(&self, subsystem: &str, name: &str, brightness: u32) -> zbus::Result<()>;
}

/// Set backlight brightness directly via sysfs or systemd-logind D-Bus (zero external tools)
#[allow(dead_code)]
pub fn set_backlight_percent(percent: u32) {
    let base = Path::new("/sys/class/backlight");
    let mut written = false;

    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let p = entry.path();
            let cur_p = p.join("brightness");
            let max_p = p.join("max_brightness");

            if let Ok(max_s) = fs::read_to_string(&max_p) {
                if let Ok(max) = max_s.trim().parse::<f64>() {
                    let target = ((percent.min(100) as f64 / 100.0) * max).round() as u64;
                    if fs::write(&cur_p, target.to_string()).is_ok() {
                        written = true;
                    }
                }
            }
        }
    }

    if !written {
        let pct = percent.min(100);
        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let max_p = entry.path().join("max_brightness");
                if let Ok(max_s) = fs::read_to_string(max_p) {
                    if let Ok(max) = max_s.trim().parse::<f64>() {
                        let target = ((pct as f64 / 100.0) * max).round() as u32;
                        let dev_name = name.clone();
                        crate::services::mpris::MPRIS_RT.spawn(async move {
                            if let Ok(conn) = zbus::Connection::system().await {
                                if let Ok(session) = LogindSessionProxy::new(&conn).await {
                                    let _ = session.set_brightness("backlight", &dev_name, target).await;
                                }
                            }
                        });
                        break;
                    }
                }
            }
        }
    }
}

/// Read battery percentage and charging state directly from Linux sysfs
#[allow(dead_code)]
pub fn read_battery() -> Option<(u32, bool)> {
    let base = Path::new("/sys/class/power_supply");
    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("BAT") {
                let cap_p = entry.path().join("capacity");
                let stat_p = entry.path().join("status");

                let cap = fs::read_to_string(cap_p)
                    .ok()
                    .and_then(|s| s.trim().parse::<u32>().ok());
                let charging = fs::read_to_string(stat_p)
                    .map(|s| s.trim().eq_ignore_ascii_case("Charging"))
                    .unwrap_or(false);

                if let Some(c) = cap {
                    return Some((c, charging));
                }
            }
        }
    }
    None
}

/// Pure parser: Read network rx and tx bytes from /proc/net/dev content
pub fn parse_proc_net_dev(content: &str) -> (u64, u64) {
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;

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
    (total_rx, total_tx)
}

/// Format speed in bytes/sec to human readable string
pub fn format_speed(bytes_per_sec: f64) -> String {
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

/// Read throughput delta using /proc/net/dev and /dev/shm cache
pub fn get_network_speed() -> (String, String) {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let content = fs::read_to_string("/proc/net/dev").unwrap_or_default();
    let (curr_rx, curr_tx) = parse_proc_net_dev(&content);
    let stat_file = "/dev/shm/akquick_net_stat";

    let mut down_speed = 0.0;
    let mut up_speed = 0.0;

    if let Ok(stat_content) = fs::read_to_string(stat_file) {
        let parts: Vec<&str> = stat_content.trim().split_whitespace().collect();
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

    let _ = fs::write(stat_file, format!("{} {} {}", now_ms, curr_rx, curr_tx));

    (format_speed(down_speed), format_speed(up_speed))
}
