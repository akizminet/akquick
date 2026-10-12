use std::collections::HashMap;
use zbus::proxy;
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

#[proxy(
    default_service = "org.bluez",
    interface = "org.bluez.Adapter1"
)]
trait Adapter1 {
    #[zbus(property)]
    fn set_powered(&self, value: bool) -> zbus::Result<()>;
}

#[proxy(
    default_service = "org.bluez",
    interface = "org.bluez.Device1"
)]
trait Device1 {
    fn connect(&self) -> zbus::Result<()>;
    fn disconnect(&self) -> zbus::Result<()>;
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
        crate::services::mpris::MPRIS_RT.spawn(async move {
            if let Ok(conn) = zbus::Connection::system().await {
                if let Ok(builder) = Adapter1Proxy::builder(&conn).path("/org/bluez/hci0") {
                    if let Ok(adapter) = builder.build().await {
                        let _ = adapter.set_powered(enabled).await;
                    }
                }
            }
        });
    }

    pub fn connect_device(address: &str) {
        let addr = address.to_string();
        crate::services::mpris::MPRIS_RT.spawn(async move {
            let path = format!("/org/bluez/hci0/dev_{}", addr.replace(':', "_"));
            if let Ok(conn) = zbus::Connection::system().await {
                if let Ok(builder) = Device1Proxy::builder(&conn).path(path) {
                    if let Ok(device) = builder.build().await {
                        let _ = device.connect().await;
                    }
                }
            }
        });
    }

    pub fn disconnect_device(address: &str) {
        let addr = address.to_string();
        crate::services::mpris::MPRIS_RT.spawn(async move {
            let path = format!("/org/bluez/hci0/dev_{}", addr.replace(':', "_"));
            if let Ok(conn) = zbus::Connection::system().await {
                if let Ok(builder) = Device1Proxy::builder(&conn).path(path) {
                    if let Ok(device) = builder.build().await {
                        let _ = device.disconnect().await;
                    }
                }
            }
        });
    }
}
