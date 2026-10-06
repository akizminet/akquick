pub mod footer;
pub mod header;
pub mod mpris;
pub mod pages;
pub mod sliders;
pub mod toggles;

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use crate::services::SystemState;

pub enum AppUpdate {
    Status {
        mpris: crate::services::MprisState,
        network: crate::services::NetworkStatus,
    },
    FullWifi(crate::services::NetworkState),
}

pub struct QuickSettingsWindow {
    pub window: gtk4::ApplicationWindow,
    pub stack: gtk4::Stack,
    #[allow(dead_code)]
    pub mpris: std::rc::Rc<mpris::MprisCard>,
    #[allow(dead_code)]
    pub toggles: std::rc::Rc<toggles::ToggleGrid>,
    #[allow(dead_code)]
    pub wifi_page: std::rc::Rc<pages::WifiPage>,
    pub poll_tx: std::sync::mpsc::Sender<AppUpdate>,
}

impl QuickSettingsWindow {
    pub fn new(app: &libadwaita::Application, is_bench: bool) -> Self {
        let t_start = std::time::Instant::now();
        let t0 = crate::START_TIME.get().copied().unwrap_or(t_start);

        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("Quick Settings")
            .default_width(420)
            .css_classes(["quicksettings-window"])
            .build();

        // Layer-Shell Configuration: Fullscreen transparent surface to capture outside clicks
        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_namespace("akquick");
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Bottom, true);
        window.set_anchor(Edge::Left, true);
        window.set_anchor(Edge::Right, true);
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);

        // Fetch live state
        let t_state_start = std::time::Instant::now();
        let state = SystemState::fetch();
        let t_state_done = std::time::Instant::now();

        // Communication channel between Tokio async runtime and GTK main thread
        let (poll_tx, rx) = std::sync::mpsc::channel::<AppUpdate>();

        // Stack for multi-page drilldown navigation
        let stack = gtk4::Stack::builder()
            .transition_type(gtk4::StackTransitionType::SlideLeftRight)
            .transition_duration(250)
            .build();

        // 1. Main Obsidian Quick Settings Card
        let main_card = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(12)
            .css_classes(["quicksettings-card"])
            .width_request(420)
            .build();

        let header = header::HeaderBar::new();
        main_card.append(&header.widget);

        let stack_wifi = stack.clone();
        let stack_bt = stack.clone();
        let stack_vpn = stack.clone();

        let wifi_tx = poll_tx.clone();
        let on_open_wifi = move || {
            stack_wifi.set_visible_child_name("wifi");
            let tx = wifi_tx.clone();
            crate::services::mpris::MPRIS_RT.spawn(async move {
                let net = crate::services::NetworkService::fetch_with_scan().await;
                let _ = tx.send(AppUpdate::FullWifi(net));
            });
        };

        let toggles = std::rc::Rc::new(toggles::ToggleGrid::new(
            &state,
            on_open_wifi,
            move || stack_bt.set_visible_child_name("bluetooth"),
            move || stack_vpn.set_visible_child_name("vpn"),
        ));
        main_card.append(&toggles.widget);

        let sliders = sliders::SlidersSection::new(&state);
        main_card.append(&sliders.widget);

        let mpris = std::rc::Rc::new(mpris::MprisCard::new(&state));
        main_card.append(&mpris.widget);

        let footer = footer::FooterBar::new(&state);
        main_card.append(&footer.widget);

        // 2. Wi-Fi Sub-Page
        let stack_back1 = stack.clone();
        let scan_tx = poll_tx.clone();
        let on_scan = move || {
            let tx = scan_tx.clone();
            crate::services::mpris::MPRIS_RT.spawn(async move {
                let _ = std::process::Command::new("nmcli").args(["dev", "wifi", "rescan"]).output();
                tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                let net = crate::services::NetworkService::fetch_with_scan().await;
                let _ = tx.send(AppUpdate::FullWifi(net));
            });
        };
        let wifi_page = std::rc::Rc::new(pages::WifiPage::new(
            &state,
            move || {
                stack_back1.set_visible_child_name("main");
            },
            on_scan,
        ));

        // 3. Bluetooth Sub-Page
        let stack_back2 = stack.clone();
        let bt_page = pages::BluetoothPage::new(&state, move || {
            stack_back2.set_visible_child_name("main");
        });

        // 4. VPN Sub-Page
        let stack_back3 = stack.clone();
        let vpn_page = pages::VpnPage::new(&state, move || {
            stack_back3.set_visible_child_name("main");
        });

        stack.add_named(&main_card, Some("main"));
        stack.add_named(&wifi_page.widget, Some("wifi"));
        stack.add_named(&bt_page.widget, Some("bluetooth"));
        stack.add_named(&vpn_page.widget, Some("vpn"));
        stack.set_visible_child_name("main");

        // Position card in top-right with margin below status bar
        let root_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .hexpand(true)
            .vexpand(true)
            .halign(gtk4::Align::End)
            .valign(gtk4::Align::Start)
            .margin_top(8)
            .margin_end(16)
            .build();
        root_box.append(&stack);

        window.set_child(Some(&root_box));

        // Auto-dismiss on click outside the card
        let gesture = gtk4::GestureClick::new();
        gesture.set_propagation_phase(gtk4::PropagationPhase::Bubble);
        let win_ref = window.clone();
        let stack_ref = stack.clone();
        gesture.connect_pressed(move |_gesture, _n_press, x, y| {
            if let Some((local_x, local_y)) = win_ref.translate_coordinates(&stack_ref, x, y) {
                let width = stack_ref.width() as f64;
                let height = stack_ref.height() as f64;
                if local_x >= 0.0 && local_x <= width && local_y >= 0.0 && local_y <= height {
                    return;
                }
            }
            win_ref.set_visible(false);
        });
        window.add_controller(gesture);

        // Escape key to dismiss
        let key_controller = gtk4::EventControllerKey::new();
        let win_key = window.clone();
        key_controller.connect_key_pressed(move |_, keyval, _, _| {
            if keyval == gtk4::gdk::Key::Escape {
                win_key.set_visible(false);
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        window.add_controller(key_controller);

        // Auto-dismiss on loss of focus (e.g. clicking other monitor or switching windows)
        let win_focus = window.clone();
        let was_active = std::rc::Rc::new(std::cell::Cell::new(false));
        let was_active_clone = was_active.clone();
        window.connect_is_active_notify(move |win| {
            if win.is_active() {
                was_active_clone.set(true);
            } else if was_active_clone.get() && win.is_visible() {
                was_active_clone.set(false);
                win_focus.set_visible(false);
            }
        });

        let t_widgets_done = std::time::Instant::now();

        // Surface Map and Frame Clock Benchmark Tracing
        let t_map_rec = std::rc::Rc::new(std::cell::Cell::new(None));
        let t_map_clone = t_map_rec.clone();

        window.connect_map(move |_| {
            if t_map_clone.get().is_none() {
                t_map_clone.set(Some(std::time::Instant::now()));
            }
        });

        let t_frame_done = std::rc::Rc::new(std::cell::Cell::new(false));
        window.add_tick_callback(move |_win, _clock| {
            if !t_frame_done.get() {
                t_frame_done.set(true);
                let t_first_frame = std::time::Instant::now();
                let t_map = t_map_rec.get().unwrap_or(t_first_frame);

                if is_bench || std::env::var("AKQUICK_BENCH").is_ok() {
                    let total_startup = t_map.duration_since(t0);
                    let ttff = t_first_frame.duration_since(t0);
                    let init_dur = t_state_start.duration_since(t0);
                    let state_dur = t_state_done.duration_since(t_state_start);
                    let widgets_dur = t_widgets_done.duration_since(t_state_done);
                    let map_dur = t_map.duration_since(t_widgets_done);
                    let render_dur = t_first_frame.duration_since(t_map);

                    eprintln!("\n\x1b[1;36m============================================================\x1b[0m");
                    eprintln!("\x1b[1;37m              AKQUICK STARTUP & TTFF BENCHMARK              \x1b[0m");
                    eprintln!("\x1b[1;36m============================================================\x1b[0m");
                    eprintln!(" [1] Process Start -> GApp/State Start : {:>9.2?}", init_dur);
                    eprintln!(" [2] SystemState::fetch() (Hardware)   : {:>9.2?}", state_dur);
                    eprintln!(" [3] GTK4 / Libadwaita Widget Build    : {:>9.2?}", widgets_dur);
                    eprintln!(" [4] Window Present -> Surface Mapped  : {:>9.2?}", map_dur);
                    eprintln!(" [5] Surface Mapped -> 1st Frame Render: {:>9.2?}", render_dur);
                    eprintln!("\x1b[1;30m------------------------------------------------------------\x1b[0m");
                    eprintln!(" \x1b[1;32mTOTAL COLD STARTUP (Process -> Map)   : {:>9.2?}\x1b[0m", total_startup);
                    eprintln!(" \x1b[1;33mTIME TO FIRST FRAME (TTFF)            : {:>9.2?}\x1b[0m", ttff);
                    eprintln!("\x1b[1;36m============================================================\x1b[0m\n");

                    if is_bench {
                        glib::timeout_add_local_once(std::time::Duration::from_millis(15), move || {
                            std::process::exit(0);
                        });
                    }
                }
            }
            glib::ControlFlow::Continue
        });

        // Live Reactive Polling Ticker (every 500ms) for MPRIS + Network & VPN
        let mpris_card = mpris.clone();
        let toggles_card = toggles.clone();
        let wifi_card = wifi_page.clone();
        let tx_fetch = poll_tx.clone();
        let stack_poll = stack.clone();
        let mut last_ssid = state.network.wifi_ssid.clone();
        let mut ticker_count: u32 = 0;

        glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
            while let Ok(data) = rx.try_recv() {
                match data {
                    AppUpdate::Status { mpris, network } => {
                        mpris_card.update(&mpris);
                        toggles_card.update_network(&network);

                        // If Wi-Fi SSID changed (e.g. disconnected or switched AP), auto-refresh wifi page
                        if network.wifi_ssid != last_ssid {
                            last_ssid = network.wifi_ssid.clone();
                            let tx = tx_fetch.clone();
                            crate::services::mpris::MPRIS_RT.spawn(async move {
                                let net = crate::services::NetworkService::fetch().await;
                                let _ = tx.send(AppUpdate::FullWifi(net));
                            });
                        }
                    }
                    AppUpdate::FullWifi(net_state) => {
                        wifi_card.update(&net_state);
                    }
                }
            }

            ticker_count += 1;
            let tx = tx_fetch.clone();
            let is_wifi_open = stack_poll.visible_child_name().as_deref() == Some("wifi");
            let should_refresh_wifi = is_wifi_open && (ticker_count % 8 == 0);

            crate::services::mpris::MPRIS_RT.spawn(async move {
                let (mpris, network) = tokio::join!(
                    crate::services::MprisService::fetch(),
                    crate::services::NetworkService::fetch_status(),
                );
                let _ = tx.send(AppUpdate::Status { mpris, network });

                if should_refresh_wifi {
                    let net = crate::services::NetworkService::fetch_with_scan().await;
                    let _ = tx.send(AppUpdate::FullWifi(net));
                }
            });

            glib::ControlFlow::Continue
        });

        Self {
            window,
            stack,
            mpris,
            toggles,
            wifi_page,
            poll_tx,
        }
    }

    pub fn refresh(&self) {
        let tx = self.poll_tx.clone();
        crate::services::mpris::MPRIS_RT.spawn(async move {
            let (mpris, network) = tokio::join!(
                crate::services::MprisService::fetch(),
                crate::services::NetworkService::fetch_status(),
            );
            let _ = tx.send(AppUpdate::Status { mpris, network });
        });
    }

    pub fn refresh_wifi(&self) {
        let tx = self.poll_tx.clone();
        crate::services::mpris::MPRIS_RT.spawn(async move {
            let net = crate::services::NetworkService::fetch_with_scan().await;
            let _ = tx.send(AppUpdate::FullWifi(net));
        });
    }

    pub fn update_active_monitor(&self) {
        if let Some(display) = gtk4::gdk::Display::default() {
            let focused_name = std::process::Command::new("hyprctl")
                .args(["monitors", "-j"])
                .output()
                .ok()
                .and_then(|out| {
                    let json: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
                    json.as_array()?.iter().find(|m| m.get("focused") == Some(&serde_json::Value::Bool(true)))
                        .and_then(|m| m.get("name")?.as_str().map(|s| s.to_string()))
                });

            if let Some(name) = focused_name {
                let monitors = display.monitors();
                for i in 0..monitors.n_items() {
                    if let Some(monitor) = monitors.item(i).and_downcast::<gtk4::gdk::Monitor>() {
                        if monitor.connector().as_deref() == Some(&name) {
                            self.window.set_monitor(&monitor);
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn open_page(&self, page_name: &str) {
        self.update_active_monitor();
        self.refresh();
        if page_name == "wifi" {
            self.refresh_wifi();
        }
        self.stack.set_visible_child_name(page_name);
        self.window.set_visible(true);
        self.window.present();
    }

    pub fn toggle(&self) {
        if self.window.is_visible() {
            self.window.set_visible(false);
        } else {
            self.update_active_monitor();
            self.refresh();
            self.window.set_visible(true);
            self.window.present();
        }
    }
}
