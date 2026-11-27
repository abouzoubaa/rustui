// SPDX-License-Identifier: Apache-2.0
//! Device-space offsets from the scene that was painted, including programmatic scrolling.
use crate::{Bounds, Pixels, Point, ScaledPixels, Scene, Window};
use std::{cell::RefCell, rc::Rc, sync::OnceLock};

/// Painted scroll container in device pixels, in surface coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FastScrollViewport {
    /// Stable element path of the scroll container.
    pub id: u64,
    /// Intersection of the container and inherited content mask.
    pub clip: Bounds<ScaledPixels>,
    /// Clamped content offset at paint time.
    pub offset: Point<ScaledPixels>,
}

#[cfg(any(test, feature = "test-support"))]
thread_local! {static TEST_SHIFT:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
/// Whether to use retained Metal scroll shifts for this process.
pub fn fast_scroll_shift_enabled() -> bool {
    #[cfg(any(test, feature = "test-support"))]
    if TEST_SHIFT.with(std::cell::Cell::get) {
        return true;
    }
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("RUSTUI_DAMAGE_SCROLL_SHIFT").as_deref() == Ok("1")
            && matches!(
                std::env::var("RUSTUI_DAMAGE").as_deref(),
                Ok("1" | "verify")
            )
    })
}
impl Window {
    pub(crate) fn with_damage_scroll<R>(
        &mut self,
        offset: Option<&Rc<RefCell<Point<Pixels>>>>,
        bounds: Bounds<Pixels>,
        paint: impl FnOnce(&mut Window) -> R,
    ) -> R {
        let Some(offset) = offset.filter(|_| fast_scroll_shift_enabled()) else {
            return paint(self);
        };
        let scale = self.scale_factor();
        let viewport = FastScrollViewport {
            id: self.element_id_stack.hash(),
            clip: bounds.intersect(&self.content_mask().bounds).scale(scale),
            offset: offset.borrow().scale(scale),
        };
        self.next_frame.scene.record_scroll_viewport(viewport);
        super::replay::with_scroll_owner(viewport.id, || paint(self))
    }
}
impl Scene {
    #[doc(hidden)]
    pub fn record_scroll_viewport(&mut self, viewport: FastScrollViewport) {
        let v = &mut self.fast_damage.scrolls;
        if let Some(slot) = v.iter_mut().find(|s| s.id == viewport.id) {
            *slot = viewport;
        } else {
            v.push(viewport);
        }
    }
    #[doc(hidden)]
    pub fn fast_scroll_viewports(&self) -> &[FastScrollViewport] {
        &self.fast_damage.scrolls
    }
}

#[cfg(any(test, feature = "test-support"))]
impl Scene {
    #[doc(hidden)]
    pub fn with_test_scroll_shift<R>(run: impl FnOnce() -> R) -> R {
        struct Restore(bool);
        impl Drop for Restore {
            fn drop(&mut self) {
                TEST_SHIFT.set(self.0);
            }
        }
        let _guard = Restore(TEST_SHIFT.replace(true));
        run()
    }
}
