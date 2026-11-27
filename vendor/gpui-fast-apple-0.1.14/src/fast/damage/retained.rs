// SPDX-License-Identifier: Apache-2.0
// Adapted from Havunen/gpui-ce, commit 28045891bfe87768a2c8e0aefd43403939844318,
// crates/gpui_render/src/retained.rs and crates/gpui_apple/src/metal_renderer.rs.
// Copyright 2022 - 2025 Zed Industries, Inc.; Havunen and gpui-ce contributors.
// RustUI modifications: objc2-metal targets and reservations held through GPU completion.
use super::budget::{self, Failure, Reservation, WindowBudget};
use anyhow::{Context as _, Result};
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{
    MTLDevice, MTLPixelFormat, MTLResource, MTLStorageMode, MTLTexture, MTLTextureDescriptor,
    MTLTextureUsage,
};
use std::sync::Arc;

pub(crate) struct Target {
    pub(crate) texture: Retained<ProtocolObject<dyn MTLTexture>>,
    pub(crate) reservation: Reservation,
    pub(crate) valid: bool,
}
impl Target {
    pub(crate) fn new(
        device: &ProtocolObject<dyn MTLDevice>,
        budget: &Arc<WindowBudget>,
        width: usize,
        height: usize,
    ) -> Result<Self, Failure> {
        let logical = budget::bytes(width, height);
        let descriptor = descriptor(width, height);
        let footprint = device.heapTextureSizeAndAlignWithDescriptor(&descriptor);
        // Metal can tile or align small drawables well above width × height × 4.
        // Standalone private textures occupy whole VM pages, whereas the heap
        // descriptor reports a suballocation size (e.g. 16,512 vs 32,768).
        // SAFETY: sysconf takes a constant selector and no pointers.
        let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) }.max(1) as u64;
        let required = logical
            .max(footprint.size as u64)
            .div_ceil(page)
            .saturating_mul(page);
        let mut reservation = budget.reserve(required, required)?;
        let texture = device
            .newTextureWithDescriptor(&descriptor)
            .ok_or_else(|| reservation.failed())?;
        reservation.confirm((texture.allocatedSize() as u64).max(logical))?;
        Ok(Self {
            texture,
            reservation,
            valid: false,
        })
    }
}
pub(crate) fn texture(
    device: &ProtocolObject<dyn MTLDevice>,
    width: usize,
    height: usize,
) -> Result<Retained<ProtocolObject<dyn MTLTexture>>> {
    anyhow::ensure!(width > 0 && height > 0, "empty damage texture");
    device
        .newTextureWithDescriptor(&descriptor(width, height))
        .context("allocating damage texture")
}
fn descriptor(width: usize, height: usize) -> Retained<MTLTextureDescriptor> {
    let descriptor = MTLTextureDescriptor::new();
    // SAFETY: Positive dimensions describe a two-dimensional BGRA8 target.
    unsafe {
        descriptor.setWidth(width);
        descriptor.setHeight(height);
    }
    descriptor.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
    descriptor.setStorageMode(MTLStorageMode::Private);
    descriptor.setUsage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);
    descriptor
}
