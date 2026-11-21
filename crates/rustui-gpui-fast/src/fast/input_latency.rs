// Added by RustUI for PX-2. Diagnostics only; disabled unless explicitly requested.
use crate::{Window, WindowId};
use std::{
    cell::Cell,
    collections::HashMap,
    fmt::Write as _,
    io::Write as _,
    sync::{Mutex, Once, OnceLock},
    time::{Instant, SystemTime},
};

#[derive(Clone, Copy, Debug)]
#[doc(hidden)]
pub struct FastPresentTag {
    window: WindowId,
    scene: &'static str,
    first: u64,
    version: u64,
}
#[derive(Default)]
struct Versions {
    latest: Option<FastPresentTag>,
    painted: Option<FastPresentTag>,
}
enum Record {
    Input(FastPresentTag, f64, f64),
    Paint(FastPresentTag, f64, f64),
    Presented(FastPresentTag, f64, f64, f64, bool),
    Reason(&'static str, &'static str, f64),
}
#[derive(Default)]
struct Recorder {
    windows: HashMap<WindowId, Versions>,
    records: Vec<Record>,
}
static RECORDER: Mutex<Option<Recorder>> = Mutex::new(None);
thread_local! { static CURRENT: Cell<Option<FastPresentTag>> = const { Cell::new(None) }; }
fn epoch_ms() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * 1000.
}
fn mono_ms() -> f64 {
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    ORIGIN.get_or_init(Instant::now).elapsed().as_secs_f64() * 1000.
}
fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("RUSTUI_G0_LATENCY").as_deref() == Ok("1"))
}
fn frame_log() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        matches!(
            std::env::var("RUSTUI_FRAME_LOG").as_deref(),
            Ok("1" | "buffered")
        )
    })
}
fn with_recorder(f: impl FnOnce(&mut Recorder)) {
    static EXIT: Once = Once::new();
    EXIT.call_once(|| {
        unsafe extern "C" {
            fn atexit(callback: extern "C" fn()) -> std::ffi::c_int;
        }
        extern "C" fn at_exit() {
            flush();
        }
        // Diagnostic data is written at normal exit, outside measured windows.
        let _ = unsafe { atexit(at_exit) };
    });
    if let Ok(mut recorder) = RECORDER.lock() {
        f(recorder.get_or_insert_with(Recorder::default));
    }
}
#[doc(hidden)]
pub struct FastInputStamp {
    tag: FastPresentTag,
    mono: f64,
    epoch: f64,
}
impl Window {
    /// Timestamp handling before dispatch, but do not publish an unhandled state.
    /// `first` is the earliest version still visible, never overwritten states.
    #[doc(hidden)]
    pub fn fast_measure_input(
        &self,
        scene: &'static str,
        version: u64,
        first: u64,
    ) -> Option<FastInputStamp> {
        if !enabled() {
            return None;
        }
        super::latency_breakdown::handled(self.window_handle().window_id(), scene, first, version);
        Some(FastInputStamp {
            tag: FastPresentTag {
                window: self.window_handle().window_id(),
                scene,
                first,
                version,
            },
            mono: mono_ms(),
            epoch: epoch_ms(),
        })
    }
    /// Publish only after the synchronous handler has committed the G0 state.
    #[doc(hidden)]
    pub fn fast_input_handled(&self, stamp: Option<FastInputStamp>) {
        let Some(FastInputStamp { tag, mono, epoch }) = stamp else {
            return;
        };
        assert_eq!(tag.window, self.window_handle().window_id());
        with_recorder(|r| {
            r.windows.entry(tag.window).or_default().latest = Some(tag);
            r.records.push(Record::Input(tag, mono, epoch));
        });
    }
}
pub(crate) fn painted(window: WindowId) {
    if !enabled() {
        return;
    }
    with_recorder(|r| {
        let Some(versions) = r.windows.get_mut(&window) else {
            return;
        };
        versions.painted = versions.latest;
        if let Some(tag) = versions.painted {
            super::latency_breakdown::painted(window, tag.scene, tag.first, tag.version);
            r.records.push(Record::Paint(tag, mono_ms(), epoch_ms()));
        }
    });
}
pub(crate) struct PresentScope(Option<FastPresentTag>);
pub(crate) fn present_scope(window: WindowId) -> PresentScope {
    let mut tag = None;
    if enabled() {
        with_recorder(|r| {
            tag = r.windows.get(&window).and_then(|w| w.painted);
        });
    }
    PresentScope(CURRENT.with(|current| current.replace(tag)))
}
impl Drop for PresentScope {
    fn drop(&mut self) {
        CURRENT.with(|current| current.set(self.0));
    }
}
#[doc(hidden)]
pub fn fast_present_tag() -> Option<FastPresentTag> {
    CURRENT.with(Cell::get)
}
/// `presented` is MTLDrawable.presentedTime, not submit/completion time.
/// Subtract callback lateness on the same Core Animation host clock, then map
/// to the recorder's monotonic timeline. Callbacks may arrive out of order;
/// consumers sort actual display times before matching the first shown frame.
#[doc(hidden)]
pub fn fast_frame_presented(tag: FastPresentTag, presented: f64, callback_host: f64) {
    let (now, epoch) = (mono_ms(), epoch_ms());
    let valid = presented > 0.
        && presented.is_finite()
        && callback_host.is_finite()
        && callback_host >= presented;
    let late = (callback_host - presented) * 1000.;
    let display = if valid { now - late } else { now };
    with_recorder(|r| {
        r.records.push(Record::Presented(
            tag,
            display,
            if valid { epoch - late } else { epoch },
            if presented.is_finite() {
                presented
            } else {
                0.0
            },
            valid,
        ))
    });
}
#[doc(hidden)]
pub fn fast_frame_reason(mode: &'static str, reason: &'static str) {
    if mode == "partial" || !frame_log() {
        return;
    }
    let epoch = epoch_ms();
    with_recorder(|r| r.records.push(Record::Reason(mode, reason, epoch)));
}
fn append(out: &mut String, record: &Record) {
    match record {
        Record::Input(t, mono, epoch) | Record::Paint(t, mono, epoch) => {
            let event = if matches!(record, Record::Input(..)) {
                "input"
            } else {
                "paint"
            };
            let _ = writeln!(
                out,
                "RUSTUI_INPUT_LOG {{\"event\":\"{event}\",\"window\":\"{:?}\",\"scene\":\"{}\",\"version\":{},\"first_visible\":{},\"mono_ms\":{mono:.9},\"epoch_ms\":{epoch:.6}}}",
                t.window, t.scene, t.version, t.first
            );
        }
        Record::Presented(t, mono, epoch, host, valid) => {
            let _ = writeln!(
                out,
                "RUSTUI_INPUT_LOG {{\"event\":\"presented\",\"window\":\"{:?}\",\"scene\":\"{}\",\"version\":{},\"first_visible\":{},\"mono_ms\":{mono:.9},\"epoch_ms\":{epoch:.6},\"presented_time\":{host:.9},\"valid\":{valid}}}",
                t.window, t.scene, t.version, t.first
            );
        }
        Record::Reason(mode, reason, epoch) => {
            let _ = writeln!(
                out,
                "RUSTUI_FRAME_LOG {{\"event\":\"frame_reason\",\"mode\":\"{mode}\",\"reason\":\"{reason}\",\"cpu_ms\":0.0,\"epoch_ms\":{epoch:.6}}}"
            );
        }
    }
}
fn flush() {
    let Ok(mut recorder) = RECORDER.lock() else {
        return;
    };
    let Some(recorder) = recorder.as_mut() else {
        return;
    };
    let mut output = String::new();
    for record in &recorder.records {
        append(&mut output, record);
    }
    recorder.records.clear();
    let _ = std::io::stderr().lock().write_all(output.as_bytes());
}
