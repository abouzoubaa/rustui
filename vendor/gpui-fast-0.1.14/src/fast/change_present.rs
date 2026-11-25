// RustUI G5-3b: opt-in per-surface submission cadence, GPUI executor timers.
use crate::{App, Window, WindowId};
use std::{
    collections::{HashMap, HashSet},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};
pub fn fast_coalesce_enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    crate::fast_immediate_enabled()
        && (*ON.get_or_init(|| {
            crate::fast_g5_profile_enabled()
                || std::env::var("GPUI_FAST_IMMEDIATE_COALESCE").as_deref() == Ok("1")
        }) || (super::native_scheduler::enabled() && fast_any_change_enabled())
            || super::native_scheduler::input_scoped_enabled())
}
pub fn fast_any_change_enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    crate::fast_immediate_enabled()
        && *ON.get_or_init(|| {
            crate::fast_g5_profile_enabled()
                || std::env::var("GPUI_FAST_IMMEDIATE_ANY_CHANGE").as_deref() == Ok("1")
        })
}
#[derive(Default)]
struct Surface {
    last: Option<Instant>,
    built: Option<Instant>,
    build_lease: bool,
    version: Option<u64>,
    pending: Option<Instant>,
}
impl Surface {
    fn before_build(&mut self, now: Instant, period: Duration) -> bool {
        self.pending = self
            .built
            .or(self.last)
            .and_then(|last| last.checked_add(period))
            .filter(|due| *due > now);
        if self.pending.is_none() {
            self.built = Some(now);
            self.build_lease = true;
            true
        } else {
            false
        }
    }
    fn consume_build(&mut self, now: Instant, period: Duration, version: u64) -> u8 {
        if std::mem::take(&mut self.build_lease) {
            if self.version.is_some_and(|v| version <= v) {
                1
            } else {
                0
            }
        } else {
            self.admit(now, period, version)
        }
    }
    fn admit(&mut self, now: Instant, period: Duration, version: u64) -> u8 {
        if self.version.is_some_and(|v| version <= v) {
            self.pending = None;
            return 1;
        }
        if let Some(due) = self
            .last
            .and_then(|last| last.checked_add(period))
            .filter(|due| *due > now)
        {
            self.pending = Some(due);
            return 2;
        }
        self.pending = None;
        0
    }
    fn submitted(&mut self, now: Instant, version: u64) {
        self.last = Some(now);
        self.version = Some(version);
        self.pending = None;
    }
}
#[derive(Default)]
struct State {
    surfaces: HashMap<(WindowId, u64), Surface>,
    periods: HashMap<WindowId, Duration>,
    timers: HashSet<WindowId>,
    causes: HashMap<WindowId, u8>,
}
fn state() -> &'static Mutex<State> {
    static S: OnceLock<Mutex<State>> = OnceLock::new();
    S.get_or_init(Default::default)
}
/// Diagnostic bit mask: input=1, binding=2, visible colours=4; zero=other.
pub fn fast_note_frame_cause(w: WindowId, cause: u8) {
    if crate::fast_latency_enabled() {
        *state().lock().unwrap().causes.entry(w).or_default() |= cause;
    }
}
pub(crate) fn drawn(w: WindowId) {
    if crate::fast_latency_enabled() {
        let mask = state().lock().unwrap().causes.remove(&w).unwrap_or(0);
        crate::fast_latency_config(w, "g53_frame_cause", [mask as f64, 0., 0., 0.]);
    }
}
pub(crate) fn configure(window: &Window, cx: &App) {
    if fast_coalesce_enabled() {
        state().lock().unwrap().periods.insert(
            window.window_handle().window_id(),
            window
                .refresh_interval(cx)
                .unwrap_or(Duration::from_secs_f64(1. / 60.)),
        );
    }
}
/// Admission before CPU scene construction. Each renderer owns its own budget;
/// an available overlay never waits for a base surface's interval.
pub fn fast_surface_build_admit(w: WindowId, surface: u64) -> bool {
    if !fast_coalesce_enabled() {
        return true;
    }
    let mut s = state().lock().unwrap();
    let period = s
        .periods
        .get(&w)
        .copied()
        .unwrap_or(Duration::from_secs_f64(1. / 60.));
    s.surfaces
        .entry((w, surface))
        .or_default()
        .before_build(Instant::now(), period)
}
pub(crate) fn before_build(window: &Window, cx: &mut App) -> bool {
    configure(window, cx);
    if !fast_coalesce_enabled() || window.platform_window.immediate_build_ready() {
        return true;
    }
    crate::fast_latency_config(
        window.window_handle().window_id(),
        "g53_cpu_build_deferred",
        [0.; 4],
    );
    arm(window, cx);
    false
}
/// 0 = submit, 1 = same scene already submitted, 2 = next refresh interval.
pub fn fast_surface_admit(surface: u64) -> u8 {
    if !fast_coalesce_enabled() {
        return 0;
    }
    let Some((w, _)) = crate::fast_order_frame() else {
        return 0;
    };
    let version = crate::fast::present_order::scene_version(w);
    let mut s = state().lock().unwrap();
    let period = s
        .periods
        .get(&w)
        .copied()
        .unwrap_or(Duration::from_secs_f64(1. / 60.));
    let reason =
        s.surfaces
            .entry((w, surface))
            .or_default()
            .consume_build(Instant::now(), period, version);
    if reason != 0 {
        crate::fast_latency_config(
            w,
            "g53_surface_coalesced",
            [
                surface as f64,
                version as f64,
                reason as f64,
                period.as_secs_f64() * 1000.,
            ],
        );
    }
    reason
}
pub fn fast_surface_submitted(surface: u64) {
    if !fast_coalesce_enabled() {
        return;
    }
    if let Some((w, _)) = crate::fast_order_frame() {
        state()
            .lock()
            .unwrap()
            .surfaces
            .entry((w, surface))
            .or_default()
            .submitted(Instant::now(), crate::fast::present_order::scene_version(w));
    }
}
pub(crate) fn timer_pending(w: WindowId) -> bool {
    fast_coalesce_enabled() && state().lock().unwrap().timers.contains(&w)
}
pub(crate) fn arm(window: &Window, cx: &mut App) -> bool {
    if !fast_coalesce_enabled() {
        return false;
    }
    let handle = window.window_handle();
    let id = handle.window_id();
    let due = {
        let mut s = state().lock().unwrap();
        let due = s
            .surfaces
            .iter()
            .filter(|((w, _), _)| *w == id)
            .filter_map(|(_, v)| v.pending)
            .min();
        if due.is_none() || !s.timers.insert(id) {
            return due.is_some();
        }
        due.unwrap()
    };
    cx.spawn(async move |cx| {
        cx.background_executor()
            .timer(due.saturating_duration_since(Instant::now()))
            .await;
        state().lock().unwrap().timers.remove(&id);
        let _ = handle.update(cx, |_, window, cx| {
            let _wake = crate::fast_wake_cause(7);
            crate::fast::immediate_present::try_draw(window, cx)
        });
    })
    .detach();
    true
}
pub(crate) fn close(w: WindowId) {
    if fast_coalesce_enabled() {
        let mut s = state().lock().unwrap();
        s.surfaces.retain(|(id, _), _| *id != w);
        s.periods.remove(&w);
        s.causes.remove(&w);
        s.timers.remove(&w);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prebuild_admission_preserves_independent_overlay_and_pending_dirty() {
        let now = Instant::now();
        let period = Duration::from_millis(8);
        let mut base = Surface::default();
        let mut overlay = Surface::default();
        base.submitted(now, 8);
        assert!(!base.before_build(now, period));
        assert_eq!(base.pending, Some(now + period));
        assert!(overlay.before_build(now, period));
        overlay.submitted(now, 9);
        assert!(base.before_build(now + period, period));
        assert!(base.pending.is_none());
    }
    #[test]
    fn early_lease_does_not_add_cpu_scene_time_to_refresh_budget() {
        let now = Instant::now();
        let period = Duration::from_millis(8);
        let mut s = Surface::default();
        assert!(s.before_build(now, period));
        assert_eq!(
            s.consume_build(now + Duration::from_millis(3), period, 1),
            0
        );
        s.submitted(now + Duration::from_millis(3), 1);
        assert!(!s.before_build(now + Duration::from_millis(7), period));
        assert!(s.before_build(now + period, period));
        assert_eq!(
            s.consume_build(now + period + Duration::from_millis(2), period, 2),
            0
        );
        assert!(!s.before_build(now + period + Duration::from_millis(3), period));
    }
    #[test]
    fn same_scene_is_not_submitted_twice() {
        let mut s = Surface::default();
        let t = Instant::now();
        let p = Duration::from_millis(8);
        assert_eq!(s.admit(t, p, 1), 0);
        s.submitted(t, 1);
        assert_eq!(s.admit(t + p, p, 1), 1);
        assert_eq!(s.admit(t + p, p, 2), 0);
    }
    #[test]
    fn new_change_waits_at_most_one_interval_and_does_not_consume_version() {
        let mut s = Surface::default();
        let t = Instant::now();
        let p = Duration::from_millis(8);
        s.submitted(t, 1);
        assert_eq!(s.admit(t + p / 2, p, 2), 2);
        assert_eq!(s.version, Some(1));
        assert_eq!(s.admit(t + p, p, 3), 0);
        s.submitted(t + p, 3);
        assert_eq!(s.version, Some(3));
    }
    #[test]
    fn surfaces_have_independent_refresh_budgets() {
        let t = Instant::now();
        let p = Duration::from_millis(8);
        let mut base = Surface::default();
        let mut overlay = Surface::default();
        base.submitted(t, 1);
        assert_eq!(base.admit(t, p, 2), 2);
        assert_eq!(overlay.admit(t, p, 2), 0);
        overlay.submitted(t, 2);
        assert_eq!(base.admit(t + p, p, 3), 0);
    }
}
