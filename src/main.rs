mod app;
mod services;
mod ui;

use app::App;

fn main() -> glib::ExitCode {
    let app = App::new();
    app.run()
}
