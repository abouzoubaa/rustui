// RustUI PX-4: opt-in observation only. Raw mach ticks are buffered until exit.
use crate::WindowId;
use std::{
    cell::Cell,
    collections::HashMap,
    io::Write as _,
    sync::{Mutex, Once, OnceLock},
};
#[derive(Clone, Copy, Debug)]
pub struct FastLatencyFrame {
    window: WindowId,
    attempt: u64,
    draw: u64,
}
#[derive(Clone, Copy)]
pub struct FastLatencyDrawable {
    window: WindowId,
    id: u64,
    key: u64,
}
#[derive(Clone, Copy)]
pub struct FastLatencyCommit {
    frame: FastLatencyFrame,
    drawable: u64,
}
#[derive(Default)]
struct Win {
    draw: u64,
    callback: u64,
    attempt: u64,
    acquired: u64,
    live: HashMap<u64, u64>,
}
enum Rec {
    Point(&'static str, WindowId, u64, u64, u64),
    Input(WindowId, &'static str, u64, u64, u64),
    Paint(WindowId, &'static str, u64, u64, u64, u64),
    Attempt(FastLatencyFrame, u64, u64),
    Acquire(FastLatencyDrawable, usize, u64),
    Release(FastLatencyDrawable, &'static str, u64),
    Present(FastLatencyDrawable, u64, f64, f64),
    Commit(FastLatencyCommit, &'static str, u64, f64, f64),
    Config(WindowId, &'static str, [f64; 4], u64),
}
// A bounded, statically allocated buffer keeps timing markers allocation-free.
#[derive(Clone, Copy)]
struct PhaseSpan {
    window: Option<WindowId>,
    frame: u64,
    attempt: bool,
    kind: &'static str,
    start: u64,
    end: u64,
}

const PHASE_CAPACITY: usize = 131_072;
struct PhaseBuffer {
    spans: [Option<PhaseSpan>; PHASE_CAPACITY],
    len: usize,
    dropped: usize,
}
static PHASES: Mutex<PhaseBuffer> = Mutex::new(PhaseBuffer {
    spans: [None; PHASE_CAPACITY],
    len: 0,
    dropped: 0,
});

fn phases_enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    fast_latency_enabled()
        && *ON.get_or_init(|| {
            std::env::var_os("RUSTUI_G5_P_BREAKDOWN").as_deref() == Some("1".as_ref())
        })
}

fn push_phase(span: PhaseSpan) {
    if let Ok(mut buffer) = PHASES.lock() {
        let index = buffer.len;
        if index < PHASE_CAPACITY {
            buffer.spans[index] = Some(span);
            buffer.len += 1;
        } else {
            buffer.dropped += 1;
        }
    }
}

#[must_use]
pub struct LatencyPhase(Option<PhaseSpan>);
impl Drop for LatencyPhase {
    fn drop(&mut self) {
        if let Some(mut span) = self.0 {
            span.end = fast_latency_tick();
            push_phase(span);
        }
    }
}

pub(crate) fn draw_phase(window: WindowId, kind: &'static str) -> LatencyPhase {
    if !phases_enabled() {
        return LatencyPhase(None);
    }
    let start = fast_latency_tick();
    let frame = RECORDS
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref()?.windows.get(&window).map(|win| win.draw));
    LatencyPhase(frame.map(|frame| PhaseSpan {
        window: Some(window),
        frame,
        attempt: false,
        kind,
        start,
        end: 0,
    }))
}

pub fn fast_latency_render_phase(kind: &'static str) -> LatencyPhase {
    if !phases_enabled() {
        return LatencyPhase(None);
    }
    LatencyPhase(fast_latency_frame().map(|frame| PhaseSpan {
        window: Some(frame.window),
        frame: frame.attempt,
        attempt: true,
        kind,
        start: fast_latency_tick(),
        end: 0,
    }))
}

pub fn fast_latency_native_handler_end() {
    if phases_enabled() {
        let tick = fast_latency_tick();
        push_phase(PhaseSpan {
            window: None,
            frame: 0,
            attempt: false,
            kind: "native_handler_end",
            start: tick,
            end: tick,
        });
    }
}

#[derive(Default)]
struct Recorder {
    windows: HashMap<WindowId, Win>,
    records: Vec<Rec>,
}
static RECORDS: Mutex<Option<Recorder>> = Mutex::new(None);
thread_local! {
 static CALLBACK: Cell<Option<(WindowId,u64)>>=const {Cell::new(None)};
 static PRESENT: Cell<Option<FastLatencyFrame>>=const {Cell::new(None)};
}
pub fn fast_latency_enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("RUSTUI_LATENCY_BREAKDOWN").as_deref() == Ok("1"))
}
#[cfg(target_os = "macos")]
#[repr(C)]
struct Timebase {
    numer: u32,
    denom: u32,
}
#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn mach_absolute_time() -> u64;
    fn mach_timebase_info(info: *mut Timebase) -> i32;
}
pub fn fast_latency_tick() -> u64 {
    #[cfg(target_os = "macos")]
    {
        unsafe { mach_absolute_time() }
    }
    #[cfg(not(target_os = "macos"))]
    {
        static START: OnceLock<std::time::Instant> = OnceLock::new();
        START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_nanos() as u64
    }
}
fn timebase() -> (u32, u32) {
    #[cfg(target_os = "macos")]
    {
        static BASE: OnceLock<(u32, u32)> = OnceLock::new();
        *BASE.get_or_init(|| {
            let mut b = Timebase { numer: 0, denom: 0 };
            assert_eq!(unsafe { mach_timebase_info(&mut b) }, 0);
            assert!(b.denom > 0);
            (b.numer, b.denom)
        })
    }
    #[cfg(not(target_os = "macos"))]
    {
        (1, 1)
    }
}
fn record(f: impl FnOnce(&mut Recorder)) {
    if !fast_latency_enabled() {
        return;
    }
    static EXIT: Once = Once::new();
    EXIT.call_once(|| {
        unsafe extern "C" {
            fn atexit(f: extern "C" fn()) -> i32;
        }
        extern "C" fn exit() {
            flush()
        }
        assert_eq!(unsafe { atexit(exit) }, 0);
    });
    if let Ok(mut r) = RECORDS.lock() {
        f(r.get_or_insert_with(Recorder::default))
    }
}
pub(crate) fn handled(window: WindowId, scene: &'static str, first: u64, version: u64) {
    if !fast_latency_enabled() {
        return;
    }
    let tick = fast_latency_tick();
    record(|r| {
        r.records
            .push(Rec::Input(window, scene, first, version, tick))
    })
}
pub(crate) fn requested(window: WindowId) {
    if !fast_latency_enabled() {
        return;
    }
    let tick = fast_latency_tick();
    record(|r| r.records.push(Rec::Point("requested", window, 0, 0, tick)))
}
pub struct FastLatencyCallback(Option<(WindowId, u64)>);
impl Drop for FastLatencyCallback {
    fn drop(&mut self) {
        CALLBACK.with(|c| c.set(self.0));
    }
}
pub fn fast_latency_callback(window: WindowId, tick: u64) -> FastLatencyCallback {
    let mut callback = None;
    record(|r| {
        let w = r.windows.entry(window).or_default();
        w.callback += 1;
        callback = Some((window, w.callback));
        r.records
            .push(Rec::Point("callback", window, 0, w.callback, tick));
    });
    FastLatencyCallback(CALLBACK.with(|c| c.replace(callback)))
}
pub(crate) fn draw(window: WindowId) {
    if !fast_latency_enabled() {
        return;
    }
    let tick = fast_latency_tick();
    record(|r| {
        let w = r.windows.entry(window).or_default();
        w.draw += 1;
        let cb = CALLBACK
            .with(Cell::get)
            .filter(|c| c.0 == window)
            .map_or(0, |c| c.1);
        r.records.push(Rec::Point("draw", window, w.draw, cb, tick));
    })
}
pub(crate) fn painted(window: WindowId, scene: &'static str, first: u64, version: u64) {
    if !fast_latency_enabled() {
        return;
    }
    let tick = fast_latency_tick();
    record(|r| {
        let draw = r.windows.entry(window).or_default().draw;
        r.records
            .push(Rec::Paint(window, scene, first, version, draw, tick));
    })
}
pub(crate) struct PresentScope(Option<FastLatencyFrame>);
impl Drop for PresentScope {
    fn drop(&mut self) {
        PRESENT.with(|p| p.set(self.0));
    }
}
pub(crate) fn present_scope(window: WindowId) -> PresentScope {
    let tick = fast_latency_tick();
    let mut frame = None;
    record(|r| {
        let w = r.windows.entry(window).or_default();
        w.attempt += 1;
        let f = FastLatencyFrame {
            window,
            attempt: w.attempt,
            draw: w.draw,
        };
        let cb = CALLBACK
            .with(Cell::get)
            .filter(|c| c.0 == window)
            .map_or(0, |c| c.1);
        r.records.push(Rec::Attempt(f, cb, tick));
        frame = Some(f);
    });
    PresentScope(PRESENT.with(|p| p.replace(frame)))
}
pub fn fast_latency_frame() -> Option<FastLatencyFrame> {
    PRESENT.with(Cell::get)
}
impl FastLatencyFrame {
    pub fn window(self) -> WindowId {
        self.window
    }
}
pub fn fast_latency_acquired(window: WindowId, key: u64) -> Option<FastLatencyDrawable> {
    let tick = fast_latency_tick();
    let mut ticket = None;
    record(|r| {
        let w = r.windows.entry(window).or_default();
        if w.live.contains_key(&key) {
            return;
        }
        w.acquired += 1;
        w.live.insert(key, w.acquired);
        let t = FastLatencyDrawable {
            window,
            id: w.acquired,
            key,
        };
        r.records.push(Rec::Acquire(t, w.live.len(), tick));
        ticket = Some(t);
    });
    ticket
}
pub fn fast_latency_released(window: WindowId, key: u64) {
    let tick = fast_latency_tick();
    record(|r| {
        let w = r.windows.entry(window).or_default();
        if let Some(id) = w.live.remove(&key) {
            r.records.push(Rec::Release(
                FastLatencyDrawable { window, id, key },
                "unused",
                tick,
            ));
        }
    })
}
pub fn fast_latency_displayed(ticket: FastLatencyDrawable, presented: f64, host: f64) {
    let tick = fast_latency_tick();
    record(|r| {
        let w = r.windows.entry(ticket.window).or_default();
        if w.live.get(&ticket.key) == Some(&ticket.id) {
            w.live.remove(&ticket.key);
        }
        r.records.push(Rec::Present(ticket, tick, presented, host));
    })
}
pub fn fast_latency_command(frame: FastLatencyFrame, key: u64) -> Option<FastLatencyCommit> {
    let mut token = None;
    record(|r| {
        let w = r.windows.entry(frame.window).or_default();
        if let Some(&drawable) = w.live.get(&key) {
            token = Some(FastLatencyCommit { frame, drawable });
        }
    });
    token
}
pub fn fast_latency_committed(token: Option<FastLatencyCommit>) {
    if let Some(t) = token {
        let tick = fast_latency_tick();
        record(|r| r.records.push(Rec::Commit(t, "committed", tick, 0., 0.)))
    }
}
pub fn fast_latency_completed(token: FastLatencyCommit, gpu_end: f64) {
    fast_latency_gpu_interval(token, 0., gpu_end);
}

pub fn fast_latency_gpu_interval(token: FastLatencyCommit, gpu_start: f64, gpu_end: f64) {
    let tick = fast_latency_tick();
    record(|r| {
        r.records.push(Rec::Commit(
            token,
            "gpu_completed",
            tick,
            gpu_start,
            gpu_end,
        ))
    })
}
pub fn fast_latency_config(window: WindowId, kind: &'static str, values: [f64; 4]) {
    let tick = fast_latency_tick();
    record(|r| r.records.push(Rec::Config(window, kind, values, tick)))
}
fn flush() {
    let Ok(mut records) = RECORDS.lock() else {
        return;
    };
    let Some(r) = records.as_mut() else { return };
    let (numer, denom) = timebase();
    let mut out = Vec::new();
    let header = serde_json::json!({"event":"clock","numer":numer,"denom":denom,"mach_absolute_time":cfg!(target_os="macos")});
    writeln!(&mut out, "RUSTUI_PX4_LOG {header}").ok();
    for rec in &r.records {
        let row = match rec {
            Rec::Point(kind, w, draw, cb, tick) => {
                serde_json::json!({"event":kind,"window":format!("{w:?}"),"draw":draw,"callback":cb,"mach":tick})
            }
            Rec::Input(w, s, first, v, t) => {
                serde_json::json!({"event":"handled","window":format!("{w:?}"),"scene":s,"first":first,"version":v,"mach":t})
            }
            Rec::Paint(w, s, first, v, draw, t) => {
                serde_json::json!({"event":"painted","window":format!("{w:?}"),"scene":s,"first":first,"version":v,"draw":draw,"mach":t})
            }
            Rec::Attempt(f, cb, t) => {
                serde_json::json!({"event":"attempt","window":format!("{:?}",f.window),"draw":f.draw,"attempt":f.attempt,"callback":cb,"mach":t})
            }
            Rec::Acquire(t, n, tick) => {
                serde_json::json!({"event":"acquired","window":format!("{:?}",t.window),"drawable":t.id,"key":t.key,"in_flight":n,"mach":tick})
            }
            Rec::Release(t, why, tick) => {
                serde_json::json!({"event":"released","window":format!("{:?}",t.window),"drawable":t.id,"reason":why,"mach":tick})
            }
            Rec::Present(t, tick, p, h) => {
                serde_json::json!({"event":"displayed","window":format!("{:?}",t.window),"drawable":t.id,"callback_mach":tick,"presented_host":p,"callback_host":h})
            }
            Rec::Commit(t, kind, tick, start, end) => {
                serde_json::json!({"event":kind,"window":format!("{:?}",t.frame.window),"draw":t.frame.draw,"attempt":t.frame.attempt,"drawable":t.drawable,"mach":tick,"gpu_start_host":start,"gpu_end_host":end})
            }
            Rec::Config(w, kind, values, tick) => {
                serde_json::json!({"event":"config","window":format!("{w:?}"),"kind":kind,"values":values,"mach":tick})
            }
        };
        writeln!(&mut out, "RUSTUI_PX4_LOG {row}").ok();
    }
    flush_phases(&mut out);
    r.records.clear();
    let _ = std::io::stderr().lock().write_all(&out);
}

fn flush_phases(out: &mut Vec<u8>) {
    let Ok(phases) = PHASES.lock() else { return };
    if !phases_enabled() {
        return;
    }
    for span in phases.spans[..phases.len].iter().flatten() {
        let window = span.window.map(|id| format!("{id:?}"));
        let row = serde_json::json!({
            "event":"phase_span", "window":window, "frame":span.frame,
            "attempt":span.attempt, "kind":span.kind, "start":span.start, "end":span.end
        });
        writeln!(out, "RUSTUI_PX4_LOG {row}").ok();
    }
    let dropped = phases.dropped;
    writeln!(
        out,
        "RUSTUI_PX4_LOG {{\"event\":\"phase_dropped\",\"count\":{dropped}}}"
    )
    .ok();
}
