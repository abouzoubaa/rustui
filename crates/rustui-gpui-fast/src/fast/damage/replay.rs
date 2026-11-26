// SPDX-License-Identifier: Apache-2.0
// RustUI Stage G: original provenance hooks for GPUI Fast 0.1.14 Scene replay.
// These records carry identity, never a second rendering or retention system.
use crate::{Primitive, Scene, Window};
use std::{
    cell::Cell,
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

#[derive(Clone, Copy, Default)]
struct Source {
    id: u64,
    replayed: bool,
    element: u64,
    view: u64,
}
#[derive(Clone, Copy)]
struct Entry {
    source: Source,
    order: u32,
    texture: u32,
    tile: u32,
    paint_index: usize,
}
#[derive(Clone, Default)]
pub(crate) struct Provenance {
    operations: Vec<Source>,
    kinds: [Vec<Entry>; 8],
}
thread_local! {
    static SOURCE:Cell<Source> = const { Cell::new(Source{id:0,replayed:false,element:0,view:0}) };
    #[cfg(any(test, feature="test-support"))]
    static TEST_ENABLED:Cell<bool> = const { Cell::new(false) };
}
fn enabled() -> bool {
    #[cfg(any(test, feature = "test-support"))]
    if TEST_ENABLED.with(Cell::get) {
        return true;
    }
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        matches!(
            std::env::var("RUSTUI_DAMAGE").as_deref(),
            Ok("1" | "verify")
        )
    })
}
impl Provenance {
    pub(crate) fn clear(&mut self) {
        self.operations.clear();
        for kind in &mut self.kinds {
            kind.clear();
        }
    }
    pub(crate) fn finish(&mut self) {
        for kind in &mut self.kinds {
            let key = |e: &Entry| (e.order, e.texture, e.tile);
            if !kind.is_sorted_by_key(key) {
                kind.sort_by_key(key);
            }
        }
    }
    pub(crate) fn restore_operations(&mut self, original: Self) {
        self.operations = original.operations;
    }
}
pub(crate) fn record(scene: &mut Scene, primitive: &Primitive) {
    if !enabled() {
        return;
    }
    let (kind, order, texture, tile) = match primitive {
        Primitive::Quad(p) => (0, p.order, 0, 0),
        Primitive::Shadow(p) => (1, p.order, 0, 0),
        Primitive::Underline(p) => (2, p.order, 0, 0),
        Primitive::MonochromeSprite(p) => (3, p.order, p.tile.texture_id.index, p.tile.tile_id.0),
        Primitive::SubpixelSprite(p) => (4, p.order, p.tile.texture_id.index, p.tile.tile_id.0),
        Primitive::PolychromeSprite(p) => (5, p.order, p.tile.texture_id.index, p.tile.tile_id.0),
        Primitive::Path(p) => (6, p.order, 0, 0),
        Primitive::Surface(p) => (7, p.order, 0, 0),
    };
    let mut source = SOURCE.with(Cell::get);
    if source.id == 0 {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        source.id = NEXT.fetch_add(1, Ordering::Relaxed);
    }
    scene
        .fast_damage
        .operations
        .resize(scene.paint_operations.len() + 1, Source::default());
    scene.fast_damage.operations[scene.paint_operations.len()] = source;
    scene.fast_damage.kinds[kind].push(Entry {
        source,
        order,
        texture,
        tile,
        paint_index: scene.paint_operations.len(),
    });
}
struct Restore(Source);
impl Drop for Restore {
    fn drop(&mut self) {
        SOURCE.with(|s| s.set(self.0));
    }
}
/// Propagate the existing paint owner's generational SlotMap key. Nested
/// views override it; replay and rounded-clip compilation retain the original.
pub(crate) fn with_view<R>(view: crate::EntityId, paint: impl FnOnce() -> R) -> R {
    static ACTIVE: OnceLock<bool> = OnceLock::new();
    if !*ACTIVE.get_or_init(|| {
        ["RUSTUI_G_R2_VIEW_SIGNATURES", "RUSTUI_G_R2_VIEW_SIZES"]
            .iter()
            .any(|key| std::env::var_os(key).as_deref() == Some("1".as_ref()))
    }) {
        return paint();
    }
    let _restore = Restore(SOURCE.with(|s| {
        s.replace(Source {
            view: if slotmap::Key::is_null(&view) {
                0
            } else {
                view.as_u64()
            },
            ..s.get()
        })
    }));
    paint()
}
fn insert(scene: &mut Scene, primitive: Primitive, source: Source) {
    let _restore = Restore(SOURCE.with(|s| s.replace(source)));
    scene.insert_primitive(primitive);
}
pub(crate) fn replay(scene: &mut Scene, primitive: Primitive, previous: &Scene, index: usize) {
    if !enabled() {
        scene.insert_primitive(primitive);
        return;
    }
    let mut source = previous
        .fast_damage
        .operations
        .get(index)
        .copied()
        .unwrap_or_default();
    source.replayed = source.id != 0;
    insert(scene, primitive, source);
}
pub(crate) fn compile(
    scene: &mut Scene,
    primitive: Primitive,
    previous: &Provenance,
    index: usize,
) {
    if !enabled() {
        scene.insert_primitive(primitive);
        return;
    }
    insert(
        scene,
        primitive,
        previous.operations.get(index).copied().unwrap_or_default(),
    );
}
impl Window {
    pub(crate) fn fast_damage_insert_primitive(&mut self, primitive: impl Into<Primitive>) {
        if !Scene::fast_damage_order_probe_enabled() {
            self.next_frame.scene.insert_primitive(primitive);
            return;
        }
        let source = Source {
            element: self.element_id_stack.hash(),
            ..SOURCE.with(Cell::get)
        };
        insert(&mut self.next_frame.scene, primitive.into(), source);
    }
}
impl Scene {
    /// Internal renderer bridge for bounded paint-order diagnostics.
    #[doc(hidden)]
    pub fn fast_damage_order_probe_enabled() -> bool {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        *ENABLED.get_or_init(|| {
            std::env::var_os("RUSTUI_G_R2_ORDER_TRACE").as_deref() == Some("1".as_ref())
        })
    }

    /// The element path hash and original scene paint-operation index.
    #[doc(hidden)]
    pub fn fast_damage_order_identity(&self, kind: usize, index: usize) -> Option<(u64, usize)> {
        let entry = self.fast_damage.kinds.get(kind)?.get(index)?;
        (entry.source.element != 0).then_some((entry.source.element, entry.paint_index))
    }

    /// Internal renderer bridge: stable identity only for exact GPUI paint replay.
    #[doc(hidden)]
    pub fn fast_damage_source(&self, kind: usize, index: usize) -> Option<(u64, bool)> {
        self.fast_damage
            .kinds
            .get(kind)?
            .get(index)
            .map(|e| (e.source.id, e.source.replayed))
    }
    /// Full SlotMap key of the GPUI view that originally painted this operation.
    #[doc(hidden)]
    pub fn fast_damage_view(&self, kind: usize, index: usize) -> u64 {
        self.fast_damage
            .kinds
            .get(kind)
            .and_then(|k| k.get(index))
            .map_or(0, |e| e.source.view)
    }
    /// Identity and owner of one exact GPUI paint operation, in renderer order.
    #[doc(hidden)]
    pub fn fast_damage_view_source(&self, kind: usize, index: usize) -> Option<(u64, bool, u64)> {
        let source = self.fast_damage.kinds.get(kind)?.get(index)?.source;
        Some((source.id, source.replayed, source.view))
    }
    /// Defer signature work for a whole frame until a later frame can be planned.
    /// Layer contents are immutable shared scenes; primitive bytes are copied.
    #[doc(hidden)]
    pub fn fast_damage_snapshot(&self) -> Scene {
        let mut scene = Scene::default();
        scene.shadows = self.shadows.clone();
        scene.quads = self.quads.clone();
        scene.paths = self.paths.clone();
        scene.underlines = self.underlines.clone();
        scene.monochrome_sprites = self.monochrome_sprites.clone();
        scene.subpixel_sprites = self.subpixel_sprites.clone();
        scene.polychrome_sprites = self.polychrome_sprites.clone();
        scene.surfaces = self.surfaces.clone();
        scene.layers.frames = self.layers.frames.clone();
        scene.fast_damage = self.fast_damage.clone();
        scene
    }
    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    pub fn with_fast_damage_provenance<R>(run: impl FnOnce() -> R) -> R {
        struct Reset(bool);
        impl Drop for Reset {
            fn drop(&mut self) {
                TEST_ENABLED.with(|s| s.set(self.0));
            }
        }
        let _reset = Reset(TEST_ENABLED.with(|s| s.replace(true)));
        run()
    }
}

#[cfg(any(test, feature = "test-support"))]
impl Scene {
    #[doc(hidden)]
    pub fn with_fast_damage_clip_for_test(
        &mut self,
        bounds: crate::Bounds<crate::ScaledPixels>,
        paint: impl FnOnce(&mut Self),
    ) {
        use crate::fast::rounded_clip::{ClipOperation, Shape, record};
        record(
            self,
            ClipOperation::Begin(Shape {
                bounds,
                radii: crate::Corners {
                    top_left: crate::size(crate::ScaledPixels(4.), crate::ScaledPixels(4.)),
                    top_right: crate::size(crate::ScaledPixels(4.), crate::ScaledPixels(4.)),
                    bottom_right: crate::size(crate::ScaledPixels(4.), crate::ScaledPixels(4.)),
                    bottom_left: crate::size(crate::ScaledPixels(4.), crate::ScaledPixels(4.)),
                },
            }),
        );
        paint(self);
        record(self, ClipOperation::End);
    }
}
