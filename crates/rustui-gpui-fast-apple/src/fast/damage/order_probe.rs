// SPDX-License-Identifier: Apache-2.0
//! Identify the actual primitive pairs behind the planner's rank inversion.
use super::plan::{Area, Sig};
use gpui::Scene;
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const LIMIT: usize = 20;
static LOGGED: AtomicUsize = AtomicUsize::new(0);

pub(super) fn active() -> bool {
    gpui::Scene::fast_damage_order_probe_enabled() && LOGGED.load(Ordering::Relaxed) < LIMIT
}

pub(super) type Identity = (u64, usize);
pub(super) fn identities(scene: &Scene) -> Vec<Option<Identity>> {
    [
        scene.quads.len(),
        scene.shadows.len(),
        scene.underlines.len(),
        scene.monochrome_sprites.len(),
        scene.subpixel_sprites.len(),
        scene.polychrome_sprites.len(),
        scene.paths.len(),
    ]
    .into_iter()
    .enumerate()
    .flat_map(|(kind, len)| {
        (0..len).map(move |index| scene.fast_damage_order_identity(kind, index))
    })
    .collect()
}

#[derive(Clone, Copy)]
pub(super) struct Primitive {
    pair: (usize, usize),
    orders: [u32; 2],
    bounds: [Area; 2],
    reach: [Area; 2],
}
#[derive(Clone, Copy)]
pub(super) struct Witness {
    primitives: [Primitive; 2],
    pub overlap: bool,
}

pub(super) fn find(old: &[Sig], new: &[Sig], stayed: &[Option<usize>]) -> Option<Witness> {
    let pairs = || {
        stayed.iter().enumerate().filter_map(|(i, j)| {
            j.map(|j| {
                let (a, b) = (old[j], new[i]);
                Primitive {
                    pair: (j, i),
                    orders: [a.order(), b.order()],
                    bounds: [a.bounds(), b.bounds()],
                    reach: [a.order_reach(), b.order_reach()],
                }
            })
        })
    };
    let mut first = None;
    for a in pairs() {
        for b in pairs() {
            if a.orders[0] < b.orders[0] && a.orders[1] > b.orders[1] {
                let overlap = (0..2).any(|i| a.reach[i].touches(&b.reach[i]));
                let witness = Witness {
                    primitives: [a, b],
                    overlap,
                };
                if overlap {
                    return Some(witness);
                }
                first.get_or_insert(witness);
            }
        }
    }
    first
}

fn rect(area: Area) -> [f32; 4] {
    [area.x0, area.y0, area.x1, area.y1]
}
pub(super) fn emit(witness: Witness, old: &[Option<Identity>], new: &[Option<Identity>]) {
    let sequence = LOGGED.fetch_add(1, Ordering::Relaxed);
    if sequence >= LIMIT {
        return;
    }
    let epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * 1000.;
    let [a, b] = witness.primitives;
    let lookup = |p: Primitive| {
        [
            old.get(p.pair.0).copied().flatten(),
            new.get(p.pair.1).copied().flatten(),
        ]
    };
    let ids = |p| lookup(p).map(|entry| entry.map_or(0, |entry| entry.0));
    let indices = |p| lookup(p).map(|entry| entry.map_or(usize::MAX, |entry| entry.1));
    super::log::line(format_args!(
        r#"RUSTUI_FRAME_LOG {{"event":"damage_order_rejection","cpu_ms":0.0,"epoch_ms":{epoch_ms:.6},"sequence":{sequence},"overlap":{},"element_a":{:?},"element_b":{:?},"paint_index_a":{:?},"paint_index_b":{:?},"order_a":{:?},"order_b":{:?},"bounds_a":{:?},"bounds_b":{:?},"reach_a":{:?},"reach_b":{:?}}}"#,
        witness.overlap,
        ids(a),
        ids(b),
        indices(a),
        indices(b),
        a.orders,
        b.orders,
        a.bounds.map(rect),
        b.bounds.map(rect),
        a.reach.map(rect),
        b.reach.map(rect),
    ));
}
