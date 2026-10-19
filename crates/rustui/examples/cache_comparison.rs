//! Paired retained-subtree probe. Launch only through CUA; see P3D_CACHE_COMPARISON.md.

use std::{
    cell::RefCell,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    rc::Rc,
    time::Duration,
};

use gpui::{
    App, AppContext as _, Bounds, Entity, WeakEntity, Window, WindowBounds, WindowOptions, px, size,
};
use rustui::{
    prelude::{Component, Container, HStack, Text, VStack, View, ViewHost},
    runtime::render_diagnostics::{enable_render_diagnostics, render_diagnostics_snapshot},
    theme::{AppearanceMode, activate_theme, parse_theme},
};
use serde_json::{Value, json};

const LINES: usize = 12;
const WIDTH: f32 = 210.0;
const HEIGHT: f32 = 220.0;
const WARMUP: u32 = 10;
const UPDATES: u32 = 100;

#[derive(Clone)]
struct Config {
    run_id: String,
    cached: bool,
    leaves: usize,
}

impl Config {
    fn load() -> Result<Self, String> {
        let mut args = std::env::args().skip(1);
        let path = match args.next().as_deref() {
            Some("--config") => {
                let path = args.next().ok_or("--config requires a JSON path")?;
                if args.next().is_some() {
                    return Err("unexpected arguments".into());
                }
                PathBuf::from(path)
            }
            None => std::env::current_exe()
                .map_err(|error| error.to_string())?
                .parent()
                .ok_or("missing executable parent")?
                .join("../Resources/cache-comparison.json"),
            _ => return Err("expected --config <JSON path> or app-bundle configuration".into()),
        };
        let value: Value =
            serde_json::from_slice(&fs::read(path).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
        let run_id = value["run_id"].as_str().ok_or("missing run_id")?;
        if run_id.is_empty()
            || run_id.len() > 80
            || !run_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            || !run_id.as_bytes()[0].is_ascii_alphanumeric()
        {
            return Err("invalid run_id".into());
        }
        let cached = match value["mode"].as_str() {
            Some("cached") => true,
            Some("uncached") => false,
            _ => return Err("mode must be cached or uncached".into()),
        };
        let leaves = usize::try_from(value["leaves"].as_u64().ok_or("missing leaves")?)
            .map_err(|_| "leaves does not fit this platform")?;
        if !matches!(leaves, 8 | 32) {
            return Err("leaves must be 8 or 32".into());
        }
        Ok(Self {
            run_id: run_id.into(),
            cached,
            leaves,
        })
    }

    fn mode(&self) -> &'static str {
        if self.cached { "cached" } else { "uncached" }
    }

    fn path(&self, kind: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/headstart/performance")
            .join(format!("cache-comparison-{}-{kind}.json", self.run_id))
    }

    fn gate(&self, kind: &str) -> bool {
        fs::read(self.path(kind))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
            .is_some_and(|value| {
                value["run_id"] == self.run_id
                    && value["mode"] == self.mode()
                    && value["pid"] == std::process::id()
            })
    }
}

fn write(config: &Config, kind: &str, document: &Value) -> std::io::Result<()> {
    let path = config.path(kind);
    let directory = path.parent().expect("fixed output parent");
    fs::create_dir_all(directory)?;
    // Output is fixed inside this repository; refuse symlinked output ancestors.
    for ancestor in path.ancestors() {
        if ancestor
            .symlink_metadata()
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return Err(std::io::Error::other(
                "comparison output contains a symlink",
            ));
        }
    }
    let mut stream = OpenOptions::new().write(true).create_new(true).open(path)?;
    serde_json::to_writer_pretty(&mut stream, document).map_err(std::io::Error::other)?;
    stream.write_all(b"\n")
}

struct ComparisonRoot {
    tick: u32,
    cached: bool,
    leaves: usize,
    values: Rc<RefCell<Vec<u32>>>,
    mounts: Rc<RefCell<Vec<u32>>>,
}

impl View for ComparisonRoot {
    fn body(&self) -> impl View {
        let mut content = VStack::new().child(Text(format!("Parent update {:06}", self.tick)));
        for row in 0..self.leaves.div_ceil(4) {
            let mut children = HStack::new();
            for column in 0..4 {
                let index = row * 4 + column;
                if index >= self.leaves {
                    break;
                }
                let values = self.values.clone();
                let mounts = self.mounts.clone();
                let child = Component(format!("comparison-leaf-{index}"), move || ComparisonLeaf {
                    index,
                    value: 0,
                    values: values.clone(),
                    mounts: mounts.clone(),
                });
                // This is the only rendering difference between the two modes.
                let child = if self.cached {
                    child.cached_fixed(WIDTH, HEIGHT)
                } else {
                    child
                };
                children = children.child(child);
            }
            content = content.child(children);
        }
        content
    }
}

struct ComparisonLeaf {
    index: usize,
    value: u32,
    values: Rc<RefCell<Vec<u32>>>,
    mounts: Rc<RefCell<Vec<u32>>>,
}

impl View for ComparisonLeaf {
    fn on_mount(&mut self, _: &mut App) {
        self.mounts.borrow_mut()[self.index] += 1;
    }

    fn body(&self) -> impl View {
        self.values.borrow_mut()[self.index] = self.value;
        let mut content = VStack::new();
        for line in 0..LINES {
            content = content.child(
                Text(format!(
                    "Leaf {:02} · line {line:02} · value {}",
                    self.index, self.value
                ))
                .font_size(10.0),
            );
        }
        Container(content).width(WIDTH).height(HEIGHT)
    }
}

#[derive(Clone, Copy)]
enum Phase {
    Waiting,
    Warmup(u32),
    Measure(u32),
    Mutation,
    Finished,
}

struct State {
    config: Config,
    leaves: usize,
    phase: Phase,
    root: Entity<ViewHost<ComparisonRoot>>,
    children: Vec<WeakEntity<ViewHost<ComparisonLeaf>>>,
    values: Rc<RefCell<Vec<u32>>>,
    mounts: Rc<RefCell<Vec<u32>>>,
    parent_root: Option<u64>,
    parent_leaf: Option<u64>,
    ax_seen: bool,
    drag_seen: bool,
    terminal_status: Option<String>,
    workload_result: Option<Value>,
}

fn counts(cx: &App) -> (u64, u64) {
    let snapshot = render_diagnostics_snapshot(cx);
    let count = |name| {
        snapshot
            .hosts
            .iter()
            .find(|host| host.component_type == name)
            .map_or(0, |host| host.count)
    };
    (
        count(std::any::type_name::<ComparisonRoot>()),
        count(std::any::type_name::<ComparisonLeaf>()),
    )
}

fn schedule(window: &Window, state: Rc<RefCell<State>>) {
    window.on_next_frame(move |window, cx| advance(window, cx, state));
}

fn finish(cx: &mut App, state: &Rc<RefCell<State>>, requested: &str) {
    let mut state = state.borrow_mut();
    if matches!(state.phase, Phase::Finished) {
        return;
    }
    let (_, leaf) = counts(cx);
    let expected = if state.config.cached {
        0
    } else {
        state.leaves as u64 * u64::from(UPDATES)
    };
    let valid = state.parent_root == Some(u64::from(UPDATES))
        && state.parent_leaf == Some(expected)
        && leaf.checked_sub(expected) == Some(state.leaves as u64)
        && state.children.len() == state.leaves
        && state.values.borrow().iter().all(|value| *value == 1)
        && state.mounts.borrow().iter().all(|mounts| *mounts == 1);
    let status = if requested != "complete" {
        requested
    } else if valid {
        "complete"
    } else {
        "failed_correctness"
    };
    let document = json!({
        "schema_version": 1, "kind": "rustui_cache_comparison_workload",
        "run_id": state.config.run_id, "pid": std::process::id(), "mode": state.config.mode(),
        "status": status, "ax_active_seen": state.ax_seen, "native_drag_seen": state.drag_seen,
        "protocol": {"leaves": state.leaves, "lines_per_leaf": LINES, "width": WIDTH, "height": HEIGHT,
            "warmup_updates": WARMUP, "measured_updates": UPDATES},
        "counts": {"parent_root": state.parent_root, "parent_leaf": state.parent_leaf,
            "final_leaf": leaf, "mutation_leaf_delta": state.parent_leaf.and_then(|n| leaf.checked_sub(n))},
        "values": *state.values.borrow(), "mounts": *state.mounts.borrow(),
        "environment_scope": "empty", "correctness_passed": valid,
    });
    state.terminal_status = Some(status.to_owned());
    state.workload_result = Some(document.clone());
    state.phase = Phase::Finished;
    if let Err(error) = write(&state.config, "result", &document) {
        eprintln!("cache comparison result write failed: {error}");
        cx.quit();
    }
}

fn advance(window: &mut Window, cx: &mut App, state: Rc<RefCell<State>>) {
    let phase = state.borrow().phase;
    {
        let mut state = state.borrow_mut();
        state.ax_seen |= window.is_a11y_active();
        state.drag_seen |= cx.has_active_drag();
    }
    if matches!(phase, Phase::Finished) {
        if state.borrow().config.gate("ack") {
            let state = state.borrow();
            let status = if state.ax_seen || state.drag_seen {
                "excluded_interaction"
            } else {
                state
                    .terminal_status
                    .as_deref()
                    .unwrap_or("failed_missing_result")
            };
            let final_document = json!({
                "schema_version": 1,
                "kind": "rustui_cache_comparison_final",
                "run_id": state.config.run_id,
                "mode": state.config.mode(),
                "pid": std::process::id(),
                "status": status,
                "ax_active_seen": state.ax_seen,
                "native_drag_seen": state.drag_seen,
                "workload": state.workload_result.clone(),
            });
            if let Err(error) = write(&state.config, "final", &final_document) {
                eprintln!("cache comparison final guard write failed: {error}");
            }
            cx.quit();
        } else {
            schedule(window, state);
        }
        return;
    }
    if state.borrow().ax_seen || state.borrow().drag_seen {
        finish(cx, &state, "excluded_interaction");
        schedule(window, state);
        return;
    }
    match phase {
        Phase::Waiting => {
            if state.borrow().config.gate("start") {
                state.borrow_mut().phase = Phase::Warmup(0);
            }
        }
        Phase::Warmup(n) if n == WARMUP => {
            enable_render_diagnostics(cx);
            state.borrow_mut().phase = Phase::Measure(0);
            advance(window, cx, state);
            return;
        }
        Phase::Warmup(n) | Phase::Measure(n)
            if n < if matches!(phase, Phase::Warmup(_)) {
                WARMUP
            } else {
                UPDATES
            } =>
        {
            state.borrow().root.update(cx, |host, cx| {
                host.component_mut().tick += 1;
                cx.notify();
            });
            state.borrow_mut().phase = if matches!(phase, Phase::Warmup(_)) {
                Phase::Warmup(n + 1)
            } else {
                Phase::Measure(n + 1)
            };
        }
        Phase::Measure(_) => {
            let (root, leaf) = counts(cx);
            state.borrow_mut().parent_root = Some(root);
            state.borrow_mut().parent_leaf = Some(leaf);
            let children = state.borrow().children.clone();
            let success = children.iter().all(|child| {
                child
                    .update(cx, |host, cx| {
                        host.component_mut().value += 1;
                        cx.notify();
                    })
                    .is_ok()
            });
            if !success || children.len() != state.borrow().leaves {
                finish(cx, &state, "failed_children");
            } else {
                state.borrow_mut().phase = Phase::Mutation;
            }
        }
        Phase::Mutation => finish(cx, &state, "complete"),
        _ => {}
    }
    schedule(window, state);
}

fn main() {
    let theme = parse_theme(include_str!("../../../themes/macos27.json"))
        .expect("bundled macOS 27 theme must parse");
    let config = Config::load().unwrap_or_else(|error| {
        eprintln!("cache comparison: {error}");
        std::process::exit(2)
    });
    let ready = json!({"schema_version": 1, "kind": "rustui_cache_comparison_ready", "run_id": config.run_id,
        "mode": config.mode(), "pid": std::process::id(), "executable": std::env::current_exe().expect("executable path")});
    write(&config, "ready", &ready).expect("new comparison ready record");
    // Finder-launched runs have no attached stderr; preserve any startup panic with the run evidence.
    let panic_path = config.path("panic");
    std::panic::set_hook(Box::new(move |info| {
        if let Ok(mut stream) = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&panic_path)
        {
            let _ = writeln!(stream, "{info}");
        }
    }));
    gpui_platform::application().run(move |cx| {
        gpui_component::init(cx);
        activate_theme(&theme, Some(AppearanceMode::Light), cx)
            .expect("bundled macOS 27 theme must activate");
        let leaves = config.leaves;
        let values = Rc::new(RefCell::new(vec![0; leaves]));
        let mounts = Rc::new(RefCell::new(vec![0; leaves]));
        let root = cx.new(|cx| {
            ViewHost::new(
                ComparisonRoot {
                    tick: 0,
                    cached: config.cached,
                    leaves,
                    values: values.clone(),
                    mounts: mounts.clone(),
                },
                cx,
            )
        });
        let state = Rc::new(RefCell::new(State {
            config,
            leaves,
            phase: Phase::Waiting,
            root: root.clone(),
            children: Vec::new(),
            values,
            mounts,
            parent_root: None,
            parent_leaf: None,
            ax_seen: false,
            drag_seen: false,
            terminal_status: None,
            workload_result: None,
        }));
        let observed = state.clone();
        cx.observe_new::<ViewHost<ComparisonLeaf>>(move |_, _, cx| {
            observed.borrow_mut().children.push(cx.weak_entity());
        })
        .detach();
        let window_state = state.clone();
        let bounds = Bounds::centered(None, size(px(900.0), px(520.0)), cx);
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
        .expect("comparison window");
        cx.spawn(async move |cx| {
            cx.background_executor()
                .timer(Duration::from_secs(120))
                .await;
            cx.update(|cx| {
                finish(cx, &state, "timeout");
                cx.quit();
            });
        })
        .detach();
        cx.activate(true);
    });
}
