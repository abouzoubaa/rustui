// SPDX-License-Identifier: Apache-2.0
//! One ordered command buffer blits retained A into B, repairs B, then presents it.
use super::{
    frame::Mode,
    partial,
    plan::{Area, Sig, scroll::ScrollPlan},
    retained::Target,
};
use crate::metal_renderer::MetalRenderer;
use anyhow::{Context as _, Result, ensure};
use gpui::{DevicePixels, Scene, Size};
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{
    MTLBlitCommandEncoder, MTLCommandBuffer, MTLCommandEncoder, MTLCommandQueue, MTLOrigin,
    MTLSize, MTLTexture,
};
use std::cell::RefCell;
type Command = Retained<ProtocolObject<dyn MTLCommandBuffer>>;
type Texture = Retained<ProtocolObject<dyn MTLTexture>>;
thread_local! {static ACTIVE:RefCell<Option<(Texture,Command)>>=const{RefCell::new(None)};}
pub(crate) fn command_for(target: &ProtocolObject<dyn MTLTexture>) -> Option<Command> {
    ACTIVE.with_borrow(|active| {
        active
            .as_ref()
            .filter(|(t, _)| std::ptr::eq(&**t, target))
            .map(|(_, c)| c.clone())
    })
}
struct Restore(Option<(Texture, Command)>);
impl Drop for Restore {
    fn drop(&mut self) {
        ACTIVE.replace(self.0.take());
    }
}
fn blit(
    command: &Command,
    source: &Texture,
    dest: &Texture,
    copies: &[(Area, Area)],
) -> Result<()> {
    ensure!(
        !std::ptr::eq(&**source, &**dest),
        "aliased retained scroll surfaces"
    );
    let encoder = command
        .blitCommandEncoder()
        .context("creating scroll blit encoder")?;
    for &(from, to) in copies {
        let origin = |r: Area| MTLOrigin {
            x: r.x0 as usize,
            y: r.y0 as usize,
            z: 0,
        };
        let size = MTLSize {
            width: (from.x1 - from.x0) as usize,
            height: (from.y1 - from.y0) as usize,
            depth: 1,
        };
        // All copies are clipped to equal-sized integer rectangles on distinct BGRA targets.
        unsafe {
            encoder.copyFromTexture_sourceSlice_sourceLevel_sourceOrigin_sourceSize_toTexture_destinationSlice_destinationLevel_destinationOrigin(source,0,0,origin(from),size,dest,0,0,origin(to));
        }
    }
    encoder.endEncoding();
    Ok(())
}
// Fast scrolling lists consist of quads and monochrome glyphs. Preserve the
// original finished batch order, and upload just the instances the scissor can
// see. A complex scene still uses the ordinary renderer with the same scissor.
fn repair<'a>(
    scene: &'a Scene,
    signatures: &[Sig],
    area: Area,
    slot: &'a mut Option<Box<Scene>>,
) -> &'a Scene {
    if !scene.layers.frames.is_empty()
        || !scene.shadows.is_empty()
        || !scene.paths.is_empty()
        || !scene.underlines.is_empty()
        || !scene.surfaces.is_empty()
        || !scene.subpixel_sprites.is_empty()
        || !scene.polychrome_sprites.is_empty()
        || signatures.len() != scene.quads.len() + scene.monochrome_sprites.len()
    {
        return scene;
    }
    let sub = slot.get_or_insert_with(Default::default);
    let (quads, glyphs) = signatures.split_at(scene.quads.len());
    sub.quads.clear();
    sub.monochrome_sprites.clear();
    sub.quads.extend(
        scene
            .quads
            .iter()
            .zip(quads)
            .filter(|(_, s)| !area.and(&s.reach).is_empty())
            .map(|(quad, _)| *quad),
    );
    sub.monochrome_sprites.extend(
        scene
            .monochrome_sprites
            .iter()
            .zip(glyphs)
            .filter(|(_, s)| !area.and(&s.reach).is_empty())
            .map(|(glyph, _)| *glyph),
    );
    sub
}
pub(super) fn render(
    renderer: &mut MetalRenderer,
    scene: &Scene,
    drawable: &ProtocolObject<dyn MTLTexture>,
    size: Size<DevicePixels>,
    mode: Mode,
    plan: Option<&ScrollPlan>,
    signatures: Option<&[Sig]>,
    repair_scene: &mut Option<Box<Scene>>,
    current: &Target,
    next: &mut Target,
    magenta: bool,
) -> Result<Command> {
    let cmd = renderer
        .command_queue
        .commandBuffer()
        .context("creating retained scroll command")?;
    let _restore = Restore(ACTIVE.replace(Some((next.texture.clone(), cmd.clone()))));
    if mode == Mode::Retained {
        renderer.render_frame(scene, &next.texture, size)?;
    } else {
        ensure!(current.valid, "scroll copy from invalid retained frame");
        let full = Area {
            x0: 0.,
            y0: 0.,
            x1: size.width.0 as f32,
            y1: size.height.0 as f32,
        };
        let whole = [(full, full)];
        let copies = plan.map_or(&whole[..], |p| p.copies.as_slice());
        blit(&cmd, &current.texture, &next.texture, copies)?;
        let single;
        let rects = if let Some(p) = plan {
            &p.damage[..]
        } else if let Mode::Partial(area) = mode {
            single = [area];
            &single[..]
        } else {
            &[]
        };
        for &area in rects {
            let scope = partial::Scope::new(
                next.texture.clone(),
                area,
                renderer.damage_pipelines.clone(),
                magenta,
            )?;
            let sub = if plan.is_some()
                && let Some(sigs) = signatures
            {
                repair(scene, sigs, area, repair_scene)
            } else {
                scene
            };
            renderer.render_frame(sub, &next.texture, size)?;
            ensure!(scope.finish().encoders > 0, "missing scroll repair encoder");
        }
    }
    renderer.damage_pipelines.copy(
        &cmd,
        &next.texture,
        drawable,
        size,
        matches!(mode, Mode::Partial(_) | Mode::Shift),
    )?;
    next.valid = true;
    Ok(cmd)
}
