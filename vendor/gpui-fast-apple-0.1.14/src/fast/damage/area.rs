// SPDX-License-Identifier: Apache-2.0
//! Verification-only reporter for editor declarations and consecutive complete images.
use super::plan::Area;
use gpui::Scene;
use std::sync::OnceLock;

pub(super) fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("RUSTUI_G_R3_AREA").as_deref() == Ok("1"))
}
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Decl {
    owner: u64,
    revision: u64,
    anchor: [u32; 3],
    bounds: Area,
}
pub(super) fn snapshot(scene: &Scene) -> Vec<Decl> {
    if !enabled() {
        return Vec::new();
    }
    scene
        .fast_damage_regions_for_reporting()
        .iter()
        .map(|r| Decl {
            owner: r.owner,
            revision: r.revision,
            anchor: r.anchor,
            bounds: Area::of(&r.bounds),
        })
        .collect()
}
pub(super) fn declared(old: &[Decl], current: &[Decl], scene: &Scene, w: usize, h: usize) -> usize {
    let mut unmatched = old.to_vec();
    let mut rects = Vec::new();
    for d in current {
        if d.revision != 0
            && let Some(i) = unmatched.iter().position(|r| r == d)
        {
            unmatched.swap_remove(i);
        } else {
            rects.push(d.bounds)
        }
    }
    rects.extend(unmatched.iter().map(|d| d.bounds));
    rects.extend(scene.extra_damage_rects().iter().map(Area::of));
    let viewport = Area {
        x0: 0.,
        y0: 0.,
        x1: w as f32,
        y1: h as f32,
    };
    let padded: Vec<_> = rects
        .iter()
        .map(|r| r.grown(1.).pixels().and(&viewport))
        .filter(|r| !r.is_empty())
        .collect();
    // Count the union in device pixels, including overlaps only once.
    let mut spans = Vec::with_capacity(padded.len());
    let mut total = 0;
    for y in 0..h {
        spans.clear();
        spans.extend(
            padded
                .iter()
                .filter(|r| r.y0 <= y as f32 && (y as f32) < r.y1)
                .map(|r| (r.x0 as usize, r.x1 as usize)),
        );
        spans.sort_unstable();
        let mut end = 0;
        for &(left, right) in &spans {
            total += right.saturating_sub(left.max(end));
            end = end.max(right);
        }
    }
    total
}
pub(super) fn emit(old: &[u8], new: &[u8], declared: usize, scissor: usize, w: usize, h: usize) {
    let changed = old
        .chunks_exact(4)
        .zip(new.chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count();
    super::log::line(format_args!(
        "RUSTUI_G_R3_AREA {w} {h} {declared} {changed} {scissor}"
    ));
}
