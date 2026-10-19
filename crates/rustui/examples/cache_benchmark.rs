//! Opt-in, frame-stepped validation of RustUI's fixed-bounds child cache.
//!
//! Run from the repository root with:
//! `./scripts/cargo-headstart run -p rustui --example cache_benchmark`

use std::{
    cell::{Cell, RefCell},
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    rc::Rc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use gpui::{
    AnyElement, App, AppContext as _, Bounds, Entity, Window, WindowBounds, WindowOptions, px, size,
};
use rustui::{
    localization::{
        Catalog, Catalogs, Locale, activate_localization, active_localization, set_locale,
    },
    prelude::{
        AnyView, Component, Container, Environment, HStack, Surface, Text, VStack, View, ViewHost,
    },
    runtime::render_diagnostics::{
        RenderDiagnosticsSnapshot, enable_render_diagnostics, render_diagnostics_snapshot,
    },
    theme::{AppearanceMode, ThemeDocument, activate_theme, parse_theme},
    view::{EmptyView, RenderContext},
};
use serde_json::{Value, json};

const WARMUP_UPDATES: u32 = 10;
const MEASURED_UPDATES: u32 = 100;
const TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    retained_children_per_side: usize,
}

const SCENARIOS: [Scenario; 3] = [
    Scenario {
        name: "small",
        retained_children_per_side: 1,
    },
    Scenario {
        name: "medium",
        retained_children_per_side: 8,
    },
    Scenario {
        name: "large",
        retained_children_per_side: 32,
    },
];

fn scenario_from_args() -> Result<Scenario, String> {
    let mut args = std::env::args().skip(1);
    let mut selected = None;
    while let Some(argument) = args.next() {
        if argument == "--scenario" {
            selected = Some(args.next().ok_or("--scenario needs a value")?);
        } else if let Some(value) = argument.strip_prefix("--scenario=") {
            selected = Some(value.to_owned());
        } else {
            return Err(format!("unknown argument: {argument}"));
        }
    }
    let selected = selected.as_deref().unwrap_or("small");
    SCENARIOS
        .iter()
        .copied()
        .find(|scenario| scenario.name == selected)
        .ok_or_else(|| format!("unknown scenario {selected:?}; choose small, medium, or large"))
}

struct BenchmarkRoot {
    tick: u32,
    scenario: Scenario,
    cached_output: Rc<Cell<u32>>,
    uncached_output: Rc<Cell<u32>>,
}

impl View for BenchmarkRoot {
    fn body(&self) -> impl View {
        let cached_output = self.cached_output.clone();
        let uncached_output = self.uncached_output.clone();
        VStack::new()
            .child(Text(format!("Parent update {:06}", self.tick)))
            .child(
                HStack::new()
                    .children((0..self.scenario.retained_children_per_side).map({
                        let output = cached_output.clone();
                        move |index| {
                            let output = output.clone();
                            Component(format!("cached-leaf-{index}"), move || CachedLeaf {
                                value: 0,
                                output: output.clone(),
                            })
                            .cached_fixed(220.0, 100.0)
                        }
                    }))
                    .children((0..self.scenario.retained_children_per_side).map({
                        let output = uncached_output.clone();
                        move |index| {
                            let output = output.clone();
                            Component(format!("uncached-leaf-{index}"), move || UncachedLeaf {
                                value: 0,
                                output: output.clone(),
                            })
                        }
                    })),
            )
    }
}

struct CachedLeaf {
    value: u32,
    output: Rc<Cell<u32>>,
}

impl View for CachedLeaf {
    fn body(&self) -> impl View {
        self.output.set(self.value);
        Container(Text(format!("Cached value: {}", self.value)))
            .width(220.0)
            .height(100.0)
    }
}

struct UncachedLeaf {
    value: u32,
    output: Rc<Cell<u32>>,
}

impl View for UncachedLeaf {
    fn body(&self) -> impl View {
        self.output.set(self.value);
        Container(Text(format!("Uncached value: {}", self.value)))
            .width(220.0)
            .height(100.0)
    }
}

#[derive(Clone, Copy)]
enum Phase {
    Warmup(u32),
    Measure(u32),
    CachedMutation,
    UncachedMutation,
    Finished,
}

impl Phase {
    fn label(self) -> &'static str {
        match self {
            Self::Warmup(_) => "warmup",
            Self::Measure(_) => "measure",
            Self::CachedMutation => "cached_mutation",
            Self::UncachedMutation => "uncached_mutation",
            Self::Finished => "finished",
        }
    }
}

struct State {
    phase: Phase,
    root: Entity<ViewHost<BenchmarkRoot>>,
    scenario: Scenario,
    cached: Option<gpui::WeakEntity<ViewHost<CachedLeaf>>>,
    uncached: Option<gpui::WeakEntity<ViewHost<UncachedLeaf>>>,
    cached_output: Rc<Cell<u32>>,
    uncached_output: Rc<Cell<u32>>,
    parent_snapshot: Option<RenderDiagnosticsSnapshot>,
    cached_mutation_snapshot: Option<RenderDiagnosticsSnapshot>,
    ax_active_seen: bool,
}

fn count(snapshot: &RenderDiagnosticsSnapshot, name: &str) -> u64 {
    snapshot
        .hosts
        .iter()
        .find(|host| host.component_type == name)
        .map_or(0, |host| host.count)
}

fn schedule(window: &Window, state: Rc<RefCell<State>>) {
    window.on_next_frame(move |window, cx| advance(window, cx, state));
}

fn tick_root(root: &Entity<ViewHost<BenchmarkRoot>>, cx: &mut App) {
    root.update(cx, |host, cx| {
        host.component_mut().tick += 1;
        cx.notify();
    });
}

fn advance(window: &mut Window, cx: &mut App, state: Rc<RefCell<State>>) {
    if window.is_a11y_active() {
        state.borrow_mut().ax_active_seen = true;
        finish(cx, state, "excluded_ax_active", None);
        return;
    }

    let phase = state.borrow().phase;
    match phase {
        Phase::Warmup(completed) if completed < WARMUP_UPDATES => {
            let root = state.borrow().root.clone();
            tick_root(&root, cx);
            state.borrow_mut().phase = Phase::Warmup(completed + 1);
            schedule(window, state);
        }
        Phase::Warmup(_) => {
            enable_render_diagnostics(cx);
            state.borrow_mut().phase = Phase::Measure(0);
            advance(window, cx, state);
        }
        Phase::Measure(completed) if completed < MEASURED_UPDATES => {
            let root = state.borrow().root.clone();
            tick_root(&root, cx);
            state.borrow_mut().phase = Phase::Measure(completed + 1);
            schedule(window, state);
        }
        Phase::Measure(_) => {
            let snapshot = render_diagnostics_snapshot(cx);
            let cached = state.borrow().cached.clone();
            state.borrow_mut().parent_snapshot = Some(snapshot);
            match cached.and_then(|child| {
                child
                    .update(cx, |host, cx| {
                        host.component_mut().value += 1;
                        cx.notify();
                    })
                    .ok()
            }) {
                Some(()) => {
                    state.borrow_mut().phase = Phase::CachedMutation;
                    schedule(window, state);
                }
                None => finish(cx, state, "failed", Some("cached child host missing")),
            }
        }
        Phase::CachedMutation => {
            let snapshot = render_diagnostics_snapshot(cx);
            let uncached = state.borrow().uncached.clone();
            state.borrow_mut().cached_mutation_snapshot = Some(snapshot);
            match uncached.and_then(|child| {
                child
                    .update(cx, |host, cx| {
                        host.component_mut().value += 1;
                        cx.notify();
                    })
                    .ok()
            }) {
                Some(()) => {
                    state.borrow_mut().phase = Phase::UncachedMutation;
                    schedule(window, state);
                }
                None => finish(cx, state, "failed", Some("uncached child host missing")),
            }
        }
        Phase::UncachedMutation => finish(cx, state, "complete", None),
        Phase::Finished => {}
    }
}

fn finish(cx: &mut App, state: Rc<RefCell<State>>, status: &str, error: Option<&str>) {
    let mut state = state.borrow_mut();
    if matches!(state.phase, Phase::Finished) {
        return;
    }
    let phase_at_finish = state.phase.label();
    state.phase = Phase::Finished;

    let final_snapshot = render_diagnostics_snapshot(cx);
    let cached_type = std::any::type_name::<CachedLeaf>();
    let uncached_type = std::any::type_name::<UncachedLeaf>();
    let parent_cached = state
        .parent_snapshot
        .as_ref()
        .map(|snapshot| count(snapshot, cached_type));
    let parent_uncached = state
        .parent_snapshot
        .as_ref()
        .map(|snapshot| count(snapshot, uncached_type));
    let after_cached_mutation = state
        .cached_mutation_snapshot
        .as_ref()
        .map(|snapshot| count(snapshot, cached_type));
    let after_uncached_mutation = count(&final_snapshot, uncached_type);
    let expected_uncached =
        state.scenario.retained_children_per_side as u64 * u64::from(MEASURED_UPDATES);
    let cache_hit_observed = parent_cached == Some(0) && parent_uncached == Some(expected_uncached);
    let child_invalidation_observed = parent_cached
        .zip(after_cached_mutation)
        .is_some_and(|(before, after)| after > before)
        && parent_uncached.is_some_and(|before| after_uncached_mutation > before)
        && state.cached_output.get() == 1
        && state.uncached_output.get() == 1;

    let result = json!({
        "schema_version": 1,
        "kind": "rustui_cached_child_workload",
        "status": status,
        "error": error,
        "phase_at_finish": phase_at_finish,
        "scenario": state.scenario.name,
        "retained_children_per_side": state.scenario.retained_children_per_side,
        "ax_active_seen": state.ax_active_seen,
        "warmup_parent_updates": WARMUP_UPDATES,
        "measured_parent_updates": MEASURED_UPDATES,
        "timeout_ms": TIMEOUT.as_millis(),
        "counts": {
            "cached_after_parent_updates": parent_cached,
            "uncached_after_parent_updates": parent_uncached,
            "cached_after_cached_mutation": after_cached_mutation,
            "cached_final": count(&final_snapshot, cached_type),
            "uncached_final": after_uncached_mutation,
        },
        "rendered_values": {
            "cached": state.cached_output.get(),
            "uncached": state.uncached_output.get(),
        },
        "cache_hit_observed": cache_hit_observed,
        "child_invalidation_observed": child_invalidation_observed,
        "measurement_scope": "RustUI host lowering counts only; not frame, GPU, or presentation time",
    });
    drop(state);

    if let Err(error) = write_result(&result) {
        eprintln!("cache benchmark could not write result: {error}");
    }
    cx.quit();
}

fn write_result(result: &Value) -> std::io::Result<()> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/headstart");
    fs::create_dir_all(&dir)?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = dir.join(format!(
        "cache-benchmark-{}-{timestamp}.json",
        std::process::id()
    ));
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    serde_json::to_writer_pretty(&mut output, result).map_err(std::io::Error::other)?;
    output.write_all(b"\n")?;
    println!("{}", path.display());
    Ok(())
}

#[derive(Clone)]
struct ProbeEnvironment(&'static str);

struct InvalidationRoot {
    scoped: bool,
    present: bool,
    output: Rc<RefCell<String>>,
    mounts: Rc<Cell<u32>>,
    unmounts: Rc<Cell<u32>>,
}

impl View for InvalidationRoot {
    fn body(&self) -> impl View {
        let child = || {
            Component("invalidation-leaf", {
                let output = self.output.clone();
                let mounts = self.mounts.clone();
                let unmounts = self.unmounts.clone();
                move || ProbeLeaf {
                    output: output.clone(),
                    mounts: mounts.clone(),
                    unmounts: unmounts.clone(),
                }
            })
            .cached_fixed(220.0, 100.0)
        };
        if !self.present {
            AnyView::new(EmptyView)
        } else if self.scoped {
            AnyView::new(Environment(ProbeEnvironment("scoped"), child()))
        } else {
            AnyView::new(child())
        }
    }
}

struct ProbeLeaf {
    output: Rc<RefCell<String>>,
    mounts: Rc<Cell<u32>>,
    unmounts: Rc<Cell<u32>>,
}

impl View for ProbeLeaf {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn on_mount(&mut self, _: &mut App) {
        self.mounts.set(self.mounts.get() + 1);
    }

    fn on_unmount(&mut self, _: &mut App) {
        self.unmounts.set(self.unmounts.get() + 1);
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let environment = context
            .environment::<ProbeEnvironment>()
            .map_or("absent", |value| value.0);
        let message = context.localized_message("cache-probe");
        *self.output.borrow_mut() = format!("{environment}:{message}");
        Container(Text(format!("{environment}: {message}")))
            .width(220.0)
            .height(100.0)
            .background(Surface::Background)
            .into_element(context)
    }
}

#[derive(Clone, Copy)]
enum InvalidationStep {
    Scope,
    Unscope,
    Remove,
    Restore,
    Locale,
    Dark,
    Light,
    Done,
}

impl InvalidationStep {
    fn label(self) -> &'static str {
        match self {
            Self::Scope => "environment_scope",
            Self::Unscope => "environment_fallback",
            Self::Remove => "removal",
            Self::Restore => "restoration",
            Self::Locale => "locale_change",
            Self::Dark => "theme_dark",
            Self::Light => "theme_light",
            Self::Done => "done",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Scope => Self::Unscope,
            Self::Unscope => Self::Remove,
            Self::Remove => Self::Restore,
            Self::Restore => Self::Locale,
            Self::Locale => Self::Dark,
            Self::Dark => Self::Light,
            Self::Light | Self::Done => Self::Done,
        }
    }
}

struct InvalidationState {
    root: Entity<ViewHost<InvalidationRoot>>,
    child: Option<gpui::WeakEntity<ViewHost<ProbeLeaf>>>,
    output: Rc<RefCell<String>>,
    mounts: Rc<Cell<u32>>,
    unmounts: Rc<Cell<u32>>,
    theme: ThemeDocument,
    step: InvalidationStep,
    sampling: bool,
    prior_count: u64,
    scenarios: Vec<Value>,
    ax_active_seen: bool,
    finished: bool,
}

fn schedule_invalidation(window: &Window, state: Rc<RefCell<InvalidationState>>) {
    window.on_next_frame(move |window, cx| advance_invalidation(window, cx, state));
}

fn advance_invalidation(window: &mut Window, cx: &mut App, state: Rc<RefCell<InvalidationState>>) {
    if window.is_a11y_active() {
        state.borrow_mut().ax_active_seen = true;
        finish_invalidation(cx, state, "excluded_ax_active", None);
        return;
    }
    let step = state.borrow().step;
    if matches!(step, InvalidationStep::Done) {
        finish_invalidation(cx, state, "complete", None);
        return;
    }
    if state.borrow().sampling {
        let mut state_ref = state.borrow_mut();
        let current = count(
            &render_diagnostics_snapshot(cx),
            std::any::type_name::<ProbeLeaf>(),
        );
        let child_alive = state_ref
            .child
            .as_ref()
            .is_some_and(|child| child.upgrade().is_some());
        let record = json!({
            "scenario": step.label(),
            "child_lowerings_since_action": current.saturating_sub(state_ref.prior_count),
            "child_lowerings_total": current,
            "child_alive": child_alive,
            "mounts": state_ref.mounts.get(),
            "unmounts": state_ref.unmounts.get(),
            "logical_output": state_ref.output.borrow().as_str(),
            "locale": active_localization(cx).map(|active| active.locale().as_str()),
            "locale_generation": active_localization(cx).map(|active| active.generation()),
            "theme_background": format!("{:?}", gpui_component::Theme::global(cx).colors.background),
        });
        state_ref.scenarios.push(record);
        state_ref.step = step.next();
        state_ref.sampling = false;
        drop(state_ref);
        schedule_invalidation(window, state);
        return;
    }
    let (root, theme) = {
        let state_ref = state.borrow();
        (state_ref.root.clone(), state_ref.theme.clone())
    };
    state.borrow_mut().prior_count = count(
        &render_diagnostics_snapshot(cx),
        std::any::type_name::<ProbeLeaf>(),
    );
    match step {
        InvalidationStep::Scope | InvalidationStep::Unscope => {
            root.update(cx, |host, cx| {
                host.component_mut().scoped = matches!(step, InvalidationStep::Scope);
                cx.notify();
            });
        }
        InvalidationStep::Remove | InvalidationStep::Restore => {
            root.update(cx, |host, cx| {
                host.component_mut().present = matches!(step, InvalidationStep::Restore);
                cx.notify();
            });
        }
        InvalidationStep::Locale => set_locale(Locale::parse("fr").expect("valid locale"), cx),
        InvalidationStep::Dark | InvalidationStep::Light => {
            let mode = if matches!(step, InvalidationStep::Dark) {
                AppearanceMode::Dark
            } else {
                AppearanceMode::Light
            };
            if let Err(error) = activate_theme(&theme, Some(mode), cx) {
                finish_invalidation(cx, state, "failed", Some(error.to_string()));
                return;
            }
        }
        InvalidationStep::Done => unreachable!(),
    }
    state.borrow_mut().sampling = true;
    schedule_invalidation(window, state);
}

fn finish_invalidation(
    cx: &mut App,
    state: Rc<RefCell<InvalidationState>>,
    status: &str,
    error: Option<String>,
) {
    let mut state = state.borrow_mut();
    if state.finished {
        return;
    }
    state.finished = true;
    let result = json!({
        "schema_version": 1,
        "kind": "rustui_cached_child_invalidation",
        "status": status,
        "error": error,
        "phase_at_finish": state.step.label(),
        "ax_active_seen": state.ax_active_seen,
        "timeout_ms": TIMEOUT.as_millis(),
        "scenarios": state.scenarios,
        "measurement_scope": "RustUI host lowering and logical output; no frame, GPU, pixel, or visual timing claim",
    });
    drop(state);
    if let Err(error) = write_result(&result) {
        eprintln!("cache invalidation benchmark could not write result: {error}");
    }
    cx.quit();
}

fn run_invalidation() {
    let theme = parse_theme(include_str!("../../../themes/macos27.json"))
        .expect("bundled theme must parse");
    gpui_platform::application().run(move |cx| {
        gpui_component::init(cx);
        let english = Locale::parse("en").expect("valid locale");
        let french = Locale::parse("fr").expect("valid locale");
        let mut catalogs = Catalogs::new(english.clone());
        catalogs
            .insert(Catalog::parse(english.clone(), "cache-probe = English").expect("catalog"))
            .expect("unique catalog");
        catalogs
            .insert(Catalog::parse(french, "cache-probe = Francais").expect("catalog"))
            .expect("unique catalog");
        activate_localization(catalogs, english, cx);
        activate_theme(&theme, Some(AppearanceMode::Light), cx)
            .expect("bundled light theme must activate");
        enable_render_diagnostics(cx);

        let output = Rc::new(RefCell::new(String::new()));
        let mounts = Rc::new(Cell::new(0));
        let unmounts = Rc::new(Cell::new(0));
        let root = cx.new(|cx| {
            ViewHost::new(
                InvalidationRoot {
                    scoped: false,
                    present: true,
                    output: output.clone(),
                    mounts: mounts.clone(),
                    unmounts: unmounts.clone(),
                },
                cx,
            )
        });
        let state = Rc::new(RefCell::new(InvalidationState {
            root: root.clone(),
            child: None,
            output,
            mounts,
            unmounts,
            theme,
            step: InvalidationStep::Scope,
            sampling: false,
            prior_count: 0,
            scenarios: Vec::new(),
            ax_active_seen: false,
            finished: false,
        }));
        let observed = state.clone();
        cx.observe_new::<ViewHost<ProbeLeaf>>(move |_, _, cx| {
            observed.borrow_mut().child = Some(cx.weak_entity());
        })
        .detach();
        let window_state = state.clone();
        let bounds = Bounds::centered(None, size(px(300.0), px(150.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |window, _| {
                schedule_invalidation(window, window_state);
                root.clone()
            },
        )
        .expect("cache invalidation window must open");
        cx.spawn(async move |cx| {
            cx.background_executor().timer(TIMEOUT).await;
            cx.update(|cx| {
                finish_invalidation(cx, state, "timeout", Some("no frame progress".into()))
            });
        })
        .detach();
        cx.activate(true);
    });
}

fn main() {
    if std::env::args().any(|argument| argument == "--invalidation") {
        run_invalidation();
        return;
    }
    let scenario = match scenario_from_args() {
        Ok(scenario) => scenario,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    let theme = parse_theme(include_str!("../../../themes/macos27.json"))
        .expect("bundled theme must parse");
    gpui_platform::application().run(move |cx| {
        gpui_component::init(cx);
        activate_theme(&theme, Some(AppearanceMode::Light), cx)
            .expect("bundled light theme must activate");

        let cached_output = Rc::new(Cell::new(0));
        let uncached_output = Rc::new(Cell::new(0));
        let root = cx.new(|cx| {
            ViewHost::new(
                BenchmarkRoot {
                    tick: 0,
                    scenario,
                    cached_output: cached_output.clone(),
                    uncached_output: uncached_output.clone(),
                },
                cx,
            )
        });
        let state = Rc::new(RefCell::new(State {
            phase: Phase::Warmup(0),
            root: root.clone(),
            scenario,
            cached: None,
            uncached: None,
            cached_output,
            uncached_output,
            parent_snapshot: None,
            cached_mutation_snapshot: None,
            ax_active_seen: false,
        }));

        let cached_state = state.clone();
        cx.observe_new::<ViewHost<CachedLeaf>>(move |_, _, cx| {
            cached_state.borrow_mut().cached = Some(cx.weak_entity());
        })
        .detach();
        let uncached_state = state.clone();
        cx.observe_new::<ViewHost<UncachedLeaf>>(move |_, _, cx| {
            uncached_state.borrow_mut().uncached = Some(cx.weak_entity());
        })
        .detach();

        let window_state = state.clone();
        let bounds = Bounds::centered(None, size(px(520.0), px(180.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |window, _| {
                schedule(window, window_state);
                root.clone()
            },
        )
        .expect("cache benchmark window must open");

        let timeout_state = state;
        cx.spawn(async move |cx| {
            cx.background_executor().timer(TIMEOUT).await;
            cx.update(|cx| finish(cx, timeout_state, "timeout", Some("no frame progress")));
        })
        .detach();
        cx.activate(true);
    });
}
