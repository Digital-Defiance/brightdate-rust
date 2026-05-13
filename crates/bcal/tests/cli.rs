use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("bcal").expect("bcal binary not found")
}

#[test]
fn version_flag() {
    cmd().arg("--version").assert().success().stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_flag() {
    cmd().arg("--help").assert().success().stdout(predicate::str::contains("BrightDate"));
}

#[test]
fn default_output_has_weekday_headers() {
    // Default (no args) shows current month — should have Su Mo Tu We Th Fr Sa
    cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("Su"))
        .stdout(predicate::str::contains("Sa"));
}

#[test]
fn specific_month_header() {
    // bcal 2000 1 → shows "January 2000"
    cmd()
        .args(["2000", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("January 2000"));
}

#[test]
fn january_2000_has_31_days() {
    // January 2000 has 31 days; Jan 31 at noon = BrightDate 30.00 (epoch is Jan 1 noon)
    cmd()
        .args(["2000", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("30.00"));
}

#[test]
fn february_2000_is_leap() {
    // 2000 is a leap year — February 29 at noon = BrightDate 59.00
    // (31 days of Jan = BD 30, then Feb 1=31, Feb 29=59)
    cmd()
        .args(["2000", "2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("59.00"));
}

#[test]
fn february_1900_not_leap() {
    // 1900 is not a leap year (divisible by 100, not 400) — February has only 28 days.
    // From 1900-01-01 to 2000-01-01 = 36524 days; 1900-01-01 noon = BD -36524.
    // Feb 28, 1900 noon = BD -36524 + 58 = -36466.00 (last day).
    // If Feb 29 existed it would be -36465.00, which should NOT appear.
    let out = cmd()
        .args(["1900", "2"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(
        text.contains("-36466"),
        "February 1900 should end on BD -36466 (Feb 28), got:\n{text}"
    );
    assert!(
        !text.contains("-36465"),
        "February 1900 should not contain -36465 (would be Feb 29), got:\n{text}"
    );
}

#[test]
fn three_month_view() {
    // -3 shows three months — should have headers for prev, current, next
    // We use 2000 6 (June) so prev=May, next=July — all known months
    cmd()
        .args(["-3", "2000", "6"])
        .assert()
        .success()
        .stdout(predicate::str::contains("May 2000"))
        .stdout(predicate::str::contains("June 2000"))
        .stdout(predicate::str::contains("July 2000"));
}

#[test]
fn year_view_has_all_months() {
    cmd()
        .args(["-y", "2000"])
        .assert()
        .success()
        .stdout(predicate::str::contains("January 2000"))
        .stdout(predicate::str::contains("June 2000"))
        .stdout(predicate::str::contains("December 2000"));
}

#[test]
fn brightdate_values_present() {
    // Each day shows a BrightDate value; for Jan 2000 they should be near 0
    let out = cmd()
        .args(["--precision", "2", "2000", "1"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&out);
    // Should contain numbers like "-0.50" or "0.50" (fractional days near epoch)
    assert!(
        text.contains("0.") || text.contains("-"),
        "expected BrightDate decimal values near J2000 epoch, got:\n{text}"
    );
}

#[test]
fn no_color_flag() {
    // With --no-color, there should be no ANSI escape sequences
    let out = cmd()
        .args(["--no-color", "2000", "1"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(
        !text.contains("\x1b["),
        "expected no ANSI codes with --no-color, but found some"
    );
}

#[test]
fn precision_flag() {
    // --precision 4 shows more decimal places.
    // Under v1.0 (TAI-coherent BD), Jan 1, 2000 UTC midnight lands at BD ~
    // 0.000_7 (= 64.184 s past J2000.0 / 86400). With precision 4 the
    // formatter rounds to .0001 for that row and .5001 for noon.
    let out = cmd()
        .args(["--precision", "4", "2000", "1"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(
        text.contains(".0007") || text.contains(".5007"),
        "expected 4 decimal places, got:\n{text}"
    );
}

// ── more month headers ────────────────────────────────────────────────────────

#[test]
fn march_2000_header() {
    cmd()
        .args(["2000", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("March 2000"));
}

#[test]
fn december_2000_header() {
    cmd()
        .args(["2000", "12"])
        .assert()
        .success()
        .stdout(predicate::str::contains("December 2000"));
}

#[test]
fn september_2000_header() {
    cmd()
        .args(["2000", "9"])
        .assert()
        .success()
        .stdout(predicate::str::contains("September 2000"));
}

// ── april 2000 has 30 days ────────────────────────────────────────────────────

#[test]
fn april_2000_last_day_bd() {
    // Jan(31) + Feb(29) + Mar(31) + Apr(1..30): Apr 30 noon = BD 120.00
    // Jan 1=0, Jan 31=30, Feb 29=59, Mar 31=90, Apr 30=120
    cmd()
        .args(["2000", "4"])
        .assert()
        .success()
        .stdout(predicate::str::contains("120.00"));
}

// ── invalid month exits with failure ─────────────────────────────────────────

#[test]
fn invalid_month_exits_failure() {
    cmd()
        .args(["2000", "13"])
        .assert()
        .failure();
}

// ── output always has decimal values ─────────────────────────────────────────

#[test]
fn default_output_has_decimal_values() {
    // Any bcal output shows BrightDate decimals containing a '.'
    cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("."));
}
