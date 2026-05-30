use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("btime").expect("btime binary not found")
}

#[test]
fn version_flag() {
    cmd().arg("--version").assert().success().stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_flag() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("BrightDate"))
        .stdout(predicate::str::contains("--color"))
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--portability"))
        .stdout(predicate::str::contains("--verbose"));
}

#[test]
fn gnu_portability_output() {
    cmd()
        .args(["-p", "true"])
        .assert()
        .success()
        .stderr(predicate::str::contains("real "))
        .stderr(predicate::str::contains("user "))
        .stderr(predicate::str::contains("sys "));
}

#[test]
fn gnu_custom_format() {
    cmd()
        .args(["-f", "elapsed=%e cpu=%P", "true"])
        .assert()
        .success()
        .stderr(predicate::str::contains("elapsed="))
        .stderr(predicate::str::contains("cpu="));
}

#[test]
fn gnu_verbose_output() {
    cmd()
        .args(["-v", "true"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Command being timed"))
        .stderr(predicate::str::contains("User time (seconds)"));
}

#[test]
fn gnu_quiet_suppresses_nonzero_message() {
    let out = cmd()
        .args(["-q", "false"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(
        !text.contains("non-zero status"),
        "expected -q to suppress abnormal exit message, got:\n{text}"
    );
}

#[test]
fn gnu_time_env_format() {
    cmd()
        .env("TIME", "fmt=%e")
        .args(["true"])
        .assert()
        .success()
        .stderr(predicate::str::contains("fmt="));
}

#[test]
fn gnu_portability_has_no_brightdate_fields() {
    cmd()
        .args(["-p", "true"])
        .assert()
        .success()
        .stderr(predicate::str::contains("real "))
        .stderr(predicate::str::is_empty().not())
        .stderr(predicate::str::contains("millidays").not())
        .stderr(predicate::str::contains("start").not());
}

#[test]
fn gnu_verbose_overrides_format() {
    cmd()
        .args(["-v", "-f", "ignored=%e", "true"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Command being timed"))
        .stderr(predicate::str::contains("ignored=").not());
}

#[test]
fn gnu_abnormal_exit_message_without_quiet() {
    let out = cmd()
        .args(["-f", "%x", "false"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(
        text.contains("non-zero status 1"),
        "expected abnormal exit message without -q, got:\n{text}"
    );
}

#[test]
fn gnu_format_command_specifier() {
    cmd()
        .args(["-f", "ran %C", "echo", "x"])
        .assert()
        .success()
        .stderr(predicate::str::contains("ran echo x"));
}

#[test]
fn gnu_format_brightdate_extensions_cli() {
    cmd()
        .args(["-f", "bd=%B", "true"])
        .assert()
        .success()
        .stderr(predicate::str::contains("bd="));
}

#[test]
fn gnu_color_disabled_in_portability_mode() {
    let out = cmd()
        .args(["-p", "--color=always", "true"])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(
        !text.contains("\x1b["),
        "GNU -p output should not be colorized, got:\n{text}"
    );
}

#[test]
fn gnu_output_to_file() {
    let path = std::env::temp_dir().join(format!("btime-o-{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&path);

    cmd()
        .args(["-p", "-o"])
        .arg(&path)
        .arg("true")
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    let text = std::fs::read_to_string(&path).expect("output file should exist");
    assert!(text.contains("real "));
    assert!(text.contains("user "));
    assert!(text.contains("sys "));

    let _ = std::fs::remove_file(&path);
}

#[test]
fn gnu_output_append_to_file() {
    let path = std::env::temp_dir().join(format!("btime-a-{}.txt", std::process::id()));
    let _ = std::fs::remove_file(&path);

    cmd()
        .args(["-f", "line=%e", "-o"])
        .arg(&path)
        .arg("true")
        .assert()
        .success();

    cmd()
        .args(["-f", "line=%e", "-a", "-o"])
        .arg(&path)
        .arg("true")
        .assert()
        .success();

    let text = std::fs::read_to_string(&path).expect("output file should exist");
    assert_eq!(text.matches("line=").count(), 2, "expected two appended lines:\n{text}");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn gnu_long_format_option() {
    cmd()
        .args(["--format=wall=%e", "true"])
        .assert()
        .success()
        .stderr(predicate::str::contains("wall="));
}

#[test]
fn no_color_flag() {
    let out = cmd()
        .args(["--no-color", "true"])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(
        !text.contains("\x1b["),
        "expected no ANSI codes with --no-color, but found some"
    );
    assert!(text.contains("real"));
}

#[test]
fn color_always_ansi() {
    let out = cmd()
        .args(["--color=ansi", "true"])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(
        text.contains("\x1b["),
        "expected ANSI codes with --color=ansi, got:\n{text}"
    );
    assert!(
        !text.contains("38;2;"),
        "expected ANSI 16-color codes, not truecolor, got:\n{text}"
    );
}

#[test]
fn color_always_truecolor() {
    let out = cmd()
        .args(["--color=truecolor", "true"])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(
        text.contains("38;2;"),
        "expected truecolor codes with --color=truecolor, got:\n{text}"
    );
}

#[test]
fn color_plain_disables_ansi() {
    let out = cmd()
        .args(["--color=plain", "true"])
        .assert()
        .success()
        .get_output()
        .stderr
        .clone();
    let text = String::from_utf8_lossy(&out);
    assert!(
        !text.contains("\x1b["),
        "expected no ANSI codes with --color=plain, but found some"
    );
}

#[test]
fn invalid_color_mode() {
    cmd()
        .args(["--color=nope", "true"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid color mode"));
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
