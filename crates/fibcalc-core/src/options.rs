//! Calculation options and configuration.

use crate::constants::{
    DEFAULT_FFT_THRESHOLD, DEFAULT_PARALLEL_THRESHOLD, DEFAULT_STRASSEN_THRESHOLD,
};

/// Options for Fibonacci calculation.
///
/// # Example
/// ```
/// use fibcalc_core::options::Options;
///
/// let opts = Options::default();
/// assert_eq!(opts.parallel_threshold, 4096);
/// assert!(opts.last_digits.is_none());
///
/// // Normalize replaces zero thresholds with defaults
/// let custom = Options { parallel_threshold: 0, ..Options::default() };
/// let normalized = custom.normalize();
/// assert_eq!(normalized.parallel_threshold, 4096);
/// ```
#[derive(Debug, Clone)]
pub struct Options {
    /// Threshold (in bits) for parallel multiplication.
    pub parallel_threshold: usize,
    /// Threshold (in bits) for FFT multiplication.
    pub fft_threshold: usize,
    /// Threshold (in bits) for Strassen multiplication.
    pub strassen_threshold: usize,
    /// Number of last digits to compute (`None` = full number).
    pub last_digits: Option<u32>,
    /// Memory limit in bytes (`None` = unlimited).
    pub memory_limit: Option<usize>,
    /// Whether to show verbose output.
    pub verbose: bool,
    /// Whether to show detailed output.
    pub details: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            parallel_threshold: DEFAULT_PARALLEL_THRESHOLD,
            fft_threshold: DEFAULT_FFT_THRESHOLD,
            strassen_threshold: DEFAULT_STRASSEN_THRESHOLD,
            last_digits: None,
            memory_limit: None,
            verbose: false,
            details: false,
        }
    }
}

impl Options {
    /// Normalize options, applying defaults where values are zero.
    #[must_use]
    pub fn normalize(mut self) -> Self {
        if self.parallel_threshold == 0 {
            self.parallel_threshold = DEFAULT_PARALLEL_THRESHOLD;
        }
        if self.fft_threshold == 0 {
            self.fft_threshold = DEFAULT_FFT_THRESHOLD;
        }
        if self.strassen_threshold == 0 {
            self.strassen_threshold = DEFAULT_STRASSEN_THRESHOLD;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options() {
        let opts = Options::default();
        assert_eq!(opts.parallel_threshold, DEFAULT_PARALLEL_THRESHOLD);
        assert_eq!(opts.fft_threshold, DEFAULT_FFT_THRESHOLD);
        assert_eq!(opts.strassen_threshold, DEFAULT_STRASSEN_THRESHOLD);
        assert!(opts.last_digits.is_none());
    }

    #[test]
    fn normalize_zero_thresholds() {
        let opts = Options {
            parallel_threshold: 0,
            fft_threshold: 0,
            strassen_threshold: 0,
            ..Default::default()
        };
        let normalized = opts.normalize();
        assert_eq!(normalized.parallel_threshold, DEFAULT_PARALLEL_THRESHOLD);
        assert_eq!(normalized.fft_threshold, DEFAULT_FFT_THRESHOLD);
        assert_eq!(normalized.strassen_threshold, DEFAULT_STRASSEN_THRESHOLD);
    }
}
