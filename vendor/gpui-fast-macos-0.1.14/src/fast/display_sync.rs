// RustUI G5-L2: windowed, opt-in display-sync experiment.
use std::sync::OnceLock;

pub(crate) fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("GPUI_FAST_NO_DISPLAY_SYNC").as_deref() == Ok("1"))
}

pub(crate) fn keep_sync(fullscreen: bool, transition: bool) -> bool {
    fullscreen || transition
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fullscreen_and_transitions_keep_the_display_synchronized() {
        assert!(!keep_sync(false, false));
        assert!(keep_sync(true, false));
        assert!(keep_sync(false, true));
        assert!(keep_sync(true, true));
    }
}
