//! Memory estimation for FFT operations.

use crate::fermat::select_fft_params;

/// Estimate memory usage for an FFT multiplication of two numbers.
#[must_use]
pub fn estimate_fft_memory(a_bits: usize, b_bits: usize) -> usize {
    let (_piece_bits, n, shift) = select_fft_params(a_bits, b_bits);
    let limbs_per_element = shift.div_ceil(64) + 1;
    let bytes_per_element = limbs_per_element * 8;

    // Two input polynomials + output + temporaries
    let poly_bytes = n * bytes_per_element * 4;
    // Root tables
    let root_bytes = n * 8 * 2;

    poly_bytes + root_bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_small() {
        let est = estimate_fft_memory(1000, 1000);
        assert!(est > 0);
    }

    #[test]
    fn estimate_grows_with_size() {
        let small = estimate_fft_memory(1000, 1000);
        let large = estimate_fft_memory(1_000_000, 1_000_000);
        assert!(large > small);
    }
}
