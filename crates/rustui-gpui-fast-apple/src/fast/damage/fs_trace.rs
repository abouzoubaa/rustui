// SPDX-License-Identifier: Apache-2.0
//! Opt-in, buffered diagnostics for a native window's Metal draws.
use super::{
    frame::{Mode, Prepared, State},
    retained::Target,
};
use objc2::runtime::ProtocolObject;
use objc2_metal::MTLTexture;
use objc2_quartz_core::CAMetalLayer;
use std::{
    cell::Cell,
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Default, Copy, Clone)]
struct Details {
    cause: Option<&'static str>,
    allocated: bool,
    used: u64,
    requested: u64,
}
thread_local! { static DETAILS: Cell<Details> = Cell::new(Details::default()); }

fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("RUSTUI_G_FS1_TRACE").as_deref() == Ok("1") && super::ring::buffered()
    })
}
pub(super) fn denied(used: u64, requested: u64) {
    if enabled() {
        DETAILS.set(Details {
            used,
            requested,
            ..DETAILS.get()
        });
    }
}
pub(super) fn cause(reason: &'static str) {
    if enabled() {
        DETAILS.set(Details {
            cause: Some(reason),
            ..DETAILS.get()
        });
    }
}
pub(super) fn allocated() {
    if enabled() {
        DETAILS.set(Details {
            allocated: true,
            ..DETAILS.get()
        });
    }
}
#[derive(Clone, Copy, Default)]
struct Image {
    width: usize,
    height: usize,
    format: usize,
    valid: bool,
}
impl Image {
    fn texture(t: &ProtocolObject<dyn MTLTexture>, valid: bool) -> Self {
        Self {
            width: t.width(),
            height: t.height(),
            format: t.pixelFormat().0,
            valid,
        }
    }
    fn kept(t: Option<&Target>) -> Self {
        t.map(|t| Self::texture(&t.texture, t.valid))
            .unwrap_or_default()
    }
}

struct Row {
    index: u64,
    epoch_ms: f64,
    surface: u64,
    width: f64,
    height: f64,
    scale: f64,
    framebuffer_only: bool,
    drawable: Image,
    before: Image,
    after: Image,
    planned: &'static str,
    decision: &'static str,
    reason: &'static str,
    previous: Details,
}
pub(crate) struct Scope(Option<Row>);
impl Scope {
    pub(crate) fn start(layer: &CAMetalLayer, state: &State, surface: u64) -> Self {
        if !enabled() {
            return Self(None);
        }
        static INDEX: AtomicU64 = AtomicU64::new(1);
        let size = layer.drawableSize();
        let before = Image::kept(state.target.as_ref());
        Self(Some(Row {
            index: INDEX.fetch_add(1, Ordering::Relaxed),
            epoch_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64()
                * 1000.,
            surface,
            width: size.width,
            height: size.height,
            scale: layer.contentsScale(),
            framebuffer_only: layer.framebufferOnly(),
            drawable: Default::default(),
            before,
            after: before,
            planned: "none",
            decision: "deferred",
            reason: "admission",
            previous: DETAILS.replace(Details::default()),
        }))
    }
    pub(crate) fn planned(&mut self, plan: &Prepared, state: &State) {
        if let Some(r) = &mut self.0 {
            r.planned = plan.mode.label();
            r.after = Image::kept(state.target.as_ref());
            if plan.mode == Mode::Same {
                r.decision = "skip";
                r.reason = plan.reason;
            } else {
                r.reason = "acquisition_or_order";
            }
        }
    }
    pub(crate) fn acquired(&mut self, t: &ProtocolObject<dyn MTLTexture>) {
        if let Some(r) = &mut self.0 {
            r.drawable = Image::texture(t, true);
            r.reason = "render_error";
        }
    }
    pub(crate) fn submitted(&mut self, state: &State, planned: Mode, reason: &'static str) {
        if let Some(r) = &mut self.0 {
            r.after = Image::kept(state.target.as_ref());
            r.decision = state.last_mode.label();
            r.reason = if state.last_mode == planned {
                reason
            } else {
                "allocation_or_verify"
            };
        }
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        let Some(r) = self.0.take() else { return };
        let d = DETAILS.replace(r.previous);
        let a = r.after;
        let b = r.before;
        let t = r.drawable;
        super::log::line(format_args!(
            "G_FS1_FRAME {} {:.6} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            r.index,
            r.epoch_ms,
            r.surface,
            r.width,
            r.height,
            r.scale,
            r.framebuffer_only,
            t.width,
            t.height,
            t.format,
            b.width,
            b.height,
            b.format,
            b.valid,
            a.width,
            a.height,
            a.format,
            a.valid,
            d.allocated,
            r.planned,
            r.decision,
            d.cause.unwrap_or(r.reason),
            d.used,
            d.requested
        ));
    }
}
