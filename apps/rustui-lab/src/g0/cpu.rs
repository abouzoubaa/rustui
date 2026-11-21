//! Process CPU snapshots bound the measured window, after warmup.
#[derive(Clone, Copy, Debug)]
pub(super) struct Snapshot {
    pub(super) user_s: f64,
    pub(super) sys_s: f64,
}
impl Snapshot {
    pub(super) fn take() -> std::io::Result<Self> {
        let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
        // SAFETY: getrusage initializes this correctly aligned rusage on success.
        if unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: the successful getrusage call initialized every field.
        let usage = unsafe { usage.assume_init() };
        let seconds = |v: libc::timeval| v.tv_sec as f64 + v.tv_usec as f64 / 1_000_000.;
        Ok(Self {
            user_s: seconds(usage.ru_utime),
            sys_s: seconds(usage.ru_stime),
        })
    }
    pub(super) fn elapsed(self, end: Self) -> (f64, f64) {
        (end.user_s - self.user_s, end.sys_s - self.sys_s)
    }
}
#[cfg(test)]
mod tests {
    use super::Snapshot;
    #[test]
    fn process_cpu_counts_both_user_and_system_without_sleep_wall_time() {
        let a = Snapshot::take().unwrap();
        let mut value = 1u64;
        for i in 1..100_000 {
            value = std::hint::black_box(value.wrapping_mul(31).wrapping_add(i));
        }
        std::hint::black_box(value);
        let b = Snapshot::take().unwrap();
        let (user, sys) = a.elapsed(b);
        assert!(user >= 0. && sys >= 0. && user + sys > 0.);
        let (user, sys) = Snapshot {
            user_s: 1.25,
            sys_s: 0.5,
        }
        .elapsed(Snapshot {
            user_s: 1.5,
            sys_s: 0.75,
        });
        assert_eq!((user, sys), (0.25, 0.25));
    }
}
