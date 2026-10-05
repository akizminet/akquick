pub mod actions;
pub mod audio;
pub mod bluetooth;
pub mod display;
pub mod mpris;
pub mod network;
pub mod telemetry;

pub use actions::ActionService;
pub use audio::{AudioService, AudioState};
pub use bluetooth::{BluetoothService, BluetoothState};
pub use display::{DisplayService, MonitorDisplay};
pub use mpris::{MprisService, MprisState};
pub use network::{NetworkService, NetworkState, NetworkStatus, WifiAccessPoint};
pub use telemetry::{TelemetryService, TelemetryState};

use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct SystemState {
    pub audio: AudioState,
    pub network: NetworkState,
    pub bluetooth: BluetoothState,
    pub mpris: MprisState,
    pub telemetry: TelemetryState,
    pub displays: Vec<MonitorDisplay>,
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
            let t0 = std::time::Instant::now();
            let network = NetworkService::fetch().await;
            let t_net = t0.elapsed();
            
            let t1 = std::time::Instant::now();
            let bluetooth = BluetoothService::fetch().await;
            let t_bt = t1.elapsed();

            let t2 = std::time::Instant::now();
            let mpris = MprisService::fetch().await;
            let t_mpris = t2.elapsed();

            let t3 = std::time::Instant::now();
            let audio = AudioService::fetch();
            let t_audio = t3.elapsed();

            let t4 = std::time::Instant::now();
            let telemetry = TelemetryService::fetch();
            let t_telem = t4.elapsed();

            let t5 = std::time::Instant::now();
            let displays = DisplayService::fetch();
            let t_disp = t5.elapsed();

            if std::env::var("AKQUICK_BENCH").is_ok() {
                eprintln!("   -> Network:   {:>8.2?}", t_net);
                eprintln!("   -> Bluetooth: {:>8.2?}", t_bt);
                eprintln!("   -> MPRIS:     {:>8.2?}", t_mpris);
                eprintln!("   -> Audio:     {:>8.2?}", t_audio);
                eprintln!("   -> Telemetry: {:>8.2?}", t_telem);
                eprintln!("   -> Displays:  {:>8.2?}", t_disp);
            }

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
                displays,
                dnd_active: false,
                night_light_active,
                dark_active: true,
            }
        })
    }
}
