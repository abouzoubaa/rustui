// SPDX-License-Identifier: Apache-2.0
// Ported from longbridge/gpui-fast, commit 2002081b4a61249a1cef2b8a552f6ba0998d9f52,
// crates/gpui_windows/src/fast/partial.rs.
// Copyright 2022 - 2025 Zed Industries, Inc.; Copyright 2026 Longbridge.
// RustUI modifications: remove Windows drawing/adapter APIs, expose Metal inputs,
// and run the preserved planner tests through test-support as well as crate tests.
//! Platform-independent scene signatures, damage rectangles, and scroll planning.
use super::{
    diagnosis::Trace,
    order_probe,
    timing::{PlanStage, plan_stage},
};
use collections::HashMap;
use gpui::{
    AtlasTextureId, AtlasTile, Background, Bounds, ContentMask, Corners, Edges, Hsla, IsZero,
    ScaledPixels, Scene, TransformationMatrix,
};
use std::collections::HashSet;
mod cache;
mod fixed;
mod hover_order;
pub(crate) use cache::Cache;
use std::hash::{Hash, Hasher};

/// More damage than this share of the window: the frame is drawn whole.
pub(crate) const MAX_DAMAGE: f32 = 0.6;
/// More damage rectangles than this are merged into their box.
const MAX_RECTS: usize = 8;
/// More rectangles than this are first gathered into bands of the window.
const MAX_MERGED: usize = 64;
/// A scroll needs this many primitives moved alike.
const MIN_MOVED: usize = 4;
/// How far two coordinates computed apart may differ and be the same.
const SAME: f32 = 1.0 / 1024.0;

// --- Tiles uploaded since the last frame -------------------------------------

/// The atlas tiles uploaded since the last frame was drawn: their sprites
/// show something new even when their primitives are byte for byte the
/// last frame's (the atlas gives a freed tile's place and id out again).
#[derive(Default)]
pub(crate) struct Uploads {
    tiles: HashSet<(AtlasTextureId, u32)>,
    /// More uploads than are worth keeping (nothing takes them: the window
    /// is drawn another way): the next frame is drawn whole.
    lost: bool,
}

impl Uploads {
    /// The most tiles kept between two frames.
    const MOST: usize = 4096;

    pub(crate) fn record(&mut self, tile: &AtlasTile) {
        if self.tiles.len() >= Self::MOST {
            self.tiles.clear();
            self.lost = true;
        }
        self.tiles.insert((tile.texture_id, tile.tile_id.0));
    }

    pub(crate) fn clear(&mut self) {
        self.tiles.clear();
        self.lost = true;
    }
}

impl Uploads {
    pub(crate) fn take(&mut self) -> Option<HashSet<(AtlasTextureId, u32)>> {
        let tiles = std::mem::take(&mut self.tiles);
        (!std::mem::take(&mut self.lost)).then_some(tiles)
    }
}

// --- Signatures --------------------------------------------------------------

/// A rectangle in device pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Area {
    pub(crate) x0: f32,
    pub(crate) y0: f32,
    pub(crate) x1: f32,
    pub(crate) y1: f32,
}

impl Area {
    pub(crate) fn of(b: &Bounds<ScaledPixels>) -> Area {
        Area {
            x0: b.origin.x.0,
            y0: b.origin.y.0,
            x1: b.origin.x.0 + b.size.width.0,
            y1: b.origin.y.0 + b.size.height.0,
        }
    }

    pub(crate) fn and(&self, o: &Area) -> Area {
        Area {
            x0: self.x0.max(o.x0),
            y0: self.y0.max(o.y0),
            x1: self.x1.min(o.x1),
            y1: self.y1.min(o.y1),
        }
    }

    pub(crate) fn union(&self, o: &Area) -> Area {
        Area {
            x0: self.x0.min(o.x0),
            y0: self.y0.min(o.y0),
            x1: self.x1.max(o.x1),
            y1: self.y1.max(o.y1),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        !(self.x1 > self.x0 && self.y1 > self.y0)
    }

    pub(crate) fn area(&self) -> f32 {
        if self.is_empty() {
            0.0
        } else {
            (self.x1 - self.x0) * (self.y1 - self.y0)
        }
    }

    fn moved(&self, dy: f32) -> Area {
        Area {
            y0: self.y0 + dy,
            y1: self.y1 + dy,
            ..*self
        }
    }

    fn grown(&self, by: f32) -> Area {
        Area {
            x0: self.x0 - by,
            y0: self.y0 - by,
            x1: self.x1 + by,
            y1: self.y1 + by,
        }
    }

    pub(super) fn touches(&self, o: &Area) -> bool {
        self.x0 <= o.x1 && o.x0 <= self.x1 && self.y0 <= o.y1 && o.y0 <= self.y1
    }

    fn within(&self, o: &Area) -> bool {
        self.is_empty()
            || (self.x0 >= o.x0 - SAME
                && self.y0 >= o.y0 - SAME
                && self.x1 <= o.x1 + SAME
                && self.y1 <= o.y1 + SAME)
    }

    fn same(&self, o: &Area) -> bool {
        (self.is_empty() && o.is_empty())
            || ((self.x0 - o.x0).abs() < SAME
                && (self.y0 - o.y0).abs() < SAME
                && (self.x1 - o.x1).abs() < SAME
                && (self.y1 - o.y1).abs() < SAME)
    }

    /// The pixels it touches.
    pub(crate) fn pixels(&self) -> Area {
        Area {
            x0: self.x0.floor(),
            y0: self.y0.floor(),
            x1: self.x1.ceil(),
            y1: self.y1.ceil(),
        }
    }

    fn on_grid(&self) -> bool {
        let on = |v: f32| (v - v.round()).abs() < SAME;
        on(self.x0) && on(self.y0) && on(self.x1) && on(self.y1)
    }

    fn bits(&self) -> [u32; 4] {
        [
            self.x0.to_bits(),
            self.y0.to_bits(),
            self.x1.to_bits(),
            self.y1.to_bits(),
        ]
    }
}

/// A primitive as the comparison sees it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Sig {
    /// What it looks like apart from where it is, in what order and through
    /// what mask.
    key: u64,
    order: u32,
    /// Its bounds, its mask, and what it may touch: its bounds (a shadow's
    /// tail, a pixel of antialiasing included) within its mask.
    bounds: Area,
    mask: Area,
    reach: Area,
    /// The same colour everywhere, through any mask.
    plain: bool,
    /// Shows something new whatever its bytes say.
    fresh: bool,
}

impl Sig {
    pub(super) fn order(self) -> u32 {
        self.order
    }
    pub(super) fn bounds(self) -> Area {
        self.bounds
    }

    pub(super) fn order_reach(&self) -> Area {
        self.reach.union(&self.bounds.grown(1.).and(&self.mask))
    }
}

struct Key(collections::FxHasher);

impl Key {
    fn new(kind: u8) -> Key {
        let mut hasher = collections::FxHasher::default();
        kind.hash(&mut hasher);
        Key(hasher)
    }

    fn f(mut self, v: f32) -> Key {
        v.to_bits().hash(&mut self.0);
        self
    }

    fn u(mut self, v: u64) -> Key {
        v.hash(&mut self.0);
        self
    }

    fn color(self, c: &Hsla) -> Key {
        self.f(c.h).f(c.s).f(c.l).f(c.a)
    }

    fn corners(self, c: &Corners<ScaledPixels>) -> Key {
        self.f(c.top_left.0)
            .f(c.top_right.0)
            .f(c.bottom_right.0)
            .f(c.bottom_left.0)
    }

    fn edges(self, e: &Edges<ScaledPixels>) -> Key {
        self.f(e.top.0).f(e.right.0).f(e.bottom.0).f(e.left.0)
    }

    fn size(self, b: &Bounds<ScaledPixels>) -> Key {
        self.f(b.size.width.0).f(b.size.height.0)
    }

    fn tile(self, t: &AtlasTile) -> Key {
        self.u(t.texture_id.index as u64)
            .u(t.texture_id.kind as u64)
            .u(t.tile_id.0 as u64)
            .u(t.bounds.origin.x.0 as u64)
            .u(t.bounds.origin.y.0 as u64)
            .u(t.bounds.size.width.0 as u64)
            .u(t.bounds.size.height.0 as u64)
    }

    /// A background: its colour, or for a gradient (rare) its description.
    fn background(mut self, background: &Background) -> Key {
        match background.as_solid() {
            Some(color) => self.u(0).color(&color),
            None => {
                format!("{background:?}").hash(&mut self.0);
                self
            }
        }
    }

    /// A sprite's transformation. A transformed sprite is drawn where its
    /// matrix takes its bounds, so it is keyed with where they are: moved,
    /// it is another sprite.
    fn transformation(self, t: &TransformationMatrix, bounds: &Bounds<ScaledPixels>) -> Key {
        let key = self
            .f(t.rotation_scale[0][0])
            .f(t.rotation_scale[0][1])
            .f(t.rotation_scale[1][0])
            .f(t.rotation_scale[1][1])
            .f(t.translation[0])
            .f(t.translation[1]);
        if is_unit(t) {
            key
        } else {
            key.f(bounds.origin.x.0).f(bounds.origin.y.0)
        }
    }

    fn done(self) -> u64 {
        self.0.finish()
    }
}

fn is_unit(t: &TransformationMatrix) -> bool {
    t.rotation_scale == [[1.0, 0.0], [0.0, 1.0]] && t.translation == [0.0, 0.0]
}

/// The box of where `t` takes `bounds`. The matrix is read both ways round
/// (rows or columns first), so that the box holds whichever the shader does.
fn transformed(bounds: &Bounds<ScaledPixels>, t: &TransformationMatrix) -> Area {
    let area = Area::of(bounds);
    if is_unit(t) {
        return area;
    }
    let m = t.rotation_scale;
    let mut out: Option<Area> = None;
    for (x, y) in [
        (area.x0, area.y0),
        (area.x1, area.y0),
        (area.x0, area.y1),
        (area.x1, area.y1),
    ] {
        for (tx, ty) in [
            (m[0][0] * x + m[0][1] * y, m[1][0] * x + m[1][1] * y),
            (m[0][0] * x + m[1][0] * y, m[0][1] * x + m[1][1] * y),
        ] {
            let (tx, ty) = (tx + t.translation[0], ty + t.translation[1]);
            let point = Area {
                x0: tx,
                y0: ty,
                x1: tx,
                y1: ty,
            };
            out = Some(out.map_or(point, |o| o.union(&point)));
        }
    }
    out.unwrap_or(area)
}

fn sig(
    key: u64,
    order: u32,
    bounds: &Bounds<ScaledPixels>,
    mask: &ContentMask<ScaledPixels>,
    grow: f32,
) -> Sig {
    let bounds = Area::of(bounds);
    let mask = Area::of(&mask.bounds);
    Sig {
        key,
        order,
        bounds,
        mask,
        reach: bounds.grown(grow).and(&mask),
        plain: false,
        fresh: false,
    }
}

/// The scene's primitives as signatures, or `None` for a scene that cannot
/// be compared (scroll layers, whose tiles change under unchanged sprites;
/// surfaces). `uploaded`: the tiles uploaded since the last frame.
pub(crate) fn signatures(
    scene: &Scene,
    uploaded: &HashSet<(AtlasTextureId, u32)>,
    dynamic: &dyn Fn(AtlasTextureId) -> bool,
) -> Option<Vec<Sig>> {
    signatures_cached(scene, uploaded, dynamic, &mut Cache::default())
}

pub(crate) fn can_plan(scene: &Scene) -> bool {
    if !scene.surfaces.is_empty() {
        return false;
    }
    scene.layers.frames.iter().all(|frame| {
        frame.tile_size == 0
            && (frame.key.0 == u32::MAX || frame.content.scene().is_some_and(can_plan))
    })
}

pub(crate) fn signatures_cached(
    scene: &Scene,
    uploaded: &HashSet<(AtlasTextureId, u32)>,
    dynamic: &dyn Fn(AtlasTextureId) -> bool,
    cache: &mut Cache,
) -> Option<Vec<Sig>> {
    // A scroll layer's tiles change under sprites that stay the same.
    if !can_plan(scene) {
        return None;
    }
    let fresh = |tile: &AtlasTile| {
        uploaded.contains(&(tile.texture_id, tile.tile_id.0)) || dynamic(tile.texture_id)
    };
    let mut out = Vec::with_capacity(
        scene.quads.len()
            + scene.shadows.len()
            + scene.underlines.len()
            + scene.monochrome_sprites.len()
            + scene.subpixel_sprites.len()
            + scene.polychrome_sprites.len()
            + scene.paths.len(),
    );
    for (index, q) in scene.quads.iter().enumerate() {
        let source = scene.fast_damage_source(0, index);
        if let Some(signature) = cache.replay(source, q.order, false) {
            out.push(signature);
            continue;
        }
        let solid = q.background.as_solid();
        let key = Key::new(1)
            .size(&q.bounds)
            .u(q.border_style as u64)
            .background(&q.background)
            .color(&q.border_color)
            .corners(&q.corner_radii)
            .edges(&q.border_widths)
            .done();
        let c = &q.corner_radii;
        let w = &q.border_widths;
        let mut s = sig(key, q.order, &q.bounds, &q.content_mask, 0.0);
        s.plain = solid.is_some()
            && c.top_left.is_zero()
            && c.top_right.is_zero()
            && c.bottom_left.is_zero()
            && c.bottom_right.is_zero()
            && w.top.is_zero()
            && w.right.is_zero()
            && w.bottom.is_zero()
            && w.left.is_zero();
        cache.push(source, s, &mut out);
    }
    for (index, s) in scene.shadows.iter().enumerate() {
        let source = scene.fast_damage_source(1, index);
        if let Some(signature) = cache.replay(source, s.order, false) {
            out.push(signature);
            continue;
        }
        let key = Key::new(2)
            .size(&s.bounds)
            .f(s.blur_radius.0)
            .corners(&s.corner_radii)
            .color(&s.color)
            .f(s.element_bounds.origin.x.0 - s.bounds.origin.x.0)
            .f(s.element_bounds.origin.y.0 - s.bounds.origin.y.0)
            .size(&s.element_bounds)
            .corners(&s.element_corner_radii)
            .u(s.inset as u64)
            .done();
        let grow = 3.0 * s.blur_radius.0.max(0.0) + 1.0;
        cache.push(
            source,
            sig(key, s.order, &s.bounds, &s.content_mask, grow),
            &mut out,
        );
    }
    for (index, u) in scene.underlines.iter().enumerate() {
        let source = scene.fast_damage_source(2, index);
        if let Some(signature) = cache.replay(source, u.order, false) {
            out.push(signature);
            continue;
        }
        let key = Key::new(3)
            .size(&u.bounds)
            .color(&u.color)
            .f(u.thickness.0)
            .u((u.wavy == true.into()) as u64)
            .done();
        cache.push(
            source,
            sig(key, u.order, &u.bounds, &u.content_mask, 1.0),
            &mut out,
        );
    }
    for (index, g) in scene.monochrome_sprites.iter().enumerate() {
        let source = scene.fast_damage_source(3, index);
        if let Some(signature) = cache.replay(source, g.order, fresh(&g.tile)) {
            out.push(signature);
            continue;
        }
        let t = &g.transformation;
        let key = Key::new(4)
            .size(&g.bounds)
            .tile(&g.tile)
            .color(&g.color)
            .transformation(t, &g.bounds)
            .done();
        let mut s = sig(key, g.order, &g.bounds, &g.content_mask, 1.0);
        s.reach = transformed(&g.bounds, t).grown(1.0).and(&s.mask);
        s.fresh = fresh(&g.tile);
        cache.push(source, s, &mut out);
    }
    for (index, g) in scene.subpixel_sprites.iter().enumerate() {
        let source = scene.fast_damage_source(4, index);
        if let Some(signature) = cache.replay(source, g.order, fresh(&g.tile)) {
            out.push(signature);
            continue;
        }
        let t = &g.transformation;
        let key = Key::new(5)
            .size(&g.bounds)
            .tile(&g.tile)
            .color(&g.color)
            .transformation(t, &g.bounds)
            .done();
        let mut s = sig(key, g.order, &g.bounds, &g.content_mask, 1.0);
        s.reach = transformed(&g.bounds, t).grown(1.0).and(&s.mask);
        s.fresh = fresh(&g.tile);
        cache.push(source, s, &mut out);
    }
    for (index, p) in scene.polychrome_sprites.iter().enumerate() {
        let source = scene.fast_damage_source(5, index);
        if crate::fast::rounded_clip::group(scene, p.tile.texture_id).is_none()
            && let Some(signature) = cache.replay(source, p.order, fresh(&p.tile))
        {
            out.push(signature);
            continue;
        }
        // A rounded clip: a sprite standing for a scene drawn through its
        // rounded bounds. It looks like that scene, seen from the sprite.
        if let Some(group) = crate::fast::rounded_clip::group(scene, p.tile.texture_id) {
            let child = group.content.scene()?;
            let inner = signatures_cached(child, uploaded, dynamic, cache)?;
            let (x, y) = (p.bounds.origin.x.0, p.bounds.origin.y.0);
            let mut key = Key::new(8)
                .size(&p.bounds)
                .corners(&p.corner_radii)
                .f(p.opacity);
            // The clip's vertical radii travel in a frame of the child's.
            for frame in &child.layers.frames {
                if frame.key.0 == u32::MAX {
                    let radii = frame.background;
                    key = key.f(radii.r).f(radii.g).f(radii.b).f(radii.a);
                }
            }
            let mut fresh_inside = false;
            for s in &inner {
                fresh_inside |= s.fresh;
                key = key.u(s.key).u(s.order as u64);
                for area in [&s.bounds, &s.mask] {
                    key = key
                        .f(area.x0 - x)
                        .f(area.y0 - y)
                        .f(area.x1 - x)
                        .f(area.y1 - y);
                }
            }
            let mut s = sig(key.done(), p.order, &p.bounds, &p.content_mask, 1.0);
            s.fresh = fresh_inside;
            cache.push(source, s, &mut out);
            continue;
        }
        let key = Key::new(6)
            .size(&p.bounds)
            .tile(&p.tile)
            .u((p.grayscale == true.into()) as u64)
            .f(p.opacity)
            .corners(&p.corner_radii)
            .done();
        let mut s = sig(key, p.order, &p.bounds, &p.content_mask, 1.0);
        s.fresh = fresh(&p.tile);
        cache.push(source, s, &mut out);
    }
    for (index, p) in scene.paths.iter().enumerate() {
        let source = scene.fast_damage_source(6, index);
        if let Some(signature) = cache.replay(source, p.order, false) {
            out.push(signature);
            continue;
        }
        // A path is its vertices where they are: one that moved is new.
        let mut key = Key::new(7).background(&p.color);
        for v in &p.vertices {
            key = key
                .f(v.xy_position.x.0)
                .f(v.xy_position.y.0)
                .f(v.st_position.x)
                .f(v.st_position.y);
        }
        cache.push(
            source,
            sig(key.done(), p.order, &p.bounds, &p.content_mask, 1.0),
            &mut out,
        );
    }
    Some(out)
}

// --- The plan ----------------------------------------------------------------

/// How a frame is made from the last one.
#[derive(Debug, PartialEq)]
pub(crate) struct Plan {
    /// The last frame's `.0` goes to `.1` (same size, whole pixels).
    pub(crate) shift: Option<(Area, Area)>,
    /// Where the scene is drawn again (whole pixels, within the window).
    pub(crate) damage: Vec<Area>,
}

fn placed(s: &Sig) -> (u64, [u32; 4], [u32; 4]) {
    (s.key, s.bounds.bits(), s.mask.bits())
}

/// A coordinate to 1/64 of a pixel: what finds candidates for a pair. Two
/// places computed apart (a scrolled place by layout, and the last one plus
/// the scroll) can differ in their last bits; candidates are then compared
/// within [`SAME`].
fn coarse(v: f32) -> i64 {
    (v * 64.0).round() as i64
}

/// A primitive apart from its vertical place.
fn shape(s: &Sig) -> (u64, i64, i64, i64) {
    (
        s.key,
        coarse(s.bounds.x0),
        coarse(s.bounds.x1 - s.bounds.x0),
        coarse(s.bounds.y1 - s.bounds.y0),
    )
}

/// Whether draw orders that rose in the last frame still rise: `pairs` are
/// the (last, new) orders of the primitives kept.
fn orders_agree(pairs: &mut [(u32, u32)]) -> bool {
    pairs.sort_unstable();
    let mut below = 0u32; // the highest new order among lower old orders
    let mut level = 0u32; // the highest new order so far
    let mut last_old = None;
    for &(old, new) in pairs.iter() {
        if last_old != Some(old) {
            below = level;
            last_old = Some(old);
        }
        if new < below {
            return false;
        }
        level = level.max(new);
    }
    true
}

fn tidy(rects: Vec<Area>, view: &Area) -> Vec<Area> {
    let mut rects: Vec<Area> = rects
        .into_iter()
        .map(|r| r.pixels().and(view))
        .filter(|r| !r.is_empty())
        .collect();
    if rects.len() > MAX_MERGED {
        // Many (a page changed): their boxes by band of the window, so that
        // what follows is quick. More damage than needed is still right.
        let band = ((view.y1 - view.y0) / 16.0).max(1.0);
        let mut bands: HashMap<i64, Area> = HashMap::default();
        for r in &rects {
            bands
                .entry((r.y0 / band).floor() as i64)
                .and_modify(|b| *b = b.union(r))
                .or_insert(*r);
        }
        rects = bands.into_values().collect();
    }
    loop {
        let mut merged = false;
        let mut i = 0;
        while i < rects.len() {
            let mut j = i + 1;
            while j < rects.len() {
                if rects[i].touches(&rects[j]) {
                    rects[i] = rects[i].union(&rects[j]);
                    rects.swap_remove(j);
                    merged = true;
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
        if !merged {
            break;
        }
    }
    if rects.len() > MAX_RECTS {
        let all = rects.iter().skip(1).fold(rects[0], |a, r| a.union(r));
        return vec![all];
    }
    rects
}

#[derive(Default)]
pub(super) struct Policy {
    early_metal: bool,
    pub allow_disjoint_order: bool,
    pub reason: Option<&'static str>,
    pub witness: Option<order_probe::Witness>,
}
impl Policy {
    pub fn metal() -> Self {
        static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        static ORDER_CULL: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        Self {
            early_metal: *ENABLED.get_or_init(|| {
                std::env::var_os("RUSTUI_G_R2_EARLY_REJECT").as_deref() == Some("1".as_ref())
            }),
            allow_disjoint_order: *ORDER_CULL.get_or_init(|| {
                std::env::var_os("RUSTUI_G_R2_DISJOINT_ORDER").as_deref() == Some("1".as_ref())
            }),
            reason: None,
            witness: None,
        }
    }

    fn rejects_area(&mut self, areas: impl Iterator<Item = Area>, view: Area) -> bool {
        if !self.early_metal {
            return false;
        }
        // Metal redraws one box. Union is invariant under tidy's band and
        // rectangle merges, so the final box can be rejected before tidy.
        let bounds = areas
            .map(|area| area.pixels().and(&view))
            .filter(|area| !area.is_empty())
            .reduce(|a, b| a.union(&b));
        if bounds.is_some_and(|bounds| bounds.area() > MAX_DAMAGE * view.area()) {
            self.reason = Some("area_bounding_box");
            true
        } else {
            false
        }
    }

    fn rejects_shift(&mut self) -> bool {
        if self.early_metal {
            self.reason = Some("shift_unsupported");
        }
        self.early_metal
    }
}

/// How to make the frame of `new` from the frame of `old` in a window of
/// `view`, or `None` to draw it whole.
#[cfg(any(test, feature = "test-support"))]
fn plan(old: &[Sig], new: &[Sig], view: Area) -> Option<Plan> {
    plan_traced(old, new, view, None, &mut Policy::default())
}

pub(super) fn plan_traced(
    old: &[Sig],
    new: &[Sig],
    view: Area,
    mut trace: Option<&mut Trace>,
    policy: &mut Policy,
) -> Option<Plan> {
    plan_stage(PlanStage::FixedCompare);
    if let Ok(plan) = fixed::plan(old, new, view, trace.as_deref_mut(), policy) {
        return plan;
    }
    general_plan(old, new, view, trace, policy)
}

fn general_plan(
    old: &[Sig],
    new: &[Sig],
    view: Area,
    mut trace: Option<&mut Trace>,
    policy: &mut Policy,
) -> Option<Plan> {
    plan_stage(PlanStage::MatchUnchanged);
    // The primitives that stayed as they were, as a multiset.
    // Intrusive index chains preserve first-to-last multiset pairing without
    // allocating one tiny Vec for every primitive in the previous frame.
    let mut left: HashMap<_, usize> = HashMap::default();
    let mut next = vec![usize::MAX; old.len()];
    for (i, s) in old.iter().enumerate().rev() {
        next[i] = left.insert(placed(s), i).unwrap_or(usize::MAX);
    }
    let mut stayed: Vec<Option<usize>> = vec![None; new.len()];
    let mut old_kept = vec![false; old.len()];
    for (i, s) in new.iter().enumerate() {
        if s.fresh {
            continue;
        }
        if let Some(head) = left.get_mut(&placed(s)) {
            if *head != usize::MAX {
                let j = *head;
                *head = next[j];
                stayed[i] = Some(j);
                old_kept[j] = true;
            }
        }
    }
    let changed_new: Vec<usize> = (0..new.len()).filter(|i| stayed[*i].is_none()).collect();
    let changed_old: Vec<usize> = (0..old.len()).filter(|j| !old_kept[*j]).collect();
    if let Some(trace) = trace.as_deref_mut() {
        trace.route = "general";
        trace.changed(
            changed_old.iter().map(|&j| old[j].reach),
            changed_new.iter().map(|&i| new[i].reach),
        );
    }
    plan_stage(PlanStage::ScrollVote);
    // A scroll: changed primitives of the same shape vote on how far they moved.
    let mut by_shape: HashMap<_, Vec<usize>> = HashMap::default();
    for &j in &changed_old {
        by_shape.entry(shape(&old[j])).or_default().push(j);
    }
    let mut votes: HashMap<i32, usize> = HashMap::default();
    let mut fractional = false;
    for &i in &changed_new {
        if new[i].fresh {
            continue;
        }
        for &j in by_shape.get(&shape(&new[i])).map_or(&[][..], Vec::as_slice) {
            let d = new[i].bounds.y0 - old[j].bounds.y0;
            if d.round() == 0.0 {
                continue;
            }
            if (d - d.round()).abs() < SAME {
                *votes.entry(d.round() as i32).or_default() += 1;
            } else {
                fractional = true;
            }
        }
    }
    let scroll = votes
        .iter()
        .max_by_key(|(dy, n)| (**n, -dy.abs()))
        .filter(|(_, n)| **n >= MIN_MOVED)
        .map(|(dy, _)| *dy as f32);
    if scroll.is_some() && policy.rejects_shift() {
        return None;
    }
    plan_stage(PlanStage::PaintOrder);
    let mut orders: Vec<(u32, u32)> = stayed
        .iter()
        .enumerate()
        .filter_map(|(i, j)| j.map(|j| (old[j].order, new[i].order)))
        .collect();

    let Some(dy) = scroll else {
        if fractional && !votes.is_empty() {
            if let Some(trace) = trace {
                trace.reason = "fractional_motion";
            }
            return None;
        }
        // No scroll: what changed is drawn again where it was and is.
        if !orders_agree(&mut orders)
            && !(policy.allow_disjoint_order && hover_order::disjoint(old, new, &stayed))
        {
            if order_probe::active() {
                policy.witness = order_probe::find(old, new, &stayed);
            }
            record_order_change(trace, &orders);
            return None;
        }
        plan_stage(PlanStage::RectMerge);
        let areas = || {
            changed_old
                .iter()
                .map(|&j| old[j].reach)
                .chain(changed_new.iter().map(|&i| new[i].reach))
        };
        if policy.rejects_area(areas(), view) {
            return None;
        }
        let damage = tidy(areas().collect(), &view);
        let total: f32 = damage.iter().map(Area::area).sum();
        record_damage(trace, &damage, total, view);
        plan_stage(PlanStage::BuildPlan);
        return (total <= MAX_DAMAGE * view.area()).then(|| Plan {
            shift: None,
            damage,
        });
    };

    plan_stage(PlanStage::ShiftMatch);
    // The scrolled clip: the masks of what moved by `dy`.
    let mut region: Option<Area> = None;
    for &i in &changed_new {
        let moved = by_shape.get(&shape(&new[i])).is_some_and(|js| {
            js.iter()
                .any(|&j| (new[i].bounds.y0 - old[j].bounds.y0 - dy).abs() < SAME)
        });
        if moved && !new[i].fresh {
            region = Some(region.map_or(new[i].mask, |r| r.union(&new[i].mask)));
        }
    }
    let Some(region) = region.map(|region| region.and(&view)) else {
        if let Some(trace) = trace {
            trace.reason = "no_shift_region";
        }
        return None;
    };
    if region.is_empty() || !region.on_grid() || dy.abs() >= region.y1 - region.y0 {
        if let Some(trace) = trace {
            trace.reason = "invalid_shift_region";
        }
        return None;
    }
    let region = region.pixels();
    // Where the copy lands, and where it comes from.
    let to = Area {
        y0: region.y0 + dy.max(0.0),
        y1: region.y1 + dy.min(0.0),
        ..region
    };
    let from = to.moved(-dy);

    // What moved with the scroll: the same shape, `dy` further, inside the
    // clip, showing the same part of itself where the copy lands.
    let mut pool: HashMap<_, Vec<usize>> = HashMap::default();
    for &j in &changed_old {
        pool.entry((shape(&old[j]), coarse(old[j].bounds.y0 + dy)))
            .or_default()
            .push(j);
    }
    let mut moved_new = vec![false; new.len()];
    let mut moved_old = vec![false; old.len()];
    for &i in &changed_new {
        let s = &new[i];
        if s.fresh || !s.reach.within(&region) {
            continue;
        }
        let Some(candidates) = pool.get_mut(&(shape(s), coarse(s.bounds.y0))) else {
            continue;
        };
        let fits = |j: &usize| {
            let o = &old[*j];
            o.bounds.moved(dy).same(&s.bounds)
                && o.reach.within(&region)
                && o.reach.moved(dy).and(&to).same(&s.reach.and(&to))
        };
        if let Some(k) = candidates.iter().position(fits) {
            let j = candidates.remove(k);
            moved_new[i] = true;
            moved_old[j] = true;
            orders.push((old[j].order, s.order));
        } else if !candidates.is_empty()
            && let Some(trace) = trace.as_deref_mut()
        {
            trace.coarse_rejections += 1;
        }
    }
    plan_stage(PlanStage::PaintOrder);
    if !orders_agree(&mut orders) {
        record_order_change(trace, &orders);
        return None;
    }

    plan_stage(PlanStage::RectMerge);
    let mut damage = Vec::new();
    // The strip the scroll exposed.
    damage.push(if dy < 0.0 {
        Area {
            y0: region.y1 + dy,
            ..region
        }
    } else {
        Area {
            y1: region.y0 + dy,
            ..region
        }
    });
    let copied = |r: &Area| r.and(&from).moved(dy);
    // What came, went or changed: where it was, where the copy took that,
    // and where it is.
    for &j in &changed_old {
        if !moved_old[j] {
            damage.push(old[j].reach);
            damage.push(copied(&old[j].reach));
        }
    }
    for &i in &changed_new {
        if !moved_new[i] {
            damage.push(new[i].reach);
        }
    }
    // What stayed over the clip while its pixels moved under it.
    for (i, s) in new.iter().enumerate() {
        if stayed[i].is_none() || !s.reach.touches(&region) {
            continue;
        }
        let over = s.reach.and(&region);
        if s.plain {
            // One colour: only the bands the copy moved its edges through,
            // and only its edges inside the clip (past the clip's edge the
            // copy brings nothing in from outside it).
            let reach = dy.abs();
            if s.reach.y0 > region.y0 + SAME {
                let top = Area {
                    y1: (over.y0 + reach).min(over.y1),
                    ..over
                };
                damage.extend([top, copied(&top)]);
            }
            if s.reach.y1 < region.y1 - SAME {
                let bottom = Area {
                    y0: (over.y1 - reach).max(over.y0),
                    ..over
                };
                damage.extend([bottom, copied(&bottom)]);
            }
        } else {
            damage.push(over);
            damage.push(copied(&over));
        }
    }
    let damage = tidy(damage, &view);
    let total: f32 = damage.iter().map(Area::area).sum();
    record_damage(trace, &damage, total, view);
    plan_stage(PlanStage::BuildPlan);
    (total <= MAX_DAMAGE * view.area()).then(|| Plan {
        shift: Some((from, to)),
        damage,
    })
}

fn record_damage(trace: Option<&mut Trace>, damage: &[Area], total: f32, view: Area) {
    if let Some(trace) = trace {
        trace.damage(damage);
        if total > MAX_DAMAGE * view.area() {
            trace.reason = "area_rectangles";
        }
    }
}

fn record_order_change(trace: Option<&mut Trace>, orders: &[(u32, u32)]) {
    if let Some(trace) = trace {
        trace.reason = "order_change";
        let mut max_prior = (0, 0);
        let mut max_current = (0, 0);
        for &(old, new) in orders {
            if old != max_current.0 && max_current.1 > max_prior.1 {
                max_prior = max_current;
            }
            if max_prior.1 > new {
                trace.order_conflict = Some([[max_prior.0, max_prior.1], [old, new]]);
                break;
            }
            if old != max_current.0 || new > max_current.1 {
                max_current = (old, new);
            }
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
mod tests {
    use super::{Area, Plan, Sig, orders_agree, plan};

    const VIEW: Area = Area {
        x0: 0.0,
        y0: 0.0,
        x1: 1000.0,
        y1: 800.0,
    };
    /// The scrolled clip of the tests.
    const CLIP: Area = Area {
        x0: 100.0,
        y0: 40.0,
        x1: 1000.0,
        y1: 800.0,
    };

    fn at(key: u64, order: u32, x: f32, y: f32, w: f32, h: f32, mask: Area) -> Sig {
        let bounds = Area {
            x0: x,
            y0: y,
            x1: x + w,
            y1: y + h,
        };
        Sig {
            key,
            order,
            bounds,
            mask,
            reach: bounds.and(&mask),
            plain: false,
            fresh: false,
        }
    }

    /// A window: a plain background, a header, and ten rows in the clip,
    /// scrolled by `offset`.
    fn frame(offset: f32) -> Vec<Sig> {
        let mut out = vec![
            Sig {
                plain: true,
                ..at(1, 1, 0.0, 0.0, 1000.0, 800.0, VIEW)
            },
            at(2, 2, 0.0, 0.0, 1000.0, 40.0, VIEW),
        ];
        for row in 0..10 {
            out.push(at(
                10 + row,
                3,
                120.0,
                60.0 + 100.0 * row as f32 + offset,
                600.0,
                80.0,
                CLIP,
            ));
        }
        out
    }

    fn total(plan: &Plan) -> f32 {
        plan.damage.iter().map(Area::area).sum()
    }

    #[cfg_attr(test, test)]
    fn an_unchanged_frame_needs_nothing() {
        let plan = plan(&frame(0.0), &frame(0.0), VIEW).unwrap();
        assert_eq!(
            plan,
            Plan {
                shift: None,
                damage: vec![]
            }
        );
    }

    #[cfg_attr(test, test)]
    fn a_scroll_copies_the_clip_and_damages_the_strip_it_exposed() {
        let plan = plan(&frame(0.0), &frame(-48.0), VIEW).unwrap();
        let (from, to) = plan.shift.unwrap();
        assert_eq!(to, Area { y1: 752.0, ..CLIP });
        assert_eq!(from, Area { y0: 88.0, ..CLIP });
        // The plain background runs past the clip: no bands of it, only
        // the strip.
        assert_eq!(plan.damage, vec![Area { y0: 752.0, ..CLIP }]);
        // The strip at the bottom, and the bands of the plain background at
        // the clip's top and bottom that the copy moved.
        assert!(plan.damage.iter().any(|r| r.y1 == 800.0 && r.y0 <= 752.0));
        assert!(total(&plan) < 0.25 * VIEW.area());
        // Scrolling back exposes the top.
        let back = super::plan(&frame(-48.0), &frame(0.0), VIEW).unwrap();
        assert_eq!(back.shift.unwrap().1, Area { y0: 88.0, ..CLIP });
    }

    #[cfg_attr(test, test)]
    fn a_change_without_a_scroll_damages_only_what_changed() {
        let mut new = frame(0.0);
        new[5].key = 99;
        let plan = plan(&frame(0.0), &new, VIEW).unwrap();
        assert_eq!(plan.shift, None);
        assert_eq!(plan.damage, vec![new[5].reach]);
    }

    #[cfg_attr(test, test)]
    fn what_does_not_move_with_the_scroll_is_damaged() {
        // A sticky bar over the clip that stays where it is.
        let bar = at(50, 9, 100.0, 40.0, 900.0, 30.0, CLIP);
        let mut old = frame(0.0);
        old.push(bar);
        let mut new = frame(-48.0);
        new.push(bar);
        let plan = plan(&old, &new, VIEW).unwrap();
        // Where it is, and where the copy took its pixels.
        let covered = |r: Area| plan.damage.iter().any(|d| r.within(d));
        assert!(covered(bar.reach));
        assert!(covered(
            Area {
                y0: 40.0,
                y1: 40.0 + 30.0,
                ..bar.reach
            }
            .moved(0.0)
        ));
    }

    #[cfg_attr(test, test)]
    fn a_fractional_scroll_copies_nothing() {
        // No pixel can be copied half a pixel further: what moved is drawn
        // again where it was and where it is.
        let plan = plan(&frame(0.0), &frame(-48.5), VIEW).unwrap();
        assert_eq!(plan.shift, None);
        assert!(!plan.damage.is_empty());
    }

    #[cfg_attr(test, test)]
    fn a_fresh_tile_is_damage_whatever_its_bytes() {
        let mut new = frame(0.0);
        new[4].fresh = true;
        let plan = plan(&frame(0.0), &new, VIEW).unwrap();
        assert_eq!(plan.damage, vec![new[4].reach]);
    }

    #[cfg_attr(test, test)]
    fn a_change_of_painting_order_is_drawn_whole() {
        // Two overlapping primitives that stay, their orders swapped.
        let a = at(70, 5, 200.0, 200.0, 100.0, 100.0, VIEW);
        let b = at(71, 6, 250.0, 250.0, 100.0, 100.0, VIEW);
        let old = vec![a, b];
        let new = vec![Sig { order: 6, ..a }, Sig { order: 5, ..b }];
        assert_eq!(plan(&old, &new, VIEW), None);
        // Orders renumbered alike are the same painting order.
        let renumbered = vec![Sig { order: 8, ..a }, Sig { order: 9, ..b }];
        assert!(plan(&old, &renumbered, VIEW).is_some());
        assert!(orders_agree(&mut [(1, 1), (1, 2), (2, 2)]));
        assert!(!orders_agree(&mut [(1, 3), (2, 2)]));
    }

    #[cfg_attr(test, test)]
    fn primitives_alike_at_one_place_keep_their_sequence() {
        // Two backgrounds, one over the other, alike in everything but order.
        let mut old = frame(0.0);
        old.insert(1, Sig { order: 2, ..old[0] });
        assert_eq!(
            plan(&old, &old, VIEW),
            Some(Plan {
                shift: None,
                damage: vec![]
            })
        );
        // And when the content between them scrolls.
        let mut new = frame(-48.0);
        new.insert(1, Sig { order: 2, ..new[0] });
        assert!(plan(&old, &new, VIEW).unwrap().shift.is_some());
    }

    #[cfg_attr(test, test)]
    fn a_scroll_between_pixels_is_a_scroll_whatever_the_last_bits() {
        // Rows a quarter of a pixel off the grid (a 125 % display), their
        // new places a bit of rounding off the old ones plus the scroll.
        let rows = |offset: f32, noise: u32| -> Vec<Sig> {
            let mut out = frame(0.0);
            for (n, s) in out.iter_mut().enumerate().skip(2) {
                let y = f32::from_bits(
                    (s.bounds.y0 + 0.25 + offset).to_bits() + noise * (n as u32 % 2),
                );
                let bounds = Area {
                    y0: y,
                    y1: y + 80.0,
                    ..s.bounds
                };
                *s = Sig {
                    bounds,
                    reach: bounds.and(&s.mask),
                    ..*s
                };
            }
            out
        };
        let plan = plan(&rows(0.0, 0), &rows(-48.0, 1), VIEW).unwrap();
        assert!(plan.shift.is_some());
        assert_eq!(plan.damage, vec![Area { y0: 752.0, ..CLIP }]);
    }

    #[cfg_attr(test, test)]
    fn a_page_of_changes_is_planned_quickly() {
        // Thousands of small changes: gathered into bands, not merged pair
        // by pair.
        let cells = |key: u64| -> Vec<Sig> {
            (0..6000u64)
                .map(|n| {
                    at(
                        key + n,
                        1,
                        (n % 100) as f32 * 10.0,
                        (n / 100) as f32 * 13.0,
                        6.0,
                        6.0,
                        VIEW,
                    )
                })
                .collect()
        };
        let started = std::time::Instant::now();
        assert_eq!(plan(&cells(0), &cells(100_000), VIEW), None);
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
    }

    #[cfg_attr(test, test)]
    fn a_clip_off_the_pixel_grid_is_drawn_whole() {
        let clip = Area { y0: 40.5, ..CLIP };
        let rows = |offset: f32| -> Vec<Sig> {
            (0..10)
                .map(|row| {
                    at(
                        10 + row,
                        3,
                        120.0,
                        60.0 + 100.0 * row as f32 + offset,
                        600.0,
                        80.0,
                        clip,
                    )
                })
                .collect()
        };
        assert_eq!(plan(&rows(0.0), &rows(-48.0), VIEW), None);
    }

    #[cfg_attr(test, test)]
    fn too_much_damage_is_drawn_whole() {
        let old = frame(0.0);
        let new: Vec<Sig> = old
            .iter()
            .map(|s| Sig {
                key: s.key + 1000,
                ..*s
            })
            .collect();
        assert_eq!(plan(&old, &new, VIEW), None);
    }
    #[cfg_attr(test, test)]
    fn signatures_use_current_scene_fields_and_decline_scroll_tiles() {
        let mut scene = gpui::Scene::default();
        let bounds = gpui::Bounds {
            origin: gpui::point(gpui::ScaledPixels(10.), gpui::ScaledPixels(20.)),
            size: gpui::size(gpui::ScaledPixels(60.), gpui::ScaledPixels(24.)),
        };
        scene.insert_primitive(gpui::Quad {
            bounds,
            content_mask: gpui::ContentMask { bounds },
            background: gpui::Hsla {
                h: 0.,
                s: 1.,
                l: 0.5,
                a: 1.,
            }
            .into(),
            ..Default::default()
        });
        scene.finish();
        let old = super::signatures(&scene, &Default::default(), &|_| false).unwrap();
        assert_eq!(old.len(), 1);
        scene.quads[0].background = gpui::Hsla {
            h: 0.5,
            s: 1.,
            l: 0.5,
            a: 1.,
        }
        .into();
        let new = super::signatures(&scene, &Default::default(), &|_| false).unwrap();
        let update = plan(&old, &new, VIEW).unwrap();
        assert!(update.shift.is_none());
        assert!(!update.damage.is_empty());
        scene.layers.frames.push(gpui::LayerFrame {
            key: gpui::LayerKey(7),
            generation: 1,
            background: gpui::Rgba {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 1.,
            },
            tile_size: 64,
            content: gpui::Scene::default().into(),
            dirty_tiles: Vec::new(),
        });
        assert!(super::signatures(&scene, &Default::default(), &|_| false).is_none());
    }

    #[cfg_attr(test, test)]
    fn scenes_with_surfaces_are_not_comparable() {
        use objc2_core_video::{
            CVPixelBuffer, CVPixelBufferCreate, kCVPixelFormatType_32BGRA, kCVReturnSuccess,
        };
        let mut buffer: *mut CVPixelBuffer = std::ptr::null_mut();
        assert_eq!(
            unsafe {
                CVPixelBufferCreate(
                    None,
                    1,
                    1,
                    kCVPixelFormatType_32BGRA,
                    None,
                    std::ptr::NonNull::from(&mut buffer),
                )
            },
            kCVReturnSuccess
        );
        let buffer = unsafe {
            objc2_core_foundation::CFRetained::from_raw(std::ptr::NonNull::new(buffer).unwrap())
        };
        let bounds = gpui::Bounds {
            origin: gpui::point(gpui::ScaledPixels(0.), gpui::ScaledPixels(0.)),
            size: gpui::size(gpui::ScaledPixels(1.), gpui::ScaledPixels(1.)),
        };
        let mut scene = gpui::Scene::default();
        scene.insert_primitive(gpui::PaintSurface {
            order: 0,
            bounds,
            content_mask: gpui::ContentMask { bounds },
            image_buffer: buffer,
        });
        scene.finish();
        assert!(super::signatures(&scene, &Default::default(), &|_| false).is_none());
    }

    #[cfg_attr(test, test)]
    fn unsupported_shift_returns_before_building_rectangles() {
        use super::{Policy, Trace, plan_traced};
        let old = frame(0.);
        let new = frame(-12.);
        assert!(plan(&old, &new, VIEW).unwrap().shift.is_some());
        let mut policy = Policy {
            early_metal: true,
            ..Policy::default()
        };
        let mut trace = Trace::test_sample();
        assert!(plan_traced(&old, &new, VIEW, Some(&mut trace), &mut policy).is_none());
        assert_eq!(policy.reason, Some("shift_unsupported"));
        assert!(!trace.damage_was_built());
    }

    #[cfg_attr(test, test)]
    fn oversized_box_returns_before_building_a_plan() {
        use super::{Policy, Trace, plan_traced};
        let old = [at(1, 1, 0., 0., 1000., 800., VIEW)];
        let new = [Sig { key: 2, ..old[0] }];
        let mut policy = Policy {
            early_metal: true,
            ..Policy::default()
        };
        let mut trace = Trace::test_sample();
        assert!(plan_traced(&old, &new, VIEW, Some(&mut trace), &mut policy).is_none());
        assert_eq!(policy.reason, Some("area_bounding_box"));
        assert!(!trace.damage_was_built());
    }

    #[cfg_attr(test, test)]
    fn hover_highlight_reproduces_an_order_group_inversion() {
        let first = at(10, 2, 50., 50., 15., 15., VIEW);
        let second = at(11, 3, 50., 80., 15., 15., VIEW);
        let highlight = at(12, 1, 40., 45., 300., 30., VIEW);
        let old = [first, second, highlight];
        let new = [
            Sig { order: 3, ..first },
            Sig { order: 2, ..second },
            at(12, 1, 40., 75., 300., 30., VIEW),
        ];
        assert!(plan(&old, &new, VIEW).is_none());
        let witness = super::order_probe::find(&old, &new, &[Some(0), Some(1), None]).unwrap();
        assert!(!witness.overlap);
        let overlapping = Sig {
            reach: first.reach,
            ..second
        };
        let old_overlap = [first, overlapping];
        let new_overlap = [
            new[0],
            Sig {
                order: 2,
                ..overlapping
            },
        ];
        let witness =
            super::order_probe::find(&old_overlap, &new_overlap, &[Some(0), Some(1)]).unwrap();
        assert!(witness.overlap);
    }

    #[cfg_attr(test, test)]
    fn hover_checks_aa_and_shadow_reach_before_accepting_order_inversions() {
        let first = at(10, 2, 50., 50., 15., 15., VIEW);
        let against = |second: Sig| {
            let old = [first, second];
            let new = [Sig { order: 3, ..first }, Sig { order: 2, ..second }];
            let mut policy = super::Policy {
                allow_disjoint_order: true,
                ..Default::default()
            };
            super::plan_traced(&old, &new, VIEW, None, &mut policy)
        };
        let separate = at(11, 3, 50., 80., 15., 15., VIEW);
        assert!(against(separate).is_some_and(|plan| plan.damage.is_empty()));
        assert!(against(at(11, 3, 50., 60., 15., 15., VIEW)).is_none());
        // The geometric bounds are 1 px apart, but antialias margins meet.
        assert!(against(at(11, 3, 50., 66., 15., 15., VIEW)).is_none());
        // A shadow with blur 6 has reach 3×6+1 px in the Metal signatures.
        let shadow = Sig {
            reach: separate.bounds.grown(19.),
            ..separate
        };
        assert!(against(shadow).is_none());
    }

    pub(super) fn run() {
        hover_checks_aa_and_shadow_reach_before_accepting_order_inversions();
        hover_highlight_reproduces_an_order_group_inversion();
        oversized_box_returns_before_building_a_plan();
        unsupported_shift_returns_before_building_rectangles();
        an_unchanged_frame_needs_nothing();
        a_scroll_copies_the_clip_and_damages_the_strip_it_exposed();
        a_change_without_a_scroll_damages_only_what_changed();
        what_does_not_move_with_the_scroll_is_damaged();
        a_fractional_scroll_copies_nothing();
        a_fresh_tile_is_damage_whatever_its_bytes();
        a_change_of_painting_order_is_drawn_whole();
        primitives_alike_at_one_place_keep_their_sequence();
        a_scroll_between_pixels_is_a_scroll_whatever_the_last_bits();
        a_page_of_changes_is_planned_quickly();
        a_clip_off_the_pixel_grid_is_drawn_whole();
        too_much_damage_is_drawn_whole();
        signatures_use_current_scene_fields_and_decline_scroll_tiles();
        scenes_with_surfaces_are_not_comparable();
    }
}

#[cfg(any(test, feature = "test-support"))]
impl crate::metal_renderer::MetalRenderer {
    /// Runs the pinned planner tests and current Scene compatibility checks.
    pub fn test_damage_planner() {
        tests::run();
        fixed::test_fixed();
        super::inputs::test_upload_observers();
        super::inputs::test_atlas_hooks().expect("Metal atlas upload/remove hooks");
    }
}
