//! Explicit snapshots of host lowering, not frame or GPU timings.

use rustui::{prelude::*, runtime::render_diagnostics::*};
use std::{cell::RefCell, rc::Rc};

pub fn demo() -> impl View {
    Component("render-timings-demo", RenderTimings::default)
}

#[component]
struct RenderTimings {
    snapshot: Rc<RefCell<String>>,
    #[state(0)]
    count: i32,
}

impl RenderTimings {
    fn increment(&mut self) {
        self.count += 1;
    }

    fn snapshot_action(&self) -> AppAction {
        let text = self.snapshot.clone();
        AppAction::new(move |cx| {
            let snapshot = render_diagnostics_snapshot(cx);
            *text.borrow_mut() = if snapshot.hosts.is_empty() {
                "No recorded host lowerings".into()
            } else {
                snapshot
                    .hosts
                    .iter()
                    .map(|host| {
                        format!(
                            "{}: {} lowerings, total {} µs, max {} µs",
                            host.component_type
                                .rsplit("::")
                                .next()
                                .unwrap_or(host.component_type),
                            host.count,
                            host.total.as_micros(),
                            host.max.as_micros(),
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
        })
        .then(AppAction::refresh_windows())
    }
}

impl View for RenderTimings {
    fn body(&self) -> impl View {
        let summary = self.snapshot.borrow().clone();
        VStack::new()
            .child(Text("Host lowering diagnostics"))
            .child(Text("Opt-in wall time for building GPUI elements. Excludes layout, paint, GPU and presentation."))
            .child(HStack::new()
                .child(Button("Enable timings", AppAction::new(enable_render_diagnostics)
                    .then(AppAction::refresh_windows())).id("enable-render-timings"))
                .child(Button("Disable timings", AppAction::new(disable_render_diagnostics)
                    .then(AppAction::refresh_windows())).id("disable-render-timings"))
                .child(Button("Clear timings", AppAction::new(clear_render_diagnostics)
                    .then(AppAction::refresh_windows())).id("clear-render-timings"))
                .spacing(6.0))
            .child(Button("Increment measured component", self.action(Self::increment))
                .id("increment-measured-component")
                .accessibility_label(format!("Measured component count: {}", self.count())))
            .child(Button("Read timing snapshot", self.snapshot_action())
                .id("snapshot-render-timings")
                .accessibility_label(format!("Read timing snapshot. {summary}")))
            .child(Text(summary))
            .spacing(8.0)
    }
}
