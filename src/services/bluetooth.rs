use zbus::proxy;
use std::collections::HashMap;
use std::process::Command;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

type ManagedObjects = HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>>;

#[proxy(
    default_service = "org.bluez",
    default_path = "/",
    interface = "org.freedesktop.DBus.ObjectManager"
)]
trait ObjectManager {
    fn get_managed_objects(&self) -> zbus::Result<ManagedObjects>;
}

#[derive(Debug, Clone, Default)]
pub struct BluetoothDevice {
    pub name: String,
    pub address: String,
    pub connected: bool,
}

#[derive(Debug, Clone, Default)]
pub struct BluetoothState {
    pub enabled: bool,
    pub adapter_name: String,
    pub connected_count: u32,
    pub devices: Vec<BluetoothDevice>,
}

pub struct BluetoothService;

impl BluetoothService {
    pub async fn fetch() -> BluetoothState {
        let mut state = BluetoothState {
            enabled: false,
            adapter_name: "hci0".to_string(),
            connected_count: 0,
            devices: Vec::new(),
        };

        let conn = match zbus::Connection::system().await {
            Ok(c) => c,
            Err(_) => return state,
        };

        let manager = match ObjectManagerProxy::new(&conn).await {
            Ok(m) => m,
            Err(_) => return state,
        };

        if let Ok(objects) = manager.get_managed_objects().await {
            for (_path, interfaces) in objects {
                // Check Adapter
                if let Some(adapter_props) = interfaces.get("org.bluez.Adapter1") {
                    if let Some(powered) = adapter_props.get("Powered") {
                        if let Ok(p) = bool::try_from(powered) {
                            state.enabled = p;
                        }
                    }
                    if let Some(name) = adapter_props.get("Name") {
                        state.adapter_name = name.to_string().trim_matches('"').to_string();
                    }
                }

                // Check Device
                if let Some(device_props) = interfaces.get("org.bluez.Device1") {
                    let mut dev = BluetoothDevice::default();

                    if let Some(name_val) = device_props.get("Name").or_else(|| device_props.get("Alias")) {
                        dev.name = name_val.to_string().trim_matches('"').to_string();
                    }
                    if let Some(addr_val) = device_props.get("Address") {
                        dev.address = addr_val.to_string().trim_matches('"').to_string();
                    }
                    if let Some(conn_val) = device_props.get("Connected") {
                        if let Ok(c) = bool::try_from(conn_val) {
                            dev.connected = c;
                            if c {
                                state.connected_count += 1;
                            }
                        }
                    }

                    if !dev.name.is_empty() {
                        state.devices.push(dev);
                    }
                }
            }
        }

        state
    }

    pub fn set_enabled(enabled: bool) {
        let arg = if enabled { "on" } else { "off" };
        let _ = Command::new("bluetoothctl").args(["power", arg]).spawn();
    }

    pub fn connect_device(address: &str) {
        let _ = Command::new("bluetoothctl").args(["connect", address]).spawn();
    }

    pub fn disconnect_device(address: &str) {
        let _ = Command::new("bluetoothctl").args(["disconnect", address]).spawn();
    }
}
