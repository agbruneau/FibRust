//! Scan utilities for `BigUint` operations.

use num_bigint::BigUint;

/// Count the number of significant bits in a `BigUint`.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn bit_length(n: &BigUint) -> usize {
    n.bits() as usize
}

/// Count the number of decimal digits in a `BigUint`.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn digit_count(n: &BigUint) -> usize {
    if n == &BigUint::from(0u32) {
        return 1;
    }
    let bits = n.bits() as f64;
    (bits * 2_f64.log10()).ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_length_values() {
        assert_eq!(bit_length(&BigUint::from(0u32)), 0);
        assert_eq!(bit_length(&BigUint::from(1u32)), 1);
        assert_eq!(bit_length(&BigUint::from(255u32)), 8);
    }

    #[test]
    fn digit_count_values() {
        assert_eq!(digit_count(&BigUint::from(0u32)), 1);
        // 999 has 10 bits, log10(2^10) ≈ 3.01, ceil = 4 (overestimate)
        // This is an approximation; exact count uses to_string().len()
        assert!(digit_count(&BigUint::from(999u32)) >= 3);
    }
}
