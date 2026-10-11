use gtk4::prelude::*;
use crate::services::{AudioService, AudioState, SystemState};

pub struct SlidersSection {
    pub widget: gtk4::Box,
    output_scale: gtk4::Scale,
    output_icon: gtk4::Image,
    output_pct: gtk4::Label,
    output_mute_btn: gtk4::Button,
    is_out_muted: std::rc::Rc<std::cell::Cell<bool>>,
    mic_scale: gtk4::Scale,
    mic_icon: gtk4::Image,
    mic_desc: gtk4::Label,
    mic_pct: gtk4::Label,
    mic_mute_btn: gtk4::Button,
    is_mic_muted: std::rc::Rc<std::cell::Cell<bool>>,
    has_input_device: std::rc::Rc<std::cell::Cell<bool>>,
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
        if state.audio.volume_muted {
            output_scale.add_css_class("is-muted");
        }

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
            } else if state.audio.volume_percent > 60 {
                "audio-volume-high-symbolic"
            } else if state.audio.volume_percent > 25 {
                "audio-volume-medium-symbolic"
            } else {
                "audio-volume-low-symbolic"
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

        let out_vol_str = format!("{}%", state.audio.volume_percent);
        let output_pct = gtk4::Label::builder()
            .label(if state.audio.volume_muted {
                "Muted"
            } else {
                &out_vol_str
            })
            .halign(gtk4::Align::End)
            .css_classes(["capsule-percentage"])
            .build();
        if state.audio.volume_muted {
            output_pct.add_css_class("is-muted");
        }

        let is_out_muted = std::rc::Rc::new(std::cell::Cell::new(state.audio.volume_muted));

        // Circular Output Mute Button (HTML lines 173-176)
        let output_mute_btn = gtk4::Button::builder()
            .icon_name(if state.audio.volume_muted {
                "audio-volume-muted-symbolic"
            } else {
                "audio-volume-high-symbolic"
            })
            .tooltip_text(if state.audio.volume_muted {
                "Unmute Output"
            } else {
                "Mute Output"
            })
            .css_classes(["quick-mute-btn"])
            .build();
        if state.audio.volume_muted {
            output_mute_btn.add_css_class("is-muted");
        }

        let is_out_muted_slider = is_out_muted.clone();
        let pct_clone = output_pct.clone();
        let output_mute_btn_scale_clone = output_mute_btn.clone();
        let output_scale_slider_clone = output_scale.clone();
        let output_icon_slider_clone = output_icon.clone();
        output_scale.connect_value_changed(move |s| {
            let val = s.value();
            let pct = val.round() as u32;
            pct_clone.set_label(&format!("{}%", pct));
            AudioService::set_volume(pct);
            if is_out_muted_slider.get() && pct > 0 {
                is_out_muted_slider.set(false);
                output_scale_slider_clone.remove_css_class("is-muted");
                pct_clone.remove_css_class("is-muted");
                output_mute_btn_scale_clone.remove_css_class("is-muted");
                output_mute_btn_scale_clone.set_icon_name("audio-volume-high-symbolic");
                output_mute_btn_scale_clone.set_tooltip_text(Some("Mute Output"));
                output_icon_slider_clone.set_icon_name(Some("audio-volume-high-symbolic"));
            }
        });

        output_overlay.set_child(Some(&output_scale));
        output_overlay.add_overlay(&output_overlay_content);

        let is_out_muted_clone = is_out_muted.clone();
        let output_mute_btn_clone = output_mute_btn.clone();
        let output_icon_clone = output_icon.clone();
        let output_scale_clone = output_scale.clone();
        let output_pct_clone = output_pct.clone();
        output_mute_btn.connect_clicked(move |_| {
            let next_val = !is_out_muted_clone.get();
            is_out_muted_clone.set(next_val);
            AudioService::toggle_volume_mute();
            if next_val {
                output_mute_btn_clone.add_css_class("is-muted");
                output_scale_clone.add_css_class("is-muted");
                output_pct_clone.add_css_class("is-muted");
                output_pct_clone.set_label("Muted");
                output_mute_btn_clone.set_icon_name("audio-volume-muted-symbolic");
                output_mute_btn_clone.set_tooltip_text(Some("Unmute Output"));
                output_icon_clone.set_icon_name(Some("audio-volume-muted-symbolic"));
            } else {
                output_mute_btn_clone.remove_css_class("is-muted");
                output_scale_clone.remove_css_class("is-muted");
                output_pct_clone.remove_css_class("is-muted");
                let val = output_scale_clone.value().round() as u32;
                output_pct_clone.set_label(&format!("{}%", val));
                output_mute_btn_clone.set_icon_name("audio-volume-high-symbolic");
                output_mute_btn_clone.set_tooltip_text(Some("Mute Output"));
                let icon_name = if val > 60 {
                    "audio-volume-high-symbolic"
                } else if val > 25 {
                    "audio-volume-medium-symbolic"
                } else {
                    "audio-volume-low-symbolic"
                };
                output_icon_clone.set_icon_name(Some(icon_name));
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
        if state.audio.mic_muted || !state.audio.has_input_device {
            mic_scale.add_css_class("is-muted");
        }

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
            .icon_name(if !state.audio.has_input_device {
                "microphone-disabled-symbolic"
            } else if state.audio.mic_muted {
                "microphone-sensitivity-muted-symbolic"
            } else {
                "audio-input-microphone-symbolic"
            })
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

        let mic_vol_str = format!("{}%", state.audio.mic_percent);
        let mic_pct = gtk4::Label::builder()
            .label(if !state.audio.has_input_device {
                "0%"
            } else if state.audio.mic_muted {
                "Muted"
            } else {
                &mic_vol_str
            })
            .halign(gtk4::Align::End)
            .css_classes(["capsule-percentage", "secondary-text"])
            .opacity(if state.audio.has_input_device { 1.0 } else { 0.4 })
            .build();
        if state.audio.mic_muted || !state.audio.has_input_device {
            mic_pct.add_css_class("is-muted");
        }

        let is_mic_muted = std::rc::Rc::new(std::cell::Cell::new(state.audio.mic_muted));
        let has_input_device = std::rc::Rc::new(std::cell::Cell::new(state.audio.has_input_device));

        let mic_pct_clone = mic_pct.clone();
        let mic_scale_slider_clone = mic_scale.clone();
        let mic_icon_slider_clone = mic_icon.clone();
        let is_mic_muted_scale = is_mic_muted.clone();
        mic_scale.connect_value_changed(move |s| {
            let val = s.value();
            let pct = val.round() as u32;
            mic_pct_clone.set_label(&format!("{}%", pct));
            AudioService::set_mic_volume(pct);
            if is_mic_muted_scale.get() && pct > 0 {
                is_mic_muted_scale.set(false);
                mic_scale_slider_clone.remove_css_class("is-muted");
                mic_pct_clone.remove_css_class("is-muted");
                mic_icon_slider_clone.set_icon_name(Some("audio-input-microphone-symbolic"));
            }
        });

        mic_overlay.set_child(Some(&mic_scale));
        mic_overlay.add_overlay(&mic_overlay_content);

        // Separate Circular Quick Mute Button (HTML line 204)
        let mute_btn = gtk4::Button::builder()
            .icon_name(if !state.audio.has_input_device {
                "microphone-disabled-symbolic"
            } else if state.audio.mic_muted {
                "microphone-sensitivity-muted-symbolic"
            } else {
                "audio-input-microphone-symbolic"
            })
            .tooltip_text(if !state.audio.has_input_device {
                "No Microphone"
            } else if state.audio.mic_muted {
                "Unmute Mic"
            } else {
                "Quick Mute Mic"
            })
            .css_classes(["quick-mute-btn"])
            .sensitive(state.audio.has_input_device)
            .build();
        if state.audio.mic_muted {
            mute_btn.add_css_class("is-muted");
        }

        let is_mic_muted_clone = is_mic_muted.clone();
        let mute_btn_clone = mute_btn.clone();
        let mic_icon_clone = mic_icon.clone();
        let mic_scale_clone = mic_scale.clone();
        let mic_pct_btn_clone = mic_pct.clone();
        mute_btn.connect_clicked(move |_| {
            let next_val = !is_mic_muted_clone.get();
            is_mic_muted_clone.set(next_val);
            AudioService::toggle_mic_mute();
            if next_val {
                mute_btn_clone.add_css_class("is-muted");
                mic_scale_clone.add_css_class("is-muted");
                mic_pct_btn_clone.add_css_class("is-muted");
                mic_pct_btn_clone.set_label("Muted");
                mute_btn_clone.set_icon_name("microphone-sensitivity-muted-symbolic");
                mute_btn_clone.set_tooltip_text(Some("Unmute Mic"));
                mic_icon_clone.set_icon_name(Some("microphone-sensitivity-muted-symbolic"));
            } else {
                mute_btn_clone.remove_css_class("is-muted");
                mic_scale_clone.remove_css_class("is-muted");
                mic_pct_btn_clone.remove_css_class("is-muted");
                let val = mic_scale_clone.value().round() as u32;
                mic_pct_btn_clone.set_label(&format!("{}%", val));
                mute_btn_clone.set_icon_name("audio-input-microphone-symbolic");
                mute_btn_clone.set_tooltip_text(Some("Quick Mute Mic"));
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

        Self {
            widget: tray,
            output_scale,
            output_icon,
            output_pct,
            output_mute_btn,
            is_out_muted,
            mic_scale,
            mic_icon,
            mic_desc,
            mic_pct,
            mic_mute_btn: mute_btn,
            is_mic_muted,
            has_input_device,
        }
    }

    pub fn update_audio(&self, audio: &AudioState) {
        self.is_out_muted.set(audio.volume_muted);
        self.output_scale.set_value(audio.volume_percent as f64);
        if audio.volume_muted {
            self.output_scale.add_css_class("is-muted");
            self.output_mute_btn.add_css_class("is-muted");
            self.output_pct.add_css_class("is-muted");
            self.output_pct.set_label("Muted");
            self.output_mute_btn.set_icon_name("audio-volume-muted-symbolic");
            self.output_mute_btn.set_tooltip_text(Some("Unmute Output"));
            self.output_icon.set_icon_name(Some("audio-volume-muted-symbolic"));
        } else {
            self.output_scale.remove_css_class("is-muted");
            self.output_mute_btn.remove_css_class("is-muted");
            self.output_pct.remove_css_class("is-muted");
            self.output_pct.set_label(&format!("{}%", audio.volume_percent));
            self.output_mute_btn.set_icon_name("audio-volume-high-symbolic");
            self.output_mute_btn.set_tooltip_text(Some("Mute Output"));
            let icon_name = if audio.volume_percent > 60 {
                "audio-volume-high-symbolic"
            } else if audio.volume_percent > 25 {
                "audio-volume-medium-symbolic"
            } else {
                "audio-volume-low-symbolic"
            };
            self.output_icon.set_icon_name(Some(icon_name));
        }

        self.is_mic_muted.set(audio.mic_muted);
        self.has_input_device.set(audio.has_input_device);
        self.mic_scale.set_sensitive(audio.has_input_device);
        self.mic_mute_btn.set_sensitive(audio.has_input_device);

        if !audio.has_input_device {
            self.mic_scale.add_css_class("is-muted");
            self.mic_mute_btn.remove_css_class("is-muted");
            self.mic_pct.add_css_class("is-muted");
            self.mic_pct.set_label("0%");
            self.mic_desc.set_label("No Microphone Detected");
            self.mic_icon.set_icon_name(Some("microphone-disabled-symbolic"));
            self.mic_mute_btn.set_icon_name("microphone-disabled-symbolic");
            self.mic_mute_btn.set_tooltip_text(Some("No Microphone"));
        } else {
            self.mic_scale.set_value(audio.mic_percent as f64);
            self.mic_desc.set_label(&audio.source_desc);
            if audio.mic_muted {
                self.mic_scale.add_css_class("is-muted");
                self.mic_mute_btn.add_css_class("is-muted");
                self.mic_pct.add_css_class("is-muted");
                self.mic_pct.set_label("Muted");
                self.mic_mute_btn.set_icon_name("microphone-sensitivity-muted-symbolic");
                self.mic_mute_btn.set_tooltip_text(Some("Unmute Mic"));
                self.mic_icon.set_icon_name(Some("microphone-sensitivity-muted-symbolic"));
            } else {
                self.mic_scale.remove_css_class("is-muted");
                self.mic_mute_btn.remove_css_class("is-muted");
                self.mic_pct.remove_css_class("is-muted");
                self.mic_pct.set_label(&format!("{}%", audio.mic_percent));
                self.mic_mute_btn.set_icon_name("audio-input-microphone-symbolic");
                self.mic_mute_btn.set_tooltip_text(Some("Quick Mute Mic"));
                self.mic_icon.set_icon_name(Some("audio-input-microphone-symbolic"));
            }
        }
    }
}
