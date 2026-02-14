//! Orchestration interfaces.

use std::time::Duration;

use num_bigint::BigUint;

use fibcalc_core::calculator::FibError;
use fibcalc_core::progress::ProgressUpdate;

/// Trait for reporting progress to the user.
pub trait ProgressReporter: Send + Sync {
    /// Report a progress update.
    fn report(&self, update: &ProgressUpdate);

    /// Report completion.
    fn complete(&self);
}

/// Trait for presenting results to the user.
pub trait ResultPresenter: Send + Sync {
    /// Present a calculation result.
    fn present_result(
        &self,
        algorithm: &str,
        n: u64,
        result: &BigUint,
        duration: Duration,
        details: bool,
    );

    /// Present a comparison result.
    fn present_comparison(&self, results: &[CalculationResult]);

    /// Present an error.
    fn present_error(&self, error: &str);
}

/// Result of a single calculation.
#[derive(Debug, Clone)]
pub struct CalculationResult {
    /// Algorithm name.
    pub algorithm: String,
    /// The computed value or a structured error.
    pub outcome: Result<BigUint, FibError>,
    /// Computation duration.
    pub duration: Duration,
}

/// Null progress reporter (does nothing).
pub struct NullProgressReporter;

impl ProgressReporter for NullProgressReporter {
    fn report(&self, _update: &ProgressUpdate) {}
    fn complete(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_reporter() {
        let reporter = NullProgressReporter;
        reporter.report(&ProgressUpdate::new(0, "test", 0.5, 1, 2));
        reporter.complete();
    }

    #[test]
    fn calculation_result() {
        let result = CalculationResult {
            algorithm: "FastDoubling".into(),
            outcome: Ok(BigUint::from(55u32)),
            duration: Duration::from_millis(100),
        };
        assert_eq!(result.algorithm, "FastDoubling");
        assert!(result.outcome.is_ok());
    }
}
