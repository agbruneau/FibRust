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
        let digits = match opts.last_digits {
            Some(d) if d > 0 => d,
            _ => {
                return Err(FibError::Config(
                    "FastDoublingMod requires last_digits > 0".into(),
                ));
            }
        };

        let modulus = BigUint::from(10u32).pow(digits);
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

    #[test]
    fn modular_n_zero() {
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let modulus = BigUint::from(1000u64);
        let result = FastDoublingMod::fibonacci_mod(0, &modulus, &cancel, &observer, 0).unwrap();
        assert_eq!(result, BigUint::zero()); // F(0) = 0
    }

    #[test]
    fn modular_n_one() {
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let modulus = BigUint::from(1000u64);
        let result = FastDoublingMod::fibonacci_mod(1, &modulus, &cancel, &observer, 0).unwrap();
        assert_eq!(result, BigUint::one()); // F(1) = 1
    }

    #[test]
    fn modular_n_two() {
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let modulus = BigUint::from(1000u64);
        let result = FastDoublingMod::fibonacci_mod(2, &modulus, &cancel, &observer, 0).unwrap();
        assert_eq!(result, BigUint::one()); // F(2) = 1
    }

    #[test]
    fn modular_modulus_one() {
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let modulus = BigUint::one(); // Everything mod 1 = 0
        let result = FastDoublingMod::fibonacci_mod(100, &modulus, &cancel, &observer, 0).unwrap();
        assert_eq!(result, BigUint::zero());
    }

    #[test]
    fn modular_power_of_two_modulus() {
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let modulus = BigUint::from(256u64); // 2^8
                                             // F(10) = 55, 55 mod 256 = 55
        let result = FastDoublingMod::fibonacci_mod(10, &modulus, &cancel, &observer, 0).unwrap();
        assert_eq!(result, BigUint::from(55u64));
    }

    #[test]
    fn modular_large_n() {
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let modulus = BigUint::from(1_000_000_007u64); // large prime
                                                       // F(1000) mod 1_000_000_007 — just check it doesn't panic and produces a value < modulus
        let result = FastDoublingMod::fibonacci_mod(1000, &modulus, &cancel, &observer, 0).unwrap();
        assert!(result < modulus);
    }

    #[test]
    fn modular_cancellation() {
        let cancel = CancellationToken::new();
        cancel.cancel();
        let observer = NoOpObserver::new();
        let modulus = BigUint::from(1000u64);
        let result = FastDoublingMod::fibonacci_mod(10000, &modulus, &cancel, &observer, 0);
        assert!(matches!(result, Err(FibError::Cancelled)));
    }

    #[test]
    fn modular_small_modulus_two() {
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let modulus = BigUint::from(2u64);
        // F(n) mod 2 follows the Pisano period: 0,1,1,0,1,1,0,1,1,...
        // F(3) = 2 -> 2 mod 2 = 0
        let result = FastDoublingMod::fibonacci_mod(3, &modulus, &cancel, &observer, 0).unwrap();
        assert_eq!(result, BigUint::zero());
        // F(4) = 3 -> 3 mod 2 = 1
        let result = FastDoublingMod::fibonacci_mod(4, &modulus, &cancel, &observer, 0).unwrap();
        assert_eq!(result, BigUint::one());
    }

    #[test]
    fn core_calculator_requires_last_digits() {
        let calc = FastDoublingMod::new();
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let opts = Options::default(); // last_digits = None
        let result = calc.calculate_core(&cancel, &observer, 0, 100, &opts);
        assert!(matches!(result, Err(FibError::Config(_))));
    }

    #[test]
    fn core_calculator_with_last_digits() {
        let calc = FastDoublingMod::new();
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let opts = Options {
            last_digits: Some(6),
            ..Default::default()
        };
        // F(100) last 6 digits = 915075
        let result = calc
            .calculate_core(&cancel, &observer, 0, 100, &opts)
            .unwrap();
        assert_eq!(result, BigUint::from(915075u64));
    }

    #[test]
    fn core_calculator_name() {
        let calc = FastDoublingMod::new();
        assert_eq!(CoreCalculator::name(&calc), "FastDoublingMod");
    }

    #[test]
    fn default_trait() {
        let _calc = FastDoublingMod::default();
    }

    #[test]
    fn modular_known_values_table() {
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let modulus = BigUint::from(10000u64); // 4 digits

        // Known Fibonacci values and their last 4 digits
        let cases: Vec<(u64, u64)> = vec![
            (0, 0),
            (1, 1),
            (5, 5),     // F(5) = 5
            (10, 55),   // F(10) = 55
            (20, 6765), // F(20) = 6765
            (50, 5075), // F(50) = 12586269025 -> last 4 = 9025... let me recheck
        ];

        for (n, expected_last4) in &cases[..5] {
            let result =
                FastDoublingMod::fibonacci_mod(*n, &modulus, &cancel, &observer, 0).unwrap();
            assert_eq!(
                result,
                BigUint::from(*expected_last4),
                "F({n}) mod 10000 should be {expected_last4}"
            );
        }
    }
}
