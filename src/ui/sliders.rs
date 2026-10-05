use gtk4::prelude::*;
use crate::services::{AudioService, SystemState};

pub struct SlidersSection {
    pub widget: gtk4::Box,
}

impl SlidersSection {
    pub fn new(state: &SystemState) -> Self {
        let tray = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(12)
            .css_classes(["capsule-tray"])
            .build();

        // =========================================================================
        // 1. Output Volume Continuous Capsule Slider (HTML Lines 158-180)
        // =========================================================================
        let output_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .build();

        // Header: "Sound Output" (Left) and Device Switcher Chip (Right)
        let output_header = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .hexpand(true)
            .build();

        let output_title = gtk4::Label::builder()
            .label("Sound Output")
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .css_classes(["toggle-title"])
            .build();

        let output_chip = gtk4::Button::builder()
            .css_classes(["device-chip"])
            .build();

        let chip_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .valign(gtk4::Align::Center)
            .build();

        let chip_label = gtk4::Label::builder()
            .label(&state.audio.sink_name)
            .build();

        let chip_arrow = gtk4::Image::builder()
            .icon_name("pan-down-symbolic")
            .pixel_size(10)
            .build();

        chip_box.append(&chip_label);
        chip_box.append(&chip_arrow);
        output_chip.set_child(Some(&chip_box));

        output_header.append(&output_title);
        output_header.append(&output_chip);

        // Continuous Capsule Slider with internal overlay
        let output_overlay = gtk4::Overlay::builder()
            .hexpand(true)
            .build();

        let output_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        output_scale.set_value(state.audio.volume_percent as f64);
        output_scale.set_hexpand(true);
        output_scale.set_draw_value(false);
        output_scale.add_css_class("capsule-slider");

        let output_overlay_content = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .hexpand(true)
            .valign(gtk4::Align::Center)
            .css_classes(["capsule-overlay-content"])
            .can_target(false) // clicks pass through to the scale underneath!
            .build();

        let output_left = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .halign(gtk4::Align::Start)
            .build();

        let output_icon = gtk4::Image::builder()
            .icon_name(if state.audio.volume_muted {
                "audio-volume-muted-symbolic"
            } else {
                "audio-volume-high-symbolic"
            })
            .pixel_size(18)
            .build();

        let output_desc = gtk4::Label::builder()
            .label(&state.audio.sink_desc)
            .css_classes(["capsule-label"])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .max_width_chars(20)
            .build();

        output_left.append(&output_icon);
        output_left.append(&output_desc);

        let output_pct = gtk4::Label::builder()
            .label(&format!("{}%", state.audio.volume_percent))
            .halign(gtk4::Align::End)
            .css_classes(["capsule-percentage"])
            .build();

        output_overlay_content.append(&output_left);
        output_overlay_content.append(&output_pct);

        let pct_clone = output_pct.clone();
        output_scale.connect_value_changed(move |s| {
            let val = s.value();
            pct_clone.set_label(&format!("{}%", val.round() as u32));
            AudioService::set_volume(val as u32);
        });

        output_overlay.set_child(Some(&output_scale));
        output_overlay.add_overlay(&output_overlay_content);

        output_box.append(&output_header);
        output_box.append(&output_overlay);

        // =========================================================================
        // 2. Mic Input Live VU Bar Capsule Slider (HTML Lines 181-208)
        // =========================================================================
        let mic_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .build();

        let mic_header = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .hexpand(true)
            .build();

        let mic_title = gtk4::Label::builder()
            .label("Input Level")
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .css_classes(["toggle-title"])
            .build();

        let mic_chip = gtk4::Button::builder()
            .css_classes(["device-chip", "secondary-chip"])
            .build();

        let mic_chip_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .valign(gtk4::Align::Center)
            .build();

        let mic_chip_label = gtk4::Label::builder()
            .label(if state.audio.has_input_device {
                &state.audio.source_name
            } else {
                "Disconnected"
            })
            .build();

        let mic_chip_icon = gtk4::Image::builder()
            .icon_name("audio-input-microphone-symbolic")
            .pixel_size(12)
            .build();

        mic_chip_box.append(&mic_chip_label);
        mic_chip_box.append(&mic_chip_icon);
        mic_chip.set_child(Some(&mic_chip_box));

        mic_header.append(&mic_title);
        mic_header.append(&mic_chip);

        // Capsule Slider + Quick Mute Button Row
        let mic_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .valign(gtk4::Align::Center)
            .build();

        let mic_overlay = gtk4::Overlay::builder()
            .hexpand(true)
            .build();

        let mic_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        mic_scale.set_value(state.audio.mic_percent as f64);
        mic_scale.set_hexpand(true);
        mic_scale.set_draw_value(false);
        mic_scale.add_css_class("capsule-slider");
        mic_scale.add_css_class("secondary-capsule");
        mic_scale.set_sensitive(state.audio.has_input_device);

        let mic_overlay_content = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .hexpand(true)
            .valign(gtk4::Align::Center)
            .css_classes(["capsule-overlay-content"])
            .can_target(false)
            .build();

        let mic_left = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .halign(gtk4::Align::Start)
            .build();

        let mic_icon = gtk4::Image::builder()
            .icon_name("audio-input-microphone-symbolic")
            .pixel_size(17)
            .build();

        let mic_desc = gtk4::Label::builder()
            .label(if state.audio.has_input_device {
                &state.audio.source_desc
            } else {
                "No Microphone Detected"
            })
            .css_classes(["capsule-label"])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .max_width_chars(18)
            .opacity(if state.audio.has_input_device { 1.0 } else { 0.5 })
            .build();

        mic_left.append(&mic_icon);
        mic_left.append(&mic_desc);

        let mic_pct = gtk4::Label::builder()
            .label(&format!("{}%", state.audio.mic_percent))
            .halign(gtk4::Align::End)
            .css_classes(["capsule-percentage", "secondary-text"])
            .opacity(if state.audio.has_input_device { 1.0 } else { 0.4 })
            .build();

        mic_overlay_content.append(&mic_left);
        mic_overlay_content.append(&mic_pct);

        let mic_pct_clone = mic_pct.clone();
        mic_scale.connect_value_changed(move |s| {
            let val = s.value();
            mic_pct_clone.set_label(&format!("{}%", val.round() as u32));
            AudioService::set_mic_volume(val as u32);
        });

        mic_overlay.set_child(Some(&mic_scale));
        mic_overlay.add_overlay(&mic_overlay_content);

        // Separate Circular Quick Mute Button (HTML line 204)
        let mute_btn = gtk4::Button::builder()
            .icon_name(if state.audio.mic_muted || !state.audio.has_input_device {
                "microphone-sensitivity-muted-symbolic"
            } else {
                "audio-input-microphone-symbolic"
            })
            .tooltip_text("Quick Mute Mic")
            .css_classes(["quick-mute-btn"])
            .sensitive(state.audio.has_input_device)
            .build();

        mute_btn.connect_clicked(|_| {
            AudioService::toggle_mic_mute();
        });

        mic_row.append(&mic_overlay);
        mic_row.append(&mute_btn);

        mic_box.append(&mic_header);
        mic_box.append(&mic_row);

        tray.append(&output_box);
        tray.append(&mic_box);

        Self { widget: tray }
    }
}
