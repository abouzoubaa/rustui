// SPDX-License-Identifier: Apache-2.0
// Adapted from Havunen/gpui-ce, commit 28045891bfe87768a2c8e0aefd43403939844318,
// crates/gpui_apple/src/metal_renderer.rs (partial Load passes and draw_clear_quad).
// Copyright 2022 - 2025 Zed Industries, Inc.; Havunen and gpui-ce contributors.
// RustUI modifications: target-specific objc2-metal hooks, scoped state and magenta proof.
use super::{plan::Area, shaders::Pipelines};
use anyhow::Result;
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{MTLClearColor, MTLRenderCommandEncoder, MTLScissorRect, MTLTexture};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

thread_local! { static PASS: RefCell<Option<Pass>> = const { RefCell::new(None) }; }
struct Pass {
    target: Retained<ProtocolObject<dyn MTLTexture>>,
    rect: MTLScissorRect,
    pipelines: Pipelines,
    color: Option<MTLClearColor>,
    cleared: bool,
    magenta: bool,
    stats: Stats,
}
#[derive(Clone, Copy, Default)]
pub(crate) struct Stats {
    pub(crate) encoders: usize,
    pub(crate) prefills: usize,
}
/// Kept on the encoding thread. Nested work restores its enclosing target,
/// including when an encoder or render_frame returns an error.
pub(crate) struct Scope {
    previous: Option<Pass>,
    _thread: PhantomData<Rc<()>>,
}
impl Scope {
    pub(crate) fn new(
        target: Retained<ProtocolObject<dyn MTLTexture>>,
        area: Area,
        pipelines: Pipelines,
        magenta: bool,
    ) -> Result<Self> {
        let area = area.pixels();
        anyhow::ensure!(
            !area.is_empty()
                && area.x0 >= 0.
                && area.y0 >= 0.
                && area.x1 <= target.width() as f32
                && area.y1 <= target.height() as f32,
            "invalid partial frame box"
        );
        let rect = MTLScissorRect {
            x: area.x0 as usize,
            y: area.y0 as usize,
            width: (area.x1 - area.x0) as usize,
            height: (area.y1 - area.y0) as usize,
        };
        let pass = Pass {
            target,
            rect,
            pipelines,
            color: None,
            cleared: false,
            magenta,
            stats: Stats::default(),
        };
        Ok(Self {
            previous: PASS.with(|p| p.replace(Some(pass))),
            _thread: PhantomData,
        })
    }
    pub(crate) fn finish(self) -> Stats {
        PASS.with(|p| p.borrow().as_ref().unwrap().stats)
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        PASS.with(|p| {
            p.replace(self.previous.take());
        });
    }
}

pub(crate) fn pass_clear(
    texture: &ProtocolObject<dyn MTLTexture>,
    color: Option<MTLClearColor>,
) -> Option<MTLClearColor> {
    PASS.with(|p| {
        let mut p = p.borrow_mut();
        if let Some(pass) = p.as_mut().filter(|p| std::ptr::eq(&*p.target, texture)) {
            if color.is_some() {
                pass.color = color;
            }
            None // Load retained pixels only on this frame's partial target.
        } else {
            color
        }
    })
}
pub(crate) fn pass_scissor(
    texture: &ProtocolObject<dyn MTLTexture>,
    encoder: &ProtocolObject<dyn MTLRenderCommandEncoder>,
) {
    PASS.with(|p| {
        let mut p = p.borrow_mut();
        let Some(pass) = p.as_mut().filter(|p| std::ptr::eq(&*p.target, texture)) else {
            return;
        };
        pass.stats.encoders += 1;
        encoder.setScissorRect(pass.rect);
        if !pass.cleared {
            // The first main pass supplies its original clear color. Restarts
            // must load what earlier batches painted and never clear again.
            let color = pass
                .color
                .expect("partial main pass has its original clear color");
            if pass.magenta {
                pass.pipelines.clear_in_encoder(encoder, [1., 0., 1., 1.]);
                pass.stats.prefills += 1;
            }
            pass.pipelines.clear_in_encoder(
                encoder,
                [
                    color.red as f32,
                    color.green as f32,
                    color.blue as f32,
                    color.alpha as f32,
                ],
            );
            pass.cleared = true;
        }
    });
}
