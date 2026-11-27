// SPDX-License-Identifier: Apache-2.0
//! Keep ordinary signatures for the scene surrounding an explicitly declared list.
use super::*;
use gpui::FastScrollViewport;

pub(crate) fn collect(
    scene: &Scene,
    uploaded: &HashSet<(AtlasTextureId, u32)>,
    dynamic: &dyn Fn(AtlasTextureId) -> bool,
    cache: &mut Cache,
) -> Option<(Vec<Sig>, Option<FastScrollViewport>)> {
    if let [viewport] = scene.fast_scroll_viewports()
        && viewport.content_revision.is_some()
        && uploaded.is_empty()
        && let Some(outside) = outside_rows(scene, *viewport)
    {
        let sigs = signatures_cached(&outside, uploaded, dynamic, cache)?;
        return Some((sigs, Some(*viewport)));
    }
    signatures_cached(scene, uploaded, dynamic, cache).map(|sigs| (sigs, None))
}

// A list may promise that row-local pixels are stable. Untracked textures,
// transformed glyphs, and overlapping foreground content retain the ordinary diff.
fn outside_rows(scene: &Scene, viewport: FastScrollViewport) -> Option<Scene> {
    if !scene.shadows.is_empty()
        || !scene.underlines.is_empty()
        || !scene.paths.is_empty()
        || !scene.subpixel_sprites.is_empty()
        || !scene.polychrome_sprites.is_empty()
        || !scene.layers.frames.is_empty()
        || !scene.surfaces.is_empty()
    {
        return None;
    }
    let clip = Area::of(&viewport.clip);
    let mut outside = Scene::default();
    let mut first = u32::MAX;
    let mut background_order = None;
    for (i, q) in scene.quads.iter().enumerate() {
        let bounds = Area::of(&q.bounds);
        let reach = bounds.and(&Area::of(&q.content_mask.bounds));
        if scene.fast_scroll_owner(0, i) == viewport.id {
            if !reach.is_empty() && !reach.within(&clip) {
                return None;
            }
            first = first.min(q.order);
        } else {
            if !reach.and(&clip).is_empty() {
                if !clip.within(&reach)
                    || !q.background.as_solid().is_some_and(|c| c.a == 1.)
                    || q.corner_radii != Default::default()
                    || q.border_widths != Default::default()
                {
                    return None;
                }
                background_order = Some(background_order.map_or(q.order, |n: u32| n.max(q.order)));
            }
            outside.quads.push(*q);
        }
    }
    for (i, g) in scene.monochrome_sprites.iter().enumerate() {
        if !is_unit(&g.transformation) || crate::fast::damage::inputs::dynamic(g.tile.texture_id) {
            return None;
        }
        let reach = Area::of(&g.bounds)
            .grown(1.)
            .and(&Area::of(&g.content_mask.bounds));
        if scene.fast_scroll_owner(3, i) == viewport.id {
            if !reach.is_empty() && !reach.within(&clip) {
                return None;
            }
            first = first.min(g.order);
        } else {
            if !reach.and(&clip).is_empty() {
                return None;
            }
            outside.monochrome_sprites.push(*g);
        }
    }
    if background_order.is_some_and(|o| o >= first) {
        return None;
    }
    outside.record_scroll_viewport(viewport);
    Some(outside)
}

pub(crate) fn changed(
    old: Option<FastScrollViewport>,
    new: Option<FastScrollViewport>,
) -> Option<Area> {
    if old.zip(new).is_some_and(|(a, b)| {
        a.id == b.id
            && a.clip == b.clip
            && a.content_revision == b.content_revision
            && a.row_height == b.row_height
    }) {
        return None;
    }
    old.into_iter()
        .chain(new)
        .map(|v| Area::of(&v.clip))
        .reduce(|a, b| a.union(&b))
}

// Virtualization may omit a glyph in the partly visible trailing row.
// That row is content damage when it leaves the painted row set.
pub(crate) fn edge(old: FastScrollViewport, new: FastScrollViewport) -> Option<Area> {
    let clip = Area::of(&new.clip);
    let dy = new.offset.y.0 - old.offset.y.0;
    let h = new.row_height.ceil().min(clip.y1 - clip.y0);
    if h <= 0. || dy == 0. {
        return None;
    }
    Some(if dy < 0. {
        Area {
            y1: clip.y0 + h,
            ..clip
        }
    } else {
        Area {
            y0: clip.y1 - h,
            ..clip
        }
    })
}
