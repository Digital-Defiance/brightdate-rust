use brightdate::{BrightDate, BrightDateOptions};
use clap::{Arg, ArgAction, Command};

fn print_all(bd: &BrightDate) {
    let c = bd.decompose();
    let unix_ms = bd.to_unix_ms();
    let unix_s = unix_ms / 1_000.0;
    let unix_s_i = unix_ms as i64 / 1000;
    let tod_s = ((unix_s_i % 86400) + 86400) % 86400;
    let hh = tod_s / 3600;
    let mm = (tod_s % 3600) / 60;
    let ss = tod_s % 60;
    let ms = (unix_ms.abs() as u64) % 1000;

    println!("BrightDate  : {}", bd.format());
    println!("Millidays   : {:.3} md", bd.value * 1_000.0);
    println!("Microdays   : {:.0} \u{00b5}d", bd.value * 1_000_000.0);
    println!("Nanodays    : {:.0} nd", bd.value * 1_000_000_000.0);
    println!("ISO 8601    : {}", bd.to_iso());
    println!("Julian Date : {:.6}", bd.to_julian_date());
    println!("Mod. Julian : {:.6}", bd.to_modified_julian_date());
    let (wk, secs) = bd.to_gps_time();
    println!("GPS         : week {} + {:.3} s", wk, secs);
    println!("Unix ms     : {}", unix_ms as i64);
    println!("Unix s      : {:.3}", unix_s);
    println!("Breakdown   : day {} + {:02}:{:02}:{:02}.{:03}", c.day, hh, mm, ss, ms);
    println!("TAI offset  : {} s", bd.tai_utc_offset_seconds());
}

fn print_breakdown(bd: &BrightDate) {
    let c = bd.decompose();
    let unix_ms = bd.to_unix_ms();
    let unix_s_i = unix_ms as i64 / 1000;
    let tod_s = ((unix_s_i % 86400) + 86400) % 86400;
    let hh = tod_s / 3600;
    let mm = (tod_s % 3600) / 60;
    let ss = tod_s % 60;
    let ms_part = (unix_ms.abs() as u64) % 1000;
    // Reconstruct from the decomposed integer slices so every line is
    // derived from the same canonical nanoday-quantized value.
    let canonical = c.day as f64
        + (c.millidays as f64 * 1e-3)
        + (c.microdays as f64 * 1e-6)
        + (c.nanodays as f64 * 1e-9);
    println!("BrightDate  : {:.9}", canonical);
    println!("Whole days  : {}", c.day);
    println!(
        "Day fraction: 0.{:03}{:03}{:03}",
        c.millidays, c.microdays, c.nanodays
    );
    println!("Hours       : {:02}:{:02}:{:02}.{:03}", hh, mm, ss, ms_part);
    println!("Milliday    : {} md", c.millidays);
    println!("Microday    : {} \u{00b5}d", c.microdays);
    println!("Nanoday     : {} nd", c.nanodays);
    println!("ISO 8601    : {}", bd.to_iso());
}

fn parse_input(input: &str) -> Result<BrightDate, String> {
    // Try "JD:<value>"
    if let Some(rest) = input.strip_prefix("JD:").or_else(|| input.strip_prefix("jd:")) {
        if let Ok(jd) = rest.parse::<f64>() {
            return Ok(BrightDate::from_julian_date(jd));
        }
    }
    // Try "MJD:<value>"
    if let Some(rest) = input.strip_prefix("MJD:").or_else(|| input.strip_prefix("mjd:")) {
        if let Ok(mjd) = rest.parse::<f64>() {
            return Ok(BrightDate::from_modified_julian_date(mjd));
        }
    }
    // Try ISO 8601
    if let Ok(bd) = BrightDate::from_iso(input) {
        return Ok(bd);
    }
    // Try numeric: large values (>1e10) treated as Unix ms, otherwise BrightDate decimal
    if let Ok(v) = input.parse::<f64>() {
        if v.abs() > 1e10 {
            return BrightDate::from_unix_ms(v).map_err(|e| e.to_string());
        }
        return Ok(BrightDate::from_value(v));
    }
    Err(format!("cannot parse '{}' as BrightDate, ISO 8601, JD:, MJD:, or Unix ms", input))
}

fn main() {
    let matches = Command::new("bdate")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Display, convert, and diff dates in BrightDate format")
        .arg(
            Arg::new("date")
                .value_name("DATE")
                .help("Date to convert: BrightDate decimal, ISO 8601, JD:<value>, MJD:<value>, or Unix ms")
                .num_args(0..=1),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .help("Output format: bright (default), millidays, iso, unix, julian, gps, all")
                .default_value("bright"),
        )
        .arg(
            Arg::new("precision")
                .short('p')
                .long("precision")
                .value_name("N")
                .help("Decimal places (1-12, default: 5)")
                .default_value("5"),
        )
        .arg(
            Arg::new("breakdown")
                .short('b')
                .long("breakdown")
                .help("Show full decomposition of the date")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("tai")
                .long("tai")
                .help("Show/convert in TAI timescale")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("diff")
                .short('d')
                .long("diff")
                .value_name("DATE2")
                .help("Compute difference between DATE and DATE2 (in days)"),
        )
        .get_matches();

    let precision: u8 = matches
        .get_one::<String>("precision")
        .and_then(|s| s.parse().ok())
        .unwrap_or(5)
        .clamp(1, 12);

    let use_tai = matches.get_flag("tai");
    let do_breakdown = matches.get_flag("breakdown");

    let raw = match matches.get_one::<String>("date") {
        Some(input) => match parse_input(input) {
            Ok(b) => b,
            Err(e) => { eprintln!("bdate: {e}"); std::process::exit(1); }
        },
        None => BrightDate::now(),
    };

    let opts = BrightDateOptions { precision: Some(precision), use_tai: Some(use_tai) };
    let bd = BrightDate::from_value_with_options(raw.value, opts);

    // --diff mode
    if let Some(date2_str) = matches.get_one::<String>("diff") {
        let bd2 = match parse_input(date2_str) {
            Ok(b) => b,
            Err(e) => { eprintln!("bdate: {e}"); std::process::exit(1); }
        };
        let diff = bd2.value - bd.value;
        println!("From : {} ({})", bd.format(), bd.to_iso());
        println!("To   : {} ({})", bd2.format(), bd2.to_iso());
        println!("Diff : {:+.prec$} days", diff, prec = precision as usize);
        println!("     = {:+.3} millidays", diff * 1_000.0);
        println!("     = {:+.0} microdays", diff * 1_000_000.0);
        let abs_s = diff.abs() * 86_400.0;
        let days = diff.abs().floor() as u64;
        let rem_s = abs_s - days as f64 * 86_400.0;
        let hours = (rem_s / 3_600.0).floor() as u64;
        let minutes = ((rem_s % 3_600.0) / 60.0).floor() as u64;
        let seconds = rem_s % 60.0;
        if days > 0 {
            println!("     = {}d {:02}h {:02}m {:.3}s", days, hours, minutes, seconds);
        } else {
            println!("     = {:02}h {:02}m {:.3}s", hours, minutes, seconds);
        }
        return;
    }

    if do_breakdown {
        print_breakdown(&bd);
        return;
    }

    match matches.get_one::<String>("format").map(|s| s.as_str()).unwrap_or("bright") {
        "millidays" => println!("{:.3} md", bd.value * 1_000.0),
        "iso"       => println!("{}", bd.to_iso()),
        "unix"      => println!("{}", bd.to_unix_ms() as i64),
        "julian"    => println!("{:.6}", bd.to_julian_date()),
        "gps" => {
            let (wk, secs) = bd.to_gps_time();
            println!("GPS week {} + {:.3} s", wk, secs);
        }
        "all" => print_all(&bd),
        _ => {
            let c = bd.decompose();
            println!("{} ({:.3} md)", bd.format(), c.millidays);
        }
    }
}
