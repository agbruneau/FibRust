//! Memory budget estimation and validation.

/// Memory estimate for a Fibonacci computation.
#[derive(Debug, Clone)]
pub struct MemoryEstimate {
    /// Estimated memory for the result itself (in bytes).
    pub result_bytes: usize,
    /// Estimated memory for temporaries (in bytes).
    pub temp_bytes: usize,
    /// Total estimated memory (in bytes).
    pub total_bytes: usize,
}

impl MemoryEstimate {
    /// Bit threshold above which FFT multiplication is used inside
    /// `fibcalc-bigfft`. Kept in sync with `FFT_BIT_THRESHOLD` in `fft.rs`.
    const FFT_BIT_THRESHOLD: usize = 10_000;

    /// Estimate memory usage for computing F(n).
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    pub fn estimate(n: u64) -> Self {
        // F(n) has approximately n * log2(phi) / 8 bytes
        // phi = (1 + sqrt(5)) / 2, log2(phi) ≈ 0.6942
        let result_bits = (n as f64 * 0.6942).ceil() as usize;
        let result_bytes = result_bits.div_ceil(8);

        // Temporaries: ~5x the result for Fast Doubling (FK, FK1, T1, T2, T3)
        let mut temp_bytes = result_bytes * 5;

        // When the result is large enough, FFT multiplication is used and
        // requires additional working memory for polynomial transforms.
        if result_bits >= Self::FFT_BIT_THRESHOLD {
            temp_bytes += fibcalc_bigfft::estimate_fft_memory(result_bits, result_bits);
        }

        Self {
            result_bytes,
            temp_bytes,
            total_bytes: result_bytes + temp_bytes,
        }
    }

    /// Check if the computation fits within the given memory limit.
    ///
    /// `None` means unlimited (always fits).
    #[must_use]
    pub fn fits_in(&self, limit: Option<usize>) -> bool {
        match limit {
            None => true,
            Some(l) => self.total_bytes <= l,
        }
    }
}

/// Parse a memory limit string (e.g., "8G", "512M", "1024K").
///
/// # Errors
///
/// Returns an error string if the format is invalid or the number cannot be parsed.
pub fn parse_memory_limit(s: &str) -> Result<usize, String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(0);
    }

    let (num_str, multiplier) = if let Some(n) = s.strip_suffix('G') {
        (n, 1024 * 1024 * 1024)
    } else if let Some(n) = s.strip_suffix('M') {
        (n, 1024 * 1024)
    } else if let Some(n) = s.strip_suffix('K') {
        (n, 1024)
    } else if let Some(n) = s.strip_suffix('B') {
        (n, 1)
    } else {
        (s, 1)
    };

    let value: usize = num_str
        .trim()
        .parse()
        .map_err(|e| format!("invalid memory limit: {e}"))?;
    Ok(value * multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_small() {
        let est = MemoryEstimate::estimate(100);
        assert!(est.result_bytes > 0);
        assert!(est.total_bytes > est.result_bytes);
    }

    #[test]
    fn estimate_large() {
        let est = MemoryEstimate::estimate(10_000_000);
        // F(10M) ≈ 10M * 0.694 bits ≈ 868KB result
        assert!(est.result_bytes > 800_000);
        assert!(est.result_bytes < 1_000_000);
    }

    #[test]
    fn fits_in_unlimited() {
        let est = MemoryEstimate::estimate(100_000_000);
        assert!(est.fits_in(None)); // None = unlimited
    }

    #[test]
    fn parse_memory_limit_values() {
        assert_eq!(parse_memory_limit("8G").unwrap(), 8 * 1024 * 1024 * 1024);
        assert_eq!(parse_memory_limit("512M").unwrap(), 512 * 1024 * 1024);
        assert_eq!(parse_memory_limit("1024K").unwrap(), 1024 * 1024);
        assert_eq!(parse_memory_limit("").unwrap(), 0);
    }

    #[test]
    fn parse_memory_limit_invalid() {
        assert!(parse_memory_limit("abc").is_err());
    }

    #[test]
    fn estimate_includes_fft_for_large_n() {
        // n = 100_000 gives ~69_420 result bits, well above FFT_BIT_THRESHOLD.
        let large = MemoryEstimate::estimate(100_000);
        // Without FFT overhead, temp_bytes would be exactly 5x result_bytes.
        let base_temp = large.result_bytes * 5;
        assert!(
            large.temp_bytes > base_temp,
            "Large n should include FFT memory overhead: temp={} base={}",
            large.temp_bytes,
            base_temp
        );

        // n = 10 gives ~7 result bits, well below FFT_BIT_THRESHOLD.
        let small = MemoryEstimate::estimate(10);
        let small_base_temp = small.result_bytes * 5;
        assert_eq!(
            small.temp_bytes, small_base_temp,
            "Small n should NOT include FFT memory overhead"
        );
    }
}
