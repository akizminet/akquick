use gtk4::prelude::*;
use crate::services::{NetworkService, SystemState};

pub struct WifiPage {
    pub widget: gtk4::Box,
}

impl WifiPage {
    pub fn new<F>(state: &SystemState, on_back: F) -> Self
    where
        F: Fn() + 'static,
    {
        let container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(12)
            .css_classes(["subpage-container"])
            .build();

        // 1. Header with Back Button and Master Switch
        let header = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(10)
            .css_classes(["subpage-header"])
            .build();

        let back_btn = gtk4::Button::builder()
            .icon_name("go-previous-symbolic")
            .css_classes(["subpage-back-btn"])
            .tooltip_text("Back to Quick Settings")
            .build();
        back_btn.connect_clicked(move |_| {
            on_back();
        });

        let title_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();

        let title_lbl = gtk4::Label::builder()
            .label("Wi-Fi Networks")
            .halign(gtk4::Align::Start)
            .css_classes(["subpage-title"])
            .build();

        let sub_lbl = gtk4::Label::builder()
            .label("Interface: wlo1")
            .halign(gtk4::Align::Start)
            .css_classes(["subpage-subtitle"])
            .build();

        title_box.append(&title_lbl);
        title_box.append(&sub_lbl);

        let master_switch = gtk4::Switch::builder()
            .active(state.network.wifi_enabled)
            .valign(gtk4::Align::Center)
            .build();
        master_switch.connect_active_notify(|s| {
            NetworkService::set_wifi_enabled(s.is_active());
        });

        header.append(&back_btn);
        header.append(&title_box);
        header.append(&master_switch);

        container.append(&header);

        // 2. Active Connected Network Card
        if state.network.wifi_enabled && state.network.wifi_ssid != "Disconnected" {
            let card = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(8)
                .css_classes(["subpage-card"])
                .build();

            let badge_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(6)
                .build();

            let badge = gtk4::Label::builder()
                .label("● CONNECTED NETWORK")
                .css_classes(["subpage-badge"])
                .halign(gtk4::Align::Start)
                .hexpand(true)
                .build();

            badge_box.append(&badge);

            let main_row = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(12)
                .valign(gtk4::Align::Center)
                .build();

            let icon = gtk4::Image::builder()
                .icon_name("network-wireless-symbolic")
                .pixel_size(24)
                .build();

            let info_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(2)
                .hexpand(true)
                .build();

            let ssid_lbl = gtk4::Label::builder()
                .label(&state.network.wifi_ssid)
                .halign(gtk4::Align::Start)
                .css_classes(["subpage-title"])
                .build();

            let sec_lbl = gtk4::Label::builder()
                .label("WPA2/WPA3 Personal • 5.0 GHz")
                .halign(gtk4::Align::Start)
                .css_classes(["subpage-subtitle"])
                .build();

            info_box.append(&ssid_lbl);
            info_box.append(&sec_lbl);

            let disconnect_btn = gtk4::Button::builder()
                .label("Disconnect")
                .css_classes(["subpage-action-btn", "secondary"])
                .build();
            let current_ssid = state.network.wifi_ssid.clone();
            disconnect_btn.connect_clicked(move |_| {
                NetworkService::disconnect_wifi(&current_ssid);
            });

            main_row.append(&icon);
            main_row.append(&info_box);
            main_row.append(&disconnect_btn);

            card.append(&badge_box);
            card.append(&main_row);

            if !state.network.wifi_ip.is_empty() {
                let ip_box = gtk4::Grid::builder()
                    .column_spacing(8)
                    .column_homogeneous(true)
                    .css_classes(["vpn-traffic-box"])
                    .margin_top(4)
                    .build();

                let ip_vbox = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Vertical)
                    .spacing(2)
                    .build();
                let ip_head = gtk4::Label::builder()
                    .label("IPV4 ADDRESS")
                    .css_classes(["vpn-traffic-header"])
                    .halign(gtk4::Align::Start)
                    .build();
                let ip_val = gtk4::Label::builder()
                    .label(&state.network.wifi_ip)
                    .css_classes(["vpn-traffic-val"])
                    .halign(gtk4::Align::Start)
                    .build();
                ip_vbox.append(&ip_head);
                ip_vbox.append(&ip_val);

                let gw_vbox = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Vertical)
                    .spacing(2)
                    .build();
                let gw_head = gtk4::Label::builder()
                    .label("GATEWAY / ROUTER")
                    .css_classes(["vpn-traffic-header"])
                    .halign(gtk4::Align::Start)
                    .build();
                let gw_val = gtk4::Label::builder()
                    .label(if !state.network.wifi_gateway.is_empty() { &state.network.wifi_gateway } else { "192.168.1.1" })
                    .css_classes(["vpn-traffic-val"])
                    .halign(gtk4::Align::Start)
                    .build();
                gw_vbox.append(&gw_head);
                gw_vbox.append(&gw_val);

                ip_box.attach(&ip_vbox, 0, 0, 1, 1);
                ip_box.attach(&gw_vbox, 1, 0, 1, 1);
                card.append(&ip_box);
            }

            container.append(&card);
        }

        // 3. Scanned / Visible Networks List
        let list_header = gtk4::Label::builder()
            .label("VISIBLE NETWORKS")
            .halign(gtk4::Align::Start)
            .css_classes(["subpage-badge"])
            .build();
        container.append(&list_header);

        let list_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .css_classes(["subpage-list"])
            .build();

        if state.network.scanned_aps.is_empty() {
            let empty_lbl = gtk4::Label::builder()
                .label("No networks found")
                .css_classes(["subpage-subtitle"])
                .margin_top(12)
                .margin_bottom(12)
                .build();
            list_box.append(&empty_lbl);
        } else {
            for ap in state.network.scanned_aps.iter().take(6) {
                if ap.ssid == state.network.wifi_ssid {
                    continue; // skip currently connected in the available list
                }

                let row = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Horizontal)
                    .spacing(10)
                    .css_classes(["subpage-item"])
                    .build();

                let ap_icon = gtk4::Image::builder()
                    .icon_name("network-wireless-symbolic")
                    .pixel_size(18)
                    .build();

                let ap_box = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Vertical)
                    .spacing(2)
                    .hexpand(true)
                    .build();

                let ap_title = gtk4::Label::builder()
                    .label(&ap.ssid)
                    .halign(gtk4::Align::Start)
                    .css_classes(["capsule-label"])
                    .build();

                let ap_info = gtk4::Label::builder()
                    .label(&format!("{}% signal • {}", ap.signal, ap.security))
                    .halign(gtk4::Align::Start)
                    .css_classes(["subpage-subtitle"])
                    .build();

                ap_box.append(&ap_title);
                ap_box.append(&ap_info);

                let connect_btn = gtk4::Button::builder()
                    .label("Connect")
                    .css_classes(["subpage-action-btn", "primary"])
                    .build();
                let target_ssid = ap.ssid.clone();
                connect_btn.connect_clicked(move |_| {
                    NetworkService::connect_wifi(&target_ssid);
                });

                row.append(&ap_icon);
                row.append(&ap_box);
                row.append(&connect_btn);

                list_box.append(&row);
            }
        }

        container.append(&list_box);

        // 4. Footer Settings Button
        let footer_btn = gtk4::Button::builder()
            .label("Wi-Fi Settings…")
            .css_classes(["subpage-action-btn", "secondary"])
            .halign(gtk4::Align::Start)
            .build();
        footer_btn.connect_clicked(|_| {
            let _ = std::process::Command::new("nmrs-gui").spawn();
        });
        container.append(&footer_btn);

        Self { widget: container }
    }
}
