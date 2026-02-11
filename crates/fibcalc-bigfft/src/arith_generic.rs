//! Portable arithmetic operations.

/// Add with carry: a + b + carry -> (sum, `new_carry`)
#[inline]
#[must_use]
pub fn add_with_carry(a: u64, b: u64, carry: u64) -> (u64, u64) {
    let sum = u128::from(a) + u128::from(b) + u128::from(carry);
    (sum as u64, (sum >> 64) as u64)
}

/// Subtract with borrow: a - b - borrow -> (diff, `new_borrow`)
#[inline]
#[must_use]
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
}
