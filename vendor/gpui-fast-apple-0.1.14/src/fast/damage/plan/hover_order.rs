// SPDX-License-Identifier: Apache-2.0
//! A stationary order inversion is harmless only when every inverted pair is disjoint.
use super::{Area, Sig};

struct Entry {
    order: (u32, u32),
    reach: Area,
}

struct Group<'a> {
    order: (u32, u32),
    reach: Area,
    entries: &'a [Entry],
}

pub(super) fn disjoint(old: &[Sig], new: &[Sig], stayed: &[Option<usize>]) -> bool {
    let mut entries = stayed
        .iter()
        .zip(new)
        .filter_map(|(previous, current)| {
            let previous = &old[(*previous)?];
            Some(Entry {
                order: (previous.order, current.order),
                // Check both frames' conservative raster reach. In particular,
                // shadow reach already covers the kernel, not just the source quad.
                reach: previous.order_reach().union(&current.order_reach()),
            })
        })
        .collect::<Vec<_>>();
    entries.sort_unstable_by_key(|entry| entry.order);
    let groups = entries
        .chunk_by(|a, b| a.order == b.order)
        .map(|entries| Group {
            order: entries[0].order,
            reach: entries[1..]
                .iter()
                .fold(entries[0].reach, |reach, entry| reach.union(&entry.reach)),
            entries,
        })
        .collect::<Vec<_>>();

    for (index, a) in groups.iter().enumerate() {
        for b in &groups[index + 1..] {
            if a.order.0 < b.order.0
                && a.order.1 > b.order.1
                && a.reach.touches(&b.reach)
                && a.entries
                    .iter()
                    .any(|a| b.entries.iter().any(|b| a.reach.touches(&b.reach)))
            {
                return false;
            }
        }
    }
    true
}
