//! Progress and ETA calculation.

use std::time::{Duration, Instant};

/// ETA calculator based on progress tracking.
pub struct ETACalculator {
    start_time: Instant,
    last_progress: f64,
    last_update: Instant,
}

impl ETACalculator {
    /// Create a new ETA calculator.
    #[must_use]
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_progress: 0.0,
            last_update: now,
        }
    }

    /// Update with new progress and return estimated time remaining.
    pub fn update(&mut self, progress: f64) -> Option<Duration> {
        if progress <= 0.0 || progress >= 1.0 {
            return None;
        }

        let elapsed = self.start_time.elapsed();
        let estimated_total = elapsed.as_secs_f64() / progress;
        let remaining = estimated_total - elapsed.as_secs_f64();

        self.last_progress = progress;
        self.last_update = Instant::now();

        if remaining > 0.0 {
            Some(Duration::from_secs_f64(remaining))
        } else {
            None
        }
    }

    /// Get elapsed time since start.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for ETACalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eta_calculator() {
        let mut calc = ETACalculator::new();
        // At 0 progress, no ETA
        assert!(calc.update(0.0).is_none());
        // At 100%, no ETA
        assert!(calc.update(1.0).is_none());
    }
}
