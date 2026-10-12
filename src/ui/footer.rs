use gtk4::prelude::*;
use crate::services::{BatteryState, SystemState};

pub struct FooterBar {
    pub widget: gtk4::Box,
    bat_icon: gtk4::Image,
    bat_val: gtk4::Label,
    bat_rem: gtk4::Label,
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
            .icon_name(state.battery.icon_name())
            .pixel_size(18)
            .css_classes(["footer-bat-icon"])
            .build();

        let bat_val = gtk4::Label::builder()
            .css_classes(["footer-bat-pct"])
            .build();

        let bat_rem = gtk4::Label::builder()
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

        let footer_bar = Self {
            widget: footer,
            bat_icon,
            bat_val,
            bat_rem,
        };

        footer_bar.update(&state.battery, state.telemetry.cpu_percent);

        footer_bar
    }

    pub fn update(&self, battery: &BatteryState, cpu_percent: u32) {
        self.bat_icon.set_icon_name(Some(battery.icon_name()));

        if battery.is_present {
            self.bat_val.set_visible(true);
            self.bat_val.set_label(&format!("{}%", battery.percentage));

            let rem = if !battery.time_remaining.is_empty() {
                format!("({} • {}% CPU)", battery.time_remaining, cpu_percent)
            } else if battery.is_charging {
                format!("(Charging • {}% CPU)", cpu_percent)
            } else {
                format!("({}% CPU)", cpu_percent)
            };
            self.bat_rem.set_label(&rem);
        } else {
            self.bat_val.set_visible(true);
            self.bat_val.set_label("AC");
            self.bat_rem.set_label(&format!("({}% CPU)", cpu_percent));
        }
    }
}
