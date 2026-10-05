use std::process::Command;

pub struct ActionService;

impl ActionService {
    pub fn take_screenshot() {
        let _ = Command::new("satty")
            .args(["--filename", "/tmp/screenshot.png"])
            .spawn()
            .or_else(|_| {
                Command::new("sh")
                    .arg("-c")
                    .arg("grim -g \"$(slurp)\" /tmp/screenshot.png && satty -f /tmp/screenshot.png")
                    .spawn()
            });
    }

    pub fn open_settings() {
        let _ = Command::new("gnome-control-center")
            .spawn()
            .or_else(|_| Command::new("nmrs-gui").spawn());
    }

    pub fn lock_session() {
        let _ = Command::new("hyprlock").spawn();
    }

    pub fn power_menu() {
        let _ = Command::new("/var/home/phamnv/.config/rofi/powermenu.sh")
            .spawn()
            .or_else(|_| Command::new("wlogout").spawn());
    }
}
