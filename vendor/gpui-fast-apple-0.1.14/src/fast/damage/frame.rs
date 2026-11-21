// SPDX-License-Identifier: Apache-2.0
// Adapted from Havunen/gpui-ce, commit 28045891bfe87768a2c8e0aefd43403939844318,
// crates/gpui_apple/src/metal_renderer.rs (frame_mode, draw_frame, retained_texture).
// Planner source: longbridge/gpui-fast, commit 2002081b4a61249a1cef2b8a552f6ba0998d9f52,
// crates/gpui_windows/src/fast/partial.rs.
// Copyright 2022 - 2025 Zed Industries, Inc.; Havunen, gpui-ce and Longbridge contributors.
// RustUI modifications: opt-in modes, pre-drawable skip, atlas observations and GPU lifetime.
use super::{
    inputs,
    plan::{self, Area, Sig},
    retained::{self, Target},
};
use crate::metal_renderer::MetalRenderer;
use anyhow::{Context as _, Result};
use block2::RcBlock;
use gpui::{DevicePixels, Scene, Size};
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{MTLCommandBuffer, MTLCommandBufferStatus, MTLCommandQueue, MTLTexture};
use std::{
    cell::Cell,
    ptr::NonNull,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Mode {
    #[default]
    Direct,
    Retained,
    Copy,
    Same,
}
impl Mode {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Retained => "retained",
            Self::Copy => "copy",
            Self::Same => "skip",
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct Geometry {
    width: i32,
    height: i32,
    scale: u64,
    opaque: bool,
}
impl Geometry {
    fn new(size: Size<DevicePixels>, scale: f64, opaque: bool) -> Self {
        Self {
            width: size.width.0,
            height: size.height.0,
            scale: scale.to_bits(),
            opaque,
        }
    }
    fn view(self) -> Area {
        Area {
            x0: 0.,
            y0: 0.,
            x1: self.width as f32,
            y1: self.height as f32,
        }
    }
}
struct Snapshot {
    geometry: Geometry,
    signatures: Vec<Sig>,
    generation: u64,
}
#[derive(Default)]
pub(crate) struct State {
    target: Option<Target>,
    snapshot: Option<Snapshot>,
    previous_large: bool,
    pub(crate) failed: Option<Arc<AtomicBool>>,
    pub(crate) last_mode: Mode,
    #[cfg(any(test, feature = "test-support"))]
    enabled_override: Option<bool>,
    #[cfg(any(test, feature = "test-support"))]
    fail_allocation: bool,
}
pub(crate) struct Prepared {
    pub(crate) mode: Mode,
    snapshot: Option<Snapshot>,
    large: bool,
}
impl Prepared {
    pub(crate) fn direct() -> Self {
        Self {
            mode: Mode::Direct,
            snapshot: None,
            large: false,
        }
    }
}
impl State {
    fn enabled(&self) -> bool {
        #[cfg(any(test, feature = "test-support"))]
        if let Some(enabled) = self.enabled_override {
            return enabled;
        }
        static ENABLED: OnceLock<bool> = OnceLock::new();
        *ENABLED.get_or_init(|| std::env::var("RUSTUI_DAMAGE").as_deref() == Ok("1"))
    }
    pub(crate) fn invalidate(&mut self) {
        self.target = None;
        self.snapshot = None;
        self.previous_large = false;
    }
    pub(crate) fn resized(&mut self, size: Size<DevicePixels>) {
        if self
            .snapshot
            .as_ref()
            .is_some_and(|s| (s.geometry.width, s.geometry.height) != (size.width.0, size.height.0))
        {
            self.invalidate();
        }
    }
    pub(crate) fn choose(
        &mut self,
        renderer_atlas: &crate::metal_atlas::MetalAtlas,
        scene: &Scene,
        size: Size<DevicePixels>,
        scale: f64,
        opaque: bool,
        synchronous: bool,
        required: bool,
    ) -> Prepared {
        if !self.enabled()
            || synchronous
            || self
                .failed
                .as_ref()
                .is_some_and(|failed| failed.load(Ordering::Acquire))
            || size.width.0 <= 0
            || size.height.0 <= 0
        {
            self.invalidate();
            return Prepared::direct();
        }
        let geometry = Geometry::new(size, scale, opaque);
        let old = self.snapshot.as_ref().filter(|s| s.geometry == geometry);
        let uploads = renderer_atlas.damage_uploads(old.map_or(0, |s| s.generation));
        // An expired atlas journal requires a whole draw; signatures use an empty
        // upload set only to establish a fresh baseline for the following frame.
        let empty = Default::default();
        let Some(signatures) = plan::signatures(
            scene,
            uploads.tiles.as_ref().unwrap_or(&empty),
            &inputs::dynamic,
        ) else {
            self.invalidate();
            return Prepared::direct();
        };
        let change = old.and_then(|old| {
            uploads
                .tiles
                .as_ref()
                .and_then(|_| plan::plan(&old.signatures, &signatures, geometry.view()))
        });
        let large = change.as_ref().is_none_or(|p| p.shift.is_some());
        if old.is_none() {
            self.target = None;
        }
        let valid = self.target.as_ref().is_some_and(|t| t.valid);
        let mode = match change {
            Some(p) if p.shift.is_none() && p.damage.is_empty() && !required && valid => Mode::Same,
            Some(p) if p.shift.is_none() && p.damage.is_empty() && valid => Mode::Copy,
            _ if large && self.previous_large && old.is_some() => Mode::Direct,
            _ => Mode::Retained,
        };
        Prepared {
            mode,
            snapshot: Some(Snapshot {
                geometry,
                signatures,
                generation: uploads.generation,
            }),
            large,
        }
    }
    fn allocate(
        &self,
        renderer: &MetalRenderer,
        size: Size<DevicePixels>,
    ) -> Result<Option<Target>> {
        #[cfg(any(test, feature = "test-support"))]
        anyhow::ensure!(
            !self.fail_allocation,
            "injected retained texture allocation failure"
        );
        Target::new(
            &renderer.device,
            size.width.0 as usize,
            size.height.0 as usize,
        )
    }
    fn complete(&mut self, prepared: Prepared, mode: Mode) {
        self.snapshot = prepared.snapshot;
        self.previous_large = prepared.large;
        self.last_mode = mode;
    }
}
pub(crate) fn choose(renderer: &mut MetalRenderer, scene: &Scene) -> Prepared {
    if !renderer.damage.enabled() {
        renderer.damage.invalidate();
        return Prepared::direct();
    }
    let Some(layer) = renderer.layer.as_ref() else {
        return Prepared::direct();
    };
    let size = layer.drawableSize();
    let size = gpui::size(
        DevicePixels(size.width.ceil() as i32),
        DevicePixels(size.height.ceil() as i32),
    );
    renderer.damage.choose(
        &renderer.sprite_atlas,
        scene,
        size,
        layer.contentsScale(),
        renderer.opaque,
        renderer.presents_with_transaction,
        gpui::fast_present_required(),
    )
}
pub(crate) fn skip(renderer: &mut MetalRenderer, prepared: Prepared) {
    renderer.damage.complete(prepared, Mode::Same);
    super::metrics::skipped();
}
pub(crate) fn render(
    renderer: &mut MetalRenderer,
    scene: &Scene,
    target: &ProtocolObject<dyn MTLTexture>,
    size: Size<DevicePixels>,
    mut prepared: Prepared,
) -> Result<Retained<ProtocolObject<dyn MTLCommandBuffer>>> {
    let mut state = std::mem::take(&mut renderer.damage);
    let result = (|| {
        if prepared
            .snapshot
            .as_ref()
            .is_some_and(|s| (s.geometry.width, s.geometry.height) != (size.width.0, size.height.0))
        {
            // A display-link drawable can predate a layer resize. Never reuse or
            // copy a differently sized retained target into it.
            state.invalidate();
            prepared = Prepared::direct();
        }
        let mut mode = prepared.mode;
        if mode != Mode::Direct && state.target.is_none() {
            state.target = state.allocate(renderer, size).unwrap_or_else(|error| {
                log::warn!("retained damage allocation failed; drawing whole: {error:#}");
                None
            });
            if state.target.is_none() {
                mode = Mode::Direct;
            }
        }
        let command = match mode {
            Mode::Direct => {
                if let Some(kept) = &mut state.target {
                    kept.valid = false;
                }
                renderer.render_frame(scene, target, size)?
            }
            Mode::Retained | Mode::Copy => {
                let kept = state.target.as_mut().context("missing retained texture")?;
                let command = if mode == Mode::Copy {
                    anyhow::ensure!(kept.valid, "copy of invalid retained pixels");
                    renderer
                        .command_queue
                        .commandBuffer()
                        .context("creating copy command buffer")?
                } else {
                    renderer.render_frame(scene, &kept.texture, size)?
                };
                renderer
                    .damage_pipelines
                    .copy(&command, &kept.texture, target, size)?;
                kept.valid = true;
                command
            }
            Mode::Same => anyhow::bail!("skipped frame reached drawable rendering"),
        };
        if state.enabled() {
            let failed = state.failed.get_or_insert_with(Default::default).clone();
            let resources = Cell::new(
                state
                    .target
                    .as_ref()
                    .map(|t| (t.texture.clone(), t.reservation.clone())),
            );
            let block = RcBlock::new(
                move |buffer: NonNull<ProtocolObject<dyn MTLCommandBuffer>>| {
                    // SAFETY: Metal supplies the live completed command buffer.
                    if unsafe { buffer.as_ref() }.status() != MTLCommandBufferStatus::Completed {
                        failed.store(true, Ordering::Release);
                    }
                    let _ = resources.take();
                },
            );
            // SAFETY: Metal copies this block and owns it until completion.
            unsafe {
                command.addCompletedHandler(RcBlock::as_ptr(&block));
            }
        }
        state.complete(prepared, mode);
        Ok(command)
    })();
    if result.is_err() {
        state.invalidate();
    }
    renderer.damage = state;
    result
}

#[cfg(any(test, feature = "test-support"))]
mod proof;
#[cfg(test)]
pub(crate) use proof::test_skip_leaves_drawable_unused;
