//! Progress tracking types and utilities.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::calculator::FibError;

/// Progress update sent from calculators to observers.
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    /// Calculator index (for multi-calculator runs).
    pub calc_index: usize,
    /// Name of the algorithm producing this update.
    pub algorithm: &'static str,
    /// Current progress as a fraction in [0.0, 1.0].
    pub progress: f64,
    /// Current iteration/step number.
    pub current_step: u64,
    /// Total number of steps.
    pub total_steps: u64,
    /// Whether this is the final update.
    pub done: bool,
}

impl ProgressUpdate {
    /// Create a new progress update.
    #[must_use]
    pub fn new(
        calc_index: usize,
        algorithm: &'static str,
        progress: f64,
        current: u64,
        total: u64,
    ) -> Self {
        Self {
            calc_index,
            algorithm,
            progress,
            current_step: current,
            total_steps: total,
            done: false,
        }
    }

    /// Create a completion update.
    #[must_use]
    pub fn done(calc_index: usize, algorithm: &'static str) -> Self {
        Self {
            calc_index,
            algorithm,
            progress: 1.0,
            current_step: 0,
            total_steps: 0,
            done: true,
        }
    }
}

/// Calculate total work for a Fibonacci computation.
///
/// Uses a geometric model based on powers of 4 to estimate work.
/// Each doubling step roughly quadruples the number of digits.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn calc_total_work(n: u64) -> f64 {
    if n == 0 {
        return 0.0;
    }
    let num_bits = 64 - u64::from(n.leading_zeros());
    let mut total = 0.0f64;
    for i in 0..num_bits {
        total += POWERS_OF_4[i as usize];
    }
    total
}

/// Precomputed powers of 4 for work estimation (4^0 through 4^63).
const POWERS_OF_4: [f64; 64] = {
    let mut table = [0.0f64; 64];
    table[0] = 1.0;
    let mut i = 1;
    while i < 64 {
        table[i] = table[i - 1] * 4.0;
        i += 1;
    }
    table
};

/// Cooperative cancellation token using atomic bool.
///
/// # Example
/// ```
/// use fibcalc_core::progress::CancellationToken;
///
/// let token = CancellationToken::new();
/// assert!(!token.is_cancelled());
///
/// token.cancel();
/// assert!(token.is_cancelled());
/// assert!(token.check_cancelled().is_err());
/// ```
#[derive(Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicU64>,
}

impl CancellationToken {
    /// Create a new cancellation token.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Check if cancellation has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed) != 0
    }

    /// Request cancellation.
    pub fn cancel(&self) {
        self.cancelled.store(1, Ordering::Relaxed);
    }

    /// Check for cancellation, returning an error if cancelled.
    ///
    /// Use this as a checkpoint in algorithm loops:
    /// ```
    /// use fibcalc_core::progress::CancellationToken;
    ///
    /// let token = CancellationToken::new();
    /// // Not cancelled yet — returns Ok
    /// assert!(token.check_cancelled().is_ok());
    ///
    /// token.cancel();
    /// // Now cancelled — returns Err(FibError::Cancelled)
    /// assert!(token.check_cancelled().is_err());
    /// ```
    pub fn check_cancelled(&self) -> Result<(), FibError> {
        if self.is_cancelled() {
            Err(FibError::Cancelled)
        } else {
            Ok(())
        }
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// A cancellation token with a timeout.
///
/// Combines cooperative cancellation with an absolute deadline.
/// The token is considered cancelled if either `cancel()` was called
/// or the deadline has passed.
#[derive(Clone)]
pub struct TimeoutCancellationToken {
    inner: CancellationToken,
    deadline: Instant,
}

impl TimeoutCancellationToken {
    /// Create a new timeout cancellation token with the given duration.
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        Self {
            inner: CancellationToken::new(),
            deadline: Instant::now() + timeout,
        }
    }

    /// Check if cancellation has been requested (either manual or timeout).
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.inner.is_cancelled() || Instant::now() >= self.deadline
    }

    /// Request manual cancellation.
    pub fn cancel(&self) {
        self.inner.cancel();
    }

    /// Check for cancellation (manual or timeout), returning an error if cancelled.
    pub fn check_cancelled(&self) -> Result<(), FibError> {
        if self.inner.is_cancelled() {
            return Err(FibError::Cancelled);
        }
        if Instant::now() >= self.deadline {
            return Err(FibError::Timeout("timeout reached".to_string()));
        }
        Ok(())
    }

    /// Get the remaining time before the deadline.
    #[must_use]
    pub fn remaining(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }

    /// Get the inner `CancellationToken` for passing to APIs that don't support timeout.
    #[must_use]
    pub fn token(&self) -> &CancellationToken {
        &self.inner
    }
}

/// Helper to check cancellation at a checkpoint. Returns `Err(FibError::Cancelled)` if cancelled.
///
/// This is a convenience function for use in algorithm loops:
/// ```
/// use fibcalc_core::progress::{CancellationToken, check_cancellation};
///
/// let token = CancellationToken::new();
/// assert!(check_cancellation(&token).is_ok());
///
/// token.cancel();
/// assert!(check_cancellation(&token).is_err());
/// ```
#[inline]
pub fn check_cancellation(token: &CancellationToken) -> Result<(), FibError> {
    token.check_cancelled()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_update_new() {
        let update = ProgressUpdate::new(0, "FastDoubling", 0.5, 16, 32);
        assert_eq!(update.calc_index, 0);
        assert_eq!(update.algorithm, "FastDoubling");
        assert!((update.progress - 0.5).abs() < f64::EPSILON);
        assert!(!update.done);
    }

    #[test]
    fn progress_update_done() {
        let update = ProgressUpdate::done(1, "Matrix");
        assert!(update.done);
        assert!((update.progress - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cancellation_token() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn check_cancelled_ok() {
        let token = CancellationToken::new();
        assert!(token.check_cancelled().is_ok());
    }

    #[test]
    fn check_cancelled_err() {
        let token = CancellationToken::new();
        token.cancel();
        let result = token.check_cancelled();
        assert!(matches!(result, Err(FibError::Cancelled)));
    }

    #[test]
    fn check_cancellation_helper() {
        let token = CancellationToken::new();
        assert!(check_cancellation(&token).is_ok());
        token.cancel();
        assert!(check_cancellation(&token).is_err());
    }

    #[test]
    fn timeout_token_not_expired() {
        let token = TimeoutCancellationToken::new(Duration::from_secs(60));
        assert!(!token.is_cancelled());
        assert!(token.check_cancelled().is_ok());
        assert!(token.remaining() > Duration::from_secs(0));
    }

    #[test]
    fn timeout_token_manual_cancel() {
        let token = TimeoutCancellationToken::new(Duration::from_secs(60));
        token.cancel();
        assert!(token.is_cancelled());
        assert!(matches!(token.check_cancelled(), Err(FibError::Cancelled)));
    }

    #[test]
    fn timeout_token_expired() {
        let token = TimeoutCancellationToken::new(Duration::from_millis(0));
        // Allow a tiny bit of time for the deadline to pass
        std::thread::sleep(Duration::from_millis(1));
        assert!(token.is_cancelled());
        assert!(matches!(token.check_cancelled(), Err(FibError::Timeout(_))));
    }

    #[test]
    fn timeout_token_inner_access() {
        let token = TimeoutCancellationToken::new(Duration::from_secs(60));
        let inner = token.token();
        assert!(!inner.is_cancelled());
        token.cancel();
        assert!(inner.is_cancelled());
    }

    #[test]
    fn total_work_zero() {
        assert!((calc_total_work(0)).abs() < f64::EPSILON);
    }

    #[test]
    fn total_work_positive() {
        assert!(calc_total_work(100) > 0.0);
        assert!(calc_total_work(1000) > calc_total_work(100));
    }

    #[test]
    fn cancellation_propagates_through_clone() {
        let token1 = CancellationToken::new();
        let token2 = token1.clone();
        token1.cancel();
        assert!(token2.is_cancelled());
    }
}
