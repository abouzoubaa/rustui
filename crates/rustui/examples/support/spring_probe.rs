//! Opt-in scheduling probe for the public state-driven SpringOpacity wrapper.

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use gpui::{App, AppContext as _, Bounds, WindowBounds, WindowOptions, px, size};
use rustui::{
    prelude::{Container, SpringOpacity, Text, VStack, View, ViewHost},
    runtime::render_diagnostics::{
        RenderDiagnosticsSnapshot, clear_render_diagnostics, enable_render_diagnostics,
        render_diagnostics_snapshot,
    },
};
use serde_json::{Value, json};

const POLL: Duration = Duration::from_millis(25);
const STARTUP_STABILITY: Duration = Duration::from_millis(750);
const STARTUP_TIMEOUT: Duration = Duration::from_secs(3);
const RETARGET_AFTER_FIRST_RENDER: Duration = Duration::from_millis(250);
const PASSIVE_AFTER_SECOND_RENDER: Duration = Duration::from_millis(250);
const SETTLE_AFTER_RETARGET: Duration = Duration::from_secs(2);
const QUIESCENCE_WINDOW: Duration = Duration::from_millis(750);
const PROCESS_TIMEOUT: Duration = Duration::from_secs(10);
const INITIAL_TARGET: f32 = 0.15;
const FIRST_TARGET: f32 = 1.0;

struct SpringProbe {
    target: f32,
}

impl View for SpringProbe {
    fn body(&self) -> impl View {
        VStack::new()
            .child(Text("SpringOpacity scheduling probe"))
            .child(
                SpringOpacity::new(
                    "spring-frame-probe",
                    Container(Text("Stable spring identity across target changes")),
                    self.target,
                )
                .expect("fixed probe target is valid"),
            )
    }
}

fn lowering_count(snapshot: &RenderDiagnosticsSnapshot) -> u64 {
    let probe_type = std::any::type_name::<SpringProbe>();
    snapshot
        .hosts
        .iter()
        .find(|host| host.component_type == probe_type)
        .map_or(0, |host| host.count)
}

#[derive(Default)]
struct Samples {
    startup_stable_lowerings: Option<u64>,
    startup_wait_ms: Option<u128>,
    first_update_ms: Option<u128>,
    first_render_ms: Option<u128>,
    first_render_count: Option<u64>,
    before_retarget_ms: Option<u128>,
    before_retarget_count: Option<u64>,
    retarget_ms: Option<u128>,
    second_render_ms: Option<u128>,
    second_render_count: Option<u64>,
    after_retarget_ms: Option<u128>,
    after_retarget_count: Option<u64>,
    settled_ms: Option<u128>,
    settled_count: Option<u64>,
    quiescence_ms: Option<u128>,
    quiescence_count: Option<u64>,
}

fn finish(cx: &mut App, requested_reduce_motion: bool, samples: Samples, error: Option<&str>) {
    let effective_reduce_motion = cx.reduce_motion();
    let first_growth = samples
        .first_render_count
        .zip(samples.before_retarget_count)
        .is_some_and(|(first, later)| later > first);
    let second_growth = samples
        .second_render_count
        .zip(samples.after_retarget_count)
        .is_some_and(|(first, later)| later > first);
    let first_flat = samples
        .first_render_count
        .zip(samples.before_retarget_count)
        .is_some_and(|(first, later)| later == first);
    let second_flat = samples
        .second_render_count
        .zip(samples.after_retarget_count)
        .is_some_and(|(first, later)| later == first);
    let quiescent = samples
        .settled_count
        .zip(samples.quiescence_count)
        .is_some_and(|(first, later)| later == first);
    let ordered = [
        samples.first_update_ms,
        samples.first_render_ms,
        samples.before_retarget_ms,
        samples.retarget_ms,
        samples.second_render_ms,
        samples.after_retarget_ms,
        samples.settled_ms,
        samples.quiescence_ms,
    ]
    .into_iter()
    .collect::<Option<Vec<_>>>()
    .is_some_and(|times| times.windows(2).all(|pair| pair[0] <= pair[1]));
    let complete = error.is_none()
        && requested_reduce_motion == effective_reduce_motion
        && ordered
        && samples.startup_stable_lowerings.is_some_and(|count| count > 0)
        && samples.first_render_count.is_some_and(|count| count > 0)
        && samples.second_render_count.zip(samples.before_retarget_count).is_some_and(|(second, before)| second > before)
        && if requested_reduce_motion {
            first_flat && second_flat && quiescent
        } else {
            first_growth && second_growth && quiescent
        };
    let status = if error.is_some() {
        "failed"
    } else if complete {
        "complete"
    } else {
        "rejected"
    };
    let result = json!({
        "kind": "rustui_spring_opacity_scheduling_probe",
        "status": status,
        "mode": if requested_reduce_motion { "reduce_motion" } else { "normal" },
        "error": error,
        "requested_reduce_motion": requested_reduce_motion,
        "effective_reduce_motion": effective_reduce_motion,
        "target_sequence": [INITIAL_TARGET, FIRST_TARGET, INITIAL_TARGET],
        "startup_stability_ms": STARTUP_STABILITY.as_millis(),
        "startup_timeout_ms": STARTUP_TIMEOUT.as_millis(),
        "startup_wait_ms": samples.startup_wait_ms,
        "startup_stable_lowerings": samples.startup_stable_lowerings,
        "poll_interval_ms": POLL.as_millis(),
        "retarget_after_first_render_ms": RETARGET_AFTER_FIRST_RENDER.as_millis(),
        "passive_after_second_render_ms": PASSIVE_AFTER_SECOND_RENDER.as_millis(),
        "settle_after_retarget_ms": SETTLE_AFTER_RETARGET.as_millis(),
        "quiescence_window_ms": QUIESCENCE_WINDOW.as_millis(),
        "process_timeout_ms": PROCESS_TIMEOUT.as_millis(),
        "event_elapsed_ms": {
            "first_update": samples.first_update_ms,
            "retarget": samples.retarget_ms,
        },
        "sample_elapsed_ms": {
            "first_render": samples.first_render_ms,
            "before_retarget": samples.before_retarget_ms,
            "second_render": samples.second_render_ms,
            "after_retarget": samples.after_retarget_ms,
            "settled": samples.settled_ms,
            "quiescence": samples.quiescence_ms,
        },
        "root_host_lowerings": {
            "first_render": samples.first_render_count,
            "before_retarget": samples.before_retarget_count,
            "second_render": samples.second_render_count,
            "after_retarget": samples.after_retarget_count,
            "settled": samples.settled_count,
            "quiescence": samples.quiescence_count,
        },
        "first_motion_growth": first_growth,
        "retarget_motion_growth": second_growth,
        "first_passive_flat": first_flat,
        "second_passive_flat": second_flat,
        "quiescent": quiescent,
        "timings_monotonic": ordered,
        "measurement_scope": "RustUI root host lowerings only; not exact GPUI frames, presented alpha, or spring velocity continuity",
    });
    if let Err(write_error) = write_result(&result) {
        eprintln!("spring probe could not write result: {write_error}");
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
        "spring-opacity-scheduling-{}-{timestamp}.json",
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

pub(super) fn run(reduce_motion: bool) {
    gpui_platform::application().run(move |cx| {
        gpui_component::init(cx);
        cx.set_reduce_motion(reduce_motion);
        enable_render_diagnostics(cx);

        let started = Instant::now();
        let root = cx.new(|cx| ViewHost::new(SpringProbe { target: INITIAL_TARGET }, cx));
        let bounds = Bounds::centered(None, size(px(460.0), px(180.0)), cx);
        let window_root = root.clone();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_, _| window_root.clone(),
        )
        .expect("spring probe window must open");

        cx.spawn(async move |cx| {
            let startup_started = Instant::now();
            let mut stable_count = None;
            let mut stable_since = None;
            let mut samples = Samples::default();
            loop {
                cx.background_executor().timer(POLL).await;
                let count = cx.update(|cx| lowering_count(&render_diagnostics_snapshot(cx)));
                if count > 0 && stable_count == Some(count) {
                    if stable_since.is_some_and(|since: Instant| since.elapsed() >= STARTUP_STABILITY) {
                        samples.startup_stable_lowerings = Some(count);
                        samples.startup_wait_ms = Some(startup_started.elapsed().as_millis());
                        break;
                    }
                } else {
                    stable_count = (count > 0).then_some(count);
                    stable_since = (count > 0).then(Instant::now);
                }
                if startup_started.elapsed() >= STARTUP_TIMEOUT {
                    cx.update(|cx| finish(cx, reduce_motion, samples, Some("static startup baseline did not stabilize")));
                    return;
                }
            }

            cx.update(|cx| {
                clear_render_diagnostics(cx);
                root.update(cx, |host, cx| {
                    host.component_mut().target = FIRST_TARGET;
                    cx.notify();
                });
            });
            samples.first_update_ms = Some(started.elapsed().as_millis());
            let first_started = Instant::now();
            loop {
                cx.background_executor().timer(POLL).await;
                let count = cx.update(|cx| lowering_count(&render_diagnostics_snapshot(cx)));
                if count > 0 {
                    samples.first_render_count = Some(count);
                    samples.first_render_ms = Some(started.elapsed().as_millis());
                    break;
                }
                if first_started.elapsed() >= STARTUP_TIMEOUT {
                    cx.update(|cx| finish(cx, reduce_motion, samples, Some("first target render did not appear")));
                    return;
                }
            }

            cx.background_executor().timer(RETARGET_AFTER_FIRST_RENDER).await;
            samples.before_retarget_count = Some(cx.update(|cx| lowering_count(&render_diagnostics_snapshot(cx))));
            samples.before_retarget_ms = Some(started.elapsed().as_millis());

            cx.update(|cx| {
                root.update(cx, |host, cx| {
                    host.component_mut().target = INITIAL_TARGET;
                    cx.notify();
                });
            });
            samples.retarget_ms = Some(started.elapsed().as_millis());
            let retarget_started = Instant::now();
            loop {
                cx.background_executor().timer(POLL).await;
                let count = cx.update(|cx| lowering_count(&render_diagnostics_snapshot(cx)));
                if count > samples.before_retarget_count.unwrap_or(0) {
                    samples.second_render_count = Some(count);
                    samples.second_render_ms = Some(started.elapsed().as_millis());
                    break;
                }
                if retarget_started.elapsed() >= STARTUP_TIMEOUT {
                    cx.update(|cx| finish(cx, reduce_motion, samples, Some("retarget render did not appear")));
                    return;
                }
            }

            cx.background_executor().timer(PASSIVE_AFTER_SECOND_RENDER).await;
            samples.after_retarget_count = Some(cx.update(|cx| lowering_count(&render_diagnostics_snapshot(cx))));
            samples.after_retarget_ms = Some(started.elapsed().as_millis());
            cx.background_executor()
                .timer(SETTLE_AFTER_RETARGET.saturating_sub(retarget_started.elapsed()))
                .await;
            samples.settled_count = Some(cx.update(|cx| lowering_count(&render_diagnostics_snapshot(cx))));
            samples.settled_ms = Some(started.elapsed().as_millis());
            cx.background_executor().timer(QUIESCENCE_WINDOW).await;
            samples.quiescence_count = Some(cx.update(|cx| lowering_count(&render_diagnostics_snapshot(cx))));
            samples.quiescence_ms = Some(started.elapsed().as_millis());
            cx.update(|cx| finish(cx, reduce_motion, samples, None));
        })
        .detach();

        cx.spawn(async move |cx| {
            cx.background_executor().timer(PROCESS_TIMEOUT).await;
            cx.update(|cx| finish(cx, reduce_motion, Samples::default(), Some("10-second runtime timeout")));
        })
        .detach();
        cx.activate(true);
    });
}
