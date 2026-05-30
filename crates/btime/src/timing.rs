use brightdate::BrightDate;
use std::io;

#[derive(Debug, Clone, Copy, Default)]
pub struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

impl Timespec {
    pub fn sub(self, other: Self) -> Self {
        let mut sec = self.tv_sec - other.tv_sec;
        let mut nsec = self.tv_nsec - other.tv_nsec;
        if nsec < 0 {
            sec -= 1;
            nsec += 1_000_000_000;
        }
        Self {
            tv_sec: sec,
            tv_nsec: nsec,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    pub utime_sec: i64,
    pub utime_usec: i64,
    pub stime_sec: i64,
    pub stime_usec: i64,
    pub maxrss: i64,
    pub ixrss: i64,
    pub idrss: i64,
    pub isrss: i64,
    pub minflt: i64,
    pub majflt: i64,
    pub nswap: i64,
    pub inblock: i64,
    pub oublock: i64,
    pub msgsnd: i64,
    pub msgrcv: i64,
    pub nsignals: i64,
    pub nvcsw: i64,
    pub nivcsw: i64,
}

#[derive(Debug, Clone)]
pub struct TimingResult {
    pub command: Vec<String>,
    pub wait_status: i32,
    pub ru: ResourceUsage,
    pub start_time: Timespec,
    pub end_time: Timespec,
    pub start_bd: f64,
    pub end_bd: f64,
}

impl TimingResult {
    pub fn elapsed(&self) -> Timespec {
        self.end_time.sub(self.start_time)
    }

    pub fn elapsed_secs(&self) -> f64 {
        let e = self.elapsed();
        e.tv_sec as f64 + e.tv_nsec as f64 * 1e-9
    }

    pub fn elapsed_days(&self) -> f64 {
        self.elapsed_secs() / 86_400.0
    }

    pub fn user_secs(&self) -> f64 {
        self.ru.utime_sec as f64 + self.ru.utime_usec as f64 * 1e-6
    }

    pub fn sys_secs(&self) -> f64 {
        self.ru.stime_sec as f64 + self.ru.stime_usec as f64 * 1e-6
    }

    #[allow(dead_code)]
    pub fn cpu_secs(&self) -> f64 {
        self.user_secs() + self.sys_secs()
    }

    pub fn cpu_percent_gnu(&self) -> Option<u64> {
        let elapsed = self.elapsed();
        let wall_ms = elapsed.tv_sec * 1000 + elapsed.tv_nsec / 1_000_000;
        let cpu_ms = self.ru.utime_sec * 1000
            + self.ru.utime_usec / 1000
            + self.ru.stime_sec * 1000
            + self.ru.stime_usec / 1000;
        if wall_ms > 0 {
            Some((cpu_ms as u64) * 100 / wall_ms as u64)
        } else {
            let wall_us = elapsed.tv_nsec / 1000;
            let cpu_us = self.ru.utime_usec + self.ru.stime_usec;
            if wall_us > 0 {
                Some(cpu_us as u64 * 100 / wall_us as u64)
            } else {
                None
            }
        }
    }

    #[allow(dead_code)]
    pub fn exit_code(&self) -> i32 {
        wait_status_to_exit_code(self.wait_status)
    }
}

#[cfg(unix)]
fn realtime_now() -> Timespec {
    unsafe {
        let mut ts = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        libc::clock_gettime(libc::CLOCK_REALTIME, &mut ts);
        Timespec {
            tv_sec: ts.tv_sec,
            tv_nsec: ts.tv_nsec,
        }
    }
}

#[cfg(unix)]
fn rusage_from_libc(ru: libc::rusage) -> ResourceUsage {
    ResourceUsage {
        utime_sec: ru.ru_utime.tv_sec,
        utime_usec: ru.ru_utime.tv_usec as i64,
        stime_sec: ru.ru_stime.tv_sec,
        stime_usec: ru.ru_stime.tv_usec as i64,
        maxrss: ru.ru_maxrss,
        ixrss: ru.ru_ixrss,
        idrss: ru.ru_idrss,
        isrss: ru.ru_isrss,
        minflt: ru.ru_minflt,
        majflt: ru.ru_majflt,
        nswap: ru.ru_nswap,
        inblock: ru.ru_inblock,
        oublock: ru.ru_oublock,
        msgsnd: ru.ru_msgsnd,
        msgrcv: ru.ru_msgrcv,
        nsignals: ru.ru_nsignals,
        nvcsw: ru.ru_nvcsw,
        nivcsw: ru.ru_nivcsw,
    }
}

#[cfg(unix)]
pub fn run_command(cmd: &[&str]) -> io::Result<TimingResult> {
    if cmd.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "missing command"));
    }

    let start_time = realtime_now();
    let start_bd = BrightDate::now().value;

    let mut wait_status = 0;
    let ru = unsafe {
        let interrupt = libc::signal(libc::SIGINT, libc::SIG_IGN);
        let quit = libc::signal(libc::SIGQUIT, libc::SIG_IGN);

        let pid = libc::fork();
        if pid < 0 {
            libc::signal(libc::SIGINT, interrupt);
            libc::signal(libc::SIGQUIT, quit);
            return Err(io::Error::last_os_error());
        }

        if pid == 0 {
            let c_strings: Vec<std::ffi::CString> = cmd
                .iter()
                .map(|arg| std::ffi::CString::new(*arg))
                .collect::<Result<_, _>>()
                .unwrap_or_else(|_| libc::_exit(126));

            let mut argv: Vec<*const libc::c_char> =
                c_strings.iter().map(|s| s.as_ptr()).collect();
            argv.push(std::ptr::null());

            libc::execvp(c_strings[0].as_ptr(), argv.as_ptr());
            let code = if io::Error::last_os_error().kind() == io::ErrorKind::NotFound {
                127
            } else {
                126
            };
            libc::_exit(code);
        }

        if libc::waitpid(pid, &mut wait_status, 0) < 0 {
            let err = io::Error::last_os_error();
            libc::signal(libc::SIGINT, interrupt);
            libc::signal(libc::SIGQUIT, quit);
            return Err(err);
        }

        libc::signal(libc::SIGINT, interrupt);
        libc::signal(libc::SIGQUIT, quit);

        let mut ru: libc::rusage = std::mem::zeroed();
        if libc::getrusage(libc::RUSAGE_CHILDREN, &mut ru) != 0 {
            return Err(io::Error::last_os_error());
        }
        ru
    };

    let end_time = realtime_now();
    let end_bd = BrightDate::now().value;

    Ok(TimingResult {
        command: cmd.iter().map(|s| (*s).to_string()).collect(),
        wait_status,
        ru: rusage_from_libc(ru),
        start_time,
        end_time,
        start_bd,
        end_bd,
    })
}

#[cfg(not(unix))]
pub fn run_command(cmd: &[&str]) -> io::Result<TimingResult> {
    use std::process::Command;
    use std::time::Instant;

    if cmd.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "missing command"));
    }

    let start_instant = Instant::now();
    let start_bd = BrightDate::now().value;
    let start_time = Timespec::default();

    let status = Command::new(cmd[0])
        .args(&cmd[1..])
        .status()
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                io::Error::new(io::ErrorKind::NotFound, e)
            } else {
                e
            }
        })?;

    let elapsed = start_instant.elapsed();
    let end_bd = BrightDate::now().value;
    let end_time = Timespec {
        tv_sec: elapsed.as_secs() as i64,
        tv_nsec: elapsed.subsec_nanos() as i64,
    };

    Ok(TimingResult {
        command: cmd.iter().map(|s| (*s).to_string()).collect(),
        wait_status: status.code().unwrap_or(1),
        ru: ResourceUsage::default(),
        start_time,
        end_time,
        start_bd,
        end_bd,
    })
}

pub fn wait_status_to_exit_code(status: i32) -> i32 {
    #[cfg(unix)]
    unsafe {
        if libc::WIFSTOPPED(status) {
            return libc::WSTOPSIG(status) + 128;
        }
        if libc::WIFSIGNALED(status) {
            return libc::WTERMSIG(status) + 128;
        }
        if libc::WIFEXITED(status) {
            return libc::WEXITSTATUS(status);
        }
        return 1;
    }
    #[cfg(not(unix))]
    {
        status
    }
}

#[cfg(unix)]
pub fn wait_status_exited(status: i32) -> bool {
    unsafe { libc::WIFEXITED(status) }
}

#[cfg(unix)]
pub fn wait_status_exit_status(status: i32) -> i32 {
    unsafe { libc::WEXITSTATUS(status) }
}

#[cfg(unix)]
pub fn wait_status_signaled(status: i32) -> bool {
    unsafe { libc::WIFSIGNALED(status) }
}

#[cfg(unix)]
pub fn wait_status_stopped(status: i32) -> bool {
    unsafe { libc::WIFSTOPPED(status) }
}

#[cfg(unix)]
pub fn wait_status_term_signal(status: i32) -> i32 {
    unsafe {
        if libc::WIFSIGNALED(status) {
            libc::WTERMSIG(status)
        } else if libc::WIFSTOPPED(status) {
            libc::WSTOPSIG(status)
        } else {
            0
        }
    }
}

#[cfg(unix)]
pub fn ticks_per_sec() -> i64 {
    unsafe {
        let t = libc::sysconf(libc::_SC_CLK_TCK);
        if t > 0 {
            t
        } else {
            100
        }
    }
}

#[cfg(not(unix))]
pub fn ticks_per_sec() -> i64 {
    100
}

pub fn msec_per_tick() -> i64 {
    1000 / ticks_per_sec().max(1)
}

pub fn msec_to_ticks(ms: i64) -> i64 {
    if msec_per_tick() > 0 {
        ms / msec_per_tick()
    } else {
        ms
    }
}

#[cfg(unix)]
pub fn maxrss_kb(maxrss: i64) -> u64 {
    if cfg!(target_os = "macos") {
        (maxrss.max(0) as u64) / 1024
    } else {
        maxrss.max(0) as u64
    }
}

#[cfg(not(unix))]
pub fn maxrss_kb(maxrss: i64) -> u64 {
    maxrss.max(0) as u64
}

#[cfg(unix)]
pub fn memory_field_kb(value: i64) -> u64 {
    if cfg!(target_os = "macos") {
        (value.max(0) as u64) / 1024
    } else {
        value.max(0) as u64
    }
}

#[cfg(not(unix))]
pub fn memory_field_kb(value: i64) -> u64 {
    value.max(0) as u64
}

pub fn avg_mem_kb(ru: &ResourceUsage, field: fn(&ResourceUsage) -> i64, cpu_ms: i64) -> u64 {
    let ticks = msec_to_ticks(cpu_ms);
    if ticks == 0 {
        return 0;
    }
    memory_field_kb(field(ru)) / ticks as u64
}

pub fn page_size() -> i32 {
    #[cfg(unix)]
    unsafe {
        let ps = libc::sysconf(libc::_SC_PAGESIZE);
        if ps > 0 {
            ps as i32
        } else {
            4096
        }
    }
    #[cfg(not(unix))]
    {
        4096
    }
}

pub fn format_time_pair(sec: i64, usec: i64) -> String {
    let centis = usec / 10_000;
    format!("{sec}.{centis:02}")
}

pub fn format_elapsed_seconds(elapsed: Timespec) -> String {
    let centis = elapsed.tv_nsec / 10_000_000;
    format!("{}.{centis:02}", elapsed.tv_sec)
}

pub fn format_elapsed_hms(elapsed: Timespec) -> String {
    if elapsed.tv_sec >= 3600 {
        format!(
            "{}:{:02}:{:02}",
            elapsed.tv_sec / 3600,
            (elapsed.tv_sec % 3600) / 60,
            elapsed.tv_sec % 60
        )
    } else {
        format!(
            "{}:{:02}.{:02}",
            elapsed.tv_sec / 60,
            elapsed.tv_sec % 60,
            elapsed.tv_nsec / 10_000_000
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_time_pair_centiseconds() {
        assert_eq!(format_time_pair(1, 234_567), "1.23");
        assert_eq!(format_time_pair(0, 10_000), "0.01");
    }

    #[test]
    fn format_elapsed_seconds_centiseconds() {
        let elapsed = Timespec {
            tv_sec: 2,
            tv_nsec: 340_000_000,
        };
        assert_eq!(format_elapsed_seconds(elapsed), "2.34");
    }

    #[test]
    fn format_elapsed_hms_under_one_hour() {
        let elapsed = Timespec {
            tv_sec: 125,
            tv_nsec: 500_000_000,
        };
        assert_eq!(format_elapsed_hms(elapsed), "2:05.50");
    }

    #[test]
    fn format_elapsed_hms_over_one_hour() {
        let elapsed = Timespec {
            tv_sec: 3661,
            tv_nsec: 0,
        };
        assert_eq!(format_elapsed_hms(elapsed), "1:01:01");
    }

    #[test]
    fn cpu_percent_gnu_integer_math() {
        let result = TimingResult {
            command: vec!["true".into()],
            wait_status: 0,
            ru: ResourceUsage {
                utime_sec: 0,
                utime_usec: 500_000,
                stime_sec: 0,
                stime_usec: 500_000,
                ..Default::default()
            },
            start_time: Timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            end_time: Timespec {
                tv_sec: 0,
                tv_nsec: 1_000_000_000,
            },
            start_bd: 0.0,
            end_bd: 0.0,
        };
        assert_eq!(result.cpu_percent_gnu(), Some(100));
    }
}
