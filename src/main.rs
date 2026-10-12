mod app;
mod ipc;
mod services;
mod status;
mod translators;
mod ui;

use app::App;
use std::time::Instant;

pub static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

fn main() -> glib::ExitCode {
    let args: Vec<String> = std::env::args().collect();

    // Fast-path: query running daemon via pure std Unix Domain Socket (<0.1ms)
    if let Some(code) = ipc::try_handle_cli(&args) {
        return code;
    }

    if let Some(code) = status::handle_cli(&args) {
        return code;
    }

    let _ = START_TIME.set(Instant::now());
    let app = App::new();
    app.run()
}

