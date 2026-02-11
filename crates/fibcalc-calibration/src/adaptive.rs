//! Adaptive threshold estimation.

use fibcalc_core::constants::{
    DEFAULT_FFT_THRESHOLD, DEFAULT_PARALLEL_THRESHOLD, DEFAULT_STRASSEN_THRESHOLD,
};

use crate::microbench;

/// Estimated thresholds from adaptive calibration.
#[derive(Debug, Clone)]
pub struct EstimatedThresholds {
    pub parallel_threshold: usize,
    pub fft_threshold: usize,
    pub strassen_threshold: usize,
}

impl Default for EstimatedThresholds {
    fn default() -> Self {
        Self {
            parallel_threshold: DEFAULT_PARALLEL_THRESHOLD,
            fft_threshold: DEFAULT_FFT_THRESHOLD,
            strassen_threshold: DEFAULT_STRASSEN_THRESHOLD,
        }
    }
}

/// Estimate optimal thresholds based on quick benchmarks.
/// Returns (`parallel_threshold`, `fft_threshold`).
#[must_use]
pub fn estimate_thresholds() -> (usize, usize) {
    let est = estimate_all_thresholds();
    (est.parallel_threshold, est.fft_threshold)
}

/// Estimate all thresholds (parallel, FFT, Strassen).
#[must_use]
pub fn estimate_all_thresholds() -> EstimatedThresholds {
    let fft_threshold = find_fft_crossover_threshold();
    let parallel_threshold = find_parallel_crossover_threshold();
    let strassen_threshold = find_strassen_threshold(fft_threshold);

    EstimatedThresholds {
        parallel_threshold,
        fft_threshold,
        strassen_threshold,
    }
}

/// Binary search for the FFT/Karatsuba crossover point.
/// Tests bit lengths between `low` and `high` to find where FFT becomes faster.
fn find_fft_crossover_threshold() -> usize {
    // Sample at exponentially spaced points first to find the rough region
    let sample_points: Vec<usize> = (10..=20).map(|exp| 1 << exp).collect(); // 1K to 1M
    let crossovers = microbench::find_fft_crossover(&sample_points);

    // Find first point where FFT is faster
    let first_fft_faster = crossovers.iter().find(|c| c.fft_is_faster);

    match first_fft_faster {
        Some(point) => {
            // Binary search around this region for precision
            let idx = crossovers
                .iter()
                .position(|c| c.bit_length == point.bit_length)
                .unwrap_or(0);
            let low = if idx > 0 {
                crossovers[idx - 1].bit_length
            } else {
                sample_points[0]
            };
            let high = point.bit_length;
            binary_search_crossover(low, high)
        }
        None => {
            // FFT never faster at tested sizes, use default
            DEFAULT_FFT_THRESHOLD
        }
    }
}

/// Binary search between `low` and `high` for the exact crossover point.
fn binary_search_crossover(mut low: usize, mut high: usize) -> usize {
    // Only do a few iterations to keep calibration fast
    for _ in 0..4 {
        if high - low < 1024 {
            break;
        }
        let mid = (low + high) / 2;
        let points = microbench::find_fft_crossover(&[mid]);
        if let Some(p) = points.first() {
            if p.fft_is_faster {
                high = mid;
            } else {
                low = mid;
            }
        }
    }
    (low + high) / 2
}

/// Find the threshold where parallel execution becomes beneficial.
fn find_parallel_crossover_threshold() -> usize {
    let test_sizes: Vec<usize> = vec![512, 1024, 2048, 4096, 8192, 16384];

    for &bits in &test_sizes {
        let overhead = microbench::measure_parallel_overhead(bits);
        if overhead.speedup > 1.1 {
            // Parallel is >10% faster, this is a good threshold
            return bits;
        }
    }
    DEFAULT_PARALLEL_THRESHOLD
}

/// Estimate the Strassen threshold relative to the FFT threshold.
fn find_strassen_threshold(fft_threshold: usize) -> usize {
    // Strassen is typically beneficial at ~60-80% of the FFT threshold
    let candidate = fft_threshold * 3 / 5;
    candidate.max(DEFAULT_STRASSEN_THRESHOLD).min(fft_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_returns_positive() {
        let (parallel, fft) = estimate_thresholds();
        assert!(parallel > 0);
        assert!(fft > 0);
    }

    #[test]
    fn estimate_all_returns_valid() {
        let est = estimate_all_thresholds();
        assert!(est.parallel_threshold > 0);
        assert!(est.fft_threshold > 0);
        assert!(est.strassen_threshold > 0);
        assert!(est.fft_threshold >= est.strassen_threshold);
    }

    #[test]
    fn strassen_relative_to_fft() {
        let strassen = find_strassen_threshold(500_000);
        assert!(strassen <= 500_000);
        assert!(strassen >= DEFAULT_STRASSEN_THRESHOLD);
    }

    #[test]
    fn binary_search_converges() {
        let result = binary_search_crossover(1024, 65536);
        assert!(result >= 1024);
        assert!(result <= 65536);
    }
}
