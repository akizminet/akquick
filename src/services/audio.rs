use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static TARGET_SINK_VOL: AtomicU32 = AtomicU32::new(u32::MAX);
static SINK_WORKER_ACTIVE: AtomicBool = AtomicBool::new(false);

static TARGET_SOURCE_VOL: AtomicU32 = AtomicU32::new(u32::MAX);
static SOURCE_WORKER_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Default, PartialEq)]
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
        let mut state = AudioState::default();

        // 1. Output Sink Volume & Mute
        let mut sink_ok = false;
        if let Ok(out) = Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SINK@"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            if !s.is_empty() && !s.contains("error") && !s.contains("Translate ID error") {
                let mut parts = s.split_whitespace();
                if let Some(vol_str) = parts.nth(1) {
                    if let Ok(vol_f) = vol_str.parse::<f32>() {
                        state.volume_percent = (vol_f * 100.0).round() as u32;
                        sink_ok = true;
                    }
                }
                state.volume_muted = s.contains("[MUTED]");
            }
        }
        if !sink_ok {
            if let Ok(out) = Command::new("pactl").args(["get-sink-volume", "@DEFAULT_SINK@"]).output() {
                let s = String::from_utf8_lossy(&out.stdout);
                if let Some(pct_str) = s.split('/').nth(1) {
                    if let Ok(pct) = pct_str.trim().trim_end_matches('%').parse::<u32>() {
                        state.volume_percent = pct;
                    }
                }
            }
            if let Ok(out) = Command::new("pactl").args(["get-sink-mute", "@DEFAULT_SINK@"]).output() {
                let s = String::from_utf8_lossy(&out.stdout);
                state.volume_muted = s.to_lowercase().contains("yes");
            }
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
        if state.sink_name.is_empty() || state.sink_desc.is_empty() {
            if let Ok(out) = Command::new("pactl").args(["get-default-sink"]).output() {
                let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !name.is_empty() {
                    state.sink_name = name.split('.').nth(1).unwrap_or(&name).to_string();
                    state.sink_desc = name;
                }
            }
        }
        if state.sink_name.is_empty() {
            state.sink_name = "Default Output".to_string();
        }
        if state.sink_desc.is_empty() {
            state.sink_desc = "Audio Sink".to_string();
        }

        // 3. Input Source Volume & Device
        if let Ok(out) = Command::new("wpctl").args(["get-volume", "@DEFAULT_AUDIO_SOURCE@"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            if !s.is_empty() && !s.contains("error") && !s.contains("Translate ID error") {
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

        if !state.has_input_device {
            // Fallback to pactl for source volume & mute
            if let Ok(out) = Command::new("pactl").args(["get-source-volume", "@DEFAULT_SOURCE@"]).output() {
                let s = String::from_utf8_lossy(&out.stdout);
                if let Some(pct_str) = s.split('/').nth(1) {
                    if let Ok(pct) = pct_str.trim().trim_end_matches('%').parse::<u32>() {
                        state.mic_percent = pct;
                        state.has_input_device = true;
                    }
                }
            }
            if let Ok(out) = Command::new("pactl").args(["get-source-mute", "@DEFAULT_SOURCE@"]).output() {
                let s = String::from_utf8_lossy(&out.stdout);
                if !s.is_empty() {
                    state.mic_muted = s.to_lowercase().contains("yes");
                    state.has_input_device = true;
                }
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
            if state.source_name.is_empty() || state.source_desc.is_empty() {
                // Fallback inspect description via pactl
                if let Ok(out) = Command::new("pactl").args(["list", "sources"]).output() {
                    let s = String::from_utf8_lossy(&out.stdout);
                    for chunk in s.split("Source #") {
                        if !chunk.contains("Monitor of") && chunk.contains("Description:") {
                            for line in chunk.lines() {
                                let line = line.trim();
                                if line.starts_with("Description: ") {
                                    state.source_desc = line.trim_start_matches("Description: ").to_string();
                                    state.source_name = state.source_desc.clone();
                                    break;
                                }
                            }
                            if !state.source_desc.is_empty() {
                                break;
                            }
                        }
                    }
                }
            }
        } else {
            state.source_name = "Disconnected".to_string();
            state.source_desc = "No Microphone Detected".to_string();
        }

        state
    }

    pub fn set_volume(percent: u32) {
        let pct = percent.min(150);
        TARGET_SINK_VOL.store(pct, Ordering::Release);
        if !SINK_WORKER_ACTIVE.swap(true, Ordering::AcqRel) {
            std::thread::spawn(|| {
                let mut current_applied = u32::MAX;
                loop {
                    let target = TARGET_SINK_VOL.load(Ordering::Acquire);
                    if target != current_applied && target != u32::MAX {
                        current_applied = target;
                        let vol_arg = format!("{}%", target);
                        let res = Command::new("wpctl")
                            .args(["set-volume", "-l", "1.5", "@DEFAULT_AUDIO_SINK@", &vol_arg])
                            .status();
                        if res.is_err() || !res.unwrap().success() {
                            let _ = Command::new("pactl")
                                .args(["set-sink-volume", "@DEFAULT_SINK@", &vol_arg])
                                .status();
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    if TARGET_SINK_VOL.load(Ordering::Acquire) == current_applied {
                        SINK_WORKER_ACTIVE.store(false, Ordering::Release);
                        if TARGET_SINK_VOL.load(Ordering::Acquire) != current_applied {
                            if !SINK_WORKER_ACTIVE.swap(true, Ordering::AcqRel) {
                                continue;
                            }
                        }
                        break;
                    }
                }
            });
        }
    }

    pub fn unmute_volume() {
        std::thread::spawn(|| {
            let res = Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "0"])
                .status();
            if res.is_err() || !res.unwrap().success() {
                let _ = Command::new("pactl")
                    .args(["set-sink-mute", "@DEFAULT_SINK@", "0"])
                    .status();
            }
        });
    }

    pub fn toggle_volume_mute() {
        std::thread::spawn(|| {
            let res = Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
                .status();
            if res.is_err() || !res.unwrap().success() {
                let _ = Command::new("pactl")
                    .args(["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
                    .status();
            }
        });
    }

    pub fn set_mic_volume(percent: u32) {
        let pct = percent.min(100);
        TARGET_SOURCE_VOL.store(pct, Ordering::Release);
        if !SOURCE_WORKER_ACTIVE.swap(true, Ordering::AcqRel) {
            std::thread::spawn(|| {
                let mut current_applied = u32::MAX;
                loop {
                    let target = TARGET_SOURCE_VOL.load(Ordering::Acquire);
                    if target != current_applied && target != u32::MAX {
                        current_applied = target;
                        let vol_arg = format!("{}%", target);
                        let res = Command::new("wpctl")
                            .args(["set-volume", "@DEFAULT_AUDIO_SOURCE@", &vol_arg])
                            .status();
                        if res.is_err() || !res.unwrap().success() {
                            let _ = Command::new("pactl")
                                .args(["set-source-volume", "@DEFAULT_SOURCE@", &vol_arg])
                                .status();
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    if TARGET_SOURCE_VOL.load(Ordering::Acquire) == current_applied {
                        SOURCE_WORKER_ACTIVE.store(false, Ordering::Release);
                        if TARGET_SOURCE_VOL.load(Ordering::Acquire) != current_applied {
                            if !SOURCE_WORKER_ACTIVE.swap(true, Ordering::AcqRel) {
                                continue;
                            }
                        }
                        break;
                    }
                }
            });
        }
    }

    pub fn unmute_mic() {
        std::thread::spawn(|| {
            let res = Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SOURCE@", "0"])
                .status();
            if res.is_err() || !res.unwrap().success() {
                let _ = Command::new("pactl")
                    .args(["set-source-mute", "@DEFAULT_SOURCE@", "0"])
                    .status();
            }
        });
    }

    pub fn toggle_mic_mute() {
        std::thread::spawn(|| {
            let res = Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SOURCE@", "toggle"])
                .status();
            if res.is_err() || !res.unwrap().success() {
                let _ = Command::new("pactl")
                    .args(["set-source-mute", "@DEFAULT_SOURCE@", "toggle"])
                    .status();
            }
        });
    }
}
