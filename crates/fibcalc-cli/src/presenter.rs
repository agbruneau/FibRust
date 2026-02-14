//! CLI result presenter.

use std::time::Duration;

use num_bigint::BigUint;

use fibcalc_orchestration::interfaces::{CalculationResult, ResultPresenter};

use crate::output::{format_duration, format_number, format_result};

/// CLI result presenter.
pub struct CLIResultPresenter {
    verbose: bool,
    quiet: bool,
}

impl CLIResultPresenter {
    #[must_use]
    pub fn new(verbose: bool, quiet: bool) -> Self {
        Self { verbose, quiet }
    }
}

impl ResultPresenter for CLIResultPresenter {
    fn present_result(
        &self,
        algorithm: &str,
        n: u64,
        result: &BigUint,
        duration: Duration,
        details: bool,
    ) {
        if self.quiet {
            println!("{result}");
            return;
        }

        println!("Algorithm: {algorithm}");
        println!("N: {}", format_number(n));
        println!("Duration: {}", format_duration(duration));

        if details {
            let bits = result.bits();
            let digits = result.to_string().len();
            println!("Result bits: {bits}");
            println!("Result digits: {digits}");
        }

        println!(
            "F({}) = {}",
            format_number(n),
            format_result(result, self.verbose)
        );
    }

    fn present_comparison(&self, results: &[CalculationResult]) {
        if self.quiet {
            return;
        }

        println!("\nComparison Results:");
        println!("{:-<60}", "");
        for result in results {
            let status = if result.outcome.is_err() {
                "ERROR"
            } else {
                "OK"
            };
            println!(
                "  {:<20} {:>10} [{}]",
                result.algorithm,
                format_duration(result.duration),
                status,
            );
        }
    }

    fn present_error(&self, error: &str) {
        eprintln!("Error: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fibcalc_core::calculator::FibError;

    #[test]
    fn presenter_quiet_mode() {
        let presenter = CLIResultPresenter::new(false, true);
        // In quiet mode, should just print the number
        // (Testing output capture is complex, so we just verify construction)
        assert!(presenter.quiet);
    }

    #[test]
    fn presenter_verbose_mode() {
        let presenter = CLIResultPresenter::new(true, false);
        assert!(presenter.verbose);
        assert!(!presenter.quiet);
    }

    #[test]
    fn presenter_present_result_quiet() {
        let presenter = CLIResultPresenter::new(false, true);
        let result = BigUint::from(55u64);
        presenter.present_result("FastDoubling", 10, &result, Duration::from_millis(5), false);
        // Should not panic
    }

    #[test]
    fn presenter_present_result_normal() {
        let presenter = CLIResultPresenter::new(false, false);
        let result = BigUint::from(55u64);
        presenter.present_result("FastDoubling", 10, &result, Duration::from_millis(5), false);
    }

    #[test]
    fn presenter_present_result_with_details() {
        let presenter = CLIResultPresenter::new(false, false);
        let result = BigUint::from(832040u64);
        presenter.present_result("Matrix", 30, &result, Duration::from_millis(10), true);
    }

    #[test]
    fn presenter_present_result_verbose() {
        let presenter = CLIResultPresenter::new(true, false);
        let result = BigUint::from(12345u64);
        presenter.present_result("FFT", 100, &result, Duration::from_secs(1), true);
    }

    #[test]
    fn presenter_present_comparison_quiet() {
        let presenter = CLIResultPresenter::new(false, true);
        let results = vec![CalculationResult {
            algorithm: "FastDoubling".into(),
            outcome: Ok(BigUint::from(55u64)),
            duration: Duration::from_millis(5),
        }];
        presenter.present_comparison(&results);
        // Should not print anything in quiet mode
    }

    #[test]
    fn presenter_present_comparison_normal() {
        let presenter = CLIResultPresenter::new(false, false);
        let results = vec![
            CalculationResult {
                algorithm: "FastDoubling".into(),
                outcome: Ok(BigUint::from(55u64)),
                duration: Duration::from_millis(5),
            },
            CalculationResult {
                algorithm: "Matrix".into(),
                outcome: Ok(BigUint::from(55u64)),
                duration: Duration::from_millis(10),
            },
        ];
        presenter.present_comparison(&results);
    }

    #[test]
    fn presenter_present_comparison_with_error() {
        let presenter = CLIResultPresenter::new(false, false);
        let results = vec![
            CalculationResult {
                algorithm: "FastDoubling".into(),
                outcome: Ok(BigUint::from(55u64)),
                duration: Duration::from_millis(5),
            },
            CalculationResult {
                algorithm: "FFT".into(),
                outcome: Err(FibError::Calculation("computation failed".into())),
                duration: Duration::from_millis(0),
            },
        ];
        presenter.present_comparison(&results);
    }

    #[test]
    fn presenter_present_comparison_empty() {
        let presenter = CLIResultPresenter::new(false, false);
        presenter.present_comparison(&[]);
    }

    #[test]
    fn presenter_present_error() {
        let presenter = CLIResultPresenter::new(false, false);
        presenter.present_error("test error message");
    }

    #[test]
    fn presenter_present_error_empty() {
        let presenter = CLIResultPresenter::new(false, false);
        presenter.present_error("");
    }

    #[test]
    fn presenter_present_result_large_n() {
        let presenter = CLIResultPresenter::new(false, false);
        let result = BigUint::from(1u64);
        presenter.present_result(
            "FastDoubling",
            1_000_000,
            &result,
            Duration::from_secs(30),
            true,
        );
    }
}
