//! Golden file integration tests.
//!
//! Verifies that all three Fibonacci algorithms produce correct results
//! against known values from tests/testdata/fibonacci_golden.json.

use std::str::FromStr;
use std::sync::Arc;

use num_bigint::BigUint;
use serde::Deserialize;

use fibcalc_core::calculator::{Calculator, FibCalculator};
use fibcalc_core::fastdoubling::OptimizedFastDoubling;
use fibcalc_core::fft_based::FFTBasedCalculator;
use fibcalc_core::matrix::MatrixExponentiation;
use fibcalc_core::observers::NoOpObserver;
use fibcalc_core::options::Options;
use fibcalc_core::progress::CancellationToken;

#[derive(Deserialize)]
struct GoldenData {
    values: Vec<GoldenEntry>,
}

#[derive(Deserialize)]
struct GoldenEntry {
    n: u64,
    fib: Option<String>,
    fib_prefix: Option<String>,
    fib_digits: Option<usize>,
}

fn load_golden() -> GoldenData {
    // Try workspace root path first, then crate-local path
    let data = std::fs::read_to_string("../../tests/testdata/fibonacci_golden.json")
        .or_else(|_| std::fs::read_to_string("tests/testdata/fibonacci_golden.json"))
        .expect("Failed to read golden file");
    serde_json::from_str(&data).expect("Failed to parse golden file")
}

fn default_opts() -> Options {
    Options::default().normalize()
}

fn make_calculator(algo: &str) -> Arc<dyn Calculator> {
    match algo {
        "fast" => Arc::new(FibCalculator::new(Arc::new(OptimizedFastDoubling::new()))),
        "matrix" => Arc::new(FibCalculator::new(Arc::new(MatrixExponentiation::new()))),
        "fft" => Arc::new(FibCalculator::new(Arc::new(FFTBasedCalculator::new()))),
        _ => panic!("Unknown algorithm: {algo}"),
    }
}

fn compute(calc: &dyn Calculator, n: u64) -> BigUint {
    let cancel = CancellationToken::new();
    let observer = NoOpObserver::new();
    let opts = default_opts();
    calc.calculate(&cancel, &observer, 0, n, &opts).unwrap()
}

#[test]
fn golden_fast_doubling_exact() {
    let golden = load_golden();
    let calc = make_calculator("fast");

    for entry in &golden.values {
        if let Some(ref expected) = entry.fib {
            let expected_val = BigUint::from_str(expected).unwrap();
            let result = compute(calc.as_ref(), entry.n);
            assert_eq!(result, expected_val, "FastDoubling F({}) mismatch", entry.n);
        }
    }
}

#[test]
fn golden_matrix_exact() {
    let golden = load_golden();
    let calc = make_calculator("matrix");

    for entry in &golden.values {
        if let Some(ref expected) = entry.fib {
            let expected_val = BigUint::from_str(expected).unwrap();
            let result = compute(calc.as_ref(), entry.n);
            assert_eq!(
                result, expected_val,
                "MatrixExponentiation F({}) mismatch",
                entry.n
            );
        }
    }
}

#[test]
fn golden_fft_exact() {
    let golden = load_golden();
    let calc = make_calculator("fft");

    for entry in &golden.values {
        if let Some(ref expected) = entry.fib {
            let expected_val = BigUint::from_str(expected).unwrap();
            let result = compute(calc.as_ref(), entry.n);
            assert_eq!(result, expected_val, "FFTBased F({}) mismatch", entry.n);
        }
    }
}

#[test]
fn golden_prefix_and_digits() {
    let golden = load_golden();
    let calc = make_calculator("fast");

    for entry in &golden.values {
        if entry.n > 10000 {
            continue; // Skip very large values for speed
        }
        if let Some(ref expected_prefix) = entry.fib_prefix {
            let result = compute(calc.as_ref(), entry.n);
            let result_str = result.to_string();
            assert!(
                result_str.starts_with(expected_prefix),
                "F({}) prefix mismatch: expected starts_with {}, got {}...",
                entry.n,
                expected_prefix,
                &result_str[..expected_prefix.len().min(result_str.len())]
            );
        }
        if let Some(expected_digits) = entry.fib_digits {
            let result = compute(calc.as_ref(), entry.n);
            let result_str = result.to_string();
            assert_eq!(
                result_str.len(),
                expected_digits,
                "F({}) digit count mismatch",
                entry.n
            );
        }
    }
}

#[test]
fn golden_cross_algorithm_consistency() {
    let golden = load_golden();
    let fast = make_calculator("fast");
    let matrix = make_calculator("matrix");
    let fft = make_calculator("fft");

    for entry in &golden.values {
        if entry.fib.is_none() {
            continue;
        }
        if entry.n > 1000 {
            continue; // Keep fast for CI
        }
        let fast_result = compute(fast.as_ref(), entry.n);
        let matrix_result = compute(matrix.as_ref(), entry.n);
        let fft_result = compute(fft.as_ref(), entry.n);

        assert_eq!(fast_result, matrix_result, "F({}) fast != matrix", entry.n);
        assert_eq!(fast_result, fft_result, "F({}) fast != fft", entry.n);
    }
}
