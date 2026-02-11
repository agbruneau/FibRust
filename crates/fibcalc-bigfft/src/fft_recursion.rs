//! Recursive FFT implementation with parallelism.

use crate::fermat::FermatNum;

/// Recursive FFT with threshold-based parallelism.
pub fn fft_recursive(data: &mut [FermatNum], shift: usize, parallel_threshold: usize) {
    let n = data.len();
    if n <= 1 {
        return;
    }

    if n <= parallel_threshold {
        // Sequential base case
        fft_sequential(data, shift);
    } else {
        // Parallel recursive case
        // Split into even and odd elements
        // TODO: Implement parallel FFT recursion using rayon
        fft_sequential(data, shift);
    }
}

/// Sequential FFT (base case for recursion).
fn fft_sequential(data: &mut [FermatNum], shift: usize) {
    use crate::fft_core::fft_forward;
    fft_forward(data, shift);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fermat::FermatNum;
    use num_bigint::BigUint;

    #[test]
    fn placeholder() {
        assert!(true);
    }

    #[test]
    fn fft_recursive_empty() {
        let mut data: Vec<FermatNum> = vec![];
        fft_recursive(&mut data, 64, 4);
        assert!(data.is_empty());
    }

    #[test]
    fn fft_recursive_single_element() {
        let shift = 64;
        let mut data = vec![FermatNum::from_biguint(&BigUint::from(42u64), shift)];
        fft_recursive(&mut data, shift, 4);
        assert_eq!(data[0].to_biguint(), BigUint::from(42u64));
    }

    #[test]
    fn fft_recursive_below_threshold_roundtrip() {
        // n=4 elements, threshold=8 -> sequential path
        let shift = 8;
        let vals = [1u64, 2, 3, 4];
        let mut data: Vec<FermatNum> = vals
            .iter()
            .map(|&v| FermatNum::from_biguint(&BigUint::from(v), shift))
            .collect();

        let original: Vec<BigUint> = data.iter().map(|f| f.to_biguint()).collect();

        // Forward via recursive (sequential path)
        fft_recursive(&mut data, shift, 8);

        // Inverse via fft_core
        use crate::fft_core::fft_inverse;
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
    fn fft_recursive_above_threshold_roundtrip() {
        // n=4 elements, threshold=2 -> would go to parallel path (which currently falls back to sequential)
        let shift = 8;
        let vals = [5u64, 10, 15, 20];
        let mut data: Vec<FermatNum> = vals
            .iter()
            .map(|&v| FermatNum::from_biguint(&BigUint::from(v), shift))
            .collect();

        let original: Vec<BigUint> = data.iter().map(|f| f.to_biguint()).collect();

        fft_recursive(&mut data, shift, 2);

        use crate::fft_core::fft_inverse;
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
    fn fft_recursive_threshold_exactly_n() {
        // n == threshold -> sequential path
        let shift = 16;
        let vals = [10u64, 20, 30, 40, 50, 60, 70, 80];
        let n = vals.len();
        let mut data: Vec<FermatNum> = vals
            .iter()
            .map(|&v| FermatNum::from_biguint(&BigUint::from(v), shift))
            .collect();

        let original: Vec<BigUint> = data.iter().map(|f| f.to_biguint()).collect();

        fft_recursive(&mut data, shift, n);

        use crate::fft_core::fft_inverse;
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
    fn fft_recursive_threshold_zero() {
        // threshold=0 means always take the "parallel" branch
        let shift = 8;
        let vals = [7u64, 14, 21, 28];
        let mut data: Vec<FermatNum> = vals
            .iter()
            .map(|&v| FermatNum::from_biguint(&BigUint::from(v), shift))
            .collect();

        let original: Vec<BigUint> = data.iter().map(|f| f.to_biguint()).collect();

        fft_recursive(&mut data, shift, 0);

        use crate::fft_core::fft_inverse;
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
    fn fft_sequential_called_via_recursive() {
        // Verify the sequential path produces a valid transform
        let shift = 8;
        let mut data: Vec<FermatNum> = (0..4)
            .map(|v| FermatNum::from_biguint(&BigUint::from(v as u64 + 1), shift))
            .collect();

        // This should not panic
        fft_recursive(&mut data, shift, 100);

        // Verify data was transformed (not all identical to original)
        let transformed: Vec<BigUint> = data.iter().map(|f| f.to_biguint()).collect();
        // The sum element (index 0 after forward FFT) should be 1+2+3+4 = 10
        assert_eq!(transformed[0], BigUint::from(10u64));
    }
}
