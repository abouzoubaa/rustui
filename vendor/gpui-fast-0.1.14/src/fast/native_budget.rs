// L1 diagnostics only: native clock capture and frame correspondence, off by default.
use crate::{PlatformInput, Window, WindowId};
use std::{cell::RefCell, collections::HashMap, sync::OnceLock};
thread_local! {static LATEST:RefCell<HashMap<WindowId,u64>>=RefCell::new(HashMap::new());}
fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("RUSTUI_L1_NATIVE").as_deref() == Ok("1"))
}
#[repr(C)]
#[derive(Default)]
struct NativeInfo {
    version: u64,
    hardware: f64,
    handler: u64,
    kind: u64,
    window: i64,
    cg_timestamp: u64,
}
pub(crate) struct Stamp(Option<crate::FastInputStamp>, u64);
pub(crate) fn begin(window: &Window, event: &PlatformInput) -> Option<Stamp> {
    if !enabled() {
        return None;
    }
    let kind = match event {
        PlatformInput::KeyDown(_) => 0,
        PlatformInput::MouseMove(_) => 1,
        _ => return None,
    };
    #[cfg(target_os = "macos")]
    {
        use std::ffi::{c_char, c_void};
        unsafe extern "C" {
            fn dlsym(h: *mut c_void, n: *const c_char) -> *mut c_void;
        }
        type Take = unsafe extern "C" fn(u32, *mut NativeInfo) -> i32;
        static F: OnceLock<Option<Take>> = OnceLock::new();
        let f = F.get_or_init(|| unsafe {
            let p = dlsym((-2isize) as *mut c_void, c"g5_native_take".as_ptr());
            if p.is_null() {
                None
            } else {
                Some(std::mem::transmute::<*mut c_void, Take>(p))
            }
        });
        let mut n = NativeInfo::default();
        if !f.is_some_and(|f| unsafe { f(kind, &mut n) != 0 }) {
            return None;
        }
        crate::fast_latency_config(
            window.window_handle().window_id(),
            "l1_native",
            [
                n.version as f64,
                n.hardware,
                n.handler as f64,
                n.window as f64,
            ],
        );
        return Some(Stamp(
            window.fast_measure_input("native", n.version, n.version),
            n.version,
        ));
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = kind;
        None
    }
}
pub(crate) fn end(window: &Window, stamp: Option<Stamp>) {
    if let Some(Stamp(stamp, v)) = stamp {
        window.fast_input_handled(stamp);
        LATEST.with(|s| {
            s.borrow_mut().insert(window.window_handle().window_id(), v);
        });
    }
}
pub(crate) fn draw_end(window: &Window) {
    if enabled() {
        crate::fast_latency_config(window.window_handle().window_id(), "l1_draw_end", [0.; 4]);
    }
}
pub(crate) fn paint(window: &mut Window) {
    if !enabled() || std::env::var("RUSTUI_G5_PIXEL_CAPTURE").as_deref() != Ok("1") {
        return;
    }
    let version = LATEST.with(|s| {
        s.borrow()
            .get(&window.window_handle().window_id())
            .copied()
            .unwrap_or(0)
    });
    for column in 0..26 {
        let color = if column == 0 || column == 25 {
            0xff0000
        } else {
            let bit = if column <= 8 {
                (0xa5 >> (8 - column)) & 1
            } else {
                (version >> (24 - column)) & 1
            };
            if bit == 1 { 0xffffff } else { 0 }
        };
        window.paint_quad(crate::fill(
            crate::Bounds::new(
                crate::point(crate::px(1000. + column as f32 * 4.), crate::px(10.)),
                crate::size(crate::px(4.), crate::px(32.)),
            ),
            crate::rgb(color),
        ));
    }
}
