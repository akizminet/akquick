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
        let mut state = TelemetryState {
            cpu_percent: 14,
            ram_used_gib: 8.4,
            ram_total_gib: 31.1,
            hostname: "homepc".to_string(),
        };

        // Hostname
        if let Ok(h) = fs::read_to_string("/etc/hostname") {
            let h = h.trim();
            if !h.is_empty() {
                state.hostname = h.to_string();
            }
        }

        // RAM from /proc/meminfo
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

        // CPU from /proc/stat
        if let Ok(content) = fs::read_to_string("/proc/stat") {
            if let Some(cpu_line) = content.lines().next() {
                let parts: Vec<u64> = cpu_line
                    .split_whitespace()
                    .skip(1)
                    .filter_map(|s| s.parse().ok())
                    .collect();

                if parts.len() >= 4 {
                    let idle = parts[3];
                    let total: u64 = parts.iter().sum();
                    // Basic rough estimation or sample
                    let non_idle = total.saturating_sub(idle);
                    if total > 0 {
                        state.cpu_percent = ((non_idle as f64 / total as f64) * 100.0).round() as u32;
                    }
                }
            }
        }

        state
    }
}
