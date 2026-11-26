// SPDX-License-Identifier: Apache-2.0
// RustUI memoization around the pinned longbridge/gpui-fast planner signatures.
// Source: 2002081b4a61249a1cef2b8a552f6ba0998d9f52, crates/gpui_windows/src/fast/partial.rs.
use super::Sig;
#[derive(Default)]
pub(crate) struct Cache {
    enabled: bool,
    views: super::views::Views,
    // Child clip scenes need one work vector per nesting level.
    buffers: Vec<Vec<Sig>>,
    previous: collections::HashMap<u64, Sig>,
    next: collections::HashMap<u64, Sig>,
    pub(crate) hashed: usize,
    pub(crate) reused: usize,
}
impl Cache {
    pub(crate) fn enable_from_env(&mut self) {
        static SWITCH: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        self.enabled = *SWITCH.get_or_init(|| {
            std::env::var_os("RUSTUI_G_R2_VIEW_SIGNATURES").as_deref() == Some("1".as_ref())
        });
    }
    pub(crate) fn pooled(&self) -> bool {
        self.enabled
    }
    pub(super) fn take_buffer(&mut self) -> Vec<Sig> {
        if self.enabled {
            self.buffers.pop().unwrap_or_default()
        } else {
            Vec::new()
        }
    }
    pub(crate) fn recycle(&mut self, mut buffer: Vec<Sig>) {
        if self.enabled && buffer.capacity() > 0 {
            buffer.clear();
            self.buffers.push(buffer);
        }
    }
    pub(crate) fn begin(&mut self) {
        if self.enabled {
            self.views.advance();
        }
        self.next.clear();
        self.hashed = 0;
        self.reused = 0;
    }
    pub(crate) fn finish(&mut self) {
        std::mem::swap(&mut self.previous, &mut self.next);
        self.next.clear();
    }
    pub(crate) fn clear(&mut self) {
        if self.enabled {
            self.views.advance();
        }
        self.previous.clear();
        self.next.clear();
        self.hashed = 0;
        self.reused = 0;
    }
    pub(crate) fn replay(
        &mut self,
        source: Option<(u64, bool, u64)>,
        order: u32,
        fresh: bool,
    ) -> Option<Sig> {
        let (id, true, view) = source? else {
            return None;
        };
        let mut signature = if self.enabled {
            self.views.previous(view, id)?
        } else {
            *self.previous.get(&id)?
        };
        signature.order = order;
        signature.fresh = fresh;
        self.remember(view, id, signature);
        self.reused += 1;
        Some(signature)
    }
    fn remember(&mut self, view: u64, id: u64, sig: Sig) {
        if self.enabled {
            self.views.record(view, id, sig);
        } else {
            self.next.insert(id, sig);
        }
    }
    pub(crate) fn push(
        &mut self,
        source: Option<(u64, bool, u64)>,
        signature: Sig,
        out: &mut Vec<Sig>,
    ) {
        self.hashed += 1;
        if let Some((id, _, view)) = source {
            self.remember(view, id, signature);
        }
        out.push(signature);
    }
}

#[cfg(any(test, feature = "test-support"))]
impl Cache {
    pub(crate) fn test_view_invalidation(first: Sig) {
        // Exercise the view-keyed candidate even when this proof is launched
        // without benchmark switches. Both owners use slot 7.
        let view = gpui::EntityId::from((1_u64 << 32) | 7).as_u64();
        let next_owner = gpui::EntityId::from((3_u64 << 32) | 7).as_u64();
        let mut cache = Self {
            enabled: true,
            ..Self::default()
        };
        let mut out = Vec::new();
        cache.begin();
        cache.push(Some((1, false, view)), first, &mut out);
        cache.finish();
        cache.begin();
        let cached = cache
            .replay(Some((1, true, view)), first.order, false)
            .expect("clean view");
        assert_eq!(cached.key, first.key);
        cache.finish();

        cache.begin();
        assert!(
            cache
                .replay(Some((2, false, view)), first.order, false)
                .is_none()
        );
        let edited = Sig {
            key: first.key ^ 1,
            ..first
        };
        cache.push(Some((2, false, view)), edited, &mut out);
        cache.finish();
        cache.begin();
        assert!(
            cache
                .replay(Some((1, true, view)), first.order, false)
                .is_none()
        );
        assert_eq!(
            cache
                .replay(Some((2, true, view)), first.order, false)
                .unwrap()
                .key,
            edited.key
        );
        cache.finish();
        cache.begin();
        assert!(
            cache
                .replay(Some((2, true, next_owner)), first.order, false)
                .is_none()
        );
    }
}
