// Added by RustUI for Stage G.
//! Opt-in GPU time for the command buffer submitted by each present.

use block2::RcBlock;
use objc2::runtime::ProtocolObject;
use objc2_metal::MTLCommandBuffer;
use std::{ptr::NonNull, sync::OnceLock, time::SystemTime};

fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        matches!(
            std::env::var("RUSTUI_FRAME_LOG").as_deref(),
            Ok("1" | "buffered")
        )
    })
}

fn gpu_ms(start: f64, end: f64) -> Option<f64> {
    let elapsed = (end - start) * 1000.0;
    (start > 0.0 && end >= start && elapsed.is_finite()).then_some(elapsed)
}

/// Register before commit; Metal calls the handler after GPU timestamps settle.
pub(crate) fn on_commit(buffer: &ProtocolObject<dyn MTLCommandBuffer>, mode: &'static str) {
    if !enabled() {
        return;
    }
    let block = RcBlock::new(
        move |buffer: NonNull<ProtocolObject<dyn MTLCommandBuffer>>| {
            // SAFETY: Metal supplies the live command buffer for this completion.
            // The vendored SDK declares both selectors as CFTimeInterval (f64).
            // Use objc2 dispatch because the generated accessors are gated by the
            // currently unused objc2-metal/objc2-core-foundation binding feature.
            let (start, end): (f64, f64) = unsafe {
                let buffer = buffer.as_ref();
                (
                    objc2::msg_send![buffer, GPUStartTime],
                    objc2::msg_send![buffer, GPUEndTime],
                )
            };
            let Some(gpu_ms) = gpu_ms(start, end) else {
                return;
            };
            let epoch_ms = std::time::SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64()
                * 1000.0;
            // GPU records contribute no CPU phase time to legacy phase readers.
            super::log::line(format_args!(
                "RUSTUI_FRAME_LOG {{\"event\":\"gpu\",\"mode\":\"{mode}\",\"cpu_ms\":0.0,\"gpu_ms\":{gpu_ms:.6},\"epoch_ms\":{epoch_ms:.6}}}"
            ));
        },
    );
    // SAFETY: the block is live here; Metal copies it until completion. Its
    // only capture is a static mode label and it owns no renderer resources.
    unsafe { buffer.addCompletedHandler(RcBlock::as_ptr(&block)) };
}

pub(crate) fn skipped() {
    if !enabled() {
        return;
    }
    let epoch_ms = std::time::SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * 1000.;
    super::log::line(format_args!(
        "RUSTUI_FRAME_LOG {{\"event\":\"gpu\",\"mode\":\"skip\",\"cpu_ms\":0.0,\"gpu_ms\":null,\"epoch_ms\":{epoch_ms:.6}}}"
    ));
}

#[cfg(test)]
mod tests {
    use super::gpu_ms;

    #[test]
    fn gpu_seconds_become_milliseconds_and_unavailable_times_are_rejected() {
        assert!((gpu_ms(10.0, 10.00125).unwrap() - 1.25).abs() < 1e-9);
        assert_eq!(gpu_ms(1.0, 1.0), Some(0.0));
        for (start, end) in [
            (0.0, 1.0),
            (2.0, 1.0),
            (1.0, f64::NAN),
            (1.0, f64::INFINITY),
        ] {
            assert_eq!(gpu_ms(start, end), None);
        }
    }
}

/// Verify is explicitly requested, so a mismatch is logged even without frame timing.
pub(crate) fn mismatch(mode: &'static str, x: usize, y: usize) {
    let epoch_ms = std::time::SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * 1000.;
    super::log::line(format_args!(
        "RUSTUI_FRAME_LOG {{\"event\":\"damage_mismatch\",\"mode\":\"{mode}\",\"x\":{x},\"y\":{y},\"cpu_ms\":0.0,\"epoch_ms\":{epoch_ms:.6}}}"
    ));
}

pub(crate) fn verified(mode: &'static str) {
    if !enabled() {
        return;
    }
    let epoch_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * 1000.;
    super::log::line(format_args!(
        "RUSTUI_FRAME_LOG {{\"event\":\"damage_verify\",\"mode\":\"{mode}\",\"cpu_ms\":0.0,\"epoch_ms\":{epoch_ms:.6}}}"
    ));
}

pub(super) fn declared_diff(rectangles: usize) {
    if enabled() {
        super::log::line(format_args!(
            "RUSTUI_FRAME_LOG {{\"event\":\"declared_full_diff\",\"rectangles\":{rectangles}}}"
        ));
    }
}
