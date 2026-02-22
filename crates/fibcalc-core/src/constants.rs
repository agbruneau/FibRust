//! Constants for Fibonacci calculation thresholds and configuration.

/// Default threshold (in bits) for parallel multiplication.
pub const DEFAULT_PARALLEL_THRESHOLD: usize = 4096;

/// Default threshold (in bits) for FFT multiplication.
pub const DEFAULT_FFT_THRESHOLD: usize = 500_000;

/// Default threshold (in bits) for Strassen multiplication.
pub const DEFAULT_STRASSEN_THRESHOLD: usize = 3072;

/// Threshold (in bits) for parallel FFT execution.
pub const PARALLEL_FFT_THRESHOLD: usize = 5_000_000;

/// Default N value for calibration benchmarks.
pub const CALIBRATION_N: u64 = 10_000_000;

/// Minimum progress change (1%) before reporting an update.
pub const PROGRESS_REPORT_THRESHOLD: f64 = 0.01;

/// Maximum Fibonacci index that fits in a u64.
/// F(93) = 12200160415121876738
pub const MAX_FIB_U64: u64 = 93;

/// Maximum bit length for pooled `BigInts` (100M bits).
pub const MAX_POOLED_BIT_LEN: usize = 100_000_000;

/// Precomputed Fibonacci values for n = 0..=93 (fast path).
///
/// F(93) = 12,200,160,415,121,876,738 is the largest Fibonacci number
/// that fits in `u64`. F(94) = 19,740,274,219,868,223,167 overflows
/// `u64::MAX` (18,446,744,073,709,551,615).
pub const FIB_TABLE: [u64; 94] = {
    let mut table = [0u64; 94];
    table[0] = 0;
    table[1] = 1;
    let mut i = 2;
    while i < 94 {
        table[i] = table[i - 1] + table[i - 2];
        i += 1;
    }
    table
};

/// Exit codes matching Go implementation.
pub mod exit_codes {
    /// Successful execution.
    pub const SUCCESS: i32 = 0;
    /// Generic error.
    pub const ERROR_GENERIC: i32 = 1;
    /// Computation timed out.
    pub const ERROR_TIMEOUT: i32 = 2;
    /// Algorithm results did not match during cross-validation.
    pub const ERROR_MISMATCH: i32 = 3;
    /// Invalid configuration.
    pub const ERROR_CONFIG: i32 = 4;
    /// Computation cancelled by user (Ctrl+C).
    pub const ERROR_CANCELED: i32 = 130;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fib_table_first_values() {
        assert_eq!(FIB_TABLE[0], 0);
        assert_eq!(FIB_TABLE[1], 1);
        assert_eq!(FIB_TABLE[2], 1);
        assert_eq!(FIB_TABLE[10], 55);
        assert_eq!(FIB_TABLE[20], 6765);
    }

    #[test]
    fn fib_table_last_value() {
        assert_eq!(FIB_TABLE[93], 12_200_160_415_121_876_738);
    }

    #[test]
    fn fib_table_consistency() {
        for i in 2..94 {
            assert_eq!(FIB_TABLE[i], FIB_TABLE[i - 1] + FIB_TABLE[i - 2]);
        }
    }
}
