// RustUI G5-2: at most one command buffer awaiting system scheduling.
use block2::RcBlock;
use objc2::{Encode, Encoding, msg_send, runtime::ProtocolObject};
use objc2_metal::{MTLCommandBuffer, MTLDrawable};
use objc2_quartz_core::CAMetalDisplayLink;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::{cell::Cell, ptr::NonNull};

#[derive(Default)]
struct Gate {
    next: AtomicU64,
    pending: AtomicU64,
}
impl Gate {
    fn ready(&self) -> bool {
        self.pending.load(Ordering::Acquire) == 0
    }
    fn begin(&self) -> Option<u64> {
        let id = self.next.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
        self.pending
            .compare_exchange(0, id, Ordering::AcqRel, Ordering::Acquire)
            .ok()
            .map(|_| id)
    }
    fn finish(&self, id: u64) -> bool {
        self.pending
            .compare_exchange(id, 0, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
}

pub(crate) struct State(Option<Arc<Gate>>);
impl Default for State {
    fn default() -> Self {
        Self(gpui::fast_frame_ahead_enabled().then(|| Arc::new(Gate::default())))
    }
}
impl State {
    pub(crate) fn ready(&self) -> bool {
        self.0.as_ref().is_none_or(|g| g.ready())
    }
    pub(crate) fn observe(
        &self,
        buffer: &ProtocolObject<dyn MTLCommandBuffer>,
        wake: Box<dyn Fn() + Send + Sync>,
    ) {
        let Some(g) = &self.0 else { return };
        let Some(id) = g.begin() else { return };
        let wake = Arc::new(wake);
        let window = gpui::fast_latency_frame().map(|f| f.window());
        let (gate, notify) = (g.clone(), wake.clone());
        let scheduled = RcBlock::new(move |_: NonNull<ProtocolObject<dyn MTLCommandBuffer>>| {
            if gate.finish(id) {
                if let Some(w) = window {
                    gpui::fast_latency_config(w, "g5_handoff", [id as f64, 1., 0., 0.]);
                }
                notify();
            }
        });
        let gate = g.clone();
        let completed = RcBlock::new(move |_: NonNull<ProtocolObject<dyn MTLCommandBuffer>>| {
            if gate.finish(id) {
                if let Some(w) = window {
                    gpui::fast_latency_config(w, "g5_handoff", [id as f64, 2., 0., 0.]);
                }
                wake();
            }
        });
        unsafe {
            buffer.addScheduledHandler(RcBlock::as_ptr(&scheduled));
            buffer.addCompletedHandler(RcBlock::as_ptr(&completed));
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
struct RateRange {
    minimum: f32,
    maximum: f32,
    preferred: f32,
}
unsafe impl Encode for RateRange {
    const ENCODING: Encoding = Encoding::Struct(
        "CAFrameRateRange",
        &[Encoding::Float, Encoding::Float, Encoding::Float],
    );
}
pub(crate) fn configure(
    link: &CAMetalDisplayLink,
    last: &Cell<Option<bool>>,
    window: gpui::WindowId,
) {
    if !gpui::fast_frame_ahead_enabled() {
        return;
    }
    let active = gpui::fast_frame_input_active(window);
    if last.replace(Some(active)) == Some(active) {
        return;
    }
    link.setPreferredFrameLatency(1.);
    let (minimum, maximum, preferred) = if active {
        (80., 120., 120.)
    } else {
        (0., 0., 0.)
    };
    unsafe {
        let _: () = msg_send![link,setPreferredFrameRateRange:RateRange{minimum,maximum,preferred}];
    }
    crate::fast::latency::link(window, link);
    gpui::fast_latency_config(
        window,
        "g5_metal_rate",
        [minimum as f64, maximum as f64, preferred as f64, 1.],
    );
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_one_frame_and_stale_completion_cannot_release_successor() {
        let g = Gate::default();
        assert!(g.ready());
        let a = g.begin().unwrap();
        assert!(!g.ready());
        assert!(g.begin().is_none());
        assert!(g.finish(a));
        let b = g.begin().unwrap();
        assert!(!g.finish(a));
        assert!(!g.ready());
        assert!(g.finish(b));
        assert!(g.ready());
    }
    #[test]
    fn completion_can_retire_a_buffer_that_never_scheduled() {
        let g = Gate::default();
        let a = g.begin().unwrap();
        assert!(g.finish(a));
        assert!(g.ready());
        assert!(!g.finish(a));
    }
}
