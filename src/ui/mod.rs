pub mod footer;
pub mod header;
pub mod mpris;
pub mod pages;
pub mod sliders;
pub mod toggles;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use crate::services::SystemState;

pub struct QuickSettingsWindow {
    pub window: gtk4::ApplicationWindow,
    pub stack: gtk4::Stack,
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

        // Stack for multi-page drilldown navigation
        let stack = gtk4::Stack::builder()
            .transition_type(gtk4::StackTransitionType::SlideLeftRight)
            .transition_duration(250)
            .build();

        // 1. Main Obsidian Quick Settings Card
        let main_card = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(12)
            .css_classes(["quicksettings-card"])
            .width_request(420)
            .build();

        let header = header::HeaderBar::new();
        main_card.append(&header.widget);

        let stack_wifi = stack.clone();
        let stack_bt = stack.clone();
        let stack_vpn = stack.clone();

        let toggles = toggles::ToggleGrid::new(
            &state,
            move || stack_wifi.set_visible_child_name("wifi"),
            move || stack_bt.set_visible_child_name("bluetooth"),
            move || stack_vpn.set_visible_child_name("vpn"),
        );
        main_card.append(&toggles.widget);

        let sliders = sliders::SlidersSection::new(&state);
        main_card.append(&sliders.widget);

        let mpris = mpris::MprisCard::new(&state);
        main_card.append(&mpris.widget);

        let footer = footer::FooterBar::new(&state);
        main_card.append(&footer.widget);

        // 2. Wi-Fi Sub-Page
        let stack_back1 = stack.clone();
        let wifi_page = pages::WifiPage::new(&state, move || {
            stack_back1.set_visible_child_name("main");
        });

        // 3. Bluetooth Sub-Page
        let stack_back2 = stack.clone();
        let bt_page = pages::BluetoothPage::new(&state, move || {
            stack_back2.set_visible_child_name("main");
        });

        // 4. VPN Sub-Page
        let stack_back3 = stack.clone();
        let vpn_page = pages::VpnPage::new(&state, move || {
            stack_back3.set_visible_child_name("main");
        });

        stack.add_named(&main_card, Some("main"));
        stack.add_named(&wifi_page.widget, Some("wifi"));
        stack.add_named(&bt_page.widget, Some("bluetooth"));
        stack.add_named(&vpn_page.widget, Some("vpn"));
        stack.set_visible_child_name("main");

        window.set_child(Some(&stack));

        Self { window, stack }
    }

    pub fn open_page(&self, page_name: &str) {
        self.stack.set_visible_child_name(page_name);
        self.window.set_visible(true);
        self.window.present();
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
