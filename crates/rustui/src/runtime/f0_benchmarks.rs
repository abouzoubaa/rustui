//! F0 before/after workloads. These test-window measurements are comparative,
//! not native display refresh timings. Run three times with the F0 harness.

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use crate::{
    components::{
        advanced::CodeEditor,
        data::collection::{Collection, CollectionItem},
    },
    prelude::{Text, VStack},
    runtime::ComponentHost,
    view::View,
};
use gpui::AppContext as _;
use rustui_core::{Binding, Identity};

fn activate_test_theme(cx: &mut gpui::TestAppContext) {
    let theme = crate::theme::parse_theme(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../themes/macos27.json"
    )))
    .unwrap();
    cx.update(|app| {
        gpui_component::init(app);
        crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
            .unwrap();
    });
}

fn report(name: &str, values: &[u128]) {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let median = sorted[sorted.len() / 2];
    let p95 = sorted[((sorted.len() * 95).div_ceil(100)).saturating_sub(1)];
    println!(
        "F0_BENCH,{name},samples={},median_ns={median},p95_ns={p95}",
        values.len()
    );
}

struct ScrollOwner {
    items: std::rc::Rc<[CollectionItem<usize>]>,
    built_rows: Arc<AtomicUsize>,
}

impl View for ScrollOwner {
    fn body(&self) -> impl View {
        let built_rows = self.built_rows.clone();
        Collection::new(
            "f0-long-list",
            self.items.clone(),
            560.0,
            28.0,
            move |index, value| {
                built_rows.fetch_add(1, Ordering::Relaxed);
                Text(format!("row {index}: item {value}"))
            },
        )
        .label("F0 long-list benchmark")
    }
}

#[gpui::test]
#[ignore = "F0 comparative performance workload"]
fn f0_long_list_scroll_benchmark(cx: &mut gpui::TestAppContext) {
    use std::time::Instant;

    activate_test_theme(cx);
    let built_rows = Arc::new(AtomicUsize::new(0));
    let items = (0..5_000)
        .map(|index| CollectionItem::new(format!("row-{index}"), index))
        .collect::<Vec<_>>()
        .into();
    let (owner, window) = cx.add_window_view(|_, app| {
        ComponentHost::new(
            ScrollOwner {
                items,
                built_rows: built_rows.clone(),
            },
            app,
        )
    });
    window.update(|window, app| {
        let _ = window.draw(app);
    });
    let mut samples = Vec::with_capacity(300);
    let rows_before = built_rows.load(Ordering::Relaxed);
    for sample in 0..300 {
        let start = Instant::now();
        window.simulate_event(gpui::ScrollWheelEvent {
            position: gpui::point(gpui::px(200.), gpui::px(200.)),
            delta: gpui::ScrollDelta::Pixels(gpui::point(
                gpui::px(0.),
                gpui::px(if sample % 2 == 0 { -84. } else { 84. }),
            )),
            ..Default::default()
        });
        window.update(|window, app| {
            let _ = window.draw(app);
        });
        samples.push(start.elapsed().as_nanos());
    }
    let rows_built = built_rows.load(Ordering::Relaxed) - rows_before;
    report("long_list_scroll_frame", &samples);
    println!("F0_BENCH,long_list_rows_rebuilt,total={rows_built},frames=300");
    window.update(|_, app| assert_eq!(owner.read(app).component().items.len(), 5_000));
}

struct EditorOwner {
    value: String,
}

impl View for EditorOwner {
    fn body(&self) -> impl View {
        CodeEditor(
            "f0-large-editor",
            Binding::new(
                self.value.clone(),
                |owner: &Self| owner.value.clone(),
                |owner, value| owner.value = value,
            ),
        )
        .language("rust")
        .height(560.0)
    }
}

#[gpui::test]
#[ignore = "F0 comparative performance workload"]
fn f0_large_editor_typing_benchmark(cx: &mut gpui::TestAppContext) {
    use std::time::Instant;

    activate_test_theme(cx);
    let source = (0..10_000)
        .map(|line| format!("fn value_{line}() -> &'static str {{ \"text\" }}\n"))
        .collect::<String>();
    let expected_len = source.len() + 100;
    let (owner, window) =
        cx.add_window_view(|_, app| ComponentHost::new(EditorOwner { value: source }, app));
    window.update(|window, app| {
        let _ = window.draw(app);
    });
    window.simulate_click(
        gpui::point(gpui::px(100.), gpui::px(30.)),
        gpui::Modifiers::default(),
    );
    let mut samples = Vec::with_capacity(100);
    for _ in 0..100 {
        let start = Instant::now();
        window.simulate_input("x");
        window.update(|window, app| {
            let _ = window.draw(app);
        });
        samples.push(start.elapsed().as_nanos());
    }
    report("large_editor_typing_dispatch_draw", &samples);
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().value.len(), expected_len);
    });
}

struct IdleOwner {
    renders: Arc<AtomicUsize>,
    state: usize,
}

impl View for IdleOwner {
    fn body(&self) -> impl View {
        self.renders.fetch_add(1, Ordering::Relaxed);
        let mut views = VStack::new();
        for row in 0..60 {
            views = views.child(Text(format!("panel {row}: {}", self.state)));
        }
        views
    }
}

#[gpui::test]
#[ignore = "F0 comparative performance workload"]
fn f0_idle_redraw_invalidation_benchmark(cx: &mut gpui::TestAppContext) {
    use std::time::Instant;

    activate_test_theme(cx);
    let renders = Arc::new(AtomicUsize::new(0));
    let (owner, window) = cx.add_window_view(|_, app| {
        ComponentHost::new(
            IdleOwner {
                renders: renders.clone(),
                state: 0,
            },
            app,
        )
    });
    window.update(|window, app| {
        let _ = window.draw(app);
    });
    let mut samples = Vec::with_capacity(60);
    for _ in 0..60 {
        let start = Instant::now();
        window.update(|window, app| {
            let _ = window.draw(app);
        });
        samples.push(start.elapsed().as_nanos());
    }
    report("idle_redraw_60_panels", &samples);
    let idle_renders = renders.load(Ordering::Relaxed);
    owner.update(window, |host, cx| {
        host.component_mut().state = 1;
        cx.notify();
    });
    window.update(|window, app| {
        let _ = window.draw(app);
        assert_eq!(owner.read(app).component().state, 1);
    });
    println!(
        "F0_BENCH,idle_redraw_render_counts,idle_body_renders={idle_renders},after_change_body_renders={}",
        renders.load(Ordering::Relaxed)
    );
}
