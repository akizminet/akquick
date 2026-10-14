pub struct BacklightService;

impl BacklightService {
    pub fn fetch() -> u32 {
        crate::translators::ddc::get_monitor_brightness("").unwrap_or(85)
    }

    pub fn set(percent: u32) {
        crate::translators::ddc::set_monitor_brightness("", percent);
    }
}
