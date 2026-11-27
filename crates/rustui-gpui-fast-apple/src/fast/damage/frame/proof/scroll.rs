// SPDX-License-Identifier: Apache-2.0
use super::*;
use gpui::{FastScrollViewport, Hsla, ScaledPixels};
use std::collections::BTreeMap;

fn color(n: i32) -> Hsla {
    Hsla {
        h: (n.rem_euclid(29) as f32) / 29.,
        s: 0.7,
        l: 0.49,
        a: 1.,
    }
}
fn rows(
    scene: &mut Scene,
    id: u64,
    clip: gpui::Bounds<ScaledPixels>,
    dx: f32,
    dy: f32,
    last: Option<i32>,
) {
    scene.record_scroll_viewport(FastScrollViewport {
        id,
        content_revision: None,
        row_height: 0.,
        clip,
        offset: gpui::point(ScaledPixels(dx), ScaledPixels(dy)),
    });
    Scene::with_test_scroll_owner(id, || {
        let first = (-dy / 9.).floor() as i32 - 1;
        for row in first..=first + 16 {
            if last.is_some_and(|last| row > last) {
                break;
            }
            let y = clip.origin.y.0 + row as f32 * 9. + dy;
            let x = clip.origin.x.0 + dx.rem_euclid(23.);
            for cell in -1..6 {
                scene.insert_primitive(gpui::Quad {
                    bounds: box_at(x + cell as f32 * 23., y, 20., 8.),
                    content_mask: gpui::ContentMask {
                        bounds: clip,
                        ..Default::default()
                    },
                    background: color(row + cell * 2).into(),
                    ..Default::default()
                });
            }
        }
    });
}
fn image(case: &str, frame: i32) -> Scene {
    let mut scene = Scene::default();
    quad(&mut scene, 0., 0., 192., 144., color(3), 0.);
    let clip = box_at(8., 10., 116., 90.);
    let n = frame as f32;
    let offset = match case {
        "up" => n,
        "momentum" => {
            -((frame / 32) * 180 + (0..frame % 32).map(|i| (32 - i) / 4 + 1).sum::<i32>()) as f32
        }
        "fractional" => -n * 0.5,
        "oversized" => -n * 96.,
        "append" => -n * 9.,
        _ => -n,
    };
    rows(
        &mut scene,
        11,
        clip,
        0.,
        offset,
        (case == "append").then_some(frame + 9),
    );
    if matches!(case, "declared" | "declared-content" | "declared-lie") {
        let mut v = scene.fast_scroll_viewports()[0];
        v.content_revision = Some(if case == "declared-content" {
            (frame / 3) as u64
        } else {
            1
        });
        v.row_height = 0.; // Fixture paints overscan at both edges.
        scene.record_scroll_viewport(v);
        if case != "declared" {
            // This belongs to the moving content. The extra stationary patch changes its pixels.
            Scene::with_test_scroll_owner(11, || {
                scene.insert_primitive(gpui::Quad {
                    bounds: box_at(46., 45. - (frame % 3) as f32, 14., 9.),
                    content_mask: gpui::ContentMask {
                        bounds: clip,
                        ..Default::default()
                    },
                    background: color(frame / 3 + 8).into(),
                    ..Default::default()
                });
            });
        }
        // This change is outside the declaration and must always be diffed.
        quad(&mut scene, 153., 88., 14., 9., color(frame), 0.);
    }
    match case {
        "sticky" => quad(&mut scene, 8., 10., 116., 14., color(7), 0.),
        "scrollbar" => quad(&mut scene, 118., 12. + n % 52., 5., 21., color(9), 0.),
        "nested" => rows(
            &mut scene,
            22,
            box_at(24., 24. + (frame % 5) as f32, 72., 42.),
            0.,
            -2. * n,
            None,
        ),
        "disjoint" => rows(&mut scene, 22, box_at(133., 20., 53., 72.), n, 0., None),
        "hover" => {
            quad(
                &mut scene,
                11.,
                20. + (frame % 6) as f32 * 10.,
                88.,
                8.,
                color(frame % 13),
                0.,
            );
            quad(&mut scene, 135., 82., 18., 13., color(frame % 23), 0.);
        }
        "sticky-alpha" => Scene::with_test_scroll_owner(11, || {
            quad(
                &mut scene,
                6.,
                9.,
                140.,
                109.,
                Hsla {
                    a: 0.4,
                    ..color(frame)
                },
                0.,
            )
        }),
        "translucent" => quad(
            &mut scene,
            16.,
            24.,
            72.,
            13.,
            Hsla { a: 0.4, ..color(1) },
            0.,
        ),
        _ => {}
    }
    scene.finish();
    scene
}
impl MetalRenderer {
    #[doc(hidden)]
    pub fn test_scroll_shift_frames() -> Result<()> {
        Scene::with_test_scroll_shift(|| Self::scroll_shift_frames())
    }
    fn scroll_shift_frames() -> Result<()> {
        for case in [
            "declared",
            "declared-content",
            "down",
            "up",
            "momentum",
            "fractional",
            "sticky",
            "scrollbar",
            "nested",
            "hover",
            "append",
            "disjoint",
            "oversized",
            "translucent",
            "sticky-alpha",
        ] {
            let mut renderer = Self::new_headless(Default::default())?;
            renderer.damage.enabled_override = Some(true);
            renderer.damage.verify_override = true;
            renderer.damage.magenta_prefill = true;
            let target = retained::texture(&renderer.device, 192, 144)?;
            let size = gpui::size(DevicePixels(192), DevicePixels(144));
            let mut counts = BTreeMap::new();
            for frame in 0..=300 {
                objc2::rc::autoreleasepool(|_| -> Result<()> {
                    let scene = Scene::with_fast_damage_provenance(|| image(case, frame));
                    let prepared = renderer.damage.choose(
                        &renderer.sprite_atlas,
                        &scene,
                        size,
                        2.,
                        true,
                        false,
                        true,
                    );
                    let reason = prepared.reason;
                    let expected = match (frame, case) {
                        (0, _) => "prime",
                        (_, "declared") => "shift_declared",
                        (_, "declared-content") if frame % 3 != 0 => "shift_declared",
                        (_, "nested") => "nested_overlap",
                        (_, "fractional") => "fractional_delta",
                        (_, "oversized") => "delta_too_large",
                        (_, "translucent" | "sticky-alpha") => "overlay_translucent",
                        _ => "shift",
                    };
                    anyhow::ensure!(
                        reason == expected,
                        "{case} frame {frame}: expected {expected}, got {reason}"
                    );
                    if frame > 0 && matches!(expected, "shift" | "shift_declared") {
                        let scroll = prepared.scroll.as_ref().unwrap();
                        anyhow::ensure!(
                            scroll.copies.iter().any(|(src, dst)| src != dst),
                            "{case}: no translated copy"
                        );
                        if case == "disjoint" {
                            let moved =
                                scroll.copies.iter().filter(|(src, dst)| src != dst).count();
                            anyhow::ensure!(
                                moved == 2,
                                "both scroll containers must shift, got {moved}"
                            );
                        }
                        if case == "down" || case == "up" {
                            anyhow::ensure!(
                                scroll.damage.iter().map(Area::area).sum::<f32>() < 116. * 90. / 2.,
                                "entire clip repaired: {:?}",
                                scroll.damage
                            );
                        }
                    }
                    let command = render(&mut renderer, &scene, &target, size, prepared)?;
                    command.commit();
                    wait_verification(&mut renderer);
                    let status = renderer
                        .damage
                        .failed
                        .as_ref()
                        .context("missing verify status")?;
                    anyhow::ensure!(
                        !status.disabled.load(Ordering::Acquire),
                        "{case} frame {frame} mismatch: {:?}",
                        status.record().map(|m| (m.x, m.y))
                    );
                    let mode = renderer.damage.last_mode.label();
                    let expected_mode = match expected {
                        "prime" => "retained",
                        "nested_overlap" | "overlay_translucent" => "direct",
                        "shift" | "shift_declared" => "shift",
                        _ => "partial",
                    };
                    anyhow::ensure!(
                        mode == expected_mode,
                        "{case} frame {frame} drew {mode}, expected {expected_mode}"
                    );
                    if frame > 0 {
                        *counts.entry((mode, reason)).or_insert(0) += 1;
                    }
                    Ok(())
                })?;
            }
            println!("G3_ORACLE {case} verified=300 prime=1 counts={counts:?}");
        }
        // Resize and explicit texture loss must both prime before the next shift.
        let mut renderer = Self::new_headless(Default::default())?;
        renderer.damage.enabled_override = Some(true);
        renderer.damage.verify_override = true;
        for (n, width, expected) in [
            (0, 192, "prime"),
            (1, 192, "shift"),
            (2, 221, "prime"),
            (3, 221, "shift"),
            (4, 221, "prime"),
            (5, 221, "shift"),
        ] {
            if n == 4 {
                renderer.damage.invalidate();
            }
            let size = gpui::size(DevicePixels(width), DevicePixels(144));
            renderer.damage.resized(size);
            let scene = Scene::with_fast_damage_provenance(|| image("down", n));
            let prepared =
                renderer
                    .damage
                    .choose(&renderer.sprite_atlas, &scene, size, 2., true, false, true);
            anyhow::ensure!(
                prepared.reason == expected,
                "transition frame {n}: {}",
                prepared.reason
            );
            let output = retained::texture(&renderer.device, width as usize, 144)?;
            let cmd = render(&mut renderer, &scene, &output, size, prepared)?;
            cmd.commit();
            wait_verification(&mut renderer);
            anyhow::ensure!(
                !renderer
                    .damage
                    .failed
                    .as_ref()
                    .unwrap()
                    .disabled
                    .load(Ordering::Acquire),
                "transition frame {n} differs"
            );
        }
        println!("G3_TRANSITIONS resize and texture loss primed and resumed shift");
        let mut renderer = Self::new_headless(Default::default())?;
        renderer.damage.enabled_override = Some(true);
        renderer.damage.verify_override = true;
        let target = retained::texture(&renderer.device, 192, 144)?;
        for n in 2..=3 {
            let scene = Scene::with_fast_damage_provenance(|| image("declared-lie", n));
            let size = gpui::size(DevicePixels(192), DevicePixels(144));
            let prepared =
                renderer
                    .damage
                    .choose(&renderer.sprite_atlas, &scene, size, 2., true, false, true);
            if n == 3 {
                anyhow::ensure!(
                    prepared.reason == "shift_declared",
                    "lying declaration was not exercised"
                );
            }
            let command = render(&mut renderer, &scene, &target, size, prepared)?;
            command.commit();
            wait_verification(&mut renderer);
        }
        anyhow::ensure!(
            renderer
                .damage
                .failed
                .as_ref()
                .unwrap()
                .disabled
                .load(Ordering::Acquire),
            "unchanged revision hid a real change"
        );
        println!("G3B fake stable revision rejected by verify");
        Self::test_fs2_drawable_sizes()
    }
}
#[cfg(test)]
#[test]
fn scroll_shift_pixel_history() {
    MetalRenderer::test_scroll_shift_frames().unwrap();
}
