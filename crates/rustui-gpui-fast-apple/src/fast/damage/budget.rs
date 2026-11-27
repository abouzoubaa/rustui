// SPDX-License-Identifier: Apache-2.0
//! BGRA8 product textures for one window, including all its composition surfaces.
//! A retired texture still contributes to the working set until its last command completes.
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

pub(super) const CEILING: u64 = 512 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reason {
    BudgetExceeded,
    AllocFailed,
}
impl Reason {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::BudgetExceeded => "budget_exceeded",
            Self::AllocFailed => "alloc_failed",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Failure {
    pub(crate) reason: Reason,
    pub(crate) requested: u64,
    pub(crate) limit: u64,
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.limit == u64::MAX {
            return write!(
                f,
                "{} requested_bytes={} limit_bytes=unbounded (verify-only resource)",
                self.reason.label(),
                self.requested
            );
        }
        write!(
            f,
            "{} requested_bytes={} limit_bytes={}",
            self.reason.label(),
            self.requested,
            self.limit
        )
    }
}
impl std::error::Error for Failure {}

pub(super) fn bytes(width: usize, height: usize) -> u64 {
    (width as u64)
        .saturating_mul(height as u64)
        .saturating_mul(4)
}
pub(super) fn limit(working_set: u64) -> u64 {
    working_set
        .saturating_add(working_set.div_ceil(10))
        .min(CEILING)
}

#[derive(Default)]
pub(crate) struct WindowBudget {
    // Required BGRA storage and Metal-charged (padded) storage.
    allocated: Mutex<(u64, u64)>,
    warned: AtomicBool,
}
impl WindowBudget {
    pub(super) fn reserve(
        self: &Arc<Self>,
        required: u64,
        bytes: u64,
    ) -> Result<Reservation, Failure> {
        let mut live = self.allocated.lock().unwrap_or_else(|e| e.into_inner());
        let needed = live.1.saturating_add(bytes);
        let limit = limit(live.0.saturating_add(required));
        if needed > limit {
            super::fs_trace::denied(live.1, bytes);
            return Err(Failure {
                reason: Reason::BudgetExceeded,
                requested: needed,
                limit,
            });
        }
        live.0 = live.0.saturating_add(required);
        live.1 = needed;
        Ok(Reservation(Arc::new(Charge {
            budget: self.clone(),
            required,
            bytes,
            limit,
        })))
    }
    pub(super) fn warn(&self, failure: Failure) -> bool {
        if self.warned.swap(true, Ordering::Relaxed) {
            return false;
        }
        super::log::line(format_args!("RUSTUI_DAMAGE_WARNING {failure}"));
        true
    }
}
struct Charge {
    budget: Arc<WindowBudget>,
    required: u64,
    bytes: u64,
    limit: u64,
}
impl Drop for Charge {
    fn drop(&mut self) {
        let mut live = self
            .budget
            .allocated
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        live.0 -= self.required;
        live.1 -= self.bytes;
    }
}
#[derive(Clone)]
pub(crate) struct Reservation(Arc<Charge>);
impl Reservation {
    // Confirm the real private allocation before it is published to a surface.
    pub(super) fn confirm(&mut self, actual: u64) -> Result<(), Failure> {
        let charge = Arc::get_mut(&mut self.0).expect("unpublished texture charge");
        let mut live = charge
            .budget
            .allocated
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let total = (live.1 - charge.bytes).saturating_add(actual);
        let limit = limit(live.0);
        if total > limit {
            super::fs_trace::denied(live.1 - charge.bytes, actual);
            return Err(Failure {
                reason: Reason::BudgetExceeded,
                requested: total,
                limit,
            });
        }
        live.1 = total;
        charge.bytes = actual;
        charge.limit = limit;
        Ok(())
    }
    pub(super) fn failed(&self) -> Failure {
        Failure {
            reason: Reason::AllocFailed,
            requested: self.0.bytes,
            limit: self.0.limit,
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
pub(super) fn test_budget() {
    assert_eq!(limit(29_970_432), 32_967_476);
    assert_eq!(limit(58_982_400), 64_880_640);
    assert_eq!(limit(81_432_576), 89_575_834);
    let window = Arc::new(WindowBudget::default());
    assert!(window.reserve(100, 110).is_ok());
    assert!(window.reserve(100, 111).is_err());
    let base = window.reserve(CEILING - 128, CEILING - 128).unwrap();
    let gpu = base.clone();
    drop(base);
    let overlay = window.reserve(128, 128).unwrap();
    let denied = window.reserve(1, 1).err().unwrap();
    assert_eq!(denied.reason, Reason::BudgetExceeded);
    assert_eq!((denied.requested, denied.limit), (CEILING + 1, CEILING));
    assert!(window.warn(denied));
    assert!(!window.warn(denied));
    let independent = Arc::new(WindowBudget::default());
    assert!(independent.reserve(CEILING, CEILING).is_ok());
    drop((gpu, overlay));
    assert!(window.reserve(CEILING, CEILING).is_ok());
}
