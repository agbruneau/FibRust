//! Dynamic threshold management with ring buffer and hysteresis.

use crate::constants::{
    DEFAULT_FFT_THRESHOLD, DEFAULT_PARALLEL_THRESHOLD, DEFAULT_STRASSEN_THRESHOLD,
};
use crate::threshold_types::{
    DynamicThresholdConfig, IterationMetric, ThresholdAdjustment, ThresholdSnapshot, ThresholdStats,
};

/// Manager for dynamically adjusting multiplication thresholds.
pub struct DynamicThresholdManager {
    config: DynamicThresholdConfig,
    metrics: Vec<IterationMetric>,
    ring_pos: usize,
    ring_full: bool,
    current_parallel: usize,
    current_fft: usize,
    current_strassen: usize,
    adjustment_count: usize,
    adjustment_history: Vec<ThresholdAdjustment>,
}

impl DynamicThresholdManager {
    /// Create a new threshold manager with the given configuration.
    #[must_use]
    pub fn new(config: DynamicThresholdConfig) -> Self {
        let ring_size = config.ring_buffer_size;
        Self {
            config,
            metrics: Vec::with_capacity(ring_size),
            ring_pos: 0,
            ring_full: false,
            current_parallel: DEFAULT_PARALLEL_THRESHOLD,
            current_fft: DEFAULT_FFT_THRESHOLD,
            current_strassen: DEFAULT_STRASSEN_THRESHOLD,
            adjustment_count: 0,
            adjustment_history: Vec::new(),
        }
    }

    /// Record a metric from an iteration.
    pub fn record(&mut self, metric: IterationMetric) {
        let ring_size = self.config.ring_buffer_size;
        if self.metrics.len() < ring_size {
            self.metrics.push(metric);
        } else {
            self.metrics[self.ring_pos] = metric;
            self.ring_full = true;
        }
        self.ring_pos = (self.ring_pos + 1) % ring_size;
    }

    /// Get the number of recorded metrics.
    #[must_use]
    pub fn metric_count(&self) -> usize {
        self.metrics.len()
    }

    /// Adjust thresholds based on collected metrics.
    pub fn adjust(&mut self) {
        if self.metrics.is_empty() {
            return;
        }

        let stats = self.compute_stats();
        let dead_zone = self.config.dead_zone;
        let hysteresis = self.config.hysteresis_factor;
        let max_adj = self.config.max_adjustment;

        // FFT threshold adjustment
        if stats.fft_benefit > hysteresis && stats.fft_benefit.abs() > dead_zone {
            let old = self.current_fft;
            let factor = (1.0 - max_adj).max(0.5);
            self.current_fft = ((self.current_fft as f64) * factor) as usize;
            self.current_fft = self.current_fft.max(1024);
            self.record_adjustment("fft", old, self.current_fft, stats.fft_benefit);
        } else if stats.fft_benefit < -hysteresis && stats.fft_benefit.abs() > dead_zone {
            let old = self.current_fft;
            let factor = (1.0 + max_adj).min(2.0);
            self.current_fft = ((self.current_fft as f64) * factor) as usize;
            self.record_adjustment("fft", old, self.current_fft, stats.fft_benefit);
        }

        // Parallel threshold adjustment
        if stats.parallel_benefit > hysteresis && stats.parallel_benefit.abs() > dead_zone {
            let old = self.current_parallel;
            let factor = (1.0 - max_adj).max(0.5);
            self.current_parallel = ((self.current_parallel as f64) * factor) as usize;
            self.current_parallel = self.current_parallel.max(512);
            self.record_adjustment(
                "parallel",
                old,
                self.current_parallel,
                stats.parallel_benefit,
            );
        } else if stats.parallel_benefit < -hysteresis && stats.parallel_benefit.abs() > dead_zone {
            let old = self.current_parallel;
            let factor = (1.0 + max_adj).min(2.0);
            self.current_parallel = ((self.current_parallel as f64) * factor) as usize;
            self.record_adjustment(
                "parallel",
                old,
                self.current_parallel,
                stats.parallel_benefit,
            );
        }

        // Strassen threshold adjustment
        if stats.strassen_benefit > hysteresis && stats.strassen_benefit.abs() > dead_zone {
            let old = self.current_strassen;
            let factor = (1.0 - max_adj).max(0.5);
            self.current_strassen = ((self.current_strassen as f64) * factor) as usize;
            self.current_strassen = self.current_strassen.max(512);
            self.record_adjustment(
                "strassen",
                old,
                self.current_strassen,
                stats.strassen_benefit,
            );
        } else if stats.strassen_benefit < -hysteresis && stats.strassen_benefit.abs() > dead_zone {
            let old = self.current_strassen;
            let factor = (1.0 + max_adj).min(2.0);
            self.current_strassen = ((self.current_strassen as f64) * factor) as usize;
            self.record_adjustment(
                "strassen",
                old,
                self.current_strassen,
                stats.strassen_benefit,
            );
        }
    }

    fn record_adjustment(&mut self, name: &str, old: usize, new: usize, benefit: f64) {
        self.adjustment_count += 1;
        self.adjustment_history.push(ThresholdAdjustment {
            threshold_name: name.to_string(),
            old_value: old,
            new_value: new,
            trigger_benefit: benefit,
        });
        // Keep only the last 64 adjustments
        if self.adjustment_history.len() > 64 {
            self.adjustment_history.remove(0);
        }
    }

    fn compute_stats(&self) -> ThresholdStats {
        let n = self.metrics.len() as f64;
        let avg_fft_benefit = self.metrics.iter().map(|m| m.fft_speedup).sum::<f64>() / n;
        let avg_parallel_benefit = self.metrics.iter().map(|m| m.parallel_speedup).sum::<f64>() / n;

        // Compute Strassen benefit from metrics at Strassen-relevant bit range.
        let strassen_metrics: Vec<&IterationMetric> = self
            .metrics
            .iter()
            .filter(|m| m.bit_length >= 1024 && m.bit_length < self.current_fft)
            .collect();
        let avg_strassen_benefit = if strassen_metrics.is_empty() {
            0.0
        } else {
            strassen_metrics.iter().map(|m| m.fft_speedup).sum::<f64>()
                / strassen_metrics.len() as f64
        };

        ThresholdStats {
            fft_benefit: avg_fft_benefit,
            parallel_benefit: avg_parallel_benefit,
            strassen_benefit: avg_strassen_benefit,
            sample_count: self.metrics.len(),
        }
    }

    /// Get current parallel threshold.
    #[must_use]
    pub fn parallel_threshold(&self) -> usize {
        self.current_parallel
    }

    /// Get current FFT threshold.
    #[must_use]
    pub fn fft_threshold(&self) -> usize {
        self.current_fft
    }

    /// Get current Strassen threshold.
    #[must_use]
    pub fn strassen_threshold(&self) -> usize {
        self.current_strassen
    }

    /// Get a serializable snapshot of current thresholds and history.
    #[must_use]
    pub fn snapshot(&self) -> ThresholdSnapshot {
        ThresholdSnapshot {
            parallel_threshold: self.current_parallel,
            fft_threshold: self.current_fft,
            strassen_threshold: self.current_strassen,
            adjustment_count: self.adjustment_count,
            adjustment_history: self.adjustment_history.clone(),
        }
    }

    /// Get computed statistics from the current metrics buffer.
    #[must_use]
    pub fn stats(&self) -> Option<ThresholdStats> {
        if self.metrics.is_empty() {
            return None;
        }
        Some(self.compute_stats())
    }

    /// Reset the manager to default thresholds and clear all metrics.
    pub fn reset(&mut self) {
        self.metrics.clear();
        self.ring_pos = 0;
        self.ring_full = false;
        self.current_parallel = DEFAULT_PARALLEL_THRESHOLD;
        self.current_fft = DEFAULT_FFT_THRESHOLD;
        self.current_strassen = DEFAULT_STRASSEN_THRESHOLD;
        self.adjustment_count = 0;
        self.adjustment_history.clear();
    }

    /// Whether the ring buffer is full (has wrapped around at least once).
    #[must_use]
    pub fn is_ring_full(&self) -> bool {
        self.ring_full
    }

    /// Number of threshold adjustments made so far.
    #[must_use]
    pub fn adjustment_count(&self) -> usize {
        self.adjustment_count
    }

    /// Set thresholds directly (e.g., from a loaded calibration profile).
    pub fn set_thresholds(&mut self, parallel: usize, fft: usize, strassen: usize) {
        self.current_parallel = parallel;
        self.current_fft = fft;
        self.current_strassen = strassen;
    }
}

impl Default for DynamicThresholdManager {
    fn default() -> Self {
        Self::new(DynamicThresholdConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_metric(bit_length: usize, fft_speedup: f64, parallel_speedup: f64) -> IterationMetric {
        IterationMetric::basic(bit_length, fft_speedup, parallel_speedup, 1_000_000)
    }

    #[test]
    fn default_thresholds() {
        let mgr = DynamicThresholdManager::default();
        assert_eq!(mgr.parallel_threshold(), DEFAULT_PARALLEL_THRESHOLD);
        assert_eq!(mgr.fft_threshold(), DEFAULT_FFT_THRESHOLD);
        assert_eq!(mgr.strassen_threshold(), DEFAULT_STRASSEN_THRESHOLD);
    }

    #[test]
    fn record_and_adjust() {
        let mut mgr = DynamicThresholdManager::default();
        mgr.record(make_metric(1000, 0.2, 0.1));
        mgr.adjust();
        // After one positive sample with large benefit, FFT threshold should decrease
        assert!(mgr.fft_threshold() < DEFAULT_FFT_THRESHOLD);
    }

    #[test]
    fn ring_buffer_wrapping() {
        let config = DynamicThresholdConfig {
            ring_buffer_size: 4,
            ..DynamicThresholdConfig::default()
        };
        let mut mgr = DynamicThresholdManager::new(config);

        for i in 0..4 {
            mgr.record(make_metric(1000 + i * 100, 0.1, 0.1));
        }
        assert_eq!(mgr.metric_count(), 4);
        assert!(!mgr.is_ring_full());

        // Wrap around
        mgr.record(make_metric(2000, 0.1, 0.1));
        assert_eq!(mgr.metric_count(), 4);
        assert!(mgr.is_ring_full());
    }

    #[test]
    fn hysteresis_dead_zone() {
        let config = DynamicThresholdConfig {
            ring_buffer_size: 8,
            hysteresis_factor: 0.1,
            max_adjustment: 0.1,
            dead_zone: 0.05,
        };
        let mut mgr = DynamicThresholdManager::new(config);

        for _ in 0..4 {
            mgr.record(make_metric(1000, 0.01, 0.01));
        }
        let fft_before = mgr.fft_threshold();
        mgr.adjust();
        assert_eq!(mgr.fft_threshold(), fft_before);
        assert_eq!(mgr.adjustment_count(), 0);
    }

    #[test]
    fn negative_benefit_increases_threshold() {
        let config = DynamicThresholdConfig {
            ring_buffer_size: 8,
            hysteresis_factor: 0.05,
            max_adjustment: 0.1,
            dead_zone: 0.02,
        };
        let mut mgr = DynamicThresholdManager::new(config);

        for _ in 0..4 {
            mgr.record(make_metric(1000, -0.2, -0.2));
        }
        mgr.adjust();
        assert!(mgr.fft_threshold() > DEFAULT_FFT_THRESHOLD);
        assert!(mgr.parallel_threshold() > DEFAULT_PARALLEL_THRESHOLD);
    }

    #[test]
    fn strassen_threshold_adjustment() {
        let config = DynamicThresholdConfig {
            ring_buffer_size: 16,
            hysteresis_factor: 0.05,
            max_adjustment: 0.1,
            dead_zone: 0.02,
        };
        let mut mgr = DynamicThresholdManager::new(config);

        for _ in 0..8 {
            mgr.record(make_metric(2000, 0.2, 0.1));
        }
        let strassen_before = mgr.strassen_threshold();
        mgr.adjust();
        assert!(mgr.strassen_threshold() < strassen_before);
    }

    #[test]
    fn snapshot_captures_state() {
        let mut mgr = DynamicThresholdManager::default();
        mgr.record(make_metric(1000, 0.2, 0.1));
        mgr.adjust();

        let snap = mgr.snapshot();
        assert_eq!(snap.fft_threshold, mgr.fft_threshold());
        assert_eq!(snap.parallel_threshold, mgr.parallel_threshold());
        assert_eq!(snap.strassen_threshold, mgr.strassen_threshold());
        assert!(snap.adjustment_count > 0);
        assert!(!snap.adjustment_history.is_empty());
    }

    #[test]
    fn stats_empty() {
        let mgr = DynamicThresholdManager::default();
        assert!(mgr.stats().is_none());
    }

    #[test]
    fn stats_with_data() {
        let mut mgr = DynamicThresholdManager::default();
        mgr.record(make_metric(1000, 0.2, 0.1));
        mgr.record(make_metric(2000, 0.4, 0.3));

        let stats = mgr.stats().unwrap();
        assert_eq!(stats.sample_count, 2);
        assert!((stats.fft_benefit - 0.3).abs() < f64::EPSILON);
        assert!((stats.parallel_benefit - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn reset_restores_defaults() {
        let mut mgr = DynamicThresholdManager::default();
        mgr.record(make_metric(1000, 0.2, 0.1));
        mgr.adjust();
        assert_ne!(mgr.fft_threshold(), DEFAULT_FFT_THRESHOLD);

        mgr.reset();
        assert_eq!(mgr.fft_threshold(), DEFAULT_FFT_THRESHOLD);
        assert_eq!(mgr.parallel_threshold(), DEFAULT_PARALLEL_THRESHOLD);
        assert_eq!(mgr.strassen_threshold(), DEFAULT_STRASSEN_THRESHOLD);
        assert_eq!(mgr.metric_count(), 0);
        assert_eq!(mgr.adjustment_count(), 0);
        assert!(!mgr.is_ring_full());
    }

    #[test]
    fn set_thresholds() {
        let mut mgr = DynamicThresholdManager::default();
        mgr.set_thresholds(2048, 250_000, 1536);
        assert_eq!(mgr.parallel_threshold(), 2048);
        assert_eq!(mgr.fft_threshold(), 250_000);
        assert_eq!(mgr.strassen_threshold(), 1536);
    }

    #[test]
    fn multiple_adjustments_track_history() {
        let config = DynamicThresholdConfig {
            ring_buffer_size: 4,
            hysteresis_factor: 0.05,
            max_adjustment: 0.1,
            dead_zone: 0.02,
        };
        let mut mgr = DynamicThresholdManager::new(config);

        for round in 0..3 {
            for _ in 0..4 {
                mgr.record(make_metric(1000, 0.2 + (round as f64 * 0.1), 0.1));
            }
            mgr.adjust();
        }
        assert!(mgr.adjustment_count() >= 3);
        let snap = mgr.snapshot();
        assert!(!snap.adjustment_history.is_empty());
    }

    #[test]
    fn floor_prevents_zero_threshold() {
        let config = DynamicThresholdConfig {
            ring_buffer_size: 4,
            hysteresis_factor: 0.01,
            max_adjustment: 0.5,
            dead_zone: 0.005,
        };
        let mut mgr = DynamicThresholdManager::new(config);
        mgr.set_thresholds(1024, 1024, 1024);

        for _ in 0..20 {
            for _ in 0..4 {
                mgr.record(make_metric(2000, 0.9, 0.9));
            }
            mgr.adjust();
        }
        assert!(mgr.fft_threshold() >= 1024);
        assert!(mgr.parallel_threshold() >= 512);
        assert!(mgr.strassen_threshold() >= 512);
    }
}
