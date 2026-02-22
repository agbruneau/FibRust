//! Allocator trait and implementations.

use num_bigint::BigUint;

/// Trait for temporary allocators used in FFT operations.
pub trait TempAllocator: Send + Sync {
    /// Allocate a `BigUint` with at least the given bit capacity.
    fn alloc(&self, min_bits: usize) -> BigUint;

    /// Return a `BigUint` for potential reuse.
    fn free(&self, value: BigUint);
}

/// Pool-based allocator.
pub struct PoolAllocator {
    pool: crate::pool::BigIntPool,
}

impl PoolAllocator {
    /// Create a new pool-based allocator with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pool: crate::pool::BigIntPool::default(),
        }
    }

    /// Return a snapshot of pool usage statistics.
    #[must_use]
    pub fn stats(&self) -> crate::pool::PoolStats {
        self.pool.stats()
    }
}

impl Default for PoolAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl TempAllocator for PoolAllocator {
    fn alloc(&self, min_bits: usize) -> BigUint {
        self.pool.acquire(min_bits)
    }

    fn free(&self, value: BigUint) {
        self.pool.release(value);
    }
}

/// Simple allocator that creates new values each time.
pub struct SimpleAllocator;

impl TempAllocator for SimpleAllocator {
    fn alloc(&self, _min_bits: usize) -> BigUint {
        BigUint::from(0u32)
    }

    fn free(&self, _value: BigUint) {
        // Drop
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_allocator() {
        let alloc = SimpleAllocator;
        let val = alloc.alloc(1000);
        alloc.free(val);
    }

    #[test]
    fn pool_allocator() {
        let alloc = PoolAllocator::new();
        let val = alloc.alloc(1000);
        alloc.free(val);
    }
}
