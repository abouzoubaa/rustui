// RustUI G5-1: restore detached frame sources for asynchronous invalidation.
use crate::display_link::MetalFrames;
use gpui::{ForegroundExecutor, WindowId};
use std::{
    cell::Cell,
    rc::{Rc, Weak},
};

#[derive(Default)]
struct RestoreWake(Cell<bool>);
impl RestoreWake {
    fn queue(&self) -> bool {
        !self.0.replace(true)
    }
    fn finished(&self) {
        self.0.set(false);
    }
}

pub(crate) fn frame_waker(
    window: WindowId,
    frames: Weak<MetalFrames>,
    executor: ForegroundExecutor,
    recreate: impl Fn() + 'static,
) -> Rc<dyn Fn()> {
    if !gpui::fast_immediate_enabled() {
        return Rc::new(move || {
            if let Some(frames) = frames.upgrade() {
                frames.wake();
            }
        });
    }
    let pending = Rc::new(RestoreWake::default());
    let recreate = Rc::new(recreate);
    Rc::new(move || {
        if gpui::fast_immediate_demand(window) {
            return;
        }
        let Some(current) = frames.upgrade() else {
            return;
        };
        if current.is_attached() {
            current.wake();
            return;
        }
        if !pending.queue() {
            return;
        }
        let (frames, pending, recreate) = (frames.clone(), pending.clone(), recreate.clone());
        // Invalidation may run with MacWindowState already locked. Recreate on
        // the existing foreground executor, coalescing repeated invalidations.
        executor
            .spawn(async move {
                if let Some(frames) = frames.upgrade() {
                    if !frames.is_attached() {
                        recreate();
                    }
                    frames.wake();
                }
                pending.finished();
            })
            .detach();
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detached_wakes_coalesce_and_can_be_queued_again() {
        let pending = RestoreWake::default();
        assert!(pending.queue());
        for _ in 0..100 {
            assert!(!pending.queue());
        }
        pending.finished();
        assert!(pending.queue());
    }
}
