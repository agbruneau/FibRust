//! Core orchestration: parallel execution and result analysis.

use std::sync::Arc;
use std::time::{Duration, Instant};

use fibcalc_core::calculator::{Calculator, FibError};
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
    let observer = NoOpObserver::new();
    let start_time = Instant::now();

    if calculators.len() == 1 {
        // Single calculator: run directly
        let calc = &calculators[0];
        let start = Instant::now();
        let result = calc.calculate(cancel, &observer, 0, n, opts);
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

            let result = calc.calculate(cancel, &observer, i, n, opts);
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
}
