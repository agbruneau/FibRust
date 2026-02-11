//! Types for dynamic threshold management.

use serde::{Deserialize, Serialize};

/// Which multiplication method was used for an iteration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultiplicationMethod {
    /// Default (Karatsuba via num-bigint).
    Karatsuba,
    /// FFT-based multiplication.
    Fft,
    /// Strassen multiplication.
    Strassen,
}

/// Metric collected for each multiplication iteration.
#[derive(Debug, Clone)]
pub struct IterationMetric {
    /// Bit length of operands in this iteration.
    pub bit_length: usize,
    /// Speedup factor from using FFT (positive = FFT was faster).
    pub fft_speedup: f64,
    /// Speedup factor from using parallel execution.
    pub parallel_speedup: f64,
    /// Duration of the iteration in nanoseconds.
    pub duration_ns: u64,
    /// Which multiplication method was actually used.
    pub method: MultiplicationMethod,
    /// Number of bits processed in this iteration.
    pub bits_processed: u64,
    /// Whether a cache hit occurred for this iteration.
    pub cache_hit: bool,
}

impl IterationMetric {
    /// Create a basic metric with minimal fields (method defaults to Karatsuba).
    #[must_use]
    pub fn basic(
        bit_length: usize,
        fft_speedup: f64,
        parallel_speedup: f64,
        duration_ns: u64,
    ) -> Self {
        Self {
            bit_length,
            fft_speedup,
            parallel_speedup,
            duration_ns,
            method: MultiplicationMethod::Karatsuba,
            bits_processed: bit_length as u64,
            cache_hit: false,
        }
    }
}

/// Aggregated statistics for threshold adjustment.
#[derive(Debug, Clone)]
pub struct ThresholdStats {
    /// Average FFT benefit (positive = FFT is better).
    pub fft_benefit: f64,
    /// Average parallel benefit.
    pub parallel_benefit: f64,
    /// Average Strassen benefit.
    pub strassen_benefit: f64,
    /// Number of samples.
    pub sample_count: usize,
}

/// Serializable snapshot of the current threshold state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdSnapshot {
    /// Current parallel threshold.
    pub parallel_threshold: usize,
    /// Current FFT threshold.
    pub fft_threshold: usize,
    /// Current Strassen threshold.
    pub strassen_threshold: usize,
    /// Number of adjustments made.
    pub adjustment_count: usize,
    /// History of recent adjustments (threshold name, old value, new value).
    pub adjustment_history: Vec<ThresholdAdjustment>,
}

/// Record of a single threshold adjustment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdAdjustment {
    /// Which threshold was adjusted.
    pub threshold_name: String,
    /// Old value.
    pub old_value: usize,
    /// New value.
    pub new_value: usize,
    /// Benefit metric that triggered the adjustment.
    pub trigger_benefit: f64,
}

/// Configuration for the `DynamicThresholdManager`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicThresholdConfig {
    /// Size of the ring buffer for metrics.
    pub ring_buffer_size: usize,
    /// Hysteresis factor: minimum benefit to trigger adjustment.
    pub hysteresis_factor: f64,
    /// Maximum adjustment per cycle (as a fraction).
    pub max_adjustment: f64,
    /// Dead zone: benefit values within this range cause no adjustment.
    pub dead_zone: f64,
}

impl Default for DynamicThresholdConfig {
    fn default() -> Self {
        Self {
            ring_buffer_size: 32,
            hysteresis_factor: 0.05,
            max_adjustment: 0.1,
            dead_zone: 0.02,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = DynamicThresholdConfig::default();
        assert_eq!(config.ring_buffer_size, 32);
        assert!((config.hysteresis_factor - 0.05).abs() < f64::EPSILON);
        assert!((config.dead_zone - 0.02).abs() < f64::EPSILON);
    }

    #[test]
    fn config_serialization() {
        let config = DynamicThresholdConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: DynamicThresholdConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.ring_buffer_size, config.ring_buffer_size);
        assert!((deserialized.dead_zone - config.dead_zone).abs() < f64::EPSILON);
    }

    #[test]
    fn basic_metric() {
        let m = IterationMetric::basic(1000, 0.1, 0.05, 500_000);
        assert_eq!(m.bit_length, 1000);
        assert_eq!(m.method, MultiplicationMethod::Karatsuba);
        assert_eq!(m.bits_processed, 1000);
        assert!(!m.cache_hit);
    }

    #[test]
    fn snapshot_serialization() {
        let snap = ThresholdSnapshot {
            parallel_threshold: 4096,
            fft_threshold: 500_000,
            strassen_threshold: 3072,
            adjustment_count: 2,
            adjustment_history: vec![ThresholdAdjustment {
                threshold_name: "fft".to_string(),
                old_value: 500_000,
                new_value: 450_000,
                trigger_benefit: 0.12,
            }],
        };
        let json = serde_json::to_string_pretty(&snap).unwrap();
        let deserialized: ThresholdSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.fft_threshold, 500_000);
        assert_eq!(deserialized.adjustment_count, 2);
        assert_eq!(deserialized.adjustment_history.len(), 1);
        assert_eq!(deserialized.adjustment_history[0].new_value, 450_000);
    }

    #[test]
    fn multiplication_method_serialization() {
        let methods = [
            MultiplicationMethod::Karatsuba,
            MultiplicationMethod::Fft,
            MultiplicationMethod::Strassen,
        ];
        for method in &methods {
            let json = serde_json::to_string(method).unwrap();
            let deserialized: MultiplicationMethod = serde_json::from_str(&json).unwrap();
            assert_eq!(*method, deserialized);
        }
    }
}
