use gtk4::prelude::*;
use crate::services::SystemState;

pub struct SlidersSection {
    pub widget: gtk4::Box,
}

impl SlidersSection {
    pub fn new(state: &SystemState) -> Self {
        let container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(10)
            .build();

        // 1. Sound Output
        let sound_box = Self::create_slider_row(
            "Sound Output",
            "Focusrite DAC",
            if state.volume_muted { "audio-volume-muted-symbolic" } else { "audio-volume-high-symbolic" },
            "Analog 1/2",
            state.volume_percent,
            None,
            Some(|| {
                SystemState::toggle_volume_mute();
            }),
            |val| {
                SystemState::set_volume(val as u32);
            },
        );

        // 2. Input Level
        let mic_box = Self::create_slider_row(
            "Input Level",
            "Shure MV7 USB",
            if state.mic_muted { "microphone-sensitivity-muted-symbolic" } else { "audio-input-microphone-symbolic" },
            "Gain Boost",
            state.mic_percent,
            Some("secondary-scale"),
            Some(|| {
                SystemState::toggle_mic_mute();
            }),
            |val| {
                SystemState::set_mic_volume(val as u32);
            },
        );

        // 3. Brightness
        let bright_box = Self::create_slider_row(
            "Display Brightness",
            "Studio Display 5K",
            "display-brightness-symbolic",
            "Studio Display 5K",
            state.brightness_percent,
            None,
            None::<fn()>,
            |val| {
                SystemState::set_brightness(val as u32);
            },
        );

        container.append(&sound_box);
        container.append(&mic_box);
        container.append(&bright_box);

        Self { widget: container }
    }

    fn create_slider_row<F, M>(
        category: &str,
        device_name: &str,
        icon: &str,
        slider_label: &str,
        current_val: u32,
        scale_class: Option<&str>,
        on_icon_click: Option<M>,
        on_change: F,
    ) -> gtk4::Box
    where
        F: Fn(f64) + 'static,
        M: Fn() + 'static,
    {
        let card = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(6)
            .css_classes(["slider-group"])
            .build();

        // Header: Category (Left) and Device Name (Right)
        let top_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .hexpand(true)
            .build();

        let cat_label = gtk4::Label::builder()
            .label(category)
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .css_classes(["slider-label"])
            .build();

        let dev_label = gtk4::Label::builder()
            .label(device_name)
            .halign(gtk4::Align::End)
            .css_classes(["slider-label"])
            .build();

        top_row.append(&cat_label);
        top_row.append(&dev_label);

        // Slider Row: Icon (Button if clickable) + Label + Scale + Percentage
        let slider_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(10)
            .valign(gtk4::Align::Center)
            .build();

        let icon_img = gtk4::Image::builder()
            .icon_name(icon)
            .pixel_size(18)
            .opacity(0.85)
            .build();

        if let Some(on_click) = on_icon_click {
            let icon_btn = gtk4::Button::builder()
                .child(&icon_img)
                .has_frame(false)
                .css_classes(["mpris-btn"])
                .build();
            icon_btn.connect_clicked(move |_| {
                on_click();
            });
            slider_row.append(&icon_btn);
        } else {
            slider_row.append(&icon_img);
        }

        let name_lbl = gtk4::Label::builder()
            .label(slider_label)
            .css_classes(["toggle-subtitle"])
            .build();

        let scale = gtk4::Scale::with_range(gtk4::Orientation::Horizontal, 0.0, 100.0, 1.0);
        scale.set_value(current_val as f64);
        scale.set_hexpand(true);
        scale.set_draw_value(false);

        if let Some(cls) = scale_class {
            scale.add_css_class(cls);
        }

        let val_label = gtk4::Label::builder()
            .label(&format!("{}%", current_val))
            .css_classes(["slider-value"])
            .build();

        let val_label_clone = val_label.clone();
        scale.connect_value_changed(move |s| {
            let val = s.value();
            val_label_clone.set_label(&format!("{}%", val.round() as u32));
            on_change(val);
        });

        slider_row.append(&name_lbl);
        slider_row.append(&scale);
        slider_row.append(&val_label);

        card.append(&top_row);
        card.append(&slider_row);

        card
    }
}
