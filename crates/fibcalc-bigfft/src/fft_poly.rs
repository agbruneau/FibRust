//! Polynomial operations for FFT multiplication.

use num_bigint::BigUint;
use num_traits::One;

use crate::fermat::FermatNum;

/// Polynomial representation for FFT multiplication.
pub struct Poly {
    /// Coefficients of the polynomial (`FermatNum` values).
    pub coeffs: Vec<FermatNum>,
    /// Fermat modulus shift parameter.
    pub fermat_shift: usize,
    /// Number of bits per piece used for splitting the input.
    pub piece_bits: usize,
}

impl Poly {
    /// Create a polynomial from a `BigUint` by splitting into pieces of `piece_bits` bits.
    ///
    /// - `n`: number of coefficients (padded with zeros)
    /// - `piece_bits`: bits per piece
    /// - `fermat_shift`: Fermat modulus parameter for the NTT
    #[must_use]
    pub fn from_biguint(value: &BigUint, n: usize, piece_bits: usize, fermat_shift: usize) -> Self {
        let mask = (BigUint::one() << piece_bits) - BigUint::one();
        let mut coeffs = Vec::with_capacity(n);
        let mut remaining = value.clone();

        for _ in 0..n {
            let piece = &remaining & &mask;
            remaining >>= piece_bits;
            coeffs.push(FermatNum::from_biguint(&piece, fermat_shift));
        }

        Self {
            coeffs,
            fermat_shift,
            piece_bits,
        }
    }

    /// Convert polynomial back to `BigUint` by evaluating at x = `2^piece_bits`.
    #[must_use]
    pub fn to_biguint(&self) -> BigUint {
        let mut result = BigUint::from(0u32);
        for (i, coeff) in self.coeffs.iter().enumerate() {
            result += coeff.to_biguint() << (i * self.piece_bits);
        }
        result
    }

    /// Get the number of coefficients.
    #[must_use]
    pub fn len(&self) -> usize {
        self.coeffs.len()
    }

    /// Check if the polynomial is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.coeffs.is_empty()
    }
}

/// Pointwise multiplication of two polynomial coefficient arrays mod Fermat.
#[must_use]
pub fn pointwise_multiply(a: &[FermatNum], b: &[FermatNum], _shift: usize) -> Vec<FermatNum> {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b.iter())
        .map(|(ai, bi)| ai.fermat_mul(bi))
        .collect()
}

/// Reassemble a `BigUint` from NTT result coefficients.
///
/// Each coefficient c[i] is placed at bit position i * `piece_bits`.
/// Carries are handled naturally by `BigUint` addition.
#[must_use]
pub fn reassemble(coeffs: &[FermatNum], piece_bits: usize) -> BigUint {
    let mut result = BigUint::from(0u32);
    for (i, coeff) in coeffs.iter().enumerate() {
        result += coeff.to_biguint() << (i * piece_bits);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poly_roundtrip_small() {
        let value = BigUint::from(12345u64);
        let poly = Poly::from_biguint(&value, 4, 16, 64);
        assert_eq!(poly.len(), 4);
        assert_eq!(poly.to_biguint(), value);
    }

    #[test]
    fn poly_roundtrip_large() {
        // Test with a value that spans multiple pieces
        let value = (BigUint::one() << 200) + BigUint::from(999u64);
        let poly = Poly::from_biguint(&value, 8, 64, 256);
        assert_eq!(poly.to_biguint(), value);
    }

    #[test]
    fn poly_zero() {
        let value = BigUint::from(0u32);
        let poly = Poly::from_biguint(&value, 4, 64, 128);
        assert_eq!(poly.to_biguint(), value);
    }

    #[test]
    fn pointwise_multiply_simple() {
        let shift = 128;
        let a = vec![
            FermatNum::from_biguint(&BigUint::from(3u64), shift),
            FermatNum::from_biguint(&BigUint::from(5u64), shift),
        ];
        let b = vec![
            FermatNum::from_biguint(&BigUint::from(7u64), shift),
            FermatNum::from_biguint(&BigUint::from(11u64), shift),
        ];
        let c = pointwise_multiply(&a, &b, shift);
        assert_eq!(c[0].to_biguint(), BigUint::from(21u64));
        assert_eq!(c[1].to_biguint(), BigUint::from(55u64));
    }
}
