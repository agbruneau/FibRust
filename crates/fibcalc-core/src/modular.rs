//! Modular Fast Doubling for --last-digits mode.
//!
//! Computes F(n) mod 10^k using modular arithmetic throughout,
//! avoiding the need to compute the full number.

use num_bigint::BigUint;
use num_traits::{One, Zero};

use crate::calculator::{CoreCalculator, FibError};
use crate::observer::ProgressObserver;
use crate::options::Options;
use crate::progress::{CancellationToken, ProgressUpdate};

/// Fast Doubling with modular arithmetic for computing last K digits.
pub struct FastDoublingMod;

impl FastDoublingMod {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Compute F(n) mod m using modular Fast Doubling.
    pub fn fibonacci_mod(
        n: u64,
        modulus: &BigUint,
        cancel: &CancellationToken,
        observer: &dyn ProgressObserver,
        calc_index: usize,
    ) -> Result<BigUint, FibError> {
        if modulus.is_zero() {
            return Err(FibError::Config("modulus cannot be zero".into()));
        }

        let num_bits = 64 - n.leading_zeros();
        let mut fk = BigUint::zero();
        let mut fk1 = BigUint::one();

        let frozen = observer.freeze();

        for i in (0..num_bits).rev() {
            if cancel.is_cancelled() {
                return Err(FibError::Cancelled);
            }

            // Modular doubling step
            let fk_sq = (&fk * &fk) % modulus;
            let fk1_sq = (&fk1 * &fk1) % modulus;
            let cross = (&fk * &fk1) % modulus;

            // F(2k) = (2*cross - fk_sq) mod m
            let double_cross = (&cross << 1) % modulus;
            let f2k = if double_cross >= fk_sq {
                (double_cross - &fk_sq) % modulus
            } else {
                (modulus - &fk_sq + double_cross) % modulus
            };

            // F(2k+1) = (fk1_sq + fk_sq) mod m
            let f2k1 = (fk1_sq + &fk_sq) % modulus;

            fk = f2k;
            fk1 = f2k1;

            // Conditional addition (modular)
            if (n >> i) & 1 == 1 {
                let sum = (&fk + &fk1) % modulus;
                fk = std::mem::replace(&mut fk1, sum);
            }

            // Progress
            let progress = 1.0 - (f64::from(i) / f64::from(num_bits));
            if frozen.should_report(progress) {
                frozen.update(progress);
                observer.on_progress(&ProgressUpdate::new(
                    calc_index,
                    "FastDoublingMod",
                    progress,
                    u64::from(num_bits - i),
                    u64::from(num_bits),
                ));
            }
        }

        Ok(fk)
    }
}

impl Default for FastDoublingMod {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreCalculator for FastDoublingMod {
    fn calculate_core(
        &self,
        cancel: &CancellationToken,
        observer: &dyn ProgressObserver,
        calc_index: usize,
        n: u64,
        opts: &Options,
    ) -> Result<BigUint, FibError> {
        if opts.last_digits == 0 {
            return Err(FibError::Config(
                "FastDoublingMod requires last_digits > 0".into(),
            ));
        }

        let modulus = BigUint::from(10u32).pow(opts.last_digits);
        let result = Self::fibonacci_mod(n, &modulus, cancel, observer, calc_index)?;
        observer.on_progress(&ProgressUpdate::done(calc_index, "FastDoublingMod"));
        Ok(result)
    }

    fn name(&self) -> &'static str {
        "FastDoublingMod"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observers::NoOpObserver;

    #[test]
    fn modular_last_digits() {
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let modulus = BigUint::from(1_000_000u64); // 6 digits

        // F(100) = 354224848179261915075 -> last 6 digits: 915075
        let result = FastDoublingMod::fibonacci_mod(100, &modulus, &cancel, &observer, 0).unwrap();
        assert_eq!(result, BigUint::from(915075u64));
    }

    #[test]
    fn modular_small_values() {
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let modulus = BigUint::from(100u64);

        // F(10) = 55 -> 55 mod 100 = 55
        let result = FastDoublingMod::fibonacci_mod(10, &modulus, &cancel, &observer, 0).unwrap();
        assert_eq!(result, BigUint::from(55u64));
    }

    #[test]
    fn modular_zero_modulus() {
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let modulus = BigUint::zero();
        let result = FastDoublingMod::fibonacci_mod(10, &modulus, &cancel, &observer, 0);
        assert!(result.is_err());
    }
}
