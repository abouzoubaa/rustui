// G5-F1: keep AppKit event delivery and frame scheduling in separate turns.
use crate::WindowId;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    sync::OnceLock,
    time::{Duration, Instant},
};

thread_local! {
    static CALLBACK: Cell<u8> = const { Cell::new(0) };
    static CAUSE: Cell<u8> = const { Cell::new(0) };
    static POST_EVENT: RefCell<HashMap<WindowId, Box<dyn FnOnce()>>> = RefCell::new(HashMap::new());
    static TRACE_COUNT: Cell<(u64, usize)> = const { Cell::new((0, 0)) };
}

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    crate::fast_immediate_enabled()
        && *ON.get_or_init(|| std::env::var("GPUI_FAST_NATIVE_SCHEDULER").as_deref() == Ok("1"))
}

pub(crate) fn input_scoped_enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    enabled()
        && *ON.get_or_init(|| std::env::var("GPUI_FAST_INPUT_SCOPED_DEMAND").as_deref() == Ok("1"))
}

pub(crate) fn trace_point(stage: &'static str, detail: u8) {
    if tracing() {
        eprintln!(
            "G5_F2_TRACE {{\"mach\":{},\"stage\":\"{stage}\",\"detail\":{detail}}}",
            crate::fast_latency_tick()
        );
    }
}

pub fn tracing() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("GPUI_FAST_TRACE_WAKES").as_deref() == Ok("1"))
}

/// An AppKit call stack is not a safe place to build or present a frame.
pub struct CallbackScope(Option<u8>, u8);
pub fn fast_native_callback(kind: u8) -> CallbackScope {
    trace_point("callback_enter", kind);
    CallbackScope(
        (enabled() || tracing()).then(|| CALLBACK.with(|slot| slot.replace(slot.get() | kind))),
        kind,
    )
}
impl Drop for CallbackScope {
    fn drop(&mut self) {
        trace_point("callback_exit", self.1);
        if let Some(previous) = self.0 {
            CALLBACK.with(|slot| slot.set(previous));
            if previous == 0 {
                let pending = POST_EVENT.with(|pending| std::mem::take(&mut *pending.borrow_mut()));
                for wake in pending.into_values() {
                    trace_point("post_event_release", 1);
                    wake();
                }
            }
        }
    }
}

pub(super) fn in_callback() -> bool {
    CALLBACK.with(|slot| slot.get() != 0)
}

pub(super) fn post_event(window: WindowId, wake: impl FnOnce() + 'static) {
    if in_callback() {
        POST_EVENT.with(|pending| {
            pending
                .borrow_mut()
                .entry(window)
                .or_insert_with(|| Box::new(wake));
        });
    } else {
        wake();
    }
}

pub struct WakeScope(Option<u8>);
pub fn fast_wake_cause(cause: u8) -> WakeScope {
    WakeScope((enabled() || tracing()).then(|| CAUSE.with(|slot| slot.replace(cause))))
}
impl Drop for WakeScope {
    fn drop(&mut self) {
        if let Some(previous) = self.0 {
            CAUSE.with(|slot| slot.set(previous));
        }
    }
}
pub(crate) fn cause() -> u8 {
    CAUSE.with(Cell::get)
}

#[derive(Default)]
pub(super) struct Watchdog {
    version: u64,
    first: Option<Instant>,
    count: u8,
    until: Option<Instant>,
}
impl Watchdog {
    pub(super) fn cooling_down(&self, now: Instant) -> bool {
        self.until.is_some_and(|until| now < until)
    }
    pub(super) fn wake(&mut self, version: u64, now: Instant, refresh: Duration) -> bool {
        if self.version != version || self.first.is_none_or(|first| now - first >= refresh) {
            self.version = version;
            self.first = Some(now);
            self.count = 0;
        }
        self.count += 1;
        if self.count > 3 {
            self.until = Some(now + Duration::from_millis(250));
            true
        } else {
            false
        }
    }
}

pub(super) fn trace(window: WindowId, reason: u8, dirty: bool, version: u64, scene: u64) {
    if !tracing() {
        return;
    }
    let callback = CALLBACK.with(Cell::get);
    // Trace-only captures are untimed. Emit a bounded live prefix so a hung app
    // still leaves the cycle on disk. The frame recorder remains buffered.
    TRACE_COUNT.with(|count| {
        let (last, repeats) = count.get();
        let repeats = if last == version { repeats } else { 0 };
        if repeats < 48 || repeats.is_power_of_two() {
            eprintln!("G5_WAKE {{\"window\":\"{window:?}\",\"cause\":{reason},\"dirty\":{dirty},\"generation\":{scene},\"input\":{version},\"callback_mask\":{callback}}}");
        }
        count.set((version, repeats.saturating_add(1)));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    #[test]
    fn nested_text_callbacks_drain_one_wake_after_the_outer_event() {
        let window = WindowId::default();
        let marks = Rc::new(Cell::new(0));
        let outer = CallbackScope(Some(CALLBACK.with(|slot| slot.replace(1))), 1);
        let nested = CallbackScope(Some(CALLBACK.with(|slot| slot.replace(5))), 4);
        let first = marks.clone();
        post_event(window, move || first.set(first.get() + 1));
        post_event(window, || panic!("duplicate event wake"));
        drop(nested);
        assert_eq!(marks.get(), 0);
        drop(outer);
        assert_eq!(marks.get(), 1);
    }

    #[test]
    fn fourth_wake_of_one_input_in_one_refresh_backs_off_for_250_ms() {
        let now = Instant::now();
        let refresh = Duration::from_millis(8);
        let mut watchdog = Watchdog::default();
        for _ in 0..3 {
            assert!(!watchdog.wake(7, now, refresh));
        }
        assert!(watchdog.wake(7, now, refresh));
        assert!(watchdog.cooling_down(now + Duration::from_millis(249)));
        assert!(!watchdog.cooling_down(now + Duration::from_millis(250)));
        assert!(!watchdog.wake(8, now + refresh, refresh));
    }
}
