//! Iterative Fibonacci sequence generator.

use num_bigint::BigUint;

use crate::calculator::FibError;
use crate::generator::SequenceGenerator;
use crate::progress::CancellationToken;

/// Iterative generator that computes sequential Fibonacci numbers.
#[allow(dead_code)] // TODO: Phase 3 — PRD §2.15.1 IterativeGenerator
pub struct IterativeGenerator;

#[allow(dead_code)] // TODO: Phase 3 — PRD §2.15.1 IterativeGenerator
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
    #[allow(clippy::cast_possible_truncation)]
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

    #[test]
    fn generate_single_element() {
        let gen = IterativeGenerator::new();
        let cancel = CancellationToken::new();
        let results = gen.generate(0, 0, &cancel).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], (0, BigUint::from(0u32)));
    }

    #[test]
    fn generate_single_element_nonzero() {
        let gen = IterativeGenerator::new();
        let cancel = CancellationToken::new();
        let results = gen.generate(10, 10, &cancel).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], (10, BigUint::from(55u32)));
    }

    #[test]
    fn generate_start_greater_than_end_errors() {
        let gen = IterativeGenerator::new();
        let cancel = CancellationToken::new();
        let result = gen.generate(10, 5, &cancel);
        assert!(matches!(result, Err(FibError::Config(_))));
    }

    #[test]
    fn generate_cancellation() {
        let gen = IterativeGenerator::new();
        let cancel = CancellationToken::new();
        cancel.cancel();
        let result = gen.generate(0, 100, &cancel);
        assert!(matches!(result, Err(FibError::Cancelled)));
    }

    #[test]
    fn generate_known_values() {
        let gen = IterativeGenerator::new();
        let cancel = CancellationToken::new();
        let results = gen.generate(0, 20, &cancel).unwrap();

        let expected: Vec<u64> = vec![
            0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597, 2584, 4181,
            6765,
        ];

        for (i, expected_val) in expected.iter().enumerate() {
            assert_eq!(
                results[i],
                (i as u64, BigUint::from(*expected_val)),
                "F({i}) should be {expected_val}"
            );
        }
    }

    #[test]
    fn default_trait() {
        let gen = IterativeGenerator::default();
        assert_eq!(gen.name(), "IterativeGenerator");
    }

    #[test]
    fn generator_name() {
        let gen = IterativeGenerator::new();
        assert_eq!(gen.name(), "IterativeGenerator");
    }

    #[test]
    fn generate_late_start() {
        let gen = IterativeGenerator::new();
        let cancel = CancellationToken::new();
        // Start from a later position
        let results = gen.generate(20, 20, &cancel).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], (20, BigUint::from(6765u32)));
    }
}
