//! FFT multiplication wrappers.
//!
//! Provides `mul_fft`, `sqr_fft`, `smart_multiply`, and `smart_square`
//! that route to FFT or Karatsuba based on operand size.

use num_bigint::BigUint;

#[cfg(test)]
use crate::constants::DEFAULT_FFT_THRESHOLD;

/// Multiply using FFT if operands are large enough, otherwise use default.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn smart_multiply(a: &BigUint, b: &BigUint, fft_threshold: usize) -> BigUint {
    let max_bits = a.bits().max(b.bits()) as usize;
    if max_bits >= fft_threshold {
        mul_fft(a, b)
    } else {
        a * b
    }
}

/// Square using FFT if operand is large enough.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn smart_square(a: &BigUint, fft_threshold: usize) -> BigUint {
    let bits = a.bits() as usize;
    if bits >= fft_threshold {
        sqr_fft(a)
    } else {
        a * a
    }
}

/// FFT multiplication via fibcalc-bigfft.
#[must_use]
pub fn mul_fft(a: &BigUint, b: &BigUint) -> BigUint {
    fibcalc_bigfft::mul(a, b)
}

/// FFT squaring via fibcalc-bigfft.
#[must_use]
pub fn sqr_fft(a: &BigUint) -> BigUint {
    fibcalc_bigfft::sqr(a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smart_multiply_small() {
        let a = BigUint::from(12345u64);
        let b = BigUint::from(67890u64);
        assert_eq!(
            smart_multiply(&a, &b, DEFAULT_FFT_THRESHOLD),
            BigUint::from(838_102_050u64)
        );
    }

    #[test]
    fn smart_square_small() {
        let a = BigUint::from(1000u64);
        assert_eq!(
            smart_square(&a, DEFAULT_FFT_THRESHOLD),
            BigUint::from(1_000_000u64)
        );
    }

    #[test]
    fn mul_fft_correctness() {
        let a = BigUint::from(999u64);
        let b = BigUint::from(1001u64);
        assert_eq!(mul_fft(&a, &b), BigUint::from(999_999u64));
    }

    #[test]
    fn sqr_fft_correctness() {
        let a = BigUint::from(1234u64);
        assert_eq!(sqr_fft(&a), BigUint::from(1_522_756u64));
    }
}
