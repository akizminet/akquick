use gtk4::glib;
use gtk4::prelude::*;
use std::cell::Cell;
use std::rc::Rc;
use crate::services::{NetworkService, NetworkState, SystemState, WifiAccessPoint};

pub struct WifiPage {
    pub widget: gtk4::Box,
    pub connected_card_container: gtk4::Box,
    pub list_box: gtk4::Box,
    pub list_header: gtk4::Label,
    pub master_switch: gtk4::Switch,
    pub scan_btn: gtk4::Button,
    pub is_updating: Rc<Cell<bool>>,
    pub sub_lbl: gtk4::Label,
}

impl WifiPage {
    pub fn new<FB, FS>(state: &SystemState, on_back: FB, on_scan: FS) -> Self
    where
        FB: Fn() + 'static,
        FS: Fn() + 'static,
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

        let is_updating = Rc::new(Cell::new(false));
        let is_updating_switch = is_updating.clone();

        let master_switch = gtk4::Switch::builder()
            .active(state.network.wifi_enabled)
            .valign(gtk4::Align::Center)
            .build();
        master_switch.connect_state_set(move |_s, active| {
            if !is_updating_switch.get() {
                NetworkService::set_wifi_enabled(active);
            }
            glib::Propagation::Proceed
        });

        header.append(&back_btn);
        header.append(&title_box);
        header.append(&master_switch);

        container.append(&header);

        // 2. Active Connected Network Card Container
        let connected_card_container = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(10)
            .build();
        container.append(&connected_card_container);

        // 3. Scanned / Visible Networks List Header
        let list_header_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .hexpand(true)
            .build();

        let list_header = gtk4::Label::builder()
            .label("VISIBLE NETWORKS")
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
        container.append(&list_box);

        // 4. Action Buttons Row: Scan for Networks + Hidden Network
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
        let scan_btn_clone = scan_btn.clone();
        scan_btn.connect_clicked(move |_| {
            scan_btn_clone.set_label("Scanning…");
            scan_btn_clone.set_sensitive(false);
            on_scan();
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

        // 5. Full Width Network Settings Button
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

        // 6. Bottom Status Footer Line
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

        let page = Self {
            widget: container,
            connected_card_container,
            list_box,
            list_header,
            master_switch,
            scan_btn,
            is_updating,
            sub_lbl,
        };

        page.update(&state.network);
        page
    }

    pub fn update(&self, network: &NetworkState) {
        if self.master_switch.is_active() != network.wifi_enabled {
            self.is_updating.set(true);
            self.master_switch.set_active(network.wifi_enabled);
            self.is_updating.set(false);
        }
        self.scan_btn.set_label("Scan for Networks");
        self.scan_btn.set_sensitive(true);

        // Dynamic interface subtitle
        if network.primary_is_ethernet || network.ethernet.connected {
            let iface_name = if !network.ethernet.interface.is_empty() {
                &network.ethernet.interface
            } else {
                "enp3s0"
            };
            self.sub_lbl.set_label(&format!("Interface: {} (Ethernet)", iface_name));
        } else if !network.wifi_ssid.is_empty() && network.wifi_ssid != "Disconnected" {
            self.sub_lbl.set_label("Interface: Wi-Fi (Connected)");
        } else {
            self.sub_lbl.set_label("Interface: Wi-Fi");
        }

        // 1. Rebuild Connected Network Card
        while let Some(child) = self.connected_card_container.first_child() {
            self.connected_card_container.remove(&child);
        }

        if network.primary_is_ethernet || network.ethernet.connected {
            let card = Self::build_ethernet_card(network);
            self.connected_card_container.append(&card);
            self.connected_card_container.set_visible(true);
        } else if network.wifi_enabled && network.wifi_ssid != "Disconnected" && !network.wifi_ssid.is_empty() {
            let card = Self::build_connected_card(network);
            self.connected_card_container.append(&card);
            self.connected_card_container.set_visible(true);
        } else {
            self.connected_card_container.set_visible(false);
        }

        // 2. Rebuild Scanned Networks List
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        let mut count = 0;
        if network.scanned_aps.is_empty() {
            let empty_lbl = gtk4::Label::builder()
                .label("No networks found")
                .css_classes(["subpage-subtitle"])
                .margin_top(12)
                .margin_bottom(12)
                .build();
            self.list_box.append(&empty_lbl);
        } else {
            for ap in &network.scanned_aps {
                // If it's the connected network, skip from available list
                if ap.in_use || (!network.wifi_ssid.is_empty() && ap.ssid == network.wifi_ssid) {
                    continue;
                }

                count += 1;
                let row = Self::build_ap_row(ap);
                self.list_box.append(&row);

                if count >= 8 {
                    break;
                }
            }

            if count == 0 {
                let empty_lbl = gtk4::Label::builder()
                    .label("No other visible networks")
                    .css_classes(["subpage-subtitle"])
                    .margin_top(12)
                    .margin_bottom(12)
                    .build();
                self.list_box.append(&empty_lbl);
            }
        }

        self.list_header.set_label(&format!("VISIBLE NETWORKS ({})", count));
    }

    fn build_ethernet_card(network: &NetworkState) -> gtk4::Box {
        let card = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(10)
            .css_classes(["subpage-card"])
            .build();

        let badge_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .build();

        let badge = gtk4::Label::builder()
            .label("● CONNECTED ETHERNET")
            .css_classes(["subpage-badge"])
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();

        let speed_str = if network.ethernet.speed_mbps > 0 {
            format!("{} Mbps", network.ethernet.speed_mbps)
        } else {
            "1000 Mbps".to_string()
        };
        let speed_badge = gtk4::Label::builder()
            .label(&speed_str)
            .css_classes(["wifi-speed-pill"])
            .halign(gtk4::Align::End)
            .build();

        badge_box.append(&badge);
        badge_box.append(&speed_badge);
        card.append(&badge_box);

        let main_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(12)
            .valign(gtk4::Align::Center)
            .build();

        let icon_box = gtk4::Box::builder()
            .css_classes(["wifi-icon-primary"])
            .build();
        let icon = gtk4::Image::builder()
            .icon_name("network-wired-symbolic")
            .pixel_size(22)
            .build();
        icon_box.append(&icon);

        let info_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();

        let name_lbl = gtk4::Label::builder()
            .label(if !network.ethernet.name.is_empty() {
                &network.ethernet.name
            } else if !network.ethernet.interface.is_empty() {
                &network.ethernet.interface
            } else {
                "Ethernet Connection"
            })
            .halign(gtk4::Align::Start)
            .css_classes(["subpage-title"])
            .build();

        let sec_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(6)
            .build();

        let iface_chip = gtk4::Label::builder()
            .label(if !network.ethernet.interface.is_empty() {
                &network.ethernet.interface
            } else {
                "enp3s0"
            })
            .css_classes(["wifi-chip"])
            .build();

        let type_chip = gtk4::Label::builder()
            .label("Wired")
            .css_classes(["wifi-chip"])
            .build();

        let state_lbl = gtk4::Label::builder()
            .label("● Connected")
            .css_classes(["wifi-sig-label"])
            .build();

        sec_row.append(&iface_chip);
        sec_row.append(&type_chip);
        sec_row.append(&state_lbl);

        info_box.append(&name_lbl);
        info_box.append(&sec_row);

        let glyphs_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(4)
            .build();

        let gear_btn = gtk4::Button::builder()
            .icon_name("emblem-system-symbolic")
            .tooltip_text("Configure Network")
            .css_classes(["subpage-glyph-btn"])
            .build();
        gear_btn.connect_clicked(|_| {
            let _ = std::process::Command::new("nm-connection-editor").spawn();
        });

        glyphs_box.append(&gear_btn);

        main_row.append(&icon_box);
        main_row.append(&info_box);
        main_row.append(&glyphs_box);
        card.append(&main_row);

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
            .label(if !network.ethernet.ip.is_empty() {
                &network.ethernet.ip
            } else {
                "10.0.1.211"
            })
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
            .label(if !network.ethernet.gateway.is_empty() {
                &network.ethernet.gateway
            } else {
                "10.0.1.1"
            })
            .css_classes(["vpn-traffic-val"])
            .halign(gtk4::Align::Start)
            .build();
        gw_vbox.append(&gw_head);
        gw_vbox.append(&gw_val);

        ip_box.attach(&ip_vbox, 0, 0, 1, 1);
        ip_box.attach(&gw_vbox, 1, 0, 1, 1);
        card.append(&ip_box);

        card
    }

    fn build_connected_card(network: &NetworkState) -> gtk4::Box {
        let card = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .spacing(10)
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

        let speed_badge = gtk4::Label::builder()
            .label("866 Mbps")
            .css_classes(["wifi-speed-pill"])
            .halign(gtk4::Align::End)
            .build();

        badge_box.append(&badge);
        badge_box.append(&speed_badge);
        card.append(&badge_box);

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
            .label(&network.wifi_ssid)
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
            .label("WPA2/WPA3")
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
            .label(if !network.wifi_ip.is_empty() { &network.wifi_ip } else { "192.168.1.129" })
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
            .label(if !network.wifi_gateway.is_empty() {
                &network.wifi_gateway
            } else {
                "192.168.1.1"
            })
            .css_classes(["vpn-traffic-val"])
            .halign(gtk4::Align::Start)
            .build();
        gw_vbox.append(&gw_head);
        gw_vbox.append(&gw_val);

        ip_box.attach(&ip_vbox, 0, 0, 1, 1);
        ip_box.attach(&gw_vbox, 1, 0, 1, 1);
        card.append(&ip_box);

        let actions_row = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk4::Align::End)
            .build();

        let forget_btn = gtk4::Button::builder()
            .label("Forget")
            .css_classes(["subpage-text-btn"])
            .build();
        let forget_ssid = network.wifi_ssid.clone();
        forget_btn.connect_clicked(move |_| {
            let id = forget_ssid.clone();
            crate::services::mpris::MPRIS_RT.spawn(async move {
                crate::translators::network_manager::delete_connection_by_id(&id).await;
            });
        });

        let disconnect_btn = gtk4::Button::builder()
            .label("Disconnect")
            .icon_name("network-offline-symbolic")
            .css_classes(["subpage-action-btn", "secondary"])
            .build();
        let current_ssid = network.wifi_ssid.clone();
        disconnect_btn.connect_clicked(move |_| {
            NetworkService::disconnect_wifi(&current_ssid);
        });

        actions_row.append(&forget_btn);
        actions_row.append(&disconnect_btn);
        card.append(&actions_row);

        card
    }

    fn build_ap_row(ap: &WifiAccessPoint) -> gtk4::Box {
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

        row
    }
}
