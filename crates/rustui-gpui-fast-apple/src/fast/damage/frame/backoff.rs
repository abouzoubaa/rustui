// SPDX-License-Identifier: Apache-2.0
//! RustUI original: bounded, opt-in planner back-off per Metal renderer.
use super::Geometry;
use std::sync::OnceLock;
const LARGE_RUN: u8 = 8;
const SKIP_FRAMES: u8 = 30;
#[derive(Default)]
pub(super) struct State {
    geometry: Option<Geometry>,
    streak: u8,
    pub(super) remaining: u8,
    probe: bool,
    #[cfg(any(test, feature = "test-support"))]
    pub(super) enabled_override: Option<bool>,
}
impl State {
    fn enabled(&self) -> bool {
        #[cfg(any(test, feature = "test-support"))]
        if let Some(value) = self.enabled_override {
            return value;
        }
        static ENABLED: OnceLock<bool> = OnceLock::new();
        *ENABLED.get_or_init(|| std::env::var("RUSTUI_DAMAGE_BACKOFF").as_deref() == Ok("1"))
    }
    pub(super) fn reset(&mut self) {
        self.geometry = None;
        self.streak = 0;
        self.remaining = 0;
        self.probe = false;
    }
    pub(super) fn observe(&mut self, geometry: Geometry, atlas_lost: bool) {
        if !self.enabled() {
            return;
        }
        if self.geometry != Some(geometry) || atlas_lost {
            self.reset();
        }
        self.geometry = Some(geometry);
    }
    pub(super) fn skipping(&self) -> bool {
        self.enabled() && self.remaining > 0
    }
    pub(super) fn probing(&self) -> bool {
        self.enabled() && self.probe && self.remaining == 0
    }
    pub(super) fn complete(
        &mut self,
        eligible: bool,
        large_or_direct: bool,
        skipped: bool,
        probe: bool,
    ) {
        if !self.enabled() {
            return;
        }
        if !eligible {
            self.reset();
            return;
        }
        if skipped {
            self.remaining = self.remaining.saturating_sub(1);
            return;
        }
        if !large_or_direct {
            self.streak = 0;
            self.probe = false;
            return;
        }
        self.streak = self.streak.saturating_add(1);
        if probe || self.streak >= LARGE_RUN {
            self.remaining = SKIP_FRAMES;
            self.probe = true;
            self.streak = 0;
        }
    }
}
#[cfg(any(test, feature = "test-support"))]
pub(super) fn test_policy() {
    let geometry = Geometry {
        width: 64,
        height: 64,
        scale: 1f64.to_bits(),
        opaque: true,
    };
    let mut state = State {
        enabled_override: Some(true),
        ..Default::default()
    };
    state.observe(geometry, false);
    for i in 0..8 {
        assert!(!state.skipping());
        state.complete(true, true, false, false);
        assert_eq!(state.remaining, if i == 7 { 30 } else { 0 });
    }
    for remaining in (1..=30).rev() {
        assert_eq!(state.remaining, remaining);
        assert!(state.skipping());
        state.complete(true, true, true, false);
    }
    assert!(state.probing());
    state.complete(true, true, false, true);
    assert_eq!(state.remaining, 30);
    state.observe(geometry, true);
    assert_eq!(state.remaining, 0);
    for _ in 0..8 {
        state.complete(true, true, false, false);
    }
    state.observe(
        Geometry {
            width: 65,
            ..geometry
        },
        false,
    );
    assert_eq!(state.remaining, 0);
    for _ in 0..8 {
        state.complete(true, true, false, false);
    }
    state.observe(
        Geometry {
            scale: 2f64.to_bits(),
            ..geometry
        },
        false,
    );
    assert_eq!(state.remaining, 0);
    for _ in 0..8 {
        state.complete(true, true, false, false);
    }
    state.reset();
    state.observe(geometry, false);
    assert_eq!(state.remaining, 0);
    for _ in 0..7 {
        state.complete(true, true, false, false);
    }
    state.complete(true, false, false, false);
    state.complete(true, true, false, false);
    assert!(!state.skipping());
    state.enabled_override = Some(false);
    for _ in 0..100 {
        state.complete(true, true, false, false);
    }
    assert!(!state.skipping());
}
