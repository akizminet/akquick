use gtk4::prelude::*;
use gtk4::{gdk, gio};
use crate::ui::QuickSettingsWindow;
use std::cell::RefCell;
use std::rc::Rc;

pub const APP_ID: &str = "dev.akquick.ControlCenter";

pub struct App {
    pub app: libadwaita::Application,
}

impl App {
    pub fn new() -> Self {
        let app = libadwaita::Application::builder()
            .application_id(APP_ID)
            .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
            .build();

        let window_holder: Rc<RefCell<Option<QuickSettingsWindow>>> = Rc::new(RefCell::new(None));

        let wh_activate = window_holder.clone();
        app.connect_activate(move |app| {
            // Load CSS styling on display
            if let Some(display) = gdk::Display::default() {
                let provider = gtk4::CssProvider::new();
                provider.load_from_data(include_str!("../resources/style.css"));
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }

            let mut wh = wh_activate.borrow_mut();
            if wh.is_none() {
                let win = QuickSettingsWindow::new(app, false);
                win.window.present();
                *wh = Some(win);
            } else if let Some(ref win) = *wh {
                win.toggle();
            }
        });

        let wh_cmd = window_holder.clone();
        app.connect_command_line(move |app, cmd| {
            let args = cmd.arguments();
            let is_bench = args.iter().any(|arg| arg == "--bench" || arg == "-b");
            let is_daemon = args.iter().any(|arg| arg == "--daemon" || arg == "-d");
            let has_toggle = args.iter().any(|arg| arg == "--toggle" || arg == "-t");
            let page_target = args.windows(2).find_map(|w| {
                if w[0] == "--page" || w[0] == "-p" {
                    Some(w[1].to_str().unwrap_or("main").to_string())
                } else {
                    None
                }
            });

            if let Some(display) = gdk::Display::default() {
                let provider = gtk4::CssProvider::new();
                provider.load_from_data(include_str!("../resources/style.css"));
                gtk4::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }

            let mut wh = wh_cmd.borrow_mut();
            if wh.is_none() {
                let win = QuickSettingsWindow::new(app, is_bench);
                if is_daemon {
                    std::mem::forget(app.hold());
                    win.window.set_visible(false);
                } else if let Some(ref target) = page_target {
                    win.open_page(target);
                } else {
                    win.window.present();
                }
                *wh = Some(win);
            } else if let Some(ref win) = *wh {
                if is_daemon {
                    // Daemon already resident and active
                } else if let Some(ref target) = page_target {
                    win.open_page(target);
                } else if has_toggle {
                    win.toggle();
                } else {
                    win.window.set_visible(true);
                    win.window.present();
                }
            }

            0
        });

        Self { app }
    }

    pub fn run(&self) -> glib::ExitCode {
        self.app.run()
    }
}
