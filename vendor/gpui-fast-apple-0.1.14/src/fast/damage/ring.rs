// SPDX-License-Identifier: Apache-2.0
//! Bounded byte ring shared by the renderer and Metal completion callbacks.
//! Retain complete records in arrival order; a full capture is marked invalid.
use std::{
    collections::VecDeque,
    io::{self, Write},
    sync::{Mutex, OnceLock},
};

struct Ring {
    bytes: VecDeque<u8>,
    limit: usize,
    lost: usize,
}
impl Ring {
    fn new(limit: usize) -> Self {
        Self {
            bytes: VecDeque::with_capacity(limit),
            limit,
            lost: 0,
        }
    }
    fn push(&mut self, line: &[u8]) {
        if line.len() > self.limit - self.bytes.len() {
            self.lost += 1;
        } else {
            self.bytes.extend(line.iter().copied());
        }
    }
    fn flush(&mut self, output: &mut impl Write) -> io::Result<()> {
        let (a, b) = self.bytes.as_slices();
        output.write_all(a)?;
        output.write_all(b)?;
        writeln!(output, "RUSTUI_G_R2_LOG_LOST {}", self.lost)?;
        self.bytes.clear();
        self.lost = 0;
        Ok(())
    }
}
static RING: OnceLock<Mutex<Ring>> = OnceLock::new();
extern "C" fn at_exit() {
    if let Some(ring) = RING.get() {
        let _ = ring
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .flush(&mut io::stderr().lock());
    }
}
pub(super) fn buffered() -> bool {
    static ACTIVE: OnceLock<bool> = OnceLock::new();
    *ACTIVE.get_or_init(|| {
        if std::env::var_os("RUSTUI_FRAME_LOG").as_deref() != Some("buffered".as_ref()) {
            return false;
        }
        RING.get_or_init(|| Mutex::new(Ring::new(32 * 1024 * 1024)));
        assert_eq!(
            unsafe { libc::atexit(at_exit) },
            0,
            "cannot register capture flush"
        );
        true
    })
}
pub(super) fn push(line: &[u8]) {
    RING.get()
        .unwrap()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push(line);
}
#[cfg(any(test, feature = "test-support"))]
pub(super) fn proof() {
    let mut ring = Ring::new(4);
    ring.push(b"a\n");
    ring.push(b"b\n");
    ring.push(b"c\n");
    let mut bytes = Vec::new();
    ring.flush(&mut bytes).unwrap();
    assert_eq!(bytes, b"a\nb\nRUSTUI_G_R2_LOG_LOST 1\n");
}
