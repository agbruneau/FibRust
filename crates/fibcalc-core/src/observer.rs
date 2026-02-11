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
}
