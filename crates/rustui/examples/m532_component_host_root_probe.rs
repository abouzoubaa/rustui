//! Native probe for ComponentHost lowering a primitive root through `View::into_element`.
//!
//! Run with `./scripts/cargo-headstart run -p rustui --example m532_component_host_root_probe`.
//! Exits successfully after the custom root renderer runs. Calling `body()` or
//! failing to render within five seconds exits with a failing process code.

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use gpui::{InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, div};
use rustui::{
    RustUI,
    prelude::{Text, View},
    view::RenderContext,
};

struct PrimitiveRoot {
    lowered: Arc<AtomicBool>,
}

impl View for PrimitiveRoot {
    fn body(&self) -> impl View {
        eprintln!("M532 FAIL: ComponentHost expanded the primitive root through body()");
        std::process::exit(1);
        #[allow(unreachable_code)]
        Text("M532 BODY FALLBACK")
    }

    fn into_element(&self, _: &mut RenderContext<'_>) -> gpui::AnyElement {
        self.lowered.store(true, Ordering::SeqCst);
        div()
            .id("m532-primitive-root")
            .aria_label("M532 custom root lowering")
            .child("M532 CUSTOM ROOT LOWERING")
            .into_any_element()
    }
}

fn main() {
    let lowered = Arc::new(AtomicBool::new(false));
    let observed = lowered.clone();
    let passed = Arc::new(AtomicBool::new(false));
    let completed = passed.clone();
    RustUI::new()
        .title("M532 ComponentHost Root Probe")
        .window_size(640.0, 240.0)
        .on_launch(move |cx| {
            cx.spawn(async move |cx| {
                cx.background_executor().timer(Duration::from_secs(5)).await;
                if !observed.load(Ordering::SeqCst) {
                    eprintln!("M532 FAIL: custom root renderer did not run within five seconds");
                    std::process::exit(1);
                }
                completed.store(true, Ordering::SeqCst);
                eprintln!("M532 PASS: ComponentHost invoked the custom root renderer");
                cx.update(|cx| cx.quit());
            })
            .detach();
        })
        .run(PrimitiveRoot { lowered });
    if !passed.load(Ordering::SeqCst) {
        eprintln!("M532 FAIL: application exited before the root lowering check completed");
        std::process::exit(1);
    }
}
