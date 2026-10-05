use gtk4::prelude::*;
use crate::services::{BluetoothService, SystemState};

pub struct BluetoothPage {
    pub widget: gtk4::Box,
}

impl BluetoothPage {
    pub fn new<F>(state: &SystemState, on_back: F) -> Self
    where
        F: Fn() + 'static,
    {
        let container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(12)
            .css_classes(["subpage-container"])
            .build();

        // 1. Header with Back Button, Master Switch, and Broadcasting Badge
        let header = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(10)
            .css_classes(["subpage-header"])
            .build();

        let back_btn = gtk4::Button::builder()
            .icon_name("go-previous-symbolic")
            .css_classes(["subpage-back-btn"])
            .tooltip_text("Back to Quick Settings")
            .build();
        back_btn.connect_clicked(move |_| {
            on_back();
        });

        let title_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();

        let title_lbl = gtk4::Label::builder()
            .label("Bluetooth")
            .halign(gtk4::Align::Start)
            .css_classes(["subpage-title"])
            .build();

        let sub_lbl = gtk4::Label::builder()
            .label(&format!("● hci0 • {}", state.bluetooth.adapter_name))
            .halign(gtk4::Align::Start)
            .css_classes(["subpage-subtitle"])
            .build();

        title_box.append(&title_lbl);
        title_box.append(&sub_lbl);

        let bcast_chip = gtk4::Label::builder()
            .label("Broadcasting")
            .css_classes(["bt-broadcast-chip"])
            .valign(gtk4::Align::Center)
            .build();

        let master_switch = gtk4::Switch::builder()
            .active(state.bluetooth.enabled)
            .valign(gtk4::Align::Center)
            .build();
        master_switch.connect_active_notify(|s| {
            BluetoothService::set_enabled(s.is_active());
        });

        header.append(&back_btn);
        header.append(&title_box);
        if state.bluetooth.enabled {
            header.append(&bcast_chip);
        }
        header.append(&master_switch);

        container.append(&header);

        // 2. Connected Devices Section (if any connected)
        let (connected, paired): (Vec<_>, Vec<_>) = state.bluetooth.devices.iter().partition(|d| d.connected);

        if !connected.is_empty() {
            let conn_header = gtk4::Label::builder()
                .label(&format!("CONNECTED DEVICES ({})", connected.len()))
                .halign(gtk4::Align::Start)
                .css_classes(["subpage-badge"])
                .build();
            container.append(&conn_header);

            for dev in connected {
                let card = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Vertical)
                    .spacing(8)
                    .css_classes(["subpage-card"])
                    .build();

                let top_row = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Horizontal)
                    .spacing(10)
                    .valign(gtk4::Align::Center)
                    .build();

                let icon = gtk4::Image::builder()
                    .icon_name("audio-headset-symbolic")
                    .pixel_size(22)
                    .build();

                let dev_box = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Vertical)
                    .spacing(2)
                    .hexpand(true)
                    .build();

                let name_row = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Horizontal)
                    .spacing(6)
                    .build();

                let dev_name = gtk4::Label::builder()
                    .label(&dev.name)
                    .halign(gtk4::Align::Start)
                    .css_classes(["capsule-label"])
                    .build();

                let codec_chip = gtk4::Label::builder()
                    .label("LDAC")
                    .css_classes(["bt-codec-chip"])
                    .build();

                name_row.append(&dev_name);
                name_row.append(&codec_chip);

                let dev_desc = gtk4::Label::builder()
                    .label("Primary Audio Sink • 990 kbps")
                    .halign(gtk4::Align::Start)
                    .css_classes(["subpage-subtitle"])
                    .build();

                dev_box.append(&name_row);
                dev_box.append(&dev_desc);

                let bat_pill = gtk4::Label::builder()
                    .label("● 82%")
                    .css_classes(["bt-battery-pill"])
                    .build();

                let disconn_btn = gtk4::Button::builder()
                    .icon_name("system-shutdown-symbolic")
                    .tooltip_text("Disconnect")
                    .css_classes(["subpage-glyph-btn"])
                    .build();

                let addr = dev.address.clone();
                disconn_btn.connect_clicked(move |_| {
                    BluetoothService::disconnect_device(&addr);
                });

                top_row.append(&icon);
                top_row.append(&dev_box);
                top_row.append(&bat_pill);
                top_row.append(&disconn_btn);
                card.append(&top_row);

                // Embedded Telemetry Bar (48kHz / 24-bit Stream + Mini Wave)
                let telem_box = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Horizontal)
                    .spacing(8)
                    .css_classes(["bt-telem-box"])
                    .build();

                let telem_icon = gtk4::Image::builder()
                    .icon_name("audio-volume-high-symbolic")
                    .pixel_size(14)
                    .css_classes(["text-secondary"])
                    .build();

                let telem_lbl = gtk4::Label::builder()
                    .label("48kHz / 24-bit Stream")
                    .hexpand(true)
                    .halign(gtk4::Align::Start)
                    .css_classes(["subpage-subtitle"])
                    .build();

                let wave_bar = gtk4::ProgressBar::builder()
                    .fraction(0.72)
                    .width_request(80)
                    .css_classes(["bt-wave-bar"])
                    .build();

                telem_box.append(&telem_icon);
                telem_box.append(&telem_lbl);
                telem_box.append(&wave_bar);
                card.append(&telem_box);

                container.append(&card);
            }
        }

        // 3. Paired Devices List (HTML lines 165-195)
        let list_header_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .hexpand(true)
            .build();

        let list_header = gtk4::Label::builder()
            .label("PAIRED DEVICES")
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .css_classes(["subpage-badge"])
            .build();

        let saved_count = gtk4::Label::builder()
            .label(&format!("{} Saved", paired.len()))
            .halign(gtk4::Align::End)
            .css_classes(["subpage-subtitle"])
            .build();

        list_header_box.append(&list_header);
        list_header_box.append(&saved_count);
        container.append(&list_header_box);

        let list_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .css_classes(["subpage-list"])
            .build();

        if paired.is_empty() {
            let empty_lbl = gtk4::Label::builder()
                .label("No paired devices found")
                .css_classes(["subpage-subtitle"])
                .margin_top(12)
                .margin_bottom(12)
                .build();
            list_box.append(&empty_lbl);
        } else {
            for dev in paired {
                let row = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Horizontal)
                    .spacing(10)
                    .css_classes(["subpage-item"])
                    .build();

                let icon = gtk4::Image::builder()
                    .icon_name("audio-headset-symbolic")
                    .pixel_size(20)
                    .build();

                let dev_box = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Vertical)
                    .spacing(2)
                    .hexpand(true)
                    .build();

                let dev_name = gtk4::Label::builder()
                    .label(&dev.name)
                    .halign(gtk4::Align::Start)
                    .css_classes(["capsule-label"])
                    .build();

                let dev_status = gtk4::Label::builder()
                    .label("Last seen recently")
                    .halign(gtk4::Align::Start)
                    .css_classes(["subpage-subtitle"])
                    .build();

                dev_box.append(&dev_name);
                dev_box.append(&dev_status);

                let connect_btn = gtk4::Button::builder()
                    .label("Connect")
                    .css_classes(["subpage-action-btn", "primary"])
                    .build();

                let addr = dev.address.clone();
                connect_btn.connect_clicked(move |_| {
                    BluetoothService::connect_device(&addr);
                });

                row.append(&icon);
                row.append(&dev_box);
                row.append(&connect_btn);

                list_box.append(&row);
            }
        }

        container.append(&list_box);

        // 4. Footer Action Buttons: Settings + Pair New Device (HTML lines 205-215)
        let footer_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .build();

        let footer_btn = gtk4::Button::builder()
            .label("Bluetooth Settings…")
            .css_classes(["subpage-action-btn", "secondary"])
            .hexpand(true)
            .build();
        footer_btn.connect_clicked(|_| {
            let _ = std::process::Command::new("gnome-control-center").arg("bluetooth").spawn();
        });

        let pair_btn = gtk4::Button::builder()
            .label("+ Pair New Device")
            .css_classes(["subpage-action-btn", "primary"])
            .build();
        pair_btn.connect_clicked(|_| {
            let _ = std::process::Command::new("gnome-control-center").arg("bluetooth").spawn();
        });

        footer_row.append(&footer_btn);
        footer_row.append(&pair_btn);
        container.append(&footer_row);

        Self { widget: container }
    }
}
