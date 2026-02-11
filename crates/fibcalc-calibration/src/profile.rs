//! Calibration profile (serializable).

use serde::{Deserialize, Serialize};

use fibcalc_core::constants::{
    DEFAULT_FFT_THRESHOLD, DEFAULT_PARALLEL_THRESHOLD, DEFAULT_STRASSEN_THRESHOLD,
};

/// Current profile format version.
pub const PROFILE_VERSION: u32 = 1;

/// Calibration profile containing optimized thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationProfile {
    /// Profile format version for compatibility checking.
    pub version: u32,
    /// Optimized parallel threshold.
    pub parallel_threshold: usize,
    /// Optimized FFT threshold.
    pub fft_threshold: usize,
    /// Optimized Strassen threshold.
    pub strassen_threshold: usize,
    /// CPU model used for calibration.
    pub cpu_model: String,
    /// Number of CPU cores.
    pub num_cores: usize,
    /// CPU fingerprint for invalidation.
    pub cpu_fingerprint: String,
    /// Calibration timestamp (ISO 8601).
    pub timestamp: String,
}

impl Default for CalibrationProfile {
    fn default() -> Self {
        Self {
            version: PROFILE_VERSION,
            parallel_threshold: DEFAULT_PARALLEL_THRESHOLD,
            fft_threshold: DEFAULT_FFT_THRESHOLD,
            strassen_threshold: DEFAULT_STRASSEN_THRESHOLD,
            cpu_model: String::new(),
            num_cores: num_cpus(),
            cpu_fingerprint: String::new(),
            timestamp: String::new(),
        }
    }
}

impl CalibrationProfile {
    /// Check if this profile is compatible with the current version.
    #[must_use]
    pub fn is_compatible(&self) -> bool {
        self.version == PROFILE_VERSION
    }

    /// Check if this profile matches the current CPU.
    #[must_use]
    pub fn matches_cpu(&self, current_fingerprint: &str) -> bool {
        if self.cpu_fingerprint.is_empty() || current_fingerprint.is_empty() {
            return true; // can't verify, assume compatible
        }
        self.cpu_fingerprint == current_fingerprint
    }

    /// Validate that thresholds are within reasonable bounds.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.parallel_threshold > 0
            && self.fft_threshold > 0
            && self.strassen_threshold > 0
            && self.fft_threshold >= self.strassen_threshold
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(4)
}

/// Get a CPU fingerprint based on model and core count.
#[must_use]
pub fn cpu_fingerprint() -> String {
    let cores = num_cpus();
    format!("cores={cores}")
}

/// Get the current CPU model string.
#[must_use]
pub fn cpu_model() -> String {
    use sysinfo::System;
    let sys = System::new_all();
    sys.cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_default()
}

/// Get the current ISO 8601 timestamp.
#[must_use]
pub fn current_timestamp() -> String {
    // Simple UTC timestamp without pulling in chrono
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("unix:{}", dur.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_serialization() {
        let profile = CalibrationProfile::default();
        let json = serde_json::to_string_pretty(&profile).unwrap();
        let deserialized: CalibrationProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.parallel_threshold, profile.parallel_threshold);
        assert_eq!(deserialized.version, PROFILE_VERSION);
    }

    #[test]
    fn profile_compatibility() {
        let profile = CalibrationProfile::default();
        assert!(profile.is_compatible());

        let mut old = CalibrationProfile::default();
        old.version = 0;
        assert!(!old.is_compatible());
    }

    #[test]
    fn profile_cpu_match() {
        let mut profile = CalibrationProfile::default();
        profile.cpu_fingerprint = "cores=8".to_string();
        assert!(profile.matches_cpu("cores=8"));
        assert!(!profile.matches_cpu("cores=4"));
        // Empty fingerprint should match anything
        profile.cpu_fingerprint = String::new();
        assert!(profile.matches_cpu("cores=8"));
    }

    #[test]
    fn profile_validation() {
        let profile = CalibrationProfile::default();
        assert!(profile.is_valid());

        let mut bad = CalibrationProfile::default();
        bad.parallel_threshold = 0;
        assert!(!bad.is_valid());
    }

    #[test]
    fn cpu_fingerprint_nonempty() {
        let fp = cpu_fingerprint();
        assert!(!fp.is_empty());
        assert!(fp.starts_with("cores="));
    }

    #[test]
    fn current_timestamp_nonempty() {
        let ts = current_timestamp();
        assert!(!ts.is_empty());
        assert!(ts.starts_with("unix:"));
    }
}
