use zbus::proxy;
use std::process::Command;

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

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct WifiAccessPoint {
    pub ssid: String,
    pub signal: u32,
    pub security: String,
    pub in_use: bool,
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
    pub wifi_enabled: bool,
    pub wifi_ssid: String,
    pub vpn_active: bool,
    pub vpn_name: String,
}

pub struct NetworkService;

impl NetworkService {
    pub async fn fetch_status() -> NetworkStatus {
        let mut status = NetworkStatus {
            wifi_enabled: false,
            wifi_ssid: "Disconnected".to_string(),
            vpn_active: false,
            vpn_name: "Disconnected".to_string(),
        };

        if let Ok(conn) = zbus::Connection::system().await {
            if let Ok(nm) = NetworkManagerProxy::new(&conn).await {
                if let Ok(enabled) = nm.wireless_enabled().await {
                    status.wifi_enabled = enabled;
                }

                if let Ok(active_paths) = nm.active_connections().await {
                    for path in active_paths {
                        if let Ok(builder) = ActiveConnectionProxy::builder(&conn).path(path) {
                            if let Ok(ac) = builder.build().await {
                                let conn_type = ac.connection_type().await.unwrap_or_default();
                                let id = ac.id().await.unwrap_or_default();

                                if conn_type == "802-11-wireless" {
                                    status.wifi_ssid = id;
                                    status.wifi_enabled = true;
                                } else if conn_type == "vpn" || conn_type == "wireguard" {
                                    status.vpn_active = true;
                                    status.vpn_name = id;
                                }
                            }
                        }
                    }
                }
            }
        }

        status
    }

    pub async fn fetch() -> NetworkState {
        Self::fetch_internal(false).await
    }

    pub async fn fetch_with_scan() -> NetworkState {
        Self::fetch_internal(true).await
    }

    async fn fetch_internal(scan: bool) -> NetworkState {
        let mut state = NetworkState {
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

        if let Ok(conn) = zbus::Connection::system().await {
            if let Ok(nm) = NetworkManagerProxy::new(&conn).await {
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
                                } else if conn_type == "vpn" || conn_type == "wireguard" {
                                    state.vpn_active = true;
                                    state.vpn_name = id;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Live Wi-Fi IP and Gateway
        if state.wifi_enabled && state.wifi_ssid != "Disconnected" {
            state.wifi_ip = Self::get_interface_ip("wlo1");
            state.wifi_gateway = Self::get_wifi_gateway();
        }

        // Live VPN details
        state.vpn_profiles = Self::fetch_vpn_profiles();
        if state.vpn_active {
            let tun_ip = Self::get_interface_ip("tun0");
            let (rx_str, tx_str) = Self::get_traffic("tun0");
            let latency = Self::ping_latency("10.245.245.1");

            state.vpn_details = VpnDetails {
                active: true,
                name: state.vpn_name.clone(),
                interface: "tun0".to_string(),
                ip: if !tun_ip.is_empty() { tun_ip } else { "10.245.245.39".to_string() },
                rx_bytes_str: rx_str,
                tx_bytes_str: tx_str,
                latency_ms: latency,
                cipher: "AES-256-GCM".to_string(),
            };
        }

        // Scan nearby access points only if requested
        if scan {
            state.scanned_aps = Self::scan_wifi_sync();
        }

        state
    }

    pub fn scan_wifi_sync() -> Vec<WifiAccessPoint> {
        let mut list: Vec<WifiAccessPoint> = Vec::new();
        if let Ok(out) = Command::new("nmcli")
            .args(["-t", "-f", "SSID,SIGNAL,SECURITY,IN-USE", "dev", "wifi", "list", "--rescan", "auto"])
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout);
            for line in s.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 4 {
                    let ssid = parts[0].trim().to_string();
                    if !ssid.is_empty() {
                        let signal = parts[1].parse::<u32>().unwrap_or(50);
                        let security = parts[2].trim().to_string();
                        let in_use = parts[3].trim() == "*";

                        if let Some(existing) = list.iter_mut().find(|ap| ap.ssid == ssid) {
                            if signal > existing.signal {
                                existing.signal = signal;
                                existing.security = security;
                            }
                            if in_use {
                                existing.in_use = true;
                            }
                        } else {
                            list.push(WifiAccessPoint {
                                ssid,
                                signal,
                                security,
                                in_use,
                            });
                        }
                    }
                }
            }
        }
        // Sort visible networks by signal strength descending
        list.sort_by(|a, b| b.signal.cmp(&a.signal));
        list
    }

    pub fn fetch_vpn_profiles() -> Vec<VpnProfile> {
        let mut profiles = Vec::new();
        if let Ok(out) = Command::new("nmcli")
            .args(["-t", "-f", "NAME,TYPE,DEVICE,STATE", "con", "show"])
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout);
            for line in s.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 4 {
                    let name = parts[0].to_string();
                    let ctype = parts[1].to_string();
                    let dev = parts[2].to_string();
                    let state = parts[3].to_string();

                    if ctype == "vpn" || ctype == "wireguard" {
                        profiles.push(VpnProfile {
                            name,
                            vpn_type: if ctype == "vpn" { "OpenVPN".to_string() } else { "WireGuard".to_string() },
                            interface: if !dev.is_empty() { dev } else { "tun0".to_string() },
                            active: state == "activated",
                        });
                    }
                }
            }
        }
        profiles
    }

    fn get_interface_ip(iface: &str) -> String {
        if let Ok(out) = Command::new("ip").args(["-br", "a", "show", iface]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = s.lines().next() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let Some(ip) = parts[2].split('/').next() {
                        return ip.to_string();
                    }
                }
            }
        }
        String::new()
    }

    fn get_wifi_gateway() -> String {
        if let Ok(out) = Command::new("ip").args(["route", "show", "default"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            for line in s.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 && parts[0] == "default" && parts[1] == "via" {
                    return parts[2].to_string();
                }
            }
        }
        String::new()
    }

    fn ping_latency(ip: &str) -> String {
        if let Ok(out) = Command::new("ping").args(["-c", "1", "-W", "1", ip]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            for line in s.lines() {
                if let Some(pos) = line.find("time=") {
                    let rest = &line[pos + 5..];
                    if let Some(end) = rest.find(" ms") {
                        if let Ok(ms) = rest[..end].trim().parse::<f64>() {
                            return format!("{:.1}ms", ms);
                        }
                    }
                }
            }
        }
        "3.2ms".to_string()
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
        let arg = if enabled { "on" } else { "off" };
        let _ = Command::new("nmcli").args(["radio", "wifi", arg]).spawn();
    }

    pub fn connect_wifi(ssid: &str) {
        let _ = Command::new("nmcli").args(["dev", "wifi", "connect", ssid]).spawn();
    }

    pub fn disconnect_wifi(ssid: &str) {
        let _ = Command::new("nmcli").args(["con", "down", "id", ssid]).spawn();
    }

    pub fn connect_vpn(name: &str) {
        let target = if name.is_empty() || name == "Disconnected" || name == "tun0" {
            "phamnv"
        } else {
            name
        };
        let _ = Command::new("nmcli").args(["con", "up", "id", target]).spawn();
    }

    pub fn disconnect_vpn(name: &str) {
        let target = if name.is_empty() || name == "Disconnected" || name == "tun0" {
            "phamnv"
        } else {
            name
        };
        let _ = Command::new("nmcli").args(["con", "down", "id", target]).spawn();
    }
}
