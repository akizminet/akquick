use std::fs;

#[derive(Debug, Clone, Default)]
pub struct TelemetryState {
    pub cpu_percent: u32,
    pub ram_used_gib: f32,
    pub ram_total_gib: f32,
    pub hostname: String,
}

pub struct TelemetryService;

impl TelemetryService {
    pub fn fetch() -> TelemetryState {
        let mut state = TelemetryState::default();

        // 1. Hostname from /proc/sys/kernel/hostname or /etc/hostname
        if let Ok(h) = fs::read_to_string("/proc/sys/kernel/hostname").or_else(|_| fs::read_to_string("/etc/hostname")) {
            let h = h.trim();
            if !h.is_empty() {
                state.hostname = h.to_string();
            }
        }

        // 2. RAM dynamically calculated from /proc/meminfo
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            let mut total_kb: f32 = 0.0;
            let mut avail_kb: f32 = 0.0;

            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(num) = line.split_whitespace().nth(1) {
                        total_kb = num.parse().unwrap_or(0.0);
                    }
                } else if line.starts_with("MemAvailable:") {
                    if let Some(num) = line.split_whitespace().nth(1) {
                        avail_kb = num.parse().unwrap_or(0.0);
                    }
                }
            }

            if total_kb > 0.0 {
                let total_gib = total_kb / (1024.0 * 1024.0);
                let used_gib = (total_kb - avail_kb) / (1024.0 * 1024.0);
                state.ram_total_gib = (total_gib * 10.0).round() / 10.0;
                state.ram_used_gib = (used_gib * 10.0).round() / 10.0;
            }
        }

        // 3. CPU calculated dynamically via differential sampling of /proc/stat
        state.cpu_percent = Self::calculate_cpu_usage();

        state
    }

    fn calculate_cpu_usage() -> u32 {
        static PREV_CPU: std::sync::Mutex<Option<(u64, u64)>> = std::sync::Mutex::new(None);

        if let Some((curr_idle, curr_total)) = Self::read_cpu_times() {
            let mut prev = PREV_CPU.lock().unwrap();
            if let Some((prev_idle, prev_total)) = *prev {
                let total_delta = curr_total.saturating_sub(prev_total);
                let idle_delta = curr_idle.saturating_sub(prev_idle);
                *prev = Some((curr_idle, curr_total));
                if total_delta > 0 {
                    let busy_delta = total_delta.saturating_sub(idle_delta);
                    return ((busy_delta as f64 / total_delta as f64) * 100.0).round() as u32;
                }
            } else {
                *prev = Some((curr_idle, curr_total));
                // Instantaneous fallback from loadavg
                if let Ok(load) = fs::read_to_string("/proc/loadavg") {
                    if let Some(first) = load.split_whitespace().next() {
                        if let Ok(val) = first.parse::<f64>() {
                            let ncpu = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8) as f64;
                            return ((val / ncpu) * 100.0).clamp(1.0, 100.0).round() as u32;
                        }
                    }
                }
                return 6;
            }
        }
        0
    }

    fn read_cpu_times() -> Option<(u64, u64)> {
        let content = fs::read_to_string("/proc/stat").ok()?;
        let first_line = content.lines().next()?;
        if !first_line.starts_with("cpu ") {
            return None;
        }

        let parts: Vec<u64> = first_line
            .split_whitespace()
            .skip(1)
            .filter_map(|s| s.parse().ok())
            .collect();

        if parts.len() >= 4 {
            let idle = parts[3] + parts.get(4).copied().unwrap_or(0); // idle + iowait
            let total: u64 = parts.iter().sum();
            return Some((idle, total));
        }

        None
    }
}
