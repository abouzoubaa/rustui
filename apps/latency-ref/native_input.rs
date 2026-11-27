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

pub fn physical() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("RUSTUI_L1_PHYSICAL").as_deref() == Ok("1"))
}

pub fn physical_count(arm: i32) -> u64 {
    type Call = unsafe extern "C" fn(i32) -> u64;
    static FUNCTION: OnceLock<Call> = OnceLock::new();
    let call = FUNCTION.get_or_init(|| unsafe {
        let pointer = dlsym((-2isize) as *mut c_void, c"g5_native_physical".as_ptr());
        assert!(!pointer.is_null(), "native keyboard helper was not loaded");
        std::mem::transmute::<*mut c_void, Call>(pointer)
    });
    unsafe { call(arm) }
}

/// The CPU measurement helpers are loaded only in local latency runs.
#[allow(dead_code)]
mod cpu_probe {
    use super::*;

    fn enabled() -> bool {
        static ON: OnceLock<bool> = OnceLock::new();
        *ON.get_or_init(|| std::env::var("RUSTUI_G5_L8_CPU").as_deref() == Ok("1"))
    }

    pub fn boundary(phase: u32) {
        if !enabled() {
            return;
        }
        unsafe {
            let symbol = dlsym((-2isize) as *mut c_void, c"g5_cpu_boundary".as_ptr());
            assert!(!symbol.is_null(), "CPU boundary helper was not loaded");
            std::mem::transmute::<*mut c_void, unsafe extern "C" fn(u32)>(symbol)(phase);
        }
    }

    pub fn idle_mode() -> Option<bool> {
        if !enabled() {
            return None;
        }
        match std::env::var("RUSTUI_G5_L8_IDLE").as_deref() {
            Ok("focused") => Some(false),
            Ok("unfocused") => Some(true),
            _ => None,
        }
    }

    pub fn prepare_idle(unfocused: bool) {
        unsafe {
            let symbol = dlsym((-2isize) as *mut c_void, c"g5_cpu_idle_prepare".as_ptr());
            assert!(!symbol.is_null(), "CPU focus helper was not loaded");
            std::mem::transmute::<*mut c_void, unsafe extern "C" fn(i32)>(symbol)(i32::from(
                unfocused,
            ));
        }
    }
}
#[allow(unused_imports)]
pub use cpu_probe::{
    boundary as cpu_boundary, idle_mode as cpu_idle_mode, prepare_idle as cpu_prepare_idle,
};

#[allow(dead_code)]
pub fn fullscreen_entered() -> bool {
    type F = unsafe extern "C" fn() -> f64;
    static FNC: OnceLock<F> = OnceLock::new();
    let callback = FNC.get_or_init(|| unsafe {
        let symbol = dlsym((-2isize) as *mut c_void, c"g_fs1_entered_ms".as_ptr());
        assert!(!symbol.is_null(), "missing native fullscreen observer");
        std::mem::transmute::<*mut c_void, F>(symbol)
    });
    unsafe { callback() > 0.0 }
}
