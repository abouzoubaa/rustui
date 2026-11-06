//! Native dispatch probe for the M49 reduced-motion and all-window refresh actions.
//!
//! Open both windows, click `M49 enable reduced motion`, then click
//! `M49 refresh all windows`. The JSON output records GPUI's preference
//! readback and per-root RustUI lowering counts before and after refresh.

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use gpui::{AppContext as _, Bounds, WindowBounds, WindowOptions, px, size};
use rustui::{
    prelude::{AppAction, Button, Text, VStack, View, ViewHost},
    runtime::render_diagnostics::{
        RenderDiagnosticsSnapshot, enable_render_diagnostics, render_diagnostics_snapshot,
    },
    theme::{AppearanceMode, activate_theme, parse_theme},
};
use serde_json::json;

const POLL: Duration = Duration::from_millis(50);
const STABLE_FOR: Duration = Duration::from_millis(400);
const AFTER_REFRESH: Duration = Duration::from_millis(350);
const TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Default)]
struct Observed {
    idle_control: bool,
    reduced_motion: Option<bool>,
    measurement_requested: bool,
    refresh_dispatched: bool,
}

struct FirstWindow {}

impl View for FirstWindow {
    fn body(&self) -> impl View {
        Text("M49 first window: refresh should lower this root again")
    }
}

struct SecondWindow {
    observed: Arc<Mutex<Observed>>,
}

impl View for SecondWindow {
    fn body(&self) -> impl View {
        let observed = Arc::clone(&self.observed);
        let idle_control = observed.lock().expect("probe mutex").idle_control;
        let enable_motion = AppAction::reduced_motion(true).then(AppAction::new({
            let observed = Arc::clone(&observed);
            move |cx| {
                observed.lock().expect("probe mutex").reduced_motion = Some(cx.reduce_motion())
            }
        }));
        let measurement = if idle_control {
            AppAction::new({
                let observed = Arc::clone(&observed);
                move |_| observed.lock().expect("probe mutex").measurement_requested = true
            })
        } else {
            AppAction::refresh_windows().then(AppAction::new({
                move |_| {
                    let mut observed = observed.lock().expect("probe mutex");
                    observed.measurement_requested = true;
                    observed.refresh_dispatched = true;
                }
            }))
        };
        let measurement_label = if idle_control {
            "M49 idle control (no refresh)"
        } else {
            "M49 refresh all windows"
        };

        VStack::new()
            .spacing(12.0)
            .child(Text("M49 runtime action dispatch probe"))
            .child(Text("Click Enable first, then the measurement button."))
            .child(
                Button("M49 enable reduced motion", enable_motion).id("m49-enable-reduced-motion"),
            )
            .child(Button(measurement_label, measurement).id("m49-measurement-trigger"))
    }
}

fn lowering_count(snapshot: &RenderDiagnosticsSnapshot, type_name: &str) -> u64 {
    snapshot
        .hosts
        .iter()
        .find(|host| host.component_type == type_name)
        .map_or(0, |host| host.count)
}

fn write_result(
    observed: &Observed,
    before: &RenderDiagnosticsSnapshot,
    after: &RenderDiagnosticsSnapshot,
    elapsed_ms: u128,
    status: &str,
) {
    let first_type = std::any::type_name::<FirstWindow>();
    let second_type = std::any::type_name::<SecondWindow>();
    let result = json!({
        "kind": "rustui_m49_runtime_action_dispatch",
        "status": status,
        "mode": if observed.idle_control { "idle_control" } else { "refresh_treatment" },
        "reduced_motion_requested": true,
        "reduced_motion_readback_after_factory_dispatch": observed.reduced_motion,
        "refresh_factory_dispatched": observed.refresh_dispatched,
        "root_host_lowerings_before_measurement": {
            "first_window": lowering_count(before, first_type),
            "second_window": lowering_count(before, second_type),
        },
        "root_host_lowerings_after_measurement": {
            "first_window": lowering_count(after, first_type),
            "second_window": lowering_count(after, second_type),
        },
        "root_host_lowerings_advanced": {
            "first_window": lowering_count(after, first_type) > lowering_count(before, first_type),
            "second_window": lowering_count(after, second_type) > lowering_count(before, second_type),
        },
        "elapsed_ms": elapsed_ms,
        "measurement_scope": "GPUI reduced-motion getter readback and RustUI root host lowerings; not presented-frame timing or pixels",
    });

    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/headstart/evidence");
    if let Err(error) = fs::create_dir_all(&dir) {
        eprintln!("could not create evidence directory: {error}");
        return;
    }
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mode = if observed.idle_control {
        "idle-control"
    } else {
        "refresh-treatment"
    };
    let path = dir.join(format!("m49-runtime-action-{mode}-{timestamp}.json"));
    let write = (|| -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        serde_json::to_writer_pretty(&mut file, &result).map_err(std::io::Error::other)?;
        file.write_all(b"\n")
    })();
    match write {
        Ok(()) => println!("{}", path.display()),
        Err(error) => eprintln!("could not write evidence: {error}"),
    }
}

fn main() {
    let idle_control = std::env::args().any(|arg| arg == "--idle-control");
    gpui_platform::application().run(move |cx| {
        gpui_component::init(cx);
        let theme = parse_theme(include_str!("../../../themes/macos27.json"))
            .expect("bundled macOS theme must parse");
        activate_theme(&theme, Some(AppearanceMode::Light), cx)
            .expect("bundled light theme must activate");
        enable_render_diagnostics(cx);

        let observed = Arc::new(Mutex::new(Observed {
            idle_control,
            ..Observed::default()
        }));
        let first = cx.new(|cx| ViewHost::new(FirstWindow {}, cx));
        let second = cx.new(|cx| {
            ViewHost::new(
                SecondWindow {
                    observed: Arc::clone(&observed),
                },
                cx,
            )
        });

        let first_root = first.clone();
        let first_bounds = Bounds::centered(None, size(px(560.0), px(260.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(first_bounds)),
                ..Default::default()
            },
            move |_, _| first_root.clone(),
        )
        .expect("first probe window must open");

        let second_root = second.clone();
        let second_bounds = Bounds::centered(None, size(px(560.0), px(220.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(second_bounds)),
                ..Default::default()
            },
            move |_, _| second_root.clone(),
        )
        .expect("second probe window must open");

        let started = Instant::now();
        cx.spawn(async move |cx| {
            let mut stable_since = None;
            let mut stable_counts = None;
            let before_refresh = loop {
                cx.background_executor().timer(POLL).await;
                let snapshot = cx.update(|cx| render_diagnostics_snapshot(cx));
                let first_count = lowering_count(&snapshot, std::any::type_name::<FirstWindow>());
                let second_count = lowering_count(&snapshot, std::any::type_name::<SecondWindow>());
                let counts = (first_count, second_count);
                if counts.0 > 0 && counts.1 > 0 && stable_counts == Some(counts) {
                    if stable_since.is_some_and(|instant: Instant| instant.elapsed() >= STABLE_FOR)
                    {
                        let observed_now = observed.lock().expect("probe mutex");
                        if observed_now.reduced_motion == Some(true) {
                            drop(observed_now);
                            break snapshot;
                        }
                    }
                } else {
                    stable_counts = Some(counts);
                    stable_since = Some(Instant::now());
                }
                if started.elapsed() >= TIMEOUT {
                    let observed_now = observed.lock().expect("probe mutex");
                    write_result(
                        &observed_now,
                        &snapshot,
                        &snapshot,
                        started.elapsed().as_millis(),
                        "timeout_before_refresh",
                    );
                    cx.update(|cx| cx.quit());
                    return;
                }
            };

            loop {
                cx.background_executor().timer(POLL).await;
                let dispatched = observed.lock().expect("probe mutex").measurement_requested;
                if dispatched {
                    break;
                }
                if started.elapsed() >= TIMEOUT {
                    let after = cx.update(|cx| render_diagnostics_snapshot(cx));
                    let observed_now = observed.lock().expect("probe mutex");
                    write_result(
                        &observed_now,
                        &before_refresh,
                        &after,
                        started.elapsed().as_millis(),
                        "timeout_waiting_for_refresh",
                    );
                    cx.update(|cx| cx.quit());
                    return;
                }
            }

            cx.background_executor().timer(AFTER_REFRESH).await;
            let after_refresh = cx.update(|cx| render_diagnostics_snapshot(cx));
            let observed_now = observed.lock().expect("probe mutex");
            let first_advanced =
                lowering_count(&after_refresh, std::any::type_name::<FirstWindow>())
                    > lowering_count(&before_refresh, std::any::type_name::<FirstWindow>());
            let second_advanced =
                lowering_count(&after_refresh, std::any::type_name::<SecondWindow>())
                    > lowering_count(&before_refresh, std::any::type_name::<SecondWindow>());
            let status = if observed_now.idle_control {
                if observed_now.reduced_motion == Some(true)
                    && !observed_now.refresh_dispatched
                    && !first_advanced
                {
                    "idle_control_complete"
                } else {
                    "idle_control_rejected"
                }
            } else if observed_now.reduced_motion == Some(true)
                && observed_now.refresh_dispatched
                && first_advanced
                && second_advanced
            {
                "complete"
            } else {
                "rejected"
            };
            write_result(
                &observed_now,
                &before_refresh,
                &after_refresh,
                started.elapsed().as_millis(),
                status,
            );
            drop(observed_now);
            cx.update(|cx| cx.quit());
        })
        .detach();
        cx.activate(true);
    });
}
