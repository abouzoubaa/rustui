//! Opt-in runtime check for GPUI-scheduled `AnimatedOpacity` host lowerings.
//!
//! Run normally and with `--reduce-motion` in separate processes. Add
//! `--limit-fps` to request native 10-FPS scheduling in either mode. The sampler
//! only reads diagnostics snapshots; it never notifies views or requests frames.

#[path = "support/spring_probe.rs"]
mod spring_probe;

use std::{
    fs::{self, OpenOptions},
    io::Write,
    num::NonZeroU16,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use gpui::{App, AppContext as _, Bounds, WindowBounds, WindowOptions, px, size};
use rustui::{
    prelude::{AnimatedOpacity, Container, Text, VStack, View, ViewHost},
    runtime::render_diagnostics::{
        RenderDiagnosticsSnapshot, clear_render_diagnostics, enable_render_diagnostics,
        render_diagnostics_snapshot,
    },
};
use serde_json::{Value, json};

const PROBE_MAX_FPS: NonZeroU16 = NonZeroU16::new(10).expect("10 is nonzero");
const ANIMATION_DURATION: Duration = Duration::from_secs(1);
const BASELINE_POLL_INTERVAL: Duration = Duration::from_millis(25);
const STARTUP_STABILITY: Duration = Duration::from_millis(750);
const STARTUP_TIMEOUT: Duration = Duration::from_secs(3);
const MIDPOINT_AFTER_BASELINE: Duration = Duration::from_millis(500);
const POST_DURATION_AFTER_BASELINE: Duration = Duration::from_millis(1_200);
const QUIESCENCE_AFTER_POST_DURATION: Duration = Duration::from_millis(1_500);
const MAX_BASELINE_POLL_INTERVAL: Duration = Duration::from_millis(250);
const PROCESS_TIMEOUT: Duration = Duration::from_secs(10);

struct OpacityProbe {
    mounted: bool,
    max_fps: Option<NonZeroU16>,
}

impl View for OpacityProbe {
    fn body(&self) -> impl View {
        let content = VStack::new()
            .spacing(8.0)
            .child(Text("AnimatedOpacity runtime frame probe"));
        if self.mounted {
            let fade = AnimatedOpacity::new(
                "opacity-frame-probe",
                Container(Text("This child fades from zero to full opacity.")),
                0.0,
                1.0,
                ANIMATION_DURATION,
            )
            .expect("fixed opacity probe parameters are valid");
            content.child(match self.max_fps {
                Some(max_fps) => fade.max_fps(max_fps),
                None => fade,
            })
        } else {
            content.child(Text("Waiting for static startup baseline."))
        }
    }
}

fn lowering_count(snapshot: &RenderDiagnosticsSnapshot) -> u64 {
    let probe_type = std::any::type_name::<OpacityProbe>();
    snapshot
        .hosts
        .iter()
        .find(|host| host.component_type == probe_type)
        .map_or(0, |host| host.count)
}

#[derive(Default)]
struct ProbeSamples {
    baseline: RenderDiagnosticsSnapshot,
    midpoint: RenderDiagnosticsSnapshot,
    post_duration: RenderDiagnosticsSnapshot,
    quiescence: RenderDiagnosticsSnapshot,
    baseline_ms: Option<u128>,
    baseline_poll_interval_ms: Option<u128>,
    baseline_stability_ms: Option<u128>,
    startup_wait_ms: Option<u128>,
    startup_stable_lowerings: Option<u64>,
    mount_ms: Option<u128>,
    midpoint_ms: Option<u128>,
    post_duration_ms: Option<u128>,
    quiescence_ms: Option<u128>,
}

fn finish(
    cx: &mut App,
    reduce_motion: bool,
    max_fps: Option<NonZeroU16>,
    samples: ProbeSamples,
    error: Option<&str>,
) {
    let baseline_count = lowering_count(&samples.baseline);
    let midpoint_count = lowering_count(&samples.midpoint);
    let post_duration_count = lowering_count(&samples.post_duration);
    let quiescence_count = lowering_count(&samples.quiescence);
    let baseline_timing_valid = samples
        .baseline_poll_interval_ms
        .is_some_and(|elapsed| elapsed <= MAX_BASELINE_POLL_INTERVAL.as_millis());
    let effective_reduce_motion = cx.reduce_motion();
    let reduced_motion_confirmed = effective_reduce_motion == reduce_motion;
    let normal_progress_observed =
        !reduce_motion && midpoint_count > baseline_count && post_duration_count > midpoint_count;
    let quiescent_after_animation = quiescence_count == post_duration_count;

    let status = if error.is_some() || !reduced_motion_confirmed || !baseline_timing_valid {
        "failed"
    } else if reduce_motion {
        if baseline_count > 0
            && midpoint_count == baseline_count
            && post_duration_count == midpoint_count
            && quiescence_count == post_duration_count
        {
            "complete"
        } else {
            "rejected"
        }
    } else if normal_progress_observed && quiescent_after_animation {
        "complete"
    } else {
        "rejected"
    };

    let result = json!({
        "kind": "rustui_opacity_frame_acceptance",
        "status": status,
        "mode": match (reduce_motion, max_fps.is_some()) {
            (false, false) => "normal", (true, false) => "reduce_motion",
            (false, true) => "limited", (true, true) => "limited_reduce_motion",
        },
        "configured_max_fps": max_fps.map(NonZeroU16::get),
        "error": error,
        "requested_reduce_motion": reduce_motion,
        "effective_reduce_motion": effective_reduce_motion,
        "reduce_motion_confirmed": reduced_motion_confirmed,
        "animation_duration_ms": ANIMATION_DURATION.as_millis(),
        "midpoint_after_baseline_ms": MIDPOINT_AFTER_BASELINE.as_millis(),
        "post_duration_after_baseline_ms": POST_DURATION_AFTER_BASELINE.as_millis(),
        "quiescence_after_post_duration_ms": QUIESCENCE_AFTER_POST_DURATION.as_millis(),
        "max_baseline_poll_interval_ms": MAX_BASELINE_POLL_INTERVAL.as_millis(),
        "startup_stability_ms": STARTUP_STABILITY.as_millis(),
        "startup_timeout_ms": STARTUP_TIMEOUT.as_millis(),
        "process_timeout_ms": PROCESS_TIMEOUT.as_millis(),
        "root_host_lowerings": {
            "baseline": baseline_count,
            "midpoint": midpoint_count,
            "post_duration": post_duration_count,
            "quiescence": quiescence_count,
        },
        "sample_elapsed_ms": {
            "baseline": samples.baseline_ms,
            "midpoint": samples.midpoint_ms,
            "post_duration": samples.post_duration_ms,
            "quiescence": samples.quiescence_ms,
        },
        "baseline_poll_interval_ms": samples.baseline_poll_interval_ms,
        "baseline_stability_ms": samples.baseline_stability_ms,
        "startup_wait_ms": samples.startup_wait_ms,
        "startup_stable_lowerings": samples.startup_stable_lowerings,
        "mount_elapsed_ms": samples.mount_ms,
        "baseline_timing_valid": baseline_timing_valid,
        "normal_progress_observed": normal_progress_observed,
        "quiescent_after_animation": quiescent_after_animation,
        "reduced_motion_endpoint": if reduce_motion {
            "GPUI one-shot contract resolves to to=1.0"
        } else {
            "not applicable"
        },
        "measurement_scope": "RustUI host lowerings only; not exact GPUI frame count, GPU frames, presentation timing, or captured pixel alpha",
    });
    if let Err(write_error) = write_result(&result) {
        eprintln!("opacity frame probe could not write result: {write_error}");
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
        "opacity-frame-acceptance-{}-{timestamp}.json",
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

fn init_probe_theme(cx: &mut App) {
    let theme = rustui::theme::parse_theme(include_str!("../../../themes/macos27.json"))
        .expect("bundled probe theme must be valid");
    rustui::theme::activate_theme(&theme, None, cx).expect("probe theme must activate");
}

fn main() {
    if std::env::args().any(|argument| argument == "--spring") {
        if std::env::args().any(|argument| argument == "--limit-fps") {
            eprintln!("--spring does not support --limit-fps");
            std::process::exit(2);
        }
        spring_probe::run(std::env::args().any(|argument| argument == "--reduce-motion"));
        return;
    }
    let reduce_motion = std::env::args().any(|argument| argument == "--reduce-motion");
    let max_fps = std::env::args()
        .any(|argument| argument == "--limit-fps")
        .then_some(PROBE_MAX_FPS);
    gpui_platform::application().run(move |cx| {
        gpui_component::init(cx);
        init_probe_theme(cx);
        cx.set_reduce_motion(reduce_motion);
        enable_render_diagnostics(cx);

        let sampling_started = Instant::now();
        let root = cx.new(|cx| {
            ViewHost::new(
                OpacityProbe {
                    mounted: false,
                    max_fps,
                },
                cx,
            )
        });
        let bounds = Bounds::centered(None, size(px(460.0), px(180.0)), cx);
        let window_root = root.clone();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_, _| window_root.clone(),
        )
        .expect("opacity frame probe window must open");

        cx.spawn(async move |cx| {
            let startup_started = Instant::now();
            let mut stable_count = None;
            let mut stable_since = None;
            let mut startup_stable_lowerings = 0;
            loop {
                cx.background_executor().timer(BASELINE_POLL_INTERVAL).await;
                let snapshot = cx.update(|cx| render_diagnostics_snapshot(cx));
                let count = lowering_count(&snapshot);
                if count > 0 && stable_count == Some(count) {
                    if stable_since
                        .is_some_and(|started: Instant| started.elapsed() >= STARTUP_STABILITY)
                    {
                        startup_stable_lowerings = count;
                        break;
                    }
                } else {
                    stable_count = (count > 0).then_some(count);
                    stable_since = (count > 0).then(Instant::now);
                }
                if startup_started.elapsed() >= STARTUP_TIMEOUT {
                    cx.update(|cx| {
                        finish(
                            cx,
                            reduce_motion,
                            max_fps,
                            ProbeSamples::default(),
                            Some("static startup baseline did not stabilize within 3 seconds"),
                        )
                    });
                    return;
                }
            }
            let startup_wait_ms = startup_started.elapsed().as_millis();
            let mount_ms = cx.update(|cx| {
                clear_render_diagnostics(cx);
                root.update(cx, |host, cx| {
                    host.component_mut().mounted = true;
                    cx.notify();
                });
                sampling_started.elapsed().as_millis()
            });
            let mount_started = Instant::now();
            let (baseline, baseline_poll_interval_ms) = loop {
                let poll_started = Instant::now();
                cx.background_executor().timer(BASELINE_POLL_INTERVAL).await;
                let snapshot = cx.update(|cx| render_diagnostics_snapshot(cx));
                if lowering_count(&snapshot) > 0 {
                    break (snapshot, Some(poll_started.elapsed().as_millis()));
                }
                if mount_started.elapsed() > STARTUP_TIMEOUT {
                    cx.update(|cx| {
                        finish(
                            cx,
                            reduce_motion,
                            max_fps,
                            ProbeSamples::default(),
                            Some("mounted animation baseline did not appear within 3 seconds"),
                        )
                    });
                    return;
                }
            };
            let mut samples = ProbeSamples {
                baseline,
                baseline_ms: Some(sampling_started.elapsed().as_millis()),
                baseline_poll_interval_ms,
                startup_wait_ms: Some(startup_wait_ms),
                startup_stable_lowerings: Some(startup_stable_lowerings),
                mount_ms: Some(mount_ms),
                ..ProbeSamples::default()
            };

            cx.background_executor()
                .timer(MIDPOINT_AFTER_BASELINE)
                .await;
            samples.midpoint = cx.update(|cx| render_diagnostics_snapshot(cx));
            samples.midpoint_ms = Some(sampling_started.elapsed().as_millis());

            cx.background_executor()
                .timer(POST_DURATION_AFTER_BASELINE - MIDPOINT_AFTER_BASELINE)
                .await;
            samples.post_duration = cx.update(|cx| render_diagnostics_snapshot(cx));
            samples.post_duration_ms = Some(sampling_started.elapsed().as_millis());

            cx.background_executor()
                .timer(QUIESCENCE_AFTER_POST_DURATION)
                .await;
            samples.quiescence = cx.update(|cx| render_diagnostics_snapshot(cx));
            samples.quiescence_ms = Some(sampling_started.elapsed().as_millis());

            cx.update(|cx| finish(cx, reduce_motion, max_fps, samples, None));
        })
        .detach();

        let timeout_reduce_motion = reduce_motion;
        cx.spawn(async move |cx| {
            cx.background_executor().timer(PROCESS_TIMEOUT).await;
            cx.update(|cx| {
                finish(
                    cx,
                    timeout_reduce_motion,
                    max_fps,
                    ProbeSamples::default(),
                    Some("10-second runtime timeout"),
                )
            });
        })
        .detach();
        cx.activate(true);
    });
}
