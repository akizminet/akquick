use gtk4::prelude::*;
use crate::services::{MprisService, SystemState};

pub struct MprisCard {
    pub widget: gtk4::Box,
}

impl MprisCard {
    pub fn new(state: &SystemState) -> Self {
        let card = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(10)
            .css_classes(["mpris-card"])
            .build();

        // Top Row: Album Art + Track Info + Playback Buttons
        let top_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(12)
            .build();

        // Album Art Icon
        let art = gtk4::Image::builder()
            .icon_name("audio-x-generic-symbolic")
            .pixel_size(44)
            .css_classes(["mpris-art"])
            .opacity(if state.mpris.has_player { 1.0 } else { 0.4 })
            .build();

        // Track Info
        let info_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .valign(gtk4::Align::Center)
            .build();

        let title_lbl = gtk4::Label::builder()
            .label(if state.mpris.has_player { &state.mpris.title } else { "Solaris" })
            .halign(gtk4::Align::Start)
            .css_classes(["mpris-title"])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .opacity(if state.mpris.has_player { 1.0 } else { 0.7 })
            .build();

        let artist_lbl = gtk4::Label::builder()
            .label(if state.mpris.has_player { &state.mpris.artist } else { "Carbon Based Lifeforms" })
            .halign(gtk4::Align::Start)
            .css_classes(["mpris-artist"])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .opacity(if state.mpris.has_player { 0.85 } else { 0.5 })
            .build();

        let source_text = if state.mpris.has_player {
            format!("● {}", state.mpris.player_name)
        } else {
            "● SPOTIFY WAYLAND MPRIS".to_string()
        };
        let source_lbl = gtk4::Label::builder()
            .label(&source_text)
            .halign(gtk4::Align::Start)
            .css_classes(["mpris-source"])
            .opacity(if state.mpris.has_player { 1.0 } else { 0.6 })
            .build();

        info_box.append(&title_lbl);
        info_box.append(&artist_lbl);
        info_box.append(&source_lbl);

        // Control Buttons
        let controls = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .valign(gtk4::Align::Center)
            .build();

        let prev_btn = gtk4::Button::builder()
            .icon_name("media-skip-backward-symbolic")
            .css_classes(["mpris-btn"])
            .sensitive(state.mpris.has_player)
            .build();
        prev_btn.connect_clicked(|_| {
            MprisService::previous();
        });

        let play_btn = gtk4::Button::builder()
            .icon_name(if state.mpris.is_playing { "media-playback-pause-symbolic" } else { "media-playback-start-symbolic" })
            .css_classes(["mpris-btn", "play-pause-btn"])
            .sensitive(state.mpris.has_player)
            .build();
        play_btn.connect_clicked(|_| {
            MprisService::play_pause();
        });

        let next_btn = gtk4::Button::builder()
            .icon_name("media-skip-forward-symbolic")
            .css_classes(["mpris-btn"])
            .sensitive(state.mpris.has_player)
            .build();
        next_btn.connect_clicked(|_| {
            MprisService::next();
        });

        controls.append(&prev_btn);
        controls.append(&play_btn);
        controls.append(&next_btn);

        top_row.append(&art);
        top_row.append(&info_box);
        top_row.append(&controls);

        card.append(&top_row);

        // Scrub Track Progress (HTML Lines 240-246)
        let progress_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .valign(gtk4::Align::Center)
            .margin_top(2)
            .build();

        let time_cur = gtk4::Label::builder()
            .label(if state.mpris.has_player { "03:42" } else { "03:42" })
            .css_classes(["mpris-timestamp"])
            .opacity(if state.mpris.has_player { 1.0 } else { 0.6 })
            .build();

        let progress_bar = gtk4::ProgressBar::builder()
            .fraction(if state.mpris.has_player { 0.58 } else { 0.58 })
            .hexpand(true)
            .css_classes(["mpris-progressbar"])
            .opacity(if state.mpris.has_player { 1.0 } else { 0.6 })
            .build();

        let time_total = gtk4::Label::builder()
            .label(if state.mpris.has_player { "06:21" } else { "06:21" })
            .css_classes(["mpris-timestamp"])
            .opacity(if state.mpris.has_player { 1.0 } else { 0.6 })
            .build();

        progress_box.append(&time_cur);
        progress_box.append(&progress_bar);
        progress_box.append(&time_total);

        card.append(&progress_box);

        Self { widget: card }
    }
}
