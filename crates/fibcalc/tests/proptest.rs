//! Property-based tests for Fibonacci algorithms.

use std::sync::Arc;

use num_bigint::BigUint;
use proptest::prelude::*;

use fibcalc_core::calculator::{Calculator, FibCalculator};
use fibcalc_core::fastdoubling::OptimizedFastDoubling;
use fibcalc_core::fft_based::FFTBasedCalculator;
use fibcalc_core::matrix::MatrixExponentiation;
use fibcalc_core::observers::NoOpObserver;
use fibcalc_core::options::Options;
use fibcalc_core::progress::CancellationToken;

fn compute(algo: &str, n: u64) -> BigUint {
    let calc: Arc<dyn Calculator> = match algo {
        "fast" => Arc::new(FibCalculator::new(Arc::new(OptimizedFastDoubling::new()))),
        "matrix" => Arc::new(FibCalculator::new(Arc::new(MatrixExponentiation::new()))),
        "fft" => Arc::new(FibCalculator::new(Arc::new(FFTBasedCalculator::new()))),
        _ => panic!("Unknown algorithm"),
    };
    let cancel = CancellationToken::new();
    let observer = NoOpObserver::new();
    let opts = Options::default().normalize();
    calc.calculate(&cancel, &observer, 0, n, &opts).unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Fast Doubling and Matrix produce the same result for random n.
    #[test]
    fn fast_equals_matrix(n in 94u64..2000) {
        let fast = compute("fast", n);
        let matrix = compute("matrix", n);
        prop_assert_eq!(fast, matrix, "F({}) fast != matrix", n);
    }

    /// Fast Doubling and FFT produce the same result for random n.
    #[test]
    fn fast_equals_fft(n in 94u64..2000) {
        let fast = compute("fast", n);
        let fft = compute("fft", n);
        prop_assert_eq!(fast, fft, "F({}) fast != fft", n);
    }

    /// F(n) + F(n+1) == F(n+2) for random n.
    #[test]
    fn fibonacci_recurrence(n in 0u64..1000) {
        let fn0 = compute("fast", n);
        let fn1 = compute("fast", n + 1);
        let fn2 = compute("fast", n + 2);
        prop_assert_eq!(&fn0 + &fn1, fn2, "F({}) + F({}) != F({})", n, n+1, n+2);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// F(n) mod 10^k matches FastDoublingMod for random n, k.
    #[test]
    fn modular_matches_full(n in 94u64..2000, k in 1u32..8) {
        let full = compute("fast", n);
        let modulus = BigUint::from(10u32).pow(k);
        let expected = &full % &modulus;

        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let result = fibcalc_core::modular::FastDoublingMod::fibonacci_mod(
            n, &modulus, &cancel, &observer, 0,
        ).unwrap();

        prop_assert_eq!(result, expected, "F({}) mod 10^{} mismatch", n, k);
    }
}

/// F(0) = 0, F(1) = 1 for all algorithms.
#[test]
fn base_cases_all_algos() {
    for algo in &["fast", "matrix", "fft"] {
        assert_eq!(compute(algo, 0), BigUint::from(0u32), "{algo} F(0)");
        assert_eq!(compute(algo, 1), BigUint::from(1u32), "{algo} F(1)");
    }
}

/// F(93) is the last value that fits in u64.
#[test]
fn boundary_93_all_algos() {
    let expected = BigUint::from(12_200_160_415_121_876_738u64);
    for algo in &["fast", "matrix", "fft"] {
        assert_eq!(compute(algo, 93), expected, "{algo} F(93)");
    }
}

/// F(94) is the first value requiring BigUint.
#[test]
fn boundary_94_all_algos() {
    let fast = compute("fast", 94);
    let matrix = compute("matrix", 94);
    let fft = compute("fft", 94);
    assert_eq!(fast, matrix, "F(94) fast != matrix");
    assert_eq!(fast, fft, "F(94) fast != fft");
    // F(94) = 19740274219868223167
    assert_eq!(fast.to_string(), "19740274219868223167");
}
