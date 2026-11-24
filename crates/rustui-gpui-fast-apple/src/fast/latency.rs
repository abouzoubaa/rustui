// RustUI PX-4: observe the existing Metal path without changing scheduling.
use block2::RcBlock;
use objc2::{Encode, Encoding, runtime::ProtocolObject};
use objc2_metal::{MTLCommandBuffer, MTLDrawable};
use objc2_quartz_core::{CAMetalDisplayLink, CAMetalDrawable, CAMetalLayer};
use std::ptr::NonNull;
#[link(name = "QuartzCore", kind = "framework")]
unsafe extern "C-unwind" {
    fn CACurrentMediaTime() -> f64;
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Range {
    minimum: f32,
    maximum: f32,
    preferred: f32,
}
// Exact CAFrameRateRange layout/encoding from the pinned QuartzCore binding.
unsafe impl Encode for Range {
    const ENCODING: Encoding = Encoding::Struct(
        "CAFrameRateRange",
        &[Encoding::Float, Encoding::Float, Encoding::Float],
    );
}
pub(crate) fn link(window: gpui::WindowId, link: &CAMetalDisplayLink) {
    if !gpui::fast_latency_enabled() {
        return;
    }
    let range: Range = unsafe { objc2::msg_send![link, preferredFrameRateRange] };
    gpui::fast_latency_config(
        window,
        "display_link",
        [
            link.preferredFrameLatency() as f64,
            range.minimum as f64,
            range.maximum as f64,
            range.preferred as f64,
        ],
    );
}
pub(crate) fn layer(layer: &CAMetalLayer, synchronous: bool) {
    let Some(frame) = gpui::fast_latency_frame() else {
        return;
    };
    // Preserve changes after startup/live resize; never change the settings.
    static SEEN: std::sync::Mutex<Vec<(gpui::WindowId, [f64; 4])>> =
        std::sync::Mutex::new(Vec::new());
    let values = [
        layer.maximumDrawableCount() as f64,
        layer.displaySyncEnabled() as u8 as f64,
        layer.presentsWithTransaction() as u8 as f64,
        synchronous as u8 as f64,
    ];
    let mut seen = SEEN.lock().unwrap();
    if let Some(old) = seen.iter_mut().find(|item| item.0 == frame.window()) {
        if old.1 == values {
            return;
        }
        old.1 = values;
    } else {
        seen.push((frame.window(), values));
    }
    gpui::fast_latency_config(frame.window(), "metal_layer", values);
}
pub fn latency_acquired(window: gpui::WindowId, drawable: &ProtocolObject<dyn CAMetalDrawable>) {
    if !gpui::fast_latency_enabled() {
        return;
    }
    let key = drawable as *const _ as *const () as usize as u64;
    let Some(ticket) = gpui::fast_latency_acquired(window, key) else {
        return;
    };
    let block = RcBlock::new(move |d: NonNull<ProtocolObject<dyn MTLDrawable>>| {
        let displayed: f64 = unsafe { objc2::msg_send![d.as_ref(), presentedTime] };
        gpui::fast_latency_displayed(ticket, displayed, unsafe { CACurrentMediaTime() });
    });
    unsafe {
        drawable.addPresentedHandler(RcBlock::as_ptr(&block));
    }
}
pub(crate) fn observe(
    command: &ProtocolObject<dyn MTLCommandBuffer>,
    drawable: &ProtocolObject<dyn CAMetalDrawable>,
) -> Option<gpui::FastLatencyCommit> {
    let frame = gpui::fast_latency_frame()?;
    let key = drawable as *const _ as *const () as usize as u64;
    let token = gpui::fast_latency_command(frame, key)?;
    let block = RcBlock::new(move |b: NonNull<ProtocolObject<dyn MTLCommandBuffer>>| {
        let end: f64 = unsafe { objc2::msg_send![b.as_ref(), GPUEndTime] };
        let start: f64 = unsafe { objc2::msg_send![b.as_ref(), GPUStartTime] };
        gpui::fast_latency_gpu_interval(token, start, end);
    });
    unsafe {
        command.addCompletedHandler(RcBlock::as_ptr(&block));
    }
    Some(token)
}

// A detached drawable is observed immediately when nextDrawable returns. If
// encoding declines/errors before submission, retire its unpresented lifetime.
pub(crate) struct HeldDrawable(Option<(gpui::WindowId, u64)>);
impl HeldDrawable {
    pub(crate) fn submitted(&mut self) {
        self.0 = None;
    }
}
impl Drop for HeldDrawable {
    fn drop(&mut self) {
        if let Some((window, key)) = self.0 {
            gpui::fast_latency_released(window, key);
        }
    }
}
pub(crate) fn hold(drawable: &ProtocolObject<dyn CAMetalDrawable>) -> HeldDrawable {
    let Some(frame) = gpui::fast_latency_frame() else {
        return HeldDrawable(None);
    };
    latency_acquired(frame.window(), drawable);
    HeldDrawable(Some((
        frame.window(),
        drawable as *const _ as *const () as usize as u64,
    )))
}
