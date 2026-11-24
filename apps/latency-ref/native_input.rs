//! Local native-event driver shared by the unshipped reference and G0 diagnostics.
use std::{
    ffi::{c_char, c_void},
    sync::OnceLock,
};
unsafe extern "C" {
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
}
pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("RUSTUI_L1_NATIVE").as_deref() == Ok("1"))
}
pub fn post(kind: u32, version: u64, x: f64, y: f64) -> bool {
    type F = unsafe extern "C" fn(u32, u64, f64, f64) -> i32;
    static FNC: OnceLock<Option<F>> = OnceLock::new();
    let f = FNC.get_or_init(|| unsafe {
        let p = dlsym((-2isize) as *mut c_void, c"g5_native_post".as_ptr());
        if p.is_null() {
            None
        } else {
            Some(std::mem::transmute::<*mut c_void, F>(p))
        }
    });
    f.is_some_and(|f| unsafe { f(kind, version, x, y) != 0 })
}
pub fn pixels_state() -> i32 {
    unsafe {
        let p = dlsym((-2isize) as *mut c_void, c"g5_pixels_state".as_ptr());
        if p.is_null() {
            -1
        } else {
            std::mem::transmute::<*mut c_void, unsafe extern "C" fn() -> i32>(p)()
        }
    }
}
pub fn pixels_stop() {
    unsafe {
        let p = dlsym((-2isize) as *mut c_void, c"g5_pixels_stop".as_ptr());
        if !p.is_null() {
            std::mem::transmute::<*mut c_void, unsafe extern "C" fn()>(p)();
        }
    }
}
