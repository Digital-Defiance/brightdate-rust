use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("btime").expect("btime binary not found")
}

#[test]
fn version_flag() {
    cmd().arg("--version").assert().success().stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn help_flag() {
    cmd().arg("--help").assert().success().stdout(predicate::str::contains("BrightDate"));
}

#[test]
fn times_true_command() {
    // `true` exits 0 instantly; elapsed output goes to stderr
    cmd()
        .args(["true"])
        .assert()
        .success()
        .stderr(predicate::str::contains("real"))
        .stderr(predicate::str::contains("millidays"))
        .stderr(predicate::str::contains("start"))
        .stderr(predicate::str::contains("end"));
}

#[test]
fn times_echo_command() {
    cmd()
        .args(["echo", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"))   // echo output on stdout
        .stderr(predicate::str::contains("real"));
}

#[test]
fn propagates_exit_code() {
    // `false` exits 1
    cmd().args(["false"]).assert().failure();
}

#[test]
fn elapsed_is_small_fraction() {
    // `true` runs in microseconds; elapsed should be < 0.001 days
    let out = cmd()
        .args(["true"])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    let text = String::from_utf8_lossy(&out);
    // Extract "real X days" line
    for line in text.lines() {
        if line.starts_with("real") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // "real  <days> days ..."
            if let Some(days_str) = parts.get(1) {
                let days: f64 = days_str.parse().unwrap_or(999.0);
                assert!(
                    days < 0.01,
                    "expected elapsed < 0.01 days for `true`, got {days}"
                );
            }
            return;
        }
    }
    panic!("did not find 'real' line in btime output: {text}");
}

// ── stdout passthrough ───────────────────────────────────────────────────────

#[test]
fn stdout_passthrough() {
    // btime should forward the child command's stdout unchanged
    cmd()
        .args(["echo", "unique_sentinel_bd_token"])
        .assert()
        .success()
        .stdout(predicate::str::contains("unique_sentinel_bd_token"));
}

#[test]
fn timing_command_with_multiple_args() {
    cmd()
        .args(["echo", "hello", "world"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

// ── stderr output fields ─────────────────────────────────────────────────────

#[test]
fn stderr_has_days_unit() {
    cmd()
        .args(["true"])
        .assert()
        .success()
        .stderr(predicate::str::contains("days"));
}

#[test]
fn stderr_has_seconds_unit() {
    // output format is "real X days (Y s)"
    cmd()
        .args(["true"])
        .assert()
        .success()
        .stderr(predicate::str::contains(" s)"));
}

// ── start/end BrightDate values ──────────────────────────────────────────────

#[test]
fn start_bd_is_parseable() {
    let err = cmd()
        .args(["true"])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    let text = String::from_utf8_lossy(&err);
    for line in text.lines() {
        if line.trim_start().starts_with("start") {
            let token = line.split_whitespace().nth(1).unwrap_or("nan");
            let v: f64 = token.parse().expect("start BD should be numeric");
            assert!(v > 0.0, "start BD should be positive (after J2000), got {v}");
            return;
        }
    }
    panic!("did not find 'start' line in btime stderr: {text}");
}

#[test]
fn end_bd_is_parseable() {
    let err = cmd()
        .args(["true"])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    let text = String::from_utf8_lossy(&err);
    for line in text.lines() {
        if line.trim_start().starts_with("end") {
            let token = line.split_whitespace().nth(1).unwrap_or("nan");
            let v: f64 = token.parse().expect("end BD should be numeric");
            assert!(v > 0.0, "end BD should be positive (after J2000), got {v}");
            return;
        }
    }
    panic!("did not find 'end' line in btime stderr: {text}");
}
