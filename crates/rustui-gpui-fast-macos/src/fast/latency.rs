// RustUI PX-4: read existing native window settings, never set them.
use objc2::{msg_send, runtime::AnyObject};
pub(crate) fn settings(window: gpui::WindowId, native: *mut AnyObject, metal_link: bool) {
    if !gpui::fast_latency_enabled() {
        return;
    }
    let screen: *mut AnyObject = unsafe { msg_send![native, screen] };
    let hz = if screen.is_null() {
        0.
    } else {
        let value: isize = unsafe { msg_send![screen, maximumFramesPerSecond] };
        value as f64
    };
    gpui::fast_latency_config(
        window,
        "frame_source",
        [metal_link as u8 as f64, hz, 0., 0.],
    );
}
