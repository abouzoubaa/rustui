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
/// Host CPU tick sample at the same boundary as the measured process.
#[cfg(target_os = "macos")]
pub(super) struct Background {
    ticks: [u32; 4],
    wall: std::time::Instant,
    services: Services,
}

#[cfg(target_os = "macos")]
impl Background {
    pub(super) fn start() -> std::io::Result<Option<Self>> {
        if std::env::var_os("RUSTUI_G_R2_LOAD").as_deref() != Some("1".as_ref()) {
            return Ok(None);
        }
        let services = Services::read()?;
        Ok(Some(Self::take(services)?))
    }

    fn take(services: Services) -> std::io::Result<Self> {
        unsafe extern "C" {
            fn mach_port_deallocate(task: libc::mach_port_t, name: libc::mach_port_t) -> i32;
        }
        let mut ticks = [0u32; 4];
        let mut count = libc::HOST_CPU_LOAD_INFO_COUNT;
        // SAFETY: the output has space for all four Mach CPU counters. Balance
        // the send right acquired by mach_host_self after the synchronous call.
        // The shared bare reference compiles this module using libc alone.
        #[allow(
            deprecated,
            reason = "use the existing libc bindings for the standalone Lab and reference"
        )]
        let status = unsafe {
            let host = libc::mach_host_self();
            let status = libc::host_statistics(
                host,
                libc::HOST_CPU_LOAD_INFO,
                ticks.as_mut_ptr().cast(),
                &mut count,
            );
            mach_port_deallocate(libc::mach_task_self(), host);
            status
        };
        if status != libc::KERN_SUCCESS || count != libc::HOST_CPU_LOAD_INFO_COUNT {
            return Err(std::io::Error::other(format!(
                "host CPU sample failed: {status}, count {count}"
            )));
        }
        Ok(Self {
            ticks,
            wall: std::time::Instant::now(),
            services,
        })
    }

    pub(super) fn finish(self, process_cpu_s: f64) -> std::io::Result<()> {
        let end = Self::take(self.services)?;
        let elapsed = (end.wall - self.wall).as_secs_f64();
        let ticks = std::array::from_fn::<_, 4, _>(|i| end.ticks[i].wrapping_sub(self.ticks[i]));
        let total = ticks.iter().map(|&v| u64::from(v)).sum::<u64>();
        if total == 0 || elapsed <= 0.0 {
            return Err(std::io::Error::other("empty host CPU interval"));
        }
        // SAFETY: sysconf takes an integer selector and returns a scalar.
        let cores = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) };
        if cores <= 0 {
            return Err(std::io::Error::last_os_error());
        }
        let idle = u64::from(ticks[libc::CPU_STATE_IDLE as usize]);
        let host = 100.0 * cores as f64 * (total - idle) as f64 / total as f64;
        let app = 100.0 * process_cpu_s / elapsed;
        let current = Services::read()?;
        let window_server = 100.0 * (current.window_server - self.services.window_server) / elapsed;
        let kernel_task = 100.0 * (current.kernel_task - self.services.kernel_task) / elapsed;
        if window_server < 0.0 || kernel_task < 0.0 {
            return Err(std::io::Error::other(
                "system process CPU counter moved backward",
            ));
        }
        let background = (host - app - window_server - kernel_task).max(0.0);
        eprintln!(
            r#"RUSTUI_G_R2_LOAD {{"one_core_percent":{background:.6},"host_one_core_percent":{host:.6},"app_one_core_percent":{app:.6},"window_server_one_core_percent":{window_server:.6},"kernel_task_one_core_percent":{kernel_task:.6},"elapsed_s":{elapsed:.9}}}"#
        );
        Ok(())
    }
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
struct Services {
    window_server: f64,
    kernel_task: f64,
}

#[cfg(target_os = "macos")]
impl Services {
    fn read() -> std::io::Result<Self> {
        let pid = std::env::var("RUSTUI_G_R2_WINDOW_SERVER_PID").map_err(std::io::Error::other)?;
        let output = std::process::Command::new("/usr/bin/top")
            .args([
                "-l",
                "1",
                "-pid",
                "0",
                "-pid",
                &pid,
                "-stats",
                "pid,time,command",
            ])
            .env("LC_ALL", "C")
            .output()?;
        if !output.status.success() {
            return Err(std::io::Error::other(
                "top failed to sample system processes",
            ));
        }
        let output = String::from_utf8(output.stdout).map_err(std::io::Error::other)?;
        let mut kernel_task = None;
        let mut window_server = None;
        for line in output.lines() {
            let mut fields = line.split_whitespace();
            let (Some(row_pid), Some(cpu), Some(command)) =
                (fields.next(), fields.next(), fields.next())
            else {
                continue;
            };
            if row_pid == "0" && command == "kernel_task" {
                kernel_task = Some(Self::cpu_seconds(cpu)?);
            } else if row_pid == pid && command == "WindowServer" {
                window_server = Some(Self::cpu_seconds(cpu)?);
            }
        }
        let missing = || std::io::Error::other("top omitted a required system process");
        Ok(Self {
            window_server: window_server.ok_or_else(missing)?,
            kernel_task: kernel_task.ok_or_else(missing)?,
        })
    }

    fn cpu_seconds(time: &str) -> std::io::Result<f64> {
        time.split(':').try_fold(0.0, |total, part| {
            part.parse::<f64>()
                .map(|part| total * 60. + part)
                .map_err(std::io::Error::other)
        })
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
