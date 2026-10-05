use std::process::Command;

pub struct BacklightService;

impl BacklightService {
    pub fn fetch() -> u32 {
        if let Ok(out) = Command::new("brightnessctl").args(["get"]).output() {
            let cur = String::from_utf8_lossy(&out.stdout).trim().parse::<f32>().unwrap_or(85.0);
            if let Ok(max_out) = Command::new("brightnessctl").args(["max"]).output() {
                let max = String::from_utf8_lossy(&max_out.stdout).trim().parse::<f32>().unwrap_or(100.0);
                return ((cur / max) * 100.0).round() as u32;
            }
        }
        85
    }

    pub fn set(percent: u32) {
        let arg = format!("{}%", percent.min(100));
        let _ = Command::new("brightnessctl").args(["set", &arg]).spawn();
    }
}
