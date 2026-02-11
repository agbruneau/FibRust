//! Concrete observer implementations.

use std::sync::atomic::{AtomicU64, Ordering};

use crossbeam_channel::Sender;
use tracing::{debug, info};

use crate::constants::PROGRESS_REPORT_THRESHOLD;
use crate::observer::{FrozenObserver, ProgressObserver};
use crate::progress::ProgressUpdate;

/// Observer that sends updates through a channel (non-blocking).
pub struct ChannelObserver {
    sender: Sender<ProgressUpdate>,
    last_reported: AtomicU64,
}

impl ChannelObserver {
    /// Create a new channel observer.
    #[must_use]
    pub fn new(sender: Sender<ProgressUpdate>) -> Self {
        Self {
            sender,
            last_reported: AtomicU64::new(0),
        }
    }
}

impl ProgressObserver for ChannelObserver {
    fn on_progress(&self, update: &ProgressUpdate) {
        let last = f64::from_bits(self.last_reported.load(Ordering::Relaxed));
        if update.done || (update.progress - last) >= PROGRESS_REPORT_THRESHOLD {
            // Non-blocking send (select/default pattern from Go)
            let _ = self.sender.try_send(update.clone());
            self.last_reported
                .store(update.progress.to_bits(), Ordering::Relaxed);
        }
    }

    fn freeze(&self) -> FrozenObserver {
        FrozenObserver::new(PROGRESS_REPORT_THRESHOLD)
    }
}

/// Observer that logs progress updates with temporal throttling.
pub struct LoggingObserver {
    last_reported: AtomicU64,
    min_interval_ms: u64,
    last_time: AtomicU64,
}

impl LoggingObserver {
    /// Create a new logging observer with the given minimum interval.
    #[must_use]
    pub fn new(min_interval_ms: u64) -> Self {
        Self {
            last_reported: AtomicU64::new(0),
            min_interval_ms,
            last_time: AtomicU64::new(0),
        }
    }
}

impl ProgressObserver for LoggingObserver {
    fn on_progress(&self, update: &ProgressUpdate) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let last_time = self.last_time.load(Ordering::Relaxed);
        if now - last_time < self.min_interval_ms && !update.done {
            return;
        }

        let last = f64::from_bits(self.last_reported.load(Ordering::Relaxed));
        if update.done || (update.progress - last) >= PROGRESS_REPORT_THRESHOLD {
            if update.done {
                info!(
                    algorithm = %update.algorithm,
                    "Calculation complete"
                );
            } else {
                debug!(
                    algorithm = %update.algorithm,
                    progress = format!("{:.1}%", update.progress * 100.0),
                    step = update.current_step,
                    total = update.total_steps,
                    "Progress update"
                );
            }
            self.last_reported
                .store(update.progress.to_bits(), Ordering::Relaxed);
            self.last_time.store(now, Ordering::Relaxed);
        }
    }

    fn freeze(&self) -> FrozenObserver {
        FrozenObserver::new(PROGRESS_REPORT_THRESHOLD)
    }
}

/// Null object pattern — does nothing with progress updates.
pub struct NoOpObserver;

impl NoOpObserver {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoOpObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressObserver for NoOpObserver {
    fn on_progress(&self, _update: &ProgressUpdate) {
        // Intentionally empty
    }

    fn freeze(&self) -> FrozenObserver {
        FrozenObserver::new(1.0) // Never reports
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_observer_does_nothing() {
        let observer = NoOpObserver::new();
        let update = ProgressUpdate::new(0, "test", 0.5, 1, 2);
        observer.on_progress(&update);
        // No assertion needed — just ensure it doesn't panic
    }

    #[test]
    fn channel_observer_sends() {
        let (tx, rx) = crossbeam_channel::bounded(10);
        let observer = ChannelObserver::new(tx);

        let update = ProgressUpdate::new(0, "test", 0.5, 1, 2);
        observer.on_progress(&update);

        let received = rx.try_recv();
        assert!(received.is_ok());
        assert_eq!(received.unwrap().algorithm, "test");
    }

    #[test]
    fn channel_observer_throttles() {
        let (tx, rx) = crossbeam_channel::bounded(10);
        let observer = ChannelObserver::new(tx);

        // First update at 0.5% should be sent (>= 1% from 0.0)
        observer.on_progress(&ProgressUpdate::new(0, "test", 0.015, 1, 200));
        assert!(rx.try_recv().is_ok());

        // Small increment (0.015 -> 0.02 = 0.005 < 0.01) should be throttled
        observer.on_progress(&ProgressUpdate::new(0, "test", 0.02, 2, 200));
        assert!(rx.try_recv().is_err());

        // Larger increment (0.015 -> 0.03 = 0.015 >= 0.01) should be sent
        observer.on_progress(&ProgressUpdate::new(0, "test", 0.03, 4, 200));
        assert!(rx.try_recv().is_ok());
    }
}
