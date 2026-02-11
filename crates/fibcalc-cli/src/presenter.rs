//! CLI result presenter and progress reporter.

use std::io::{self, Write};
use std::time::Duration;

use num_bigint::BigUint;

use fibcalc_core::progress::ProgressUpdate;
use fibcalc_orchestration::interfaces::{CalculationResult, ProgressReporter, ResultPresenter};

use crate::output::{format_duration, format_number, format_result};

/// CLI progress reporter using stderr.
pub struct CLIProgressReporter {
    quiet: bool,
}

impl CLIProgressReporter {
    #[must_use]
    pub fn new(quiet: bool) -> Self {
        Self { quiet }
    }
}

impl ProgressReporter for CLIProgressReporter {
    fn report(&self, update: &ProgressUpdate) {
        if self.quiet {
            return;
        }
        eprint!(
            "\r  [{:>6.1}%] {} — step {}/{}",
            update.progress * 100.0,
            update.algorithm,
            update.current_step,
            update.total_steps,
        );
        let _ = io::stderr().flush();
    }

    fn complete(&self) {
        if !self.quiet {
            eprintln!();
        }
    }
}

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
            let status = if result.error.is_some() {
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

    #[test]
    fn presenter_quiet_mode() {
        let presenter = CLIResultPresenter::new(false, true);
        // In quiet mode, should just print the number
        // (Testing output capture is complex, so we just verify construction)
        assert!(presenter.quiet);
    }

    #[test]
    fn reporter_quiet_mode() {
        let reporter = CLIProgressReporter::new(true);
        let update = ProgressUpdate::new(0, "test", 0.5, 1, 2);
        reporter.report(&update);
        reporter.complete();
        // Should not panic in quiet mode
    }

    // --- CLIProgressReporter additional tests ---

    #[test]
    fn reporter_non_quiet_mode() {
        let reporter = CLIProgressReporter::new(false);
        let update = ProgressUpdate::new(0, "FastDoubling", 0.5, 50, 100);
        reporter.report(&update);
        reporter.complete();
        // Should not panic in non-quiet mode
    }

    #[test]
    fn reporter_report_at_zero_progress() {
        let reporter = CLIProgressReporter::new(false);
        let update = ProgressUpdate::new(0, "Matrix", 0.0, 0, 100);
        reporter.report(&update);
        reporter.complete();
    }

    #[test]
    fn reporter_report_at_full_progress() {
        let reporter = CLIProgressReporter::new(false);
        let update = ProgressUpdate::new(0, "FFT", 1.0, 100, 100);
        reporter.report(&update);
        reporter.complete();
    }

    #[test]
    fn reporter_multiple_updates() {
        let reporter = CLIProgressReporter::new(false);
        for step in 0..=10 {
            let progress = step as f64 / 10.0;
            let update = ProgressUpdate::new(0, "FastDoubling", progress, step, 10);
            reporter.report(&update);
        }
        reporter.complete();
    }

    #[test]
    fn reporter_quiet_complete() {
        let reporter = CLIProgressReporter::new(true);
        reporter.complete();
        // Should not panic or print anything
    }

    // --- CLIResultPresenter additional tests ---

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
            value: Some(BigUint::from(55u64)),
            duration: Duration::from_millis(5),
            error: None,
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
                value: Some(BigUint::from(55u64)),
                duration: Duration::from_millis(5),
                error: None,
            },
            CalculationResult {
                algorithm: "Matrix".into(),
                value: Some(BigUint::from(55u64)),
                duration: Duration::from_millis(10),
                error: None,
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
                value: Some(BigUint::from(55u64)),
                duration: Duration::from_millis(5),
                error: None,
            },
            CalculationResult {
                algorithm: "FFT".into(),
                value: None,
                duration: Duration::from_millis(0),
                error: Some("computation failed".into()),
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
