use brightdate::BrightDate;
use clap::{Arg, Command};
use std::process;
use std::time::Instant;

#[cfg(unix)]
fn children_cpu_secs() -> Option<(f64, f64)> {
    // Returns (user_secs, sys_secs) accumulated by terminated children
    // since this process started. Safe to call before/after spawning a
    // child; the difference attributes time to that child.
    unsafe {
        let mut ru: libc::rusage = std::mem::zeroed();
        if libc::getrusage(libc::RUSAGE_CHILDREN, &mut ru) != 0 {
            return None;
        }
        let user = ru.ru_utime.tv_sec as f64 + ru.ru_utime.tv_usec as f64 * 1e-6;
        let sys = ru.ru_stime.tv_sec as f64 + ru.ru_stime.tv_usec as f64 * 1e-6;
        Some((user, sys))
    }
}

#[cfg(not(unix))]
fn children_cpu_secs() -> Option<(f64, f64)> {
    None
}

fn main() {
    let matches = Command::new("btime")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Time a command, reporting elapsed time in BrightDate units")
        .arg(
            Arg::new("command")
                .help("Command and arguments to time")
                .num_args(1..)
                .required(true)
                .trailing_var_arg(true),
        )
        .get_matches();

    let args: Vec<&str> = matches
        .get_many::<String>("command")
        .unwrap()
        .map(|s| s.as_str())
        .collect();

    let cpu_before = children_cpu_secs();
    let start = Instant::now();
    let start_bd = BrightDate::now();

    let status = process::Command::new(args[0])
        .args(&args[1..])
        .status()
        .unwrap_or_else(|e| {
            eprintln!("btime: failed to run '{}': {}", args[0], e);
            process::exit(1);
        });

    let elapsed_secs = start.elapsed().as_secs_f64();
    let elapsed_days = elapsed_secs / 86_400.0;
    let end_bd = BrightDate::now();
    let cpu_after = children_cpu_secs();

    eprintln!();
    eprintln!("real     {:.9} days  ({:.6} s)", elapsed_days, elapsed_secs);
    eprintln!("         {:.6} millidays", elapsed_days * 1_000.0);
    eprintln!("         {:.3} microdays", elapsed_days * 1_000_000.0);
    eprintln!("         {:.0} nanodays", elapsed_days * 1_000_000_000.0);

    if let (Some((u0, s0)), Some((u1, s1))) = (cpu_before, cpu_after) {
        let user = (u1 - u0).max(0.0);
        let sys = (s1 - s0).max(0.0);
        let cpu = user + sys;
        let cpu_pct = if elapsed_secs > 0.0 {
            (cpu / elapsed_secs) * 100.0
        } else {
            0.0
        };
        eprintln!("user     {:.6} s  ({:.6} millidays)", user, user / 86.4);
        eprintln!("sys      {:.6} s  ({:.6} millidays)", sys, sys / 86.4);
        eprintln!("cpu      {:.1}%", cpu_pct);
    }

    eprintln!("start    {:.9}", start_bd.value);
    eprintln!("end      {:.9}", end_bd.value);

    process::exit(status.code().unwrap_or(1));
}
