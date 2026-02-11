//! Fermat number arithmetic for FFT.
//!
//! Fermat numbers `F_k` = 2^(2^k) + 1 are used as moduli for
//! Number Theoretic Transform (NTT) based multiplication.

use num_bigint::BigUint;
use num_traits::One;

/// A Fermat number representation: value = data mod (2^shift + 1).
#[derive(Debug, Clone)]
pub struct FermatNum {
    /// The value represented as limbs (little-endian u64).
    pub data: Vec<u64>,
    /// The shift: the Fermat modulus is 2^shift + 1.
    pub shift: usize,
}

impl FermatNum {
    /// Create a new zero Fermat number with the given shift.
    #[must_use]
    pub fn new(shift: usize) -> Self {
        let num_limbs = shift.div_ceil(64) + 1;
        Self {
            data: vec![0; num_limbs],
            shift,
        }
    }

    /// Create from a `BigUint` value.
    #[must_use]
    pub fn from_biguint(value: &BigUint, shift: usize) -> Self {
        let digits = value.to_u64_digits();
        let num_limbs = shift.div_ceil(64) + 1;
        let mut data = vec![0u64; num_limbs];
        let copy_len = digits.len().min(num_limbs);
        data[..copy_len].copy_from_slice(&digits[..copy_len]);
        Self { data, shift }
    }

    /// Convert back to `BigUint`.
    #[must_use]
    pub fn to_biguint(&self) -> BigUint {
        // Convert u64 limbs to little-endian bytes for reliable conversion
        let bytes: Vec<u8> = self
            .data
            .iter()
            .flat_map(|&limb| limb.to_le_bytes())
            .collect();
        BigUint::from_bytes_le(&bytes)
    }

    /// Get the Fermat modulus: 2^shift + 1.
    #[must_use]
    pub fn modulus(&self) -> BigUint {
        (BigUint::one() << self.shift) + BigUint::one()
    }

    /// Normalize: reduce mod (2^shift + 1).
    pub fn normalize(&mut self) {
        let modulus = self.modulus();
        let val = self.to_biguint() % &modulus;
        *self = Self::from_biguint(&val, self.shift);
    }

    /// Add two Fermat numbers mod (2^shift + 1).
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        assert_eq!(self.shift, other.shift);
        let modulus = self.modulus();
        let a = self.to_biguint();
        let b = other.to_biguint();
        let sum = (a + b) % modulus;
        Self::from_biguint(&sum, self.shift)
    }

    /// Subtract other from self mod (2^shift + 1).
    #[must_use]
    pub fn sub(&self, other: &Self) -> Self {
        assert_eq!(self.shift, other.shift);
        let modulus = self.modulus();
        let a = self.to_biguint() % &modulus;
        let b = other.to_biguint() % &modulus;
        let result = if a >= b { a - b } else { &modulus - &b + &a };
        Self::from_biguint(&result, self.shift)
    }

    /// Multiply two Fermat numbers mod (2^shift + 1).
    #[must_use]
    pub fn fermat_mul(&self, other: &Self) -> Self {
        assert_eq!(self.shift, other.shift);
        let modulus = self.modulus();
        let a = self.to_biguint();
        let b = other.to_biguint();
        let product = (a * b) % modulus;
        Self::from_biguint(&product, self.shift)
    }

    /// Multiply by 2^s mod (2^shift + 1).
    pub fn shift_left(&mut self, s: usize) {
        if s == 0 {
            return;
        }
        let modulus = self.modulus();
        let val = self.to_biguint();
        let shifted = (val << s) % modulus;
        *self = Self::from_biguint(&shifted, self.shift);
    }

    /// Divide by 2^k mod (2^shift + 1).
    ///
    /// Uses the identity: 2^(2*shift) ≡ 1 (mod 2^shift + 1),
    /// so 2^(-k) ≡ 2^(2*shift - k).
    pub fn shift_right(&mut self, k: usize) {
        if k == 0 {
            return;
        }
        let two_shift = 2 * self.shift;
        let effective = (two_shift - (k % two_shift)) % two_shift;
        if effective > 0 {
            // Multiply by 2^effective which equals 2^(-k) mod (2^shift + 1)
            let modulus = self.modulus();
            let val = self.to_biguint();
            let shifted = (val << effective) % modulus;
            *self = Self::from_biguint(&shifted, self.shift);
        }
    }

    /// Check if this is zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.data.iter().all(|&x| x == 0)
    }
}

/// Select optimal FFT parameters for multiplying two numbers.
///
/// Returns `(piece_bits, n, fermat_shift)` where:
/// - `piece_bits`: number of bits per polynomial piece
/// - `n`: transform length (power of 2)
/// - `fermat_shift`: Fermat modulus parameter (`2^fermat_shift` + 1)
#[must_use]
pub fn select_fft_params(a_bits: usize, b_bits: usize) -> (usize, usize, usize) {
    let max_bits = a_bits.max(b_bits);

    // Choose piece_bits based on operand size
    let piece_bits = if max_bits < 10_000 {
        64
    } else if max_bits < 100_000 {
        256
    } else if max_bits < 1_000_000 {
        1024
    } else {
        4096
    };

    // Number of pieces for each operand
    let n_a = a_bits.div_ceil(piece_bits);
    let n_b = b_bits.div_ceil(piece_bits);

    // Transform length: power of 2, >= n_a + n_b (avoids aliasing in cyclic convolution)
    let n = (n_a + n_b).max(4).next_power_of_two();

    // Fermat shift must be:
    // 1. > 2*piece_bits + log2(n) + 1 (so coefficients fit without overflow)
    // 2. A multiple of n/2 (so 2*fermat_shift/size is integer for all butterfly levels)
    let log_n = (n as f64).log2().ceil() as usize;
    let min_shift = 2 * piece_bits + log_n + 2;
    let half_n = n / 2;
    let fermat_shift = min_shift.div_ceil(half_n) * half_n;

    (piece_bits, n, fermat_shift)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fermat_new() {
        let f = FermatNum::new(64);
        assert_eq!(f.shift, 64);
        assert!(!f.data.is_empty());
    }

    #[test]
    fn fermat_modulus() {
        let f = FermatNum::new(64);
        let modulus = f.modulus();
        // 2^64 + 1
        let expected = (BigUint::one() << 64) + BigUint::one();
        assert_eq!(modulus, expected);
    }

    #[test]
    fn fermat_to_from_biguint_roundtrip() {
        let val = BigUint::from(123_456_789u64);
        let f = FermatNum::from_biguint(&val, 128);
        assert_eq!(f.to_biguint(), val);
    }

    #[test]
    fn fermat_to_from_large_value() {
        // Test with a value that spans multiple u64 limbs
        let val = (BigUint::one() << 100) + BigUint::from(42u64);
        let f = FermatNum::from_biguint(&val, 128);
        assert_eq!(f.to_biguint(), val);
    }

    #[test]
    fn fermat_add_mod() {
        let a = FermatNum::from_biguint(&BigUint::from(100u64), 64);
        let b = FermatNum::from_biguint(&BigUint::from(200u64), 64);
        let c = a.add(&b);
        assert_eq!(c.to_biguint(), BigUint::from(300u64));
    }

    #[test]
    fn fermat_add_wraps() {
        // Test modular wrap-around: (2^64) + 1 mod (2^64 + 1) = 0
        let _modulus = (BigUint::one() << 64) + BigUint::one();
        let a = FermatNum::from_biguint(&(BigUint::one() << 64), 64);
        let b = FermatNum::from_biguint(&BigUint::one(), 64);
        let c = a.add(&b);
        assert_eq!(c.to_biguint(), BigUint::from(0u64));
    }

    #[test]
    fn fermat_sub() {
        let a = FermatNum::from_biguint(&BigUint::from(300u64), 64);
        let b = FermatNum::from_biguint(&BigUint::from(200u64), 64);
        let c = a.sub(&b);
        assert_eq!(c.to_biguint(), BigUint::from(100u64));
    }

    #[test]
    fn fermat_sub_wraps() {
        // 100 - 200 mod (2^64 + 1) = modulus - 100
        let modulus = (BigUint::one() << 64) + BigUint::one();
        let a = FermatNum::from_biguint(&BigUint::from(100u64), 64);
        let b = FermatNum::from_biguint(&BigUint::from(200u64), 64);
        let c = a.sub(&b);
        let expected = &modulus - BigUint::from(100u64);
        assert_eq!(c.to_biguint(), expected);
    }

    #[test]
    fn fermat_mul() {
        let a = FermatNum::from_biguint(&BigUint::from(100u64), 128);
        let b = FermatNum::from_biguint(&BigUint::from(200u64), 128);
        let c = a.fermat_mul(&b);
        assert_eq!(c.to_biguint(), BigUint::from(20000u64));
    }

    #[test]
    fn fermat_shift_left() {
        let mut a = FermatNum::from_biguint(&BigUint::one(), 128);
        a.shift_left(10);
        assert_eq!(a.to_biguint(), BigUint::from(1024u64));
    }

    #[test]
    fn fermat_shift_left_wraps() {
        // 2^64 mod (2^64 + 1) = modulus - 1 = 2^64
        // But 1 << 64 in the Fermat ring: 2^64 ≡ -1 mod (2^64 + 1)
        let modulus = (BigUint::one() << 64) + BigUint::one();
        let mut a = FermatNum::from_biguint(&BigUint::one(), 64);
        a.shift_left(64);
        // 2^64 mod (2^64 + 1) = 2^64 (which is modulus - 1)
        assert_eq!(a.to_biguint(), &modulus - BigUint::one());
    }

    #[test]
    fn fermat_shift_right_inverse_of_left() {
        let original = BigUint::from(12345u64);
        let mut a = FermatNum::from_biguint(&original, 128);
        a.shift_left(20);
        a.shift_right(20);
        assert_eq!(a.to_biguint(), original);
    }

    #[test]
    fn fermat_normalize() {
        // Create a value larger than the modulus
        let modulus = (BigUint::one() << 64) + BigUint::one();
        let val = &modulus + BigUint::from(5u64);
        let mut f = FermatNum::from_biguint(&val, 64);
        f.normalize();
        assert_eq!(f.to_biguint(), BigUint::from(5u64));
    }

    #[test]
    fn select_params_small() {
        let (piece_bits, n, shift) = select_fft_params(1000, 1000);
        assert!(piece_bits > 0);
        assert!(n > 0);
        assert!(shift > 0);
        // n must be power of 2
        assert_eq!(n & (n - 1), 0);
        // shift must be divisible by n/2
        assert_eq!(shift % (n / 2), 0);
    }

    #[test]
    fn select_params_large() {
        let (piece_bits, n, shift) = select_fft_params(100_000, 100_000);
        assert!(piece_bits > 0);
        assert!(n > 0);
        // n must be power of 2
        assert_eq!(n & (n - 1), 0);
        // shift must be divisible by n/2
        assert_eq!(shift % (n / 2), 0);
        // shift must be large enough for convolution coefficients
        assert!(shift >= 2 * piece_bits + 1);
    }
}
