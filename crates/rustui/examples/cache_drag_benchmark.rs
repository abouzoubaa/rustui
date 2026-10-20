//! Interactive native-drag correctness probe for the fixed-bounds child cache.
//!
//! Run with `./scripts/cargo-headstart run -p rustui --example cache_drag_benchmark`.
//! Click item A once to mutate its local state, then drag A onto B.

use std::{
    cell::{Cell, RefCell},
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    rc::Rc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use gpui::{
    AnyElement, App, AppContext as _, Bounds, Context, Entity, IntoElement, ParentElement, Render,
    Styled, Window, WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use rustui::{
    prelude::{Component, Text, VStack, View, ViewHost},
    runtime::render_diagnostics::{enable_render_diagnostics, render_diagnostics_snapshot},
    theme::{AppearanceMode, activate_theme, parse_theme},
    view::{EmptyView, RenderContext},
};
use serde_json::{Value, json};

const TIMEOUT: Duration = Duration::from_secs(120);
const IDS: [char; 3] = ['A', 'B', 'C'];

#[derive(Clone, Copy)]
struct DragId(char);

struct DragGhost;

impl Render for DragGhost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(150.))
            .h(px(72.))
            .bg(gpui::blue())
            .text_color(gpui::white())
            .child("Dragging item")
    }
}

struct Root {
    order: Vec<char>,
    local: Rc<RefCell<Vec<(char, u32)>>>,
    owner: Rc<RefCell<Option<gpui::WeakEntity<ViewHost<Root>>>>>,
    children: Rc<RefCell<Vec<(char, gpui::WeakEntity<ViewHost<DragLeaf>>)>>>,
    moves: u32,
    result: Rc<RefCell<Option<Value>>>,
    active_moves: Rc<RefCell<Vec<Value>>>,
    mounts: Rc<RefCell<Vec<(char, u32)>>>,
}

impl View for Root {
    fn body(&self) -> impl View {
        let owner = self.owner.clone();
        let local = self.local.clone();
        let result = self.result.clone();
        let active_moves = self.active_moves.clone();
        let mounts = self.mounts.clone();
        let children = self.children.clone();
        VStack::new()
            .child(Text("Click A once, then drag A onto B"))
            .child(Text(format!(
                "Order: {:?} | drag moves: {}",
                self.order, self.moves
            )))
            .children(self.order.iter().copied().map(|id| {
                let owner = owner.clone();
                let local = local.clone();
                let result = result.clone();
                let active_moves = active_moves.clone();
                let mounts = mounts.clone();
                let children = children.clone();
                Component(id.to_string(), move || DragLeaf {
                    id,
                    local: 100 + u32::from(id),
                    owner: owner.clone(),
                    children: children.clone(),
                    local_values: local.clone(),
                    result: result.clone(),
                    active_moves: active_moves.clone(),
                    mounts: mounts.clone(),
                })
                .cached_fixed(170.0, 78.0)
            }))
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        div()
            .size_full()
            .p_4()
            .bg(gpui::rgb(0x101827))
            .text_color(gpui::white())
            .child(self.body().into_element(context))
            .into_any_element()
    }
}

struct DragLeaf {
    id: char,
    local: u32,
    owner: Rc<RefCell<Option<gpui::WeakEntity<ViewHost<Root>>>>>,
    children: Rc<RefCell<Vec<(char, gpui::WeakEntity<ViewHost<DragLeaf>>)>>>,
    local_values: Rc<RefCell<Vec<(char, u32)>>>,
    result: Rc<RefCell<Option<Value>>>,
    active_moves: Rc<RefCell<Vec<Value>>>,
    mounts: Rc<RefCell<Vec<(char, u32)>>>,
}

impl View for DragLeaf {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn on_mount(&mut self, _: &mut App) {
        let mut mounts = self.mounts.borrow_mut();
        if let Some((_, count)) = mounts.iter_mut().find(|(id, _)| *id == self.id) {
            *count += 1;
        } else {
            mounts.push((self.id, 1));
        }
    }

    fn into_element(&self, _: &mut RenderContext<'_>) -> AnyElement {
        let id = self.id;
        let value = self.local;
        self.local_values.borrow_mut().retain(|(key, _)| *key != id);
        self.local_values.borrow_mut().push((id, value));
        let owner_move = self.owner.clone();
        let owner_drop = self.owner.clone();
        let children = self.children.clone();
        let result = self.result.clone();
        let active_moves = self.active_moves.clone();
        div()
            .id(format!("drag-{id}"))
            .w(px(170.))
            .h(px(78.))
            .bg(gpui::rgb(0x1e293b))
            .text_color(gpui::white())
            .flex()
            .items_center()
            .justify_center()
            .border_2()
            .border_color(gpui::blue())
            .cursor_move()
            .child(format!("{id}  local={value}"))
            .on_click(move |_, _, cx| {
                if let Some(child) = children.borrow().iter().find(|(key, _)| *key == id).map(|(_, child)| child.clone()) {
                    let _ = child.update(cx, |host, cx| {
                        host.component_mut().local += 1;
                        cx.notify();
                    });
                }
            })
            .on_drag(DragId(id), |_, _, _, cx| cx.new(|_| DragGhost))
            .on_drag_move::<DragId>(move |_, window, cx| {
                let active_drag = cx.has_active_drag();
                let ax_active = window.is_a11y_active();
                let snapshot = render_diagnostics_snapshot(cx);
                let count = snapshot.hosts.iter()
                    .find(|host| host.component_type == std::any::type_name::<DragLeaf>())
                    .map_or(0, |host| host.count);
                active_moves.borrow_mut().push(json!({
                    "active_drag": active_drag,
                    "ax_active": ax_active,
                    "cache_bypass_reason": if active_drag { "active_native_drag" } else if ax_active { "active_accessibility" } else { "none_detected" },
                    "child_host_lowerings": count,
                }));
                if let Some(owner) = owner_move.borrow().as_ref().and_then(gpui::WeakEntity::upgrade) {
                    owner.update(cx, |root, cx| {
                        root.component_mut().moves += 1;
                        cx.notify();
                    });
                }
            })
            .on_drop(move |drag: &DragId, _, cx| {
                if let Some(owner) = owner_drop.borrow().as_ref().and_then(gpui::WeakEntity::upgrade) {
                    owner.update(cx, |root, cx| {
                        let root = root.component_mut();
                        if drag.0 != id {
                            let Some(source) = root.order.iter().position(|item| *item == drag.0) else { return };
                            let Some(target) = root.order.iter().position(|item| *item == id) else { return };
                            let moved = root.order.remove(source);
                            root.order.insert(target, moved);
                            cx.notify();
                        }
                    });
                    let active = cx.has_active_drag();
                    *result.borrow_mut() = Some(json!({
                        "drop_target": id.to_string(),
                        "dragged_id": drag.0.to_string(),
                        "active_drag_in_drop_callback": active
                    }));
                }
            })
            .into_any_element()
    }
}

struct Probe {
    root: Entity<ViewHost<Root>>,
    local: Rc<RefCell<Vec<(char, u32)>>>,
    result: Rc<RefCell<Option<Value>>>,
    active_moves: Rc<RefCell<Vec<Value>>>,
    mounts: Rc<RefCell<Vec<(char, u32)>>>,
    idle_stage: Cell<u8>,
    idle_baseline: Cell<Option<(u64, u64)>>,
    previous_active_counts: Cell<Option<(u64, u64)>>,
    active_frames: RefCell<Vec<Value>>,
    ax_active_seen: Cell<bool>,
}

fn host_lowerings<V: View>(cx: &App) -> u64 {
    render_diagnostics_snapshot(cx)
        .hosts
        .iter()
        .filter(|host| host.component_type == std::any::type_name::<V>())
        .map(|host| host.count)
        .sum()
}

fn schedule_poll(window: &Window, probe: Rc<Probe>) {
    window.on_next_frame(move |window, cx| {
        let ax_active = window.is_a11y_active();
        probe.ax_active_seen.set(probe.ax_active_seen.get() || ax_active);
        // GPUI invokes these callbacks before drawing. Consecutive active
        // samples bracket a completed frame with native drag still active.
        let active_drag = cx.has_active_drag();
        let counts = (host_lowerings::<Root>(cx), host_lowerings::<DragLeaf>(cx));
        if active_drag {
            let previous = if ax_active {
                probe.previous_active_counts.replace(None);
                None
            } else {
                probe.previous_active_counts.replace(Some(counts))
            };
            probe.active_frames.borrow_mut().push(json!({
                "active_drag": true,
                "ax_active": ax_active,
                "cache_bypass_reason": if ax_active { "active_native_drag_and_accessibility" } else { "active_native_drag" },
                "root_host_lowerings": counts.0,
                "child_host_lowerings": counts.1,
                "root_lowering_delta": previous.map(|before| counts.0.saturating_sub(before.0)),
                "child_lowering_delta": previous.map(|before| counts.1.saturating_sub(before.1)),
            }));
        } else {
            probe.previous_active_counts.set(None);
        }
        if probe.result.borrow().is_none() || active_drag {
            schedule_poll(window, probe);
            return;
        }
        match probe.idle_stage.get() {
            0 => {
                probe.idle_stage.set(1);
                probe.root.update(cx, |_, cx| cx.notify());
                schedule_poll(window, probe);
            }
            1 => {
                // Warm the cached representation after reorder, then measure
                // a separate parent render to establish idle cache reuse.
                probe.idle_baseline.set(Some(counts));
                probe.idle_stage.set(2);
                probe.root.update(cx, |_, cx| cx.notify());
                schedule_poll(window, probe);
            }
            _ => {
                let (order, moves) = probe.root.read_with(cx, |root, _| {
                    (root.component().order.clone(), root.component().moves)
                });
                let local = probe.local.borrow().clone();
                let mutated_a_survived = local.iter().any(|(id, value)| *id == 'A' && *value > 165);
                let mount_identity_preserved = probe.mounts.borrow().len() == IDS.len()
                    && probe.mounts.borrow().iter().all(|(_, count)| *count == 1);
                let children = counts.1;
                let baseline = probe.idle_baseline.get().expect("idle baseline recorded");
                let idle_root_delta = counts.0.saturating_sub(baseline.0);
                let idle_child_delta = children.saturating_sub(baseline.1);
                let idle_reuse_observed = idle_root_delta > 0 && idle_child_delta == 0;
                let active_fallback_observed = probe.active_frames.borrow().iter().any(|sample| {
                    sample["root_lowering_delta"].as_u64().unwrap_or(0) > 0
                        && sample["child_lowering_delta"].as_u64().unwrap_or(0) >= IDS.len() as u64
                });
                let mut record = probe.result.borrow().clone().unwrap_or_default();
                let ax_active_seen = probe.ax_active_seen.get() || probe.active_moves.borrow().iter().any(|sample| {
                    sample.get("ax_active").and_then(Value::as_bool).unwrap_or(true)
                });
                record["runtime_identity_status"] = json!(if order != ['B', 'A', 'C'] { "order_mismatch" } else if !mutated_a_survived { "local_mutation_missing" } else if !mount_identity_preserved { "child_identity_changed" } else { "complete" });
                record["cache_acceptance_eligible"] = json!(!ax_active_seen);
                record["status"] = json!(if ax_active_seen { "excluded_ax_active" } else if order != ['B', 'A', 'C'] { "order_mismatch" } else if !mutated_a_survived { "local_mutation_missing" } else if !mount_identity_preserved { "child_identity_changed" } else if !active_fallback_observed { "active_fallback_unobserved" } else if !idle_reuse_observed { "idle_cache_reuse_unobserved" } else { "complete" });
                record["final_order"] = json!(order.iter().map(char::to_string).collect::<Vec<_>>());
                record["local_values"] = json!(local.iter().map(|(id, value)| (id.to_string(), value)).collect::<Vec<_>>());
                record["mutated_a_survived_reorder"] = json!(mutated_a_survived);
                record["drag_move_updates"] = json!(moves);
                record["active_move_samples"] = json!(probe.active_moves.borrow().as_slice());
                record["mount_counts_by_id"] = json!(probe.mounts.borrow().iter().map(|(id, count)| (id.to_string(), count)).collect::<Vec<_>>());
                record["mount_identity_preserved"] = json!(mount_identity_preserved);
                record["active_frame_samples"] = json!(probe.active_frames.borrow().as_slice());
                record["active_drag_fallback_observed"] = json!(active_fallback_observed);
                record["post_drag_idle_root_lowering_delta"] = json!(idle_root_delta);
                record["post_drag_idle_child_lowering_delta"] = json!(idle_child_delta);
                record["post_drag_idle_cache_reuse_observed"] = json!(idle_reuse_observed);
                record["final_active_drag"] = json!(active_drag);
                record["final_ax_active"] = json!(window.is_a11y_active());
                record["post_drag_idle_child_host_lowerings"] = json!(children);
                let active_count = probe.active_moves.borrow().last()
                    .and_then(|sample| sample.get("child_host_lowerings"))
                    .and_then(Value::as_u64);
                record["active_to_idle_lowering_delta"] = active_count.map_or(Value::Null, |count| json!(children.saturating_sub(count)));
                record["ax_active_seen"] = json!(ax_active_seen);
                record["measurement_scope"] = json!("interactive correctness observations only; no speed or presentation claim");
                if let Ok(path) = write_result(&record) { println!("{}", path.display()); }
                cx.quit();
            }
        }
    });
}

fn write_result(result: &Value) -> std::io::Result<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/headstart");
    fs::create_dir_all(&dir)?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = dir.join(format!(
        "cache-drag-benchmark-{}-{timestamp}.json",
        std::process::id()
    ));
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    serde_json::to_writer_pretty(&mut output, result).map_err(std::io::Error::other)?;
    output.write_all(b"\n")?;
    Ok(path)
}

fn main() {
    let theme = parse_theme(include_str!("../../../themes/macos27.json"))
        .expect("bundled theme must parse");
    gpui_platform::application().run(move |cx| {
        gpui_component::init(cx);
        activate_theme(&theme, Some(AppearanceMode::Light), cx)
            .expect("bundled light theme must activate");
        enable_render_diagnostics(cx);
        let local = Rc::new(RefCell::new(Vec::new()));
        let owner = Rc::new(RefCell::new(None));
        let children = Rc::new(RefCell::new(Vec::new()));
        let result = Rc::new(RefCell::new(None));
        let active_moves = Rc::new(RefCell::new(Vec::new()));
        let mounts = Rc::new(RefCell::new(Vec::new()));
        let root = cx.new(|cx| ViewHost::new(Root {
            order: IDS.to_vec(),
            local: local.clone(),
            owner: owner.clone(),
            children: children.clone(),
            moves: 0,
            result: result.clone(),
            active_moves: active_moves.clone(),
            mounts: mounts.clone(),
        }, cx));
        *owner.borrow_mut() = Some(root.downgrade());
        let observed_children = children.clone();
        cx.observe_new::<ViewHost<DragLeaf>>(move |host, _, cx| {
            let id = host.component().id;
            let weak = cx.weak_entity();
            let mut children = observed_children.borrow_mut();
            children.retain(|(key, _)| *key != id);
            children.push((id, weak));
        }).detach();
        let bounds = Bounds::centered(None, size(px(640.), px(420.)), cx);
        let probe = Rc::new(Probe { root: root.clone(), local, result, active_moves, mounts, idle_stage: Cell::new(0), idle_baseline: Cell::new(None), previous_active_counts: Cell::new(None), active_frames: RefCell::new(Vec::new()), ax_active_seen: Cell::new(false) });
        let timeout_probe = probe.clone();
        cx.open_window(WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        }, move |window, _| {
            window.set_window_title("RustUI Cache Drag Probe");
            let probe = probe.clone();
            window.on_next_frame(move |window, cx| {
                window.activate_window();
                probe.ax_active_seen.set(window.is_a11y_active());
                schedule_poll(window, probe);
            });
            root.clone()
        }).expect("cache drag window must open");
        cx.spawn(async move |cx| {
            cx.background_executor().timer(TIMEOUT).await;
            cx.update(|cx| {
                let ax_active_seen = timeout_probe.ax_active_seen.get();
                let _ = write_result(&json!({"kind": "cache_drag_correctness", "status": "timeout_no_native_drop", "ax_active_seen": ax_active_seen, "cache_acceptance_eligible": !ax_active_seen, "cache_bypass_reason": if ax_active_seen { "active_accessibility_seen" } else { "none_detected" }}));
                cx.quit();
            });
        }).detach();
        cx.activate(true);
    });
}
