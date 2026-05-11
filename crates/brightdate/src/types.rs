//! Core type definitions for BrightDate.

use thiserror::Error;

/// A BrightDate value: decimal days since J2000.0.
pub type BrightDateValue = f64;

/// Display precision (decimal places, 1–12).
pub type Precision = u8;

/// Options when constructing a BrightDate.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct BrightDateOptions {
    /// Override default display precision (5).
    pub precision: Option<Precision>,
    /// If true, interpret/return value on TAI timescale.
    pub use_tai: Option<bool>,
}

/// Decomposed components of a BrightDate value.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BrightDateComponents {
    /// Integer day number since J2000.0.
    pub day: i64,
    /// Fractional part of the day `[0, 1)`.
    pub fraction: f64,
    /// Full decimal value.
    pub value: f64,
    /// Millidays within the day (0–999).
    pub millidays: u32,
    /// Microdays within the current milliday (0–999).
    pub microdays: u32,
    /// Nanodays within the current microday (0–999).
    pub nanodays: u32,
}

/// Duration expressed in BrightDate metric units.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BrightDuration {
    /// Total duration in decimal days.
    pub days: f64,
    /// Duration in millidays.
    pub millidays: f64,
    /// Duration in microdays.
    pub microdays: f64,
    /// Duration in nanodays.
    pub nanodays: f64,
}

/// Result of formatting a BrightDate for display.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FormattedBrightDate {
    /// Full formatted string, e.g. `"9622.50417"`.
    pub full: String,
    /// Day part only, e.g. `"9622"`.
    pub day: String,
    /// Fractional part only, e.g. `"50417"`.
    pub fraction: String,
    /// Human-friendly with metric units, e.g. `"Day 9622, 504 md"`.
    pub friendly: String,
}

/// Errors that can arise from BrightDate operations.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum BrightDateError {
    #[error("invalid or non-finite input: {0}")]
    InvalidInput(String),
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("invalid number: {0}")]
    InvalidNumber(String),
    #[error("invalid precision: {0}")]
    InvalidPrecision(String),
    #[error("out of range: {0}")]
    OutOfRange(String),
    #[error("invalid GPS week: {0}")]
    InvalidGpsWeek(String),
    #[error("invalid GPS seconds: {0}")]
    InvalidGpsSeconds(String),
}
