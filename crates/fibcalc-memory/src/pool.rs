//! BigInt pool with size classes for object reuse.

use std::collections::HashMap;

use num_bigint::BigUint;
use parking_lot::Mutex;

use crate::stats::{AtomicPoolStats, PoolStats};

/// Clear a `BigUint` by setting it to zero.
///
/// `BigUint::ZERO` is the simplest approach -- num-bigint doesn't expose its
/// internal Vec, so we can't preserve capacity. The allocation cost is negligible
/// compared to computation.
fn clear_value(value: &mut BigUint) {
    *value = BigUint::ZERO;
}

/// Pool for `BigUint` objects, organized by size class (power of 4).
pub struct BigIntPool {
    pools: Mutex<HashMap<usize, Vec<BigUint>>>,
    max_bit_len: usize,
    max_per_class: usize,
    stats: AtomicPoolStats,
}

impl BigIntPool {
    /// Create a new pool.
    #[must_use]
    pub fn new(max_bit_len: usize, max_per_class: usize) -> Self {
        Self {
            pools: Mutex::new(HashMap::new()),
            max_bit_len,
            max_per_class,
            stats: AtomicPoolStats::new(),
        }
    }

    /// Get a `BigUint` from the pool, or create a new one.
    pub fn acquire(&self, min_bits: usize) -> BigUint {
        let class = Self::size_class(min_bits);
        let mut pools = self.pools.lock();
        if let Some(pool) = pools.get_mut(&class) {
            if let Some(value) = pool.pop() {
                self.stats.record_hit();
                return value;
            }
        }
        self.stats.record_miss();
        BigUint::from(0u32)
    }

    /// Return a `BigUint` to the pool.
    #[allow(clippy::cast_possible_truncation)]
    pub fn release(&self, mut value: BigUint) {
        let bits = value.bits() as usize;
        if bits > self.max_bit_len {
            self.stats.record_eviction();
            return; // Too large to pool
        }

        let class = Self::size_class(bits);
        let mut pools = self.pools.lock();
        let pool = pools.entry(class).or_default();
        if pool.len() < self.max_per_class {
            clear_value(&mut value);
            pool.push(value);
        } else {
            self.stats.record_eviction();
        }
    }

    /// Compute size class (round up to next power of 4).
    fn size_class(bits: usize) -> usize {
        if bits <= 64 {
            return 64;
        }
        let mut class = 64;
        while class < bits {
            class *= 4;
        }
        class
    }

    /// Get total number of pooled objects.
    #[must_use]
    pub fn total_pooled(&self) -> usize {
        self.pools.lock().values().map(Vec::len).sum()
    }

    /// Get a snapshot of pool statistics.
    #[must_use]
    pub fn stats(&self) -> PoolStats {
        self.stats.snapshot()
    }

    /// Reset pool statistics counters.
    pub fn reset_stats(&self) {
        self.stats.reset();
    }

    /// Clear all pooled objects, releasing memory.
    pub fn clear(&self) {
        self.pools.lock().clear();
    }

    /// Drain all pooled objects for a given size class, returning them.
    pub fn drain_class(&self, min_bits: usize) -> Vec<BigUint> {
        let class = Self::size_class(min_bits);
        let mut pools = self.pools.lock();
        pools.remove(&class).unwrap_or_default()
    }

    /// Drain all pooled objects, returning them grouped by size class.
    pub fn drain_all(&self) -> HashMap<usize, Vec<BigUint>> {
        let mut pools = self.pools.lock();
        std::mem::take(&mut *pools)
    }

    /// Pre-populate a size class with the given number of entries.
    pub fn warm(&self, bits: usize, count: usize) {
        let class = Self::size_class(bits);
        let mut pools = self.pools.lock();
        let pool = pools.entry(class).or_default();
        let to_add = count
            .saturating_sub(pool.len())
            .min(self.max_per_class - pool.len());
        for _ in 0..to_add {
            pool.push(BigUint::from(0u32));
        }
    }
}

impl Default for BigIntPool {
    fn default() -> Self {
        Self::new(100_000_000, 32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_acquire_release() {
        let pool = BigIntPool::default();
        let value = pool.acquire(1000);
        assert_eq!(value, BigUint::from(0u32));
        pool.release(value);
        assert_eq!(pool.total_pooled(), 1);
    }

    #[test]
    fn size_class_computation() {
        assert_eq!(BigIntPool::size_class(32), 64);
        assert_eq!(BigIntPool::size_class(64), 64);
        assert_eq!(BigIntPool::size_class(65), 256);
        assert_eq!(BigIntPool::size_class(256), 256);
        assert_eq!(BigIntPool::size_class(257), 1024);
    }

    #[test]
    fn pool_stats_tracking() {
        let pool = BigIntPool::default();

        // Miss: nothing in pool
        let _ = pool.acquire(100);
        let stats = pool.stats();
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hits, 0);

        // Release and re-acquire from same class: should be a hit
        pool.release(BigUint::from(42u32));
        let _ = pool.acquire(0); // class 64, same as what we released
        let stats = pool.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn pool_stats_eviction_too_large() {
        let pool = BigIntPool::new(64, 4);
        let large = BigUint::from(1u32) << 128;
        pool.release(large);
        let stats = pool.stats();
        assert_eq!(stats.evictions, 1);
    }

    #[test]
    fn pool_stats_eviction_pool_full() {
        let pool = BigIntPool::new(100_000, 2);
        pool.release(BigUint::from(1u32));
        pool.release(BigUint::from(2u32));
        pool.release(BigUint::from(3u32)); // pool is full for class 64
        let stats = pool.stats();
        assert_eq!(stats.evictions, 1);
    }

    #[test]
    fn pool_stats_reset() {
        let pool = BigIntPool::default();
        let _ = pool.acquire(100);
        pool.reset_stats();
        let stats = pool.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.evictions, 0);
    }

    #[test]
    fn pool_clear() {
        let pool = BigIntPool::default();
        pool.release(BigUint::from(1u32));
        pool.release(BigUint::from(2u32));
        assert_eq!(pool.total_pooled(), 2);
        pool.clear();
        assert_eq!(pool.total_pooled(), 0);
    }

    #[test]
    fn pool_drain_class() {
        let pool = BigIntPool::default();
        pool.release(BigUint::from(1u32));
        pool.release(BigUint::from(2u32));
        let drained = pool.drain_class(0);
        assert_eq!(drained.len(), 2);
        assert_eq!(pool.total_pooled(), 0);
    }

    #[test]
    fn pool_drain_all() {
        let pool = BigIntPool::default();
        pool.release(BigUint::from(1u32));
        let large = BigUint::from(1u32) << 100;
        pool.release(large);
        assert_eq!(pool.total_pooled(), 2);
        let drained = pool.drain_all();
        assert!(!drained.is_empty());
        assert_eq!(pool.total_pooled(), 0);
    }

    #[test]
    fn pool_warm() {
        let pool = BigIntPool::default();
        pool.warm(1000, 5);
        assert_eq!(pool.total_pooled(), 5);

        // Warming again should not add duplicates beyond count
        pool.warm(1000, 5);
        assert_eq!(pool.total_pooled(), 5);

        // Warming with higher count should add more
        pool.warm(1000, 8);
        assert_eq!(pool.total_pooled(), 8);
    }
}
