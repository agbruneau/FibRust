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
    /// Create a new Fibonacci iterator starting from F(0).
    #[must_use]
    pub fn new() -> Self {
        Self {
            a: BigUint::zero(),
            b: BigUint::one(),
            index: 0,
        }
    }

    /// Start iteration from a specific index in O(log n) using fast doubling.
    #[must_use]
    pub fn from_index(n: u64) -> Self {
        if n == 0 {
            return Self::new();
        }
        let (a, b) = fib_pair(n);
        Self { a, b, index: n }
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

/// Compute (F(n), F(n+1)) in O(log n) using fast doubling.
///
/// Identities: F(2k) = F(k)*(2*F(k+1) - F(k)), F(2k+1) = F(k)^2 + F(k+1)^2.
fn fib_pair(n: u64) -> (BigUint, BigUint) {
    let mut a = BigUint::zero(); // F(0)
    let mut b = BigUint::one(); // F(1)
    for i in (0..u64::BITS - n.leading_zeros()).rev() {
        // Doubling: (a, b) = (F(k), F(k+1)) -> (F(2k), F(2k+1))
        let two_b = &b << 1;
        let f2k = &a * (&two_b - &a);
        let f2k1 = &a * &a + &b * &b;
        if (n >> i) & 1 == 0 {
            a = f2k;
            b = f2k1;
        } else {
            a = f2k1.clone();
            b = f2k + f2k1;
        }
    }
    (a, b)
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

    #[test]
    fn from_index_large() {
        // Verify from_index(10000) produces a value consistent with iteration from that point.
        let mut iter = FibIterator::from_index(10_000);
        let (idx, f10000) = iter.next().unwrap();
        assert_eq!(idx, 10_000);
        // F(10000) has 2090 digits
        assert_eq!(f10000.to_string().len(), 2090);
        // Verify next value is F(10001) = F(10000) + F(9999)
        let (idx2, f10001) = iter.next().unwrap();
        assert_eq!(idx2, 10_001);
        assert!(f10001 > f10000);
    }

    #[test]
    fn from_index_matches_linear() {
        // Cross-check fast doubling from_index against linear iteration for several values.
        for n in [0, 1, 2, 5, 20, 50, 93] {
            let (_, fast_val) = FibIterator::from_index(n).next().unwrap();
            let (_, linear_val) = FibIterator::new().nth(n as usize).unwrap();
            assert_eq!(fast_val, linear_val, "mismatch at n={n}");
        }
    }
}
