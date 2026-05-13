use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("buptime").expect("buptime binary not found")
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
    let text = String::from_utf8_lossy(&out);
    // "up X.XXXXX days" — extract X
    for token in text.split_whitespace() {
        if let Ok(days) = token.parse::<f64>() {
            assert!(days > 0.0, "uptime should be > 0, got {days}");
            return;
        }
    }
    panic!("no numeric value found in buptime output: {text}");
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
    // find the token just before "millidays"
    let tokens: Vec<&str> = text.split_whitespace().collect();
    for i in 1..tokens.len() {
        if tokens[i] == "millidays)" {
            // strip leading "("
            let raw = tokens[i - 1].trim_start_matches('(');
            let v: f64 = raw.parse().expect("millidays value should be numeric");
            assert!(v > 0.0, "millidays should be > 0, got {v}");
            return;
        }
    }
    panic!("did not find millidays value in buptime output: {text}");
}

// ── sanity range ──────────────────────────────────────────────────────────────

#[test]
fn uptime_in_reasonable_range() {
    let out = cmd().assert().success().get_output().stdout.clone();
    let text = String::from_utf8_lossy(&out);
    // first numeric token is uptime in days; must be < 3650 (10 years) and > 0
    for token in text.split_whitespace() {
        if let Ok(days) = token.parse::<f64>() {
            assert!(days > 0.0, "uptime must be positive, got {days}");
            assert!(days < 3650.0, "uptime > 10 years seems wrong, got {days}");
            return;
        }
    }
    panic!("no numeric value in buptime output: {text}");
}
