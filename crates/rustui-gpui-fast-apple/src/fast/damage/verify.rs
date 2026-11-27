// SPDX-License-Identifier: Apache-2.0
// Adapted from Havunen/gpui-ce, commit 28045891bfe87768a2c8e0aefd43403939844318,
// crates/gpui_apple/src/metal_renderer.rs (whole reference rendering and readback).
// Copyright 2022 - 2025 Zed Industries, Inc.; Havunen and gpui-ce contributors.
// RustUI modifications: async byte oracle, first mismatch coordinates and window disable.
use super::{frame::Mode, retained};
use crate::metal_renderer::MetalRenderer;
use anyhow::{Context as _, Result};
use block2::RcBlock;
use gpui::{DevicePixels, Scene, Size};
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{
    MTLBlitCommandEncoder, MTLBuffer, MTLCommandBuffer, MTLCommandBufferStatus, MTLCommandEncoder,
    MTLDevice, MTLOrigin, MTLResourceOptions, MTLSize, MTLTexture,
};
use std::{
    cell::Cell,
    ptr::NonNull,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Mismatch {
    pub(crate) mode: &'static str,
    pub(crate) x: usize,
    pub(crate) y: usize,
}
#[derive(Default)]
pub(crate) struct Status {
    pub(crate) disabled: AtomicBool,
    first: Mutex<Option<Mismatch>>,
}
impl Status {
    pub(crate) fn record(&self) -> Option<Mismatch> {
        *self.first.lock().unwrap_or_else(|e| e.into_inner())
    }
    fn mismatch(&self, mode: &'static str, x: usize, y: usize) {
        if self
            .disabled
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            *self.first.lock().unwrap_or_else(|e| e.into_inner()) = Some(Mismatch { mode, x, y });
            super::metrics::mismatch(mode, x, y);
        }
    }
}
pub(crate) fn first_mismatch(a: &[u8], b: &[u8], width: usize) -> Option<(usize, usize)> {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b)
        .position(|(a, b)| a != b)
        .map(|index| (index / 4 % width, index / 4 / width))
}
#[derive(Clone)]
pub(crate) struct Readback {
    buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    width: usize,
    height: usize,
    row: usize,
}
impl Readback {
    pub(crate) fn encode(
        renderer: &MetalRenderer,
        command: &ProtocolObject<dyn MTLCommandBuffer>,
        texture: &ProtocolObject<dyn MTLTexture>,
    ) -> Result<Self> {
        let (width, height) = (texture.width(), texture.height());
        let row = (width * 4).div_ceil(256) * 256;
        let buffer = renderer
            .device
            .newBufferWithLength_options(row * height, MTLResourceOptions::StorageModeShared)
            .ok_or_else(|| super::budget::Failure {
                reason: super::budget::Reason::AllocFailed,
                requested: (row * height) as u64,
                limit: u64::MAX,
            })?;
        let encoder = command.blitCommandEncoder().context("damage pixel blit")?;
        // SAFETY: Both the exact source rectangle and aligned destination span
        // are in bounds; Metal retains both resources through completion.
        unsafe {
            encoder.copyFromTexture_sourceSlice_sourceLevel_sourceOrigin_sourceSize_toBuffer_destinationOffset_destinationBytesPerRow_destinationBytesPerImage(texture,0,0,MTLOrigin{x:0,y:0,z:0},MTLSize{width,height,depth:1},&buffer,0,row,row*height);
        }
        encoder.endEncoding();
        Ok(Self {
            buffer,
            width,
            height,
            row,
        })
    }
    pub(crate) fn pixels(&self) -> Vec<u8> {
        // SAFETY: The caller has waited for the command that wrote this shared
        // buffer. Only initialized pixel bytes (not row padding) are returned.
        let bytes = unsafe {
            std::slice::from_raw_parts(
                self.buffer.contents().as_ptr().cast::<u8>(),
                self.row * self.height,
            )
        };
        bytes
            .chunks_exact(self.row)
            .flat_map(|row| row[..self.width * 4].iter().copied())
            .collect()
    }
}

pub(crate) struct History {
    scrolls: Vec<gpui::FastScrollViewport>,
    declarations: Vec<super::area::Decl>,
    reference: Readback,
    signatures: Vec<super::plan::Sig>,
    generation: u64,
}
fn changed_outside(
    old: &[u8],
    new: &[u8],
    width: usize,
    areas: &[super::plan::Area],
) -> Option<(usize, usize)> {
    old.chunks_exact(4)
        .zip(new.chunks_exact(4))
        .enumerate()
        .find_map(|(i, (a, b))| {
            if a == b {
                return None;
            }
            let (x, y) = (i % width, i / width);
            let inside = areas.iter().any(|r| {
                x as f32 >= r.x0 && (x as f32) < r.x1 && y as f32 >= r.y0 && (y as f32) < r.y1
            });
            (!inside).then_some((x, y))
        })
}
/// Encode the reference without queueing it. The renderer submits the reference
/// after the primary passes the surface order guard.
pub(crate) fn schedule(
    renderer: &mut MetalRenderer,
    scene: &Scene,
    primary: &Retained<ProtocolObject<dyn MTLCommandBuffer>>,
    output: &ProtocolObject<dyn MTLTexture>,
    size: Size<DevicePixels>,
    mode: Mode,
    status: Arc<Status>,
    fail_allocation: bool,
    history: &mut Option<History>,
    allowed: Option<Vec<super::plan::Area>>,
    shift: Option<(
        Vec<(super::plan::Area, super::plan::Area)>,
        Vec<super::plan::Area>,
    )>,
) -> Result<Retained<ProtocolObject<dyn MTLCommandBuffer>>> {
    let requested = super::budget::bytes(output.width(), output.height());
    let failure = super::budget::Failure {
        reason: super::budget::Reason::AllocFailed,
        requested,
        limit: u64::MAX,
    };
    if fail_allocation {
        return Err(failure.into());
    }
    // A verify-only reference is never a product retained texture.
    let reference = retained::texture(&renderer.device, output.width(), output.height())
        .map_err(|_| failure)?;
    let actual = Readback::encode(renderer, primary, output)?;
    let command = renderer.render_frame(scene, &reference, size)?;
    let expected = Readback::encode(renderer, &command, &reference)?;
    let width = output.width();
    let previous = history.take();
    let uploads = renderer
        .sprite_atlas
        .damage_uploads(previous.as_ref().map_or(0, |h| h.generation));
    let mut prior = None;
    let mut shifted_prior = None;
    let declarations = super::area::snapshot(scene);
    let height = output.height();
    if let Some(tiles) = uploads.tiles {
        if let Some(full) = super::plan::signatures(scene, &tiles, &super::inputs::dynamic) {
            if let Some(old) = previous
                && old.reference.width == width
                && old.reference.height == output.height()
            {
                if let Some(shift) = shift {
                    let whole = super::plan::Area {
                        x0: 0.,
                        y0: 0.,
                        x1: width as f32,
                        y1: height as f32,
                    };
                    let diff = super::plan::scroll::plan(
                        &old.signatures,
                        &full,
                        &old.scrolls,
                        scene.fast_scroll_viewports(),
                        whole,
                        &mut Default::default(),
                    );
                    if let super::plan::scroll::Decision::Planned(full_plan) = diff {
                        super::metrics::declared_diff(full_plan.damage.len());
                    }
                    shifted_prior = Some((old.reference.clone(), shift));
                }
                if let Some(allowed) = allowed {
                    let whole = super::plan::Area {
                        x0: 0.,
                        y0: 0.,
                        x1: width as f32,
                        y1: output.height() as f32,
                    };
                    let diff = super::plan::plan_traced(
                        &old.signatures,
                        &full,
                        whole,
                        None,
                        &mut super::plan::Policy::default(),
                    );
                    // Use the original full diff as an independent diagnostic;
                    // readback is definitive even when it declines the frame.
                    if let Some(diff) = diff {
                        super::metrics::declared_diff(diff.damage.len());
                    }
                    let area = super::area::enabled().then(|| {
                        super::area::declared(
                            &old.declarations,
                            &declarations,
                            scene,
                            width,
                            height,
                        )
                    });
                    prior = Some((old.reference, allowed, area));
                }
            }
            *history = Some(History {
                scrolls: scene.fast_scroll_viewports().to_vec(),
                declarations,
                reference: expected.clone(),
                signatures: full,
                generation: uploads.generation,
            });
        }
    }
    let lifetime = Cell::new(Some(reference));
    let submitted = primary.clone();
    let callback = RcBlock::new(
        move |buffer: NonNull<ProtocolObject<dyn MTLCommandBuffer>>| {
            // SAFETY: Metal supplies this completed reference buffer. Its queue
            // position ensures the primary buffer and both readbacks precede it.
            let buffer = unsafe { buffer.as_ref() };
            if buffer.status() != MTLCommandBufferStatus::Completed
                || submitted.status() != MTLCommandBufferStatus::Completed
            {
                status.disabled.store(true, Ordering::Release);
                log::error!("damage reference GPU failure: {:?}", buffer.error());
            } else if !status.disabled.load(Ordering::Acquire) {
                let reference = expected.pixels();
                if let Some((x, y)) = shifted_prior.as_ref().and_then(|(old, (copies, areas))| {
                    shifted_outside(&old.pixels(), &reference, width, copies, areas)
                }) {
                    status.mismatch("scroll_declared_outside", x, y);
                } else if let Some((x, y)) =
                    prior.as_ref().and_then(|(old, permitted, declared)| {
                        let old = old.pixels();
                        if let Some(area) = declared
                            && let Mode::Partial(scissor) = mode
                        {
                            super::area::emit(
                                &old,
                                &reference,
                                *area,
                                scissor.area() as usize,
                                width,
                                height,
                            );
                        }
                        changed_outside(&old, &reference, width, permitted)
                    })
                {
                    status.mismatch("declared_outside", x, y);
                } else if let Some((x, y)) = first_mismatch(&actual.pixels(), &reference, width) {
                    status.mismatch(mode.label(), x, y);
                } else {
                    super::metrics::verified(mode.label());
                }
            }
            let _ = lifetime.take();
        },
    );
    // SAFETY: Metal copies the live block and invokes it once on completion.
    unsafe {
        command.addCompletedHandler(RcBlock::as_ptr(&callback));
    }
    Ok(command)
}

fn shifted_outside(
    old: &[u8],
    new: &[u8],
    width: usize,
    copies: &[(super::plan::Area, super::plan::Area)],
    repairs: &[super::plan::Area],
) -> Option<(usize, usize)> {
    for &(src, dst) in copies {
        let mut pieces = vec![dst];
        for repair in repairs {
            let mut remaining = Vec::new();
            for r in pieces {
                super::plan::scroll::subtract(r, *repair, &mut remaining);
            }
            pieces = remaining;
        }
        for r in pieces {
            for y in r.y0 as usize..r.y1 as usize {
                let x = r.x0 as usize;
                let start = (width * y + x) * 4;
                let prior = ((y as i32 + (src.y0 - dst.y0) as i32) as usize * width
                    + (x as i32 + (src.x0 - dst.x0) as i32) as usize)
                    * 4;
                let len = (r.x1 as usize - x) * 4;
                let (a, b) = (&old[prior..prior + len], &new[start..start + len]);
                if a != b {
                    let byte = a.iter().zip(b).position(|(l, r)| l != r).unwrap();
                    return Some((x + byte / 4, y));
                }
            }
        }
    }
    None
}
