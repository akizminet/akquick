use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

const APP_ID: &str = "dev.akquick.ControlCenter";

fn main() -> glib::ExitCode {
    let app = libadwaita::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &libadwaita::Application) {
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Quick Settings")
        .default_width(420)
        .build();

    // Enable Wayland Layer-Shell
    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.set_namespace("akquick");
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Right, true);
    window.set_margin(Edge::Top, 42);
    window.set_margin(Edge::Right, 16);

    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(12)
        .margin_top(16)
        .margin_bottom(16)
        .margin_start(16)
        .margin_end(16)
        .build();

    let label = gtk4::Label::builder()
        .label("akquick — Control Center")
        .css_classes(["title-label"])
        .build();

    container.append(&label);
    window.set_child(Some(&container));

    window.present();
}
