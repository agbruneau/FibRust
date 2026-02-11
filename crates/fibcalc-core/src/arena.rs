//! Calculation arena for temporary allocations.
//!
//! Uses bumpalo for O(1) allocation of temporaries during Fibonacci computation.

use bumpalo::Bump;

/// Arena allocator for Fibonacci calculation temporaries.
pub struct CalculationArena {
    bump: Bump,
}

impl CalculationArena {
    /// Create a new arena with default capacity.
    #[must_use]
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    /// Create a new arena with the given initial capacity in bytes.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bump: Bump::with_capacity(capacity),
        }
    }

    /// Reset the arena, deallocating all objects at once.
    pub fn reset(&mut self) {
        self.bump.reset();
    }

    /// Get the number of bytes allocated in the arena.
    #[must_use]
    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes()
    }

    /// Get a reference to the underlying bumpalo allocator.
    #[must_use]
    pub fn bump(&self) -> &Bump {
        &self.bump
    }
}

impl Default for CalculationArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_create_and_reset() {
        let mut arena = CalculationArena::new();
        // Allocate something
        let _ = arena.bump().alloc(42u64);
        assert!(arena.allocated_bytes() > 0);
        arena.reset();
    }

    #[test]
    fn arena_with_capacity() {
        let arena = CalculationArena::with_capacity(1024 * 1024);
        // bumpalo may pre-allocate a chunk, so allocated_bytes >= 0
        let _ = arena.allocated_bytes();
    }
}
