use crate::translators::network_manager;

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct WifiAccessPoint {
    pub ssid: String,
    pub signal: u32,
    pub security: String,
    pub in_use: bool,
}

#[derive(Debug, Clone, Default)]
pub struct EthernetDetails {
    pub connected: bool,
    pub name: String,
    pub interface: String,
    pub ip: String,
    pub gateway: String,
    pub speed_mbps: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct VpnDetails {
    pub active: bool,
    pub name: String,
    pub interface: String,
    pub ip: String,
    pub rx_bytes_str: String,
    pub tx_bytes_str: String,
    pub latency_ms: String,
    pub cipher: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct VpnProfile {
    pub name: String,
    pub vpn_type: String,
    pub interface: String,
    pub active: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkState {
    pub primary_is_ethernet: bool,
    pub ethernet: EthernetDetails,
    pub wifi_enabled: bool,
    pub wifi_ssid: String,
    pub wifi_ip: String,
    pub wifi_gateway: String,
    pub vpn_active: bool,
    pub vpn_name: String,
    pub vpn_details: VpnDetails,
    pub vpn_profiles: Vec<VpnProfile>,
    pub scanned_aps: Vec<WifiAccessPoint>,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkStatus {
    pub primary_is_ethernet: bool,
    pub ethernet_connected: bool,
    pub ethernet_name: String,
    pub wifi_enabled: bool,
    pub wifi_ssid: String,
    pub vpn_active: bool,
    pub vpn_name: String,
}

impl From<&NetworkState> for NetworkStatus {
    fn from(s: &NetworkState) -> Self {
        Self {
            primary_is_ethernet: s.primary_is_ethernet,
            ethernet_connected: s.ethernet.connected,
            ethernet_name: s.ethernet.name.clone(),
            wifi_enabled: s.wifi_enabled,
            wifi_ssid: s.wifi_ssid.clone(),
            vpn_active: s.vpn_active,
            vpn_name: s.vpn_name.clone(),
        }
    }
}

pub struct NetworkService;

impl NetworkService {
    pub async fn fetch_status() -> NetworkStatus {
        let (wifi_enabled, active) = network_manager::query_active_connections().await;
        let mut status = NetworkStatus {
            primary_is_ethernet: false,
            ethernet_connected: false,
            ethernet_name: String::new(),
            wifi_enabled,
            wifi_ssid: "Disconnected".to_string(),
            vpn_active: false,
            vpn_name: "Disconnected".to_string(),
        };

        for ac in active {
            if ac.conn_type == "802-3-ethernet" || ac.conn_type == "ethernet" {
                status.ethernet_connected = true;
                status.ethernet_name = ac.id;
                if ac.is_default || !status.primary_is_ethernet {
                    status.primary_is_ethernet = true;
                }
            } else if ac.conn_type == "802-11-wireless" {
                status.wifi_ssid = ac.id;
                status.wifi_enabled = true;
                if ac.is_default {
                    status.primary_is_ethernet = false;
                }
            } else if ac.conn_type == "vpn" || ac.conn_type == "wireguard" {
                status.vpn_active = true;
                status.vpn_name = ac.id;
            }
        }

        status
    }

    pub async fn fetch() -> NetworkState {
        Self::fetch_internal(false).await
    }

    pub async fn fetch_with_cached_aps() -> NetworkState {
        Self::fetch_internal(true).await
    }

    pub async fn fetch_with_scan() -> NetworkState {
        network_manager::trigger_wifi_scan().await;
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
        Self::fetch_internal(true).await
    }

    async fn fetch_internal(scan: bool) -> NetworkState {
        let mut state = NetworkState {
            primary_is_ethernet: false,
            ethernet: EthernetDetails::default(),
            wifi_enabled: false,
            wifi_ssid: "Disconnected".to_string(),
            wifi_ip: String::new(),
            wifi_gateway: String::new(),
            vpn_active: false,
            vpn_name: "Disconnected".to_string(),
            vpn_details: VpnDetails::default(),
            vpn_profiles: Vec::new(),
            scanned_aps: Vec::new(),
        };

        let status = Self::fetch_status().await;
        state.primary_is_ethernet = status.primary_is_ethernet;
        state.ethernet.connected = status.ethernet_connected;
        state.ethernet.name = status.ethernet_name.clone();
        state.wifi_enabled = status.wifi_enabled;
        state.wifi_ssid = status.wifi_ssid;
        state.vpn_active = status.vpn_active;
        state.vpn_name = status.vpn_name;

        // Dynamic interface, IP and gateway discovery via /proc and UDP kernel route (zero subprocesses)
        let (def_iface, def_ip, def_gw) = network_manager::get_default_route_info();
        if state.primary_is_ethernet {
            state.ethernet.interface = def_iface.clone();
            state.ethernet.ip = def_ip.clone();
            state.ethernet.gateway = def_gw.clone();
            if !def_iface.is_empty() {
                let speed_path = format!("/sys/class/net/{}/speed", def_iface);
                if let Ok(spd) = std::fs::read_to_string(&speed_path) {
                    if let Ok(spd_u) = spd.trim().parse::<u32>() {
                        state.ethernet.speed_mbps = spd_u;
                    }
                }
            }
        } else if state.wifi_enabled && state.wifi_ssid != "Disconnected" {
            state.wifi_ip = def_ip;
            state.wifi_gateway = def_gw;
        }

        // Live VPN details via D-Bus
        state.vpn_profiles = network_manager::query_vpn_profiles().await;
        if state.vpn_active {
            let (rx_str, tx_str) = Self::get_traffic("tun0");
            let latency = network_manager::measure_latency("1.1.1.1");

            state.vpn_details = VpnDetails {
                active: true,
                name: state.vpn_name.clone(),
                interface: "tun0".to_string(),
                ip: state.wifi_ip.clone(),
                rx_bytes_str: rx_str,
                tx_bytes_str: tx_str,
                latency_ms: latency,
                cipher: "AES-256-GCM".to_string(),
            };
        }

        // Scan nearby access points via NetworkManager D-Bus
        if scan {
            state.scanned_aps = network_manager::scan_access_points().await;
            if let Some(in_use_ap) = state.scanned_aps.iter_mut().find(|ap| ap.ssid == state.wifi_ssid) {
                in_use_ap.in_use = true;
            }
        }

        state
    }

    fn format_traffic_bytes(bytes: u64) -> String {
        if bytes >= 1024 * 1024 * 1024 {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if bytes >= 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else if bytes >= 1024 {
            format!("{:.0} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        }
    }

    fn get_traffic(iface: &str) -> (String, String) {
        if let Ok(content) = std::fs::read_to_string("/proc/net/dev") {
            for line in content.lines() {
                if let Some(pos) = line.find(':') {
                    let name = line[..pos].trim();
                    if name == iface {
                        let parts: Vec<&str> = line[pos + 1..].split_whitespace().collect();
                        if parts.len() >= 9 {
                            let rx = parts[0].parse::<u64>().unwrap_or(0);
                            let tx = parts[8].parse::<u64>().unwrap_or(0);
                            return (Self::format_traffic_bytes(rx), Self::format_traffic_bytes(tx));
                        }
                    }
                }
            }
        }
        ("0 B".to_string(), "0 B".to_string())
    }

    pub fn set_wifi_enabled(enabled: bool) {
        crate::services::mpris::MPRIS_RT.spawn(async move {
            network_manager::set_wifi_enabled_dbus(enabled).await;
        });
    }

    pub fn connect_wifi(ssid: &str) {
        let s = ssid.to_string();
        crate::services::mpris::MPRIS_RT.spawn(async move {
            network_manager::activate_connection_by_id(&s).await;
        });
    }

    pub fn disconnect_wifi(ssid: &str) {
        let s = ssid.to_string();
        crate::services::mpris::MPRIS_RT.spawn(async move {
            network_manager::deactivate_connection_by_id(&s).await;
        });
    }

    pub fn connect_vpn(name: &str) {
        let target = if name.is_empty() || name == "Disconnected" || name == "tun0" {
            "phamnv".to_string()
        } else {
            name.to_string()
        };
        crate::services::mpris::MPRIS_RT.spawn(async move {
            network_manager::activate_connection_by_id(&target).await;
        });
    }

    pub fn disconnect_vpn(name: &str) {
        let target = if name.is_empty() || name == "Disconnected" || name == "tun0" {
            "phamnv".to_string()
        } else {
            name.to_string()
        };
        crate::services::mpris::MPRIS_RT.spawn(async move {
            network_manager::deactivate_connection_by_id(&target).await;
        });
    }
}
