// RustUI G5-1: opt-in input-tick presentation on GPUI's executor.
use crate::{AnyWindowHandle, App, PlatformWindow, Window, WindowId};
use futures::{StreamExt, channel::mpsc};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    marker::PhantomData,
    rc::Rc,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
thread_local! { static ACTIVE: Cell<bool> = const {Cell::new(false)}; }
pub fn fast_immediate_enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        [
            "GPUI_FAST_IMMEDIATE_PRESENT",
            "GPUI_FAST_FORCE_LEGACY_LINK",
            "GPUI_FAST_VIEW_DISPLAY_LINK",
        ]
        .iter()
        .any(|key| std::env::var(key).as_deref() == Ok("1"))
    })
}
pub fn fast_immediate_active() -> bool {
    ACTIVE.with(Cell::get)
}
struct Scope {
    previous: bool,
    _thread: PhantomData<Rc<()>>,
}
impl Scope {
    fn new() -> Self {
        Self {
            previous: ACTIVE.with(|a| a.replace(true)),
            _thread: PhantomData,
        }
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        ACTIVE.with(|a| a.set(self.previous));
    }
}
// Admission gate: dirty demand waits for drawable availability, never spins.
#[derive(Default)]
struct Admission {
    waiting: AtomicBool,
}
impl Admission {
    fn block(&self) {
        self.waiting.store(true, Ordering::Release);
    }
    fn available(&self) {
        self.waiting.store(false, Ordering::Release);
    }
    fn accepts_demand(&self) -> bool {
        !self.waiting.load(Ordering::Acquire)
    }
}
// Hold clock: pure, independently tested; zero disables only the hold.
#[derive(Default)]
struct Activity {
    last_input: Option<Instant>,
    until: Option<Instant>,
    timer_running: bool,
}
impl Activity {
    fn note_input(&mut self, now: Instant, duration: Duration) -> bool {
        let burst = self
            .last_input
            .replace(now)
            .and_then(|last| now.checked_duration_since(last))
            .is_some_and(|gap| gap < Duration::from_millis(100));
        if burst {
            self.note(now, duration)
        } else {
            self.until = None;
            false
        }
    }
    fn note(&mut self, now: Instant, duration: Duration) -> bool {
        if duration.is_zero() {
            return false;
        }
        self.until = now.checked_add(duration);
        if !self.timer_running && self.until.is_some() {
            self.timer_running = true;
            true
        } else {
            false
        }
    }
    fn remaining(&self, now: Instant) -> Option<Duration> {
        self.until
            .and_then(|until| until.checked_duration_since(now))
            .filter(|d| !d.is_zero())
    }
    fn expire(&mut self, now: Instant) -> bool {
        if self.remaining(now).is_some() {
            false
        } else {
            self.until = None;
            self.timer_running = false;
            true
        }
    }
}
fn hold_duration() -> Duration {
    static MS: OnceLock<u64> = OnceLock::new();
    Duration::from_millis(*MS.get_or_init(|| {
        std::env::var("GPUI_FAST_IMMEDIATE_HOLD_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(250)
    }))
}
// Per-window state is removed on close; no retained Window or alternate runtime.
struct Entry {
    activity: Activity,
    send: mpsc::UnboundedSender<bool>,
    admission: Arc<Admission>,
}
thread_local! {static WINDOWS:RefCell<HashMap<WindowId,Entry>>=RefCell::new(HashMap::new());}
pub fn fast_immediate_hold_active(id: WindowId) -> bool {
    fast_immediate_enabled()
        && !crate::fast_frame_continuous(id)
        && !crate::fast_frame_uses_view(id)
        && WINDOWS.with(|w| {
            w.borrow()
                .get(&id)
                .is_some_and(|e| e.activity.remaining(Instant::now()).is_some())
        })
}
pub fn fast_immediate_demand(id: WindowId) -> bool {
    if !fast_immediate_hold_active(id) && !crate::fast_any_change_enabled() {
        return false;
    }
    if crate::fast::change_present::timer_pending(id) {
        return true;
    }
    WINDOWS.with(|w| {
        if let Some(e) = w.borrow().get(&id) {
            if e.admission.accepts_demand() {
                let _ = e.send.unbounded_send(true);
            }
        }
    });
    true
}
pub fn fast_immediate_suspend(id: WindowId) {
    if fast_immediate_enabled() {
        WINDOWS.with(|w| {
            if let Some(e) = w.borrow_mut().get_mut(&id) {
                e.activity.until = None;
            }
        });
    }
}
pub(crate) fn close(id: WindowId) {
    crate::fast::present_order::close(id);
    crate::fast::change_present::close(id);
    crate::fast::frame_pacing::close(id);
    if fast_immediate_enabled() {
        WINDOWS.with(|w| {
            if let Some(e) = w.borrow_mut().remove(&id) {
                let _ = e.send.unbounded_send(false);
            }
        });
    }
}
pub(crate) fn before_input(window: &Window, cx: &mut App) {
    crate::fast::change_present::fast_note_frame_cause(window.handle.window_id(), 1);
    let id = window.handle.window_id();
    if crate::fast_latency_enabled() {
        crate::fast_latency_config(
            id,
            "g5_input_state",
            window.platform_window.immediate_input_state(),
        );
    }
    if !fast_immediate_enabled() || !window.platform_window.immediate_supported() {
        return;
    }
    if crate::fast_frame_continuous(id) || crate::fast_frame_uses_view(id) {
        return;
    }
    let start = WINDOWS.with(|w| {
        w.borrow_mut()
            .get_mut(&id)
            .is_some_and(|e| e.activity.note_input(Instant::now(), hold_duration()))
    });
    if !start {
        return;
    }
    let handle = window.handle;
    cx.spawn(async move |cx| {
        loop {
            let remaining = WINDOWS.with(|w| {
                w.borrow()
                    .get(&id)
                    .and_then(|e| e.activity.remaining(Instant::now()))
            });
            if let Some(wait) = remaining {
                cx.background_executor().timer(wait).await;
                continue;
            }
            WINDOWS.with(|w| {
                if let Some(e) = w.borrow_mut().get_mut(&id) {
                    e.activity.expire(Instant::now());
                }
            });
            let _ = handle.update(cx, |_, window, _| {
                crate::fast_latency_config(id, "g5_hold_expired", [0.; 4]);
                if window.invalidator.is_dirty() || !window.next_frame_callbacks.borrow().is_empty()
                {
                    window.platform_window.schedule_frame();
                }
            });
            break;
        }
    })
    .detach();
}
pub(crate) fn scheduled(handle: AnyWindowHandle, force: bool, cx: &mut crate::AsyncApp) -> bool {
    if !fast_immediate_hold_active(handle.window_id()) && !crate::fast_any_change_enabled() {
        return false;
    }
    if !handle
        .update(cx, |_, window, _| {
            window.platform_window.immediate_supported()
        })
        .unwrap_or(false)
    {
        return false;
    }
    if force {
        let _ = handle.update(cx, |_, window, _| window.refresh());
    }
    fast_immediate_demand(handle.window_id());
    true
}
pub(crate) fn try_draw(window: &mut Window, cx: &mut App) {
    crate::fast::change_present::configure(window, cx);
    window.platform_window.resume_frame_source_after_drain();
    if crate::fast_frame_continuous(window.handle.window_id()) {
        window.platform_window.schedule_frame();
        return;
    }
    if crate::window::draw_in_progress() {
        return;
    }
    let callbacks = (fast_immediate_hold_active(window.handle.window_id())
        || crate::fast_any_change_enabled())
        && !window.next_frame_callbacks.borrow().is_empty();
    if !window.invalidator.is_dirty() && !callbacks {
        return;
    }
    if !crate::fast::change_present::before_build(window, cx) {
        return;
    }
    let admission = WINDOWS.with(|w| {
        w.borrow()
            .get(&window.handle.window_id())
            .map(|e| e.admission.clone())
    });
    if let Some(g) = &admission {
        g.block();
    }
    if !window.platform_window.can_present_immediately() {
        return;
    }
    if let Some(g) = &admission {
        g.available();
    }
    if callbacks {
        if !crate::fast_coalesce_enabled() {
            window.refresh();
        }
        for callback in window.next_frame_callbacks.take() {
            callback(window, cx);
        }
    }
    if crate::fast_coalesce_enabled() && !window.invalidator.is_dirty() {
        return;
    }
    let _scope = Scope::new();
    let arena = window.draw(cx);
    window.present();
    arena.clear(cx);
    if crate::fast::change_present::arm(window, cx) {
        return;
    }
    if !window.platform_window.immediate_submitted() {
        window.refresh();
        // A full pool at the acquisition-site recheck keeps dirty work queued
        // for the existing presented-handler wake instead of recreating a link.
        if let Some(g) = &admission {
            g.block();
        }
        if !window.platform_window.can_present_immediately() {
            return;
        }
        if let Some(g) = &admission {
            g.available();
        }
        fast_immediate_suspend(window.handle.window_id());
        window.platform_window.schedule_frame();
    } else if window.invalidator.is_dirty() || !window.next_frame_callbacks.borrow().is_empty() {
        window.platform_window.schedule_frame();
    }
}
pub(crate) fn after_input(window: &Window, cx: &mut App) {
    if !fast_immediate_enabled() {
        return;
    }
    if crate::fast_frame_continuous(window.handle.window_id()) {
        window.platform_window.schedule_frame();
        return;
    }
    let handle = window.handle;
    cx.defer(move |cx| {
        let _ = handle.update(cx, |_, window, cx| try_draw(window, cx));
    });
}
pub(crate) fn install(handle: AnyWindowHandle, platform: &dyn PlatformWindow, cx: &mut App) {
    if !fast_immediate_enabled() {
        return;
    }
    let id = handle.window_id();
    crate::fast::frame_pacing::install(id, platform.immediate_uses_view_link());
    let (send, mut receive) = mpsc::unbounded();
    let admission = Arc::new(Admission::default());
    let ready = admission.clone();
    WINDOWS.with(|w| {
        w.borrow_mut().insert(
            id,
            Entry {
                activity: Activity::default(),
                send: send.clone(),
                admission,
            },
        );
    });
    platform.on_drawable_available(Box::new(move || {
        ready.available();
        let _ = send.unbounded_send(true);
    }));
    cx.spawn(async move |cx| {
        while receive.next().await == Some(true) {
            if handle
                .update(cx, |_, window, cx| try_draw(window, cx))
                .is_err()
            {
                break;
            }
        }
        WINDOWS.with(|w| {
            w.borrow_mut().remove(&id);
        });
    })
    .detach();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scope_restores_nested_and_unwound_state() {
        assert!(!fast_immediate_active());
        {
            let _a = Scope::new();
            {
                let _b = Scope::new();
                assert!(fast_immediate_active());
            }
            assert!(fast_immediate_active());
        }
        assert!(!fast_immediate_active());
    }
    #[test]
    fn active_inputs_extend_one_timer_and_expiry() {
        let t = Instant::now();
        let mut a = Activity::default();
        assert!(a.note(t, Duration::from_millis(250)));
        assert!(!a.note(t + Duration::from_millis(200), Duration::from_millis(250)));
        assert!(!a.expire(t + Duration::from_millis(250)));
        assert_eq!(
            a.remaining(t + Duration::from_millis(300)),
            Some(Duration::from_millis(150))
        );
        assert!(a.expire(t + Duration::from_millis(450)));
        assert!(a.note(t + Duration::from_millis(500), Duration::from_millis(250)));
    }
    #[test]
    fn blocked_dirty_demand_waits_for_availability() {
        let a = Arc::new(Admission::default());
        assert!(a.accepts_demand());
        a.block();
        for _ in 0..10000 {
            assert!(!a.accepts_demand());
        }
        let other = a.clone();
        std::thread::spawn(move || other.available())
            .join()
            .unwrap();
        assert!(a.accepts_demand());
        a.block();
        a.available();
        assert!(a.accepts_demand());
    }
    #[test]
    fn hold_requires_two_inputs_strictly_under_100_ms() {
        let now = Instant::now();
        let duration = Duration::from_millis(250);
        let mut a = Activity::default();
        assert!(!a.note_input(now, duration));
        assert!(a.remaining(now).is_none());
        assert!(!a.note_input(now + Duration::from_millis(100), duration));
        assert!(a.remaining(now + Duration::from_millis(100)).is_none());
        assert!(a.note_input(now + Duration::from_millis(199), duration));
        assert!(a.remaining(now + Duration::from_millis(200)).is_some());
        assert!(!a.note_input(now + Duration::from_millis(299), duration));
        assert!(a.remaining(now + Duration::from_millis(299)).is_none());
        assert!(a.expire(now + Duration::from_millis(299)));
        assert!(a.note_input(now + Duration::from_millis(332), duration));
        assert!(a.remaining(now + Duration::from_millis(333)).is_some());
        let mut disabled = Activity::default();
        assert!(!disabled.note_input(now, Duration::ZERO));
        assert!(!disabled.note_input(now + Duration::from_millis(33), Duration::ZERO));
        assert!(
            disabled
                .remaining(now + Duration::from_millis(33))
                .is_none()
        );
    }
    #[test]
    fn zero_hold_and_windows_are_independent() {
        let t = Instant::now();
        let mut a = Activity::default();
        let b = Activity::default();
        assert!(!a.note(t, Duration::ZERO));
        assert!(a.remaining(t).is_none());
        a.note(t, Duration::from_millis(250));
        assert!(b.remaining(t).is_none());
    }
}
