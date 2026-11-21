// RustUI G5-1: opt-in drawable lifetime admission. No alternate event runtime.
use block2::RcBlock;
use objc2::{class, msg_send, runtime::ProtocolObject};
use objc2_metal::MTLDrawable;
use objc2_quartz_core::{CAMetalDrawable, CAMetalLayer};
use std::{
    collections::HashMap,
    ptr::NonNull,
    sync::{Arc, Mutex, OnceLock},
    time::Instant,
};
#[derive(Default)]
struct Pool {
    next: u64,
    live: HashMap<u64, usize>,
    wake: Option<Box<dyn Fn() + Send + Sync>>,
    submitted: bool,
}
impl Pool {
    fn acquire(&mut self, key: usize) -> Option<u64> {
        self.next += 1;
        let id = self.next;
        self.live.insert(id, key);
        Some(id)
    }
    fn release(&mut self, key: usize, id: u64) -> bool {
        if self.live.get(&id) == Some(&key) {
            self.live.remove(&id);
            true
        } else {
            false
        }
    }
    fn ready(&self, limit: usize) -> bool {
        self.live.len() < limit
    }
}
pub(crate) struct State(Option<Arc<Mutex<Pool>>>);
impl Default for State {
    fn default() -> Self {
        Self(
            (gpui::fast_immediate_enabled() || gpui::fast_latency_enabled())
                .then(|| Arc::new(Mutex::new(Pool::default()))),
        )
    }
}
impl State {
    pub(crate) fn replacing(
        &self,
        current: &Option<objc2::rc::Retained<ProtocolObject<dyn CAMetalDrawable>>>,
        incoming: &Option<objc2::rc::Retained<ProtocolObject<dyn CAMetalDrawable>>>,
    ) {
        if self.0.is_none() {
            return;
        }
        if let Some(d) = current {
            self.unused(d);
        }
        if let Some(d) = incoming {
            self.track(d);
        }
    }
    pub(crate) fn set_waker(&self, wake: Box<dyn Fn() + Send + Sync>) {
        if let Some(s) = &self.0 {
            s.lock().unwrap().wake = Some(wake);
        }
    }
    pub(crate) fn ready(&self, layer: &CAMetalLayer) -> bool {
        self.0.as_ref().is_some_and(|s| {
            let mut s = s.lock().unwrap();
            let ready = s.ready(layer.maximumDrawableCount());
            if ready {
                s.submitted = false;
            }
            ready
        })
    }
    pub(crate) fn submitted(&self) -> bool {
        self.0.as_ref().is_some_and(|s| s.lock().unwrap().submitted)
    }
    pub(crate) fn did_submit(&self) {
        if let Some(s) = &self.0 {
            s.lock().unwrap().submitted = true;
        }
    }
    pub(crate) fn hold(&self, d: &ProtocolObject<dyn CAMetalDrawable>) -> Held {
        let key = d as *const _ as *const () as usize;
        let ticket = self.0.as_ref().and_then(|s| {
            let id = s
                .lock()
                .unwrap()
                .live
                .iter()
                .filter_map(|(id, k)| (*k == key).then_some(*id))
                .max()?;
            Some((s.clone(), key, id))
        });
        Held(ticket)
    }
    pub(crate) fn track(&self, d: &ProtocolObject<dyn CAMetalDrawable>) {
        let Some(state) = &self.0 else { return };
        let key = d as *const _ as *const () as usize;
        let Some(id) = state.lock().unwrap().acquire(key) else {
            return;
        };
        let state = state.clone();
        let block = RcBlock::new(move |_: NonNull<ProtocolObject<dyn MTLDrawable>>| {
            let mut s = state.lock().unwrap();
            if s.release(key, id) {
                if let Some(w) = &s.wake {
                    w();
                }
            }
        });
        unsafe {
            d.addPresentedHandler(RcBlock::as_ptr(&block));
        }
    }
    pub(crate) fn unused(&self, d: &ProtocolObject<dyn CAMetalDrawable>) {
        if let Some(s) = &self.0 {
            let key = d as *const _ as *const () as usize;
            let mut s = s.lock().unwrap();
            if let Some(id) = s
                .live
                .iter()
                .filter_map(|(id, k)| (*k == key).then_some(*id))
                .max()
            {
                s.release(key, id);
            }
        }
    }
    pub(crate) fn report(&self, layer: &CAMetalLayer) {
        if let Some(frame) = gpui::fast_latency_frame() {
            gpui::fast_latency_config(
                frame.window(),
                "g5_queue",
                [
                    self.count() as f64,
                    layer.maximumDrawableCount() as f64,
                    gpui::fast_immediate_active() as u8 as f64,
                    0.,
                ],
            );
        }
    }
    pub(crate) fn count(&self) -> usize {
        self.0.as_ref().map_or(0, |s| s.lock().unwrap().live.len())
    }
}
pub(crate) struct Held(Option<(Arc<Mutex<Pool>>, usize, u64)>);
impl Held {
    pub(crate) fn submitted(&mut self) {
        self.0 = None;
    }
}
impl Drop for Held {
    fn drop(&mut self) {
        if let Some((state, key, id)) = self.0.take() {
            state.lock().unwrap().release(key, id);
        }
    }
}
pub(crate) fn next_drawable(
    layer: &CAMetalLayer,
    state: &State,
    display_link_attached: bool,
) -> Option<objc2::rc::Retained<ProtocolObject<dyn CAMetalDrawable>>> {
    let active = gpui::fast_immediate_active();
    let count = state.count();
    let limit = layer.maximumDrawableCount();
    if let Some(frame) = gpui::fast_latency_frame() {
        gpui::fast_latency_config(
            frame.window(),
            "g5_acquire_start",
            [
                count as f64,
                limit as f64,
                active as u8 as f64,
                display_link_attached as u8 as f64,
            ],
        );
    }
    // Acquisitions run on the renderer's foreground thread; background handlers
    // can only release slots. Check again here, immediately before the API call.
    if active && (display_link_attached || count >= limit) {
        return None;
    }
    let start = (gpui::fast_latency_enabled() || gpui::fast_immediate_enabled()).then(Instant::now);
    let drawable = layer.nextDrawable();
    if let (Some(start), Some(frame)) = (start, gpui::fast_latency_frame()) {
        gpui::fast_latency_config(
            frame.window(),
            "next_drawable",
            [
                start.elapsed().as_secs_f64() * 1000.,
                count as f64,
                limit as f64,
                active as u8 as f64,
            ],
        );
    }
    if let Some(d) = &drawable {
        state.track(d);
    }
    drawable
}
pub(crate) fn transaction() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    gpui::fast_immediate_enabled()
        && *ON
            .get_or_init(|| std::env::var("GPUI_FAST_IMMEDIATE_TRANSACTION").as_deref() == Ok("1"))
}
pub(crate) fn begin(transaction: bool) {
    if transaction {
        unsafe {
            let _: () = msg_send![class!(CATransaction), begin];
        }
    }
}
pub(crate) fn end(transaction: bool) {
    if transaction {
        unsafe {
            let _: () = msg_send![class!(CATransaction), commit];
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capacity_admission_and_stale_release_cannot_lose_edit() {
        let mut p = Pool::default();
        assert!(!p.ready(0));
        assert!(p.ready(3));
        let a = p.acquire(10).unwrap();
        assert!(p.ready(3));
        let b = p.acquire(10).unwrap();
        assert!(p.ready(3));
        let c = p.acquire(20).unwrap();
        assert_eq!(p.live.len(), 3);
        assert!(!p.ready(3));
        assert!(p.release(10, a));
        assert!(!p.release(10, a));
        assert!(p.ready(3));
        assert!(p.release(10, b));
        assert!(p.release(20, c));
        assert!(p.ready(3));
    }
}
