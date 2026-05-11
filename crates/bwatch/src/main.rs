use brightdate::BrightDate;
use clap::{Arg, Command};
use std::io::{self, Write};
use std::process;
use std::time::Duration;

fn main() {
    let matches = Command::new("bwatch")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Repeatedly run a command, showing elapsed time in BrightDate units")
        .arg(
            Arg::new("interval")
                .short('n')
                .long("interval")
                .value_name("SECONDS")
                .help("Interval between runs in seconds (default: 2.0)")
                .default_value("2.0"),
        )
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .value_name("N")
                .help("Number of times to run (0 = infinite)")
                .default_value("0"),
        )
        .arg(
            Arg::new("command")
                .help("Command to run")
                .num_args(1..)
                .required(true)
                .trailing_var_arg(true),
        )
        .get_matches();

    let interval_secs: f64 = matches
        .get_one::<String>("interval")
        .and_then(|s| s.parse().ok())
        .unwrap_or(2.0);

    let count: u64 = matches
        .get_one::<String>("count")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let args: Vec<&str> = matches
        .get_many::<String>("command")
        .unwrap()
        .map(|s| s.as_str())
        .collect();

    let start_bd = BrightDate::now();
    let mut iteration: u64 = 0;

    loop {
        // Clear screen
        print!("\x1b[2J\x1b[H");
        let _ = io::stdout().flush();

        let now = BrightDate::now();
        let elapsed = now.difference(&start_bd) * 86_400.0; // seconds

        println!(
            "Every {:.1}s: {}  —  BD: {}  (elapsed: {:.3}s / {:.5} days)\n",
            interval_secs,
            args.join(" "),
            now,
            elapsed,
            now.difference(&start_bd)
        );

        let _ = process::Command::new(args[0])
            .args(&args[1..])
            .status();

        iteration += 1;
        if count > 0 && iteration >= count {
            break;
        }

        std::thread::sleep(Duration::from_secs_f64(interval_secs));
    }
}
