use gtk4::prelude::*;
use crate::services::{MprisService, MprisState, SystemState};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub struct MprisCard {
    pub widget: gtk4::Box,
    pub title_lbl: gtk4::Label,
    pub artist_lbl: gtk4::Label,
    pub source_lbl: gtk4::Label,
    pub play_btn: gtk4::Button,
    pub prev_btn: gtk4::Button,
    pub next_btn: gtk4::Button,
    pub time_cur: gtk4::Label,
    pub time_total: gtk4::Label,
    pub progress_bar: gtk4::ProgressBar,
    pub art: gtk4::Image,
    pub active_bus: Rc<RefCell<Option<String>>>,
    pub is_playing: Rc<Cell<bool>>,
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
            .label(if state.mpris.has_player { &state.mpris.title } else { "No Media Playing" })
            .halign(gtk4::Align::Start)
            .css_classes(["mpris-title"])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .opacity(if state.mpris.has_player { 1.0 } else { 0.6 })
            .build();

        let artist_lbl = gtk4::Label::builder()
            .label(if state.mpris.has_player { &state.mpris.artist } else { "Audio Idle" })
            .halign(gtk4::Align::Start)
            .css_classes(["mpris-artist"])
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .opacity(if state.mpris.has_player { 0.85 } else { 0.4 })
            .build();

        let source_text = if state.mpris.has_player {
            format!("● {}", state.mpris.player_name)
        } else {
            "● MPRIS".to_string()
        };
        let source_lbl = gtk4::Label::builder()
            .label(&source_text)
            .halign(gtk4::Align::Start)
            .css_classes(["mpris-source"])
            .opacity(if state.mpris.has_player { 1.0 } else { 0.5 })
            .build();

        info_box.append(&title_lbl);
        info_box.append(&artist_lbl);
        info_box.append(&source_lbl);

        let active_bus = Rc::new(RefCell::new(state.mpris.active_bus_name.clone()));
        let is_playing = Rc::new(Cell::new(state.mpris.is_playing));

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
        let active_bus_prev = active_bus.clone();
        prev_btn.connect_clicked(move |_| {
            let bus = active_bus_prev.borrow().clone();
            MprisService::previous(bus);
        });

        let play_btn = gtk4::Button::builder()
            .icon_name(if state.mpris.is_playing {
                "media-playback-pause-symbolic"
            } else {
                "media-playback-start-symbolic"
            })
            .css_classes(["mpris-btn", "play-pause-btn"])
            .sensitive(state.mpris.has_player)
            .build();

        let is_playing_clone = is_playing.clone();
        let play_btn_clone = play_btn.clone();
        let active_bus_play = active_bus.clone();
        play_btn.connect_clicked(move |_| {
            let next_val = !is_playing_clone.get();
            is_playing_clone.set(next_val);
            if next_val {
                play_btn_clone.set_icon_name("media-playback-pause-symbolic");
            } else {
                play_btn_clone.set_icon_name("media-playback-start-symbolic");
            }
            let bus = active_bus_play.borrow().clone();
            MprisService::play_pause(bus);
        });

        let next_btn = gtk4::Button::builder()
            .icon_name("media-skip-forward-symbolic")
            .css_classes(["mpris-btn"])
            .sensitive(state.mpris.has_player)
            .build();
        let active_bus_next = active_bus.clone();
        next_btn.connect_clicked(move |_| {
            let bus = active_bus_next.borrow().clone();
            MprisService::next(bus);
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

        let cur_str = if state.mpris.has_player && state.mpris.position_sec > 0 {
            format!("{:02}:{:02}", state.mpris.position_sec / 60, state.mpris.position_sec % 60)
        } else {
            "00:00".to_string()
        };

        let tot_str = if state.mpris.has_player && state.mpris.length_sec > 0 {
            format!("{:02}:{:02}", state.mpris.length_sec / 60, state.mpris.length_sec % 60)
        } else {
            "00:00".to_string()
        };

        let initial_frac = if state.mpris.has_player && state.mpris.length_sec > 0 {
            (state.mpris.position_sec as f64 / state.mpris.length_sec as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let time_cur = gtk4::Label::builder()
            .label(&cur_str)
            .css_classes(["mpris-timestamp"])
            .opacity(if state.mpris.has_player { 1.0 } else { 0.6 })
            .build();

        let progress_bar = gtk4::ProgressBar::builder()
            .fraction(initial_frac)
            .hexpand(true)
            .css_classes(["mpris-progressbar"])
            .opacity(if state.mpris.has_player { 1.0 } else { 0.6 })
            .build();

        let time_total = gtk4::Label::builder()
            .label(&tot_str)
            .css_classes(["mpris-timestamp"])
            .opacity(if state.mpris.has_player { 1.0 } else { 0.6 })
            .build();

        progress_box.append(&time_cur);
        progress_box.append(&progress_bar);
        progress_box.append(&time_total);

        card.append(&progress_box);

        Self {
            widget: card,
            title_lbl,
            artist_lbl,
            source_lbl,
            play_btn,
            prev_btn,
            next_btn,
            time_cur,
            time_total,
            progress_bar,
            art,
            active_bus,
            is_playing,
        }
    }

    pub fn update(&self, state: &MprisState) {
        if state.has_player {
            self.title_lbl.set_label(&state.title);
            self.artist_lbl.set_label(&state.artist);
            self.source_lbl.set_label(&format!("● {}", state.player_name));
            self.title_lbl.set_opacity(1.0);
            self.artist_lbl.set_opacity(0.85);
            self.source_lbl.set_opacity(1.0);
            self.art.set_opacity(1.0);

            self.prev_btn.set_sensitive(true);
            self.play_btn.set_sensitive(true);
            self.next_btn.set_sensitive(true);

            self.is_playing.set(state.is_playing);
            if state.is_playing {
                self.play_btn.set_icon_name("media-playback-pause-symbolic");
            } else {
                self.play_btn.set_icon_name("media-playback-start-symbolic");
            }

            *self.active_bus.borrow_mut() = state.active_bus_name.clone();

            if state.length_sec > 0 {
                let cur_m = state.position_sec / 60;
                let cur_s = state.position_sec % 60;
                let tot_m = state.length_sec / 60;
                let tot_s = state.length_sec % 60;
                self.time_cur.set_label(&format!("{:02}:{:02}", cur_m, cur_s));
                self.time_total.set_label(&format!("{:02}:{:02}", tot_m, tot_s));
                let frac = (state.position_sec as f64 / state.length_sec as f64).clamp(0.0, 1.0);
                self.progress_bar.set_fraction(frac);
            } else if state.is_playing {
                let cur_m = state.position_sec / 60;
                let cur_s = state.position_sec % 60;
                self.time_cur.set_label(&format!("{:02}:{:02}", cur_m, cur_s));
                self.time_total.set_label("LIVE");
                self.progress_bar.set_fraction(1.0);
            }
            self.time_cur.set_opacity(1.0);
            self.time_total.set_opacity(1.0);
            self.progress_bar.set_opacity(1.0);
        } else {
            self.title_lbl.set_label("No Media Playing");
            self.artist_lbl.set_label("Audio Idle");
            self.source_lbl.set_label("● MPRIS");
            self.title_lbl.set_opacity(0.5);
            self.artist_lbl.set_opacity(0.4);
            self.source_lbl.set_opacity(0.5);
            self.art.set_opacity(0.4);

            self.play_btn.set_icon_name("media-playback-start-symbolic");
            self.prev_btn.set_sensitive(false);
            self.play_btn.set_sensitive(false);
            self.next_btn.set_sensitive(false);

            self.progress_bar.set_fraction(0.0);
            self.time_cur.set_label("00:00");
            self.time_total.set_label("00:00");
            self.time_cur.set_opacity(0.6);
            self.time_total.set_opacity(0.6);
            self.progress_bar.set_opacity(0.6);

            *self.active_bus.borrow_mut() = None;
            self.is_playing.set(false);
        }
    }
}
