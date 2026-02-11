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
