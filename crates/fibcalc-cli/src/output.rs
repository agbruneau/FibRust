//! CLI output formatting.

use std::io::{self, Write};
use std::time::Duration;

use num_bigint::BigUint;

/// Format a `BigUint` for display, potentially truncating.
#[must_use]
pub fn format_result(value: &BigUint, verbose: bool) -> String {
    let s = value.to_string();
    if !verbose && s.len() > 100 {
        format!("{}...{} ({} digits)", &s[..50], &s[s.len() - 50..], s.len())
    } else {
        s
    }
}

/// Format a duration for display.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn format_duration(d: Duration) -> String {
    let secs = d.as_secs_f64();
    if secs < 0.001 {
        format!("{:.2}µs", secs * 1_000_000.0)
    } else if secs < 1.0 {
        format!("{:.2}ms", secs * 1000.0)
    } else if secs < 60.0 {
        format!("{secs:.3}s")
    } else {
        let mins = (secs / 60.0).floor() as u64;
        let remaining = secs - (mins as f64 * 60.0);
        format!("{mins}m{remaining:.1}s")
    }
}

/// Format a number with thousand separators.
#[must_use]
pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Write result to a file.
///
/// # Errors
///
/// Returns an I/O error if the file cannot be created or written.
pub fn write_to_file(path: &str, value: &BigUint) -> io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    write!(file, "{value}")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_micro() {
        let s = format_duration(Duration::from_nanos(500));
        assert!(s.contains("µs"));
    }

    #[test]
    fn format_duration_milli() {
        let s = format_duration(Duration::from_millis(42));
        assert!(s.contains("ms"));
    }

    #[test]
    fn format_duration_seconds() {
        let s = format_duration(Duration::from_secs_f64(3.14));
        assert!(s.contains("s"));
    }

    #[test]
    fn format_duration_minutes() {
        let s = format_duration(Duration::from_secs(90));
        assert!(s.contains("m"));
    }

    #[test]
    fn format_number_thousands() {
        assert_eq!(format_number(1_000_000), "1,000,000");
        assert_eq!(format_number(42), "42");
        assert_eq!(format_number(1234), "1,234");
    }

    #[test]
    fn format_result_short() {
        let value = BigUint::from(12345u64);
        let s = format_result(&value, false);
        assert_eq!(s, "12345");
    }
}
