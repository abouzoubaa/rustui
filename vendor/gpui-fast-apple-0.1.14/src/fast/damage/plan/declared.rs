// SPDX-License-Identifier: Apache-2.0
//! Replace adjacent primitives painted by the same immutable row with one
//! signature. Keep renderer kind, draw order and clip boundaries intact.
use super::{Area, Key, Sig};
pub(super) fn push(
    out: &mut Vec<Sig>,
    owner: u64,
    revision: u64,
    anchor: [u32; 3],
    kind: u64,
    order: u32,
    bounds: Area,
    mask: Area,
    fresh: bool,
) {
    let key = Key::new(31)
        .u(owner)
        .u(revision)
        .u(kind)
        .u(anchor[0] as u64)
        .u(anchor[1] as u64)
        .u(anchor[2] as u64)
        .done();
    let fresh = fresh || revision == 0;
    if let Some(last) = out.last_mut()
        && last.key == key
        && last.order == order
        && last.bounds == bounds
        && last.mask == mask
    {
        last.fresh |= fresh;
        return;
    }
    out.push(Sig {
        key,
        order,
        bounds,
        mask,
        reach: bounds.grown(2.).and(&mask),
        plain: false,
        owner: 0,
        opaque: false,
        batch_rank: 0,
        fresh,
    });
}
