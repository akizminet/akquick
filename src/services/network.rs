use zbus::proxy;

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager",
    interface = "org.freedesktop.NetworkManager"
)]
trait NetworkManager {
    #[zbus(property)]
    fn wireless_enabled(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn active_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    interface = "org.freedesktop.NetworkManager.Connection.Active"
)]
trait ActiveConnection {
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;

    #[zbus(property, name = "Type")]
    fn connection_type(&self) -> zbus::Result<String>;
}

#[derive(Debug, Clone, Default)]
pub struct NetworkState {
    pub wifi_enabled: bool,
    pub wifi_ssid: String,
    pub vpn_active: bool,
    pub vpn_name: String,
}

pub struct NetworkService;

impl NetworkService {
    pub async fn fetch() -> NetworkState {
        let mut state = NetworkState {
            wifi_enabled: false,
            wifi_ssid: "Disconnected".to_string(),
            vpn_active: false,
            vpn_name: "Disconnected".to_string(),
        };

        let conn = match zbus::Connection::system().await {
            Ok(c) => c,
            Err(_) => return state,
        };

        let nm = match NetworkManagerProxy::new(&conn).await {
            Ok(p) => p,
            Err(_) => return state,
        };

        if let Ok(enabled) = nm.wireless_enabled().await {
            state.wifi_enabled = enabled;
        }

        if let Ok(active_paths) = nm.active_connections().await {
            for path in active_paths {
                if let Ok(builder) = ActiveConnectionProxy::builder(&conn).path(path) {
                    if let Ok(ac) = builder.build().await {
                        let conn_type = ac.connection_type().await.unwrap_or_default();
                        let id = ac.id().await.unwrap_or_default();

                        if conn_type == "802-11-wireless" {
                            state.wifi_ssid = id;
                            state.wifi_enabled = true;
                        } else if conn_type == "vpn" || conn_type == "wireguard" || conn_type == "tun" {
                            state.vpn_active = true;
                            state.vpn_name = id;
                        }
                    }
                }
            }
        }

        state
    }
}
