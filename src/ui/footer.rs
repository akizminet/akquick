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

        // Left: Real Desktop CPU & RAM telemetry
        let left_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .halign(gtk4::Align::Start)
            .build();

        let cpu_label = gtk4::Label::builder()
            .label(&format!("💻 CPU: {}%", state.telemetry.cpu_percent))
            .build();

        let ram_label = gtk4::Label::builder()
            .label(&format!("🧠 RAM: {:.1} GiB / {:.1} GiB", state.telemetry.ram_used_gib, state.telemetry.ram_total_gib))
            .build();

        left_box.append(&cpu_label);
        left_box.append(&ram_label);

        // Right: Desktop Hostname
        let right_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .halign(gtk4::Align::End)
            .build();

        let host_label = gtk4::Label::builder()
            .label(&format!("● {}", state.telemetry.hostname))
            .css_classes(["profile-badge"])
            .build();

        right_box.append(&host_label);

        footer.append(&left_box);
        footer.append(&right_box);

        Self { widget: footer }
    }
}
