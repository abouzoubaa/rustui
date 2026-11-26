// SPDX-License-Identifier: Apache-2.0
//! RustUI damage records: format first, then issue one complete stderr write.
use std::{
    cell::RefCell,
    fmt,
    io::{self, Write as _},
};
thread_local! {
    static BUFFER: RefCell<String> = const { RefCell::new(String::new()) };
}
fn with_line<R>(
    args: fmt::Arguments<'_>,
    emit: impl FnOnce(&[u8]) -> io::Result<R>,
) -> io::Result<R> {
    BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        buffer.clear();
        fmt::write(&mut *buffer, args).map_err(io::Error::other)?;
        buffer.push('\n');
        emit(buffer.as_bytes())
    })
}
pub(crate) fn line(args: fmt::Arguments<'_>) {
    with_line(args, |bytes| io::stderr().lock().write_all(bytes))
        .expect("writing RustUI frame log");
}
#[cfg(any(test, feature = "test-support"))]
pub(crate) fn test_records() {
    #[derive(Default)]
    struct Sink {
        calls: usize,
        bytes: Vec<u8>,
    }
    impl io::Write for Sink {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.calls += 1;
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut sink = Sink::default();
    for ms in [1.25, 2.5] {
        with_line(
            format_args!("RUSTUI_FRAME_LOG {{\"event\":\"gpu\",\"gpu_ms\":{ms:.6}}}"),
            |bytes| sink.write_all(bytes),
        )
        .unwrap();
    }
    let mut streamed = Sink::default();
    for ms in [1.25, 2.5] {
        writeln!(
            &mut streamed,
            "RUSTUI_FRAME_LOG {{\"event\":\"gpu\",\"gpu_ms\":{ms:.6}}}"
        )
        .unwrap();
    }
    assert_eq!(
        sink.bytes, streamed.bytes,
        "preserve the original record bytes"
    );
    assert!(
        streamed.calls > sink.calls,
        "avoid the streaming formatter's fragmented writes"
    );
    assert_eq!(sink.calls, 2, "one complete write per record");
    assert_eq!(sink.bytes, b"RUSTUI_FRAME_LOG {\"event\":\"gpu\",\"gpu_ms\":1.250000}\nRUSTUI_FRAME_LOG {\"event\":\"gpu\",\"gpu_ms\":2.500000}\n");
}
#[cfg(test)]
#[test]
fn records_keep_their_format_and_use_one_write_each() {
    test_records();
}
