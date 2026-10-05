use gtk4::prelude::*;
use crate::services::SystemState;

pub struct FooterBar {
    pub widget: gtk4::Box,
}

impl FooterBar {
    pub fn new(state: &SystemState) -> Self {
        let footer = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .hexpand(true)
            .css_classes(["footer-bar"])
            .build();

        // Left: Battery / Power Status
        let left_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .hexpand(true)
            .halign(gtk4::Align::Start)
            .valign(gtk4::Align::Center)
            .build();

        let bat_icon = gtk4::Image::builder()
            .icon_name("battery-charging-symbolic")
            .pixel_size(18)
            .css_classes(["footer-bat-icon"])
            .build();

        let bat_val = gtk4::Label::builder()
            .label("88%")
            .css_classes(["footer-bat-pct"])
            .build();

        let bat_rem = gtk4::Label::builder()
            .label(&format!("(5h 20m rem. • {}% CPU)", state.telemetry.cpu_percent))
            .css_classes(["footer-bat-rem"])
            .build();

        left_box.append(&bat_icon);
        left_box.append(&bat_val);
        left_box.append(&bat_rem);

        // Right: Profile: Balanced Chip
        let profile_chip = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk4::Align::End)
            .valign(gtk4::Align::Center)
            .css_classes(["footer-profile-chip"])
            .build();

        let dot = gtk4::Box::builder()
            .css_classes(["profile-dot"])
            .build();

        let profile_lbl = gtk4::Label::builder()
            .label("Profile: Balanced")
            .css_classes(["footer-profile-text"])
            .build();

        profile_chip.append(&dot);
        profile_chip.append(&profile_lbl);

        footer.append(&left_box);
        footer.append(&profile_chip);

        Self { widget: footer }
    }
}
