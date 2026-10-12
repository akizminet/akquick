use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::sync::{Arc, LazyLock, RwLock};
use std::thread;
use crate::ipc::protocol::{socket_path, IpcCommand};
use crate::status::FullStatus;

pub static LATEST_STATUS: LazyLock<Arc<RwLock<Option<FullStatus>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

pub fn update_cached_status(status: FullStatus) {
    if let Ok(mut lock) = LATEST_STATUS.write() {
        *lock = Some(status);
    }
}

pub struct IpcServer;

impl IpcServer {
    pub fn start<F>(action_handler: F) -> std::io::Result<()>
    where
        F: Fn(IpcCommand) + Send + Sync + 'static,
    {
        let path = socket_path();
        let _ = std::fs::remove_file(&path);

        let listener = UnixListener::bind(&path)?;

        let handler = Arc::new(action_handler);

        thread::Builder::new()
            .name("akquick-ipc".into())
            .spawn(move || {
                for stream in listener.incoming() {
                    match stream {
                        Ok(mut s) => {
                            let mut buf = String::new();
                            if s.read_to_string(&mut buf).is_ok() {
                                let cmd = IpcCommand::parse(&buf);
                                let response = match &cmd {
                                    IpcCommand::StatusFull => {
                                        let lock = LATEST_STATUS.read().ok();
                                        lock.as_ref()
                                            .and_then(|g| g.as_ref())
                                            .map(|st| st.format_full())
                                            .unwrap_or_else(|| "akquick starting...".to_string())
                                    }
                                    IpcCommand::StatusIcons => {
                                        let lock = LATEST_STATUS.read().ok();
                                        lock.as_ref()
                                            .and_then(|g| g.as_ref())
                                            .map(|st| st.format_icons_only())
                                            .unwrap_or_else(|| "󰤨  󰕾  󰍬".to_string())
                                    }
                                    IpcCommand::StatusJson => {
                                        let lock = LATEST_STATUS.read().ok();
                                        if let Some(Some(st)) = lock.as_deref() {
                                            format!(
                                                r#"{{"wifi_icon":"{}","wifi_name":"{}","down_speed":"{}","up_speed":"{}","audio_icon":"{}","audio_volume":{},"audio_muted":{},"mic_icon":"{}","mic_volume":{},"mic_muted":{}}}"#,
                                                st.wifi_icon,
                                                st.wifi_name,
                                                st.down_speed,
                                                st.up_speed,
                                                st.audio_icon,
                                                st.audio_vol,
                                                st.audio_muted,
                                                st.mic_icon,
                                                st.mic_vol,
                                                st.mic_muted
                                            )
                                        } else {
                                            "{}".to_string()
                                        }
                                    }
                                    IpcCommand::StatusWifi => {
                                        let lock = LATEST_STATUS.read().ok();
                                        if let Some(Some(st)) = lock.as_deref() {
                                            format!("{} {}", st.wifi_icon, st.wifi_name)
                                        } else {
                                            "󰤨 Wi-Fi".to_string()
                                        }
                                    }
                                    IpcCommand::StatusNet => {
                                        let lock = LATEST_STATUS.read().ok();
                                        if let Some(Some(st)) = lock.as_deref() {
                                            format!("⬇{} ⬆{}", st.down_speed, st.up_speed)
                                        } else {
                                            "⬇0K ⬆0K".to_string()
                                        }
                                    }
                                    IpcCommand::StatusAudio => {
                                        let lock = LATEST_STATUS.read().ok();
                                        if let Some(Some(st)) = lock.as_deref() {
                                            if st.audio_muted {
                                                format!("󰝟 {}%", st.audio_vol)
                                            } else {
                                                format!("{} {}%", st.audio_icon, st.audio_vol)
                                            }
                                        } else {
                                            "󰕾 50%".to_string()
                                        }
                                    }
                                    IpcCommand::StatusMic => {
                                        let lock = LATEST_STATUS.read().ok();
                                        if let Some(Some(st)) = lock.as_deref() {
                                            if !st.has_mic {
                                                "󰍭 0%".to_string()
                                            } else if st.mic_muted {
                                                format!("󰍭 {}%", st.mic_vol)
                                            } else {
                                                format!("{} {}%", st.mic_icon, st.mic_vol)
                                            }
                                        } else {
                                            "󰍬 100%".to_string()
                                        }
                                    }
                                    IpcCommand::Toggle
                                    | IpcCommand::OpenPage(_)
                                    | IpcCommand::Volume(_)
                                    | IpcCommand::ToggleVolumeMute
                                    | IpcCommand::ToggleMicMute => {
                                        handler(cmd);
                                        "ok".to_string()
                                    }
                                    IpcCommand::Unknown(u) => {
                                        format!("error: unknown command '{}'", u)
                                    }
                                };
                                let _ = s.write_all(response.as_bytes());
                            }
                        }
                        Err(_) => break,
                    }
                }
            })?;

        Ok(())
    }
}
