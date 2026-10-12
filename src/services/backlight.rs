pub struct BacklightService;

impl BacklightService {
    pub fn fetch() -> u32 {
        crate::translators::sysfs::read_backlight_percent().unwrap_or(85)
    }

    pub fn set(percent: u32) {
        crate::translators::sysfs::set_backlight_percent(percent);
    }
}
