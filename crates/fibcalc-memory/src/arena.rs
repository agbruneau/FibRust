//! Bump arena allocator for temporary allocations.
//!
//! Uses bumpalo for O(1) allocation of temporaries during Fibonacci computation.
//! Supports both typed allocations (via `bump()`) and slice allocations for FFT.

use bumpalo::Bump;

/// Unified bump arena for calculation and FFT temporaries.
pub struct BumpArena {
    bump: Bump,
}

impl BumpArena {
    /// Create a new arena with default capacity.
    #[must_use]
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    /// Create a new arena with the given initial capacity in bytes.
    #[must_use]
    pub fn with_capacity(bytes: usize) -> Self {
        Self {
            bump: Bump::with_capacity(bytes),
        }
    }

    /// Allocate a zero-filled slice of u64 values (for FFT scratch buffers).
    pub fn alloc_slice(&self, len: usize) -> &mut [u64] {
        self.bump.alloc_slice_fill_default(len)
    }

    /// Get a reference to the underlying bumpalo allocator (for typed allocations).
    #[must_use]
    pub fn bump(&self) -> &Bump {
        &self.bump
    }

    /// Reset the arena, deallocating all objects at once.
    pub fn reset(&mut self) {
        self.bump.reset();
    }

    /// Get the number of bytes currently allocated.
    #[must_use]
    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes()
    }
}

impl Default for BumpArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_create_and_reset() {
        let mut arena = BumpArena::new();
        let _ = arena.bump().alloc(42u64);
        assert!(arena.allocated_bytes() > 0);
        arena.reset();
    }

    #[test]
    fn arena_with_capacity() {
        let arena = BumpArena::with_capacity(1024 * 1024);
        let _ = arena.allocated_bytes();
    }

    #[test]
    fn alloc_and_reset() {
        let mut alloc = BumpArena::new();
        let slice = alloc.alloc_slice(100);
        assert_eq!(slice.len(), 100);
        assert!(alloc.allocated_bytes() > 0);
        alloc.reset();
    }

    #[test]
    fn default_creates_new_allocator() {
        let alloc = BumpArena::default();
        assert_eq!(alloc.allocated_bytes(), 0);
    }

    #[test]
    fn with_capacity_then_alloc() {
        let alloc = BumpArena::with_capacity(4096);
        let _ = alloc.allocated_bytes();
    }

    #[test]
    fn alloc_slice_zero_length() {
        let alloc = BumpArena::new();
        let slice = alloc.alloc_slice(0);
        assert_eq!(slice.len(), 0);
    }

    #[test]
    fn alloc_slice_values_are_zeroed() {
        let alloc = BumpArena::new();
        let slice = alloc.alloc_slice(10);
        for &val in slice.iter() {
            assert_eq!(val, 0);
        }
    }

    #[test]
    fn alloc_slice_is_writable() {
        let alloc = BumpArena::new();
        let slice = alloc.alloc_slice(5);
        for (i, val) in slice.iter_mut().enumerate() {
            *val = i as u64 * 100;
        }
        assert_eq!(slice[0], 0);
        assert_eq!(slice[4], 400);
    }

    #[test]
    fn multiple_allocations_independent() {
        let alloc = BumpArena::new();
        let s1 = alloc.alloc_slice(10);
        s1[0] = 42;
        let s2 = alloc.alloc_slice(10);
        s2[0] = 99;
        assert_eq!(s1[0], 42);
        assert_eq!(s2[0], 99);
    }

    #[test]
    fn allocated_bytes_grows() {
        let alloc = BumpArena::new();
        let before = alloc.allocated_bytes();
        let _ = alloc.alloc_slice(1000);
        let after = alloc.allocated_bytes();
        assert!(after > before);
    }

    #[test]
    fn reset_then_reuse() {
        let mut alloc = BumpArena::new();
        let _ = alloc.alloc_slice(500);
        let bytes_before_reset = alloc.allocated_bytes();
        assert!(bytes_before_reset > 0);
        alloc.reset();
        let slice = alloc.alloc_slice(100);
        assert_eq!(slice.len(), 100);
        for &val in slice.iter() {
            assert_eq!(val, 0);
        }
    }

    #[test]
    fn large_allocation() {
        let alloc = BumpArena::new();
        let slice = alloc.alloc_slice(10_000);
        assert_eq!(slice.len(), 10_000);
        assert!(alloc.allocated_bytes() >= 10_000 * 8);
    }
}
