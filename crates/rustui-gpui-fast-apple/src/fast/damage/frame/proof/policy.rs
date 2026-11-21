// SPDX-License-Identifier: Apache-2.0
// RustUI original: Direct hysteresis and full-reference Metal proofs.
use super::*;

fn whole_scene(lightness: f32) -> Scene {
    let mut scene = Scene::default();
    scene.insert_primitive(gpui::Quad {
        bounds: box_at(0., 0., 64., 64.),
        content_mask: gpui::ContentMask {
            bounds: box_at(0., 0., 64., 64.),
        },
        background: gpui::Hsla {
            h: 0.3,
            s: 0.8,
            l: lightness,
            a: 1.,
        }
        .into(),
        ..Default::default()
    });
    scene.finish();
    scene
}
fn draw_exact(renderer: &mut MetalRenderer, scene: &Scene, expected_mode: Mode) -> Result<()> {
    let size = gpui::size(DevicePixels(64), DevicePixels(64));
    let reference = retained::texture(&renderer.device, 64, 64)?;
    let command = renderer.render_frame(scene, &reference, size)?;
    let expected = finish(renderer, &command, &reference)?;
    let target = retained::texture(&renderer.device, 64, 64)?;
    let prepared = prepare(renderer, scene, false);
    assert_eq!(prepared.mode, expected_mode, "the selected path must run");
    let command = render(renderer, scene, &target, size, prepared)?;
    assert_eq!(renderer.damage.last_mode, expected_mode);
    assert_eq!(finish(renderer, &command, &target)?, expected);
    Ok(())
}
fn direct_policy() -> Result<()> {
    let mut renderer = MetalRenderer::new_headless(Default::default())?;
    renderer.damage.enabled_override = Some(true);
    draw_exact(&mut renderer, &whole_scene(0.2), Mode::Retained)?;
    for index in 0..6 {
        let scene = whole_scene(if index % 2 == 0 { 0.8 } else { 0.2 });
        draw_exact(&mut renderer, &scene, Mode::Direct)?;
        assert!(renderer.damage.previous_large);
        // A single identical frame between large changes must not trigger a
        // Retained/Direct cycle. Its metadata is useful even with invalid pixels.
        draw_exact(&mut renderer, &scene, Mode::Direct)?;
        assert!(!renderer.damage.previous_large);
    }
    let scene = whole_scene(0.2);
    draw_exact(&mut renderer, &scene, Mode::Retained)?;
    draw_exact(&mut renderer, &scene, Mode::Copy)?;
    let mut unsupported = Scene::default();
    unsupported.layers.frames.push(gpui::LayerFrame {
        key: gpui::LayerKey(3),
        generation: 0,
        background: Default::default(),
        tile_size: 64,
        content: Scene::default().into(),
        dirty_tiles: vec![],
    });
    for _ in 0..3 {
        let prepared = prepare(&mut renderer, &unsupported, false);
        assert_eq!(prepared.mode, Mode::Direct);
        assert!(
            renderer.damage.previous_large,
            "unsupported preflight preserves history"
        );
        assert!(prepared.large);
        assert_eq!(renderer.damage.cache.hashed, 0);
        renderer.damage.complete(prepared, Mode::Direct);
    }
    // Metadata primes directly after unsupported frames. Two comparable cheap
    // follow-ups then justify rebuilding retained pixels, followed by Copy.
    draw_exact(&mut renderer, &scene, Mode::Direct)?;
    draw_exact(&mut renderer, &scene, Mode::Direct)?;
    draw_exact(&mut renderer, &scene, Mode::Retained)?;
    draw_exact(&mut renderer, &scene, Mode::Copy)?;
    Ok(())
}
impl MetalRenderer {
    pub fn test_damage_direct_policy() -> Result<()> {
        let _serial = PROOF_LOCK.lock().unwrap();
        objc2::rc::autoreleasepool(|_| direct_policy())
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn large_and_unsupported_runs_do_not_alternate_retained_and_direct() -> Result<(), anyhow::Error>
    {
        MetalRenderer::test_damage_direct_policy()
    }
}
