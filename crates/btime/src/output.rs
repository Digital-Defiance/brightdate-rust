use crate::color::Colors;

pub fn print_timing_report(
    colors: &Colors,
    elapsed_secs: f64,
    elapsed_days: f64,
    user: Option<f64>,
    sys: Option<f64>,
    cpu_pct: Option<f64>,
    start_bd: f64,
    end_bd: f64,
) {
    eprintln!();
    eprintln!(
        "{} {}{:.9}{} {}{}{}  ({}{:.6} s{})",
        colors.label("real", colors.real),
        colors.value,
        elapsed_days,
        colors.reset,
        colors.unit,
        "days",
        colors.reset,
        colors.value,
        elapsed_secs,
        colors.reset,
    );

    eprintln!(
        "         {}{:.6}{} {}{}{}",
        colors.detail,
        elapsed_days * 1_000.0,
        colors.reset,
        colors.unit,
        "millidays",
        colors.reset,
    );
    eprintln!(
        "         {}{:.3}{} {}{}{}",
        colors.detail,
        elapsed_days * 1_000_000.0,
        colors.reset,
        colors.unit,
        "microdays",
        colors.reset,
    );
    eprintln!(
        "         {}{:.0}{} {}{}{}",
        colors.detail,
        elapsed_days * 1_000_000_000.0,
        colors.reset,
        colors.unit,
        "nanodays",
        colors.reset,
    );

    if let (Some(user), Some(sys), Some(cpu_pct)) = (user, sys, cpu_pct) {
        eprintln!(
            "{} {}{:.6} s{}  ({}{:.6} millidays{})",
            colors.label("user", colors.user),
            colors.value,
            user,
            colors.reset,
            colors.unit,
            user / 86.4,
            colors.reset,
        );
        eprintln!(
            "{} {}{:.6} s{}  ({}{:.6} millidays{})",
            colors.label("sys", colors.sys),
            colors.value,
            sys,
            colors.reset,
            colors.unit,
            sys / 86.4,
            colors.reset,
        );
        let cpu_style = colors.cpu_pct_style(cpu_pct);
        eprintln!(
            "{} {}{:.1}%{}{}",
            colors.label("cpu", colors.cpu),
            cpu_style,
            cpu_pct,
            colors.reset,
            colors.reset,
        );
    }

    eprintln!(
        "{} {}{:.9}{}",
        colors.label("start", colors.stamp),
        colors.value,
        start_bd,
        colors.reset,
    );
    eprintln!(
        "{} {}{:.9}{}",
        colors.label("end", colors.stamp),
        colors.value,
        end_bd,
        colors.reset,
    );
}
