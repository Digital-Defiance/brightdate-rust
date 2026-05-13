use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("bdate").expect("bdate binary not found")
}

// ── version / help ──────────────────────────────────────────────────────────

#[test]
fn version_flag() {
    cmd().arg("--version").assert().success().stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_flag() {
    cmd().arg("--help").assert().success().stdout(predicate::str::contains("BrightDate"));
}

// ── epoch: BrightDate 0 == J2000.0 == 2000-01-01T11:58:55.816 UTC ─────────────

#[test]
fn epoch_from_zero() {
    // bdate 0 → "0.00000 (0.000 md)"
    cmd()
        .arg("0")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.00000"));
}

#[test]
fn epoch_from_jd() {
    // J2000.0 Julian Date = 2451545.0 → BrightDate 0
    cmd()
        .arg("JD:2451545.0")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.00000"));
}

#[test]
fn epoch_from_iso() {
    // J2000.0 in UTC = 2000-01-01T11:58:55.816 ≡ BrightDate 0.
    cmd()
        .arg("2000-01-01T11:58:55.816Z")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.00000"));
}

// ── format flags ────────────────────────────────────────────────────────────

#[test]
fn format_iso() {
    cmd()
        .args(["--format", "iso", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2000-01-01"));
}

#[test]
fn format_millidays() {
    cmd()
        .args(["--format", "millidays", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1000.000 md"));
}

#[test]
fn format_julian() {
    // BrightDate 0 → JD 2451545.0
    cmd()
        .args(["--format", "julian", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("2451545"));
}

#[test]
fn format_all() {
    cmd()
        .args(["--format", "all", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ISO 8601"))
        .stdout(predicate::str::contains("Julian Date"))
        .stdout(predicate::str::contains("GPS"));
}

#[test]
fn format_unix() {
    // BrightDate 0 = J2000.0 = 2000-01-01T11:58:55.816 UTC = Unix ms 946727935816.
    cmd()
        .args(["--format", "unix", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("946727935816"));
}

#[test]
fn format_gps() {
    cmd()
        .args(["--format", "gps", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("GPS week"));
}

// ── breakdown ────────────────────────────────────────────────────────────────

#[test]
fn breakdown_flag() {
    cmd()
        .args(["--breakdown", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Whole days"))
        .stdout(predicate::str::contains("Day fraction"))
        .stdout(predicate::str::contains("ISO 8601"));
}

// ── precision ────────────────────────────────────────────────────────────────

#[test]
fn precision_flag() {
    let out = cmd()
        .args(["--precision", "3", "0"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&out);
    // With precision 3, output should be "0.000 (0.000 md)"
    assert!(text.contains("0.000"), "expected '0.000' in: {text}");
}

// ── diff ─────────────────────────────────────────────────────────────────────

#[test]
fn diff_same() {
    cmd()
        .args(["0", "--diff", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("+0.00000 days"));
}

#[test]
fn diff_one_day() {
    cmd()
        .args(["0", "--diff", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("+1.00000 days"));
}

#[test]
fn diff_negative() {
    cmd()
        .args(["1", "--diff", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-1.00000 days"));
}

#[test]
fn diff_hours_display() {
    // 0.5 days = 12 hours
    cmd()
        .args(["0", "--diff", "0.5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("+0.50000 days"))
        .stdout(predicate::str::contains("+500.000 millidays"));
}

// ── MJD prefix ───────────────────────────────────────────────────────────────

#[test]
fn mjd_prefix() {
    // MJD 51544.5 = J2000.0 epoch = BrightDate 0
    cmd()
        .arg("MJD:51544.5")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.00000"));
}

// ── no args → current time (just check it's a float) ────────────────────────

#[test]
fn no_args_outputs_current_time() {
    let out = cmd().assert().success().get_output().stdout.clone();
    let text = String::from_utf8_lossy(&out);
    // current BrightDate should be a positive number around 9000-10000
    let first_word = text.split_whitespace().next().unwrap_or("");
    let v: f64 = first_word.parse().expect("first token should be a number");
    assert!(v > 8000.0, "expected current BrightDate > 8000, got {v}");
}

// ── TAI flag ─────────────────────────────────────────────────────────────────

#[test]
fn tai_flag_at_epoch() {
    // --tai 0 still shows 0.00000 (TAI offset at J2000 epoch is defined)
    cmd()
        .args(["--tai", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0.00000"));
}

// ── negative BD values ───────────────────────────────────────────────────────

#[test]
fn negative_bd_value() {
    // BD -1 is one day before J2000 epoch (1999-12-31T12:00:00Z)
    cmd()
        .args(["--", "-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-1.00000"));
}

// ── Unix ms input ────────────────────────────────────────────────────────────

#[test]
fn unix_ms_input_at_epoch() {
    // 946727935816 is J2000.0 in Unix milliseconds → BD 0.
    cmd()
        .arg("946727935816")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.00000"));
}

// ── explicit bright format ───────────────────────────────────────────────────

#[test]
fn format_bright_explicit() {
    cmd()
        .args(["--format", "bright", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1.00000"));
}

// ── breakdown detail lines ───────────────────────────────────────────────────

#[test]
fn breakdown_shows_hours() {
    cmd()
        .args(["--breakdown", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hours"));
}

#[test]
fn breakdown_shows_microday() {
    cmd()
        .args(["--breakdown", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Microday"));
}

// ── format all ───────────────────────────────────────────────────────────────

#[test]
fn format_all_shows_millidays() {
    cmd()
        .args(["--format", "all", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Millidays"));
}

#[test]
fn format_all_shows_unix_ms() {
    cmd()
        .args(["--format", "all", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Unix ms"));
}

// ── ISO input ────────────────────────────────────────────────────────────────

#[test]
fn iso_date_input_2025() {
    // 2025-01-01T00:00:00Z corresponds to BD ~9131.50080 under TAI-coherent
    // semantics (37 leap seconds + 32.184 s TT−TAI past round noon).
    cmd()
        .arg("2025-01-01T00:00:00Z")
        .assert()
        .success()
        .stdout(predicate::str::contains("9131.50080"));
}
