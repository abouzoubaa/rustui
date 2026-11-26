// SPDX-License-Identifier: Apache-2.0
//! RustUI per-attempt damage CPU timing. Logging is opt-in with RUSTUI_FRAME_LOG.
use std::{
    sync::OnceLock,
    time::{Instant, SystemTime},
};
fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        matches!(
            std::env::var("RUSTUI_FRAME_LOG").as_deref(),
            Ok("1" | "buffered")
        )
    })
}
pub(crate) struct Measure {
    start: Option<Instant>,
    pub(crate) signatures_ms: f64,
    pub(crate) plan_ms: f64,
    pub(crate) hashed: usize,
    pub(crate) reused: usize,
    pub(crate) reason: &'static str,
}
impl Measure {
    pub(crate) fn new(reason: &'static str) -> Self {
        Self {
            start: enabled().then(Instant::now),
            signatures_ms: 0.,
            plan_ms: 0.,
            hashed: 0,
            reused: 0,
            reason,
        }
    }
}
impl Drop for Measure {
    fn drop(&mut self) {
        let Some(start) = self.start else { return };
        let total_ms = start.elapsed().as_secs_f64() * 1000.;
        let epoch_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.;
        super::log::line(format_args!(
            "RUSTUI_FRAME_LOG {{\"event\":\"damage_cpu\",\"cpu_ms\":0.0,\"signatures_ms\":{:.6},\"plan_ms\":{:.6},\"total_ms\":{total_ms:.6},\"hashed\":{},\"reused\":{},\"reason\":\"{}\",\"epoch_ms\":{epoch_ms:.6}}}",
            self.signatures_ms, self.plan_ms, self.hashed, self.reused, self.reason
        ));
    }
}
