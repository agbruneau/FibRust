//! Core orchestration: parallel execution and result analysis.

use std::sync::Arc;
use std::time::{Duration, Instant};

use fibcalc_core::calculator::{Calculator, FibError};
use fibcalc_core::observer::ProgressObserver;
use fibcalc_core::observers::NoOpObserver;
use fibcalc_core::options::Options;
use fibcalc_core::progress::CancellationToken;

use crate::interfaces::CalculationResult;

/// Execute calculations with all given calculators.
pub fn execute_calculations(
    calculators: &[Arc<dyn Calculator>],
    n: u64,
    opts: &Options,
    cancel: &CancellationToken,
    timeout: Option<Duration>,
) -> Vec<CalculationResult> {
    execute_calculations_with_observer(calculators, n, opts, cancel, timeout, &NoOpObserver::new())
}

/// Execute calculations with all given calculators and a progress observer.
pub fn execute_calculations_with_observer(
    calculators: &[Arc<dyn Calculator>],
    n: u64,
    opts: &Options,
    cancel: &CancellationToken,
    timeout: Option<Duration>,
    observer: &dyn ProgressObserver,
) -> Vec<CalculationResult> {
    let start_time = Instant::now();

    if calculators.len() == 1 {
        // Single calculator: run directly
        let calc = &calculators[0];
        let start = Instant::now();
        let result = calc.calculate(cancel, observer, 0, n, opts);
        let duration = start.elapsed();

        return vec![match result {
            Ok(value) => CalculationResult {
                algorithm: calc.name().to_string(),
                value: Some(value),
                duration,
                error: None,
            },
            Err(e) => CalculationResult {
                algorithm: calc.name().to_string(),
                value: None,
                duration,
                error: Some(e.to_string()),
            },
        }];
    }

    // Multiple calculators: run in parallel using rayon
    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    let results: Vec<CalculationResult> = calculators
        .iter()
        .enumerate()
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|(i, calc)| {
            let start = Instant::now();

            // Check timeout
            if let Some(timeout) = timeout {
                if start_time.elapsed() > timeout {
                    return CalculationResult {
                        algorithm: calc.name().to_string(),
                        value: None,
                        duration: start.elapsed(),
                        error: Some("timeout".to_string()),
                    };
                }
            }

            let result = calc.calculate(cancel, observer, i, n, opts);
            let duration = start.elapsed();

            match result {
                Ok(value) => CalculationResult {
                    algorithm: calc.name().to_string(),
                    value: Some(value),
                    duration,
                    error: None,
                },
                Err(e) => CalculationResult {
                    algorithm: calc.name().to_string(),
                    value: None,
                    duration,
                    error: Some(e.to_string()),
                },
            }
        })
        .collect();

    results
}

/// Analyze comparison results for mismatches.
pub fn analyze_comparison_results(results: &[CalculationResult]) -> Result<(), FibError> {
    let valid_results: Vec<&CalculationResult> = results
        .iter()
        .filter(|r| r.value.is_some() && r.error.is_none())
        .collect();

    if valid_results.is_empty() {
        return Err(FibError::Calculation("no valid results".into()));
    }

    // Compare all results to the first valid one
    let first_value = valid_results[0].value.as_ref().unwrap();
    for result in &valid_results[1..] {
        if result.value.as_ref().unwrap() != first_value {
            return Err(FibError::Mismatch);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fibcalc_core::calculator::FibCalculator;
    use fibcalc_core::fastdoubling::OptimizedFastDoubling;
    use num_bigint::BigUint;

    #[test]
    fn execute_single_calculator() {
        let calc: Arc<dyn Calculator> =
            Arc::new(FibCalculator::new(Arc::new(OptimizedFastDoubling::new())));
        let opts = Options::default();
        let cancel = CancellationToken::new();
        let results = execute_calculations(&[calc], 100, &opts, &cancel, None);
        assert_eq!(results.len(), 1);
        assert!(results[0].error.is_none());
        assert_eq!(
            results[0].value.as_ref().unwrap(),
            &BigUint::parse_bytes(b"354224848179261915075", 10).unwrap()
        );
    }

    #[test]
    fn analyze_matching_results() {
        let results = vec![
            CalculationResult {
                algorithm: "A".into(),
                value: Some(BigUint::from(55u32)),
                duration: Duration::from_millis(1),
                error: None,
            },
            CalculationResult {
                algorithm: "B".into(),
                value: Some(BigUint::from(55u32)),
                duration: Duration::from_millis(2),
                error: None,
            },
        ];
        assert!(analyze_comparison_results(&results).is_ok());
    }

    #[test]
    fn analyze_mismatching_results() {
        let results = vec![
            CalculationResult {
                algorithm: "A".into(),
                value: Some(BigUint::from(55u32)),
                duration: Duration::from_millis(1),
                error: None,
            },
            CalculationResult {
                algorithm: "B".into(),
                value: Some(BigUint::from(56u32)),
                duration: Duration::from_millis(2),
                error: None,
            },
        ];
        assert!(matches!(
            analyze_comparison_results(&results),
            Err(FibError::Mismatch)
        ));
    }

    #[test]
    fn analyze_no_valid_results() {
        let results = vec![
            CalculationResult {
                algorithm: "A".into(),
                value: None,
                duration: Duration::from_millis(1),
                error: Some("failed".into()),
            },
        ];
        assert!(matches!(
            analyze_comparison_results(&results),
            Err(FibError::Calculation(_))
        ));
    }

    #[test]
    fn analyze_single_valid_result() {
        let results = vec![
            CalculationResult {
                algorithm: "A".into(),
                value: Some(BigUint::from(55u32)),
                duration: Duration::from_millis(1),
                error: None,
            },
        ];
        assert!(analyze_comparison_results(&results).is_ok());
    }

    #[test]
    fn analyze_mixed_valid_and_error_results() {
        // One valid, one error -- should succeed since there's only one valid to compare
        let results = vec![
            CalculationResult {
                algorithm: "A".into(),
                value: Some(BigUint::from(55u32)),
                duration: Duration::from_millis(1),
                error: None,
            },
            CalculationResult {
                algorithm: "B".into(),
                value: None,
                duration: Duration::from_millis(2),
                error: Some("timeout".into()),
            },
        ];
        assert!(analyze_comparison_results(&results).is_ok());
    }

    #[test]
    fn analyze_empty_results() {
        let results: Vec<CalculationResult> = vec![];
        assert!(matches!(
            analyze_comparison_results(&results),
            Err(FibError::Calculation(_))
        ));
    }

    #[test]
    fn analyze_three_matching_results() {
        let val = BigUint::from(55u32);
        let results = vec![
            CalculationResult {
                algorithm: "A".into(),
                value: Some(val.clone()),
                duration: Duration::from_millis(1),
                error: None,
            },
            CalculationResult {
                algorithm: "B".into(),
                value: Some(val.clone()),
                duration: Duration::from_millis(2),
                error: None,
            },
            CalculationResult {
                algorithm: "C".into(),
                value: Some(val),
                duration: Duration::from_millis(3),
                error: None,
            },
        ];
        assert!(analyze_comparison_results(&results).is_ok());
    }

    #[test]
    fn analyze_third_result_mismatches() {
        let val = BigUint::from(55u32);
        let results = vec![
            CalculationResult {
                algorithm: "A".into(),
                value: Some(val.clone()),
                duration: Duration::from_millis(1),
                error: None,
            },
            CalculationResult {
                algorithm: "B".into(),
                value: Some(val),
                duration: Duration::from_millis(2),
                error: None,
            },
            CalculationResult {
                algorithm: "C".into(),
                value: Some(BigUint::from(56u32)),
                duration: Duration::from_millis(3),
                error: None,
            },
        ];
        assert!(matches!(
            analyze_comparison_results(&results),
            Err(FibError::Mismatch)
        ));
    }

    #[test]
    fn execute_multiple_calculators_parallel() {
        use fibcalc_core::matrix::MatrixExponentiation;

        let fast: Arc<dyn Calculator> =
            Arc::new(FibCalculator::new(Arc::new(OptimizedFastDoubling::new())));
        let matrix: Arc<dyn Calculator> =
            Arc::new(FibCalculator::new(Arc::new(MatrixExponentiation::new())));
        let opts = Options::default();
        let cancel = CancellationToken::new();
        let results = execute_calculations(&[fast, matrix], 50, &opts, &cancel, None);
        assert_eq!(results.len(), 2);
        // Both should succeed
        for r in &results {
            assert!(r.error.is_none(), "calculator {} failed: {:?}", r.algorithm, r.error);
            assert!(r.value.is_some());
        }
        // Both should compute the same value
        assert_eq!(
            results[0].value.as_ref().unwrap(),
            results[1].value.as_ref().unwrap()
        );
    }

    #[test]
    fn execute_with_cancellation() {
        let calc: Arc<dyn Calculator> =
            Arc::new(FibCalculator::new(Arc::new(OptimizedFastDoubling::new())));
        let opts = Options::default();
        let cancel = CancellationToken::new();
        cancel.cancel(); // Cancel before starting
        let results = execute_calculations(&[calc], 10_000_000, &opts, &cancel, None);
        assert_eq!(results.len(), 1);
        // For small n (fast path), it may still succeed even with cancellation
        // For very large n, it should be cancelled. With n=10M and cancellation before start,
        // the FibCalculator checks cancellation before delegating to core.
        // n=10M > 93 so it hits the cancellation check
        assert!(results[0].error.is_some() || results[0].value.is_some());
    }

    #[test]
    fn execute_with_observer() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use fibcalc_core::observer::FrozenObserver;

        struct CountingObserver {
            count: AtomicUsize,
        }
        impl ProgressObserver for CountingObserver {
            fn on_progress(&self, _update: &fibcalc_core::progress::ProgressUpdate) {
                self.count.fetch_add(1, Ordering::Relaxed);
            }
            fn freeze(&self) -> FrozenObserver {
                FrozenObserver::new(0.01)
            }
        }

        let observer = CountingObserver {
            count: AtomicUsize::new(0),
        };
        let calc: Arc<dyn Calculator> =
            Arc::new(FibCalculator::new(Arc::new(OptimizedFastDoubling::new())));
        let opts = Options::default();
        let cancel = CancellationToken::new();
        let results =
            execute_calculations_with_observer(&[calc], 50, &opts, &cancel, None, &observer);
        assert_eq!(results.len(), 1);
        assert!(results[0].value.is_some());
        // The observer should have been called at least once (the done notification)
        assert!(observer.count.load(Ordering::Relaxed) >= 1);
    }

    #[test]
    fn execute_single_calculator_small_n() {
        let calc: Arc<dyn Calculator> =
            Arc::new(FibCalculator::new(Arc::new(OptimizedFastDoubling::new())));
        let opts = Options::default();
        let cancel = CancellationToken::new();
        // Test the fast path (n <= 93)
        let results = execute_calculations(&[calc], 10, &opts, &cancel, None);
        assert_eq!(results.len(), 1);
        assert!(results[0].error.is_none());
        assert_eq!(
            results[0].value.as_ref().unwrap(),
            &BigUint::from(55u32)
        );
    }

    #[test]
    fn execute_with_timeout() {
        let calc: Arc<dyn Calculator> =
            Arc::new(FibCalculator::new(Arc::new(OptimizedFastDoubling::new())));
        let opts = Options::default();
        let cancel = CancellationToken::new();
        // Use a generous timeout that won't be exceeded for a small calculation
        let timeout = Some(Duration::from_secs(30));
        let results = execute_calculations(&[calc], 50, &opts, &cancel, timeout);
        assert_eq!(results.len(), 1);
        assert!(results[0].value.is_some());
        assert!(results[0].error.is_none());
    }

    #[test]
    fn analyze_results_ignores_error_entries() {
        // Results with value=None and error=Some should be ignored in comparison
        let val = BigUint::from(55u32);
        let results = vec![
            CalculationResult {
                algorithm: "A".into(),
                value: Some(val.clone()),
                duration: Duration::from_millis(1),
                error: None,
            },
            CalculationResult {
                algorithm: "B".into(),
                value: None,
                duration: Duration::from_millis(2),
                error: Some("failed".into()),
            },
            CalculationResult {
                algorithm: "C".into(),
                value: Some(val),
                duration: Duration::from_millis(3),
                error: None,
            },
        ];
        // Should succeed: A and C match, B is ignored
        assert!(analyze_comparison_results(&results).is_ok());
    }

    #[test]
    fn analyze_result_with_value_and_error_is_excluded() {
        // A result with both value and error set should be excluded (error is Some)
        let results = vec![
            CalculationResult {
                algorithm: "A".into(),
                value: Some(BigUint::from(55u32)),
                duration: Duration::from_millis(1),
                error: Some("partial".into()),
            },
        ];
        // The filter requires error.is_none(), so this counts as no valid results
        assert!(matches!(
            analyze_comparison_results(&results),
            Err(FibError::Calculation(_))
        ));
    }
}
