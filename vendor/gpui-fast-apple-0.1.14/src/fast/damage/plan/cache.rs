// SPDX-License-Identifier: Apache-2.0
// RustUI memoization around the pinned longbridge/gpui-fast planner signatures.
// Source: 2002081b4a61249a1cef2b8a552f6ba0998d9f52, crates/gpui_windows/src/fast/partial.rs.
use super::Sig;
#[derive(Default)]
pub(crate) struct Cache {
    previous: collections::HashMap<u64, Sig>,
    next: collections::HashMap<u64, Sig>,
    pub(crate) hashed: usize,
    pub(crate) reused: usize,
}
impl Cache {
    pub(crate) fn begin(&mut self) {
        self.next.clear();
        self.hashed = 0;
        self.reused = 0;
    }
    pub(crate) fn finish(&mut self) {
        std::mem::swap(&mut self.previous, &mut self.next);
        self.next.clear();
    }
    pub(crate) fn clear(&mut self) {
        self.previous.clear();
        self.next.clear();
        self.hashed = 0;
        self.reused = 0;
    }
    pub(crate) fn replay(
        &mut self,
        source: Option<(u64, bool)>,
        order: u32,
        fresh: bool,
    ) -> Option<Sig> {
        let (id, true) = source? else { return None };
        let mut signature = *self.previous.get(&id)?;
        signature.order = order;
        signature.fresh = fresh;
        self.next.insert(id, signature);
        self.reused += 1;
        Some(signature)
    }
    pub(crate) fn push(&mut self, source: Option<(u64, bool)>, signature: Sig, out: &mut Vec<Sig>) {
        self.hashed += 1;
        if let Some((id, _)) = source {
            self.next.insert(id, signature);
        }
        out.push(signature);
    }
}
