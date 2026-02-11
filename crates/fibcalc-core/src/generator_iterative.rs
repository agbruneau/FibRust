//! Iterative Fibonacci sequence generator.

use num_bigint::BigUint;

use crate::calculator::FibError;
use crate::generator::SequenceGenerator;
use crate::progress::CancellationToken;

/// Iterative generator that computes sequential Fibonacci numbers.
pub struct IterativeGenerator;

impl IterativeGenerator {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for IterativeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl SequenceGenerator for IterativeGenerator {
    fn generate(
        &self,
        start: u64,
        end: u64,
        cancel: &CancellationToken,
    ) -> Result<Vec<(u64, BigUint)>, FibError> {
        if start > end {
            return Err(FibError::Config("start must be <= end".into()));
        }

        let mut results = Vec::with_capacity((end - start + 1) as usize);
        let mut a = BigUint::from(0u32);
        let mut b = BigUint::from(1u32);

        for i in 0..=end {
            if cancel.is_cancelled() {
                return Err(FibError::Cancelled);
            }

            if i >= start {
                results.push((i, a.clone()));
            }

            let next = &a + &b;
            a = std::mem::replace(&mut b, next);
        }

        Ok(results)
    }

    fn name(&self) -> &'static str {
        "IterativeGenerator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_first_ten() {
        let gen = IterativeGenerator::new();
        let cancel = CancellationToken::new();
        let results = gen.generate(0, 9, &cancel).unwrap();
        assert_eq!(results.len(), 10);
        assert_eq!(results[0], (0, BigUint::from(0u32)));
        assert_eq!(results[1], (1, BigUint::from(1u32)));
        assert_eq!(results[9], (9, BigUint::from(34u32)));
    }

    #[test]
    fn generate_range() {
        let gen = IterativeGenerator::new();
        let cancel = CancellationToken::new();
        let results = gen.generate(5, 7, &cancel).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], (5, BigUint::from(5u32)));
        assert_eq!(results[1], (6, BigUint::from(8u32)));
        assert_eq!(results[2], (7, BigUint::from(13u32)));
    }
}
