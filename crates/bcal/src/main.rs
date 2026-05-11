use brightdate::BrightDate;
use chrono::{Datelike, NaiveDate, Utc};
use clap::{Arg, ArgAction, Command};

const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

fn main() {
    let matches = Command::new("bcal")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Calendar with BrightDate values")
        .arg(
            Arg::new("year")
                .help("Year (default: current year)")
                .value_parser(clap::value_parser!(i32)),
        )
        .arg(
            Arg::new("month")
                .help("Month 1-12 (default: current month)")
                .value_parser(clap::value_parser!(u32)),
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
                .help("Show all twelve months of the year")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no_color")
                .long("no-color")
                .help("Disable ANSI color/highlight")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("precision")
                .short('p')
                .long("precision")
                .value_name("N")
                .help("Decimal places for BrightDate values (default: 2)")
                .default_value("2"),
        )
        .get_matches();

    let today = Utc::now().date_naive();
    let year = matches.get_one::<i32>("year").copied().unwrap_or(today.year());
    let month = matches.get_one::<u32>("month").copied().unwrap_or(today.month());
    let three = matches.get_flag("three");
    let year_view = matches.get_flag("year_view");
    let no_color = matches.get_flag("no_color");
    let precision: usize = matches
        .get_one::<String>("precision")
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);

    if year_view {
        for m in 1u32..=12 {
            print_month(year, m, today, no_color, precision);
            if m < 12 { println!(); }
        }
    } else if three {
        let (prev_year, prev_month) = prev_month(year, month);
        let (next_year, next_month) = next_month_pair(year, month);
        print_month(prev_year, prev_month, today, no_color, precision);
        println!();
        print_month(year, month, today, no_color, precision);
        println!();
        print_month(next_year, next_month, today, no_color, precision);
    } else {
        print_month(year, month, today, no_color, precision);
    }
}

fn prev_month(year: i32, month: u32) -> (i32, u32) {
    if month == 1 { (year - 1, 12) } else { (year, month - 1) }
}

fn next_month_pair(year: i32, month: u32) -> (i32, u32) {
    if month == 12 { (year + 1, 1) } else { (year, month + 1) }
}

fn print_month(year: i32, month: u32, today: NaiveDate, no_color: bool, precision: usize) {
    let first = NaiveDate::from_ymd_opt(year, month, 1).expect("invalid date");
    let dim = days_in_month(year, month);

    // Column width = precision + 5 (e.g. "9622.50" → 7 chars) + 1 space padding
    let col_w = precision + 6; // "NNNNN.pp" + leading space

    let header = format!("{} {}", MONTH_NAMES[(month - 1) as usize], year);
    let total_w = col_w * 7;
    println!("{:^width$}", header, width = total_w);

    let dow_labels = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
    for lbl in &dow_labels {
        print!("{:>width$}", lbl, width = col_w);
    }
    println!();

    let start_dow = first.weekday().num_days_from_sunday();
    let mut col = 0u32;

    // Leading blanks
    for _ in 0..start_dow {
        print!("{:>width$}", "", width = col_w);
        col += 1;
    }

    for day in 1..=dim {
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        let bd_str = BrightDate::from_iso(&format!("{date}T12:00:00.000Z"))
            .map(|b| format!("{:.prec$}", b.value, prec = precision))
            .unwrap_or_else(|_| "?".to_string());

        if date == today && !no_color {
            print!("\x1b[7m{:>width$}\x1b[0m", bd_str, width = col_w);
        } else {
            print!("{:>width$}", bd_str, width = col_w);
        }

        col += 1;
        if col.is_multiple_of(7) {
            println!();
        }
    }
    if !col.is_multiple_of(7) {
        println!();
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    let first_next = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
    let first_this = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    (first_next - first_this).num_days() as u32
}

