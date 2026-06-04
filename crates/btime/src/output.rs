use crate::color::Colors;

/// Colored BrightDate timing report (stderr).
pub struct TimingReport {
    pub elapsed_secs: f64,
    pub elapsed_days: f64,
    pub user: Option<f64>,
    pub sys: Option<f64>,
    pub cpu_pct: Option<f64>,
    pub start_bd: f64,
    pub end_bd: f64,
}

pub fn print_timing_report(colors: &Colors, report: &TimingReport) {
    eprintln!();
    eprintln!(
        "{} {}{:.9}{} {}days{}  ({}{:.6} s{})",
        colors.label("real", colors.real),
        colors.value,
        report.elapsed_days,
        colors.reset,
        colors.unit,
        colors.reset,
        colors.value,
        report.elapsed_secs,
        colors.reset,
    );

    eprintln!(
        "         {}{:.6}{} {}millidays{}",
        colors.detail,
        report.elapsed_days * 1_000.0,
        colors.reset,
        colors.unit,
        colors.reset,
    );
    eprintln!(
        "         {}{:.3}{} {}microdays{}",
        colors.detail,
        report.elapsed_days * 1_000_000.0,
        colors.reset,
        colors.unit,
        colors.reset,
    );
    eprintln!(
        "         {}{:.0}{} {}nanodays{}",
        colors.detail,
        report.elapsed_days * 1_000_000_000.0,
        colors.reset,
        colors.unit,
        colors.reset,
    );

    if let (Some(user), Some(sys), Some(cpu_pct)) = (report.user, report.sys, report.cpu_pct) {
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
        report.start_bd,
        colors.reset,
    );
    eprintln!(
        "{} {}{:.9}{}",
        colors.label("end", colors.stamp),
        colors.value,
        report.end_bd,
        colors.reset,
    );
}
