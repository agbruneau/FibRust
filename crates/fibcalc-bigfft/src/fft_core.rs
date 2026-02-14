//! Core FFT transform: forward and inverse (NTT over Fermat ring).

use crate::fermat::FermatNum;

/// Perform forward NTT transform in-place over Z/(2^shift + 1).
///
/// Uses the Cooley-Tukey butterfly with twiddle factors that are powers of 2,
/// exploiting the fact that 2 is a root of unity modulo Fermat numbers.
pub fn fft_forward(data: &mut [FermatNum], shift: usize) {
    let n = data.len();
    if n <= 1 {
        return;
    }

    // Bit-reversal permutation
    bit_reverse_permutation(data);

    // Iterative Cooley-Tukey FFT
    let mut size = 2;
    while size <= n {
        let half = size / 2;
        // Root of unity for this level: ω_size = 2^(2*shift/size)
        // Twiddle step (in bits): how many bits to shift for each j increment
        let step = 2 * shift / size;

        for start in (0..n).step_by(size) {
            for j in 0..half {
                let s = step * j;
                // Split to get simultaneous mutable access to indices [start+j] and [start+j+half]
                let (lo, hi) = data.split_at_mut(start + j + half);
                let upper = &lo[start + j];
                let lower = &mut hi[0];
                lower.shift_left(s); // lower = data[start+j+half] * ω^j (in-place)

                let sum = upper.add(lower); // u + ω^j * t
                let diff = upper.sub(lower); // u - ω^j * t
                lo[start + j] = sum;
                hi[0] = diff;
            }
        }
        size *= 2;
    }
}

/// Perform inverse NTT transform in-place.
///
/// Standard technique: reverse elements (except first), apply forward FFT,
/// then divide each element by n.
pub fn fft_inverse(data: &mut [FermatNum], shift: usize) {
    let n = data.len();
    if n <= 1 {
        return;
    }

    // Step 1: Reverse elements at indices 1..n-1
    data[1..].reverse();

    // Step 2: Apply forward FFT
    fft_forward(data, shift);

    // Step 3: Divide each element by n = 2^log2(n)
    let log_n = n.trailing_zeros() as usize;
    for elem in data.iter_mut() {
        elem.shift_right(log_n);
    }
}

/// Bit-reversal permutation.
fn bit_reverse_permutation(data: &mut [FermatNum]) {
    let n = data.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            data.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;

    #[test]
    fn fft_roundtrip_identity() {
        // Forward then inverse should recover the original data.
        // Use n=4, shift=8 (modulus = 257, a Fermat prime)
        let shift = 8;
        let vals = [1u64, 2, 3, 4];
        let mut data: Vec<FermatNum> = vals
            .iter()
            .map(|&v| FermatNum::from_biguint(&BigUint::from(v), shift))
            .collect();

        let original: Vec<BigUint> = data.iter().map(|f| f.to_biguint()).collect();

        fft_forward(&mut data, shift);
        fft_inverse(&mut data, shift);

        for (i, (got, expected)) in data
            .iter()
            .map(|f| f.to_biguint())
            .zip(original.iter())
            .enumerate()
        {
            assert_eq!(&got, expected, "Mismatch at index {i}");
        }
    }

    #[test]
    fn fft_roundtrip_8_elements() {
        // Test with n=8 elements
        let shift = 16; // Must be divisible by n/2 = 4
        let vals = [10u64, 20, 30, 40, 50, 60, 70, 80];
        let mut data: Vec<FermatNum> = vals
            .iter()
            .map(|&v| FermatNum::from_biguint(&BigUint::from(v), shift))
            .collect();

        let original: Vec<BigUint> = data.iter().map(|f| f.to_biguint()).collect();

        fft_forward(&mut data, shift);
        fft_inverse(&mut data, shift);

        for (i, (got, expected)) in data
            .iter()
            .map(|f| f.to_biguint())
            .zip(original.iter())
            .enumerate()
        {
            assert_eq!(&got, expected, "Mismatch at index {i}");
        }
    }

    #[test]
    fn fft_single_element() {
        let shift = 64;
        let mut data = vec![FermatNum::from_biguint(&BigUint::from(42u64), shift)];
        fft_forward(&mut data, shift);
        assert_eq!(data[0].to_biguint(), BigUint::from(42u64));
    }
}
