// SPDX-License-Identifier: Apache-2.0
//! RustUI scene-instance upload counts and synchronous mode sequences.
//! Fullscreen copy/clear triangles and tessellation vertices are excluded.
use gpui::Scene;
use std::{
    cell::{Cell, RefCell},
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Counts {
    encoded: usize,
    total: usize,
}
thread_local! {
    static COUNTS: RefCell<Option<Counts>> = const { RefCell::new(None) };
    static SEQUENCE: Cell<u64> = const { Cell::new(0) };
}
fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("RUSTUI_FRAME_LOG").as_deref() == Ok("1"))
}
fn regular(scene: &Scene) -> usize {
    scene.quads.len()
        + scene.shadows.len()
        + scene.underlines.len()
        + scene.monochrome_sprites.len()
        + scene.polychrome_sprites.len()
        + scene.surfaces.len()
}
fn total(scene: &Scene) -> usize {
    regular(scene)
        + scene.subpixel_sprites.len()
        + scene.paths.len()
        + scene
            .layers
            .frames
            .iter()
            .filter_map(|f| f.content.scene())
            .map(total)
            .sum::<usize>()
}
fn emit(counts: Counts, mode: &str, reason: &str) {
    if !enabled() {
        return;
    }
    let sequence = SEQUENCE.with(|n| {
        let next = n.get() + 1;
        n.set(next);
        next
    });
    let epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * 1000.;
    super::log::line(format_args!(
        "RUSTUI_FRAME_LOG {{\"event\":\"encode\",\"cpu_ms\":0.0,\"sequence\":{sequence},\"mode\":\"{mode}\",\"reason\":\"{reason}\",\"instances_encoded\":{},\"instances_total\":{},\"epoch_ms\":{epoch_ms:.6}}}",
        counts.encoded, counts.total
    ));
}
pub(crate) struct Scope {
    previous: Option<Counts>,
    active: bool,
}
impl Scope {
    pub(crate) fn new(scene: &Scene) -> Self {
        Self::start(if enabled() { total(scene) } else { 0 }, enabled())
    }
    fn start(total: usize, active: bool) -> Self {
        let previous = if active {
            COUNTS.with(|c| c.replace(Some(Counts { encoded: 0, total })))
        } else {
            None
        };
        Self { previous, active }
    }
    fn take(&mut self) -> Counts {
        if !self.active {
            return Counts::default();
        }
        self.active = false;
        COUNTS.with(|c| c.replace(self.previous.take()).unwrap())
    }
    pub(crate) fn finish(mut self, mode: &str, reason: &str) {
        let counts = self.take();
        emit(counts, mode, reason);
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        let _ = self.take();
    }
}
pub(crate) fn uploaded(count: usize) {
    COUNTS.with(|c| {
        if let Some(c) = c.borrow_mut().as_mut() {
            c.encoded += count;
        }
    });
}
pub(crate) fn uploaded_regular(scene: &Scene) {
    uploaded(regular(scene));
}
pub(crate) fn skipped(scene: &Scene) {
    if enabled() {
        emit(
            Counts {
                encoded: 0,
                total: total(scene),
            },
            "skip",
            "skip",
        );
    }
}
#[cfg(any(test, feature = "test-support"))]
impl crate::metal_renderer::MetalRenderer {
    /// Validates nested counting scopes and error-path restoration without GPU work.
    pub fn test_damage_encoding_metrics() {
        let mut outer = Scope::start(10, true);
        uploaded(3);
        let mut inner = Scope::start(5, true);
        uploaded(2);
        assert_eq!(
            inner.take(),
            Counts {
                encoded: 2,
                total: 5
            }
        );
        uploaded(4);
        {
            let _aborted = Scope::start(90, true);
            uploaded(80);
        }
        assert_eq!(
            outer.take(),
            Counts {
                encoded: 7,
                total: 10
            }
        );
        assert!(COUNTS.with(|c| c.borrow().is_none()));
    }
}
#[cfg(test)]
#[test]
fn nested_encoding_counters_restore_the_outer_frame() {
    crate::metal_renderer::MetalRenderer::test_damage_encoding_metrics();
}
