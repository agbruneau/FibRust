//! Lazy Fibonacci iterator using the standard additive recurrence.

use num_bigint::BigUint;
use num_traits::{One, Zero};

/// Lazy iterator over the Fibonacci sequence.
///
/// Yields `(index, F(index))` pairs starting from F(0).
///
/// # Example
/// ```
/// use fibcalc_core::iterator::FibIterator;
/// let fibs: Vec<_> = FibIterator::new().take(7).map(|(_, v)| v.to_string()).collect();
/// assert_eq!(fibs, ["0", "1", "1", "2", "3", "5", "8"]);
/// ```
pub struct FibIterator {
    a: BigUint,
    b: BigUint,
    index: u64,
}

impl FibIterator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            a: BigUint::zero(),
            b: BigUint::one(),
            index: 0,
        }
    }

    /// Start iteration from a specific index using the fast path for small n.
    #[must_use]
    pub fn from_index(n: u64) -> Self {
        // For simplicity, start from 0 and skip.
        // A more efficient version could use fast doubling to jump to n.
        let mut iter = Self::new();
        for _ in 0..n {
            iter.next();
        }
        iter
    }
}

impl Default for FibIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for FibIterator {
    type Item = (u64, BigUint);

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.a.clone();
        let idx = self.index;
        let next = &self.a + &self.b;
        self.a = std::mem::replace(&mut self.b, next);
        self.index += 1;
        Some((idx, val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_ten() {
        let vals: Vec<u64> = FibIterator::new()
            .take(10)
            .map(|(_, v)| v.try_into().unwrap())
            .collect();
        assert_eq!(vals, [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
    }

    #[test]
    fn yields_correct_indices() {
        let indices: Vec<u64> = FibIterator::new().take(5).map(|(i, _)| i).collect();
        assert_eq!(indices, [0, 1, 2, 3, 4]);
    }

    #[test]
    fn from_index_skips() {
        let (idx, val) = FibIterator::from_index(10).next().unwrap();
        assert_eq!(idx, 10);
        assert_eq!(val, BigUint::from(55u32));
    }
}
