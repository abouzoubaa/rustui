// SPDX-License-Identifier: Apache-2.0
//! Signatures of retained views. A replayed paint operation carries its
//! original unique source ID. A repaint gets a new ID even at the same slot.
use super::Sig;
use smallvec::SmallVec;

// Median of 1,749 view observations (1, 5, 883 or 884 signatures).
// A large view retains its spill on both sides of the frame swap.
const INLINE: usize = 5;
type List = SmallVec<[(u64, Sig); INLINE]>;

#[derive(Default)]
struct Entry {
    view: u64,
    frame: [u64; 2],
    lists: [List; 2],
    cursor: usize,
}

#[derive(Default)]
pub(super) struct Views {
    // SlotMap slots may be sparse; only views with signatures own an Entry.
    slots: Vec<Option<Box<Entry>>>,
    frame: u64,
    touched: [Vec<usize>; 2],
}

impl Views {
    pub(super) fn allocated_bytes(&self) -> usize {
        use std::mem::size_of;
        size_of::<Self>()
            + self.slots.capacity() * size_of::<Option<Box<Entry>>>()
            + self
                .touched
                .iter()
                .map(|v| v.capacity() * size_of::<usize>())
                .sum::<usize>()
            + self
                .slots
                .iter()
                .flatten()
                .map(|e| {
                    size_of::<Entry>()
                        + e.lists
                            .iter()
                            .filter(|list| list.spilled())
                            .map(|list| list.capacity() * size_of::<(u64, Sig)>())
                            .sum::<usize>()
                })
                .sum::<usize>()
    }

    pub(super) fn advance(&mut self) {
        self.frame = self
            .frame
            .checked_add(1)
            .expect("view signature frame overflow");
        let now = (self.frame & 1) as usize;
        for slot in self.touched[now].drain(..) {
            if self.slots[slot]
                .as_ref()
                .is_some_and(|e| e.frame[now ^ 1] != self.frame - 1)
            {
                self.slots[slot] = None;
            }
        }
    }

    fn entry(&mut self, view: u64) -> &mut Entry {
        // The low half of KeyData::as_ffi() is the slot, the high half its
        // generation. Check the full key before accepting either list.
        let slot = view as u32 as usize;
        if slot >= self.slots.len() {
            self.slots.resize_with(slot + 1, || None);
        }
        let entry = self.slots[slot].get_or_insert_with(Default::default);
        if entry.view != view {
            entry.view = view;
            entry.frame = [0; 2];
            for list in &mut entry.lists {
                list.clear();
            }
        }
        let now = (self.frame & 1) as usize;
        if entry.frame[now] != self.frame {
            self.touched[now].push(slot);
            entry.frame[now] = self.frame;
            entry.lists[now].clear();
            entry.cursor = 0;
        }
        entry
    }

    pub(super) fn previous(&mut self, view: u64, id: u64) -> Option<Sig> {
        let frame = self.frame;
        let old = ((frame & 1) as usize) ^ 1;
        let entry = self.entry(view);
        if frame <= 1 || entry.frame[old] != frame - 1 {
            return None;
        }
        let list = &entry.lists[old];
        let i = if list.get(entry.cursor).is_some_and(|record| record.0 == id) {
            entry.cursor
        } else {
            // GPUI can splice a clean child into a changed parent, or reorder
            // sorted primitive batches. Validate the exact source in both cases.
            list.iter().position(|record| record.0 == id)?
        };
        entry.cursor = i + 1;
        Some(list[i].1)
    }

    pub(super) fn record(&mut self, view: u64, id: u64, sig: Sig) {
        let now = (self.frame & 1) as usize;
        self.entry(view).lists[now].push((id, sig));
    }
}
