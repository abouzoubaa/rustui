// SPDX-License-Identifier: Apache-2.0
//! Associate painter-supplied, immutable content revisions with pixel regions.
//! Metadata follows scene operations. Drawing and batching are unchanged.
use crate::{Bounds, Pixels, ScaledPixels, Scene, Window};
use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq)]
pub struct DamageRegion {
    pub owner: u64,
    pub revision: u64,
    pub bounds: Bounds<ScaledPixels>,
    pub anchor: [u32; 3],
}

impl Window {
    /// Enable view declarations only for an opted-in damage session.
    pub fn declared_damage_enabled() -> bool {
        static ON: OnceLock<bool> = OnceLock::new();
        *ON.get_or_init(|| {
            matches!(
                std::env::var("RUSTUI_DAMAGE").as_deref(),
                Ok("1" | "verify")
            ) && std::env::var_os("RUSTUI_DAMAGE_DECLARED").as_deref() == Some("1".as_ref())
        })
    }

    /// Declare additional changed pixels, including an area that is empty after
    /// an edit. Coordinates are in this window's logical pixels for this frame.
    pub fn declare_damage_rect(&mut self, bounds: Bounds<Pixels>) {
        if Self::declared_damage_enabled() {
            let rect = bounds
                .intersect(&self.content_mask().bounds)
                .scale(self.scale_factor());
            if !rect.is_empty() {
                self.next_frame.scene.fast_damage.extra_damage.push(rect);
            }
        }
    }

    /// Declare the pixel footprint of this paint scope. Give it a new revision
    /// whenever its appearance changes. Zero means each paint can change it.
    /// The renderer damages the previous and current rectangles on change.
    pub fn paint_with_declared_damage<R>(
        &mut self,
        bounds: Bounds<Pixels>,
        origin: crate::Point<Pixels>,
        revision: u64,
        paint: impl FnOnce(&mut Self) -> R,
    ) -> R {
        if !Self::declared_damage_enabled() {
            return paint(self);
        }
        let region = DamageRegion {
            owner: self.current_view().as_u64(),
            revision,
            bounds: bounds.scale(self.scale_factor()),
            anchor: [
                origin.x.0.to_bits(),
                origin.y.0.to_bits(),
                self.element_opacity().to_bits(),
            ],
        };
        let start = self.next_frame.scene.fast_damage.declaration_start();
        let result = paint(self);
        self.next_frame.scene.fast_damage.declare(start, region);
        result
    }
}

impl Scene {
    /// Unmerged painter declarations from this frame, for the pixel verifier.
    #[doc(hidden)]
    pub fn fast_damage_regions_for_reporting(&self) -> &[DamageRegion] {
        &self.fast_damage.regions
    }

    #[doc(hidden)]
    pub fn extra_damage_rects(&self) -> &[Bounds<ScaledPixels>] {
        &self.fast_damage.extra_damage
    }

    #[doc(hidden)]
    pub fn fast_damage_region(&self, kind: usize, index: usize) -> Option<DamageRegion> {
        self.fast_damage.region(kind, index)
    }
    #[doc(hidden)]
    pub fn has_damage_declarations(&self) -> bool {
        !self.fast_damage.regions.is_empty()
    }

    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    pub fn with_test_damage_region(
        &mut self,
        bounds: Bounds<ScaledPixels>,
        revision: u64,
        paint: impl FnOnce(&mut Self),
    ) {
        let start = self.fast_damage.declaration_start();
        paint(self);
        self.fast_damage.declare(
            start,
            DamageRegion {
                owner: 1,
                revision,
                bounds,
                anchor: [0; 3],
            },
        );
    }
}
