//! Application configuration from CLI flags and environment.

use clap::Parser;

/// FibCalc-rs — High-performance Fibonacci calculator.
#[derive(Parser, Debug)]
#[command(name = "fibcalc", version, about)]
#[allow(clippy::struct_excessive_bools)]
pub struct AppConfig {
    /// Fibonacci number to compute.
    #[arg(short, long, default_value = "100000000", env = "FIBCALC_N")]
    pub n: u64,

    /// Algorithm to use: fast, matrix, fft, or all.
    #[arg(long, default_value = "all")]
    pub algo: String,

    /// Calculate and display the result.
    #[arg(short, long)]
    pub calculate: bool,

    /// Verbose output.
    #[arg(short, long)]
    pub verbose: bool,

    /// Show detailed information.
    #[arg(short, long)]
    pub details: bool,

    /// Output file path.
    #[arg(short, long)]
    pub output: Option<String>,

    /// Quiet mode (only output the number).
    #[arg(short, long)]
    pub quiet: bool,

    /// Run full calibration.
    #[arg(long)]
    pub calibrate: bool,

    /// Run automatic calibration.
    #[arg(long)]
    pub auto_calibrate: bool,

    /// Timeout duration (e.g., "5m", "1h").
    #[arg(long, default_value = "5m")]
    pub timeout: String,

    /// Parallel multiplication threshold in bits.
    #[arg(long, default_value = "0")]
    pub threshold: usize,

    /// FFT multiplication threshold in bits.
    #[arg(long, default_value = "0")]
    pub fft_threshold: usize,

    /// Strassen multiplication threshold in bits.
    #[arg(long, default_value = "0")]
    pub strassen_threshold: usize,

    /// Launch interactive TUI.
    #[arg(long)]
    pub tui: bool,

    /// Generate shell completion.
    #[arg(long, value_enum)]
    pub completion: Option<clap_complete::Shell>,

    /// Compute only last K digits.
    #[arg(long, default_value = "0")]
    pub last_digits: u32,

    /// Memory limit (e.g., "8G", "512M").
    #[arg(long, default_value = "")]
    pub memory_limit: String,
}

impl AppConfig {
    /// Parse CLI arguments.
    #[must_use]
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }

    /// Parse timeout string into Duration.
    #[must_use]
    pub fn timeout_duration(&self) -> std::time::Duration {
        parse_duration(&self.timeout).unwrap_or(std::time::Duration::from_secs(300))
    }
}

/// Parse a duration string like "5m", "1h", "30s".
fn parse_duration(s: &str) -> Option<std::time::Duration> {
    let s = s.trim();
    if let Some(mins) = s.strip_suffix('m') {
        let n: u64 = mins.parse().ok()?;
        Some(std::time::Duration::from_secs(n * 60))
    } else if let Some(hours) = s.strip_suffix('h') {
        let n: u64 = hours.parse().ok()?;
        Some(std::time::Duration::from_secs(n * 3600))
    } else if let Some(ms) = s.strip_suffix("ms") {
        let n: u64 = ms.parse().ok()?;
        Some(std::time::Duration::from_millis(n))
    } else if let Some(secs) = s.strip_suffix('s') {
        let n: u64 = secs.parse().ok()?;
        Some(std::time::Duration::from_secs(n))
    } else {
        let n: u64 = s.parse().ok()?;
        Some(std::time::Duration::from_secs(n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_duration_formats() {
        assert_eq!(
            parse_duration("5m"),
            Some(std::time::Duration::from_secs(300))
        );
        assert_eq!(
            parse_duration("1h"),
            Some(std::time::Duration::from_secs(3600))
        );
        assert_eq!(
            parse_duration("30s"),
            Some(std::time::Duration::from_secs(30))
        );
    }

    #[test]
    fn parse_duration_ms() {
        assert_eq!(
            parse_duration("1ms"),
            Some(std::time::Duration::from_millis(1))
        );
        assert_eq!(
            parse_duration("500ms"),
            Some(std::time::Duration::from_millis(500))
        );
    }
}
