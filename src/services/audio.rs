use std::sync::mpsc::{channel, Sender};
use std::sync::LazyLock;
use std::thread;

use crate::translators::pulse::PulseAudioClient;

enum AudioCommand {
    SetSinkVolume(u32),
    SetSourceVolume(u32),
    ToggleSinkMute,
    ToggleSourceMute,
    UnmuteSink,
    UnmuteSource,
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
                let mut pending_toggle_sink_mute = 0;
                let mut pending_toggle_source_mute = 0;
                let mut unmute_sink = false;
                let mut unmute_source = false;

                let apply_cmd = |c: AudioCommand,
                                 l_sink: &mut Option<u32>,
                                 l_src: &mut Option<u32>,
                                 t_sink: &mut usize,
                                 t_src: &mut usize,
                                 u_sink: &mut bool,
                                 u_src: &mut bool| {
                    match c {
                        AudioCommand::SetSinkVolume(v) => *l_sink = Some(v),
                        AudioCommand::SetSourceVolume(v) => *l_src = Some(v),
                        AudioCommand::ToggleSinkMute => *t_sink += 1,
                        AudioCommand::ToggleSourceMute => *t_src += 1,
                        AudioCommand::UnmuteSink => *u_sink = true,
                        AudioCommand::UnmuteSource => *u_src = true,
                    }
                };

                apply_cmd(
                    cmd,
                    &mut latest_sink_vol,
                    &mut latest_source_vol,
                    &mut pending_toggle_sink_mute,
                    &mut pending_toggle_source_mute,
                    &mut unmute_sink,
                    &mut unmute_source,
                );

                // Drain coalescing queue to collapse rapid scroll wheel ticks
                while let Ok(next) = rx.try_recv() {
                    apply_cmd(
                        next,
                        &mut latest_sink_vol,
                        &mut latest_source_vol,
                        &mut pending_toggle_sink_mute,
                        &mut pending_toggle_source_mute,
                        &mut unmute_sink,
                        &mut unmute_source,
                    );
                }

                if let Some(client) = client_opt.as_mut() {
                    if let Some(vol) = latest_sink_vol {
                        let _ = client.set_sink_volume(vol);
                    }

                    if unmute_sink {
                        let _ = client.set_sink_mute(false);
                    } else if pending_toggle_sink_mute % 2 != 0 {
                        if let Ok((_, _, _, muted)) = client.get_sink_info() {
                            let _ = client.set_sink_mute(!muted);
                        }
                    }

                    if let Some(vol) = latest_source_vol {
                        let _ = client.set_source_volume(vol);
                    }

                    if unmute_source {
                        let _ = client.set_source_mute(false);
                    } else if pending_toggle_source_mute % 2 != 0 {
                        if let Ok((_, _, _, muted)) = client.get_source_info() {
                            let _ = client.set_source_mute(!muted);
                        }
                    }
                } else {
                    // Try reconnecting next time
                    client_opt = None;
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
        let mut state = AudioState::default();

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

    pub fn set_volume(percent: u32) {
        let _ = AUDIO_CMD_TX.send(AudioCommand::SetSinkVolume(percent));
    }

    pub fn unmute_volume() {
        let _ = AUDIO_CMD_TX.send(AudioCommand::UnmuteSink);
    }

    pub fn toggle_volume_mute() {
        let _ = AUDIO_CMD_TX.send(AudioCommand::ToggleSinkMute);
    }

    pub fn set_mic_volume(percent: u32) {
        let _ = AUDIO_CMD_TX.send(AudioCommand::SetSourceVolume(percent));
    }

    pub fn unmute_mic() {
        let _ = AUDIO_CMD_TX.send(AudioCommand::UnmuteSource);
    }

    pub fn toggle_mic_mute() {
        let _ = AUDIO_CMD_TX.send(AudioCommand::ToggleSourceMute);
    }
}
