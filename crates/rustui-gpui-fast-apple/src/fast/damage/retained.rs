// SPDX-License-Identifier: Apache-2.0
// Adapted from Havunen/gpui-ce, commit 28045891bfe87768a2c8e0aefd43403939844318,
// crates/gpui_render/src/retained.rs and crates/gpui_apple/src/metal_renderer.rs.
// Copyright 2022 - 2025 Zed Industries, Inc.; Havunen and gpui-ce contributors.
// RustUI modifications: objc2-metal targets and reservations held through GPU completion.
use anyhow::{Context as _, Result};
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{
    MTLDevice, MTLPixelFormat, MTLStorageMode, MTLTexture, MTLTextureDescriptor, MTLTextureUsage,
};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicU64, Ordering},
    },
};

pub(crate) const DEVICE_LIMIT: u64 = 64 * 1024 * 1024;
struct DeviceBudget(AtomicU64);
pub(crate) struct Reservation {
    budget: Arc<DeviceBudget>,
    bytes: u64,
}
impl Drop for Reservation {
    fn drop(&mut self) {
        self.budget.0.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}
pub(crate) fn reserve(device: u64, bytes: u64) -> Option<Arc<Reservation>> {
    static BUDGETS: OnceLock<Mutex<HashMap<u64, Weak<DeviceBudget>>>> = OnceLock::new();
    let mut budgets = BUDGETS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    budgets.retain(|_, budget| budget.strong_count() != 0);
    let budget = budgets
        .get(&device)
        .and_then(Weak::upgrade)
        .unwrap_or_else(|| {
            let budget = Arc::new(DeviceBudget(AtomicU64::new(0)));
            budgets.insert(device, Arc::downgrade(&budget));
            budget
        });
    budget
        .0
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |used| {
            used.checked_add(bytes).filter(|n| *n <= DEVICE_LIMIT)
        })
        .inspect_err(|used| super::fs_trace::denied(*used, bytes))
        .ok()?;
    Some(Arc::new(Reservation { budget, bytes }))
}
pub(crate) struct Target {
    pub(crate) texture: Retained<ProtocolObject<dyn MTLTexture>>,
    pub(crate) reservation: Arc<Reservation>,
    pub(crate) valid: bool,
}
impl Target {
    pub(crate) fn new(
        device: &ProtocolObject<dyn MTLDevice>,
        width: usize,
        height: usize,
    ) -> Result<Option<Self>> {
        let bytes = (width as u64)
            .checked_mul(height as u64)
            .and_then(|n| n.checked_mul(4))
            .context("retained texture byte count")?;
        let Some(reservation) = reserve(device.registryID(), bytes) else {
            return Ok(None);
        };
        let texture = texture(device, width, height)?;
        Ok(Some(Self {
            texture,
            reservation,
            valid: false,
        }))
    }
}
pub(crate) fn texture(
    device: &ProtocolObject<dyn MTLDevice>,
    width: usize,
    height: usize,
) -> Result<Retained<ProtocolObject<dyn MTLTexture>>> {
    anyhow::ensure!(width > 0 && height > 0, "empty damage texture");
    let descriptor = MTLTextureDescriptor::new();
    // SAFETY: Positive dimensions describe a two-dimensional BGRA8 target.
    unsafe {
        descriptor.setWidth(width);
        descriptor.setHeight(height);
    }
    descriptor.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
    descriptor.setStorageMode(MTLStorageMode::Private);
    descriptor.setUsage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);
    device
        .newTextureWithDescriptor(&descriptor)
        .context("allocating retained damage texture")
}

#[cfg(any(test, feature = "test-support"))]
pub(crate) fn test_budget() {
    let first = reserve(u64::MAX, DEVICE_LIMIT - 1).unwrap();
    assert!(reserve(u64::MAX, 2).is_none());
    assert!(reserve(u64::MAX - 1, 2).is_some());
    let in_flight = first.clone();
    drop(first);
    assert!(
        reserve(u64::MAX, 2).is_none(),
        "GPU completion must retain the reservation"
    );
    drop(in_flight);
    assert!(reserve(u64::MAX, DEVICE_LIMIT).is_some());
    let target = 3840 * 2160 * 4;
    let a = reserve(u64::MAX - 2, target).unwrap();
    let b = reserve(u64::MAX - 2, target).unwrap();
    assert!(reserve(u64::MAX - 2, target).is_none());
    drop((a, b));
}
