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

    #[test]
    fn default_creates_new_allocator() {
        let alloc = FFTBumpAllocator::default();
        assert_eq!(alloc.allocated_bytes(), 0);
    }

    #[test]
    fn with_capacity() {
        let alloc = FFTBumpAllocator::with_capacity(4096);
        // with_capacity pre-allocates, so allocated_bytes reflects internal overhead
        // Just verify it does not panic and can be used
        let _ = alloc.allocated_bytes();
    }

    #[test]
    fn alloc_slice_zero_length() {
        let alloc = FFTBumpAllocator::new();
        let slice = alloc.alloc_slice(0);
        assert_eq!(slice.len(), 0);
    }

    #[test]
    fn alloc_slice_values_are_zeroed() {
        let alloc = FFTBumpAllocator::new();
        let slice = alloc.alloc_slice(10);
        for &val in slice.iter() {
            assert_eq!(val, 0);
        }
    }

    #[test]
    fn alloc_slice_is_writable() {
        let alloc = FFTBumpAllocator::new();
        let slice = alloc.alloc_slice(5);
        for (i, val) in slice.iter_mut().enumerate() {
            *val = i as u64 * 100;
        }
        assert_eq!(slice[0], 0);
        assert_eq!(slice[4], 400);
    }

    #[test]
    fn multiple_allocations_independent() {
        let alloc = FFTBumpAllocator::new();
        let s1 = alloc.alloc_slice(10);
        s1[0] = 42;
        let s2 = alloc.alloc_slice(10);
        s2[0] = 99;
        // They should be independent slices
        assert_eq!(s1[0], 42);
        assert_eq!(s2[0], 99);
    }

    #[test]
    fn allocated_bytes_grows() {
        let alloc = FFTBumpAllocator::new();
        let before = alloc.allocated_bytes();
        let _ = alloc.alloc_slice(1000);
        let after = alloc.allocated_bytes();
        assert!(after > before);
    }

    #[test]
    fn reset_then_reuse() {
        let mut alloc = FFTBumpAllocator::new();
        let _ = alloc.alloc_slice(500);
        let bytes_before_reset = alloc.allocated_bytes();
        assert!(bytes_before_reset > 0);

        alloc.reset();

        // After reset, new allocations work fine
        let slice = alloc.alloc_slice(100);
        assert_eq!(slice.len(), 100);
        for &val in slice.iter() {
            assert_eq!(val, 0);
        }
    }

    #[test]
    fn with_capacity_then_alloc() {
        let alloc = FFTBumpAllocator::with_capacity(8192);
        let slice = alloc.alloc_slice(100);
        assert_eq!(slice.len(), 100);
        assert!(alloc.allocated_bytes() > 0);
    }

    #[test]
    fn large_allocation() {
        let alloc = FFTBumpAllocator::new();
        let slice = alloc.alloc_slice(10_000);
        assert_eq!(slice.len(), 10_000);
        assert!(alloc.allocated_bytes() >= 10_000 * 8);
    }
}
