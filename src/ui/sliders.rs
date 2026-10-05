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
        // 1. Sound Output Capsule Slider + Quick Mute Button (HTML Lines 149-180)
        // =========================================================================
        let output_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .build();

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

        // Sound Output Row: Capsule Slider + Circular Mute Button
        let output_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .valign(gtk4::Align::Center)
            .build();

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
            .can_target(false)
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

        // Circular Output Mute Button (HTML lines 173-176)
        let is_out_muted = std::rc::Rc::new(std::cell::Cell::new(state.audio.volume_muted));
        let output_mute_btn = gtk4::Button::builder()
            .icon_name(if state.audio.volume_muted {
                "audio-volume-muted-symbolic"
            } else {
                "audio-volume-high-symbolic"
            })
            .tooltip_text("Mute Output")
            .css_classes(["quick-mute-btn"])
            .build();

        let is_out_muted_clone = is_out_muted.clone();
        let output_mute_btn_clone = output_mute_btn.clone();
        let output_icon_clone = output_icon.clone();
        output_mute_btn.connect_clicked(move |_| {
            let next_val = !is_out_muted_clone.get();
            is_out_muted_clone.set(next_val);
            AudioService::toggle_volume_mute();
            if next_val {
                output_mute_btn_clone.set_icon_name("audio-volume-muted-symbolic");
                output_icon_clone.set_icon_name(Some("audio-volume-muted-symbolic"));
            } else {
                output_mute_btn_clone.set_icon_name("audio-volume-high-symbolic");
                output_icon_clone.set_icon_name(Some("audio-volume-high-symbolic"));
            }
        });

        output_row.append(&output_overlay);
        output_row.append(&output_mute_btn);

        output_box.append(&output_header);
        output_box.append(&output_row);

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

        // Mic Row: Capsule Slider + Quick Mute Button
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
        let is_mic_muted = std::rc::Rc::new(std::cell::Cell::new(state.audio.mic_muted));
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

        let is_mic_muted_clone = is_mic_muted.clone();
        let mute_btn_clone = mute_btn.clone();
        let mic_icon_clone = mic_icon.clone();
        mute_btn.connect_clicked(move |_| {
            let next_val = !is_mic_muted_clone.get();
            is_mic_muted_clone.set(next_val);
            AudioService::toggle_mic_mute();
            if next_val {
                mute_btn_clone.set_icon_name("microphone-sensitivity-muted-symbolic");
                mic_icon_clone.set_icon_name(Some("microphone-sensitivity-muted-symbolic"));
            } else {
                mute_btn_clone.set_icon_name("audio-input-microphone-symbolic");
                mic_icon_clone.set_icon_name(Some("audio-input-microphone-symbolic"));
            }
        });

        mic_row.append(&mic_overlay);
        mic_row.append(&mute_btn);

        mic_box.append(&mic_header);
        mic_box.append(&mic_row);

        // =========================================================================
        // 3. Display Brightness Capsule Sliders (Dual 2K Monitors)
        // =========================================================================
        let display_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(8)
            .margin_top(4)
            .build();

        for disp in &state.displays {
            let display_overlay = gtk4::Overlay::builder()
                .hexpand(true)
                .build();

            let display_scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
            display_scale.set_value(disp.brightness_percent as f64);
            display_scale.set_hexpand(true);
            display_scale.set_draw_value(false);
            display_scale.add_css_class("capsule-slider");
            display_scale.add_css_class("brightness-capsule");

            let display_overlay_content = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .hexpand(true)
                .valign(gtk4::Align::Center)
                .css_classes(["capsule-overlay-content"])
                .can_target(false)
                .build();

            let display_left = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(8)
                .hexpand(true)
                .halign(gtk4::Align::Start)
                .build();

            let display_icon = gtk4::Image::builder()
                .icon_name("display-brightness-symbolic")
                .pixel_size(18)
                .build();

            let display_desc = gtk4::Label::builder()
                .label(&disp.label)
                .css_classes(["capsule-label"])
                .build();

            display_left.append(&display_icon);
            display_left.append(&display_desc);

            let display_pct = gtk4::Label::builder()
                .label(&format!("{}%", disp.brightness_percent))
                .halign(gtk4::Align::End)
                .css_classes(["capsule-percentage"])
                .build();

            display_overlay_content.append(&display_left);
            display_overlay_content.append(&display_pct);

            let disp_pct_clone = display_pct.clone();
            display_scale.connect_value_changed(move |s| {
                let val = s.value();
                disp_pct_clone.set_label(&format!("{}%", val.round() as u32));
            });

            display_overlay.set_child(Some(&display_scale));
            display_overlay.add_overlay(&display_overlay_content);

            display_box.append(&display_overlay);
        }

        tray.append(&output_box);
        tray.append(&mic_box);
        tray.append(&display_box);

        Self { widget: tray }
    }
}
