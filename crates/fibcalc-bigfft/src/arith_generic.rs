//! Portable arithmetic operations.
#![allow(dead_code)] // Infrastructure: available for optimized limb-level arithmetic

/// Add with carry: a + b + carry -> (sum, `new_carry`)
#[inline]
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn add_with_carry(a: u64, b: u64, carry: u64) -> (u64, u64) {
    let sum = u128::from(a) + u128::from(b) + u128::from(carry);
    (sum as u64, (sum >> 64) as u64)
}

/// Subtract with borrow: a - b - borrow -> (diff, `new_borrow`)
#[inline]
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn sub_with_borrow(a: u64, b: u64, borrow: u64) -> (u64, u64) {
    let diff = i128::from(a) - i128::from(b) - i128::from(borrow);
    if diff < 0 {
        ((diff + (1i128 << 64)) as u64, 1)
    } else {
        (diff as u64, 0)
    }
}

/// Multiply: a * b -> (low, high)
#[inline]
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn mul_wide(a: u64, b: u64) -> (u64, u64) {
    let prod = u128::from(a) * u128::from(b);
    (prod as u64, (prod >> 64) as u64)
}

/// Add a slice of u64 values with a scalar, returning carry.
pub fn add_scalar(data: &mut [u64], scalar: u64) -> u64 {
    let mut carry = scalar;
    for limb in data.iter_mut() {
        let (sum, c) = add_with_carry(*limb, carry, 0);
        *limb = sum;
        carry = c;
        if carry == 0 {
            break;
        }
    }
    carry
}

/// Subtract a scalar from a slice of u64 values, returning borrow.
pub fn sub_scalar(data: &mut [u64], scalar: u64) -> u64 {
    let mut borrow = scalar;
    for limb in data.iter_mut() {
        let (diff, b) = sub_with_borrow(*limb, borrow, 0);
        *limb = diff;
        borrow = b;
        if borrow == 0 {
            break;
        }
    }
    borrow
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_carry() {
        let (sum, carry) = add_with_carry(u64::MAX, 1, 0);
        assert_eq!(sum, 0);
        assert_eq!(carry, 1);
    }

    #[test]
    fn sub_borrow() {
        let (diff, borrow) = sub_with_borrow(0, 1, 0);
        assert_eq!(diff, u64::MAX);
        assert_eq!(borrow, 1);
    }

    #[test]
    fn multiply_wide() {
        let (low, high) = mul_wide(u64::MAX, 2);
        assert_eq!(low, u64::MAX - 1);
        assert_eq!(high, 1);
    }

    // --- add_with_carry additional tests ---

    #[test]
    fn add_carry_zero_plus_zero() {
        let (sum, carry) = add_with_carry(0, 0, 0);
        assert_eq!(sum, 0);
        assert_eq!(carry, 0);
    }

    #[test]
    fn add_carry_with_carry_in() {
        let (sum, carry) = add_with_carry(u64::MAX, 0, 1);
        assert_eq!(sum, 0);
        assert_eq!(carry, 1);
    }

    #[test]
    fn add_carry_max_plus_max() {
        let (sum, carry) = add_with_carry(u64::MAX, u64::MAX, 0);
        assert_eq!(sum, u64::MAX - 1);
        assert_eq!(carry, 1);
    }

    #[test]
    fn add_carry_max_plus_max_plus_carry() {
        let (sum, carry) = add_with_carry(u64::MAX, u64::MAX, 1);
        assert_eq!(sum, u64::MAX);
        assert_eq!(carry, 1);
    }

    #[test]
    fn add_carry_no_overflow() {
        let (sum, carry) = add_with_carry(100, 200, 0);
        assert_eq!(sum, 300);
        assert_eq!(carry, 0);
    }

    #[test]
    fn add_carry_half_max() {
        let half = u64::MAX / 2;
        let (sum, carry) = add_with_carry(half, half, 0);
        assert_eq!(sum, u64::MAX - 1);
        assert_eq!(carry, 0);
    }

    // --- sub_with_borrow additional tests ---

    #[test]
    fn sub_borrow_zero_minus_zero() {
        let (diff, borrow) = sub_with_borrow(0, 0, 0);
        assert_eq!(diff, 0);
        assert_eq!(borrow, 0);
    }

    #[test]
    fn sub_borrow_with_borrow_in() {
        let (diff, borrow) = sub_with_borrow(0, 0, 1);
        assert_eq!(diff, u64::MAX);
        assert_eq!(borrow, 1);
    }

    #[test]
    fn sub_borrow_no_underflow() {
        let (diff, borrow) = sub_with_borrow(500, 200, 0);
        assert_eq!(diff, 300);
        assert_eq!(borrow, 0);
    }

    #[test]
    fn sub_borrow_exact() {
        let (diff, borrow) = sub_with_borrow(100, 100, 0);
        assert_eq!(diff, 0);
        assert_eq!(borrow, 0);
    }

    #[test]
    fn sub_borrow_max_minus_max() {
        let (diff, borrow) = sub_with_borrow(u64::MAX, u64::MAX, 0);
        assert_eq!(diff, 0);
        assert_eq!(borrow, 0);
    }

    #[test]
    fn sub_borrow_underflow_with_borrow() {
        let (diff, borrow) = sub_with_borrow(100, 100, 1);
        assert_eq!(diff, u64::MAX);
        assert_eq!(borrow, 1);
    }

    #[test]
    fn sub_borrow_one_minus_max() {
        let (diff, borrow) = sub_with_borrow(1, u64::MAX, 0);
        assert_eq!(diff, 2);
        assert_eq!(borrow, 1);
    }

    // --- mul_wide additional tests ---

    #[test]
    fn mul_wide_zero() {
        let (low, high) = mul_wide(0, 0);
        assert_eq!(low, 0);
        assert_eq!(high, 0);
    }

    #[test]
    fn mul_wide_one() {
        let (low, high) = mul_wide(1, 1);
        assert_eq!(low, 1);
        assert_eq!(high, 0);
    }

    #[test]
    fn mul_wide_by_zero() {
        let (low, high) = mul_wide(u64::MAX, 0);
        assert_eq!(low, 0);
        assert_eq!(high, 0);
    }

    #[test]
    fn mul_wide_max_times_max() {
        let (low, high) = mul_wide(u64::MAX, u64::MAX);
        // u64::MAX * u64::MAX = (2^64 - 1)^2 = 2^128 - 2*2^64 + 1
        // high = 2^64 - 2 = u64::MAX - 1
        // low  = 1
        assert_eq!(low, 1);
        assert_eq!(high, u64::MAX - 1);
    }

    #[test]
    fn mul_wide_identity() {
        let (low, high) = mul_wide(12345, 1);
        assert_eq!(low, 12345);
        assert_eq!(high, 0);
    }

    #[test]
    fn mul_wide_power_of_two() {
        let (low, high) = mul_wide(1u64 << 32, 1u64 << 32);
        assert_eq!(low, 0);
        assert_eq!(high, 1);
    }

    // --- add_scalar tests ---

    #[test]
    fn add_scalar_no_carry() {
        let mut data = vec![10, 20, 30];
        let carry = add_scalar(&mut data, 5);
        assert_eq!(data, vec![15, 20, 30]);
        assert_eq!(carry, 0);
    }

    #[test]
    fn add_scalar_with_propagation() {
        let mut data = vec![u64::MAX, 0, 0];
        let carry = add_scalar(&mut data, 1);
        assert_eq!(data, vec![0, 1, 0]);
        assert_eq!(carry, 0);
    }

    #[test]
    fn add_scalar_carry_propagates_through_all() {
        let mut data = vec![u64::MAX, u64::MAX, u64::MAX];
        let carry = add_scalar(&mut data, 1);
        assert_eq!(data, vec![0, 0, 0]);
        assert_eq!(carry, 1);
    }

    #[test]
    fn add_scalar_zero() {
        let mut data = vec![100, 200, 300];
        let carry = add_scalar(&mut data, 0);
        assert_eq!(data, vec![100, 200, 300]);
        assert_eq!(carry, 0);
    }

    #[test]
    fn add_scalar_empty() {
        let mut data: Vec<u64> = vec![];
        let carry = add_scalar(&mut data, 42);
        assert_eq!(carry, 42);
    }

    // --- sub_scalar tests ---

    #[test]
    fn sub_scalar_no_borrow() {
        let mut data = vec![100, 200, 300];
        let borrow = sub_scalar(&mut data, 50);
        assert_eq!(data, vec![50, 200, 300]);
        assert_eq!(borrow, 0);
    }

    #[test]
    fn sub_scalar_with_propagation() {
        let mut data = vec![0, 1, 0];
        let borrow = sub_scalar(&mut data, 1);
        assert_eq!(data, vec![u64::MAX, 0, 0]);
        assert_eq!(borrow, 0);
    }

    #[test]
    fn sub_scalar_borrow_propagates_through_all() {
        let mut data = vec![0, 0, 0];
        let borrow = sub_scalar(&mut data, 1);
        assert_eq!(data, vec![u64::MAX, u64::MAX, u64::MAX]);
        assert_eq!(borrow, 1);
    }

    #[test]
    fn sub_scalar_zero() {
        let mut data = vec![100, 200, 300];
        let borrow = sub_scalar(&mut data, 0);
        assert_eq!(data, vec![100, 200, 300]);
        assert_eq!(borrow, 0);
    }

    #[test]
    fn sub_scalar_empty() {
        let mut data: Vec<u64> = vec![];
        let borrow = sub_scalar(&mut data, 42);
        assert_eq!(borrow, 42);
    }

    #[test]
    fn sub_scalar_exact() {
        let mut data = vec![42, 0, 0];
        let borrow = sub_scalar(&mut data, 42);
        assert_eq!(data, vec![0, 0, 0]);
        assert_eq!(borrow, 0);
    }
}
