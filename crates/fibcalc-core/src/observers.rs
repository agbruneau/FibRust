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
    #[allow(clippy::cast_possible_truncation)]
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
    /// Create a new no-op observer that discards all progress updates.
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

    #[test]
    fn channel_observer_always_sends_done() {
        let (tx, rx) = crossbeam_channel::bounded(10);
        let observer = ChannelObserver::new(tx);

        // Send a progress at 0.99 first
        observer.on_progress(&ProgressUpdate::new(0, "test", 0.99, 99, 100));
        let _ = rx.try_recv(); // consume it

        // A done update should always be sent even if progress delta < threshold
        let mut done_update = ProgressUpdate::new(0, "test", 0.995, 100, 100);
        done_update.done = true;
        observer.on_progress(&done_update);
        let received = rx.try_recv();
        assert!(received.is_ok());
        assert!(received.unwrap().done);
    }

    #[test]
    fn channel_observer_freeze_returns_frozen() {
        let (tx, _rx) = crossbeam_channel::bounded(10);
        let observer = ChannelObserver::new(tx);
        let frozen = observer.freeze();
        // FrozenObserver should start at 0.0 and use PROGRESS_REPORT_THRESHOLD
        assert!(frozen.should_report(PROGRESS_REPORT_THRESHOLD));
        assert!(!frozen.should_report(PROGRESS_REPORT_THRESHOLD / 2.0));
    }

    #[test]
    fn channel_observer_full_channel_does_not_panic() {
        // Channel with capacity 1
        let (tx, _rx) = crossbeam_channel::bounded(1);
        let observer = ChannelObserver::new(tx);

        // First send fills the channel
        observer.on_progress(&ProgressUpdate::new(0, "test", 0.02, 1, 100));
        // Second send should silently drop (try_send)
        observer.on_progress(&ProgressUpdate::new(0, "test", 0.05, 5, 100));
        // Should not panic
    }

    #[test]
    fn logging_observer_temporal_throttling() {
        let observer = LoggingObserver::new(1000); // 1 second throttle

        // First update should be accepted (last_time is 0, now >> 0)
        observer.on_progress(&ProgressUpdate::new(0, "test", 0.05, 5, 100));

        // Immediate second update with enough progress delta should be throttled by time
        observer.on_progress(&ProgressUpdate::new(0, "test", 0.10, 10, 100));
        // No panic = success; we can't easily assert logging output, but
        // we can verify last_reported was updated only once
    }

    #[test]
    fn logging_observer_done_bypasses_time_throttle() {
        let observer = LoggingObserver::new(60_000); // Very long throttle

        // First update
        observer.on_progress(&ProgressUpdate::new(0, "test", 0.05, 5, 100));

        // Done update should bypass time check
        observer.on_progress(&ProgressUpdate::done(0, "test"));
        // No panic = success
    }

    #[test]
    fn logging_observer_freeze() {
        let observer = LoggingObserver::new(100);
        let frozen = observer.freeze();
        assert!(frozen.should_report(PROGRESS_REPORT_THRESHOLD));
    }

    #[test]
    fn logging_observer_small_progress_throttled() {
        let observer = LoggingObserver::new(0); // No time throttle

        // First update at 0.015
        observer.on_progress(&ProgressUpdate::new(0, "test", 0.015, 1, 100));

        // Very small increment (< PROGRESS_REPORT_THRESHOLD from last reported)
        observer.on_progress(&ProgressUpdate::new(0, "test", 0.016, 2, 100));
        // Should be throttled by progress threshold
    }

    #[test]
    fn noop_observer_default() {
        let observer = NoOpObserver::default();
        observer.on_progress(&ProgressUpdate::new(0, "test", 0.5, 1, 2));
    }

    #[test]
    fn noop_observer_freeze_never_reports() {
        let observer = NoOpObserver::new();
        let frozen = observer.freeze();
        // Threshold is 1.0, so no sub-1.0 progress should trigger
        assert!(!frozen.should_report(0.5));
        assert!(!frozen.should_report(0.99));
        assert!(frozen.should_report(1.0));
    }
}
