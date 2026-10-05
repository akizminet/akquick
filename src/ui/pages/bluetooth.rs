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

        // 1. Header with Back Button and Master Switch
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
            .label(&format!("Adapter: {}", state.bluetooth.adapter_name))
            .halign(gtk4::Align::Start)
            .css_classes(["subpage-subtitle"])
            .build();

        title_box.append(&title_lbl);
        title_box.append(&sub_lbl);

        let master_switch = gtk4::Switch::builder()
            .active(state.bluetooth.enabled)
            .valign(gtk4::Align::Center)
            .build();
        master_switch.connect_active_notify(|s| {
            BluetoothService::set_enabled(s.is_active());
        });

        header.append(&back_btn);
        header.append(&title_box);
        header.append(&master_switch);

        container.append(&header);

        // 2. Paired & Known Devices List
        let list_header = gtk4::Label::builder()
            .label("PAIRED DEVICES")
            .halign(gtk4::Align::Start)
            .css_classes(["subpage-badge"])
            .build();
        container.append(&list_header);

        let list_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .css_classes(["subpage-list"])
            .build();

        if state.bluetooth.devices.is_empty() {
            let empty_lbl = gtk4::Label::builder()
                .label("No paired devices found")
                .css_classes(["subpage-subtitle"])
                .margin_top(12)
                .margin_bottom(12)
                .build();
            list_box.append(&empty_lbl);
        } else {
            for dev in &state.bluetooth.devices {
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
                    .label(if dev.connected { "Connected" } else { "Paired" })
                    .halign(gtk4::Align::Start)
                    .css_classes(["subpage-subtitle"])
                    .build();

                dev_box.append(&dev_name);
                dev_box.append(&dev_status);

                let action_btn = gtk4::Button::builder()
                    .label(if dev.connected { "Disconnect" } else { "Connect" })
                    .css_classes([
                        "subpage-action-btn",
                        if dev.connected { "secondary" } else { "primary" },
                    ])
                    .build();

                let addr = dev.address.clone();
                let is_connected = dev.connected;
                action_btn.connect_clicked(move |_| {
                    if is_connected {
                        BluetoothService::disconnect_device(&addr);
                    } else {
                        BluetoothService::connect_device(&addr);
                    }
                });

                row.append(&icon);
                row.append(&dev_box);
                row.append(&action_btn);

                list_box.append(&row);
            }
        }

        container.append(&list_box);

        // 3. Footer Settings Button
        let footer_btn = gtk4::Button::builder()
            .label("Bluetooth Settings…")
            .css_classes(["subpage-action-btn", "secondary"])
            .halign(gtk4::Align::Start)
            .build();
        footer_btn.connect_clicked(|_| {
            let _ = std::process::Command::new("gnome-control-center").arg("bluetooth").spawn();
        });
        container.append(&footer_btn);

        Self { widget: container }
    }
}
