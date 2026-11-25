// RustUI G5-L6 opt-in handler subphases. Serialize with the existing buffered recorder.
pub(super) struct Span(Option<(gpui::WindowId, &'static str, u64)>);

pub(super) fn span(window: gpui::WindowId, kind: &'static str) -> Span {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let enabled = *ENABLED
        .get_or_init(|| std::env::var_os("RUSTUI_G5_L6_TRACE").as_deref() == Some("1".as_ref()));
    Span(
        (enabled && gpui::fast_latency_enabled())
            .then(|| (window, kind, gpui::fast_latency_tick())),
    )
}

impl Drop for Span {
    fn drop(&mut self) {
        if let Some((window, kind, start)) = self.0 {
            let end = gpui::fast_latency_tick();
            gpui::fast_latency_config(window, kind, [start as f64, end as f64, 0., 0.]);
        }
    }
}
