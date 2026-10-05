use zbus::proxy;
use std::collections::HashMap;
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
pub struct BluetoothState {
    pub enabled: bool,
    pub connected_count: u32,
}

pub struct BluetoothService;

impl BluetoothService {
    pub async fn fetch() -> BluetoothState {
        let mut state = BluetoothState {
            enabled: false,
            connected_count: 0,
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
                }

                // Check Device
                if let Some(device_props) = interfaces.get("org.bluez.Device1") {
                    if let Some(connected) = device_props.get("Connected") {
                        if let Ok(c) = bool::try_from(connected) {
                            if c {
                                state.connected_count += 1;
                            }
                        }
                    }
                }
            }
        }

        state
    }
}
