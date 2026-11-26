// SPDX-License-Identifier: Apache-2.0
//! Read the opt-in Lab interposer on the renderer thread in separate captures.
use std::{
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

type Read = unsafe extern "C" fn() -> u64;
pub(super) struct Sample(Option<(Read, u64, &'static str)>);
impl Sample {
    pub(super) fn start(label: &'static str) -> Self {
        static READ: OnceLock<Option<Read>> = OnceLock::new();
        let read = READ.get_or_init(|| {
            if std::env::var_os("RUSTUI_G_R2_ALLOCATIONS").as_deref() != Some("1".as_ref()) {
                return None;
            }
            // The opt-in capture driver injects this exact C ABI function.
            let symbol =
                unsafe { libc::dlsym(libc::RTLD_DEFAULT, c"rustui_gr2_main_allocations".as_ptr()) };
            assert!(!symbol.is_null(), "missing G-R2 allocation observer");
            Some(unsafe { std::mem::transmute::<*mut libc::c_void, Read>(symbol) })
        });
        Self(read.map(|f| (f, unsafe { f() }, label)))
    }
}
impl Drop for Sample {
    fn drop(&mut self) {
        if let Some((read, before, label)) = self.0 {
            let calls = unsafe { read() } - before;
            let epoch_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64()
                * 1000.;
            eprintln!("RUSTUI_G_R2_ALLOC {label} {calls} {epoch_ms:.6}");
        }
    }
}
