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
    fn position(&self) -> zbus::Result<i64>;

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
    pub position_sec: u64,
    pub length_sec: u64,
    pub active_bus_name: Option<String>,
}

pub struct MprisService;

impl MprisService {
    pub async fn fetch() -> MprisState {
        let conn = match zbus::Connection::session().await {
            Ok(c) => c,
            Err(_) => return MprisState::default(),
        };

        let dbus_proxy = match zbus::fdo::DBusProxy::new(&conn).await {
            Ok(p) => p,
            Err(_) => return MprisState::default(),
        };

        let names = match dbus_proxy.list_names().await {
            Ok(n) => n,
            Err(_) => return MprisState::default(),
        };

        let mut candidates = Vec::new();

        for bus_name in names {
            if !bus_name.starts_with("org.mpris.MediaPlayer2.") {
                continue;
            }

            if let Ok(builder) = MediaPlayerControlProxy::builder(&conn).destination(bus_name.as_str()) {
                if let Ok(player) = builder.build().await {
                    let status = player.playback_status().await.unwrap_or_else(|_| "Stopped".into());
                    let is_playing = status == "Playing";
                    let is_paused = status == "Paused";

                    let mut title = String::new();
                    let mut artist = String::new();
                    let mut length_sec = 0;

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
                        if let Some(l_val) = meta.get("mpris:length") {
                            if let Ok(us) = l_val.to_string().trim().parse::<i64>() {
                                if us > 0 {
                                    length_sec = (us / 1_000_000) as u64;
                                }
                            }
                        }
                    }

                    if !title.is_empty() || is_playing || is_paused {
                        let position_sec = player.position().await
                            .map(|us| (us / 1_000_000).max(0) as u64)
                            .unwrap_or(0);

                        let mut player_name = "MPRIS".to_string();
                        if let Ok(id_builder) = MediaPlayerProxy::builder(&conn).destination(bus_name.as_str()) {
                            if let Ok(id_proxy) = id_builder.build().await {
                                if let Ok(id) = id_proxy.identity().await {
                                    player_name = format!("{} MPRIS", id.to_uppercase());
                                }
                            }
                        }

                        // Score 2: Playing, Score 1: Paused with title, Score 0: Other
                        let score = if is_playing {
                            2
                        } else if is_paused && !title.is_empty() {
                            1
                        } else {
                            0
                        };

                        candidates.push((
                            score,
                            MprisState {
                                has_player: true,
                                title: if title.is_empty() { "Unknown Title".into() } else { title },
                                artist: if artist.is_empty() { "Unknown Artist".into() } else { artist },
                                player_name,
                                is_playing,
                                position_sec,
                                length_sec,
                                active_bus_name: Some(bus_name.to_string()),
                            }
                        ));
                    }
                }
            }
        }

        candidates.sort_by(|a, b| b.0.cmp(&a.0));
        if let Some((_, best)) = candidates.into_iter().next() {
            return best;
        }

        MprisState {
            has_player: false,
            title: "No Media Playing".to_string(),
            artist: "Audio Idle".to_string(),
            player_name: "MPRIS".to_string(),
            is_playing: false,
            position_sec: 0,
            length_sec: 0,
            active_bus_name: None,
        }
    }

    pub fn play_pause(bus_name: Option<String>) {
        MPRIS_RT.spawn(async move {
            if let Ok(conn) = zbus::Connection::session().await {
                let target = match bus_name {
                    Some(b) => Some(b),
                    None => Self::get_active_or_first_player(&conn).await,
                };
                if let Some(bus) = target {
                    if let Ok(builder) = MediaPlayerControlProxy::builder(&conn).destination(bus.as_str()) {
                        if let Ok(player) = builder.build().await {
                            let _ = player.play_pause().await;
                        }
                    }
                }
            }
        });
    }

    pub fn next(bus_name: Option<String>) {
        MPRIS_RT.spawn(async move {
            if let Ok(conn) = zbus::Connection::session().await {
                let target = match bus_name {
                    Some(b) => Some(b),
                    None => Self::get_active_or_first_player(&conn).await,
                };
                if let Some(bus) = target {
                    if let Ok(builder) = MediaPlayerControlProxy::builder(&conn).destination(bus.as_str()) {
                        if let Ok(player) = builder.build().await {
                            let _ = player.next().await;
                        }
                    }
                }
            }
        });
    }

    pub fn previous(bus_name: Option<String>) {
        MPRIS_RT.spawn(async move {
            if let Ok(conn) = zbus::Connection::session().await {
                let target = match bus_name {
                    Some(b) => Some(b),
                    None => Self::get_active_or_first_player(&conn).await,
                };
                if let Some(bus) = target {
                    if let Ok(builder) = MediaPlayerControlProxy::builder(&conn).destination(bus.as_str()) {
                        if let Ok(player) = builder.build().await {
                            let _ = player.previous().await;
                        }
                    }
                }
            }
        });
    }

    async fn get_active_or_first_player(conn: &zbus::Connection) -> Option<String> {
        let dbus_proxy = zbus::fdo::DBusProxy::new(conn).await.ok()?;
        let names = dbus_proxy.list_names().await.ok()?;
        let mpris_names: Vec<String> = names
            .into_iter()
            .filter(|n| n.starts_with("org.mpris.MediaPlayer2."))
            .map(|n| n.to_string())
            .collect();

        for name in &mpris_names {
            if let Ok(builder) = MediaPlayerControlProxy::builder(conn).destination(name.as_str()) {
                if let Ok(player) = builder.build().await {
                    if let Ok(status) = player.playback_status().await {
                        if status == "Playing" {
                            return Some(name.clone());
                        }
                    }
                }
            }
        }

        mpris_names.into_iter().next()
    }
}

pub static MPRIS_RT: std::sync::LazyLock<tokio::runtime::Runtime> = std::sync::LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .expect("Failed to create background MPRIS runtime")
});
