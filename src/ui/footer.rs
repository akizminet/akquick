use gtk4::prelude::*;
use crate::services::SystemState;

pub struct FooterBar {
    pub widget: gtk4::Box,
    cpu_val: gtk4::Label,
    ram_val: gtk4::Label,
}

impl FooterBar {
    pub fn new(state: &SystemState) -> Self {
        let footer = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .hexpand(true)
            .css_classes(["footer-bar"])
            .build();

        // Left: CPU & RAM Hardware Monitor
        let left_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .hexpand(true)
            .halign(gtk4::Align::Start)
            .valign(gtk4::Align::Center)
            .build();

        let hw_icon = gtk4::Image::builder()
            .icon_name("applications-system-symbolic")
            .pixel_size(16)
            .css_classes(["footer-hw-icon"])
            .build();

        let cpu_val = gtk4::Label::builder()
            .css_classes(["footer-hw-text"])
            .build();

        let dot_sep = gtk4::Label::builder()
            .label("•")
            .css_classes(["footer-hw-sep"])
            .build();

        let ram_val = gtk4::Label::builder()
            .css_classes(["footer-hw-text"])
            .build();

        left_box.append(&hw_icon);
        left_box.append(&cpu_val);
        left_box.append(&dot_sep);
        left_box.append(&ram_val);

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
            cpu_val,
            ram_val,
        };

        footer_bar.update(state.telemetry.cpu_percent, state.telemetry.ram_used_gib, state.telemetry.ram_total_gib);

        footer_bar
    }

    pub fn update(&self, cpu_percent: u32, ram_used_gib: f32, ram_total_gib: f32) {
        self.cpu_val.set_label(&format!("CPU: {}%", cpu_percent));

        if ram_total_gib > 0.0 {
            let ram_pct = ((ram_used_gib / ram_total_gib) * 100.0).clamp(0.0, 100.0).round() as u32;
            self.ram_val.set_label(&format!("RAM: {:.1}/{:.1} GB ({}%)", ram_used_gib, ram_total_gib, ram_pct));
        } else {
            self.ram_val.set_label("RAM: --");
        }
    }
}
