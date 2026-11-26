// SPDX-License-Identifier: Apache-2.0
// Adapted from Havunen/gpui-ce, commit 28045891bfe87768a2c8e0aefd43403939844318,
// crates/gpui_apple/src/metal_renderer.rs (whole reference rendering and readback).
// Copyright 2022 - 2025 Zed Industries, Inc.; Havunen and gpui-ce contributors.
// RustUI modifications: async byte oracle, first mismatch coordinates and window disable.
use super::{frame::Mode, retained::Target};
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
            .context("damage pixel readback")?;
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
) -> Result<Retained<ProtocolObject<dyn MTLCommandBuffer>>> {
    anyhow::ensure!(!fail_allocation, "injected verification allocation failure");
    let reference = Target::new(&renderer.device, output.width(), output.height())?
        .context("verification exceeds device retained budget")?;
    let actual = Readback::encode(renderer, primary, output)?;
    let command = renderer.render_frame(scene, &reference.texture, size)?;
    let expected = Readback::encode(renderer, &command, &reference.texture)?;
    let width = output.width();
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
                if let Some((x, y)) = first_mismatch(&actual.pixels(), &expected.pixels(), width) {
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
