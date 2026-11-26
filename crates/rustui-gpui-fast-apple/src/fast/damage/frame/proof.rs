// SPDX-License-Identifier: Apache-2.0
// Frame modes adapted from Havunen/gpui-ce, commit 28045891bfe87768a2c8e0aefd43403939844318,
// crates/gpui_apple/src/metal_renderer.rs and crates/gpui_render/src/retained.rs.
// Copyright 2022 - 2025 Zed Industries, Inc.; Havunen and gpui-ce contributors.
// RustUI modifications: headless byte comparisons and real display-link skip assertions.
use super::super::verify::Readback;
use super::*;
use objc2_metal::{MTLDevice, MTLResource, MTLStorageMode};
use std::sync::Mutex;
static PROOF_LOCK: Mutex<()> = Mutex::new(());

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
fn preacquisition_instances_match_normal_frame() -> Result<()> {
    let mut renderer = MetalRenderer::new_headless(Default::default())?;
    let scene = scene();
    let size = gpui::size(DevicePixels(64), DevicePixels(64));

    let ordinary = retained::texture(&renderer.device, 64, 64)?;
    let command = renderer.render_frame(&scene, &ordinary, size)?;
    let expected = finish(&renderer, &command, &ordinary)?;

    let (writer, bindings) = renderer.prepare_frame_instances(&scene)?;
    let later = retained::texture(&renderer.device, 64, 64)?;
    let command = renderer.encode_frame(&scene, &later, size, writer, bindings)?;
    let actual = finish(&renderer, &command, &later)?;
    assert!(actual.iter().any(|byte| *byte != 0));
    assert_eq!(actual, expected);
    Ok(())
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
            preacquisition_instances_match_normal_frame()?;
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

fn box_at(x: f32, y: f32, w: f32, h: f32) -> gpui::Bounds<gpui::ScaledPixels> {
    gpui::Bounds {
        origin: gpui::point(gpui::ScaledPixels(x), gpui::ScaledPixels(y)),
        size: gpui::size(gpui::ScaledPixels(w), gpui::ScaledPixels(h)),
    }
}
fn quad(scene: &mut Scene, x: f32, y: f32, w: f32, h: f32, color: gpui::Hsla, radius: f32) {
    scene.insert_primitive(gpui::Quad {
        bounds: box_at(x, y, w, h),
        content_mask: gpui::ContentMask {
            bounds: box_at(0., 0., 256., 256.),
            ..Default::default()
        },
        corner_radii: gpui::Corners::all(gpui::ScaledPixels(radius)),
        background: color.into(),
        ..Default::default()
    });
}
fn partial_scene(case: &str, changed: bool) -> Result<Scene> {
    let mut scene = Scene::default();
    let color = |h, a| gpui::Hsla {
        h,
        s: 0.6,
        l: 0.45,
        a,
    };
    quad(
        &mut scene,
        0.,
        0.,
        256.,
        256.,
        color(0.6, if case == "translucent" { 0.2 } else { 1. }),
        0.,
    );
    match case {
        "row hover" => {
            for row in 0..3 {
                quad(
                    &mut scene,
                    12.,
                    32. + row as f32 * 24.,
                    220.,
                    18.,
                    color(if row == 1 && changed { 0.1 } else { 0.7 }, 1.),
                    0.,
                );
                // Text-shaped unchanged foreground strokes cross the highlight.
                quad(
                    &mut scene,
                    24.,
                    38. + row as f32 * 24.,
                    72.,
                    3.,
                    color(0.9, 1.),
                    0.,
                );
            }
        }
        "caret blink" => {
            quad(&mut scene, 20., 40., 100., 3., color(0.8, 1.), 0.);
            if !changed {
                quad(&mut scene, 124., 36., 2., 14., color(0.2, 1.), 0.);
            }
        }
        "card move" | "translucent" => {
            quad(
                &mut scene,
                if changed { 54. } else { 40. },
                100.,
                48.,
                36.,
                color(0.15, 0.55),
                6.,
            );
            quad(&mut scene, 60., 112., 60., 40., color(0.8, 0.65), 8.);
        }
        "path restart" => {
            let path = |x: f32, color: gpui::Hsla| -> Result<gpui::Path<gpui::ScaledPixels>> {
                let mut builder = gpui::PathBuilder::fill();
                builder.move_to(gpui::point(gpui::px(x), gpui::px(x)));
                builder.line_to(gpui::point(gpui::px(x + 30.), gpui::px(x)));
                builder.line_to(gpui::point(gpui::px(x), gpui::px(x + 30.)));
                builder.close();
                let mut path = builder.build()?.scale(1.);
                path.content_mask.bounds = box_at(0., 0., 256., 256.);
                path.color = color.into();
                Ok(path)
            };
            scene.insert_primitive(path(12., color(0.1, 0.7))?);
            quad(&mut scene, 8., 8., 40., 40., color(0.9, 0.3), 0.);
            scene.insert_primitive(path(16., color(if changed { 0.6 } else { 0.3 }, 0.7))?);
        }
        _ => anyhow::bail!("unknown partial proof"),
    }
    scene.finish();
    Ok(scene)
}
fn partial_pixels(case: &str) -> Result<()> {
    let mut renderer = MetalRenderer::new_headless(Default::default())?;
    renderer.update_transparency(case == "translucent");
    renderer.damage.enabled_override = Some(true);
    renderer.damage.magenta_prefill = true;
    let size = gpui::size(DevicePixels(256), DevicePixels(256));
    let target = retained::texture(&renderer.device, 256, 256)?;
    let old = partial_scene(case, false)?;
    let first = renderer.damage.choose(
        &renderer.sprite_atlas,
        &old,
        size,
        1.,
        renderer.opaque,
        false,
        true,
    );
    assert_eq!(first.mode, Mode::Retained);
    let command = render(&mut renderer, &old, &target, size, first)?;
    let old_pixels = finish(&renderer, &command, &target)?;
    let new = partial_scene(case, true)?;
    let reference = retained::texture(&renderer.device, 256, 256)?;
    let whole = renderer.render_frame(&new, &reference, size)?;
    let expected = finish(&renderer, &whole, &reference)?;
    assert_ne!(old_pixels, expected, "{case} must visibly change");
    let prepared = renderer.damage.choose(
        &renderer.sprite_atlas,
        &new,
        size,
        1.,
        renderer.opaque,
        false,
        true,
    );
    assert!(
        matches!(prepared.mode, Mode::Partial(_)),
        "{case} planned {:?}",
        prepared.mode
    );
    let command = render(&mut renderer, &new, &target, size, prepared)?;
    let actual = finish(&renderer, &command, &target)?;
    assert!(
        matches!(renderer.damage.last_mode, Mode::Partial(_)),
        "{case} must execute Partial"
    );
    assert!(
        renderer.damage.last_partial_passes >= if case == "path restart" { 2 } else { 1 },
        "{case} must reach every main encoder hook"
    );
    assert_eq!(
        renderer.damage.last_magenta_prefills, 1,
        "{case}: prefill and clear happen exactly once"
    );
    if let Some(index) = actual.iter().zip(&expected).position(|(a, b)| a != b) {
        anyhow::bail!(
            "{case}: Partial differs from whole at ({},{}), channel {}, actual {}, expected {}",
            index / 4 % 256,
            index / 4 / 256,
            index % 4,
            actual[index],
            expected[index]
        );
    }
    Ok(())
}
impl MetalRenderer {
    /// Magenta-prefilled Partial proofs; each case asserts the actual frame mode.
    pub fn test_damage_partials() -> Result<()> {
        let _serial = PROOF_LOCK.lock().unwrap();
        objc2::rc::autoreleasepool(|_| {
            for case in [
                "row hover",
                "caret blink",
                "card move",
                "translucent",
                "path restart",
            ] {
                partial_pixels(case)?;
            }
            Ok(())
        })
    }
}
#[cfg(test)]
mod partial_tests {
    #[test]
    fn magenta_partial_frames_equal_whole_and_restart_with_scissor() -> anyhow::Result<()> {
        crate::metal_renderer::MetalRenderer::test_damage_partials()
    }
}

fn wait_verification(renderer: &mut MetalRenderer) {
    let reference = renderer
        .damage
        .take_verification()
        .expect("verify mode must encode a reference");
    reference.commit();
    reference.waitUntilCompleted();
}
fn verify_pair(wrong_box: bool) -> Result<()> {
    let mut renderer = MetalRenderer::new_headless(Default::default())?;
    renderer.damage.enabled_override = Some(true);
    renderer.damage.verify_override = true;
    let size = gpui::size(DevicePixels(256), DevicePixels(256));
    let target = retained::texture(&renderer.device, 256, 256)?;
    let old = partial_scene("card move", false)?;
    let first = renderer.damage.choose(
        &renderer.sprite_atlas,
        &old,
        size,
        1.,
        renderer.opaque,
        false,
        true,
    );
    assert_eq!(first.mode, Mode::Retained);
    let command = render(&mut renderer, &old, &target, size, first)?;
    finish(&renderer, &command, &target)?;
    wait_verification(&mut renderer);
    let status = renderer.damage.failed.as_ref().unwrap().clone();
    assert!(!status.disabled.load(Ordering::Acquire));
    assert_eq!(status.record(), None);
    let new = partial_scene("card move", true)?;
    let reference = retained::texture(&renderer.device, 256, 256)?;
    let whole = renderer.render_frame(&new, &reference, size)?;
    let expected = finish(&renderer, &whole, &reference)?;
    let mut prepared = renderer.damage.choose(
        &renderer.sprite_atlas,
        &new,
        size,
        1.,
        renderer.opaque,
        false,
        true,
    );
    assert!(matches!(prepared.mode, Mode::Partial(_)));
    if wrong_box {
        // The old card is cleared, but the new card's right edge is omitted.
        prepared.mode = Mode::Partial(Area {
            x0: 39.,
            y0: 99.,
            x1: 90.,
            y1: 137.,
        });
    }
    let command = render(&mut renderer, &new, &target, size, prepared)?;
    assert!(matches!(renderer.damage.last_mode, Mode::Partial(_)));
    let actual = finish(&renderer, &command, &target)?;
    wait_verification(&mut renderer);
    let mismatch = super::super::verify::first_mismatch(&actual, &expected, 256);
    if wrong_box {
        let (x, y) = mismatch.expect("deliberately wrong box must leave different pixels");
        assert_eq!(
            status.record(),
            Some(super::super::verify::Mismatch {
                mode: "partial",
                x,
                y
            })
        );
        assert!(
            status.disabled.load(Ordering::Acquire),
            "first mismatch disables damage"
        );
        let next = renderer.damage.choose(
            &renderer.sprite_atlas,
            &new,
            size,
            1.,
            renderer.opaque,
            false,
            true,
        );
        assert_eq!(next.mode, Mode::Direct);
        let command = render(&mut renderer, &new, &target, size, next)?;
        assert_eq!(finish(&renderer, &command, &target)?, expected);
        assert_eq!(renderer.damage.last_mode, Mode::Direct);
        // Composition renderers share their window's atlas. A newly created
        // sibling must observe the sticky disable, while another window does not.
        let mut sibling = MetalRenderer::new_headless(Default::default())?;
        sibling.sprite_atlas = renderer.sprite_atlas.clone();
        sibling.damage.enabled_override = Some(true);
        sibling.damage.verify_override = true;
        let prepared = sibling.damage.choose(
            &sibling.sprite_atlas,
            &new,
            size,
            1.,
            sibling.opaque,
            false,
            true,
        );
        assert_eq!(
            prepared.mode,
            Mode::Direct,
            "all surfaces of the failing window disable damage"
        );
        let mut separate = MetalRenderer::new_headless(Default::default())?;
        separate.damage.enabled_override = Some(true);
        separate.damage.verify_override = true;
        let prepared = separate.damage.choose(
            &separate.sprite_atlas,
            &new,
            size,
            1.,
            separate.opaque,
            false,
            true,
        );
        assert_eq!(
            prepared.mode,
            Mode::Retained,
            "another window stays enabled"
        );
        assert_eq!(
            status.record(),
            Some(super::super::verify::Mismatch {
                mode: "partial",
                x,
                y
            })
        );
    } else {
        assert_eq!(mismatch, None);
        assert_eq!(status.record(), None);
        let copy = renderer.damage.choose(
            &renderer.sprite_atlas,
            &new,
            size,
            1.,
            renderer.opaque,
            false,
            true,
        );
        assert_eq!(copy.mode, Mode::Copy);
        let command = render(&mut renderer, &new, &target, size, copy)?;
        assert_eq!(finish(&renderer, &command, &target)?, expected);
        wait_verification(&mut renderer);
        assert!(!status.disabled.load(Ordering::Acquire));
        assert_eq!(status.record(), None);
    }
    Ok(())
}
fn verify_failure_falls_back() -> Result<()> {
    let mut renderer = MetalRenderer::new_headless(Default::default())?;
    renderer.damage.enabled_override = Some(true);
    renderer.damage.verify_override = true;
    renderer.damage.fail_verification = true;
    let size = gpui::size(DevicePixels(64), DevicePixels(64));
    let scene = scene();
    let target = retained::texture(&renderer.device, 64, 64)?;
    let whole = renderer.render_frame(&scene, &target, size)?;
    let expected = finish(&renderer, &whole, &target)?;
    let prepared = prepare(&mut renderer, &scene, false);
    assert_eq!(prepared.mode, Mode::Retained);
    let command = render(&mut renderer, &scene, &target, size, prepared)?;
    assert_eq!(renderer.damage.last_mode, Mode::Direct);
    assert_eq!(finish(&renderer, &command, &target)?, expected);
    assert!(
        renderer
            .damage
            .failed
            .as_ref()
            .unwrap()
            .disabled
            .load(Ordering::Acquire)
    );
    Ok(())
}
impl MetalRenderer {
    /// Full-reference oracle, deliberate bad-box detection and window isolation.
    pub fn test_damage_verify() -> Result<()> {
        let _serial = PROOF_LOCK.lock().unwrap();
        objc2::rc::autoreleasepool(|_| {
            assert_eq!(config(None), Config::Off);
            assert_eq!(config(Some("0")), Config::Off);
            assert_eq!(config(Some("1")), Config::On);
            assert_eq!(config(Some("verify")), Config::Verify);
            verify_pair(false)?;
            verify_pair(true)?;
            verify_failure_falls_back()
        })
    }
}
#[cfg(test)]
mod verify_tests {
    #[test]
    fn reference_catches_wrong_box_and_disables_only_its_window() -> anyhow::Result<()> {
        crate::metal_renderer::MetalRenderer::test_damage_verify()
    }
}

fn replay_budget_proof() -> Result<()> {
    let mut renderer = MetalRenderer::new_headless(Default::default())?;
    renderer.damage.enabled_override = Some(true);
    let size = gpui::size(DevicePixels(64), DevicePixels(64));
    let target = retained::texture(&renderer.device, 64, 64)?;
    let mut old = Scene::default();
    old.with_fast_damage_clip_for_test(box_at(0., 0., 64., 64.), |s| {
        s.insert_primitive(gpui::Quad {
            bounds: box_at(0., 0., 64., 64.),
            content_mask: gpui::ContentMask {
                bounds: box_at(0., 0., 64., 64.),
            },
            background: gpui::red().into(),
            ..Default::default()
        });
    });
    old.finish();
    let first = prepare(&mut renderer, &old, false);
    assert_eq!(first.mode, Mode::Retained);
    assert!(first.snapshot.as_ref().unwrap().signatures.is_empty());
    assert_eq!(
        renderer.damage.cache.hashed, 0,
        "known whole frame must not hash"
    );
    let command = render(&mut renderer, &old, &target, size, first)?;
    let expected = finish(&renderer, &command, &target)?;
    let mut next = Scene::default();
    next.replay(0..old.len(), &old);
    next.finish();
    let prepared = prepare(&mut renderer, &next, false);
    assert_eq!(prepared.mode, Mode::Copy);
    assert_eq!(
        renderer.damage.cache.reused, 1,
        "rounded compilation must preserve replay identity"
    );
    assert_eq!(
        renderer.damage.cache.hashed, 1,
        "only the new aggregate clip signature is hashed"
    );
    let command = render(&mut renderer, &next, &target, size, prepared)?;
    assert_eq!(finish(&renderer, &command, &target)?, expected);
    let resized = renderer.damage.choose(
        &renderer.sprite_atlas,
        &next,
        gpui::size(DevicePixels(65), DevicePixels(64)),
        1.,
        renderer.opaque,
        false,
        true,
    );
    assert_eq!(resized.mode, Mode::Retained);
    assert!(resized.snapshot.unwrap().signatures.is_empty());
    assert_eq!(renderer.damage.cache.hashed, 0);
    let scaled = renderer.damage.choose(
        &renderer.sprite_atlas,
        &next,
        size,
        2.,
        renderer.opaque,
        false,
        true,
    );
    assert_eq!(scaled.mode, Mode::Retained);
    assert_eq!(renderer.damage.cache.hashed, 0);
    let sync = prepare(&mut renderer, &next, true);
    assert_eq!(sync.mode, Mode::Direct);
    assert_eq!(renderer.damage.cache.hashed, 0);
    let mut tiles = Scene::default();
    tiles.layers.frames.push(gpui::LayerFrame {
        key: gpui::LayerKey(3),
        generation: 0,
        background: Default::default(),
        tile_size: 64,
        content: Scene::default().into(),
        dirty_tiles: vec![],
    });
    assert_eq!(prepare(&mut renderer, &tiles, false).mode, Mode::Direct);
    assert_eq!(
        renderer.damage.cache.hashed, 0,
        "unsupported tiles must not hash"
    );
    let prime = prepare(&mut renderer, &old, false);
    let command = render(&mut renderer, &old, &target, size, prime)?;
    finish(&renderer, &command, &target)?;
    {
        use gpui::PlatformAtlas as _;
        let key = gpui::AtlasKey::Svg(gpui::RenderSvgParams {
            path: "g2-7-loss".into(),
            size: gpui::size(DevicePixels(4), DevicePixels(4)),
        });
        renderer
            .sprite_atlas
            .get_or_insert_with(key.clone(), &mut || {
                Ok(Some((
                    gpui::size(DevicePixels(4), DevicePixels(4)),
                    std::borrow::Cow::Owned(vec![128; 16]),
                )))
            })?;
        renderer.sprite_atlas.remove(&key);
    }
    assert_eq!(prepare(&mut renderer, &old, false).mode, Mode::Direct);
    assert_eq!(
        renderer.damage.cache.hashed, 0,
        "atlas loss must precede hashing"
    );
    Ok(())
}
impl MetalRenderer {
    pub fn test_damage_cpu_budget() -> Result<()> {
        super::super::log::test_records();
        let _serial = PROOF_LOCK.lock().unwrap();
        objc2::rc::autoreleasepool(|_| Scene::with_fast_damage_provenance(replay_budget_proof))
    }
}
#[cfg(test)]
mod cpu_budget_tests {
    #[test]
    fn known_whole_frames_do_not_hash_and_clip_replay_reuses_signatures() -> anyhow::Result<()> {
        crate::metal_renderer::MetalRenderer::test_damage_cpu_budget()
    }
}

mod policy;

mod backoff;
