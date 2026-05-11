use chrono::{Datelike, NaiveDate, Utc};
use clap::{Arg, ArgAction, Command};

const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March",     "April",   "May",      "June",
    "July",    "August",   "September", "October", "November", "December",
];

fn days_in_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(
        if month == 12 { year + 1 } else { year },
        if month == 12 { 1 } else { month + 1 },
        1,
    )
    .and_then(|d| d.pred_opt())
    .map(|d| d.day())
    .unwrap_or(30)
}

fn prev_month(y: i32, m: u32) -> (i32, u32) {
    if m == 1 { (y - 1, 12) } else { (y, m - 1) }
}

fn next_month_pair(y: i32, m: u32) -> (i32, u32) {
    if m == 12 { (y + 1, 1) } else { (y, m + 1) }
}

fn print_month(year: i32, month: u32, today: &NaiveDate, no_color: bool, _precision: u8) {
    println!("{} {}", MONTH_NAMES[(month - 1) as usize], year);
    println!("Su Mo Tu We Th Fr Sa");

    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let start_dow = first.weekday().num_days_from_sunday();
    let dim = days_in_month(year, month);

    let reset  = if no_color { "" } else { "\x1b[0m" };
    let today_s = if no_color { "" } else { "\x1b[7m" };  // reverse video

    let mut col = 0u32;
    for _ in 0..start_dow {
        print!("   ");
        col += 1;
    }

    for day in 1..=dim {
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        let is_today = &date == today;

        if is_today {
            print!("{}{:2}{}", today_s, day, reset);
        } else {
            print!("{:2}", day);
        }

        col += 1;
        if col % 7 == 0 {
            println!();
        } else {
            print!(" ");
        }
    }
    if col % 7 != 0 {
        println!();
    }
    println!();
}

pub fn run(args: &[String]) -> i32 {
    let cmd = Command::new("bcal")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Display a calendar with BrightDate annotations")
        .arg(
            Arg::new("year")
                .value_name("YEAR")
                .help("Year to display (optional; defaults to current year when combined with --year)")
                .num_args(0..=1),
        )
        .arg(
            Arg::new("month")
                .value_name("MONTH")
                .help("Month (1-12) to display")
                .num_args(0..=1),
        )
        .arg(
            Arg::new("three")
                .short('3')
                .long("three")
                .help("Show three months: previous, current, next")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("year_view")
                .short('y')
                .long("year")
                .help("Show the full year")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no_color")
                .long("no-color")
                .help("Disable color output")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("precision")
                .short('p')
                .long("precision")
                .value_name("N")
                .help("BrightDate decimal places (default: 2)")
                .default_value("2"),
        );

    let matches = match cmd.try_get_matches_from(args) {
        Ok(m) => m,
        Err(e) => {
            let _ = e.print();
            return e.exit_code();
        }
    };

    let today = Utc::now().date_naive();
    let now_year = today.year();
    let now_month = today.month();

    let no_color = matches.get_flag("no_color");
    let precision: u8 = matches
        .get_one::<String>("precision")
        .and_then(|s| s.parse().ok())
        .unwrap_or(2)
        .clamp(1, 9);

    let year_view = matches.get_flag("year_view");
    let three_months = matches.get_flag("three");

    let positional: Vec<i32> = matches
        .get_many::<String>("year")
        .into_iter()
        .flatten()
        .chain(matches.get_many::<String>("month").into_iter().flatten())
        .filter_map(|s| s.parse().ok())
        .collect();

    let (disp_year, disp_month) = match positional.as_slice() {
        [y, m] => (*y, *m as u32),
        [y] if year_view => (*y, now_month),
        [m] => (now_year, *m as u32),
        _ => (now_year, now_month),
    };

    if year_view {
        println!("{}", disp_year);
        for m in 1u32..=12 {
            print_month(disp_year, m, &today, no_color, precision);
        }
    } else if three_months {
        let (py, pm) = prev_month(disp_year, disp_month);
        let (ny, nm) = next_month_pair(disp_year, disp_month);
        print_month(py, pm, &today, no_color, precision);
        print_month(disp_year, disp_month, &today, no_color, precision);
        print_month(ny, nm, &today, no_color, precision);
    } else {
        print_month(disp_year, disp_month, &today, no_color, precision);
    }

    0
}
