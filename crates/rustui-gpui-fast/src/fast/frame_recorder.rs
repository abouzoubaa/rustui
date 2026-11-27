// Added by RustUI for PX-1.
//! Queue raw timestamps; formatting and output happen outside measured phases.
use crate::WindowId;
use std::{
    collections::HashMap,
    collections::VecDeque,
    fmt::Write as _,
    io::Write as _,
    sync::{LazyLock, Mutex, OnceLock},
    time::{Instant, SystemTime},
};

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Off,
    Frame,
    Exit,
}
fn mode() -> Mode {
    static MODE: OnceLock<Mode> = OnceLock::new();
    *MODE.get_or_init(|| match std::env::var("RUSTUI_FRAME_LOG").as_deref() {
        Ok("1") => Mode::Frame,
        Ok("buffered") => {
            // C exit handlers also run for GPUI's process::exit quit path.
            unsafe extern "C" {
                fn atexit(callback: extern "C" fn()) -> std::ffi::c_int;
            }
            extern "C" fn at_exit() {
                flush();
            }
            assert_eq!(
                unsafe { atexit(at_exit) },
                0,
                "cannot register buffered frame log"
            );
            Mode::Exit
        }
        _ => Mode::Off,
    })
}
pub(crate) fn start() -> Option<Instant> {
    (mode() != Mode::Off).then(Instant::now)
}
pub(crate) fn phase_start() -> Option<Instant> {
    static PROFILE: OnceLock<bool> = OnceLock::new();
    if *PROFILE.get_or_init(|| std::env::var("RUSTUI_FRAME_PROFILE").as_deref() == Ok("1")) {
        start()
    } else {
        None
    }
}
struct Record {
    window: WindowId,
    event: &'static str,
    time_ms: f64,
    cpu_ms: f64,
    gap_ms: Option<f64>,
    epoch_ms: f64,
}
impl Record {
    fn append(&self, output: &mut String) {
        // Keep the original field order, precision, null representation and newline.
        let _ = writeln!(
            output,
            "RUSTUI_FRAME_LOG {{\"event\":\"{}\",\"window\":\"{:?}\",\"time_ms\":{:.6},\"cpu_ms\":{:.6},\"gap_ms\":{},\"epoch_ms\":{:.6}}}",
            self.event,
            self.window,
            self.time_ms,
            self.cpu_ms,
            self.gap_ms
                .map_or_else(|| "null".into(), |gap| format!("{gap:.6}")),
            self.epoch_ms
        );
    }
}
// A 60-second native hover capture exceeded 196,000 phase records.
const LIMIT: usize = 524_288;
struct Queue {
    records: VecDeque<Record>,
    lost: usize,
}
impl Queue {
    fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}
static RECORDS: LazyLock<Mutex<Queue>> = LazyLock::new(|| {
    Mutex::new(Queue {
        records: VecDeque::with_capacity(LIMIT),
        lost: 0,
    })
});
pub(crate) fn record(window: WindowId, event: &'static str, start: Option<Instant>) {
    let Some(start) = start else { return };
    let end = Instant::now();
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    let origin = *ORIGIN.get_or_init(|| start);
    thread_local! { static LAST: std::cell::RefCell<HashMap<WindowId, Instant>> = std::cell::RefCell::new(HashMap::new()); }
    let gap_ms = if event == "present" {
        LAST.with(|last| {
            last.borrow_mut()
                .insert(window, end)
                .map(|old| end.duration_since(old).as_secs_f64() * 1000.)
        })
    } else {
        None
    };
    let record = Record {
        window,
        event,
        time_ms: end.duration_since(origin).as_secs_f64() * 1000.,
        cpu_ms: end.duration_since(start).as_secs_f64() * 1000.,
        gap_ms,
        epoch_ms: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
            * 1000.,
    };
    if let Ok(mut records) = RECORDS.lock() {
        if records.records.len() < LIMIT {
            records.records.push_back(record);
        } else {
            records.lost += 1;
        }
    }
}
pub(crate) fn after_present() {
    if mode() == Mode::Frame {
        flush();
    }
}
fn flush() {
    let Ok(mut records) = RECORDS.lock() else {
        return;
    };
    if records.is_empty() {
        return;
    }
    let mut output = String::with_capacity(records.records.len() * 180);
    for record in records.records.iter() {
        record.append(&mut output);
    }
    records.records.clear();
    if records.lost > 0 {
        let _ = writeln!(output, "RUSTUI_FRAME_LOG_LOST {}", records.lost);
        records.lost = 0;
    }
    let _ = std::io::stderr().lock().write_all(output.as_bytes());
}

pub(crate) fn close_phase() -> Option<Instant> {
    static ON: OnceLock<bool> = OnceLock::new();
    if *ON.get_or_init(|| std::env::var("RUSTUI_G_R3_CPU").as_deref() == Ok("1")) {
        phase_start()
    } else {
        None
    }
}
