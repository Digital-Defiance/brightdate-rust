use brightdate::BrightDate;
use clap::{Arg, Command};
use std::time::{Duration, Instant};
use std::thread;

pub fn run(args: &[String]) -> i32 {
    let cmd = Command::new("bwatch")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Execute a program periodically, showing output (BrightDate clock)")
        .arg(
            Arg::new("interval")
                .short('n')
                .long("interval")
                .value_name("SECONDS")
                .help("Seconds to wait between updates (default: 2.0)")
                .default_value("2.0"),
        )
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .value_name("N")
                .help("Exit after N iterations (0 = run forever, default: 0)")
                .default_value("0"),
        )
        .arg(
            Arg::new("command")
                .help("Command and arguments to run")
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

    let interval_secs: f64 = matches
        .get_one::<String>("interval")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(2.0)
        .max(0.1);

    let max_count: u64 = matches
        .get_one::<String>("count")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let cmd_args: Vec<&str> = matches
        .get_many::<String>("command")
        .unwrap()
        .map(|s| s.as_str())
        .collect();

    let sleep_dur = Duration::from_secs_f64(interval_secs);

    let mut iteration: u64 = 0;
    loop {
        iteration += 1;
        let now = BrightDate::now();

        // Clear screen and move to top-left
        print!("\x1b[2J\x1b[H");
        println!(
            "Every {:.1}s: {}   BD: {:.5}  (Ctrl-C to stop)",
            interval_secs,
            cmd_args.join(" "),
            now.value
        );
        println!();

        let t0 = Instant::now();
        let _ = std::process::Command::new(cmd_args[0])
            .args(&cmd_args[1..])
            .status();
        let elapsed = t0.elapsed().as_secs_f64();
        let elapsed_days = elapsed / 86400.0;
        println!("elapsed: {:.3}s / {:.5} days", elapsed, elapsed_days);

        if max_count > 0 && iteration >= max_count {
            break;
        }

        thread::sleep(sleep_dur);
    }

    0
}
