//! Fermat number arithmetic for FFT.
//!
//! Fermat numbers `F_k` = 2^(2^k) + 1 are used as moduli for
//! Number Theoretic Transform (NTT) based multiplication.
//!
//! Add and subtract operate directly on u64 limbs to avoid
//! heap-allocating BigUint conversions in hot loops.

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

    /// Normalize: reduce mod (2^shift + 1) via BigUint fallback.
    pub fn normalize(&mut self) {
        let modulus = self.modulus();
        let val = self.to_biguint() % &modulus;
        *self = Self::from_biguint(&val, self.shift);
    }

    /// Add two Fermat numbers mod (2^shift + 1).
    /// Uses in-place limb arithmetic instead of BigUint conversion.
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        assert_eq!(self.shift, other.shift);
        let n = self.data.len();
        let mut result = Self {
            data: vec![0u64; n],
            shift: self.shift,
        };

        // Limb-level addition with carry
        let mut carry = 0u64;
        for i in 0..n {
            let a = self.data[i];
            let b = if i < other.data.len() { other.data[i] } else { 0 };
            let (sum1, c1) = a.overflowing_add(b);
            let (sum2, c2) = sum1.overflowing_add(carry);
            result.data[i] = sum2;
            carry = u64::from(c1) + u64::from(c2);
        }

        // If carry or result >= modulus, reduce
        if carry > 0 || result.ge_modulus() {
            result.normalize();
        }
        result
    }

    /// Subtract other from self mod (2^shift + 1).
    /// Uses in-place limb arithmetic instead of BigUint conversion.
    #[must_use]
    pub fn sub(&self, other: &Self) -> Self {
        assert_eq!(self.shift, other.shift);
        let n = self.data.len();
        let mut result = Self {
            data: vec![0u64; n],
            shift: self.shift,
        };

        // Limb-level subtraction with borrow
        let mut borrow = 0u64;
        for i in 0..n {
            let a = self.data[i];
            let b = if i < other.data.len() { other.data[i] } else { 0 };
            let (diff1, b1) = a.overflowing_sub(b);
            let (diff2, b2) = diff1.overflowing_sub(borrow);
            result.data[i] = diff2;
            borrow = u64::from(b1) + u64::from(b2);
        }

        if borrow > 0 {
            // Result is negative: add modulus (2^shift + 1)
            result.add_modulus();
        }

        result
    }

    /// Check if self >= modulus (2^shift + 1).
    fn ge_modulus(&self) -> bool {
        let limb_idx = self.shift / 64;
        let bit_idx = self.shift % 64;

        // Check for any bits strictly above position shift
        for i in (limb_idx + 1)..self.data.len() {
            if self.data[i] != 0 {
                return true;
            }
        }

        if bit_idx == 0 {
            // Bit at position shift is in data[limb_idx] bit 0
            if limb_idx >= self.data.len() {
                return false;
            }
            if self.data[limb_idx] > 1 {
                return true;
            }
            if self.data[limb_idx] == 1 {
                // Value >= 2^shift. Check if also >= 2^shift + 1
                for i in 0..limb_idx {
                    if self.data[i] != 0 {
                        return true; // >= 2^shift + something
                    }
                }
            }
            false
        } else {
            if limb_idx >= self.data.len() {
                return false;
            }
            // Check if there are bits above bit_idx in this limb
            let above_mask = !((1u64 << (bit_idx + 1)) - 1);
            if self.data[limb_idx] & above_mask != 0 {
                return true;
            }
            // Check if bit at shift is set
            if self.data[limb_idx] & (1u64 << bit_idx) != 0 {
                // Value >= 2^shift. Check if also >= 2^shift + 1
                let low_mask = (1u64 << bit_idx) - 1;
                if self.data[limb_idx] & low_mask != 0 {
                    return true;
                }
                for i in 0..limb_idx {
                    if self.data[i] != 0 {
                        return true;
                    }
                }
            }
            false
        }
    }

    /// Add the modulus 2^shift + 1 to self.data.
    fn add_modulus(&mut self) {
        // Add 1
        let mut carry = 1u64;
        for limb in &mut self.data {
            let (sum, c) = limb.overflowing_add(carry);
            *limb = sum;
            carry = u64::from(c);
            if carry == 0 {
                break;
            }
        }

        // Add 2^shift
        let limb_idx = self.shift / 64;
        let bit_idx = self.shift % 64;
        if limb_idx < self.data.len() {
            let (sum, mut c) = self.data[limb_idx].overflowing_add(1u64 << bit_idx);
            self.data[limb_idx] = sum;
            let mut i = limb_idx + 1;
            while c && i < self.data.len() {
                let (s, c2) = self.data[i].overflowing_add(1);
                self.data[i] = s;
                c = c2;
                i += 1;
            }
        }
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
    pub fn shift_right(&mut self, k: usize) {
        if k == 0 {
            return;
        }
        let two_shift = 2 * self.shift;
        let effective = (two_shift - (k % two_shift)) % two_shift;
        if effective > 0 {
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
#[must_use]
pub fn select_fft_params(a_bits: usize, b_bits: usize) -> (usize, usize, usize) {
    let max_bits = a_bits.max(b_bits);

    let piece_bits = if max_bits < 10_000 {
        64
    } else if max_bits < 100_000 {
        256
    } else if max_bits < 1_000_000 {
        1024
    } else {
        4096
    };

    let n_a = a_bits.div_ceil(piece_bits);
    let n_b = b_bits.div_ceil(piece_bits);
    let n = (n_a + n_b).max(4).next_power_of_two();

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
        let modulus = (BigUint::one() << 64) + BigUint::one();
        let mut a = FermatNum::from_biguint(&BigUint::one(), 64);
        a.shift_left(64);
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
        assert_eq!(n & (n - 1), 0);
        assert_eq!(shift % (n / 2), 0);
    }

    #[test]
    fn select_params_large() {
        let (piece_bits, n, shift) = select_fft_params(100_000, 100_000);
        assert!(piece_bits > 0);
        assert!(n > 0);
        assert_eq!(n & (n - 1), 0);
        assert_eq!(shift % (n / 2), 0);
        assert!(shift >= 2 * piece_bits + 1);
    }
}
