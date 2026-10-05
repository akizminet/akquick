use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct AudioState {
    pub volume_percent: u32,
    pub volume_muted: bool,
    pub mic_percent: u32,
    pub mic_muted: bool,
    pub sink_name: String,
    pub source_name: String,
}

pub struct AudioService;

impl AudioService {
    pub fn fetch() -> AudioState {
        let mut state = AudioState {
            volume_percent: 72,
            volume_muted: false,
            mic_percent: 65,
            mic_muted: false,
            sink_name: "Analog 1/2".to_string(),
            source_name: "Gain Boost".to_string(),
        };

        // Output sink volume
        if let Ok(out) = Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SINK@"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            let mut parts = s.split_whitespace();
            if let Some(vol_str) = parts.nth(1) {
                if let Ok(vol_f) = vol_str.parse::<f32>() {
                    state.volume_percent = (vol_f * 100.0).round() as u32;
                }
            }
            state.volume_muted = s.contains("[MUTED]");
        }

        // Input mic volume
        if let Ok(out) = Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SOURCE@"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            let mut parts = s.split_whitespace();
            if let Some(vol_str) = parts.nth(1) {
                if let Ok(vol_f) = vol_str.parse::<f32>() {
                    state.mic_percent = (vol_f * 100.0).round() as u32;
                }
            }
            state.mic_muted = s.contains("[MUTED]");
        }

        state
    }

    pub fn set_volume(percent: u32) {
        let vol_arg = format!("{}%", percent.min(150));
        let _ = Command::new("wpctl")
            .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &vol_arg])
            .spawn();
    }

    pub fn toggle_volume_mute() {
        let _ = Command::new("wpctl")
            .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
            .spawn();
    }

    pub fn set_mic_volume(percent: u32) {
        let vol_arg = format!("{}%", percent.min(100));
        let _ = Command::new("wpctl")
            .args(["set-volume", "@DEFAULT_AUDIO_SOURCE@", &vol_arg])
            .spawn();
    }

    pub fn toggle_mic_mute() {
        let _ = Command::new("wpctl")
            .args(["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"])
            .spawn();
    }
}
