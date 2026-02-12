//! Calibration engine.

use crate::adaptive::EstimatedThresholds;
use crate::microbench;
use crate::profile::{self, CalibrationProfile};

/// Mode of calibration.
#[derive(Debug, Clone, Copy)]
pub enum CalibrationMode {
    /// Full calibration: run all benchmarks.
    Full,
    /// Auto calibration: quick adaptive benchmarks.
    Auto,
    /// Cached: load from profile file.
    Cached,
}

/// Progress callback for calibration.
pub type ProgressCallback = Box<dyn Fn(CalibrationProgress) + Send>;

/// Progress information during calibration.
#[derive(Debug, Clone)]
pub struct CalibrationProgress {
    /// Current step name.
    pub step: String,
    /// Current step number (1-based).
    pub current: usize,
    /// Total number of steps.
    pub total: usize,
}

/// Calibration engine that determines optimal thresholds.
pub struct CalibrationEngine {
    mode: CalibrationMode,
    progress_cb: Option<ProgressCallback>,
}

impl CalibrationEngine {
    /// Create a new calibration engine.
    #[must_use]
    pub fn new(mode: CalibrationMode) -> Self {
        Self {
            mode,
            progress_cb: None,
        }
    }

    /// Set a progress callback.
    #[must_use]
    pub fn with_progress(mut self, cb: ProgressCallback) -> Self {
        self.progress_cb = Some(cb);
        self
    }

    /// Run calibration and produce a profile.
    #[must_use]
    pub fn calibrate(&self) -> CalibrationProfile {
        match self.mode {
            CalibrationMode::Full => self.full_calibration(),
            CalibrationMode::Auto => self.auto_calibration(),
            CalibrationMode::Cached => self.load_cached(),
        }
    }

    fn report_progress(&self, step: &str, current: usize, total: usize) {
        if let Some(cb) = &self.progress_cb {
            cb(CalibrationProgress {
                step: step.to_string(),
                current,
                total,
            });
        }
    }

    fn full_calibration(&self) -> CalibrationProfile {
        let total_steps = 4;

        // Step 1: Benchmark multiplication at various bit lengths
        self.report_progress("Benchmarking Karatsuba multiplication", 1, total_steps);
        let karatsuba_points: Vec<usize> = (10..=18).map(|exp| 1 << exp).collect();
        let crossovers = microbench::find_fft_crossover(&karatsuba_points);

        // Step 2: Find FFT crossover
        self.report_progress("Finding FFT crossover point", 2, total_steps);
        let fft_threshold = crossovers
            .iter()
            .find(|c| c.fft_is_faster)
            .map_or(fibcalc_core::constants::DEFAULT_FFT_THRESHOLD, |c| {
                c.bit_length
            });

        // Step 3: Measure parallel overhead
        self.report_progress("Measuring parallel overhead", 3, total_steps);
        let parallel_test_sizes = [1024, 2048, 4096, 8192, 16384, 32768];
        let mut parallel_threshold = fibcalc_core::constants::DEFAULT_PARALLEL_THRESHOLD;
        for &bits in &parallel_test_sizes {
            let overhead = microbench::measure_parallel_overhead(bits);
            if overhead.speedup > 1.1 {
                parallel_threshold = bits;
                break;
            }
        }

        // Step 4: Compute Strassen threshold
        self.report_progress("Computing Strassen threshold", 4, total_steps);
        let strassen_threshold = (fft_threshold * 3 / 5)
            .max(fibcalc_core::constants::DEFAULT_STRASSEN_THRESHOLD)
            .min(fft_threshold);

        let cpu = profile::cpu_model();
        let fingerprint = profile::cpu_fingerprint();
        let timestamp = profile::current_timestamp();

        CalibrationProfile {
            version: profile::PROFILE_VERSION,
            parallel_threshold,
            fft_threshold,
            strassen_threshold,
            cpu_model: cpu,
            num_cores: std::thread::available_parallelism()
                .map(std::num::NonZero::get)
                .unwrap_or(4),
            cpu_fingerprint: fingerprint,
            timestamp,
        }
    }

    fn auto_calibration(&self) -> CalibrationProfile {
        let total_steps = 2;

        // Step 1: Quick adaptive estimation
        self.report_progress("Running adaptive estimation", 1, total_steps);
        let est = crate::adaptive::estimate_all_thresholds();

        // Step 2: Build profile
        self.report_progress("Building profile", 2, total_steps);
        self.build_profile_from_estimate(&est)
    }

    #[allow(clippy::unused_self)]
    fn build_profile_from_estimate(&self, est: &EstimatedThresholds) -> CalibrationProfile {
        let cpu = profile::cpu_model();
        let fingerprint = profile::cpu_fingerprint();
        let timestamp = profile::current_timestamp();

        CalibrationProfile {
            version: profile::PROFILE_VERSION,
            parallel_threshold: est.parallel_threshold,
            fft_threshold: est.fft_threshold,
            strassen_threshold: est.strassen_threshold,
            cpu_model: cpu,
            num_cores: std::thread::available_parallelism()
                .map(std::num::NonZero::get)
                .unwrap_or(4),
            cpu_fingerprint: fingerprint,
            timestamp,
        }
    }

    #[allow(clippy::unused_self)]
    fn load_cached(&self) -> CalibrationProfile {
        // Try to load from file, validate, fallback to defaults
        match crate::io::load_profile() {
            Some(p) if p.is_compatible() && p.is_valid() => {
                let current_fp = profile::cpu_fingerprint();
                if p.matches_cpu(&current_fp) {
                    p
                } else {
                    tracing::warn!("Cached profile CPU mismatch, using defaults");
                    CalibrationProfile::default()
                }
            }
            Some(_) => {
                tracing::warn!("Cached profile incompatible or invalid, using defaults");
                CalibrationProfile::default()
            }
            None => CalibrationProfile::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibration_modes() {
        let engine = CalibrationEngine::new(CalibrationMode::Auto);
        let profile = engine.calibrate();
        assert!(profile.parallel_threshold > 0);
        assert!(profile.fft_threshold > 0);
        assert!(profile.strassen_threshold > 0);
        assert!(profile.is_valid());
    }

    #[test]
    fn cached_mode_returns_valid() {
        let engine = CalibrationEngine::new(CalibrationMode::Cached);
        let profile = engine.calibrate();
        assert!(profile.parallel_threshold > 0);
        assert!(profile.is_valid());
    }

    #[test]
    fn full_calibration_returns_valid() {
        let engine = CalibrationEngine::new(CalibrationMode::Full);
        let profile = engine.calibrate();
        assert!(profile.parallel_threshold > 0);
        assert!(profile.fft_threshold > 0);
        assert!(profile.strassen_threshold > 0);
        assert!(profile.is_valid());
        assert!(!profile.cpu_fingerprint.is_empty());
        assert!(!profile.timestamp.is_empty());
    }

    #[test]
    fn progress_callback() {
        use std::sync::{Arc, Mutex};

        let steps = Arc::new(Mutex::new(Vec::new()));
        let steps_clone = Arc::clone(&steps);

        let engine = CalibrationEngine::new(CalibrationMode::Auto).with_progress(Box::new(
            move |progress| {
                steps_clone.lock().unwrap().push(progress.step.clone());
            },
        ));

        let _profile = engine.calibrate();

        let recorded = steps.lock().unwrap();
        assert!(!recorded.is_empty());
        assert!(recorded.iter().any(|s| s.contains("adaptive")));
    }
}
