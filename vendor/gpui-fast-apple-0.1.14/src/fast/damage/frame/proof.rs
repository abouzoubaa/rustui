// SPDX-License-Identifier: Apache-2.0
// Frame modes adapted from Havunen/gpui-ce, commit 28045891bfe87768a2c8e0aefd43403939844318,
// crates/gpui_apple/src/metal_renderer.rs and crates/gpui_render/src/retained.rs.
// Copyright 2022 - 2025 Zed Industries, Inc.; Havunen and gpui-ce contributors.
// RustUI modifications: headless byte comparisons and real display-link skip assertions.
use super::*;
use objc2_metal::{
    MTLBlitCommandEncoder, MTLBuffer, MTLCommandEncoder, MTLDevice, MTLOrigin, MTLResource,
    MTLResourceOptions, MTLSize, MTLStorageMode,
};
use std::sync::Mutex;
static PROOF_LOCK: Mutex<()> = Mutex::new(());

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
fn finish(
    renderer: &MetalRenderer,
    command: &ProtocolObject<dyn MTLCommandBuffer>,
    target: &ProtocolObject<dyn MTLTexture>,
) -> Result<Vec<u8>> {
    let read = Readback::encode(renderer, command, target)?;
    command.commit();
    command.waitUntilCompleted();
    anyhow::ensure!(
        command.status() == MTLCommandBufferStatus::Completed,
        "frame proof GPU failure: {:?}",
        command.error()
    );
    Ok(read.pixels())
}
fn scene() -> Scene {
    let bounds = |x, y, w, h| gpui::Bounds {
        origin: gpui::point(gpui::ScaledPixels(x), gpui::ScaledPixels(y)),
        size: gpui::size(gpui::ScaledPixels(w), gpui::ScaledPixels(h)),
    };
    let mut scene = Scene::default();
    scene.insert_primitive(gpui::Quad {
        bounds: bounds(7., 9., 31., 23.),
        content_mask: gpui::ContentMask {
            bounds: bounds(0., 0., 64., 64.),
            ..Default::default()
        },
        background: gpui::Hsla {
            h: 0.3,
            s: 0.8,
            l: 0.5,
            a: 0.65,
        }
        .into(),
        ..Default::default()
    });
    scene.finish();
    scene
}
fn prepare(renderer: &mut MetalRenderer, scene: &Scene, synchronous: bool) -> Prepared {
    renderer.damage.choose(
        &renderer.sprite_atlas,
        scene,
        gpui::size(DevicePixels(64), DevicePixels(64)),
        1.,
        renderer.opaque,
        synchronous,
        true,
    )
}
fn copy_and_fallbacks() -> Result<()> {
    let mut renderer = MetalRenderer::new_headless(Default::default())?;
    let scene = scene();
    let size = gpui::size(DevicePixels(64), DevicePixels(64));
    let target = retained::texture(&renderer.device, 64, 64)?;
    let whole = renderer.render_frame(&scene, &target, size)?;
    let expected = finish(&renderer, &whole, &target)?;
    let off = prepare(&mut renderer, &scene, false);
    assert_eq!(off.mode, Mode::Direct, "damage defaults off");
    let command = render(&mut renderer, &scene, &target, size, off)?;
    assert_eq!(finish(&renderer, &command, &target)?, expected);
    assert!(renderer.damage.target.is_none());
    assert!(
        renderer.damage.failed.is_none(),
        "default-off frames need no damage callback allocation"
    );
    renderer.damage.enabled_override = Some(true);
    let first = prepare(&mut renderer, &scene, false);
    assert_eq!(first.mode, Mode::Retained);
    let command = render(&mut renderer, &scene, &target, size, first)?;
    assert_eq!(finish(&renderer, &command, &target)?, expected);
    assert_eq!(renderer.damage.last_mode, Mode::Retained);
    assert_eq!(
        renderer
            .damage
            .target
            .as_ref()
            .unwrap()
            .texture
            .storageMode(),
        MTLStorageMode::Private
    );
    // Replace the output so equality cannot be satisfied by pixels left there
    // by the first whole frame. The Copy path must write every output byte.
    let copied = retained::texture(&renderer.device, 64, 64)?;
    let same = prepare(&mut renderer, &scene, false);
    assert_eq!(same.mode, Mode::Copy);
    let command = render(&mut renderer, &scene, &copied, size, same)?;
    assert_eq!(
        renderer.damage.last_mode,
        Mode::Copy,
        "copy may not fall back to a whole draw"
    );
    assert_eq!(finish(&renderer, &command, &copied)?, expected);
    let synchronous = prepare(&mut renderer, &scene, true);
    assert_eq!(
        synchronous.mode,
        Mode::Direct,
        "live resize must draw whole"
    );
    let command = render(&mut renderer, &scene, &target, size, synchronous)?;
    assert_eq!(finish(&renderer, &command, &target)?, expected);
    assert_eq!(renderer.damage.last_mode, Mode::Direct);
    // Occupy this physical device's complete retained budget, then prove the
    // production allocation path falls back without changing any pixels.
    renderer.damage.invalidate();
    let budget = retained::reserve(renderer.device.registryID(), retained::DEVICE_LIMIT)
        .context("exclusive test budget")?;
    let denied = prepare(&mut renderer, &scene, false);
    assert_eq!(denied.mode, Mode::Retained);
    let command = render(&mut renderer, &scene, &target, size, denied)?;
    assert_eq!(
        renderer.damage.last_mode,
        Mode::Direct,
        "over budget must draw whole"
    );
    assert!(renderer.damage.target.is_none());
    assert_eq!(finish(&renderer, &command, &target)?, expected);
    let no_kept = renderer.damage.choose(
        &renderer.sprite_atlas,
        &scene,
        size,
        1.,
        renderer.opaque,
        false,
        false,
    );
    assert_ne!(
        no_kept.mode,
        Mode::Same,
        "over-budget frames must draw whole"
    );
    let command = render(&mut renderer, &scene, &target, size, no_kept)?;
    assert_eq!(renderer.damage.last_mode, Mode::Direct);
    assert_eq!(finish(&renderer, &command, &target)?, expected);
    drop(budget);
    renderer.damage.invalidate();
    renderer.damage.fail_allocation = true;
    for _ in 0..2 {
        let failed = prepare(&mut renderer, &scene, false);
        let command = render(&mut renderer, &scene, &target, size, failed)?;
        assert_eq!(
            renderer.damage.last_mode,
            Mode::Direct,
            "allocation errors must still present a whole draw"
        );
        assert!(renderer.damage.target.is_none());
        assert_eq!(finish(&renderer, &command, &target)?, expected);
    }
    renderer.damage.fail_allocation = false;
    // A stale-size display-link drawable also forces the direct path.
    let mut stale = prepare(&mut renderer, &scene, false);
    stale.snapshot.as_mut().unwrap().geometry.width = 65;
    let command = render(&mut renderer, &scene, &target, size, stale)?;
    assert_eq!(renderer.damage.last_mode, Mode::Direct);
    assert_eq!(finish(&renderer, &command, &target)?, expected);
    Ok(())
}
fn skip_drawable() -> Result<()> {
    let mut renderer = MetalRenderer::new(Default::default(), false, false)?;
    renderer.damage.enabled_override = Some(true);
    renderer.update_drawable_size(gpui::size(DevicePixels(64), DevicePixels(64)));
    let scene = scene();
    renderer.set_display_link_attached(true);
    let drawable = renderer
        .layer()
        .unwrap()
        .nextDrawable()
        .context("prime display-link drawable")?;
    renderer.set_frame_drawable(Some(drawable));
    renderer.draw(&scene);
    assert_eq!(renderer.damage.last_mode, Mode::Retained);
    assert!(
        renderer.set_frame_drawable(None).is_none(),
        "draw must consume its drawable"
    );
    let fence = renderer
        .command_queue
        .commandBuffer()
        .context("completion fence")?;
    fence.commit();
    fence.waitUntilCompleted();
    let drawable = renderer
        .layer()
        .unwrap()
        .nextDrawable()
        .context("skip display-link drawable")?;
    renderer.set_frame_drawable(Some(drawable.clone()));
    gpui::with_fast_present_required(false, || renderer.draw(&scene));
    assert_eq!(
        renderer.damage.last_mode,
        Mode::Same,
        "the skip path must actually run"
    );
    let unused = renderer
        .set_frame_drawable(None)
        .context("Skip consumed the drawable")?;
    assert!(
        std::ptr::eq(&*unused, &*drawable),
        "Skip must return the untouched drawable"
    );
    Ok(())
}
impl MetalRenderer {
    /// Real headless Copy, device-budget fallback, live-resize and drawable Skip proofs.
    pub fn test_damage_frames() -> Result<()> {
        let _serial = PROOF_LOCK.lock().unwrap();
        objc2::rc::autoreleasepool(|_| {
            retained::test_budget();
            copy_and_fallbacks()?;
            skip_drawable()
        })
    }
}
#[cfg(test)]
pub(crate) fn test_skip_leaves_drawable_unused() -> Result<()> {
    let _serial = PROOF_LOCK.lock().unwrap();
    objc2::rc::autoreleasepool(|_| skip_drawable())
}
#[cfg(test)]
mod tests {
    #[test]
    fn retained_copy_budget_resize_and_skip_are_correct() -> anyhow::Result<()> {
        crate::metal_renderer::MetalRenderer::test_damage_frames()
    }
}
