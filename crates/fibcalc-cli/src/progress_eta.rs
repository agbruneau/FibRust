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
    use std::thread;

    #[test]
    fn eta_calculator() {
        let mut calc = ETACalculator::new();
        // At 0 progress, no ETA
        assert!(calc.update(0.0).is_none());
        // At 100%, no ETA
        assert!(calc.update(1.0).is_none());
    }

    #[test]
    fn eta_default() {
        let calc = ETACalculator::default();
        assert!(calc.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn eta_negative_progress() {
        let mut calc = ETACalculator::new();
        assert!(calc.update(-0.5).is_none());
    }

    #[test]
    fn eta_progress_above_one() {
        let mut calc = ETACalculator::new();
        assert!(calc.update(1.5).is_none());
    }

    #[test]
    fn eta_mid_progress_returns_some() {
        let mut calc = ETACalculator::new();
        // Wait a small amount so elapsed > 0
        thread::sleep(Duration::from_millis(10));
        let eta = calc.update(0.5);
        // With 50% done and some elapsed time, ETA should be Some
        assert!(eta.is_some());
    }

    #[test]
    fn eta_small_progress_large_remaining() {
        let mut calc = ETACalculator::new();
        thread::sleep(Duration::from_millis(10));
        let eta = calc.update(0.01);
        // At 1% done, remaining should be much larger than elapsed
        assert!(eta.is_some());
        let remaining = eta.unwrap();
        let elapsed = calc.elapsed();
        assert!(remaining > elapsed);
    }

    #[test]
    fn eta_high_progress_small_remaining() {
        let mut calc = ETACalculator::new();
        thread::sleep(Duration::from_millis(10));
        let eta = calc.update(0.99);
        // At 99% done, remaining should be small relative to elapsed
        assert!(eta.is_some());
        let remaining = eta.unwrap();
        let elapsed = calc.elapsed();
        assert!(remaining < elapsed);
    }

    #[test]
    fn eta_elapsed_increases() {
        let calc = ETACalculator::new();
        let e1 = calc.elapsed();
        thread::sleep(Duration::from_millis(10));
        let e2 = calc.elapsed();
        assert!(e2 >= e1);
    }

    #[test]
    fn eta_successive_updates() {
        let mut calc = ETACalculator::new();
        thread::sleep(Duration::from_millis(5));
        let eta1 = calc.update(0.25);
        thread::sleep(Duration::from_millis(5));
        let eta2 = calc.update(0.75);

        // Both should return Some (given elapsed time > 0)
        assert!(eta1.is_some());
        assert!(eta2.is_some());

        // At higher progress, remaining time should be less
        assert!(eta2.unwrap() < eta1.unwrap());
    }

    #[test]
    fn eta_boundary_just_above_zero() {
        let mut calc = ETACalculator::new();
        thread::sleep(Duration::from_millis(5));
        let eta = calc.update(0.001);
        assert!(eta.is_some());
    }

    #[test]
    fn eta_boundary_just_below_one() {
        let mut calc = ETACalculator::new();
        thread::sleep(Duration::from_millis(5));
        let eta = calc.update(0.999);
        assert!(eta.is_some());
    }
}
