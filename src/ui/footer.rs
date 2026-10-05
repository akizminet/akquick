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

        // Left: Battery indicator (from UPower DBus)
        let left_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .hexpand(true)
            .halign(gtk4::Align::Start)
            .build();

        let bat_icon = gtk4::Image::builder()
            .icon_name(if state.battery.is_charging {
                "battery-level-80-charging-symbolic"
            } else {
                "battery-level-80-symbolic"
            })
            .pixel_size(16)
            .build();

        let bat_label = gtk4::Label::builder()
            .label(&format!("{}% ({})", state.battery.percentage, state.battery.time_remaining))
            .build();

        left_box.append(&bat_icon);
        left_box.append(&bat_label);

        // Right: Profile indicator
        let right_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk4::Align::End)
            .build();

        let profile_label = gtk4::Label::builder()
            .label(&format!("● Profile: {}", state.power_mode))
            .css_classes(["profile-badge"])
            .build();

        right_box.append(&profile_label);

        footer.append(&left_box);
        footer.append(&right_box);

        Self { widget: footer }
    }
}
