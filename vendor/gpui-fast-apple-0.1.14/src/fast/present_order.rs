use block2::RcBlock;
use objc2::runtime::ProtocolObject;
use objc2_metal::MTLDrawable;
use std::{cell::Cell, ptr::NonNull, sync::Arc};
pub(crate) fn acquisition_allowed(
    enabled: bool,
    attached: bool,
    count: usize,
    limit: usize,
) -> bool {
    !enabled || (!attached && count < limit)
}
// Track complete drawable lifetimes separately from command-buffer scheduling.
// 0 = nextDrawable (NSView/CV/immediate); 1 = supplied Metal-link drawable.
use std::{collections::HashMap, sync::Mutex};
struct LiveOrder {
    next: u64,
    tickets: HashMap<u64, (usize, gpui::FastOrderTicket)>,
    wake: Option<Arc<dyn Fn() + Send + Sync>>,
}
pub(crate) struct OrderState {
    live: Option<Arc<Mutex<LiveOrder>>>,
    window: Cell<Option<gpui::WindowId>>,
    surface: u64,
}
impl Default for OrderState {
    fn default() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Self {
            surface: NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            live: gpui::fast_immediate_enabled().then(|| {
                Arc::new(Mutex::new(LiveOrder {
                    next: 0,
                    tickets: HashMap::new(),
                    wake: None,
                }))
            }),
            window: Cell::new(None),
        }
    }
}
impl OrderState {
    pub(crate) fn drained(&self, window: gpui::WindowId) -> bool {
        gpui::fast_order_drained(window, self.surface)
    }
    pub(crate) fn surface(&self) -> u64 {
        self.surface
    }
    pub(crate) fn set_waker(&self, wake: Box<dyn Fn() + Send + Sync>) {
        if let Some(s) = &self.live {
            s.lock().unwrap().wake = Some(Arc::from(wake));
        }
    }
    pub(crate) fn ready(&self, window: gpui::WindowId, source: u8) -> bool {
        self.window.set(Some(window));
        gpui::fast_order_ready(window, self.surface, source)
    }
    pub(crate) fn current_ready(&self, source: u8) -> bool {
        gpui::fast_order_frame().map_or(self.live.is_none(), |(w, _)| self.ready(w, source))
    }
    pub(crate) fn track(
        &self,
        d: &ProtocolObject<dyn objc2_quartz_core::CAMetalDrawable>,
        source: u8,
    ) -> bool {
        let Some(live) = &self.live else {
            return true;
        };
        let key = d as *const _ as *const () as usize;
        let Some(window) = gpui::fast_order_frame().map(|f| f.0).or(self.window.get()) else {
            return false;
        };
        let Some(ticket) = gpui::fast_order_acquire(window, self.surface, source) else {
            return false;
        };
        let id = {
            let mut s = live.lock().unwrap();
            s.next += 1;
            let id = s.next;
            s.tickets.insert(id, (key, ticket));
            id
        };
        let live = live.clone();
        let block = RcBlock::new(
            move |_: NonNull<ProtocolObject<dyn objc2_metal::MTLDrawable>>| {
                let wake = {
                    let mut s = live.lock().unwrap();
                    if s.tickets.get(&id).is_some_and(|v| v.0 == key) {
                        s.tickets.remove(&id);
                    }
                    s.wake.clone()
                };
                if let Some(wake) = wake {
                    wake();
                }
            },
        );
        unsafe {
            d.addPresentedHandler(RcBlock::as_ptr(&block));
        }
        true
    }
    pub(crate) fn unused(&self, d: &ProtocolObject<dyn objc2_quartz_core::CAMetalDrawable>) {
        if let Some(live) = &self.live {
            let key = d as *const _ as *const () as usize;
            let mut s = live.lock().unwrap();
            if let Some(id) = s
                .tickets
                .iter()
                .filter_map(|(id, (k, _))| (*k == key).then_some(*id))
                .max()
            {
                s.tickets.remove(&id);
            }
        }
    }
    pub(crate) fn hold(
        &self,
        d: &ProtocolObject<dyn objc2_quartz_core::CAMetalDrawable>,
        source: u8,
    ) -> Option<OrderHold> {
        if source == 0 && !self.track(d, source) {
            return None;
        }
        let key = d as *const _ as *const () as usize;
        let id = self.live.as_ref().and_then(|live| {
            live.lock()
                .unwrap()
                .tickets
                .iter()
                .filter_map(|(id, (k, _))| (*k == key).then_some(*id))
                .max()
        });
        if self.live.is_some() && id.is_none() {
            return None;
        }
        Some(OrderHold {
            live: self.live.clone(),
            id: id.unwrap_or(0),
            submitted: false,
        })
    }
}
pub(crate) struct OrderHold {
    live: Option<Arc<Mutex<LiveOrder>>>,
    id: u64,
    submitted: bool,
}
impl OrderHold {
    pub(crate) fn submit(&mut self) -> bool {
        let Some(live) = &self.live else {
            return true;
        };
        let Some((window, version)) = gpui::fast_order_frame() else {
            return false;
        };
        let s = live.lock().unwrap();
        let okay = s
            .tickets
            .get(&self.id)
            .is_some_and(|(_, ticket)| ticket.submit(version));
        if !okay {
            gpui::fast_latency_config(window, "g5_stale_submission", [version as f64, 0., 0., 0.]);
        }
        okay
    }
    pub(crate) fn submitted(&mut self) {
        self.submitted = true;
    }
}
impl Drop for OrderHold {
    fn drop(&mut self) {
        if !self.submitted {
            if let Some(live) = &self.live {
                live.lock().unwrap().tickets.remove(&self.id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_callback_capacity_is_guarded_without_input_or_pacing() {
        // macOS 13's ordinary CV callback has neither an immediate scope nor
        // an experimental pacing flag. The master switch still guards it.
        assert!(!acquisition_allowed(true, false, 3, 3));
        assert!(acquisition_allowed(true, false, 2, 3));
        assert!(!acquisition_allowed(true, true, 0, 3));
        assert!(!acquisition_allowed(true, false, 0, 0));
        assert!(acquisition_allowed(false, false, 3, 3));
    }
}
