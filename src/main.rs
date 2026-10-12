mod app;
mod services;
mod status;
mod ui;

use app::App;
use std::time::Instant;

pub static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

fn main() -> glib::ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if let Some(code) = status::handle_cli(&args) {
        return code;
    }

    let _ = START_TIME.set(Instant::now());
    let app = App::new();
    app.run()
}

