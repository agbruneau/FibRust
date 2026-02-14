//! Sequence generator trait.

use num_bigint::BigUint;

use crate::calculator::FibError;
use crate::progress::CancellationToken;

/// Trait for generating sequences of Fibonacci numbers.
#[allow(dead_code)] // TODO: Phase 3 — PRD §2.15.1 SequenceGenerator
pub trait SequenceGenerator: Send + Sync {
    /// Generate Fibonacci numbers from start to end.
    fn generate(
        &self,
        start: u64,
        end: u64,
        cancel: &CancellationToken,
    ) -> Result<Vec<(u64, BigUint)>, FibError>;

    /// Get the name of this generator.
    fn name(&self) -> &str;
}
