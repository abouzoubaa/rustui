// RustUI G5-L3: prepare drawable-independent frame work before acquisition.
use std::sync::OnceLock;

pub(crate) fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        gpui::fast_g5_profile_enabled()
            || std::env::var("GPUI_FAST_LATE_DRAWABLE").as_deref() == Ok("1")
    })
}
