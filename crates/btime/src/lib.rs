mod color;
mod format;
mod output;
mod timing;

use clap::{Arg, ArgAction, Command};
use color::{parse_color_scheme, parse_color_when, ColorScheme, ColorWhen, Colors};
use format::{print_abnormal_termination, summarize, DEFAULT_FORMAT, POSIX_FORMAT, VERBOSE_FORMAT};
use output::{print_timing_report, TimingReport};
use std::fs::OpenOptions;
use std::io::{self, Write};
use timing::{run_command, wait_status_to_exit_code, TimingResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReportStyle {
    BrightDate,
    Gnu,
}

struct ReportConfig {
    style: ReportStyle,
    format: String,
    is_posix: bool,
    quiet: bool,
}

struct OutputTarget {
    append: bool,
    path: Option<String>,
}

fn parse_color_setting(value: &str, setting: &str) -> Result<ColorWhen, i32> {
    parse_color_when(value).map_err(|e| {
        eprintln!("btime: {e}");
        eprintln!("btime: try `btime --help` for valid {setting} values");
        2
    })
}

fn parse_scheme_setting(value: &str) -> Result<ColorScheme, i32> {
    parse_color_scheme(value).map_err(|e| {
        eprintln!("btime: {e}");
        eprintln!("btime: try `btime --help` for valid --color-scheme values");
        2
    })
}

fn resolve_color_settings(matches: &clap::ArgMatches) -> Result<Colors, i32> {
    let color_when = if matches.get_flag("no_color") {
        ColorWhen::Never
    } else if let Some(value) = matches.get_one::<String>("color") {
        parse_color_setting(value, "--color values")?
    } else if let Ok(value) = std::env::var("BTIME_COLOR") {
        parse_color_setting(&value, "BTIME_COLOR values")?
    } else {
        ColorWhen::Auto
    };

    let scheme = if let Some(value) = matches.get_one::<String>("color_scheme") {
        parse_scheme_setting(value)?
    } else if let Ok(value) = std::env::var("BTIME_COLOR_SCHEME") {
        parse_scheme_setting(&value)?
    } else {
        ColorScheme::Default
    };

    Ok(Colors::resolve(color_when, scheme))
}

fn resolve_report_config(matches: &clap::ArgMatches) -> ReportConfig {
    if matches.get_flag("verbose") {
        return ReportConfig {
            style: ReportStyle::Gnu,
            format: VERBOSE_FORMAT.to_string(),
            is_posix: false,
            quiet: matches.get_flag("quiet"),
        };
    }

    if matches.get_flag("portability") {
        return ReportConfig {
            style: ReportStyle::Gnu,
            format: POSIX_FORMAT.to_string(),
            is_posix: true,
            quiet: matches.get_flag("quiet"),
        };
    }

    if let Some(fmt) = matches.get_one::<String>("format") {
        return ReportConfig {
            style: ReportStyle::Gnu,
            format: fmt.clone(),
            is_posix: false,
            quiet: matches.get_flag("quiet"),
        };
    }

    if let Ok(fmt) = std::env::var("TIME") {
        return ReportConfig {
            style: ReportStyle::Gnu,
            format: fmt,
            is_posix: false,
            quiet: matches.get_flag("quiet"),
        };
    }

    ReportConfig {
        style: ReportStyle::BrightDate,
        format: DEFAULT_FORMAT.to_string(),
        is_posix: false,
        quiet: matches.get_flag("quiet"),
    }
}

fn open_output(target: &OutputTarget) -> io::Result<OutputWriter> {
    if let Some(path) = &target.path {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(target.append)
            .truncate(!target.append)
            .open(path)?;
        Ok(OutputWriter::File(file))
    } else {
        Ok(OutputWriter::Stderr(io::stderr()))
    }
}

enum OutputWriter {
    Stderr(io::Stderr),
    File(std::fs::File),
}

impl Write for OutputWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Stderr(s) => s.write(buf),
            Self::File(f) => f.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Stderr(s) => s.flush(),
            Self::File(f) => f.flush(),
        }
    }
}

fn emit_report(
    out: &mut OutputWriter,
    report: &ReportConfig,
    colors: &Colors,
    result: &TimingResult,
) -> io::Result<()> {
    match report.style {
        ReportStyle::Gnu => {
            if !report.quiet && !report.is_posix {
                print_abnormal_termination(out, result)?;
            }
            summarize(out, &report.format, result)
        }
        ReportStyle::BrightDate => {
            if is_stderr(out) && colors.enabled() {
                print_timing_report(
                    colors,
                    &TimingReport {
                        elapsed_secs: result.elapsed_secs(),
                        elapsed_days: result.elapsed_days(),
                        user: Some(result.user_secs()),
                        sys: Some(result.sys_secs()),
                        cpu_pct: result.cpu_percent_gnu().map(|p| p as f64),
                        start_bd: result.start_bd,
                        end_bd: result.end_bd,
                    },
                );
                Ok(())
            } else {
                emit_brightdate_plain(out, result)
            }
        }
    }
}

fn is_stderr(out: &OutputWriter) -> bool {
    matches!(out, OutputWriter::Stderr(_))
}

fn emit_brightdate_plain(out: &mut dyn Write, result: &TimingResult) -> io::Result<()> {
    writeln!(out)?;
    writeln!(
        out,
        "real     {:.9} days  ({:.6} s)",
        result.elapsed_days(),
        result.elapsed_secs()
    )?;
    writeln!(out, "         {:.6} millidays", result.elapsed_days() * 1_000.0)?;
    writeln!(
        out,
        "         {:.3} microdays",
        result.elapsed_days() * 1_000_000.0
    )?;
    writeln!(
        out,
        "         {:.0} nanodays",
        result.elapsed_days() * 1_000_000_000.0
    )?;
    writeln!(
        out,
        "user     {:.6} s  ({:.6} millidays)",
        result.user_secs(),
        result.user_secs() / 86.4
    )?;
    writeln!(
        out,
        "sys      {:.6} s  ({:.6} millidays)",
        result.sys_secs(),
        result.sys_secs() / 86.4
    )?;
    if let Some(cpu_pct) = result.cpu_percent_gnu() {
        writeln!(out, "cpu      {cpu_pct:.1}%")?;
    }
    writeln!(out, "start    {:.9}", result.start_bd)?;
    writeln!(out, "end      {:.9}", result.end_bd)
}

pub fn run(args: &[String]) -> i32 {
    if args.len() <= 1 {
        return bdate::run(args);
    }

    let cmd = Command::new("btime")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Time a command, reporting elapsed time in BrightDate units (GNU time compatible)")
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .help("Use GNU time-style FORMAT string for statistics output"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Write resource statistics to FILE instead of stderr"),
        )
        .arg(
            Arg::new("append")
                .short('a')
                .long("append")
                .help("Append statistics to FILE instead of overwriting (with -o)")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("portability")
                .short('p')
                .long("portability")
                .help("POSIX output: real/user/sys (GNU time -p)")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Do not print abnormal termination messages")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Print all resource usage information (GNU time -v)")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .value_name("WHEN")
                .num_args(0..=1)
                .default_missing_value("always")
                .help("Colorize BrightDate timing output: auto, always, never, plain, ansi, or truecolor")
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

    let colors = match resolve_color_settings(&matches) {
        Ok(colors) => colors,
        Err(code) => return code,
    };
    let report = resolve_report_config(&matches);
    let output = OutputTarget {
        path: matches.get_one::<String>("output").cloned(),
        append: matches.get_flag("append"),
    };

    let cmd_args: Vec<&str> = matches
        .get_many::<String>("command")
        .unwrap()
        .map(|s| s.as_str())
        .collect();

    let result = match run_command(&cmd_args) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("btime: failed to run '{}': {}", cmd_args[0], e);
            return if e.kind() == io::ErrorKind::NotFound {
                127
            } else {
                126
            };
        }
    };

    let mut out = match open_output(&output) {
        Ok(out) => out,
        Err(e) => {
            eprintln!("btime: {}", e);
            return 1;
        }
    };

    if let Err(e) = emit_report(&mut out, &report, &colors, &result) {
        eprintln!("btime: {e}");
        return 1;
    }

    let _ = out.flush();
    wait_status_to_exit_code(result.wait_status)
}
