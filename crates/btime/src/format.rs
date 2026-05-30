use crate::timing::{
    avg_mem_kb, format_elapsed_hms, format_elapsed_seconds, format_time_pair, maxrss_kb,
    memory_field_kb, page_size, msec_to_ticks, TimingResult,
};
use std::io::{self, Write};

pub const DEFAULT_FORMAT: &str = concat!(
    "%Uuser %Ssystem %Eelapsed %PCPU (%Xavgtext+%Davgdata %Mmaxresident)k\n",
    "%Iinputs+%Ooutputs (%Fmajor+%Rminor)pagefaults %Wswaps"
);

pub const POSIX_FORMAT: &str = "real %e\nuser %U\nsys %S";

pub const VERBOSE_FORMAT: &str = concat!(
    "\tCommand being timed: \"%C\"\n",
    "\tUser time (seconds): %U\n",
    "\tSystem time (seconds): %S\n",
    "\tPercent of CPU this job got: %P\n",
    "\tElapsed (wall clock) time (h:mm:ss or m:ss): %E\n",
    "\tAverage shared text size (kbytes): %X\n",
    "\tAverage unshared data size (kbytes): %D\n",
    "\tAverage stack size (kbytes): %p\n",
    "\tAverage total size (kbytes): %K\n",
    "\tMaximum resident set size (kbytes): %M\n",
    "\tAverage resident set size (kbytes): %t\n",
    "\tMajor (requiring I/O) page faults: %F\n",
    "\tMinor (reclaiming a frame) page faults: %R\n",
    "\tVoluntary context switches: %w\n",
    "\tInvoluntary context switches: %c\n",
    "\tSwaps: %W\n",
    "\tFile system inputs: %I\n",
    "\tFile system outputs: %O\n",
    "\tSocket messages sent: %s\n",
    "\tSocket messages received: %r\n",
    "\tSignals delivered: %k\n",
    "\tPage size (bytes): %Z\n",
    "\tExit status: %x"
);

pub fn print_abnormal_termination(out: &mut dyn Write, result: &TimingResult) -> io::Result<()> {
    #[cfg(unix)]
    {
        use crate::timing::{wait_status_exit_status, wait_status_exited, wait_status_signaled,
                            wait_status_stopped, wait_status_term_signal};
        if wait_status_stopped(result.wait_status) {
            writeln!(
                out,
                "Command stopped by signal {}",
                wait_status_term_signal(result.wait_status)
            )?;
        } else if wait_status_signaled(result.wait_status) {
            writeln!(
                out,
                "Command terminated by signal {}",
                wait_status_term_signal(result.wait_status)
            )?;
        } else if wait_status_exited(result.wait_status) {
            let code = wait_status_exit_status(result.wait_status);
            if code != 0 {
                writeln!(out, "Command exited with non-zero status {code}")?;
            }
        }
    }
    Ok(())
}

pub fn summarize(out: &mut dyn Write, fmt: &str, result: &TimingResult) -> io::Result<()> {
    let elapsed = result.elapsed();
    let cpu_ms = result.ru.utime_sec * 1000
        + result.ru.utime_usec / 1000
        + result.ru.stime_sec * 1000
        + result.ru.stime_usec / 1000;

    let mut chars = fmt.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '%' => match chars.next() {
                Some('%') => out.write_all(b"%")?,
                Some('C') => write_command(out, &result.command)?,
                Some('D') => {
                    let ticks = msec_to_ticks(cpu_ms);
                    let val = if ticks == 0 {
                        0
                    } else {
                        (memory_field_kb(result.ru.idrss) + memory_field_kb(result.ru.isrss))
                            / ticks as u64
                    };
                    write!(out, "{val}")?;
                }
                Some('E') => write!(out, "{}", format_elapsed_hms(elapsed))?,
                Some('F') => write!(out, "{}", result.ru.majflt)?,
                Some('I') => write!(out, "{}", result.ru.inblock)?,
                Some('K') => {
                    let ticks = msec_to_ticks(cpu_ms);
                    let val = if ticks == 0 {
                        0
                    } else {
                        (memory_field_kb(result.ru.idrss)
                            + memory_field_kb(result.ru.isrss)
                            + memory_field_kb(result.ru.ixrss))
                            / ticks as u64
                    };
                    write!(out, "{val}")?;
                }
                Some('M') => write!(out, "{}", maxrss_kb(result.ru.maxrss))?,
                Some('O') => write!(out, "{}", result.ru.oublock)?,
                Some('P') => match result.cpu_percent_gnu() {
                    Some(pct) => write!(out, "{pct}%")?,
                    None => out.write_all(b"?%")?,
                },
                Some('R') => write!(out, "{}", result.ru.minflt)?,
                Some('S') => write!(
                    out,
                    "{}",
                    format_time_pair(result.ru.stime_sec, result.ru.stime_usec)
                )?,
                Some('T') => match chars.next() {
                    Some('t') => write_termination_type(out, result)?,
                    Some('n') => write_signal_number(out, result)?,
                    Some('s') => write_signal_name(out, result)?,
                    Some('x') => write_exit_if_normal(out, result)?,
                    Some('o') => write_ok_if_zero(out, result)?,
                    Some('\0') | None => out.write_all(b"T=missing letter")?,
                    Some(other) => write!(out, "T?={other}")?,
                },
                Some('U') => write!(
                    out,
                    "{}",
                    format_time_pair(result.ru.utime_sec, result.ru.utime_usec)
                )?,
                Some('W') => match chars.peek().copied() {
                    // bfind-compatible BrightDate wall-clock timestamps (%.9f)
                    Some('t') => {
                        chars.next();
                        write!(out, "{:.9}", result.end_bd)?;
                    }
                    Some('s') => {
                        chars.next();
                        write!(out, "{:.9}", result.start_bd)?;
                    }
                    _ => write!(out, "{}", result.ru.nswap)?,
                },
                Some('X') => {
                    write!(out, "{}", avg_mem_kb(&result.ru, |ru| ru.ixrss, cpu_ms))?;
                }
                Some('Z') => write!(out, "{}", page_size())?,
                Some('c') => write!(out, "{}", result.ru.nivcsw)?,
                Some('e') => write!(out, "{}", format_elapsed_seconds(elapsed))?,
                Some('k') => write!(out, "{}", result.ru.nsignals)?,
                Some('p') => {
                    write!(out, "{}", avg_mem_kb(&result.ru, |ru| ru.isrss, cpu_ms))?;
                }
                Some('r') => write!(out, "{}", result.ru.msgrcv)?,
                Some('s') => write!(out, "{}", result.ru.msgsnd)?,
                Some('t') => {
                    write!(out, "{}", avg_mem_kb(&result.ru, |ru| ru.idrss, cpu_ms))?;
                }
                Some('w') => write!(out, "{}", result.ru.nvcsw)?,
                Some('x') => write!(out, "{}", exit_status_for_x(result))?,
                // BrightDate extensions (not in GNU time)
                Some('B') => write!(out, "{:.9}", result.elapsed_days())?,
                Some('b') => write!(out, "{:.6}", result.elapsed_days() * 1_000.0)?,
                Some('N') => write!(out, "{:.9}", result.start_bd)?,
                Some('n') => write!(out, "{:.9}", result.end_bd)?,
                Some('\0') | None => out.write_all(b"?")?,
                Some(other) => write!(out, "?{other}")?,
            },
            '\\' => match chars.next() {
                Some('t') => out.write_all(b"\t")?,
                Some('n') => out.write_all(b"\n")?,
                Some('\\') => out.write_all(b"\\")?,
                Some('\0') | None => out.write_all(b"?\\")?,
                Some(other) => write!(out, "?\\{other}")?,
            },
            other => write!(out, "{other}")?,
        }
    }
    writeln!(out)
}

fn write_command(out: &mut dyn Write, command: &[String]) -> io::Result<()> {
    for (i, part) in command.iter().enumerate() {
        if i > 0 {
            out.write_all(b" ")?;
        }
        out.write_all(part.as_bytes())?;
    }
    Ok(())
}

#[cfg(unix)]
fn write_termination_type(out: &mut dyn Write, result: &TimingResult) -> io::Result<()> {
    use crate::timing::{wait_status_signaled, wait_status_stopped};
    let text = if wait_status_stopped(result.wait_status) {
        "stopped"
    } else if wait_status_signaled(result.wait_status) {
        "signalled"
    } else {
        "normal"
    };
    out.write_all(text.as_bytes())
}

#[cfg(not(unix))]
fn write_termination_type(out: &mut dyn Write, _result: &TimingResult) -> io::Result<()> {
    out.write_all(b"normal")
}

#[cfg(unix)]
fn write_signal_number(out: &mut dyn Write, result: &TimingResult) -> io::Result<()> {
    use crate::timing::{wait_status_signaled, wait_status_term_signal};
    if wait_status_signaled(result.wait_status) {
        write!(out, "{}", wait_status_term_signal(result.wait_status))
    } else {
        Ok(())
    }
}

#[cfg(not(unix))]
fn write_signal_number(out: &mut dyn Write, _result: &TimingResult) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn write_signal_name(out: &mut dyn Write, result: &TimingResult) -> io::Result<()> {
    use crate::timing::{wait_status_signaled, wait_status_term_signal};
    if wait_status_signaled(result.wait_status) {
        let sig = wait_status_term_signal(result.wait_status);
        if let Some(name) = signal_name(sig) {
            out.write_all(name.as_bytes())
        } else {
            write!(out, "({sig})")
        }
    } else {
        Ok(())
    }
}

#[cfg(not(unix))]
fn write_signal_name(out: &mut dyn Write, _result: &TimingResult) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn write_exit_if_normal(out: &mut dyn Write, result: &TimingResult) -> io::Result<()> {
    use crate::timing::{wait_status_exit_status, wait_status_exited};
    if wait_status_exited(result.wait_status) {
        write!(out, "{}", wait_status_exit_status(result.wait_status))
    } else {
        Ok(())
    }
}

#[cfg(not(unix))]
fn write_exit_if_normal(out: &mut dyn Write, result: &TimingResult) -> io::Result<()> {
    write!(out, "{}", result.exit_code())
}

#[cfg(unix)]
fn write_ok_if_zero(out: &mut dyn Write, result: &TimingResult) -> io::Result<()> {
    use crate::timing::{wait_status_exit_status, wait_status_exited};
    if wait_status_exited(result.wait_status) && wait_status_exit_status(result.wait_status) == 0 {
        out.write_all(b"ok")
    } else {
        Ok(())
    }
}

#[cfg(not(unix))]
fn write_ok_if_zero(out: &mut dyn Write, result: &TimingResult) -> io::Result<()> {
    if result.exit_code() == 0 {
        out.write_all(b"ok")
    } else {
        Ok(())
    }
}

#[cfg(unix)]
fn exit_status_for_x(result: &TimingResult) -> i32 {
    use crate::timing::wait_status_exit_status;
    wait_status_exit_status(result.wait_status)
}

#[cfg(not(unix))]
fn exit_status_for_x(result: &TimingResult) -> i32 {
    result.exit_code()
}

#[cfg(unix)]
fn signal_name(sig: i32) -> Option<&'static str> {
    let name = unsafe {
        let ptr = libc::strsignal(sig);
        if ptr.is_null() {
            return None;
        }
        std::ffi::CStr::from_ptr(ptr).to_str().ok()?
    };
    Some(match name {
        "Terminated" => "TERM",
        "Interrupt" => "INT",
        "Quit" => "QUIT",
        "Kill" => "KILL",
        "Hangup" => "HUP",
        other => return Some(other),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timing::{ResourceUsage, Timespec};

    fn sample_result() -> TimingResult {
        TimingResult {
            command: vec!["btime".into(), "true".into()],
            wait_status: 0,
            ru: ResourceUsage {
                utime_sec: 0,
                utime_usec: 10_000,
                stime_sec: 0,
                stime_usec: 20_000,
                maxrss: 1024,
                minflt: 3,
                majflt: 1,
                inblock: 4,
                oublock: 5,
                ..Default::default()
            },
            start_time: Timespec {
                tv_sec: 100,
                tv_nsec: 0,
            },
            end_time: Timespec {
                tv_sec: 100,
                tv_nsec: 500_000_000,
            },
            start_bd: 9645.0,
            end_bd: 9645.00001,
        }
    }

    #[test]
    fn time_resource_specifiers() {
        let mut buf = Vec::new();
        summarize(&mut buf, "E=%E e=%e U=%U S=%S P=%P", &sample_result()).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("E=0:00.50"));
        assert!(text.contains("e=0.50"));
        assert!(text.contains("U=0.01"));
        assert!(text.contains("S=0.02"));
        assert!(text.contains("P=6%"));
    }

    #[test]
    fn command_and_exit_specifiers() {
        let mut buf = Vec::new();
        summarize(&mut buf, "cmd=%C x=%x", &sample_result()).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("cmd=btime true"));
        assert!(text.contains("x=0"));
    }

    #[test]
    fn io_and_memory_specifiers() {
        let mut buf = Vec::new();
        summarize(
            &mut buf,
            "M=%M F=%F R=%R I=%I O=%O Z=%Z",
            &sample_result(),
        )
        .unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("M=1024") || text.contains("M=1"));
        assert!(text.contains("F=1"));
        assert!(text.contains("R=3"));
        assert!(text.contains("I=4"));
        assert!(text.contains("O=5"));
        assert!(text.contains("Z="));
    }

    #[test]
    fn termination_specifiers_normal_exit() {
        let mut buf = Vec::new();
        summarize(&mut buf, "t=%Tt x=%Tx n=%Tn s=%Ts o=%To", &sample_result()).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("t=normal"));
        assert!(text.contains("x=0"));
        assert!(text.contains("n="));
        assert!(text.contains("s="));
        assert!(text.contains("o=ok"));
    }

    #[test]
    fn invalid_specifier_and_escape() {
        let mut buf = Vec::new();
        summarize(&mut buf, "bad=%z \\x", &sample_result()).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("bad=?z"));
        assert!(text.contains("?\\x"));
    }

    #[test]
    fn default_and_verbose_formats_smoke() {
        let result = sample_result();
        let mut default = Vec::new();
        summarize(&mut default, DEFAULT_FORMAT, &result).unwrap();
        let default_text = String::from_utf8(default).unwrap();
        assert!(default_text.contains("user"));
        assert!(default_text.contains("system"));
        assert!(default_text.contains("pagefaults"));

        let mut verbose = Vec::new();
        summarize(&mut verbose, VERBOSE_FORMAT, &result).unwrap();
        let verbose_text = String::from_utf8(verbose).unwrap();
        assert!(verbose_text.contains("Command being timed"));
        assert!(verbose_text.contains("Page size (bytes)"));
    }

    #[test]
    fn abnormal_termination_message() {
        #[cfg(unix)]
        let mut result = sample_result();
        #[cfg(unix)]
        {
            result.wait_status = 256; // exit code 1
        }
        #[cfg(not(unix))]
        let mut result = {
            let mut r = sample_result();
            r.wait_status = 1;
            r
        };

        let mut buf = Vec::new();
        print_abnormal_termination(&mut buf, &result).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("non-zero status 1"));
    }

    #[test]
    fn posix_format_fields() {
        let mut buf = Vec::new();
        summarize(&mut buf, POSIX_FORMAT, &sample_result()).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("real "));
        assert!(text.contains("user "));
        assert!(text.contains("sys "));
    }

    #[test]
    fn literal_percent_and_escape() {
        let mut buf = Vec::new();
        summarize(&mut buf, "100%% done\\t!", &sample_result()).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("100% done\t!"));
    }

    #[test]
    fn brightdate_extensions() {
        let mut buf = Vec::new();
        summarize(
            &mut buf,
            "bd=%B md=%b start=%N end=%n Wt=%Wt Ws=%Ws",
            &sample_result(),
        )
        .unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("bd=0.000005787"));
        assert!(text.contains("md=0.005787"));
        assert!(text.contains("start=9645.000000000"));
        assert!(text.contains("end=9645.000010000"));
        assert!(text.contains("Wt=9645.000010000"));
        assert!(text.contains("Ws=9645.000000000"));
    }

    #[test]
    fn gnu_w_swaps_without_brightdate_suffix() {
        let mut result = sample_result();
        result.ru.nswap = 7;
        let mut buf = Vec::new();
        summarize(&mut buf, "swaps=%W", &result).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("swaps=7"));
    }
}
