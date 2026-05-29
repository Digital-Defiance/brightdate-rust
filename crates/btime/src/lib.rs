mod color;
mod output;

use bdate;
use brightdate::BrightDate;
use clap::{Arg, ArgAction, Command};
use color::{parse_color_scheme, parse_color_when, ColorScheme, ColorWhen, Colors};
use output::print_timing_report;
use std::process;
use std::time::Instant;

#[cfg(unix)]
fn children_cpu_secs() -> Option<(f64, f64)> {
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

fn parse_color_setting(value: &str, setting: &str) -> ColorWhen {
    match parse_color_when(value) {
        Ok(when) => when,
        Err(e) => {
            eprintln!("btime: {e}");
            eprintln!("btime: try `btime --help` for valid {setting} values");
            process::exit(2);
        }
    }
}

fn parse_scheme_setting(value: &str) -> ColorScheme {
    match parse_color_scheme(value) {
        Ok(scheme) => scheme,
        Err(e) => {
            eprintln!("btime: {e}");
            eprintln!("btime: try `btime --help` for valid --color-scheme values");
            process::exit(2);
        }
    }
}

fn resolve_color_settings(matches: &clap::ArgMatches) -> Colors {
    let color_when = if matches.get_flag("no_color") {
        ColorWhen::Never
    } else if let Some(value) = matches.get_one::<String>("color") {
        parse_color_setting(value, "--color values")
    } else if let Ok(value) = std::env::var("BTIME_COLOR") {
        parse_color_setting(&value, "BTIME_COLOR values")
    } else {
        ColorWhen::Auto
    };

    let scheme = if let Some(value) = matches.get_one::<String>("color_scheme") {
        parse_scheme_setting(value)
    } else if let Ok(value) = std::env::var("BTIME_COLOR_SCHEME") {
        parse_scheme_setting(&value)
    } else {
        ColorScheme::Default
    };

    Colors::resolve(color_when, scheme)
}

pub fn run(args: &[String]) -> i32 {
    // No command given → show current BrightDate date/time
    if args.len() <= 1 {
        return bdate::run(args);
    }

    let cmd = Command::new("btime")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Time a command, reporting elapsed time in BrightDate units")
        .arg(
            Arg::new("color")
                .long("color")
                .value_name("WHEN")
                .num_args(0..=1)
                .default_missing_value("always")
                .help("Colorize timing output: auto, always, never, ansi, or truecolor")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("no_color")
                .long("no-color")
                .help("Disable color output")
                .action(ArgAction::SetTrue)
                .conflicts_with("color"),
        )
        .arg(
            Arg::new("color_scheme")
                .long("color-scheme")
                .value_name("SCHEME")
                .help("Color palette: default or bright")
                .default_value("default"),
        )
        .arg(
            Arg::new("command")
                .help("Command and arguments to time")
                .num_args(1..)
                .required(true)
                .trailing_var_arg(true),
        );

    let matches = match cmd.try_get_matches_from(args) {
        Ok(m) => m,
        Err(e) => {
            let _ = e.print();
            return e.exit_code();
        }
    };

    let colors = resolve_color_settings(&matches);

    let cmd_args: Vec<&str> = matches
        .get_many::<String>("command")
        .unwrap()
        .map(|s| s.as_str())
        .collect();

    let cpu_before = children_cpu_secs();
    let start = Instant::now();
    let start_bd = BrightDate::now();

    let status = process::Command::new(cmd_args[0])
        .args(&cmd_args[1..])
        .status()
        .unwrap_or_else(|e| {
            eprintln!("btime: failed to run '{}': {}", cmd_args[0], e);
            process::exit(1);
        });

    let elapsed_secs = start.elapsed().as_secs_f64();
    let elapsed_days = elapsed_secs / 86_400.0;
    let end_bd = BrightDate::now();
    let cpu_after = children_cpu_secs();

    let (user, sys, cpu_pct) = if let (Some((u0, s0)), Some((u1, s1))) = (cpu_before, cpu_after) {
        let user = (u1 - u0).max(0.0);
        let sys = (s1 - s0).max(0.0);
        let cpu = user + sys;
        let cpu_pct = if elapsed_secs > 0.0 {
            (cpu / elapsed_secs) * 100.0
        } else {
            0.0
        };
        (Some(user), Some(sys), Some(cpu_pct))
    } else {
        (None, None, None)
    };

    print_timing_report(
        &colors,
        elapsed_secs,
        elapsed_days,
        user,
        sys,
        cpu_pct,
        start_bd.value,
        end_bd.value,
    );

    status.code().unwrap_or(1)
}
