use crate::WindowId;
use std::{collections::HashMap, sync::OnceLock};
// Independent source cohorts per presentation surface; surface 0 holds window metadata.
#[derive(Default)]
struct PresentOrder {
    source: Option<u8>,
    live: usize,
    last_submitted: u64,
    painted: u64,
    deferred: bool,
}
impl PresentOrder {
    fn drained(&self) -> bool {
        self.live == 0
    }
    fn ready(&self, source: u8) -> bool {
        self.live == 0 || self.source == Some(source)
    }
    fn acquire(&mut self, source: u8) -> bool {
        if !self.ready(source) {
            return false;
        }
        self.source = Some(source);
        self.live += 1;
        true
    }
    fn submit(&mut self, version: u64) -> bool {
        if version < self.last_submitted {
            return false;
        }
        self.last_submitted = version;
        true
    }
    fn release(&mut self) {
        assert!(self.live > 0);
        self.live -= 1;
    }
    fn defer(&mut self) {
        self.deferred = true;
    }
    fn take_deferred(&mut self) -> bool {
        std::mem::take(&mut self.deferred)
    }
}
type SharedOrder = std::sync::Arc<std::sync::Mutex<PresentOrder>>;
fn orders() -> &'static std::sync::Mutex<HashMap<(WindowId, u64), SharedOrder>> {
    static ORDERS: OnceLock<std::sync::Mutex<HashMap<(WindowId, u64), SharedOrder>>> =
        OnceLock::new();
    ORDERS.get_or_init(Default::default)
}
fn order(window: WindowId, surface: u64) -> SharedOrder {
    orders()
        .lock()
        .unwrap()
        .entry((window, surface))
        .or_default()
        .clone()
}
thread_local! {static ORDER_FRAME: std::cell::Cell<Option<(WindowId,u64)>>=const{std::cell::Cell::new(None)};}
pub(crate) fn scene_version(window: WindowId) -> u64 {
    order(window, 0).lock().unwrap().painted
}
pub(crate) fn painted(window: WindowId) {
    if crate::fast_immediate_enabled() {
        let s = order(window, 0);
        let mut s = s.lock().unwrap();
        s.painted = s.painted.wrapping_add(1);
    }
}
pub(crate) struct OrderScope(Option<(WindowId, u64)>);
pub(crate) fn present_scope(window: WindowId) -> OrderScope {
    let next = crate::fast_immediate_enabled().then(|| {
        let s = order(window, 0);
        let s = s.lock().unwrap();
        (
            window,
            if crate::fast::input_latency::enabled() {
                crate::fast_present_tag().map_or(0, |tag| tag.fast_version())
            } else {
                s.painted
            },
        )
    });
    OrderScope(ORDER_FRAME.with(|f| f.replace(next)))
}
impl Drop for OrderScope {
    fn drop(&mut self) {
        ORDER_FRAME.with(|f| f.set(self.0));
    }
}
/// Current window and painted version for the guarded present scope.
pub fn fast_order_frame() -> Option<(WindowId, u64)> {
    ORDER_FRAME.with(std::cell::Cell::get)
}
/// Keep the current scene dirty if guarded presentation could not submit.
pub fn fast_order_defer() {
    if let Some((window, _)) = fast_order_frame() {
        order(window, 0).lock().unwrap().defer();
    }
}
pub(crate) fn retry(window: &mut crate::Window) {
    if crate::fast_immediate_enabled()
        && order(window.window_handle().window_id(), 0)
            .lock()
            .unwrap()
            .take_deferred()
    {
        window.refresh();
    }
}

/// Whether this source can join the current surface drawable cohort.
pub fn fast_order_drained(window: WindowId, surface: u64) -> bool {
    !crate::fast_immediate_enabled() || order(window, surface).lock().unwrap().drained()
}
pub fn fast_order_ready(window: WindowId, surface: u64, source: u8) -> bool {
    !crate::fast_immediate_enabled() || order(window, surface).lock().unwrap().ready(source)
}
/// One acquired drawable lifetime in a surface source cohort.
pub struct FastOrderTicket {
    state: SharedOrder,
}
impl FastOrderTicket {
    /// Reject a version older than the most recently submitted scene.
    pub fn submit(&self, version: u64) -> bool {
        self.state.lock().unwrap().submit(version)
    }
}
impl Drop for FastOrderTicket {
    fn drop(&mut self) {
        self.state.lock().unwrap().release();
    }
}
/// Reserve a drawable lifetime, refusing a different live source.
pub fn fast_order_acquire(window: WindowId, surface: u64, source: u8) -> Option<FastOrderTicket> {
    let state = order(window, surface);
    if !state.lock().unwrap().acquire(source) {
        return None;
    }
    Some(FastOrderTicket { state })
}
#[cfg(test)]
mod order_tests {
    use super::*;
    #[test]
    fn base_typing_does_not_defer_popover_or_extra_surface() {
        let base = order(WindowId::default(), 90011);
        let overlay = order(WindowId::default(), 90012);
        let extra = order(WindowId::default(), 90013);
        assert!(base.lock().unwrap().acquire(1));
        assert!(base.lock().unwrap().submit(500));
        for surface in [&overlay, &extra] {
            let mut s = surface.lock().unwrap();
            assert!(s.acquire(0));
            assert!(s.submit(1));
            assert!(!s.deferred);
            s.release();
        }
        assert_eq!(base.lock().unwrap().live, 1);
        base.lock().unwrap().release();
    }
    #[test]
    fn overlay_drag_progresses_while_base_pool_is_full() {
        let base = order(WindowId::default(), 90021);
        let overlay = order(WindowId::default(), 90022);
        for _ in 0..3 {
            assert!(base.lock().unwrap().acquire(1));
        }
        for version in 1..=120 {
            let mut s = overlay.lock().unwrap();
            assert!(s.acquire(0));
            assert!(s.submit(version));
            s.release();
            assert!(s.drained());
        }
        assert_eq!(base.lock().unwrap().live, 3);
        assert_eq!(overlay.lock().unwrap().last_submitted, 120);
        for _ in 0..3 {
            base.lock().unwrap().release();
        }
    }
    #[test]
    fn restarting_the_same_source_requires_complete_drain() {
        let mut state = PresentOrder::default();
        assert!(state.drained());
        assert!(state.acquire(1));
        assert!(state.ready(1));
        assert!(!state.drained());
        state.release();
        assert!(state.drained());
    }
    #[test]
    fn scheduled_future_and_immediate_reproduce_8_6_7_then_guard_drains() {
        // The recorded source cohort has future displays at t=30 and t=40;
        // an immediate drawable would display at t=20 despite newer state.
        let mut unguarded = vec![(30, 6), (40, 7), (20, 8)];
        unguarded.sort();
        assert_eq!(unguarded.iter().map(|x| x.1).collect::<Vec<_>>(), [8, 6, 7]);
        for surface in 1..=3 {
            let state = order(WindowId::default(), 90030 + surface);
            let mut gate = state.lock().unwrap();
            let mut displayed = vec![];
            assert!(gate.acquire(1));
            assert!(gate.submit(6));
            assert!(gate.acquire(1));
            assert!(gate.submit(7));
            assert!(!gate.acquire(0));
            assert_eq!(gate.live, 2);
            displayed.push(6);
            gate.release();
            assert!(!gate.acquire(0));
            displayed.push(7);
            gate.release();
            assert!(gate.acquire(0));
            assert!(gate.submit(8));
            displayed.push(8);
            gate.release();
            assert_eq!(displayed, [6, 7, 8]);
            assert!(!gate.submit(6));
            assert!(!gate.submit(7));
            assert!(gate.submit(8));
            assert!(gate.submit(9));
        }
    }
    #[test]
    fn declined_present_preserves_one_retry_without_consuming_the_scene() {
        let mut state = PresentOrder::default();
        assert!(state.acquire(1));
        assert!(state.submit(6));
        assert!(!state.ready(0));
        state.defer();
        assert!(state.take_deferred());
        assert!(!state.take_deferred());
        state.release();
        assert!(state.acquire(0));
        assert!(state.submit(7));
        state.release();
    }
    #[test]
    fn source_switch_waits_for_its_surface_and_equal_versions_are_allowed() {
        let mut gate = PresentOrder::default();
        for _ in 0..3 {
            assert!(gate.acquire(1));
            assert!(gate.submit(4));
        }
        for _ in 0..2 {
            gate.release();
            assert!(!gate.ready(0));
        }
        gate.release();
        assert!(gate.acquire(0));
        assert!(gate.submit(5));
        assert!(!gate.ready(1));
        assert!(!gate.submit(4));
        gate.release();
        assert!(gate.ready(1));
    }
}

pub(crate) fn close(window: WindowId) {
    if crate::fast_immediate_enabled() {
        orders().lock().unwrap().retain(|(w, _), _| *w != window);
    }
}
