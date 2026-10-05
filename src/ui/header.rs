use gtk4::prelude::*;
use crate::services::ActionService;

pub struct HeaderBar {
    pub widget: gtk4::Box,
}

impl HeaderBar {
    pub fn new() -> Self {
        let container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .css_classes(["header-bar"])
            .build();

        // Left controls (Screenshot, Settings)
        let left_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .halign(gtk4::Align::Start)
            .build();

        let screenshot_btn = gtk4::Button::builder()
            .icon_name("camera-photo-symbolic")
            .tooltip_text("Take Screenshot")
            .css_classes(["header-button"])
            .build();
        screenshot_btn.connect_clicked(|_| {
            ActionService::take_screenshot();
        });

        let settings_btn = gtk4::Button::builder()
            .icon_name("emblem-system-symbolic")
            .tooltip_text("Settings & Control Center")
            .css_classes(["header-button"])
            .build();
        settings_btn.connect_clicked(|_| {
            ActionService::open_settings();
        });

        left_box.append(&screenshot_btn);
        left_box.append(&settings_btn);

        // Right controls (Lock, Power)
        let right_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::End)
            .build();

        let lock_btn = gtk4::Button::builder()
            .icon_name("system-lock-screen-symbolic")
            .tooltip_text("Lock Session")
            .css_classes(["header-button"])
            .build();
        lock_btn.connect_clicked(|_| {
            ActionService::lock_session();
        });

        let power_btn = gtk4::Button::builder()
            .icon_name("system-shutdown-symbolic")
            .tooltip_text("Power Menu")
            .css_classes(["header-button", "power-btn"])
            .build();
        power_btn.connect_clicked(|_| {
            ActionService::power_menu();
        });

        right_box.append(&lock_btn);
        right_box.append(&power_btn);

        container.append(&left_box);
        container.append(&right_box);

        Self { widget: container }
    }
}
