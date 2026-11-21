// SPDX-License-Identifier: Apache-2.0
//! RustUI original: planner call counts, recovery latency and whole-frame pixels.
use super::*;
fn scene_at(lightness: f32, caret: Option<f32>) -> Scene {
    let mut s = Scene::default();
    for (bounds, color) in [
        (
            box_at(0., 0., 64., 64.),
            gpui::Hsla {
                h: 0.3,
                s: 0.8,
                l: lightness,
                a: 1.,
            },
        ),
        (
            box_at(caret.unwrap_or(0.), 20., 2., 12.),
            gpui::Hsla {
                h: 0.,
                s: 1.,
                l: 0.5,
                a: 1.,
            },
        ),
    ]
    .into_iter()
    .take(if caret.is_some() { 2 } else { 1 })
    {
        s.insert_primitive(gpui::Quad {
            bounds,
            content_mask: gpui::ContentMask {
                bounds: box_at(0., 0., 64., 64.),
            },
            background: color.into(),
            ..Default::default()
        });
    }
    s.finish();
    s
}
fn draw(renderer: &mut MetalRenderer, scene: &Scene, expected: Mode, backoff: bool) -> Result<()> {
    let size = gpui::size(DevicePixels(64), DevicePixels(64));
    let reference = retained::texture(&renderer.device, 64, 64)?;
    let whole = renderer.render_frame(scene, &reference, size)?;
    let expected_pixels = finish(renderer, &whole, &reference)?;
    let target = retained::texture(&renderer.device, 64, 64)?;
    let prepared = prepare(renderer, scene, false);
    assert_eq!(
        prepared.mode, expected,
        "expected frame mode must actually run"
    );
    assert_eq!(prepared.backoff, backoff);
    let command = render(renderer, scene, &target, size, prepared)?;
    assert_eq!(renderer.damage.last_mode, expected);
    assert_eq!(finish(renderer, &command, &target)?, expected_pixels);
    Ok(())
}
fn proof() -> Result<()> {
    super::super::backoff::test_policy();
    let mut r = MetalRenderer::new_headless(Default::default())?;
    r.damage.enabled_override = Some(true);
    r.damage.backoff.enabled_override = Some(true);
    draw(&mut r, &scene_at(0.2, None), Mode::Retained, false)?;
    let mut lightness = 0.2;
    for _ in 0..7 {
        lightness = 1. - lightness;
        draw(&mut r, &scene_at(lightness, None), Mode::Direct, false)?;
    }
    assert_eq!(r.damage.backoff.remaining, 30);
    for _ in 0..2 {
        let calls = r.damage.planner_calls;
        for _ in 0..30 {
            lightness = 1. - lightness;
            draw(&mut r, &scene_at(lightness, None), Mode::Direct, true)?;
            assert_eq!(
                r.damage.planner_calls, calls,
                "no planner calls during back-off"
            );
        }
        lightness = 1. - lightness;
        draw(&mut r, &scene_at(lightness, None), Mode::Direct, false)?;
        assert_eq!(
            r.damage.planner_calls,
            calls + 1,
            "one probe between runs of 30"
        );
        assert_eq!(r.damage.backoff.remaining, 30);
    }
    // Count latency in frame intervals from the first small-change frame (0).
    // 0..29 are Direct; 30 is the Retained probe; 31 can be truly Partial.
    let calls = r.damage.planner_calls;
    for frame in 0..30 {
        draw(
            &mut r,
            &scene_at(lightness, Some(12. + (frame % 2) as f32)),
            Mode::Direct,
            true,
        )?;
        assert_eq!(r.damage.planner_calls, calls);
    }
    draw(
        &mut r,
        &scene_at(lightness, Some(12.)),
        Mode::Retained,
        false,
    )?;
    r.damage.magenta_prefill = true;
    let changed = scene_at(lightness, Some(13.));
    let prepared = prepare(&mut r, &changed, false);
    assert!(
        matches!(prepared.mode, Mode::Partial(_)),
        "must recover to actual Partial at frame interval 31"
    );
    // This second prepare is harmless: no completion/counter advance occurred.
    draw(&mut r, &changed, prepared.mode, false)?;
    assert!(r.damage.last_magenta_prefills > 0);
    assert_eq!(r.damage.backoff.remaining, 0);
    Ok(())
}
impl MetalRenderer {
    /// Proves 30 unplanned frames, one probe, resets and byte-exact recovery.
    pub fn test_damage_backoff() -> Result<()> {
        let _serial = PROOF_LOCK.lock().unwrap();
        objc2::rc::autoreleasepool(|_| proof())
    }
}
#[cfg(test)]
#[test]
fn backoff_stops_planning_and_recovers_byte_exactly() -> Result<()> {
    MetalRenderer::test_damage_backoff()
}
