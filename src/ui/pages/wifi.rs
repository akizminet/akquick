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

        // 1. Header with Back Button and Master Switch (HTML lines 110-125)
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
            .label("Interface: wlo1 (Intel Wi-Fi 6E)")
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

        // 2. Active Connected Network Card (HTML lines 128-191)
        if state.network.wifi_enabled && state.network.wifi_ssid != "Disconnected" {
            let card = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(10)
                .css_classes(["subpage-card"])
                .build();

            // Badge Row: "● CONNECTED NETWORK" + "866 Mbps" pill
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

            let speed_badge = gtk4::Label::builder()
                .label("866 Mbps")
                .css_classes(["wifi-speed-pill"])
                .halign(gtk4::Align::End)
                .build();

            badge_box.append(&badge);
            badge_box.append(&speed_badge);
            card.append(&badge_box);

            // Main Info Row: Icon box + SSID/Badges + Quick Action Glyphs (QR & Settings)
            let main_row = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(12)
                .valign(gtk4::Align::Center)
                .build();

            let icon_box = gtk4::Box::builder()
                .css_classes(["wifi-icon-primary"])
                .build();
            let icon = gtk4::Image::builder()
                .icon_name("network-wireless-symbolic")
                .pixel_size(22)
                .build();
            icon_box.append(&icon);

            let info_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(2)
                .hexpand(true)
                .build();

            let ssid_row = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(6)
                .build();

            let ssid_lbl = gtk4::Label::builder()
                .label(&state.network.wifi_ssid)
                .halign(gtk4::Align::Start)
                .css_classes(["subpage-title"])
                .build();

            let lock_icon = gtk4::Image::builder()
                .icon_name("channel-secure-symbolic")
                .pixel_size(14)
                .css_classes(["text-secondary"])
                .build();

            ssid_row.append(&ssid_lbl);
            ssid_row.append(&lock_icon);

            let sec_row = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(6)
                .build();

            let band_lbl = gtk4::Label::builder()
                .label("5.0 GHz")
                .css_classes(["wifi-chip"])
                .build();

            let proto_lbl = gtk4::Label::builder()
                .label("WPA3-Personal")
                .css_classes(["wifi-chip"])
                .build();

            let sig_lbl = gtk4::Label::builder()
                .label("● 98%")
                .css_classes(["wifi-sig-label"])
                .build();

            sec_row.append(&band_lbl);
            sec_row.append(&proto_lbl);
            sec_row.append(&sig_lbl);

            info_box.append(&ssid_row);
            info_box.append(&sec_row);

            // Quick Action Glyphs (QR Code and Gear)
            let glyphs_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(4)
                .build();

            let qr_btn = gtk4::Button::builder()
                .icon_name("document-share-symbolic")
                .tooltip_text("Share via QR Code")
                .css_classes(["subpage-glyph-btn"])
                .build();

            let gear_btn = gtk4::Button::builder()
                .icon_name("emblem-system-symbolic")
                .tooltip_text("Configure Network")
                .css_classes(["subpage-glyph-btn"])
                .build();
            gear_btn.connect_clicked(|_| {
                let _ = std::process::Command::new("nm-connection-editor").spawn();
            });

            glyphs_box.append(&qr_btn);
            glyphs_box.append(&gear_btn);

            main_row.append(&icon_box);
            main_row.append(&info_box);
            main_row.append(&glyphs_box);
            card.append(&main_row);

            // IP & Gateway Detail Bar (HTML lines 170-179)
            let ip_box = gtk4::Grid::builder()
                .column_spacing(8)
                .column_homogeneous(true)
                .css_classes(["vpn-traffic-box"])
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
                .label(if !state.network.wifi_ip.is_empty() { &state.network.wifi_ip } else { "192.168.1.142" })
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
                .label("GATEWAY / DNS")
                .css_classes(["vpn-traffic-header"])
                .halign(gtk4::Align::Start)
                .build();
            let gw_val = gtk4::Label::builder()
                .label(if !state.network.wifi_gateway.is_empty() {
                    &state.network.wifi_gateway
                } else {
                    "192.168.1.1 (Cloudflare)"
                })
                .css_classes(["vpn-traffic-val"])
                .halign(gtk4::Align::Start)
                .build();
            gw_vbox.append(&gw_head);
            gw_vbox.append(&gw_val);

            ip_box.attach(&ip_vbox, 0, 0, 1, 1);
            ip_box.attach(&gw_vbox, 1, 0, 1, 1);
            card.append(&ip_box);

            // Disconnect and Forget Buttons (HTML lines 181-190)
            let actions_row = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(8)
                .halign(gtk4::Align::End)
                .build();

            let forget_btn = gtk4::Button::builder()
                .label("Forget")
                .css_classes(["subpage-text-btn"])
                .build();

            let disconnect_btn = gtk4::Button::builder()
                .label("Disconnect")
                .icon_name("network-offline-symbolic")
                .css_classes(["subpage-action-btn", "secondary"])
                .build();
            let current_ssid = state.network.wifi_ssid.clone();
            disconnect_btn.connect_clicked(move |_| {
                NetworkService::disconnect_wifi(&current_ssid);
            });

            actions_row.append(&forget_btn);
            actions_row.append(&disconnect_btn);
            card.append(&actions_row);

            container.append(&card);
        }

        // 3. Scanned / Visible Networks List (HTML lines 193-260)
        let count_str = format!("VISIBLE NETWORKS ({})", state.network.scanned_aps.len().max(1));
        let list_header_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .hexpand(true)
            .build();

        let list_header = gtk4::Label::builder()
            .label(&count_str)
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .css_classes(["subpage-badge"])
            .build();

        let update_lbl = gtk4::Label::builder()
            .label("Updated just now")
            .halign(gtk4::Align::End)
            .css_classes(["subpage-subtitle"])
            .build();

        list_header_box.append(&list_header);
        list_header_box.append(&update_lbl);
        container.append(&list_header_box);

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
            for ap in state.network.scanned_aps.iter().take(5) {
                if ap.ssid == state.network.wifi_ssid {
                    continue; // skip currently connected
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

                let ap_title_row = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Horizontal)
                    .spacing(6)
                    .build();

                let ap_title = gtk4::Label::builder()
                    .label(&ap.ssid)
                    .halign(gtk4::Align::Start)
                    .css_classes(["capsule-label"])
                    .build();

                let ap_lock = gtk4::Image::builder()
                    .icon_name("channel-secure-symbolic")
                    .pixel_size(12)
                    .opacity(0.7)
                    .build();

                ap_title_row.append(&ap_title);
                ap_title_row.append(&ap_lock);

                let ap_info = gtk4::Label::builder()
                    .label(&format!("5 GHz • {}% signal • {}", ap.signal, ap.security))
                    .halign(gtk4::Align::Start)
                    .css_classes(["subpage-subtitle"])
                    .build();

                ap_box.append(&ap_title_row);
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

        // 4. Action Buttons Row: Scan for Networks + Hidden Network (HTML lines 280-290)
        let actions_bar = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .build();

        let scan_btn = gtk4::Button::builder()
            .label("Scan for Networks")
            .icon_name("view-refresh-symbolic")
            .css_classes(["subpage-action-btn", "secondary"])
            .hexpand(true)
            .build();
        scan_btn.connect_clicked(|_| {
            let _ = std::process::Command::new("nmcli").args(["dev", "wifi", "rescan"]).spawn();
        });

        let hidden_btn = gtk4::Button::builder()
            .label("Hidden Network…")
            .icon_name("list-add-symbolic")
            .css_classes(["subpage-action-btn", "secondary"])
            .hexpand(true)
            .build();

        actions_bar.append(&scan_btn);
        actions_bar.append(&hidden_btn);
        container.append(&actions_bar);

        // 5. Full Width Network Settings Button (HTML line 300)
        let settings_btn = gtk4::Button::builder()
            .css_classes(["subpage-full-btn"])
            .build();

        let set_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .build();

        let set_icon = gtk4::Image::builder()
            .icon_name("emblem-system-symbolic")
            .pixel_size(16)
            .build();

        let set_lbl = gtk4::Label::builder()
            .label("Network Settings…")
            .hexpand(true)
            .halign(gtk4::Align::Start)
            .css_classes(["capsule-label"])
            .build();

        let set_arrow = gtk4::Image::builder()
            .icon_name("go-next-symbolic")
            .pixel_size(14)
            .build();

        set_box.append(&set_icon);
        set_box.append(&set_lbl);
        set_box.append(&set_arrow);
        settings_btn.set_child(Some(&set_box));

        settings_btn.connect_clicked(|_| {
            let _ = std::process::Command::new("nm-connection-editor").spawn();
        });

        container.append(&settings_btn);

        // 6. Bottom Status Footer Line (HTML line 310)
        let status_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .build();

        let ipv6_lbl = gtk4::Label::builder()
            .label("● IPv6: fe80::d9:f5ff:fe8e:124a")
            .css_classes(["wifi-footer-text"])
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();

        let reg_lbl = gtk4::Label::builder()
            .label("Regulatory: FCC (US)")
            .css_classes(["wifi-footer-text"])
            .halign(gtk4::Align::End)
            .build();

        status_row.append(&ipv6_lbl);
        status_row.append(&reg_lbl);
        container.append(&status_row);

        Self { widget: container }
    }
}
