pub mod actions;
pub mod audio;
pub mod bluetooth;
pub mod mpris;
pub mod network;
pub mod telemetry;

pub use actions::ActionService;
pub use audio::{AudioService, AudioState};
pub use bluetooth::{BluetoothService, BluetoothState};
pub use mpris::{MprisService, MprisState};
pub use network::{NetworkService, NetworkState};
pub use telemetry::{TelemetryService, TelemetryState};

use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct SystemState {
    pub audio: AudioState,
    pub network: NetworkState,
    pub bluetooth: BluetoothState,
    pub mpris: MprisState,
    pub telemetry: TelemetryState,
    pub dnd_active: bool,
    pub night_light_active: bool,
    pub dark_active: bool,
}

impl SystemState {
    pub fn fetch() -> Self {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        rt.block_on(async {
            let (network, bluetooth, mpris) = tokio::join!(
                NetworkService::fetch(),
                BluetoothService::fetch(),
                MprisService::fetch(),
            );

            let audio = AudioService::fetch();
            let telemetry = TelemetryService::fetch();

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
                telemetry,
                dnd_active: false,
                night_light_active,
                dark_active: true,
            }
        })
    }
}
