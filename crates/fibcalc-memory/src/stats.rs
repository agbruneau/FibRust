//! Atomic pool statistics for lock-free usage tracking.

use std::sync::atomic::{AtomicU64, Ordering};

/// Statistics for pool usage.
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Number of cache hits (acquired from pool).
    pub hits: u64,
    /// Number of cache misses (created new).
    pub misses: u64,
    /// Number of evictions (too large or pool full).
    pub evictions: u64,
}

/// Atomic pool statistics for lock-free updates.
pub struct AtomicPoolStats {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
}

impl AtomicPoolStats {
    /// Create new zeroed stats.
    pub fn new() -> Self {
        Self {
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
        }
    }

    /// Take a snapshot of current stats.
    pub fn snapshot(&self) -> PoolStats {
        PoolStats {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
        }
    }

    /// Reset all counters.
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
    }

    /// Increment hit counter.
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment miss counter.
    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment eviction counter.
    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }
}

impl Default for AtomicPoolStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stats_are_zeroed() {
        let stats = AtomicPoolStats::new();
        let snap = stats.snapshot();
        assert_eq!(snap.hits, 0);
        assert_eq!(snap.misses, 0);
        assert_eq!(snap.evictions, 0);
    }

    #[test]
    fn record_and_snapshot() {
        let stats = AtomicPoolStats::new();
        stats.record_hit();
        stats.record_hit();
        stats.record_miss();
        stats.record_eviction();
        stats.record_eviction();
        stats.record_eviction();
        let snap = stats.snapshot();
        assert_eq!(snap.hits, 2);
        assert_eq!(snap.misses, 1);
        assert_eq!(snap.evictions, 3);
    }

    #[test]
    fn reset_clears_counters() {
        let stats = AtomicPoolStats::new();
        stats.record_hit();
        stats.record_miss();
        stats.record_eviction();
        stats.reset();
        let snap = stats.snapshot();
        assert_eq!(snap.hits, 0);
        assert_eq!(snap.misses, 0);
        assert_eq!(snap.evictions, 0);
    }
}
