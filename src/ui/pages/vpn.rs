use gtk4::prelude::*;
use crate::services::{NetworkService, SystemState};

pub struct VpnPage {
    pub widget: gtk4::Box,
}

impl VpnPage {
    pub fn new<F>(state: &SystemState, on_back: F) -> Self
    where
        F: Fn() + 'static,
    {
        let container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(12)
            .css_classes(["subpage-container"])
            .build();

        // 1. Header with Back Button, Title, and Kill-Switch Toggle Chip
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

        let title_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .build();

        let title_lbl = gtk4::Label::builder()
            .label("VPN & WireGuard")
            .halign(gtk4::Align::Start)
            .css_classes(["subpage-title"])
            .build();

        let shield_icon = gtk4::Image::builder()
            .icon_name("security-high-symbolic")
            .pixel_size(16)
            .css_classes(["vpn-shield-icon"])
            .build();

        title_row.append(&title_lbl);
        title_row.append(&shield_icon);

        let sub_lbl = gtk4::Label::builder()
            .label("Encrypted Tunnel Telemetry")
            .halign(gtk4::Align::Start)
            .css_classes(["subpage-subtitle"])
            .build();

        title_box.append(&title_row);
        title_box.append(&sub_lbl);

        // Master Kill-Switch Toggle Chip
        let kill_switch_btn = gtk4::Button::builder()
            .css_classes(["vpn-kill-switch", "active"])
            .valign(gtk4::Align::Center)
            .build();

        let ks_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .build();
        let ks_dot = gtk4::Box::builder()
            .css_classes(["vpn-ks-dot"])
            .build();
        let ks_label = gtk4::Label::builder()
            .label("Kill Switch: ON")
            .css_classes(["vpn-ks-label"])
            .build();
        ks_box.append(&ks_dot);
        ks_box.append(&ks_label);
        kill_switch_btn.set_child(Some(&ks_box));

        let is_ks_on = std::cell::Cell::new(true);
        let ks_label_clone = ks_label.clone();
        let ks_btn_clone = kill_switch_btn.clone();
        kill_switch_btn.connect_clicked(move |_| {
            let next_val = !is_ks_on.get();
            is_ks_on.set(next_val);
            if next_val {
                ks_label_clone.set_label("Kill Switch: ON");
                ks_btn_clone.remove_css_class("inactive");
                ks_btn_clone.add_css_class("active");
            } else {
                ks_label_clone.set_label("Kill Switch: OFF");
                ks_btn_clone.remove_css_class("active");
                ks_btn_clone.add_css_class("inactive");
            }
        });

        header.append(&back_btn);
        header.append(&title_box);
        header.append(&kill_switch_btn);

        container.append(&header);

        // 2. Active Tunnel Card (if VPN is active)
        if state.network.vpn_active {
            let card = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(10)
                .css_classes(["vpn-active-card"])
                .build();

            // Card Top Row: Icon, Title, Interface badge, and Latency Pill
            let top_row = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(10)
                .valign(gtk4::Align::Center)
                .build();

            let icon_box = gtk4::Box::builder()
                .css_classes(["vpn-icon-emerald"])
                .build();
            let vpn_icon = gtk4::Image::builder()
                .icon_name("network-vpn-symbolic")
                .pixel_size(20)
                .build();
            icon_box.append(&vpn_icon);

            let vpn_info = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(2)
                .hexpand(true)
                .build();

            let name_row = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(6)
                .build();

            let name_lbl = gtk4::Label::builder()
                .label(&state.network.vpn_details.name)
                .css_classes(["subpage-title"])
                .halign(gtk4::Align::Start)
                .build();

            let iface_badge = gtk4::Label::builder()
                .label(&state.network.vpn_details.interface)
                .css_classes(["vpn-iface-badge"])
                .halign(gtk4::Align::Start)
                .build();

            name_row.append(&name_lbl);
            name_row.append(&iface_badge);

            let ip_sub = gtk4::Label::builder()
                .label(&format!("● Connected • {}", state.network.vpn_details.ip))
                .css_classes(["vpn-ip-subtitle"])
                .halign(gtk4::Align::Start)
                .build();

            vpn_info.append(&name_row);
            vpn_info.append(&ip_sub);

            // Latency badge
            let latency_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(4)
                .css_classes(["vpn-latency-pill"])
                .valign(gtk4::Align::Center)
                .build();

            let lat_icon = gtk4::Image::builder()
                .icon_name("network-transmit-receive-symbolic")
                .pixel_size(12)
                .build();
            let lat_lbl = gtk4::Label::builder()
                .label(&state.network.vpn_details.latency_ms)
                .css_classes(["font-mono"])
                .build();
            latency_box.append(&lat_icon);
            latency_box.append(&lat_lbl);

            top_row.append(&icon_box);
            top_row.append(&vpn_info);
            top_row.append(&latency_box);

            card.append(&top_row);

            // Live Throughput Telemetry Badges (Ingress / Egress)
            let traffic_grid = gtk4::Grid::builder()
                .column_spacing(8)
                .column_homogeneous(true)
                .build();

            // Ingress Box
            let ingress_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(2)
                .css_classes(["vpn-traffic-box"])
                .build();
            let in_header = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(4)
                .build();
            let in_icon = gtk4::Image::builder()
                .icon_name("pan-down-symbolic")
                .pixel_size(12)
                .css_classes(["text-secondary"])
                .build();
            let in_title = gtk4::Label::builder()
                .label("INGRESS")
                .css_classes(["vpn-traffic-header"])
                .build();
            in_header.append(&in_icon);
            in_header.append(&in_title);

            let in_value = gtk4::Label::builder()
                .label(&state.network.vpn_details.rx_bytes_str)
                .css_classes(["vpn-traffic-val"])
                .halign(gtk4::Align::Start)
                .build();
            ingress_box.append(&in_header);
            ingress_box.append(&in_value);

            // Egress Box
            let egress_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .spacing(2)
                .css_classes(["vpn-traffic-box"])
                .build();
            let out_header = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(4)
                .build();
            let out_icon = gtk4::Image::builder()
                .icon_name("pan-up-symbolic")
                .pixel_size(12)
                .css_classes(["text-primary"])
                .build();
            let out_title = gtk4::Label::builder()
                .label("EGRESS")
                .css_classes(["vpn-traffic-header"])
                .build();
            out_header.append(&out_icon);
            out_header.append(&out_title);

            let out_value = gtk4::Label::builder()
                .label(&state.network.vpn_details.tx_bytes_str)
                .css_classes(["vpn-traffic-val"])
                .halign(gtk4::Align::Start)
                .build();
            egress_box.append(&out_header);
            egress_box.append(&out_value);

            traffic_grid.attach(&ingress_box, 0, 0, 1, 1);
            traffic_grid.attach(&egress_box, 1, 0, 1, 1);
            card.append(&traffic_grid);

            // Action Buttons Bar
            let action_bar = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(8)
                .build();

            let disconnect_btn = gtk4::Button::builder()
                .label("Disconnect")
                .icon_name("system-shutdown-symbolic")
                .hexpand(true)
                .css_classes(["vpn-disconnect-btn"])
                .build();
            let vpn_id = state.network.vpn_name.clone();
            disconnect_btn.connect_clicked(move |_| {
                NetworkService::disconnect_vpn(&vpn_id);
            });

            let reconnect_btn = gtk4::Button::builder()
                .label("Reconnect")
                .icon_name("view-refresh-symbolic")
                .hexpand(true)
                .css_classes(["subpage-action-btn", "secondary"])
                .build();
            let vpn_id2 = state.network.vpn_name.clone();
            reconnect_btn.connect_clicked(move |_| {
                NetworkService::connect_vpn(&vpn_id2);
            });

            action_bar.append(&disconnect_btn);
            action_bar.append(&reconnect_btn);
            card.append(&action_bar);

            container.append(&card);
        }

        // 3. Available Profiles & Tunnels Section
        let list_header = gtk4::Label::builder()
            .label("AVAILABLE TUNNELS")
            .halign(gtk4::Align::Start)
            .css_classes(["subpage-badge"])
            .build();
        container.append(&list_header);

        let list_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .css_classes(["subpage-list"])
            .build();

        if state.network.vpn_profiles.is_empty() {
            let empty_lbl = gtk4::Label::builder()
                .label("No configured VPN tunnels")
                .css_classes(["subpage-subtitle"])
                .margin_top(12)
                .margin_bottom(12)
                .build();
            list_box.append(&empty_lbl);
        } else {
            for profile in &state.network.vpn_profiles {
                let row = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Horizontal)
                    .spacing(10)
                    .css_classes(["subpage-item"])
                    .build();

                let icon = gtk4::Image::builder()
                    .icon_name(if profile.active { "network-vpn-symbolic" } else { "network-vpn-acquiring-symbolic" })
                    .pixel_size(20)
                    .build();

                let p_box = gtk4::Box::builder()
                    .orientation(gtk4::Orientation::Vertical)
                    .spacing(2)
                    .hexpand(true)
                    .build();

                let p_title = gtk4::Label::builder()
                    .label(&profile.name)
                    .halign(gtk4::Align::Start)
                    .css_classes(["capsule-label"])
                    .build();

                let p_sub = gtk4::Label::builder()
                    .label(&format!("{} • {}", profile.interface, profile.vpn_type))
                    .halign(gtk4::Align::Start)
                    .css_classes(["subpage-subtitle"])
                    .build();

                p_box.append(&p_title);
                p_box.append(&p_sub);

                let toggle_btn = gtk4::Button::builder()
                    .label(if profile.active { "Disconnect" } else { "Connect" })
                    .css_classes([
                        "subpage-action-btn",
                        if profile.active { "secondary" } else { "primary" },
                    ])
                    .build();

                let p_name = profile.name.clone();
                let is_act = profile.active;
                toggle_btn.connect_clicked(move |_| {
                    if is_act {
                        NetworkService::disconnect_vpn(&p_name);
                    } else {
                        NetworkService::connect_vpn(&p_name);
                    }
                });

                row.append(&icon);
                row.append(&p_box);
                row.append(&toggle_btn);

                list_box.append(&row);
            }
        }
        container.append(&list_box);

        // 4. Advanced Routing & Security Options Group
        let sec_header = gtk4::Label::builder()
            .label("ROUTING & TELEMETRY GUARDS")
            .halign(gtk4::Align::Start)
            .css_classes(["subpage-badge"])
            .build();
        container.append(&sec_header);

        let guards_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .css_classes(["subpage-list"])
            .build();

        // Split Tunneling Row
        let split_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(10)
            .css_classes(["subpage-item"])
            .build();

        let split_icon = gtk4::Image::builder()
            .icon_name("network-workgroup-symbolic")
            .pixel_size(18)
            .build();

        let split_info = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();

        let split_title = gtk4::Label::builder()
            .label("Split Tunneling")
            .halign(gtk4::Align::Start)
            .css_classes(["capsule-label"])
            .build();

        let split_sub = gtk4::Label::builder()
            .label("Exclude LAN 192.168.1.0/24")
            .halign(gtk4::Align::Start)
            .css_classes(["subpage-subtitle"])
            .build();

        split_info.append(&split_title);
        split_info.append(&split_sub);

        let split_switch = gtk4::Switch::builder()
            .active(true)
            .valign(gtk4::Align::Center)
            .build();

        split_row.append(&split_icon);
        split_row.append(&split_info);
        split_row.append(&split_switch);
        guards_box.append(&split_row);

        // DNS Leak Guard Row
        let dns_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(10)
            .css_classes(["subpage-item"])
            .build();

        let dns_icon = gtk4::Image::builder()
            .icon_name("channel-secure-symbolic")
            .pixel_size(18)
            .build();

        let dns_info = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();

        let dns_title = gtk4::Label::builder()
            .label("DNS Leak Guard")
            .halign(gtk4::Align::Start)
            .css_classes(["capsule-label"])
            .build();

        let dns_sub = gtk4::Label::builder()
            .label("Enforce VPN DNS (172.16.254.9)")
            .halign(gtk4::Align::Start)
            .css_classes(["subpage-subtitle"])
            .build();

        dns_info.append(&dns_title);
        dns_info.append(&dns_sub);

        let dns_switch = gtk4::Switch::builder()
            .active(true)
            .valign(gtk4::Align::Center)
            .build();

        dns_row.append(&dns_icon);
        dns_row.append(&dns_info);
        dns_row.append(&dns_switch);
        guards_box.append(&dns_row);

        container.append(&guards_box);

        // 5. Footer Action Tray
        let footer_tray = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .build();

        let import_btn = gtk4::Button::builder()
            .label("Import .conf / .ovpn")
            .icon_name("document-open-symbolic")
            .css_classes(["subpage-action-btn", "secondary"])
            .build();
        import_btn.connect_clicked(|_| {
            let _ = std::process::Command::new("nm-connection-editor").spawn();
        });

        let settings_btn = gtk4::Button::builder()
            .label("Network Settings")
            .icon_name("emblem-system-symbolic")
            .css_classes(["subpage-action-btn", "secondary"])
            .hexpand(true)
            .build();
        settings_btn.connect_clicked(|_| {
            let _ = std::process::Command::new("nm-connection-editor").spawn();
        });

        footer_tray.append(&import_btn);
        footer_tray.append(&settings_btn);
        container.append(&footer_tray);

        Self { widget: container }
    }
}
