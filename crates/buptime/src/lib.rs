use brightdate::BrightDate;
use clap::Command;

#[cfg(target_os = "linux")]
fn get_uptime_seconds() -> f64 {
    std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split_whitespace().next().and_then(|v| v.parse::<f64>().ok()))
        .unwrap_or(0.0)
}

#[cfg(target_os = "macos")]
fn get_uptime_seconds() -> f64 {
    let out = std::process::Command::new("sysctl")
        .arg("-n")
        .arg("kern.boottime")
        .output()
        .ok();
    if let Some(out) = out {
        let s = String::from_utf8_lossy(&out.stdout);
        // format: { sec = 1234567890, usec = 0 } ...
        if let Some(sec_str) = s.split("sec = ").nth(1) {
            if let Some(val) = sec_str.split(',').next() {
                if let Ok(boot) = val.trim().parse::<i64>() {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);
                    return (now - boot).max(0) as f64;
                }
            }
        }
    }
    0.0
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_uptime_seconds() -> f64 {
    0.0
}

fn active_users() -> Option<usize> {
    let out = std::process::Command::new("who").output().ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    Some(s.lines().filter(|l| !l.trim().is_empty()).count())
}

#[cfg(unix)]
fn load_averages() -> Option<(f64, f64, f64)> {
    let mut avgs = [0.0f64; 3];
    let n = unsafe { libc::getloadavg(avgs.as_mut_ptr(), 3) };
    if n >= 3 {
        Some((avgs[0], avgs[1], avgs[2]))
    } else {
        None
    }
}

#[cfg(not(unix))]
fn load_averages() -> Option<(f64, f64, f64)> {
    None
}

pub fn run(args: &[String]) -> i32 {
    let cmd = Command::new("buptime")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Show system uptime in BrightDate format");

    let _matches = match cmd.try_get_matches_from(args) {
        Ok(m) => m,
        Err(e) => {
            let _ = e.print();
            return e.exit_code();
        }
    };

    let uptime_s = get_uptime_seconds();
    let uptime_days = uptime_s / 86_400.0;
    let millidays = (uptime_days * 1_000.0).floor() as u64;
    let whole_days = (uptime_days).floor() as u64;

    let boot_unix_ms = {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as f64)
            .unwrap_or(0.0);
        now - uptime_s * 1_000.0
    };

    let boot_bd = BrightDate::from_unix_ms(boot_unix_ms).ok();

    let mut parts = Vec::new();
    if whole_days > 0 {
        parts.push(format!("{} day{}", whole_days, if whole_days == 1 { "" } else { "s" }));
    }
    let hrs = ((uptime_s as u64 % 86400) / 3600) as u64;
    let mins = ((uptime_s as u64 % 3600) / 60) as u64;
    parts.push(format!("{:02}:{:02}", hrs, mins));

    print!("up {} ({} millidays)", parts.join(", "), millidays);
    if let Some(bd) = boot_bd {
        print!("  — boot {:.5}", bd.value);
    }
    println!();

    if let Some(users) = active_users() {
        print!("{} user{}", users, if users == 1 { "" } else { "s" });
    }

    if let Some((l1, l5, l15)) = load_averages() {
        println!("  load average: {:.2}, {:.2}, {:.2}", l1, l5, l15);
    } else {
        println!();
    }

    0
}
