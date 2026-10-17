use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::LazyLock;
use std::thread;

use crate::translators::pulse::PulseAudioClient;

enum AudioCommand {
    SetSinkVolume(u32, u64),
    SetSourceVolume(u32, u64),
    SetSinkMute(bool, u64),
    SetSourceMute(bool, u64),
    ToggleSinkMute(u64),
    ToggleSourceMute(u64),
}

static START_INSTANT: LazyLock<std::time::Instant> = LazyLock::new(std::time::Instant::now);

fn now_ms() -> u64 {
    START_INSTANT.elapsed().as_millis() as u64
}

static DISPATCHED_SINK_SEQ: AtomicU64 = AtomicU64::new(0);
static APPLIED_SINK_SEQ: AtomicU64 = AtomicU64::new(0);
static LAST_SINK_CMD_MS: AtomicU64 = AtomicU64::new(0);

static DISPATCHED_SOURCE_SEQ: AtomicU64 = AtomicU64::new(0);
static APPLIED_SOURCE_SEQ: AtomicU64 = AtomicU64::new(0);
static LAST_SOURCE_CMD_MS: AtomicU64 = AtomicU64::new(0);

fn next_sink_seq() -> u64 {
    LAST_SINK_CMD_MS.store(now_ms(), Ordering::Release);
    DISPATCHED_SINK_SEQ.fetch_add(1, Ordering::SeqCst) + 1
}

fn next_source_seq() -> u64 {
    LAST_SOURCE_CMD_MS.store(now_ms(), Ordering::Release);
    DISPATCHED_SOURCE_SEQ.fetch_add(1, Ordering::SeqCst) + 1
}

static AUDIO_CMD_TX: LazyLock<Sender<AudioCommand>> = LazyLock::new(|| {
    let (tx, rx) = channel::<AudioCommand>();
    thread::Builder::new()
        .name("audio-worker".into())
        .spawn(move || {
            let mut client_opt = PulseAudioClient::connect().ok();

            while let Ok(cmd) = rx.recv() {
                // Ensure client is connected
                if client_opt.is_none() {
                    client_opt = PulseAudioClient::connect().ok();
                }

                let mut latest_sink_vol = None;
                let mut latest_source_vol = None;
                let mut latest_sink_mute = None;
                let mut latest_source_mute = None;
                let mut pending_toggle_sink_mute = 0;
                let mut pending_toggle_source_mute = 0;
                let mut target_sink_seq = 0;
                let mut target_source_seq = 0;

                let mut apply_cmd = |c: AudioCommand| {
                    match c {
                        AudioCommand::SetSinkVolume(v, seq) => {
                            latest_sink_vol = Some(v);
                            target_sink_seq = target_sink_seq.max(seq);
                        }
                        AudioCommand::SetSourceVolume(v, seq) => {
                            latest_source_vol = Some(v);
                            target_source_seq = target_source_seq.max(seq);
                        }
                        AudioCommand::SetSinkMute(m, seq) => {
                            latest_sink_mute = Some(m);
                            pending_toggle_sink_mute = 0;
                            target_sink_seq = target_sink_seq.max(seq);
                        }
                        AudioCommand::SetSourceMute(m, seq) => {
                            latest_source_mute = Some(m);
                            pending_toggle_source_mute = 0;
                            target_source_seq = target_source_seq.max(seq);
                        }
                        AudioCommand::ToggleSinkMute(seq) => {
                            pending_toggle_sink_mute += 1;
                            target_sink_seq = target_sink_seq.max(seq);
                        }
                        AudioCommand::ToggleSourceMute(seq) => {
                            pending_toggle_source_mute += 1;
                            target_source_seq = target_source_seq.max(seq);
                        }
                    }
                };

                apply_cmd(cmd);

                // Drain coalescing queue to collapse rapid scroll wheel ticks
                while let Ok(next) = rx.try_recv() {
                    apply_cmd(next);
                }

                if let Some(client) = client_opt.as_mut() {
                    if let Some(vol) = latest_sink_vol {
                        let _ = client.set_sink_volume(vol);
                    }

                    if let Some(mute) = latest_sink_mute {
                        let _ = client.set_sink_mute(mute);
                    } else if pending_toggle_sink_mute % 2 != 0 {
                        if let Ok((_, _, _, muted)) = client.get_sink_info() {
                            let _ = client.set_sink_mute(!muted);
                        }
                    }

                    if let Some(vol) = latest_source_vol {
                        let _ = client.set_source_volume(vol);
                    }

                    if let Some(mute) = latest_source_mute {
                        let _ = client.set_source_mute(mute);
                    } else if pending_toggle_source_mute % 2 != 0 {
                        if let Ok((_, _, _, muted)) = client.get_source_info() {
                            let _ = client.set_source_mute(!muted);
                        }
                    }
                } else {
                    // Try reconnecting next time
                    client_opt = None;
                }

                if target_sink_seq > 0 {
                    APPLIED_SINK_SEQ.fetch_max(target_sink_seq, Ordering::SeqCst);
                }
                if target_source_seq > 0 {
                    APPLIED_SOURCE_SEQ.fetch_max(target_source_seq, Ordering::SeqCst);
                }
            }
        })
        .expect("Failed to spawn audio worker thread");
    tx
});

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
    pub sink_seq: u64,
    pub source_seq: u64,
}

pub struct AudioService;

impl AudioService {
    pub fn format_friendly_name(raw_name: &str, raw_desc: &str, is_source: bool) -> String {
        let mut desc = raw_desc.trim();
        if desc.is_empty() || desc == "PipeWire" {
            desc = raw_name.trim();
        }

        if is_source {
            if desc.contains("Built-in Audio") || desc.contains("Built-in") || raw_name.contains("analog-stereo") {
                return "Built-in Microphone".to_string();
            }
            if desc.ends_with(" Analog Stereo") {
                desc = desc.trim_end_matches(" Analog Stereo");
            }
            desc.to_string()
        } else {
            if desc == "Built-in Audio Analog Stereo" {
                return "Built-in Audio".to_string();
            }
            if desc.ends_with(" Analog Stereo") {
                desc = desc.trim_end_matches(" Analog Stereo");
            }
            desc.to_string()
        }
    }

    pub fn fetch() -> AudioState {
        let mut state = AudioState {
            sink_seq: APPLIED_SINK_SEQ.load(Ordering::Acquire),
            source_seq: APPLIED_SOURCE_SEQ.load(Ordering::Acquire),
            ..AudioState::default()
        };

        if let Ok(mut client) = PulseAudioClient::connect() {
            if let Ok((name, desc, vol, muted)) = client.get_sink_info() {
                state.sink_name = if !name.is_empty() { name.clone() } else { "Default Output".to_string() };
                state.sink_desc = Self::format_friendly_name(&name, &desc, false);
                state.volume_percent = vol;
                state.volume_muted = muted;
            }

            if let Ok((name, desc, vol, muted)) = client.get_source_info() {
                state.source_name = if !name.is_empty() { name.clone() } else { "Default Input".to_string() };
                state.source_desc = Self::format_friendly_name(&name, &desc, true);
                state.mic_percent = vol;
                state.mic_muted = muted;
                state.has_input_device = true;
            }
        }

        if state.sink_name.is_empty() {
            state.sink_name = "Default Output".to_string();
            state.sink_desc = "Audio Output".to_string();
        }

        if !state.has_input_device {
            state.source_name = "Disconnected".to_string();
            state.source_desc = "No Microphone Detected".to_string();
        }

        state
    }

    pub fn is_sink_poll_fresh(poll_seq: u64) -> bool {
        let dispatched = DISPATCHED_SINK_SEQ.load(Ordering::Acquire);
        if poll_seq < dispatched {
            return false;
        }
        let last_cmd = LAST_SINK_CMD_MS.load(Ordering::Acquire);
        if last_cmd > 0 && now_ms().saturating_sub(last_cmd) < 400 {
            return false;
        }
        true
    }

    pub fn is_source_poll_fresh(poll_seq: u64) -> bool {
        let dispatched = DISPATCHED_SOURCE_SEQ.load(Ordering::Acquire);
        if poll_seq < dispatched {
            return false;
        }
        let last_cmd = LAST_SOURCE_CMD_MS.load(Ordering::Acquire);
        if last_cmd > 0 && now_ms().saturating_sub(last_cmd) < 400 {
            return false;
        }
        true
    }

    pub fn set_volume(percent: u32) {
        let seq = next_sink_seq();
        let _ = AUDIO_CMD_TX.send(AudioCommand::SetSinkVolume(percent, seq));
    }

    pub fn set_volume_mute(muted: bool) {
        let seq = next_sink_seq();
        let _ = AUDIO_CMD_TX.send(AudioCommand::SetSinkMute(muted, seq));
    }

    #[allow(dead_code)]
    pub fn unmute_volume() {
        Self::set_volume_mute(false);
    }

    pub fn toggle_volume_mute() {
        let seq = next_sink_seq();
        let _ = AUDIO_CMD_TX.send(AudioCommand::ToggleSinkMute(seq));
    }

    pub fn set_mic_volume(percent: u32) {
        let seq = next_source_seq();
        let _ = AUDIO_CMD_TX.send(AudioCommand::SetSourceVolume(percent, seq));
    }

    pub fn set_mic_mute(muted: bool) {
        let seq = next_source_seq();
        let _ = AUDIO_CMD_TX.send(AudioCommand::SetSourceMute(muted, seq));
    }

    #[allow(dead_code)]
    pub fn unmute_mic() {
        Self::set_mic_mute(false);
    }

    pub fn toggle_mic_mute() {
        let seq = next_source_seq();
        let _ = AUDIO_CMD_TX.send(AudioCommand::ToggleSourceMute(seq));
    }
}
