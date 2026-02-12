//! Public FFT multiplication API.
//!
//! Routes to the NTT-based multiplication pipeline for large operands,
//! and falls back to standard num-bigint multiplication for small ones.

use num_bigint::BigUint;
use num_traits::Zero;

use crate::fermat::select_fft_params;
use crate::fft_core::{fft_forward, fft_inverse};
use crate::fft_poly::{pointwise_multiply, reassemble, Poly};

/// Threshold in bits above which FFT multiplication is used.
const FFT_BIT_THRESHOLD: usize = 10_000;

/// Multiply two `BigUints`, using FFT for large operands.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn mul(a: &BigUint, b: &BigUint) -> BigUint {
    let max_bits = a.bits().max(b.bits()) as usize;
    if max_bits >= FFT_BIT_THRESHOLD {
        fft_multiply(a, b)
    } else {
        a * b
    }
}

/// Square a `BigUint`, using FFT with transform reuse for large operands.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn sqr(a: &BigUint) -> BigUint {
    let bits = a.bits() as usize;
    if bits >= FFT_BIT_THRESHOLD {
        fft_square(a)
    } else {
        a * a
    }
}

/// Multiply and store result in destination.
pub fn mul_to(dst: &mut BigUint, a: &BigUint, b: &BigUint) {
    *dst = mul(a, b);
}

/// Square and store result in destination.
pub fn sqr_to(dst: &mut BigUint, a: &BigUint) {
    *dst = sqr(a);
}

/// FFT multiplication core using Schönhage-Strassen NTT over Fermat ring.
#[allow(clippy::cast_possible_truncation)]
fn fft_multiply(a: &BigUint, b: &BigUint) -> BigUint {
    if a.is_zero() || b.is_zero() {
        return BigUint::ZERO;
    }

    let a_bits = a.bits() as usize;
    let b_bits = b.bits() as usize;
    let (piece_bits, n, fermat_shift) = select_fft_params(a_bits, b_bits);

    // Split into polynomials
    let poly_a = Poly::from_biguint(a, n, piece_bits, fermat_shift);
    let poly_b = Poly::from_biguint(b, n, piece_bits, fermat_shift);

    let mut coeffs_a = poly_a.coeffs;
    let mut coeffs_b = poly_b.coeffs;

    // Forward NTT
    fft_forward(&mut coeffs_a, fermat_shift);
    fft_forward(&mut coeffs_b, fermat_shift);

    // Pointwise multiply in transform domain
    let mut result_coeffs = pointwise_multiply(&coeffs_a, &coeffs_b, fermat_shift);

    // Inverse NTT
    fft_inverse(&mut result_coeffs, fermat_shift);

    // Reassemble from polynomial coefficients
    reassemble(&result_coeffs, piece_bits)
}

/// FFT squaring with transform reuse optimization.
///
/// Only performs one forward NTT instead of two.
#[allow(clippy::cast_possible_truncation)]
fn fft_square(a: &BigUint) -> BigUint {
    if a.is_zero() {
        return BigUint::ZERO;
    }

    let a_bits = a.bits() as usize;
    let (piece_bits, n, fermat_shift) = select_fft_params(a_bits, a_bits);

    // Split into polynomial
    let poly_a = Poly::from_biguint(a, n, piece_bits, fermat_shift);
    let mut coeffs = poly_a.coeffs;

    // Forward NTT (only once for squaring)
    fft_forward(&mut coeffs, fermat_shift);

    // Pointwise square in-place (reuse same transform, no new allocation)
    for coeff in &mut coeffs {
        let squared = coeff.fermat_mul(coeff);
        *coeff = squared;
    }

    // Inverse NTT
    fft_inverse(&mut coeffs, fermat_shift);

    // Reassemble
    reassemble(&coeffs, piece_bits)
}

/// Direct FFT multiply (always uses FFT, for testing purposes).
#[cfg(test)]
fn fft_multiply_direct(a: &BigUint, b: &BigUint) -> BigUint {
    fft_multiply(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::One;

    #[test]
    fn fft_mul_small() {
        let a = BigUint::from(12345u64);
        let b = BigUint::from(67890u64);
        assert_eq!(mul(&a, &b), BigUint::from(838_102_050u64));
    }

    #[test]
    fn fft_sqr_small() {
        let a = BigUint::from(99999u64);
        assert_eq!(sqr(&a), BigUint::from(9_999_800_001u64));
    }

    #[test]
    fn fft_mul_zero() {
        let a = BigUint::from(12345u64);
        let b = BigUint::ZERO;
        assert_eq!(mul(&a, &b), BigUint::ZERO);
    }

    #[test]
    fn fft_multiply_correctness() {
        // Test FFT multiplication directly with numbers of various sizes.
        // Use deterministic patterns rather than random numbers.
        for &bit_size in &[128, 256, 512, 1024] {
            let a = (BigUint::one() << bit_size) - BigUint::one(); // 2^n - 1
            let b = (BigUint::one() << bit_size) - BigUint::from(3u64); // 2^n - 3
            let expected = &a * &b;
            let got = fft_multiply_direct(&a, &b);
            assert_eq!(
                expected, got,
                "FFT multiply failed for {bit_size}-bit numbers"
            );
        }
    }

    #[test]
    fn fft_square_correctness() {
        for &bit_size in &[128, 256, 512, 1024] {
            let a = (BigUint::one() << bit_size) - BigUint::one();
            let expected = &a * &a;
            let got = fft_square(&a);
            assert_eq!(
                expected, got,
                "FFT square failed for {bit_size}-bit numbers"
            );
        }
    }

    #[test]
    fn fft_multiply_asymmetric() {
        // Test with operands of different sizes
        let a = (BigUint::one() << 512) - BigUint::one();
        let b = BigUint::from(12345u64);
        let expected = &a * &b;
        let got = fft_multiply_direct(&a, &b);
        assert_eq!(expected, got, "FFT multiply failed for asymmetric operands");
    }
}
