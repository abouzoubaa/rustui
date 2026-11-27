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
    scroll_owner: u64,
    id: u64,
    replayed: bool,
    region: u32,
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
    scroll_changed: bool,
    pub(super) scrolls: Vec<super::scroll::FastScrollViewport>,
    pub(super) extra_damage: Vec<crate::Bounds<crate::ScaledPixels>>,
    pub(super) regions: Vec<super::declared::DamageRegion>,
    // Rounded-clip compilation retains original operations for future replay,
    // while its renderer-order primitives belong to the compiled scene.
    operation_regions: Option<Vec<super::declared::DamageRegion>>,
    operations: Vec<Source>,
    kinds: [Vec<Entry>; 8],
}
thread_local! {
    static SOURCE:Cell<Source> = const { Cell::new(Source{scroll_owner:0,id:0,region:0,replayed:false,element:0,view:0}) };
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
    pub(super) fn declaration_start(&self) -> [usize; 8] {
        std::array::from_fn(|i| self.kinds[i].len())
    }
    fn intern_region(&mut self, region: super::declared::DamageRegion) -> u32 {
        if self.regions.last() != Some(&region) {
            self.regions.push(region);
        }
        u32::try_from(self.regions.len()).expect("too many declared damage regions")
    }
    pub(super) fn declare(&mut self, start: [usize; 8], region: super::declared::DamageRegion) {
        let id = self.intern_region(region);
        for (kind, start) in self.kinds.iter_mut().zip(start) {
            for entry in &mut kind[start..] {
                entry.source.region = id;
                self.operations[entry.paint_index].region = id;
            }
        }
    }
    pub(super) fn region(
        &self,
        kind: usize,
        index: usize,
    ) -> Option<super::declared::DamageRegion> {
        let id = self
            .kinds
            .get(kind)?
            .get(index)?
            .source
            .region
            .checked_sub(1)?;
        self.regions.get(id as usize).copied()
    }
    fn import(&mut self, original: &Self, mut source: Source) -> Source {
        if source.scroll_owner != 0
            && !self.scrolls.iter().any(|v| v.id == source.scroll_owner)
            && let Some(v) = original
                .scrolls
                .iter()
                .find(|v| v.id == source.scroll_owner)
        {
            self.scrolls.push(*v);
        }
        if let Some(i) = source.region.checked_sub(1) {
            let regions = original
                .operation_regions
                .as_ref()
                .unwrap_or(&original.regions);
            source.region = self.intern_region(regions[i as usize]);
        }
        source
    }
    pub(crate) fn clear(&mut self) {
        self.extra_damage.clear();
        self.regions.clear();
        self.operation_regions = None;
        self.scrolls.clear();
        self.scroll_changed = false;
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
        self.extra_damage = original.extra_damage;
        self.scrolls = original.scrolls;
        self.scroll_changed = original.scroll_changed;
        self.operation_regions = original.operation_regions.or(Some(original.regions));
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
    source = scene.fast_damage.import(&previous.fast_damage, source);
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
    let source = scene.fast_damage.import(
        previous,
        previous.operations.get(index).copied().unwrap_or_default(),
    );
    insert(scene, primitive, source);
}
impl Window {
    pub(crate) fn fast_damage_observe_scroll(
        &mut self,
        previous: &mut Option<crate::Point<crate::Pixels>>,
        current: crate::Point<crate::Pixels>,
    ) {
        static ACTIVE: OnceLock<bool> = OnceLock::new();
        if enabled()
            && *ACTIVE.get_or_init(|| {
                std::env::var_os("RUSTUI_G_R2_EARLY_REJECT").as_deref() == Some("1".as_ref())
            })
        {
            self.next_frame
                .scene
                .observe_damage_scroll(previous, current);
        }
    }

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
    /// Observe the final clamped offset, including a programmatic scroll-to-item.
    #[doc(hidden)]
    pub fn observe_damage_scroll(
        &mut self,
        previous: &mut Option<crate::Point<crate::Pixels>>,
        current: crate::Point<crate::Pixels>,
    ) {
        self.fast_damage.scroll_changed |=
            previous.replace(current).is_some_and(|old| old != current);
    }
    #[doc(hidden)]
    pub fn fast_damage_scroll_changed(&self) -> bool {
        self.fast_damage.scroll_changed
    }

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

pub(super) fn current_scroll_owner() -> u64 {
    SOURCE.with(|s| s.get().scroll_owner)
}
pub(super) fn with_scroll_owner<R>(id: u64, paint: impl FnOnce() -> R) -> R {
    let _restore = Restore(SOURCE.with(|s| {
        s.replace(Source {
            scroll_owner: id,
            ..s.get()
        })
    }));
    paint()
}
impl Scene {
    #[doc(hidden)]
    pub fn fast_scroll_owner(&self, kind: usize, index: usize) -> u64 {
        self.fast_damage.kinds[kind]
            .get(index)
            .map_or(0, |e| e.source.scroll_owner)
    }
    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    pub fn with_test_scroll_owner<R>(id: u64, paint: impl FnOnce() -> R) -> R {
        with_scroll_owner(id, paint)
    }
}
