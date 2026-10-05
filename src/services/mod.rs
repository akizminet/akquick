pub mod actions;
pub mod audio;
pub mod backlight;
pub mod battery;
pub mod bluetooth;
pub mod mpris;
pub mod network;

pub use actions::ActionService;
pub use audio::{AudioService, AudioState};
pub use backlight::BacklightService;
pub use battery::{BatteryService, BatteryState};
pub use bluetooth::{BluetoothService, BluetoothState};
pub use mpris::{MprisService, MprisState};
pub use network::{NetworkService, NetworkState};

use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct SystemState {
    pub audio: AudioState,
    pub network: NetworkState,
    pub bluetooth: BluetoothState,
    pub mpris: MprisState,
    pub battery: BatteryState,
    pub brightness_percent: u32,
    pub dnd_active: bool,
    pub night_light_active: bool,
    pub power_mode: String,
    pub dark_active: bool,
}

impl SystemState {
    pub fn fetch() -> Self {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        rt.block_on(async {
            let (network, bluetooth, mpris, battery) = tokio::join!(
                NetworkService::fetch(),
                BluetoothService::fetch(),
                MprisService::fetch(),
                BatteryService::fetch(),
            );

            let audio = AudioService::fetch();
            let brightness_percent = BacklightService::fetch();

            let night_light_active = Command::new("pgrep")
                .arg("-x")
                .arg("hyprsunset")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            SystemState {
                audio,
                network,
                bluetooth,
                mpris,
                battery,
                brightness_percent,
                dnd_active: false,
                night_light_active,
                power_mode: "Balanced".to_string(),
                dark_active: true,
            }
        })
    }
}
