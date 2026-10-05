use gtk4::prelude::*;
use crate::services::{AudioService, SystemState};

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

        // 1. Wi-Fi (from NetworkManager DBus)
        let wifi_pill = Self::create_pill(
            "network-wireless-symbolic",
            "Wi-Fi",
            &state.network.wifi_ssid,
            true,
            if state.network.wifi_enabled { Some("active-primary") } else { None },
        );
        wifi_pill.connect_clicked(|_| {
            let _ = std::process::Command::new("nmrs-gui").spawn();
        });

        // 2. Bluetooth (from BlueZ DBus)
        let bt_sub = if state.bluetooth.connected_count > 0 {
            format!("{} Connected", state.bluetooth.connected_count)
        } else {
            "No Devices".to_string()
        };
        let bt_pill = Self::create_pill(
            "bluetooth-active-symbolic",
            "Bluetooth",
            &bt_sub,
            true,
            if state.bluetooth.enabled && state.bluetooth.connected_count > 0 {
                Some("active-primary")
            } else {
                None // Inactive grey when 0 devices connected!
            },
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

        // 4. Night Light (Live hyprsunset check)
        let night_pill = Self::create_pill(
            "weather-clear-night-symbolic",
            "Night Light",
            if state.night_light_active { "4000K Warm" } else { "Off" },
            false,
            if state.night_light_active { Some("active-tertiary") } else { None },
        );
        night_pill.connect_clicked(|_| {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg("if pgrep -x hyprsunset; then pkill hyprsunset; else hyprsunset -t 4000 & fi")
                .spawn();
        });

        // 5. Microphone
        let mic_label = if !state.audio.has_input_device {
            "● No Input"
        } else if state.audio.mic_muted {
            "● Muted"
        } else {
            "● Unmuted"
        };
        let mic_pill = Self::create_pill(
            if state.audio.mic_muted || !state.audio.has_input_device {
                "microphone-sensitivity-muted-symbolic"
            } else {
                "audio-input-microphone-symbolic"
            },
            "Microphone",
            mic_label,
            false,
            if state.audio.has_input_device && !state.audio.mic_muted {
                Some("active-secondary")
            } else {
                None
            },
        );
        mic_pill.connect_clicked(|_| {
            AudioService::toggle_mic_mute();
        });

        // 6. VPN (Real active connection name from NetworkManager DBus)
        let vpn_pill = Self::create_pill(
            "network-vpn-symbolic",
            "VPN",
            &state.network.vpn_name,
            true,
            if state.network.vpn_active { Some("active-secondary") } else { None },
        );

        // 7. Power Menu
        let power_pill = Self::create_pill(
            "system-shutdown-symbolic",
            "Power Menu",
            "Shutdown / Lock",
            false,
            None,
        );
        power_pill.connect_clicked(|_| {
            let _ = std::process::Command::new("/var/home/phamnv/.config/rofi/powermenu.sh").spawn();
        });

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
        grid.attach(&vpn_pill, 1, 2, 1, 1);
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
