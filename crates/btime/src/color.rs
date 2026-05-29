use std::io::{self, IsTerminal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorWhen {
    Auto,
    Always,
    Never,
    Ansi,
    TrueColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorScheme {
    #[default]
    Default,
    Bright,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderMode {
    Off,
    Ansi,
    TrueColor,
}

#[derive(Debug, Clone, Copy)]
pub struct Colors {
    pub reset: &'static str,
    pub real: &'static str,
    pub detail: &'static str,
    pub user: &'static str,
    pub sys: &'static str,
    pub cpu: &'static str,
    pub cpu_low: &'static str,
    pub cpu_mid: &'static str,
    pub cpu_high: &'static str,
    pub stamp: &'static str,
    pub value: &'static str,
    pub unit: &'static str,
}

impl Colors {
    pub const OFF: Self = Self {
        reset: "",
        real: "",
        detail: "",
        user: "",
        sys: "",
        cpu: "",
        cpu_low: "",
        cpu_mid: "",
        cpu_high: "",
        stamp: "",
        value: "",
        unit: "",
    };

    pub fn enabled(&self) -> bool {
        !self.reset.is_empty()
    }

    pub fn label(&self, name: &str, style: &str) -> String {
        if self.enabled() {
            format!("{}{:<8}{}", style, name, self.reset)
        } else {
            format!("{:<8}", name)
        }
    }

    pub fn cpu_pct_style(&self, pct: f64) -> &str {
        if !self.enabled() {
            return "";
        }
        if pct >= 90.0 {
            self.cpu_high
        } else if pct >= 50.0 {
            self.cpu_mid
        } else {
            self.cpu_low
        }
    }

    pub fn resolve(when: ColorWhen, scheme: ColorScheme) -> Self {
        if when == ColorWhen::Never {
            return Self::OFF;
        }

        if when == ColorWhen::Auto && no_color_requested() {
            return Self::OFF;
        }

        let mode = match when {
            ColorWhen::Never => RenderMode::Off,
            ColorWhen::Ansi => RenderMode::Ansi,
            ColorWhen::TrueColor => RenderMode::TrueColor,
            ColorWhen::Always => preferred_render_mode(),
            ColorWhen::Auto => {
                if io::stderr().is_terminal() || force_color_enabled() {
                    preferred_render_mode()
                } else {
                    RenderMode::Off
                }
            }
        };

        match mode {
            RenderMode::Off => Self::OFF,
            RenderMode::Ansi => Self::ansi(scheme),
            RenderMode::TrueColor => Self::truecolor(scheme),
        }
    }

    fn ansi(scheme: ColorScheme) -> Self {
        match scheme {
            ColorScheme::Default => Self {
                reset: "\x1b[0m",
                real: "\x1b[1;36m",
                detail: "\x1b[2m",
                user: "\x1b[32m",
                sys: "\x1b[33m",
                cpu: "\x1b[35m",
                cpu_low: "\x1b[32m",
                cpu_mid: "\x1b[33m",
                cpu_high: "\x1b[31m",
                stamp: "\x1b[34m",
                value: "\x1b[1m",
                unit: "\x1b[2m",
            },
            ColorScheme::Bright => Self {
                reset: "\x1b[0m",
                real: "\x1b[1;96m",
                detail: "\x1b[2m",
                user: "\x1b[92m",
                sys: "\x1b[93m",
                cpu: "\x1b[95m",
                cpu_low: "\x1b[92m",
                cpu_mid: "\x1b[93m",
                cpu_high: "\x1b[91m",
                stamp: "\x1b[94m",
                value: "\x1b[1;97m",
                unit: "\x1b[2m",
            },
        }
    }

    fn truecolor(scheme: ColorScheme) -> Self {
        match scheme {
            ColorScheme::Default => Self {
                reset: "\x1b[0m",
                real: "\x1b[1;38;2;80;200;220m",
                detail: "\x1b[2;38;2;120;120;120m",
                user: "\x1b[38;2;120;220;140m",
                sys: "\x1b[38;2;240;200;80m",
                cpu: "\x1b[38;2;200;120;220m",
                cpu_low: "\x1b[38;2;100;200;100m",
                cpu_mid: "\x1b[38;2;240;200;80m",
                cpu_high: "\x1b[38;2;240;100;100m",
                stamp: "\x1b[38;2;120;160;240m",
                value: "\x1b[1;38;2;240;240;240m",
                unit: "\x1b[2;38;2;128;128;128m",
            },
            ColorScheme::Bright => Self {
                reset: "\x1b[0m",
                real: "\x1b[1;38;2;0;220;255m",
                detail: "\x1b[2;38;2;160;160;160m",
                user: "\x1b[38;2;80;255;120m",
                sys: "\x1b[38;2;255;220;80m",
                cpu: "\x1b[38;2;255;120;255m",
                cpu_low: "\x1b[38;2;80;255;120m",
                cpu_mid: "\x1b[38;2;255;220;80m",
                cpu_high: "\x1b[38;2;255;80;80m",
                stamp: "\x1b[38;2;120;180;255m",
                value: "\x1b[1;38;2;255;255;255m",
                unit: "\x1b[2;38;2;160;160;160m",
            },
        }
    }
}

pub fn parse_color_when(value: &str) -> Result<ColorWhen, String> {
    match value.to_ascii_lowercase().as_str() {
        "auto" => Ok(ColorWhen::Auto),
        "always" | "on" | "yes" => Ok(ColorWhen::Always),
        "never" | "off" | "no" => Ok(ColorWhen::Never),
        "ansi" | "16" => Ok(ColorWhen::Ansi),
        "truecolor" | "24bit" | "rgb" => Ok(ColorWhen::TrueColor),
        other => Err(format!(
            "invalid color mode '{other}' (expected auto, always, never, ansi, or truecolor)"
        )),
    }
}

pub fn parse_color_scheme(value: &str) -> Result<ColorScheme, String> {
    match value.to_ascii_lowercase().as_str() {
        "default" => Ok(ColorScheme::Default),
        "bright" => Ok(ColorScheme::Bright),
        other => Err(format!("invalid color scheme '{other}' (expected default or bright)")),
    }
}

fn no_color_requested() -> bool {
    std::env::var_os("NO_COLOR").is_some()
        || std::env::var("CLICOLOR")
            .map(|v| matches!(v.as_str(), "0" | "false"))
            .unwrap_or(false)
}

fn force_color_enabled() -> bool {
    std::env::var("CLICOLOR_FORCE")
        .map(|v| !matches!(v.as_str(), "" | "0" | "false"))
        .unwrap_or(false)
}

fn preferred_render_mode() -> RenderMode {
    if term_supports_truecolor() {
        RenderMode::TrueColor
    } else {
        RenderMode::Ansi
    }
}

fn term_supports_truecolor() -> bool {
    std::env::var("COLORTERM")
        .map(|v| {
            let v = v.to_ascii_lowercase();
            v == "truecolor" || v == "24bit"
        })
        .unwrap_or(false)
        || std::env::var("TERM")
            .map(|v| v.contains("truecolor"))
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_color_when_values() {
        assert_eq!(parse_color_when("auto").unwrap(), ColorWhen::Auto);
        assert_eq!(parse_color_when("ANSI").unwrap(), ColorWhen::Ansi);
        assert_eq!(parse_color_when("24bit").unwrap(), ColorWhen::TrueColor);
        assert!(parse_color_when("nope").is_err());
    }

    #[test]
    fn off_palette_has_no_escapes() {
        let c = Colors::resolve(ColorWhen::Never, ColorScheme::Default);
        assert!(!c.enabled());
        assert_eq!(c.label("real", c.real), "real    ");
    }

    #[test]
    fn explicit_ansi_overrides_no_color_env() {
        let _guard = EnvGuard::set("NO_COLOR", "1");
        let c = Colors::resolve(ColorWhen::Ansi, ColorScheme::Default);
        assert!(c.enabled());
    }

    struct EnvGuard {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var_os(key);
            unsafe { std::env::set_var(key, value) };
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => unsafe { std::env::set_var(self.key, value) },
                None => unsafe { std::env::remove_var(self.key) },
            }
        }
    }
}
