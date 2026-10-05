use zbus::proxy;
use std::collections::HashMap;
use zbus::zvariant::OwnedValue;

#[proxy(
    default_path = "/org/mpris/MediaPlayer2",
    interface = "org.mpris.MediaPlayer2"
)]
trait MediaPlayer {
    #[zbus(property)]
    fn identity(&self) -> zbus::Result<String>;
}

#[proxy(
    default_path = "/org/mpris/MediaPlayer2",
    interface = "org.mpris.MediaPlayer2.Player"
)]
trait MediaPlayerControl {
    fn play_pause(&self) -> zbus::Result<()>;
    fn next(&self) -> zbus::Result<()>;
    fn previous(&self) -> zbus::Result<()>;

    #[zbus(property)]
    fn playback_status(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn metadata(&self) -> zbus::Result<HashMap<String, OwnedValue>>;
}

#[derive(Debug, Clone, Default)]
pub struct MprisState {
    pub has_player: bool,
    pub title: String,
    pub artist: String,
    pub player_name: String,
    pub is_playing: bool,
    pub active_bus_name: Option<String>,
}

pub struct MprisService;

impl MprisService {
    pub async fn fetch() -> MprisState {
        let mut state = MprisState {
            has_player: false,
            title: "No Media Playing".to_string(),
            artist: "Audio Idle".to_string(),
            player_name: "MPRIS".to_string(),
            is_playing: false,
            active_bus_name: None,
        };

        let conn = match zbus::Connection::session().await {
            Ok(c) => c,
            Err(_) => return state,
        };

        let dbus_proxy = match zbus::fdo::DBusProxy::new(&conn).await {
            Ok(p) => p,
            Err(_) => return state,
        };

        if let Ok(names) = dbus_proxy.list_names().await {
            // Prefer actual music players first (spotify, vlc, mpv, etc.) over browser tabs
            let mut sorted_names: Vec<String> = names
                .into_iter()
                .filter(|n| n.starts_with("org.mpris.MediaPlayer2."))
                .map(|n| n.to_string())
                .collect();

            sorted_names.sort_by_key(|n| {
                if n.contains("spotify") || n.contains("mpv") || n.contains("vlc") || n.contains("celluloid") {
                    0
                } else {
                    1
                }
            });

            for bus_name in sorted_names {
                if let Ok(builder) = MediaPlayerControlProxy::builder(&conn).destination(bus_name.as_str()) {
                    if let Ok(player) = builder.build().await {
                        let status = player.playback_status().await.unwrap_or_else(|_| "Stopped".into());
                        let is_playing = status == "Playing";

                        let mut title = String::new();
                        let mut artist = String::new();

                        if let Ok(meta) = player.metadata().await {
                            if let Some(t_val) = meta.get("xesam:title") {
                                let s = t_val.to_string();
                                let cleaned = s.trim_matches('"').trim();
                                if !cleaned.is_empty() {
                                    title = cleaned.to_string();
                                }
                            }
                            if let Some(a_val) = meta.get("xesam:artist") {
                                let s = a_val.to_string();
                                let cleaned = s
                                    .trim_start_matches('[')
                                    .trim_end_matches(']')
                                    .trim_matches('"')
                                    .trim();
                                if !cleaned.is_empty() {
                                    artist = cleaned.to_string();
                                }
                            }
                        }

                        // Only consider it an active player if it is playing or has track title
                        if !title.is_empty() || is_playing {
                            state.has_player = true;
                            state.title = if title.is_empty() { "Unknown Title".into() } else { title };
                            state.artist = if artist.is_empty() { "Unknown Artist".into() } else { artist };
                            state.is_playing = is_playing;

                            if let Ok(id_builder) = MediaPlayerProxy::builder(&conn).destination(bus_name.as_str()) {
                                if let Ok(id_proxy) = id_builder.build().await {
                                    if let Ok(id) = id_proxy.identity().await {
                                        state.player_name = format!("{} MPRIS", id.to_uppercase());
                                    }
                                }
                            }

                            state.active_bus_name = Some(bus_name);
                            break;
                        }
                    }
                }
            }
        }

        state
    }

    pub fn play_pause() {
        tokio::spawn(async {
            if let Ok(conn) = zbus::Connection::session().await {
                if let Some(bus) = Self::get_first_player_name(&conn).await {
                    if let Ok(builder) = MediaPlayerControlProxy::builder(&conn).destination(bus.as_str()) {
                        if let Ok(player) = builder.build().await {
                            let _ = player.play_pause().await;
                        }
                    }
                }
            }
        });
    }

    pub fn next() {
        tokio::spawn(async {
            if let Ok(conn) = zbus::Connection::session().await {
                if let Some(bus) = Self::get_first_player_name(&conn).await {
                    if let Ok(builder) = MediaPlayerControlProxy::builder(&conn).destination(bus.as_str()) {
                        if let Ok(player) = builder.build().await {
                            let _ = player.next().await;
                        }
                    }
                }
            }
        });
    }

    pub fn previous() {
        tokio::spawn(async {
            if let Ok(conn) = zbus::Connection::session().await {
                if let Some(bus) = Self::get_first_player_name(&conn).await {
                    if let Ok(builder) = MediaPlayerControlProxy::builder(&conn).destination(bus.as_str()) {
                        if let Ok(player) = builder.build().await {
                            let _ = player.previous().await;
                        }
                    }
                }
            }
        });
    }

    async fn get_first_player_name(conn: &zbus::Connection) -> Option<String> {
        let dbus_proxy = zbus::fdo::DBusProxy::new(conn).await.ok()?;
        let names = dbus_proxy.list_names().await.ok()?;
        names
            .into_iter()
            .find(|n| n.starts_with("org.mpris.MediaPlayer2."))
            .map(|n| n.to_string())
    }
}
