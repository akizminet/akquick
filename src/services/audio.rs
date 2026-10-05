use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct AudioState {
    pub volume_percent: u32,
    pub volume_muted: bool,
    pub sink_name: String,
    pub sink_desc: String,
    pub has_input_device: bool,
    pub mic_percent: u32,
    pub mic_muted: bool,
    pub source_name: String,
    pub source_desc: String,
}

pub struct AudioService;

impl AudioService {
    pub fn fetch() -> AudioState {
        let mut state = AudioState {
            volume_percent: 50,
            volume_muted: false,
            sink_name: "ALC897 Analog".to_string(),
            sink_desc: "Built-in Audio".to_string(),
            has_input_device: false,
            mic_percent: 0,
            mic_muted: true,
            source_name: "No Input Device".to_string(),
            source_desc: "Disconnected".to_string(),
        };

        // 1. Output Sink Volume & Mute
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

        // 2. Output Sink Device Name & Description
        if let Ok(out) = Command::new("wpctl").args(["inspect", "@DEFAULT_AUDIO_SINK@"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            for line in s.lines() {
                let line = line.trim();
                if line.contains("alsa.mixer_name = ") {
                    if let Some(val) = line.split('"').nth(1) {
                        state.sink_name = val.to_string();
                    }
                } else if line.contains("node.description = ") {
                    if let Some(val) = line.split('"').nth(1) {
                        state.sink_desc = val.to_string();
                    }
                }
            }
        }

        // 3. Input Source Volume & Device
        if let Ok(out) = Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SOURCE@"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            if !s.contains("error") && !s.contains("Translate ID error") {
                let mut parts = s.split_whitespace();
                if let Some(vol_str) = parts.nth(1) {
                    if let Ok(vol_f) = vol_str.parse::<f32>() {
                        state.mic_percent = (vol_f * 100.0).round() as u32;
                        state.has_input_device = true;
                    }
                }
                state.mic_muted = s.contains("[MUTED]");
            }
        }

        if state.has_input_device {
            if let Ok(out) = Command::new("wpctl").args(["inspect", "@DEFAULT_AUDIO_SOURCE@"]).output() {
                let s = String::from_utf8_lossy(&out.stdout);
                for line in s.lines() {
                    let line = line.trim();
                    if line.contains("alsa.mixer_name = ") {
                        if let Some(val) = line.split('"').nth(1) {
                            state.source_name = val.to_string();
                        }
                    } else if line.contains("node.description = ") {
                        if let Some(val) = line.split('"').nth(1) {
                            state.source_desc = val.to_string();
                        }
                    }
                }
            }
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
