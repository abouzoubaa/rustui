// RustUI G5-3b: prioritize a completed local edit, never a borrowed entity.
use crate::{App, Window};
use std::sync::OnceLock;
pub fn fast_draw_first_enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    crate::fast_immediate_enabled()
        && *ON.get_or_init(|| std::env::var("RUSTUI_DRAW_FIRST").as_deref() == Ok("1"))
}
impl Window {
    #[doc(hidden)]
    pub fn fast_draw_after_edit(&self, cx: &mut App) {
        if !fast_draw_first_enabled() {
            return;
        }
        if super::native_scheduler::enabled() && super::native_scheduler::in_callback() {
            super::immediate_present::after_input(self, cx);
            return;
        }
        let handle = self.window_handle();
        cx.fast_defer_before_effects(move |cx| {
            let _ = handle.update(cx, |_, window, cx| {
                crate::fast_latency_config(handle.window_id(), "g53_edit_frame", [0.; 4]);
                crate::fast::immediate_present::try_draw(window, cx);
            });
        });
    }
}
