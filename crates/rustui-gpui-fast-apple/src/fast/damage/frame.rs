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
use crate::metal_renderer::{InstanceBindings, InstanceBufferWriter, MetalRenderer};
use anyhow::{Context as _, Result};
use block2::RcBlock;
use gpui::{DevicePixels, Scene, Size};
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{MTLCommandBuffer, MTLCommandBufferStatus, MTLCommandQueue, MTLTexture};
use std::{
    cell::Cell,
    ptr::NonNull,
    sync::{Arc, OnceLock, atomic::Ordering},
};

mod backoff;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Config {
    Off,
    On,
    Verify,
}
fn config(value: Option<&str>) -> Config {
    match value {
        Some("1") => Config::On,
        Some("verify") => Config::Verify,
        _ => Config::Off,
    }
}
fn configured() -> Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    *CONFIG.get_or_init(|| config(std::env::var("RUSTUI_DAMAGE").ok().as_deref()))
}

// Only the explicit pixel oracle needs release drawables to be readable.
pub(crate) fn verification_requested() -> bool {
    configured() == Config::Verify
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Mode {
    #[default]
    Direct,
    Retained,
    Copy,
    Same,
    Partial(Area),
}
impl Mode {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Retained => "retained",
            Self::Copy => "copy",
            Self::Same => "skip",
            Self::Partial(_) => "partial",
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
    order_identities: Option<Vec<Option<super::order_probe::Identity>>>,
    generation: u64,
    deferred: Option<Box<Scene>>,
}
#[derive(Default)]
pub(crate) struct State {
    target: Option<Target>,
    snapshot: Option<Snapshot>,
    previous_large: bool,
    backoff: backoff::State,
    #[cfg(any(test, feature = "test-support"))]
    planner_calls: usize,
    cache: plan::Cache,
    pub(crate) failed: Option<Arc<super::verify::Status>>,
    pub(crate) last_mode: Mode,
    #[cfg(any(test, feature = "test-support"))]
    enabled_override: Option<bool>,
    #[cfg(any(test, feature = "test-support"))]
    verify_override: bool,
    #[cfg(any(test, feature = "test-support"))]
    disjoint_order_override: bool,
    #[cfg(any(test, feature = "test-support"))]
    fail_verification: bool,
    verification: Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>>,
    #[cfg(any(test, feature = "test-support"))]
    fail_allocation: bool,
    #[cfg(any(test, feature = "test-support"))]
    magenta_prefill: bool,
    #[cfg(any(test, feature = "test-support"))]
    last_partial_passes: usize,
    #[cfg(any(test, feature = "test-support"))]
    last_magenta_prefills: usize,
}
pub(crate) struct Prepared {
    pub(crate) mode: Mode,
    pub(crate) reason: &'static str,
    snapshot: Option<Snapshot>,
    large: bool,
    plannable: bool,
    backoff: bool,
    probe: bool,
    preserve_snapshot: bool,
}
impl Prepared {
    pub(crate) fn direct() -> Self {
        Self {
            mode: Mode::Direct,
            reason: "disabled",
            snapshot: None,
            large: false,
            plannable: false,
            backoff: false,
            probe: false,
            preserve_snapshot: false,
        }
    }
}
impl State {
    fn enabled(&self) -> bool {
        #[cfg(any(test, feature = "test-support"))]
        if let Some(enabled) = self.enabled_override {
            return enabled;
        }
        configured() != Config::Off
    }
    pub(crate) fn take_verification(
        &mut self,
    ) -> Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>> {
        self.verification.take()
    }
    fn verifying(&self) -> bool {
        #[cfg(any(test, feature = "test-support"))]
        if let Some(enabled) = self.enabled_override {
            return enabled && self.verify_override;
        }
        configured() == Config::Verify
    }
    fn verification_failure(&self) -> bool {
        #[cfg(any(test, feature = "test-support"))]
        {
            self.fail_verification
        }
        #[cfg(not(any(test, feature = "test-support")))]
        {
            false
        }
    }
    pub(crate) fn invalidate(&mut self) {
        self.target = None;
        self.verification = None;
        self.snapshot = None;
        self.previous_large = false;
        self.backoff.reset();
        self.cache.clear();
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
                .is_some_and(|failed| failed.disabled.load(Ordering::Acquire))
            || size.width.0 <= 0
            || size.height.0 <= 0
        {
            let _cpu = super::cpu::Measure::new("off_path");
            let reason = if !self.enabled() {
                "disabled"
            } else if synchronous {
                "live_resize"
            } else if size.width.0 <= 0 || size.height.0 <= 0 {
                "geometry"
            } else {
                "verify_off_path"
            };
            self.invalidate();
            return Prepared {
                reason,
                ..Prepared::direct()
            };
        }
        let mut cpu = super::cpu::Measure::new("off_path");
        if !plan::can_plan(scene) {
            cpu.reason = "unsupported";
            self.invalidate();
            // Unsupported frames predict another whole draw. Keep this history
            // until a comparable frame can demonstrate a cheap follow-up.
            self.previous_large = true;
            return Prepared {
                reason: if !scene.layers.frames.is_empty() {
                    "unsupported_scroll_layer"
                } else if !scene.surfaces.is_empty() {
                    "surface"
                } else {
                    "unsupported_scene"
                },
                large: true,
                ..Prepared::direct()
            };
        }
        super::timing::plan_stage(super::timing::PlanStage::AtlasAndBackoff);
        let geometry = Geometry::new(size, scale, opaque);
        let previous_generation = self
            .snapshot
            .as_ref()
            .filter(|s| s.geometry == geometry)
            .map_or(0, |s| s.generation);
        let uploads = renderer_atlas.damage_uploads(previous_generation);
        self.failed = Some(uploads.status.clone());
        if uploads.status.disabled.load(Ordering::Acquire) {
            cpu.reason = "disabled";
            self.invalidate();
            return Prepared {
                reason: "verify_off_path",
                ..Prepared::direct()
            };
        }
        self.backoff.observe(geometry, uploads.tiles.is_none());
        if self.backoff.skipping() {
            cpu.reason = "backoff";
            super::timing::plan_stage(super::timing::PlanStage::PrimeSnapshot);
            let tail = self.backoff.remaining == 1;
            // Retain metadata without hashing or planning. On the last skipped
            // frame, snapshot that scene for an adjacent-frame recovery probe;
            // comparing with a 30-frame-old scene could falsely restart back-off.
            let snapshot = tail.then(|| Snapshot {
                geometry,
                signatures: Vec::new(),
                order_identities: super::order_probe::active()
                    .then(|| super::order_probe::identities(scene)),
                generation: uploads.generation,
                deferred: Some(Box::new(scene.fast_damage_snapshot())),
            });
            return Prepared {
                reason: "backoff",
                snapshot,
                large: true,
                plannable: true,
                backoff: true,
                preserve_snapshot: !tail,
                ..Prepared::direct()
            };
        }
        let probe = self.backoff.probing();
        let comparable = self
            .snapshot
            .as_ref()
            .is_some_and(|s| s.geometry == geometry)
            && uploads.tiles.is_some();
        // Scene metadata remains comparable after Direct invalidates kept pixels.
        // Pixel validity is checked only when selecting Copy, Skip or Partial.
        if !comparable {
            super::timing::plan_stage(super::timing::PlanStage::PrimeSnapshot);
            cpu.reason = if uploads.tiles.is_none() {
                "atlas_lost"
            } else {
                "prime"
            };
            if self
                .snapshot
                .as_ref()
                .is_none_or(|s| s.geometry != geometry)
            {
                self.target = None;
                if self
                    .snapshot
                    .as_ref()
                    .is_some_and(|s| s.geometry != geometry)
                {
                    self.previous_large = false;
                }
            }
            self.cache.clear();
            return Prepared {
                reason: cpu.reason,
                mode: if self.previous_large {
                    Mode::Direct
                } else {
                    Mode::Retained
                },
                snapshot: Some(Snapshot {
                    geometry,
                    signatures: Vec::new(),
                    order_identities: super::order_probe::active()
                        .then(|| super::order_probe::identities(scene)),
                    generation: uploads.generation,
                    deferred: Some(Box::new(scene.fast_damage_snapshot())),
                }),
                // This is a whole draw, preserving the large-change hysteresis.
                large: true,
                plannable: true,
                probe,
                ..Prepared::direct()
            };
        }
        super::timing::plan_stage(super::timing::PlanStage::SignatureCollection);
        let signature_start = std::time::Instant::now();
        if let Some(deferred) = self.snapshot.as_mut().unwrap().deferred.take() {
            self.cache.begin();
            let empty = Default::default();
            let Some(old) =
                plan::signatures_cached(&deferred, &empty, &inputs::dynamic, &mut self.cache)
            else {
                self.invalidate();
                return Prepared::direct();
            };
            self.snapshot.as_mut().unwrap().signatures = old;
            cpu.hashed += self.cache.hashed;
            cpu.reused += self.cache.reused;
            self.cache.finish();
        }
        self.cache.begin();
        let Some(signatures) = plan::signatures_cached(
            scene,
            uploads.tiles.as_ref().unwrap(),
            &inputs::dynamic,
            &mut self.cache,
        ) else {
            self.invalidate();
            let mut direct = Prepared::direct();
            direct.reason = if scene.layers.frames.iter().any(|f| f.tile_size != 0) {
                "unsupported_scroll_layer"
            } else if !scene.surfaces.is_empty() {
                "surface"
            } else {
                "unsupported_scene"
            };
            return direct;
        };
        cpu.hashed += self.cache.hashed;
        cpu.reused += self.cache.reused;
        self.cache.finish();
        cpu.signatures_ms = signature_start.elapsed().as_secs_f64() * 1000.;
        let plan_start = std::time::Instant::now();
        #[cfg(any(test, feature = "test-support"))]
        {
            self.planner_calls += 1;
        }
        let mut trace = super::diagnosis::Trace::start();
        let mut policy = plan::Policy::metal();
        #[cfg(any(test, feature = "test-support"))]
        if self.disjoint_order_override {
            policy.allow_disjoint_order = true;
        }
        let change = plan::plan_traced(
            &self.snapshot.as_ref().unwrap().signatures,
            &signatures,
            geometry.view(),
            trace.as_mut(),
            &mut policy,
        );
        cpu.plan_ms = plan_start.elapsed().as_secs_f64() * 1000.;
        super::timing::plan_stage(super::timing::PlanStage::SelectMode);
        let order_identities =
            super::order_probe::active().then(|| super::order_probe::identities(scene));
        if let Some(witness) = policy.witness
            && let Some(old) = self
                .snapshot
                .as_ref()
                .and_then(|s| s.order_identities.as_deref())
            && let Some(new) = order_identities.as_deref()
        {
            super::order_probe::emit(witness, old, new);
        }
        let damage_box = change
            .as_ref()
            .and_then(|p| p.damage.iter().copied().reduce(|a, b| a.union(&b)))
            .map(|r| r.pixels().and(&geometry.view()));
        let large = change.as_ref().is_none_or(|p| p.shift.is_some())
            || damage_box.is_some_and(|r| r.area() > plan::MAX_DAMAGE * geometry.view().area());
        let valid = self.target.as_ref().is_some_and(|t| t.valid);
        let rejection = policy.reason.or_else(|| {
            trace.as_ref().and_then(|_| match change.as_ref() {
                Some(plan) if plan.shift.is_some() => Some("shift_unimplemented"),
                Some(_) if large => Some("area_bounding_box"),
                None => Some("no_plan"),
                _ => None,
            })
        });
        let mode = match change {
            Some(p)
                if p.shift.is_none()
                    && p.damage.is_empty()
                    && !required
                    && valid
                    && !self.verifying() =>
            {
                Mode::Same
            }
            Some(p) if p.shift.is_none() && p.damage.is_empty() && valid => Mode::Copy,
            Some(p) if !large && p.shift.is_none() && valid && damage_box.is_some() => {
                Mode::Partial(damage_box.unwrap())
            }
            // Large changes never pay for a full draw plus a full copy. After
            // one, require two consecutive cheap plans before priming again:
            // this Direct records large=false; the next cheap frame may prime.
            _ if large || (self.previous_large && !probe) => Mode::Direct,
            _ => Mode::Retained,
        };
        cpu.reason = mode.label();
        if let Some(mut trace) = trace {
            if let Some(rejection) = rejection.filter(|_| trace.reason == "planned") {
                trace.reason = rejection;
            } else if !large && matches!(mode, Mode::Direct) {
                trace.reason = "large_hysteresis";
            }
            trace.emit(mode.label(), geometry.view(), damage_box);
        }
        Prepared {
            mode,
            reason: match mode {
                Mode::Same => "identical",
                Mode::Copy => "required",
                Mode::Partial(_) => "changed",
                Mode::Retained => "prime",
                _ => policy.reason.unwrap_or("large"),
            },
            snapshot: Some(Snapshot {
                geometry,
                signatures,
                order_identities,
                generation: uploads.generation,
                deferred: None,
            }),
            large,
            plannable: true,
            probe,
            ..Prepared::direct()
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
        self.backoff.complete(
            prepared.plannable,
            prepared.large || mode == Mode::Direct,
            prepared.backoff,
            prepared.probe,
        );
        if !prepared.preserve_snapshot {
            self.snapshot = prepared.snapshot;
        }
        self.previous_large = prepared.large;
        self.last_mode = mode;
    }
}
pub(crate) fn choose(renderer: &mut MetalRenderer, scene: &Scene) -> Prepared {
    if !renderer.damage.enabled() {
        let _cpu = super::cpu::Measure::new("off");
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
#[cfg(any(test, feature = "test-support"))]
pub(crate) fn render(
    renderer: &mut MetalRenderer,
    scene: &Scene,
    target: &ProtocolObject<dyn MTLTexture>,
    size: Size<DevicePixels>,
    prepared: Prepared,
) -> Result<Retained<ProtocolObject<dyn MTLCommandBuffer>>> {
    render_with_instances(
        renderer,
        scene,
        target,
        size,
        prepared,
        None,
        super::encode::Scope::new(scene),
    )
}

pub(crate) fn render_with_instances(
    renderer: &mut MetalRenderer,
    scene: &Scene,
    target: &ProtocolObject<dyn MTLTexture>,
    size: Size<DevicePixels>,
    mut prepared: Prepared,
    staged: Option<(InstanceBufferWriter, InstanceBindings)>,
    encoding: super::encode::Scope,
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
        let mut command = match mode {
            Mode::Direct => {
                if let Some(kept) = &mut state.target {
                    kept.valid = false;
                }
                if let Some((writer, bindings)) = staged {
                    renderer.encode_frame(scene, target, size, writer, bindings)?
                } else {
                    renderer.render_frame(scene, target, size)?
                }
            }
            Mode::Retained | Mode::Copy | Mode::Partial(_) => {
                let kept = state.target.as_mut().context("missing retained texture")?;
                let command = if mode == Mode::Copy {
                    anyhow::ensure!(kept.valid, "copy of invalid retained pixels");
                    renderer
                        .command_queue
                        .commandBuffer()
                        .context("creating copy command buffer")?
                } else if let Mode::Partial(area) = mode {
                    anyhow::ensure!(kept.valid, "partial over invalid retained pixels");
                    let magenta = {
                        #[cfg(any(test, feature = "test-support"))]
                        {
                            state.magenta_prefill
                        }
                        #[cfg(not(any(test, feature = "test-support")))]
                        {
                            false
                        }
                    };
                    let scope = super::partial::Scope::new(
                        kept.texture.clone(),
                        area,
                        renderer.damage_pipelines.clone(),
                        magenta,
                    )?;
                    let command = renderer.render_frame(scene, &kept.texture, size)?;
                    let stats = scope.finish();
                    anyhow::ensure!(
                        stats.encoders > 0,
                        "partial target never reached its encoder hook"
                    );
                    #[cfg(any(test, feature = "test-support"))]
                    {
                        state.last_partial_passes = stats.encoders;
                        state.last_magenta_prefills = stats.prefills;
                    }
                    command
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
                        failed.disabled.store(true, Ordering::Release);
                    }
                    let _ = resources.take();
                },
            );
            // SAFETY: Metal copies this block and owns it until completion.
            unsafe {
                command.addCompletedHandler(RcBlock::as_ptr(&block));
            }
        }
        if state.verifying()
            && state
                .failed
                .as_ref()
                .is_some_and(|s| !s.disabled.load(Ordering::Acquire))
        {
            let status = state
                .failed
                .as_ref()
                .context("verify window status")?
                .clone();
            match super::verify::schedule(
                renderer,
                scene,
                &command,
                target,
                size,
                mode,
                status.clone(),
                state.verification_failure(),
            ) {
                Ok(reference) => state.verification = Some(reference),
                Err(error) => {
                    log::error!("damage verification unavailable; drawing whole: {error:#}");
                    status.disabled.store(true, Ordering::Release);
                    // Preserve uploads and GPU lifetimes already encoded in the
                    // first buffer. It never presents; a full draw follows it.
                    command.commit();
                    state.invalidate();
                    command = renderer.render_frame(scene, target, size)?;
                    mode = Mode::Direct;
                }
            }
        }
        let reason = if prepared.backoff {
            "backoff"
        } else {
            mode.label()
        };
        state.complete(prepared, mode);
        encoding.finish(mode.label(), reason);
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

pub(crate) use super::partial::{pass_clear, pass_scissor};
