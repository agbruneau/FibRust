//! Property-based tests for core Fibonacci algorithms.
//!
//! These tests exercise the CoreCalculator trait directly (without
//! the FibCalculator decorator fast path).

use num_bigint::BigUint;
use proptest::prelude::*;

use fibcalc_core::calculator::CoreCalculator;
use fibcalc_core::fastdoubling::OptimizedFastDoubling;
use fibcalc_core::fft_based::FFTBasedCalculator;
use fibcalc_core::matrix::MatrixExponentiation;
use fibcalc_core::modular::FastDoublingMod;
use fibcalc_core::observers::NoOpObserver;
use fibcalc_core::options::Options;
use fibcalc_core::progress::CancellationToken;

fn compute_core(algo: &dyn CoreCalculator, n: u64) -> BigUint {
    let cancel = CancellationToken::new();
    let observer = NoOpObserver::new();
    let opts = Options::default();
    algo.calculate_core(&cancel, &observer, 0, n, &opts)
        .unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// For random n in [94..5000], all three core algorithms agree.
    #[test]
    fn all_core_algorithms_agree(n in 94u64..5000) {
        let fd = OptimizedFastDoubling::new();
        let mx = MatrixExponentiation::new();
        let fft = FFTBasedCalculator::new();

        let fd_result = compute_core(&fd, n);
        let mx_result = compute_core(&mx, n);
        let fft_result = compute_core(&fft, n);

        prop_assert_eq!(&fd_result, &mx_result, "FastDoubling != Matrix at n={}", n);
        prop_assert_eq!(&fd_result, &fft_result, "FastDoubling != FFT at n={}", n);
    }

    /// F(n) + F(n+1) == F(n+2) for random n.
    #[test]
    fn fibonacci_addition_property(n in 2u64..2000) {
        let algo = OptimizedFastDoubling::new();
        let fn_val = compute_core(&algo, n);
        let fn1_val = compute_core(&algo, n + 1);
        let fn2_val = compute_core(&algo, n + 2);
        prop_assert_eq!(&fn_val + &fn1_val, fn2_val, "F({}) + F({}) != F({})", n, n + 1, n + 2);
    }

    /// F(n) mod 10^k matches FastDoublingMod for random n, k.
    #[test]
    fn modular_matches_full_computation(n in 94u64..2000, k in 1u32..8) {
        let algo = OptimizedFastDoubling::new();
        let full = compute_core(&algo, n);
        let modulus = BigUint::from(10u32).pow(k);
        let expected = &full % &modulus;

        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let result = FastDoublingMod::fibonacci_mod(
            n, &modulus, &cancel, &observer, 0,
        ).unwrap();

        prop_assert_eq!(result, expected, "F({}) mod 10^{} mismatch", n, k);
    }
}
