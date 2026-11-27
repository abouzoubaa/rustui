// SPDX-License-Identifier: Apache-2.0
//! Compare the image a blit would produce to the new scene inside each destination.
use super::{Area, Sig};
use collections::HashMap;
use gpui::FastScrollViewport;

#[derive(Default, Debug)]
pub(crate) struct ScrollPlan {
    pub(crate) copies: Vec<(Area, Area)>,
    pub(crate) damage: Vec<Area>,
    pub(crate) exposed: Vec<Area>,
    pub(crate) reason: &'static str,
    pub(crate) shifted: bool,
}
fn translate(a: Area, dx: f32, dy: f32) -> Area {
    Area {
        x0: a.x0 + dx,
        x1: a.x1 + dx,
        y0: a.y0 + dy,
        y1: a.y1 + dy,
    }
}
fn integral(a: Area) -> bool {
    [a.x0, a.y0, a.x1, a.y1]
        .into_iter()
        .all(|v| v.is_finite() && v.fract() == 0.)
}
pub(crate) fn subtract(a: Area, cut: Area, out: &mut Vec<Area>) {
    let hit = a.and(&cut);
    if hit.is_empty() {
        out.push(a);
        return;
    }
    for r in [
        Area { y1: hit.y0, ..a },
        Area { y0: hit.y1, ..a },
        Area {
            x1: hit.x0,
            y0: hit.y0,
            y1: hit.y1,
            ..a
        },
        Area {
            x0: hit.x1,
            y0: hit.y0,
            y1: hit.y1,
            ..a
        },
    ] {
        if !r.is_empty() {
            out.push(r);
        }
    }
}
impl ScrollPlan {
    pub(crate) fn dirty(&mut self, rect: Area) {
        if rect.is_empty() {
            return;
        }
        let mut r = rect.pixels();
        let mut i = 0;
        while i < self.damage.len() {
            let other = self.damage[i];
            let joined = other.union(&r);
            // Edge glyphs often produce several tiny repairs on the same scanlines.
            // A short gap is cheaper to redraw than another Metal render pass.
            let band = other.y0 == r.y0
                && other.y1 == r.y1
                && other.x0 <= r.x1 + 64.
                && r.x0 <= other.x1 + 64.;
            if band || (other.touches(&r) && joined.area() <= 1.2 * (other.area() + r.area())) {
                r = joined;
                self.damage.swap_remove(i);
                i = 0;
            } else {
                i += 1;
            }
        }
        self.damage.push(r);
    }
    pub(crate) fn bounds(&self) -> Option<Area> {
        self.damage.iter().copied().reduce(|a, b| a.union(&b))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Placement {
    appearance: u64,
    bounds: [u32; 4],
    clip: [u32; 4],
    reach: [u32; 4],
}
struct Item {
    place: Placement,
    reach: Area,
    fresh: bool,
    order: (u32, u8),
}
#[derive(Default)]
pub(crate) struct Scratch {
    before: Vec<Item>,
    after: Vec<Item>,
}
fn items(sigs: &[Sig], dest: Area, dx: f32, dy: f32, out: &mut Vec<Item>) {
    out.clear();
    let src = translate(dest, -dx, -dy);
    for s in sigs {
        let reach = s.reach.and(&src);
        if reach.is_empty() {
            continue;
        }
        let reach = translate(reach, dx, dy);
        let bounds = translate(s.bounds, dx, dy);
        let clip = translate(s.mask.and(&src), dx, dy);
        // Inside a uniform fill the original rectangle's origin cannot affect a pixel.
        let flat = s.plain
            && dest.and(&bounds) == dest
            && dest.and(&clip) == dest
            && integral(bounds)
            && integral(clip);
        let place = Placement {
            appearance: s.key,
            bounds: if flat { [0; 4] } else { bounds.bits() },
            clip: clip.bits(),
            reach: reach.bits(),
        };
        out.push(Item {
            place,
            reach,
            fresh: s.fresh,
            order: (s.order, s.batch_rank),
        });
    }
    // Stable ties preserve GPUI's batch-kind and insertion order.
    if !out.is_sorted_by_key(|p| p.order) {
        out.sort_by_key(|p| p.order);
    }
}
fn compare(
    old: &[Sig],
    new: &[Sig],
    dest: Area,
    dx: f32,
    dy: f32,
    plan: &mut ScrollPlan,
    view: Area,
    scratch: &mut Scratch,
) -> bool {
    items(old, dest, dx, dy, &mut scratch.before);
    items(new, dest, 0., 0., &mut scratch.after);
    let before = &scratch.before;
    let after = &scratch.after;
    let equal = |a: &Item, b: &Item| !b.fresh && a.place == b.place;
    let prefix = before
        .iter()
        .zip(after)
        .take_while(|(a, b)| equal(a, b))
        .count();
    let a = &before[prefix..];
    let b = &after[prefix..];
    let suffix = a
        .iter()
        .rev()
        .zip(b.iter().rev())
        .take_while(|(a, b)| equal(a, b))
        .count();
    let a = &a[..a.len() - suffix];
    let b = &b[..b.len() - suffix];
    if a.is_empty() && b.is_empty() {
        return false;
    }
    let mut by_shape = HashMap::default();
    for (i, item) in a.iter().enumerate().rev() {
        by_shape.entry(item.place).or_insert_with(Vec::new).push(i);
    }
    let mut used = vec![false; a.len()];
    let mut last = None;
    for item in b {
        if !item.fresh
            && let Some(i) = by_shape.get_mut(&item.place).and_then(Vec::pop)
        {
            if last.is_some_and(|prev| prev >= i) {
                // Reordered matching primitives can change their overlap.
                plan.dirty(dest);
                return true;
            }
            used[i] = true;
            last = Some(i);
        } else {
            plan.dirty(item.reach.grown(1.).and(&view));
        }
    }
    for (i, item) in a.iter().enumerate() {
        if !used[i] {
            plan.dirty(item.reach.grown(1.).and(&view));
        }
    }
    true
}
fn translucent_overlay(sigs: &[Sig], id: u64, clip: Area) -> bool {
    let base = sigs
        .iter()
        .filter(|s| s.owner == id && !s.reach.and(&clip).is_empty())
        .map(|s| (s.order, s.batch_rank))
        .min();
    base.is_some_and(|base| {
        sigs.iter().any(|s| {
            s.owner != id
                && !s.opaque
                && (s.order, s.batch_rank) >= base
                && !s.reach.and(&clip).is_empty()
        })
    })
}
fn fixed_overlay(old: &[Sig], new: &[Sig], id: u64, clip: Area, dx: f32, dy: f32) -> bool {
    // Scrolling text is translucent too. An overlay stays in screen coordinates
    // while the content under it moves. First look up the projected old position.
    let old_positions: collections::HashSet<_> = old
        .iter()
        .filter(|s| s.owner == id && !s.opaque && !s.reach.and(&clip).is_empty())
        .map(|s| (s.key, s.bounds.bits()))
        .collect();
    if old_positions.is_empty() {
        return false;
    }
    new.iter().any(|s| {
        s.owner == id
            && !s.opaque
            && !s.reach.and(&clip).is_empty()
            && !old_positions.contains(&(s.key, translate(s.bounds, -dx, -dy).bits()))
            && old.iter().any(|previous| {
                previous.owner == id && !previous.opaque && previous.bounds == s.bounds
            })
            && new.iter().any(|below| {
                below.owner == id
                    && (below.order, below.batch_rank) <= (s.order, s.batch_rank)
                    && !std::ptr::eq(below, s)
                    && !below.reach.and(&s.reach).and(&clip).is_empty()
            })
    })
}
pub(crate) enum Decision {
    NoMotion,
    Direct(&'static str),
    Planned(ScrollPlan),
}
pub(crate) fn plan(
    old: &[Sig],
    new: &[Sig],
    before: &[FastScrollViewport],
    after: &[FastScrollViewport],
    view: Area,
    scratch: &mut Scratch,
) -> Decision {
    let mut changes = Vec::new();
    let mut clip_changed = false;
    for v in after {
        let Some(prior) = before.iter().find(|old| old.id == v.id) else {
            continue;
        };
        if prior.offset == v.offset {
            continue;
        }
        let clip = Area::of(&v.clip).and(&view);
        if clip.is_empty() {
            continue;
        }
        clip_changed |= prior.clip != v.clip || !integral(clip);
        if changes
            .iter()
            .any(|(_, other, _, _): &(_, Area, _, _)| !clip.and(other).is_empty())
        {
            return Decision::Direct("nested_overlap");
        }
        let dx = v.offset.x.0 - prior.offset.x.0;
        let dy = v.offset.y.0 - prior.offset.y.0;
        changes.push((v.id, clip, dx, dy));
    }
    if clip_changed {
        return Decision::Direct("clip_changed");
    }
    if changes.is_empty() {
        return Decision::NoMotion;
    }
    let mut result = ScrollPlan {
        reason: "shift",
        ..Default::default()
    };
    let mut stationary = vec![view];
    for (id, clip, dx, dy) in changes {
        let has_alpha_fill = new
            .iter()
            .any(|s| s.owner == id && s.plain && !s.opaque && !s.reach.and(&clip).is_empty());
        if translucent_overlay(old, id, clip)
            || translucent_overlay(new, id, clip)
            || (has_alpha_fill && fixed_overlay(old, new, id, clip, dx, dy))
        {
            return Decision::Direct("overlay_translucent");
        }
        let mut next = Vec::new();
        for area in stationary.drain(..) {
            subtract(area, clip, &mut next);
        }
        stationary = next;
        if dx.fract() != 0. || dy.fract() != 0. {
            result.reason = "fractional_delta";
            result.dirty(clip);
            continue;
        }
        if dx.abs() >= clip.x1 - clip.x0 || dy.abs() >= clip.y1 - clip.y0 {
            result.reason = "delta_too_large";
            result.dirty(clip);
            continue;
        }
        let to = clip.and(&translate(clip, dx, dy));
        let from = translate(to, -dx, -dy);
        result.copies.push((from, to));
        result.shifted = true;
        let mut strips = Vec::new();
        subtract(clip, to, &mut strips);
        result.exposed.extend_from_slice(&strips);
        for r in strips {
            result.dirty(r);
        }
        if compare(old, new, to, dx, dy, &mut result, view, scratch)
            && !has_alpha_fill
            && fixed_overlay(old, new, id, clip, dx, dy)
        {
            return Decision::Direct("overlay_translucent");
        }
    }
    for r in stationary {
        result.copies.push((r, r));
        compare(old, new, r, 0., 0., &mut result, view, scratch);
    }
    if result.damage.len() > 16
        && let Some(bounds) = result.bounds()
    {
        result.damage.clear();
        result.damage.push(bounds);
    }
    Decision::Planned(result)
}
