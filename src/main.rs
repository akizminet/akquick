mod app;
mod services;
mod ui;

use app::App;
use std::time::Instant;

pub static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

fn main() -> glib::ExitCode {
    let _ = START_TIME.set(Instant::now());
    let app = App::new();
    app.run()
}

