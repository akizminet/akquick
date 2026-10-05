pub mod footer;
pub mod header;
pub mod mpris;
pub mod sliders;
pub mod toggles;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use crate::services::SystemState;

pub struct QuickSettingsWindow {
    pub window: gtk4::ApplicationWindow,
}

impl QuickSettingsWindow {
    pub fn new(app: &libadwaita::Application) -> Self {
        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("Quick Settings")
            .default_width(420)
            .css_classes(["quicksettings-window"])
            .build();

        // Layer-Shell Configuration
        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_namespace("akquick");
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, 42);
        window.set_margin(Edge::Right, 16);

        // Allow keyboard focus if needed, or leave normal
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);

        // Fetch live state
        let state = SystemState::fetch();

        // Main obsidian card container
        let card = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(12)
            .css_classes(["quicksettings-card"])
            .width_request(420)
            .build();

        // 1. Header Action Bar
        let header = header::HeaderBar::new();
        card.append(&header.widget);

        // 2. 2-Column Quick Toggles Grid
        let toggles = toggles::ToggleGrid::new(&state);
        card.append(&toggles.widget);

        // 3. Audio & Brightness Sliders
        let sliders = sliders::SlidersSection::new(&state);
        card.append(&sliders.widget);

        // 4. MPRIS Media Player Card
        let mpris = mpris::MprisCard::new(&state);
        card.append(&mpris.widget);

        // 5. Telemetry Footer Bar
        let footer = footer::FooterBar::new(&state);
        card.append(&footer.widget);

        window.set_child(Some(&card));

        Self { window }
    }

    pub fn toggle(&self) {
        if self.window.is_visible() {
            self.window.set_visible(false);
        } else {
            self.window.set_visible(true);
            self.window.present();
        }
    }
}
