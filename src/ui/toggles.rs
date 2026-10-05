use gtk4::prelude::*;
use crate::services::{AudioService, NetworkService, BluetoothService, SystemState};

pub struct ToggleGrid {
    pub widget: gtk4::Grid,
}

impl ToggleGrid {
    pub fn new<FW, FB, FV>(
        state: &SystemState,
        on_open_wifi: FW,
        on_open_bluetooth: FB,
        on_open_vpn: FV,
    ) -> Self
    where
        FW: Fn() + 'static,
        FB: Fn() + 'static,
        FV: Fn() + 'static,
    {
        let grid = gtk4::Grid::builder()
            .column_spacing(10)
            .row_spacing(10)
            .column_homogeneous(true)
            .row_homogeneous(true)
            .css_classes(["toggle-grid"])
            .build();

        // 1. Wi-Fi (Split Pill: Toggle wifi on click, Open Wi-Fi page on chevron)
        let wifi_en = state.network.wifi_enabled;
        let wifi_pill = Self::create_split_pill(
            "network-wireless-symbolic",
            "Wi-Fi",
            &state.network.wifi_ssid,
            if state.network.wifi_enabled { Some("active-primary") } else { None },
            move || {
                NetworkService::set_wifi_enabled(!wifi_en);
            },
            on_open_wifi,
        );

        // 2. Bluetooth (Split Pill: Toggle power on click, Open Bluetooth page on chevron)
        let bt_sub = if state.bluetooth.connected_count > 0 {
            format!("{} Connected", state.bluetooth.connected_count)
        } else {
            "No Devices".to_string()
        };
        let bt_en = state.bluetooth.enabled;
        let bt_pill = Self::create_split_pill(
            "bluetooth-active-symbolic",
            "Bluetooth",
            &bt_sub,
            if state.bluetooth.enabled && state.bluetooth.connected_count > 0 {
                Some("active-primary")
            } else {
                None
            },
            move || {
                BluetoothService::set_enabled(!bt_en);
            },
            on_open_bluetooth,
        );

        // 3. Do Not Disturb (Standard Pill)
        let dnd_pill = Self::create_pill(
            "notifications-disabled-symbolic",
            "Do Not Disturb",
            if state.dnd_active { "On" } else { "Off" },
            if state.dnd_active { Some("active-tertiary") } else { None },
        );
        dnd_pill.connect_clicked(|_| {
            let _ = std::process::Command::new("swaync-client").arg("-d").spawn();
        });

        // 4. Night Light (Standard Pill)
        let night_pill = Self::create_pill(
            "weather-clear-night-symbolic",
            "Night Light",
            if state.night_light_active { "4000K Warm" } else { "Off" },
            if state.night_light_active { Some("active-tertiary") } else { None },
        );
        night_pill.connect_clicked(|_| {
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg("if pgrep -x hyprsunset; then pkill hyprsunset; else hyprsunset -t 4000 & fi")
                .spawn();
        });

        // 5. Microphone (Standard Pill)
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
            if state.audio.has_input_device && !state.audio.mic_muted {
                Some("active-secondary")
            } else {
                None
            },
        );
        mic_pill.connect_clicked(|_| {
            AudioService::toggle_mic_mute();
        });

        // 6. VPN & WireGuard (Split Pill: Toggle connect/disconnect on click, Open VPN page on chevron)
        let is_vpn = state.network.vpn_active;
        let vpn_id = state.network.vpn_name.clone();
        let vpn_pill = Self::create_split_pill(
            "network-vpn-symbolic",
            "VPN & WG",
            &state.network.vpn_name,
            if state.network.vpn_active { Some("active-secondary") } else { None },
            move || {
                if is_vpn {
                    NetworkService::disconnect_vpn(&vpn_id);
                } else {
                    NetworkService::connect_vpn("phamnv");
                }
            },
            on_open_vpn,
        );

        // 7. Power Menu (Standard Pill)
        let power_pill = Self::create_pill(
            "system-shutdown-symbolic",
            "Power Menu",
            "Shutdown / Lock",
            None,
        );
        power_pill.connect_clicked(|_| {
            let _ = std::process::Command::new("/var/home/phamnv/.config/rofi/powermenu.sh").spawn();
        });

        // 8. Dark Style (Standard Pill)
        let dark_pill = Self::create_pill(
            "night-light-symbolic",
            "Dark Style",
            if state.dark_active { "Dark Active" } else { "Light Active" },
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

    fn create_split_pill<FT, FD>(
        icon: &str,
        title: &str,
        subtitle: &str,
        active_class: Option<&str>,
        on_toggle: FT,
        on_drilldown: FD,
    ) -> gtk4::Box
    where
        FT: Fn() + 'static,
        FD: Fn() + 'static,
    {
        let pill_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .css_classes(["toggle-pill-split"])
            .hexpand(true)
            .build();

        if let Some(cls) = active_class {
            pill_box.add_css_class(cls);
        }

        let main_btn = gtk4::Button::builder()
            .css_classes(["toggle-split-main"])
            .hexpand(true)
            .build();

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
        main_btn.set_child(Some(&hbox));
        main_btn.connect_clicked(move |_| {
            on_toggle();
        });

        let chevron_btn = gtk4::Button::builder()
            .css_classes(["toggle-split-chevron"])
            .tooltip_text("Open Details")
            .build();

        let chevron_img = gtk4::Image::builder()
            .icon_name("go-next-symbolic")
            .pixel_size(14)
            .build();
        chevron_btn.set_child(Some(&chevron_img));
        chevron_btn.connect_clicked(move |_| {
            on_drilldown();
        });

        pill_box.append(&main_btn);
        pill_box.append(&chevron_btn);

        pill_box
    }

    fn create_pill(
        icon: &str,
        title: &str,
        subtitle: &str,
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

        btn.set_child(Some(&hbox));
        btn
    }
}
