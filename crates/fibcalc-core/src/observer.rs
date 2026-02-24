//! Observer pattern for progress tracking.
//!
//! Implements the Observer pattern with a lock-free `Freeze()` mechanism
//! for high-frequency updates in hot loops.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::progress::ProgressUpdate;

/// Observer trait for receiving progress updates.
pub trait ProgressObserver: Send + Sync {
    /// Receive a progress update.
    fn on_progress(&self, update: &ProgressUpdate);

    /// Create a frozen snapshot for lock-free access in hot loops.
    fn freeze(&self) -> FrozenObserver;
}

/// A frozen observer that can be used in hot loops without locks.
///
/// Stores progress as atomic values for lock-free reads.
pub struct FrozenObserver {
    progress: Arc<AtomicU64>,
    threshold: f64,
}

impl FrozenObserver {
    /// Create a new frozen observer with the given reporting threshold.
    #[must_use]
    pub fn new(threshold: f64) -> Self {
        Self {
            progress: Arc::new(AtomicU64::new(0)),
            threshold,
        }
    }

    /// Check if progress has changed enough to warrant reporting.
    #[inline]
    #[must_use]
    pub fn should_report(&self, new_progress: f64) -> bool {
        let current = f64::from_bits(self.progress.load(Ordering::Relaxed));
        (new_progress - current) >= self.threshold
    }

    /// Update the stored progress value.
    pub fn update(&self, new_progress: f64) {
        self.progress
            .store(new_progress.to_bits(), Ordering::Relaxed);
    }

    /// Get the current progress value.
    #[must_use]
    pub fn current(&self) -> f64 {
        f64::from_bits(self.progress.load(Ordering::Relaxed))
    }
}

/// Subject that manages a collection of observers.
pub struct ProgressSubject {
    observers: RwLock<Vec<Arc<dyn ProgressObserver>>>,
}

impl ProgressSubject {
    /// Create a new subject with no observers.
    #[must_use]
    pub fn new() -> Self {
        Self {
            observers: RwLock::new(Vec::new()),
        }
    }

    /// Register an observer.
    pub fn register(&self, observer: Arc<dyn ProgressObserver>) {
        self.observers.write().push(observer);
    }

    /// Unregister all observers.
    pub fn clear(&self) {
        self.observers.write().clear();
    }

    /// Notify all observers of a progress update.
    pub fn notify(&self, update: &ProgressUpdate) {
        let observers = self.observers.read();
        for observer in observers.iter() {
            observer.on_progress(update);
        }
    }

    /// Get the number of registered observers.
    #[must_use]
    pub fn count(&self) -> usize {
        self.observers.read().len()
    }
}

impl Default for ProgressSubject {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::PROGRESS_REPORT_THRESHOLD;

    #[test]
    fn frozen_observer_should_report() {
        let frozen = FrozenObserver::new(PROGRESS_REPORT_THRESHOLD);
        assert!(frozen.should_report(0.02));
        frozen.update(0.02);
        assert!(!frozen.should_report(0.025));
        assert!(frozen.should_report(0.04));
    }

    #[test]
    fn subject_register_and_count() {
        let subject = ProgressSubject::new();
        assert_eq!(subject.count(), 0);
    }

    #[test]
    fn subject_clear() {
        let subject = ProgressSubject::new();
        subject.clear();
        assert_eq!(subject.count(), 0);
    }

    #[test]
    fn frozen_observer_initial_progress_is_zero() {
        let frozen = FrozenObserver::new(0.05);
        assert!((frozen.current() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn frozen_observer_update_and_current() {
        let frozen = FrozenObserver::new(0.01);
        frozen.update(0.5);
        assert!((frozen.current() - 0.5).abs() < f64::EPSILON);
        frozen.update(0.75);
        assert!((frozen.current() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn frozen_observer_should_report_exact_threshold() {
        let frozen = FrozenObserver::new(0.1);
        // Exactly at threshold boundary
        assert!(frozen.should_report(0.1));
        frozen.update(0.1);
        assert!(!frozen.should_report(0.15));
        assert!(frozen.should_report(0.2));
    }

    #[test]
    fn frozen_observer_zero_threshold_always_reports() {
        let frozen = FrozenObserver::new(0.0);
        assert!(frozen.should_report(0.0));
        frozen.update(0.0);
        assert!(frozen.should_report(0.001));
    }

    #[test]
    fn subject_register_increases_count() {
        use crate::observers::NoOpObserver;

        let subject = ProgressSubject::new();
        assert_eq!(subject.count(), 0);

        subject.register(Arc::new(NoOpObserver::new()));
        assert_eq!(subject.count(), 1);

        subject.register(Arc::new(NoOpObserver::new()));
        assert_eq!(subject.count(), 2);
    }

    #[test]
    fn subject_clear_removes_all() {
        use crate::observers::NoOpObserver;

        let subject = ProgressSubject::new();
        subject.register(Arc::new(NoOpObserver::new()));
        subject.register(Arc::new(NoOpObserver::new()));
        assert_eq!(subject.count(), 2);

        subject.clear();
        assert_eq!(subject.count(), 0);
    }

    #[test]
    fn subject_notify_calls_all_observers() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct CountingObserver {
            count: AtomicUsize,
        }

        impl CountingObserver {
            fn new() -> Self {
                Self {
                    count: AtomicUsize::new(0),
                }
            }
        }

        impl ProgressObserver for CountingObserver {
            fn on_progress(&self, _update: &ProgressUpdate) {
                self.count.fetch_add(1, Ordering::Relaxed);
            }

            fn freeze(&self) -> FrozenObserver {
                FrozenObserver::new(PROGRESS_REPORT_THRESHOLD)
            }
        }

        let subject = ProgressSubject::new();
        let obs1 = Arc::new(CountingObserver::new());
        let obs2 = Arc::new(CountingObserver::new());

        subject.register(obs1.clone());
        subject.register(obs2.clone());

        let update = ProgressUpdate::new(0, "test", 0.5, 1, 2);
        subject.notify(&update);

        assert_eq!(obs1.count.load(Ordering::Relaxed), 1);
        assert_eq!(obs2.count.load(Ordering::Relaxed), 1);

        // Notify again
        subject.notify(&update);
        assert_eq!(obs1.count.load(Ordering::Relaxed), 2);
        assert_eq!(obs2.count.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn subject_notify_empty_does_not_panic() {
        let subject = ProgressSubject::new();
        let update = ProgressUpdate::new(0, "test", 0.5, 1, 2);
        subject.notify(&update); // Should not panic
    }

    #[test]
    fn subject_default() {
        let subject = ProgressSubject::default();
        assert_eq!(subject.count(), 0);
    }
}
