use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("bwatch").expect("bwatch binary not found")
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
fn runs_once_with_count_1() {
    // --count 1 runs the command once and exits; no sleep before exit
    cmd()
        .args(["--count", "1", "echo", "bwatch-test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bwatch-test"));
}

#[test]
fn output_shows_brightdate_timestamp() {
    cmd()
        .args(["--count", "1", "true"])
        .assert()
        .success()
        .stdout(predicate::str::contains("BD:"));
}

#[test]
fn output_shows_elapsed() {
    cmd()
        .args(["--count", "1", "true"])
        .assert()
        .success()
        .stdout(predicate::str::contains("elapsed"));
}

#[test]
fn output_shows_interval() {
    cmd()
        .args(["--count", "1", "--interval", "0.1", "true"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1s"));
}

#[test]
fn runs_command_with_args() {
    // bwatch passes all arguments to the command
    cmd()
        .args(["--count", "1", "echo", "hello", "world"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

// ── header content ───────────────────────────────────────────────────────────

#[test]
fn header_contains_every() {
    // bwatch header line is: "Every Xs: <cmd>  —  BD: XXXXX  (elapsed: ...)"
    cmd()
        .args(["--count", "1", "true"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Every"));
}

#[test]
fn header_shows_command_name() {
    // header includes the command being run
    cmd()
        .args(["--count", "1", "echo", "marker"])
        .assert()
        .success()
        .stdout(predicate::str::contains("echo"));
}

// ── exit code propagation ────────────────────────────────────────────────────

#[test]
fn exits_success_even_when_command_fails() {
    // bwatch always exits 0 regardless of child exit code
    cmd()
        .args(["--count", "1", "false"])
        .assert()
        .success();
}

// ── count = 2 runs twice ──────────────────────────────────────────────────────

#[test]
fn count_two_runs_twice() {
    // Each run emits one "BD:" header; count=2 should yield exactly 2 headers
    let out = cmd()
        .args(["--count", "2", "--interval", "0.01", "true"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&out);
    let bd_count = text.matches("BD:").count();
    assert_eq!(bd_count, 2, "expected 2 BD: header lines for count=2, got {bd_count}: {text}");
}

// ── elapsed format ───────────────────────────────────────────────────────────

#[test]
fn elapsed_shows_seconds_suffix() {
    // elapsed line format: "elapsed: <n>s / <n> days"
    cmd()
        .args(["--count", "1", "true"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"elapsed: \d+\.\d+s").unwrap());
}

// ── stderr passthrough ───────────────────────────────────────────────────────

#[test]
fn command_stderr_passthrough() {
    // bwatch should forward the child's stderr
    cmd()
        .args(["--count", "1", "sh", "-c", "echo bwatch_err_sentinel >&2"])
        .assert()
        .success()
        .stderr(predicate::str::contains("bwatch_err_sentinel"));
}
