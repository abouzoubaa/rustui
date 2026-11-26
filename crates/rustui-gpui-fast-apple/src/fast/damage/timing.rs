// SPDX-License-Identifier: Apache-2.0
//! Fixed-capacity, exit-flushed samples for CPU attribution of Metal frames.
use std::{
    cell::RefCell,
    io::{self, Write as _},
    sync::{Mutex, OnceLock},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

const MAX_SAMPLES: usize = 16384;

#[derive(Clone, Copy)]
pub(crate) enum Reason {
    Admission,
    Disabled,
    UnsupportedScene,
    LiveResize,
    Geometry,
    Verification,
    Backoff,
    Prime,
    AtlasLost,
    Large,
    ShiftUnsupported,
    AreaLimit,
    Deferral,
    RenderError,
    Engaged,
}
impl Reason {
    pub(crate) fn selected(reason: &str) -> Self {
        match reason {
            "disabled" => Self::Disabled,
            "unsupported_scene" | "unsupported_scroll_layer" | "surface" => Self::UnsupportedScene,
            "live_resize" => Self::LiveResize,
            "geometry" => Self::Geometry,
            "verify_off_path" => Self::Verification,
            "backoff" => Self::Backoff,
            "prime" => Self::Prime,
            "atlas_lost" => Self::AtlasLost,
            "shift_unsupported" => Self::ShiftUnsupported,
            "area_bounding_box" => Self::AreaLimit,
            "large" => Self::Large,
            _ => Self::Engaged,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Admission => "admission",
            Self::Disabled => "disabled",
            Self::UnsupportedScene => "unsupported_scene",
            Self::LiveResize => "live_resize",
            Self::Geometry => "geometry",
            Self::Verification => "allocation_or_verify",
            Self::Backoff => "backoff",
            Self::Prime => "prime",
            Self::AtlasLost => "atlas_lost",
            Self::Large => "large",
            Self::ShiftUnsupported => "shift_unsupported",
            Self::AreaLimit => "area_bounding_box",
            Self::Deferral => "deferred",
            Self::RenderError => "render_error",
            Self::Engaged => "engaged",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Phase {
    Plan,
    Encode,
    Present,
}

#[derive(Clone, Copy)]
pub(crate) enum PlanStage {
    Preflight,
    AtlasAndBackoff,
    PrimeSnapshot,
    SignatureCollection,
    FixedCompare,
    MatchUnchanged,
    ScrollVote,
    PaintOrder,
    ShiftMatch,
    RectMerge,
    BuildPlan,
    SelectMode,
    EmitCpuRecord,
}

const PLAN_STAGES: usize = 13;
const PLAN_LABELS: [&str; PLAN_STAGES] = [
    "preflight_us",
    "atlas_and_backoff_us",
    "prime_snapshot_us",
    "signature_collection_us",
    "fixed_compare_us",
    "match_unchanged_us",
    "scroll_vote_us",
    "paint_order_us",
    "shift_match_us",
    "rect_merge_us",
    "build_plan_us",
    "select_mode_us",
    "emit_cpu_record_us",
];

struct PlanPhases {
    last: Instant,
    stage: PlanStage,
    elapsed: [u64; PLAN_STAGES],
}
impl PlanPhases {
    fn advance(&mut self, now: Instant, stage: PlanStage) {
        self.elapsed[self.stage as usize] += (now - self.last).as_nanos() as u64;
        self.last = now;
        self.stage = stage;
    }
}

thread_local! {
    static PLAN: RefCell<Option<PlanPhases>> = const { RefCell::new(None) };
}

#[inline]
pub(crate) fn plan_stage(stage: PlanStage) {
    if enabled() {
        PLAN.with(|phases| {
            if let Some(phases) = phases.borrow_mut().as_mut() {
                phases.advance(Instant::now(), stage);
            }
        });
    }
}

#[derive(Clone, Copy)]
struct Sample {
    elapsed_ns: [u64; 3],
    plan_stages_ns: [u64; PLAN_STAGES],
    reason: Reason,
    mode: &'static str,
    epoch_ms: f64,
}

struct Recorder {
    samples: [Option<Sample>; MAX_SAMPLES],
    len: usize,
    lost: usize,
}
static RECORDER: Mutex<Recorder> = Mutex::new(Recorder {
    samples: [None; MAX_SAMPLES],
    len: 0,
    lost: 0,
});

extern "C" fn flush() {
    let Ok(recorder) = RECORDER.lock() else {
        return;
    };
    let mut stderr = io::BufWriter::new(io::stderr().lock());
    for sample in recorder.samples[..recorder.len].iter().flatten() {
        let [plan, encode, present] = sample.elapsed_ns.map(|ns| ns as f64 / 1000.);
        let _ = write!(
            stderr,
            r#"RUSTUI_FRAME_LOG {{"event":"damage_attribution","cpu_ms":0.0,"epoch_ms":{:.6},"mode":"{}","fallback_reason":"{}","plan_us":{plan:.3},"encode_us":{encode:.3},"present_us":{present:.3}"#,
            sample.epoch_ms,
            sample.mode,
            sample.reason.label(),
        );
        for (label, ns) in PLAN_LABELS.iter().zip(sample.plan_stages_ns) {
            let _ = write!(stderr, ",\"{label}\":{:.3}", ns as f64 / 1000.);
        }
        let _ = writeln!(stderr, "}}");
    }
    let _ = writeln!(stderr, "RUSTUI_G_R2_LOST {}", recorder.lost);
}

pub(crate) fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        if std::env::var_os("RUSTUI_G_R2_ATTRIBUTION").as_deref() != Some("1".as_ref()) {
            return false;
        }
        unsafe extern "C" {
            fn atexit(callback: extern "C" fn()) -> std::ffi::c_int;
        }
        // The callback is static. It flushes the fixed buffer at normal exit.
        unsafe { atexit(flush) == 0 }
    })
}

pub(crate) struct Timing {
    _allocations: super::allocation_probe::Sample,
    last: Option<Instant>,
    phase: Phase,
    elapsed_ns: [u64; 3],
    plan_stages_ns: [u64; PLAN_STAGES],
    pub(crate) reason: Reason,
    pub(crate) mode: &'static str,
}
impl Timing {
    pub(crate) fn start() -> Self {
        Self {
            _allocations: super::allocation_probe::Sample::start("renderer"),
            last: enabled().then(Instant::now),
            phase: Phase::Present,
            elapsed_ns: [0; 3],
            plan_stages_ns: [0; PLAN_STAGES],
            reason: Reason::Admission,
            mode: "unsubmitted",
        }
    }

    pub(crate) fn phase(&mut self, phase: Phase) {
        if let Some(last) = self.last {
            let now = Instant::now();
            self.elapsed_ns[self.phase as usize] += (now - last).as_nanos() as u64;
            if self.phase == Phase::Plan {
                PLAN.with(|phases| {
                    if let Some(mut phases) = phases.borrow_mut().take() {
                        phases.advance(now, PlanStage::Preflight);
                        self.plan_stages_ns = phases.elapsed;
                    }
                });
            }
            if phase == Phase::Plan {
                PLAN.with(|phases| {
                    phases.replace(Some(PlanPhases {
                        last: now,
                        stage: PlanStage::Preflight,
                        elapsed: [0; PLAN_STAGES],
                    }));
                });
            }
            self.last = Some(now);
        }
        self.phase = phase;
    }
}
impl Drop for Timing {
    fn drop(&mut self) {
        if self.last.is_none() {
            return;
        }
        self.phase(Phase::Present);
        let epoch_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.;
        if let Ok(mut recorder) = RECORDER.lock() {
            let index = recorder.len;
            if index < MAX_SAMPLES {
                recorder.samples[index] = Some(Sample {
                    elapsed_ns: self.elapsed_ns,
                    plan_stages_ns: self.plan_stages_ns,
                    reason: self.reason,
                    mode: self.mode,
                    epoch_ms,
                });
                recorder.len += 1;
            } else {
                recorder.lost += 1;
            }
        }
    }
}
