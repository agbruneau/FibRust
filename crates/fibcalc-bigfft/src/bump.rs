//! O(1) bump allocator for FFT temporaries.

use bumpalo::Bump;

/// FFT bump allocator using bumpalo.
pub struct FFTBumpAllocator {
    bump: Bump,
}

impl FFTBumpAllocator {
    /// Create a new bump allocator.
    #[must_use]
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    /// Create with initial capacity.
    #[must_use]
    pub fn with_capacity(bytes: usize) -> Self {
        Self {
            bump: Bump::with_capacity(bytes),
        }
    }

    /// Allocate a slice of u64 values.
    pub fn alloc_slice(&self, len: usize) -> &mut [u64] {
        self.bump.alloc_slice_fill_default(len)
    }

    /// Reset the allocator (free all allocations at once).
    pub fn reset(&mut self) {
        self.bump.reset();
    }

    /// Get bytes allocated.
    #[must_use]
    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes()
    }
}

impl Default for FFTBumpAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_reset() {
        let mut alloc = FFTBumpAllocator::new();
        let slice = alloc.alloc_slice(100);
        assert_eq!(slice.len(), 100);
        assert!(alloc.allocated_bytes() > 0);
        alloc.reset();
    }
}
