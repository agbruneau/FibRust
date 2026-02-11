//! Bridge between calculation progress and TUI messages.

use std::time::Duration;

use crossbeam_channel::Sender;
use num_bigint::BigUint;

use fibcalc_core::constants::PROGRESS_REPORT_THRESHOLD;
use fibcalc_core::observer::{FrozenObserver, ProgressObserver};
use fibcalc_core::progress::ProgressUpdate;
use fibcalc_orchestration::interfaces::{CalculationResult, ProgressReporter, ResultPresenter};

use crate::messages::TuiMessage;

/// TUI progress reporter that sends messages to the TUI.
pub struct TUIProgressReporter {
    tx: Sender<TuiMessage>,
}

impl TUIProgressReporter {
    #[must_use]
    pub fn new(tx: Sender<TuiMessage>) -> Self {
        Self { tx }
    }
}

impl ProgressReporter for TUIProgressReporter {
    fn report(&self, update: &ProgressUpdate) {
        let _ = self.tx.try_send(TuiMessage::Progress {
            index: update.calc_index,
            progress: update.progress,
            algorithm: update.algorithm.clone(),
        });
    }

    fn complete(&self) {
        // TUI handles completion via the Complete message
    }
}

/// Core-level progress observer that forwards updates to the TUI channel.
///
/// Implements `ProgressObserver` (fibcalc-core trait) so it can be passed
/// directly to `execute_calculations_with_observer`.
pub struct TuiBridgeObserver {
    tx: Sender<TuiMessage>,
}

impl TuiBridgeObserver {
    #[must_use]
    pub fn new(tx: Sender<TuiMessage>) -> Self {
        Self { tx }
    }
}

impl ProgressObserver for TuiBridgeObserver {
    fn on_progress(&self, update: &ProgressUpdate) {
        let _ = self.tx.try_send(TuiMessage::Progress {
            index: update.calc_index,
            progress: update.progress,
            algorithm: update.algorithm.clone(),
        });
    }

    fn freeze(&self) -> FrozenObserver {
        FrozenObserver::new(PROGRESS_REPORT_THRESHOLD)
    }
}

/// TUI result presenter.
pub struct TUIResultPresenter {
    tx: Sender<TuiMessage>,
}

impl TUIResultPresenter {
    #[must_use]
    pub fn new(tx: Sender<TuiMessage>) -> Self {
        Self { tx }
    }
}

impl ResultPresenter for TUIResultPresenter {
    fn present_result(
        &self,
        algorithm: &str,
        n: u64,
        _result: &BigUint,
        duration: Duration,
        _details: bool,
    ) {
        let _ = self.tx.try_send(TuiMessage::Complete {
            algorithm: algorithm.to_string(),
            duration,
        });
        let _ = self.tx.try_send(TuiMessage::Log(format!(
            "F({n}) computed by {algorithm} in {duration:.3?}"
        )));
    }

    fn present_comparison(&self, results: &[CalculationResult]) {
        for r in results {
            let status = if r.error.is_some() { "ERROR" } else { "OK" };
            let _ = self.tx.try_send(TuiMessage::Log(format!(
                "{}: {:.3?} [{}]",
                r.algorithm, r.duration, status
            )));
        }
    }

    fn present_error(&self, error: &str) {
        let _ = self.tx.try_send(TuiMessage::Log(format!("Error: {error}")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::unbounded;
    use fibcalc_core::progress::ProgressUpdate;

    fn make_progress_update(index: usize, progress: f64, algo: &str) -> ProgressUpdate {
        ProgressUpdate {
            calc_index: index,
            algorithm: algo.to_string(),
            progress,
            current_step: 50,
            total_steps: 100,
            done: false,
        }
    }

    // --- TUIProgressReporter tests ---

    #[test]
    fn progress_reporter_new() {
        let (tx, _rx) = unbounded();
        let reporter = TUIProgressReporter::new(tx);
        // Simply verify construction does not panic
        drop(reporter);
    }

    #[test]
    fn progress_reporter_report_sends_progress_message() {
        let (tx, rx) = unbounded();
        let reporter = TUIProgressReporter::new(tx);

        let update = make_progress_update(0, 0.75, "FastDoubling");
        reporter.report(&update);

        let msg = rx.try_recv().expect("should receive a message");
        match msg {
            TuiMessage::Progress {
                index,
                progress,
                algorithm,
            } => {
                assert_eq!(index, 0);
                assert!((progress - 0.75).abs() < f64::EPSILON);
                assert_eq!(algorithm, "FastDoubling");
            }
            other => panic!("expected Progress message, got {other:?}"),
        }
    }

    #[test]
    fn progress_reporter_report_multiple() {
        let (tx, rx) = unbounded();
        let reporter = TUIProgressReporter::new(tx);

        for i in 0..5 {
            let update = make_progress_update(i, i as f64 * 0.2, "Matrix");
            reporter.report(&update);
        }

        let mut count = 0;
        while rx.try_recv().is_ok() {
            count += 1;
        }
        assert_eq!(count, 5);
    }

    #[test]
    fn progress_reporter_complete_does_not_panic() {
        let (tx, rx) = unbounded();
        let reporter = TUIProgressReporter::new(tx);
        reporter.complete();
        // complete() is a no-op for TUI, should send nothing
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn progress_reporter_report_with_dropped_receiver() {
        let (tx, rx) = unbounded();
        let reporter = TUIProgressReporter::new(tx);
        drop(rx);
        // Should not panic even with dropped receiver
        let update = make_progress_update(0, 0.5, "Test");
        reporter.report(&update);
    }

    // --- TuiBridgeObserver tests ---

    #[test]
    fn bridge_observer_new() {
        let (tx, _rx) = unbounded();
        let observer = TuiBridgeObserver::new(tx);
        drop(observer);
    }

    #[test]
    fn bridge_observer_on_progress_sends_message() {
        let (tx, rx) = unbounded();
        let observer = TuiBridgeObserver::new(tx);

        let update = make_progress_update(2, 0.33, "FFT");
        observer.on_progress(&update);

        let msg = rx.try_recv().expect("should receive a message");
        match msg {
            TuiMessage::Progress {
                index,
                progress,
                algorithm,
            } => {
                assert_eq!(index, 2);
                assert!((progress - 0.33).abs() < f64::EPSILON);
                assert_eq!(algorithm, "FFT");
            }
            other => panic!("expected Progress message, got {other:?}"),
        }
    }

    #[test]
    fn bridge_observer_freeze_returns_frozen_observer() {
        let (tx, _rx) = unbounded();
        let observer = TuiBridgeObserver::new(tx);
        let frozen = observer.freeze();
        // FrozenObserver should be created with PROGRESS_REPORT_THRESHOLD
        // Just verify it doesn't panic
        drop(frozen);
    }

    #[test]
    fn bridge_observer_on_progress_with_dropped_receiver() {
        let (tx, rx) = unbounded();
        let observer = TuiBridgeObserver::new(tx);
        drop(rx);
        let update = make_progress_update(0, 0.5, "Test");
        observer.on_progress(&update);
    }

    // --- TUIResultPresenter tests ---

    #[test]
    fn result_presenter_new() {
        let (tx, _rx) = unbounded();
        let presenter = TUIResultPresenter::new(tx);
        drop(presenter);
    }

    #[test]
    fn result_presenter_present_result_sends_complete_and_log() {
        let (tx, rx) = unbounded();
        let presenter = TUIResultPresenter::new(tx);

        let result = BigUint::from(42u32);
        let duration = Duration::from_millis(123);
        presenter.present_result("FastDoubling", 100, &result, duration, false);

        // First message: Complete
        let msg1 = rx.try_recv().expect("should receive Complete");
        match msg1 {
            TuiMessage::Complete {
                algorithm,
                duration: d,
            } => {
                assert_eq!(algorithm, "FastDoubling");
                assert_eq!(d, Duration::from_millis(123));
            }
            other => panic!("expected Complete message, got {other:?}"),
        }

        // Second message: Log
        let msg2 = rx.try_recv().expect("should receive Log");
        match msg2 {
            TuiMessage::Log(text) => {
                assert!(text.contains("F(100)"));
                assert!(text.contains("FastDoubling"));
            }
            other => panic!("expected Log message, got {other:?}"),
        }
    }

    #[test]
    fn result_presenter_present_comparison_ok_results() {
        let (tx, rx) = unbounded();
        let presenter = TUIResultPresenter::new(tx);

        let results = vec![
            CalculationResult {
                algorithm: "FastDoubling".to_string(),
                value: Some(BigUint::from(42u32)),
                duration: Duration::from_millis(100),
                error: None,
            },
            CalculationResult {
                algorithm: "Matrix".to_string(),
                value: Some(BigUint::from(42u32)),
                duration: Duration::from_millis(200),
                error: None,
            },
        ];

        presenter.present_comparison(&results);

        let msg1 = rx.try_recv().expect("should receive first log");
        match msg1 {
            TuiMessage::Log(text) => {
                assert!(text.contains("FastDoubling"));
                assert!(text.contains("[OK]"));
            }
            other => panic!("expected Log, got {other:?}"),
        }

        let msg2 = rx.try_recv().expect("should receive second log");
        match msg2 {
            TuiMessage::Log(text) => {
                assert!(text.contains("Matrix"));
                assert!(text.contains("[OK]"));
            }
            other => panic!("expected Log, got {other:?}"),
        }
    }

    #[test]
    fn result_presenter_present_comparison_with_error() {
        let (tx, rx) = unbounded();
        let presenter = TUIResultPresenter::new(tx);

        let results = vec![CalculationResult {
            algorithm: "FFT".to_string(),
            value: None,
            duration: Duration::from_millis(50),
            error: Some("overflow".to_string()),
        }];

        presenter.present_comparison(&results);

        let msg = rx.try_recv().expect("should receive log");
        match msg {
            TuiMessage::Log(text) => {
                assert!(text.contains("FFT"));
                assert!(text.contains("[ERROR]"));
            }
            other => panic!("expected Log, got {other:?}"),
        }
    }

    #[test]
    fn result_presenter_present_comparison_empty() {
        let (tx, rx) = unbounded();
        let presenter = TUIResultPresenter::new(tx);
        presenter.present_comparison(&[]);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn result_presenter_present_error() {
        let (tx, rx) = unbounded();
        let presenter = TUIResultPresenter::new(tx);
        presenter.present_error("something went wrong");

        let msg = rx.try_recv().expect("should receive log");
        match msg {
            TuiMessage::Log(text) => {
                assert!(text.contains("Error:"));
                assert!(text.contains("something went wrong"));
            }
            other => panic!("expected Log, got {other:?}"),
        }
    }

    #[test]
    fn result_presenter_present_error_with_dropped_receiver() {
        let (tx, rx) = unbounded();
        let presenter = TUIResultPresenter::new(tx);
        drop(rx);
        // Should not panic
        presenter.present_error("dropped");
    }
}
