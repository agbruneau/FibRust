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
}
