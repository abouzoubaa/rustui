//! Opt-in normal-input playback for the native scrolling comparison.
use rustui::gpui;
use rustui::view::{EmptyView, RenderContext, View};
use std::time::{Duration, Instant};

fn epoch_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

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
        eprintln!("RUSTUI_SCROLL_LOG start {panel} epoch_ms={}", epoch_ms());
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
            if std::env::var("RUSTUI_F0_PIXELS").as_deref() == Ok("1") {
                eprintln!("RUSTUI_SCROLL_STEP {} {}", epoch_ms(), delta);
            }
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
        eprintln!(
            "RUSTUI_SCROLL_LOG end {panel} events={events} epoch_ms={}",
            epoch_ms()
        );
    })
    .detach();
}

/// Transparent geometry probe used only by the separate pixel-validation pass.
pub(crate) fn measured<V: View>(panel: impl Into<String>, child: V) -> impl View {
    Measured {
        panel: panel.into(),
        child,
    }
}
struct Measured<V> {
    panel: String,
    child: V,
}
impl<V: View> View for Measured<V> {
    fn body(&self) -> impl View {
        EmptyView
    }
    fn into_element(&self, context: &mut RenderContext<'_>) -> gpui::AnyElement {
        use gpui::IntoElement as _;
        let child = self.child.into_element(context);
        if std::env::var("RUSTUI_F0_PIXELS").as_deref() != Ok("1") {
            return child;
        }
        Probe {
            panel: self.panel.clone(),
            child,
        }
        .into_any_element()
    }
}
struct Probe {
    panel: String,
    child: gpui::AnyElement,
}
impl gpui::IntoElement for Probe {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl gpui::Element for Probe {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<gpui::ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> (gpui::LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _: &mut (),
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) {
        let rect = bounds.intersect(&window.content_mask().bounds);
        let size = window.viewport_size();
        eprintln!(
            "RUSTUI_VIEWPORT_LOG {}",
            serde_json::json!({"panel":self.panel, "x":f32::from(rect.origin.x),"y":f32::from(rect.origin.y),"width":f32::from(rect.size.width),"height":f32::from(rect.size.height),"window_width":f32::from(size.width),"window_height":f32::from(size.height),"scale":window.scale_factor()})
        );
        self.child.prepaint(window, cx);
    }
    fn paint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        _: gpui::Bounds<gpui::Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) {
        self.child.paint(window, cx);
    }
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
