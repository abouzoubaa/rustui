//! Opt-in normal-input playback for the native scrolling comparison.
use rustui::gpui::{self, AppContext as _};
use std::time::{Duration, Instant};

fn scroll_distance(panel: &str) -> f32 {
    match panel {
        "editor" => 300_000.0,
        "grid" => 36_000.0,
        _ => 150_000.0,
    }
}

/// Keep the cursor in the workload, below the page controls and beside the sidebar.
fn scroll_position(size: gpui::Size<gpui::Pixels>) -> gpui::Point<gpui::Pixels> {
    gpui::point(size.width * 0.75, size.height * 0.65)
}

pub(crate) fn start(app: &mut gpui::App) {
    if std::env::var("RUSTUI_F0_SCROLL").as_deref() != Ok("1") {
        return;
    }
    let panel = std::env::var("RUSTUI_F0_PANEL").unwrap_or_else(|_| "collection".into());
    app.spawn(async move |cx| {
        // Let initial layout, syntax parsing and the platform's frame source settle.
        cx.background_executor().timer(Duration::from_secs(2)).await;
        let Some(window) = cx.update(|app| app.windows().first().copied()) else {
            return;
        };
        eprintln!("RUSTUI_SCROLL_LOG start {panel}");
        let start = Instant::now();
        let mut previous = start;
        let mut events = 0;
        while start.elapsed() < Duration::from_secs(5) {
            let now = Instant::now();
            let delta = scroll_distance(&panel) * now.duration_since(previous).as_secs_f32() / 2.5;
            let delta = if start.elapsed().as_secs_f32() < 2.5 {
                -delta
            } else {
                delta
            };
            previous = now;
            // Dispatch the same input path as a wheel/trackpad; never force a redraw.
            let result = window.update(cx, |_, window, app| {
                #[allow(deprecated)]
                let event = gpui::ScrollWheelEvent {
                    position: scroll_position(window.viewport_size()),
                    delta: gpui::ScrollDelta::Pixels(gpui::point(gpui::px(0.0), gpui::px(delta))),
                    touch_phase: if events == 0 {
                        gpui::TouchPhase::Started
                    } else {
                        gpui::TouchPhase::Moved
                    },
                    ..Default::default()
                };
                window.dispatch_event(gpui::PlatformInput::Scroll(event), app);
            });
            if result.is_err() {
                break;
            }
            events += 1;
            cx.background_executor()
                .timer(Duration::from_nanos(8_333_333))
                .await;
        }
        let _ = window.update(cx, |_, window, app| {
            #[allow(deprecated)]
            let event = gpui::ScrollWheelEvent {
                position: scroll_position(window.viewport_size()),
                touch_phase: gpui::TouchPhase::Ended,
                ..Default::default()
            };
            window.dispatch_event(gpui::PlatformInput::Scroll(event), app);
        });
        eprintln!("RUSTUI_SCROLL_LOG end {panel} events={events}");
    })
    .detach();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_uses_workload_coordinates_and_symmetric_distances() {
        let position = scroll_position(gpui::size(gpui::px(1400.0), gpui::px(1000.0)));
        assert_eq!(position, gpui::point(gpui::px(1050.0), gpui::px(650.0)));
        for panel in ["collection", "list", "keyed", "editor", "grid"] {
            assert!(scroll_distance(panel) > 30_000.0);
        }
    }
}
