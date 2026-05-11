use brightdate::BrightDate;
use clap::Command;

fn main() {
    Command::new("buptime")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Show system uptime in BrightDate units")
        .get_matches();

    // Read /proc/uptime on Linux, sysctl on macOS
    let uptime_secs = get_uptime_seconds();

    let days = uptime_secs / 86_400.0;
    let millidays = days * 1_000.0;

    let boot_bd = BrightDate::now().sub_days(days);

    println!(
        "up {:.5} days ({:.3} millidays)  —  boot {}",
        days, millidays, boot_bd
    );

    if let Some(users) = active_users() {
        let plural = if users == 1 { "user" } else { "users" };
        println!("{} {}", users, plural);
    }

    if let Some((m1, m5, m15)) = load_averages() {
        println!("load averages: {:.2} {:.2} {:.2}", m1, m5, m15);
    }
}

#[cfg(unix)]
fn load_averages() -> Option<(f64, f64, f64)> {
    let mut loads = [0.0f64; 3];
    let n = unsafe { libc::getloadavg(loads.as_mut_ptr(), 3) };
    if n == 3 {
        Some((loads[0], loads[1], loads[2]))
    } else {
        None
    }
}

#[cfg(not(unix))]
fn load_averages() -> Option<(f64, f64, f64)> {
    None
}

#[cfg(unix)]
fn active_users() -> Option<usize> {
    use std::process;
    // `who | wc -l` matches what the standard `uptime` reports.
    let out = process::Command::new("sh")
        .args(["-c", "who | wc -l"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    s.trim().parse::<usize>().ok()
}

#[cfg(not(unix))]
fn active_users() -> Option<usize> {
    None
}

fn get_uptime_seconds() -> f64 {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(content) = fs::read_to_string("/proc/uptime") {
            if let Some(part) = content.split_whitespace().next() {
                if let Ok(secs) = part.parse::<f64>() {
                    return secs;
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::process;
        if let Ok(output) = process::Command::new("sysctl")
            .args(["-n", "kern.boottime"])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            // Format: "{ sec = 1715000000, usec = 0 } Sat May 10 00:00:00 2026"
            if let Some(sec_part) = text.split("sec = ").nth(1) {
                if let Some(sec_str) = sec_part.split(',').next() {
                    if let Ok(boot_unix) = sec_str.trim().parse::<u64>() {
                        let now_unix =
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                        return (now_unix.saturating_sub(boot_unix)) as f64;
                    }
                }
            }
        }
    }

    0.0
}
