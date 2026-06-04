use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("buptime").expect("buptime binary not found")
}

/// Parse `up <days> days` from buptime stdout (not the boot BD field).
fn parse_uptime_days(stdout: &[u8]) -> f64 {
    let text = String::from_utf8_lossy(stdout);
    let tokens: Vec<&str> = text.split_whitespace().collect();
    for (i, token) in tokens.iter().enumerate() {
        if *token == "up" {
            let days = tokens
                .get(i + 1)
                .and_then(|t| t.parse::<f64>().ok())
                .expect("token after 'up' should be uptime days");
            assert_eq!(
                tokens.get(i + 2),
                Some(&"days"),
                "expected 'days' after uptime value, got: {text}"
            );
            return days;
        }
    }
    panic!("did not find 'up <days> days' in buptime output: {text}");
}

#[test]
fn version_flag() {
    cmd().arg("--version").assert().success().stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_flag() {
    cmd().arg("--help").assert().success().stdout(predicate::str::contains("uptime"));
}

#[test]
fn output_contains_up() {
    cmd().assert().success().stdout(predicate::str::contains("up"));
}

#[test]
fn output_contains_days() {
    cmd().assert().success().stdout(predicate::str::contains("days"));
}

#[test]
fn output_contains_millidays() {
    cmd().assert().success().stdout(predicate::str::contains("millidays"));
}

#[test]
fn output_contains_boot() {
    cmd().assert().success().stdout(predicate::str::contains("boot"));
}

#[test]
fn uptime_is_positive() {
    let out = cmd().assert().success().get_output().stdout.clone();
    let days = parse_uptime_days(&out);
    assert!(days >= 0.0, "uptime should be >= 0, got {days}");
}

// ── output structure ─────────────────────────────────────────────────────────

#[test]
fn output_contains_open_paren() {
    // millidays section is parenthesised: "(23258.530 millidays)"
    cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("("));
}

#[test]
fn output_contains_em_dash() {
    // separator between uptime and boot: "—"
    cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("—"));
}

// ── boot BrightDate ──────────────────────────────────────────────────────────

#[test]
fn boot_bd_is_positive() {
    // boot BD should be positive — all machines booted after J2000
    let out = cmd().assert().success().get_output().stdout.clone();
    let text = String::from_utf8_lossy(&out);
    let mut found_boot = false;
    for (i, token) in text.split_whitespace().enumerate() {
        if token == "boot" {
            found_boot = true;
            // the next token after "boot" should be a positive number
            // collect all tokens, then take index i+1
            let tokens: Vec<&str> = text.split_whitespace().collect();
            if let Some(val_str) = tokens.get(i + 1) {
                let v: f64 = val_str.parse().expect("boot BD should be numeric");
                assert!(v > 0.0, "boot BD should be positive, got {v}");
            }
            return;
        }
    }
    assert!(found_boot, "did not find 'boot' in buptime output: {text}");
}

// ── millidays value ───────────────────────────────────────────────────────────

#[test]
fn millidays_value_is_positive() {
    let out = cmd().assert().success().get_output().stdout.clone();
    let text = String::from_utf8_lossy(&out);
    // format: "up X.XXXXX days (Y.YYY millidays)  —  boot Z"
    let tokens: Vec<&str> = text.split_whitespace().collect();
    for i in 1..tokens.len() {
        if tokens[i] == "millidays)" || tokens[i].starts_with("millidays") {
            let raw = tokens[i - 1].trim_start_matches('(');
            let v: f64 = raw.parse().expect("millidays value should be numeric");
            assert!(v >= 0.0, "millidays should be >= 0, got {v}");
            return;
        }
    }
    panic!("did not find millidays value in buptime output: {text}");
}

// ── sanity range ──────────────────────────────────────────────────────────────

#[test]
fn uptime_in_reasonable_range() {
    let out = cmd().assert().success().get_output().stdout.clone();
    let days = parse_uptime_days(&out);
    // Host uptime in BrightDate-days; boot BD is ~9600+ and must not be mistaken for this.
    assert!(days >= 0.0, "uptime must be non-negative, got {days}");
    assert!(
        days < 3650.0,
        "uptime > 10 years seems wrong (got {days} days; check you parsed uptime, not boot BD)"
    );
}
