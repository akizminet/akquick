use gtk4::prelude::*;
use crate::services::SystemState;

pub struct ToggleGrid {
    pub widget: gtk4::Grid,
}

impl ToggleGrid {
    pub fn new(state: &SystemState) -> Self {
        let grid = gtk4::Grid::builder()
            .column_spacing(10)
            .row_spacing(10)
            .column_homogeneous(true)
            .row_homogeneous(true)
            .css_classes(["toggle-grid"])
            .build();

        // 1. Wi-Fi
        let wifi_pill = Self::create_pill(
            "network-wireless-symbolic",
            "Wi-Fi",
            &state.wifi_ssid,
            true, // chevron
            if state.wifi_enabled { Some("active-primary") } else { None },
        );
        wifi_pill.connect_clicked(|_| {
            let _ = std::process::Command::new("nmrs-gui").spawn();
        });

        // 2. Bluetooth
        let bt_sub = format!("{} Connected", state.bluetooth_connected);
        let bt_pill = Self::create_pill(
            "bluetooth-active-symbolic",
            "Bluetooth",
            &bt_sub,
            true,
            if state.bluetooth_enabled { Some("active-primary") } else { None },
        );
        bt_pill.connect_clicked(|_| {
            let _ = std::process::Command::new("bluetoothctl").args(["power", "toggle"]).spawn();
        });

        // 3. Do Not Disturb
        let dnd_pill = Self::create_pill(
            "notifications-disabled-symbolic",
            "Do Not Disturb",
            if state.dnd_active { "On" } else { "Off" },
            false,
            if state.dnd_active { Some("active-tertiary") } else { None },
        );
        dnd_pill.connect_clicked(|_| {
            let _ = std::process::Command::new("swaync-client").arg("-d").spawn();
        });

        // 4. Night Light
        let night_pill = Self::create_pill(
            "weather-clear-night-symbolic",
            "Night Light",
            if state.night_light_active { "4000K Warm" } else { "Off" },
            false,
            if state.night_light_active { Some("active-tertiary") } else { Some("active-tertiary") },
        );
        night_pill.connect_clicked(|_| {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg("if pgrep -x hyprsunset; then pkill hyprsunset; else hyprsunset -t 4000 & fi")
                .spawn();
        });

        // 5. Microphone
        let mic_pill = Self::create_pill(
            if state.mic_muted { "microphone-sensitivity-muted-symbolic" } else { "audio-input-microphone-symbolic" },
            "Microphone",
            if state.mic_muted { "● Muted" } else { "● Unmuted" },
            false,
            if !state.mic_muted { Some("active-secondary") } else { None },
        );
        mic_pill.connect_clicked(|_| {
            SystemState::toggle_mic_mute();
        });

        // 6. WireGuard
        let wg_pill = Self::create_pill(
            "network-vpn-symbolic",
            "WireGuard",
            if state.wireguard_active { "Zurich-01" } else { "Disconnected" },
            true,
            if state.wireguard_active { Some("active-secondary") } else { Some("active-secondary") },
        );

        // 7. Power Mode
        let power_pill = Self::create_pill(
            "power-profile-balanced-symbolic",
            "Power Mode",
            &state.power_mode,
            false,
            None,
        );

        // 8. Dark Style
        let dark_pill = Self::create_pill(
            "night-light-symbolic",
            "Dark Style",
            if state.dark_active { "Dark Active" } else { "Light Active" },
            false,
            None,
        );

        grid.attach(&wifi_pill, 0, 0, 1, 1);
        grid.attach(&bt_pill, 1, 0, 1, 1);
        grid.attach(&dnd_pill, 0, 1, 1, 1);
        grid.attach(&night_pill, 1, 1, 1, 1);
        grid.attach(&mic_pill, 0, 2, 1, 1);
        grid.attach(&wg_pill, 1, 2, 1, 1);
        grid.attach(&power_pill, 0, 3, 1, 1);
        grid.attach(&dark_pill, 1, 3, 1, 1);

        Self { widget: grid }
    }

    fn create_pill(
        icon: &str,
        title: &str,
        subtitle: &str,
        show_chevron: bool,
        active_class: Option<&str>,
    ) -> gtk4::Button {
        let btn = gtk4::Button::builder()
            .css_classes(["toggle-pill"])
            .hexpand(true)
            .build();

        if let Some(cls) = active_class {
            btn.add_css_class(cls);
        }

        let hbox = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(10)
            .valign(gtk4::Align::Center)
            .build();

        let icon_img = gtk4::Image::builder()
            .icon_name(icon)
            .pixel_size(20)
            .valign(gtk4::Align::Center)
            .build();

        let vbox = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .valign(gtk4::Align::Center)
            .build();

        let title_label = gtk4::Label::builder()
            .label(title)
            .halign(gtk4::Align::Start)
            .css_classes(["toggle-title"])
            .build();

        let sub_label = gtk4::Label::builder()
            .label(subtitle)
            .halign(gtk4::Align::Start)
            .css_classes(["toggle-subtitle"])
            .build();

        vbox.append(&title_label);
        vbox.append(&sub_label);

        hbox.append(&icon_img);
        hbox.append(&vbox);

        if show_chevron {
            let chevron = gtk4::Image::builder()
                .icon_name("go-next-symbolic")
                .pixel_size(14)
                .halign(gtk4::Align::End)
                .valign(gtk4::Align::Center)
                .opacity(0.8)
                .build();
            hbox.append(&chevron);
        }

        btn.set_child(Some(&hbox));
        btn
    }
}
