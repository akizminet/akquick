use gtk4::prelude::*;
use crate::services::{AudioService, NetworkService, NetworkStatus, BluetoothService, SystemState};
use std::cell::Cell;
use std::rc::Rc;

pub struct ToggleGrid {
    pub widget: gtk4::Grid,
    pub vpn_box: gtk4::Box,
    pub vpn_sub: gtk4::Label,
    pub is_vpn: Rc<Cell<bool>>,
    pub wifi_box: gtk4::Box,
    pub wifi_sub: gtk4::Label,
    pub is_wifi: Rc<Cell<bool>>,
    #[allow(dead_code)]
    pub bt_box: gtk4::Box,
    #[allow(dead_code)]
    pub bt_sub: gtk4::Label,
    #[allow(dead_code)]
    pub is_bt: Rc<Cell<bool>>,
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
        let wifi_en = Rc::new(Cell::new(state.network.wifi_enabled));
        let wifi_en_clone = wifi_en.clone();
        let last_ssid = state.network.wifi_ssid.clone();
        let (wifi_pill, wifi_sub) = Self::create_split_pill(
            "network-wireless-symbolic",
            "Wi-Fi",
            &state.network.wifi_ssid,
            if state.network.wifi_enabled { Some("active-primary") } else { None },
            move |pill, sub, _icon| {
                let next = !wifi_en_clone.get();
                wifi_en_clone.set(next);
                NetworkService::set_wifi_enabled(next);
                if next {
                    pill.add_css_class("active-primary");
                    sub.set_label(&last_ssid);
                } else {
                    pill.remove_css_class("active-primary");
                    sub.set_label("Disconnected");
                }
            },
            on_open_wifi,
        );

        // 2. Bluetooth (Split Pill: Toggle power on click, Open Bluetooth page on chevron)
        let bt_sub_text = if state.bluetooth.connected_count > 0 {
            format!("{} Connected", state.bluetooth.connected_count)
        } else {
            "No Devices".to_string()
        };
        let bt_en = Rc::new(Cell::new(state.bluetooth.enabled));
        let bt_en_clone = bt_en.clone();
        let (bt_pill, bt_sub) = Self::create_split_pill(
            "bluetooth-active-symbolic",
            "Bluetooth",
            &bt_sub_text,
            if state.bluetooth.enabled && state.bluetooth.connected_count > 0 {
                Some("active-primary")
            } else {
                None
            },
            move |pill, sub, _icon| {
                let next = !bt_en_clone.get();
                bt_en_clone.set(next);
                BluetoothService::set_enabled(next);
                if next {
                    pill.add_css_class("active-primary");
                    sub.set_label("Enabled");
                } else {
                    pill.remove_css_class("active-primary");
                    sub.set_label("Off");
                }
            },
            on_open_bluetooth,
        );

        // 3. Do Not Disturb (Standard Pill)
        let (dnd_pill, dnd_sub, _dnd_icon) = Self::create_pill(
            "notifications-disabled-symbolic",
            "Do Not Disturb",
            if state.dnd_active { "On" } else { "Off" },
            if state.dnd_active { Some("active-tertiary") } else { None },
        );
        let is_dnd = Rc::new(Cell::new(state.dnd_active));
        let is_dnd_clone = is_dnd.clone();
        let dnd_pill_clone = dnd_pill.clone();
        dnd_pill.connect_clicked(move |_| {
            let next = !is_dnd_clone.get();
            is_dnd_clone.set(next);
            let _ = std::process::Command::new("swaync-client").arg("-d").spawn();
            if next {
                dnd_pill_clone.add_css_class("active-tertiary");
                dnd_sub.set_label("On");
            } else {
                dnd_pill_clone.remove_css_class("active-tertiary");
                dnd_sub.set_label("Off");
            }
        });

        // 4. Night Light (Standard Pill)
        let (night_pill, night_sub, _night_icon) = Self::create_pill(
            "weather-clear-night-symbolic",
            "Night Light",
            if state.night_light_active { "4000K Warm" } else { "Off" },
            if state.night_light_active { Some("active-tertiary") } else { None },
        );
        let is_night = Rc::new(Cell::new(state.night_light_active));
        let is_night_clone = is_night.clone();
        let night_pill_clone = night_pill.clone();
        night_pill.connect_clicked(move |_| {
            let next = !is_night_clone.get();
            is_night_clone.set(next);
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg("if pgrep -x hyprsunset; then pkill hyprsunset; else hyprsunset -t 4000 & fi")
                .spawn();
            if next {
                night_pill_clone.add_css_class("active-tertiary");
                night_sub.set_label("4000K Warm");
            } else {
                night_pill_clone.remove_css_class("active-tertiary");
                night_sub.set_label("Off");
            }
        });

        // 5. Microphone (Standard Pill)
        let mic_label = if !state.audio.has_input_device {
            "● No Input"
        } else if state.audio.mic_muted {
            "● Muted"
        } else {
            "● Unmuted"
        };
        let (mic_pill, mic_sub, mic_icon) = Self::create_pill(
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
        let is_mic_muted = Rc::new(Cell::new(state.audio.mic_muted));
        let is_mic_muted_clone = is_mic_muted.clone();
        let has_input = state.audio.has_input_device;
        let mic_pill_clone = mic_pill.clone();
        mic_pill.connect_clicked(move |_| {
            if !has_input {
                return;
            }
            let next_muted = !is_mic_muted_clone.get();
            is_mic_muted_clone.set(next_muted);
            AudioService::toggle_mic_mute();
            if next_muted {
                mic_pill_clone.remove_css_class("active-secondary");
                mic_icon.set_icon_name(Some("microphone-sensitivity-muted-symbolic"));
                mic_sub.set_label("● Muted");
            } else {
                mic_pill_clone.add_css_class("active-secondary");
                mic_icon.set_icon_name(Some("audio-input-microphone-symbolic"));
                mic_sub.set_label("● Unmuted");
            }
        });

        // 6. VPN & WireGuard (Split Pill: Toggle connect/disconnect on click, Open VPN page on chevron)
        let is_vpn = Rc::new(Cell::new(state.network.vpn_active));
        let is_vpn_clone = is_vpn.clone();
        let vpn_id = if state.network.vpn_name.is_empty() || state.network.vpn_name == "Disconnected" {
            "phamnv".to_string()
        } else {
            state.network.vpn_name.clone()
        };
        let vpn_id_clone = vpn_id.clone();
        let (vpn_pill, vpn_sub) = Self::create_split_pill(
            "network-vpn-symbolic",
            "VPN & WG",
            if state.network.vpn_active { &vpn_id } else { "Disconnected" },
            if state.network.vpn_active { Some("active-secondary") } else { None },
            move |pill, sub, _icon| {
                let next = !is_vpn_clone.get();
                is_vpn_clone.set(next);
                if next {
                    NetworkService::connect_vpn(&vpn_id_clone);
                    pill.add_css_class("active-secondary");
                    sub.set_label(&vpn_id_clone);
                } else {
                    NetworkService::disconnect_vpn(&vpn_id_clone);
                    pill.remove_css_class("active-secondary");
                    sub.set_label("Disconnected");
                }
            },
            on_open_vpn,
        );

        // 7. Power Menu (Standard Pill)
        let (power_pill, _power_sub, _power_icon) = Self::create_pill(
            "system-shutdown-symbolic",
            "Power Menu",
            "Shutdown / Lock",
            None,
        );
        power_pill.connect_clicked(|_| {
            let _ = std::process::Command::new("/var/home/phamnv/.config/rofi/powermenu.sh").spawn();
        });

        // 8. Dark Style (Standard Pill)
        let (dark_pill, _dark_sub, _dark_icon) = Self::create_pill(
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

        Self {
            widget: grid,
            vpn_box: vpn_pill,
            vpn_sub,
            is_vpn,
            wifi_box: wifi_pill,
            wifi_sub,
            is_wifi: wifi_en,
            bt_box: bt_pill,
            bt_sub,
            is_bt: bt_en,
        }
    }

    pub fn update_network(&self, status: &NetworkStatus) {
        // 1. VPN
        self.is_vpn.set(status.vpn_active);
        if status.vpn_active {
            self.vpn_box.add_css_class("active-secondary");
            self.vpn_sub.set_label(&status.vpn_name);
        } else {
            self.vpn_box.remove_css_class("active-secondary");
            self.vpn_sub.set_label("Disconnected");
        }

        // 2. Wi-Fi
        self.is_wifi.set(status.wifi_enabled);
        if status.wifi_enabled && status.wifi_ssid != "Disconnected" {
            self.wifi_box.add_css_class("active-primary");
            self.wifi_sub.set_label(&status.wifi_ssid);
        } else {
            self.wifi_box.remove_css_class("active-primary");
            self.wifi_sub.set_label("Disconnected");
        }
    }

    fn create_split_pill<FT, FD>(
        icon: &str,
        title: &str,
        subtitle: &str,
        active_class: Option<&str>,
        on_toggle: FT,
        on_drilldown: FD,
    ) -> (gtk4::Box, gtk4::Label)
    where
        FT: Fn(&gtk4::Box, &gtk4::Label, &gtk4::Image) + 'static,
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

        let pill_box_clone = pill_box.clone();
        let sub_label_clone = sub_label.clone();
        let icon_img_clone = icon_img.clone();
        main_btn.connect_clicked(move |_| {
            on_toggle(&pill_box_clone, &sub_label_clone, &icon_img_clone);
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

        (pill_box, sub_label)
    }

    fn create_pill(
        icon: &str,
        title: &str,
        subtitle: &str,
        active_class: Option<&str>,
    ) -> (gtk4::Button, gtk4::Label, gtk4::Image) {
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
        (btn, sub_label, icon_img)
    }
}
