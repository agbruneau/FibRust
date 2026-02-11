//! Golden file integration tests.
//!
//! Reads tests/testdata/fibonacci_golden.json and verifies all 3 algorithms
//! produce the correct values for known Fibonacci numbers.

use std::sync::Arc;
use std::time::Duration;

use num_bigint::BigUint;
use serde::Deserialize;

use fibcalc_core::calculator::{Calculator, CoreCalculator, FibCalculator, FibError};
use fibcalc_core::fastdoubling::OptimizedFastDoubling;
use fibcalc_core::fft_based::FFTBasedCalculator;
use fibcalc_core::matrix::MatrixExponentiation;
use fibcalc_core::observers::NoOpObserver;
use fibcalc_core::options::Options;
use fibcalc_core::progress::CancellationToken;

// ---------------------------------------------------------------------------
// Golden data structures
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GoldenData {
    #[allow(dead_code)]
    description: String,
    values: Vec<GoldenEntry>,
}

#[derive(Deserialize)]
struct GoldenEntry {
    n: u64,
    #[serde(default)]
    fib: Option<String>,
    #[serde(default)]
    fib_prefix: Option<String>,
    #[serde(default)]
    fib_digits: Option<usize>,
}

fn load_golden_data() -> GoldenData {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/testdata/fibonacci_golden.json"
    );
    let data = std::fs::read_to_string(path).expect("failed to read golden file");
    serde_json::from_str(&data).expect("failed to parse golden JSON")
}

// ---------------------------------------------------------------------------
// Helpers — compute F(n) via different routes
// ---------------------------------------------------------------------------

fn compute_with_core(core: &dyn CoreCalculator, n: u64) -> BigUint {
    let cancel = CancellationToken::new();
    let observer = NoOpObserver::new();
    let opts = Options::default();
    core.calculate_core(&cancel, &observer, 0, n, &opts)
        .unwrap()
}

fn compute_with_calculator(calc: &dyn Calculator, n: u64) -> BigUint {
    let cancel = CancellationToken::new();
    let observer = NoOpObserver::new();
    let opts = Options::default();
    calc.calculate(&cancel, &observer, 0, n, &opts).unwrap()
}

// ---------------------------------------------------------------------------
// Golden: exact values — all 3 core algorithms
// ---------------------------------------------------------------------------

#[test]
fn golden_exact_fast_doubling() {
    let algo = OptimizedFastDoubling::new();
    let data = load_golden_data();
    for entry in &data.values {
        if let Some(expected) = &entry.fib {
            let result = compute_with_core(&algo, entry.n);
            assert_eq!(
                result.to_string(),
                *expected,
                "FastDoubling mismatch at n={}",
                entry.n,
            );
        }
    }
}

#[test]
fn golden_exact_matrix() {
    let algo = MatrixExponentiation::new();
    let data = load_golden_data();
    for entry in &data.values {
        if let Some(expected) = &entry.fib {
            let result = compute_with_core(&algo, entry.n);
            assert_eq!(
                result.to_string(),
                *expected,
                "Matrix mismatch at n={}",
                entry.n,
            );
        }
    }
}

#[test]
fn golden_exact_fft_based() {
    let algo = FFTBasedCalculator::new();
    let data = load_golden_data();
    for entry in &data.values {
        if let Some(expected) = &entry.fib {
            let result = compute_with_core(&algo, entry.n);
            assert_eq!(
                result.to_string(),
                *expected,
                "FFTBased mismatch at n={}",
                entry.n,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Golden: prefix & digit count (n=5000, n=10000)
// ---------------------------------------------------------------------------

#[test]
fn golden_prefix_and_digits() {
    let algos: Vec<(&str, Box<dyn CoreCalculator>)> = vec![
        ("FastDoubling", Box::new(OptimizedFastDoubling::new())),
        ("Matrix", Box::new(MatrixExponentiation::new())),
        ("FFTBased", Box::new(FFTBasedCalculator::new())),
    ];

    let data = load_golden_data();
    for entry in &data.values {
        // Only test prefix/digit entries up to n=10000 (fast enough)
        if entry.n > 10_000 {
            continue;
        }

        if let Some(prefix) = &entry.fib_prefix {
            for (name, algo) in &algos {
                let result = compute_with_core(algo.as_ref(), entry.n);
                let s = result.to_string();
                assert!(
                    s.starts_with(prefix.as_str()),
                    "{name} prefix mismatch at n={}: expected starts_with '{}', got '{}'",
                    entry.n,
                    prefix,
                    &s[..prefix.len().min(s.len())],
                );
            }
        }

        if let Some(expected_digits) = entry.fib_digits {
            for (name, algo) in &algos {
                let result = compute_with_core(algo.as_ref(), entry.n);
                let s = result.to_string();
                assert_eq!(
                    s.len(),
                    expected_digits,
                    "{name} digit count mismatch at n={}: expected {}, got {}",
                    entry.n,
                    expected_digits,
                    s.len(),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Golden: large n (slow — marked #[ignore])
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn golden_large_n_100000() {
    let algo = OptimizedFastDoubling::new();
    let data = load_golden_data();
    for entry in &data.values {
        if entry.n != 100_000 {
            continue;
        }
        let result = compute_with_core(&algo, entry.n);
        let s = result.to_string();
        if let Some(prefix) = &entry.fib_prefix {
            assert!(
                s.starts_with(prefix.as_str()),
                "prefix mismatch for n=100000"
            );
        }
        if let Some(expected_digits) = entry.fib_digits {
            assert_eq!(
                s.len(),
                expected_digits,
                "digit count mismatch for n=100000"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Golden: FibCalculator decorator (adds fast path for n <= 93)
// ---------------------------------------------------------------------------

#[test]
fn golden_via_fib_calculator_decorator() {
    let data = load_golden_data();
    let fast_calc = FibCalculator::new(Arc::new(OptimizedFastDoubling::new()));
    let matrix_calc = FibCalculator::new(Arc::new(MatrixExponentiation::new()));
    let fft_calc = FibCalculator::new(Arc::new(FFTBasedCalculator::new()));

    for entry in &data.values {
        if let Some(expected) = &entry.fib {
            let calcs: &[(&str, &dyn Calculator)] = &[
                ("FastDoubling", &fast_calc),
                ("Matrix", &matrix_calc),
                ("FFTBased", &fft_calc),
            ];
            for (name, calc) in calcs {
                let result = compute_with_calculator(*calc, entry.n);
                assert_eq!(
                    result.to_string(),
                    *expected,
                    "{name} (via FibCalculator) mismatch at n={}",
                    entry.n,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Edge cases: boundary values
// ---------------------------------------------------------------------------

#[test]
fn edge_case_n0() {
    let algos: Vec<Box<dyn CoreCalculator>> = vec![
        Box::new(OptimizedFastDoubling::new()),
        Box::new(MatrixExponentiation::new()),
        Box::new(FFTBasedCalculator::new()),
    ];
    for algo in &algos {
        let result = compute_with_core(algo.as_ref(), 0);
        assert_eq!(result, BigUint::ZERO, "{} F(0) != 0", algo.name());
    }
}

#[test]
fn edge_case_n1() {
    let algos: Vec<Box<dyn CoreCalculator>> = vec![
        Box::new(OptimizedFastDoubling::new()),
        Box::new(MatrixExponentiation::new()),
        Box::new(FFTBasedCalculator::new()),
    ];
    for algo in &algos {
        let result = compute_with_core(algo.as_ref(), 1);
        assert_eq!(result, BigUint::from(1u64), "{} F(1) != 1", algo.name());
    }
}

#[test]
fn edge_case_n2() {
    let algos: Vec<Box<dyn CoreCalculator>> = vec![
        Box::new(OptimizedFastDoubling::new()),
        Box::new(MatrixExponentiation::new()),
        Box::new(FFTBasedCalculator::new()),
    ];
    for algo in &algos {
        let result = compute_with_core(algo.as_ref(), 2);
        assert_eq!(result, BigUint::from(1u64), "{} F(2) != 1", algo.name());
    }
}

#[test]
fn edge_case_n93_fast_path_boundary() {
    // n=93 is the last value that fits in u64 (fast path boundary)
    let fast_calc = FibCalculator::new(Arc::new(OptimizedFastDoubling::new()));
    let result = compute_with_calculator(&fast_calc, 93);
    assert_eq!(result, BigUint::from(12_200_160_415_121_876_738u64));
}

#[test]
fn edge_case_n94_first_big_number() {
    // n=94 is the first value requiring BigUint computation
    let algos: Vec<Box<dyn CoreCalculator>> = vec![
        Box::new(OptimizedFastDoubling::new()),
        Box::new(MatrixExponentiation::new()),
        Box::new(FFTBasedCalculator::new()),
    ];
    let expected = BigUint::parse_bytes(b"19740274219868223167", 10).unwrap();
    for algo in &algos {
        let result = compute_with_core(algo.as_ref(), 94);
        assert_eq!(result, expected, "{} F(94) mismatch", algo.name());
    }

    // Also test via FibCalculator (ensures decorator routes to core for n>93)
    let calc = FibCalculator::new(Arc::new(OptimizedFastDoubling::new()));
    let result = compute_with_calculator(&calc, 94);
    assert_eq!(result, expected, "FibCalculator F(94) mismatch");
}

// ---------------------------------------------------------------------------
// Edge case: large n with timeout
// ---------------------------------------------------------------------------

#[test]
fn edge_case_large_n_with_timeout() {
    // n=100000 should complete within 30 seconds
    let algo = OptimizedFastDoubling::new();
    let cancel = CancellationToken::new();
    let observer = NoOpObserver::new();
    let opts = Options::default();

    let cancel_clone = cancel.clone();
    let timeout_handle = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(30));
        cancel_clone.cancel();
    });

    let result = algo.calculate_core(&cancel, &observer, 0, 100_000, &opts);
    assert!(result.is_ok(), "F(100000) computation failed or timed out");

    let s = result.unwrap().to_string();
    assert_eq!(s.len(), 20899, "F(100000) should have 20899 digits");

    // Clean up - cancel the timeout thread so it doesn't hang
    cancel.cancel();
    let _ = timeout_handle.join();
}

// ---------------------------------------------------------------------------
// Edge case: cancellation
// ---------------------------------------------------------------------------

#[test]
fn edge_case_cancellation_fast_doubling() {
    let algo = OptimizedFastDoubling::new();
    let cancel = CancellationToken::new();
    cancel.cancel(); // Cancel immediately
    let observer = NoOpObserver::new();
    let opts = Options::default();
    let result = algo.calculate_core(&cancel, &observer, 0, 10_000, &opts);
    assert!(matches!(result, Err(FibError::Cancelled)));
}

#[test]
fn edge_case_cancellation_matrix() {
    let algo = MatrixExponentiation::new();
    let cancel = CancellationToken::new();
    cancel.cancel();
    let observer = NoOpObserver::new();
    let opts = Options::default();
    let result = algo.calculate_core(&cancel, &observer, 0, 10_000, &opts);
    assert!(matches!(result, Err(FibError::Cancelled)));
}

#[test]
fn edge_case_cancellation_fft() {
    let algo = FFTBasedCalculator::new();
    let cancel = CancellationToken::new();
    cancel.cancel();
    let observer = NoOpObserver::new();
    let opts = Options::default();
    let result = algo.calculate_core(&cancel, &observer, 0, 10_000, &opts);
    assert!(matches!(result, Err(FibError::Cancelled)));
}

#[test]
fn edge_case_cancellation_via_decorator() {
    let calc = FibCalculator::new(Arc::new(OptimizedFastDoubling::new()));
    let cancel = CancellationToken::new();
    cancel.cancel();
    let observer = NoOpObserver::new();
    let opts = Options::default();
    let result = calc.calculate(&cancel, &observer, 0, 10_000, &opts);
    assert!(matches!(result, Err(FibError::Cancelled)));
}

// ---------------------------------------------------------------------------
// Cross-algorithm agreement
// ---------------------------------------------------------------------------

#[test]
fn all_algorithms_agree_medium_values() {
    let fd = OptimizedFastDoubling::new();
    let mx = MatrixExponentiation::new();
    let fft = FFTBasedCalculator::new();

    for n in [94, 100, 200, 300, 500, 1000, 2000, 5000] {
        let fd_result = compute_with_core(&fd, n);
        let mx_result = compute_with_core(&mx, n);
        let fft_result = compute_with_core(&fft, n);

        assert_eq!(fd_result, mx_result, "FastDoubling != Matrix at n={n}");
        assert_eq!(fd_result, fft_result, "FastDoubling != FFT at n={n}");
    }
}

// ---------------------------------------------------------------------------
// Invalid config tests
// ---------------------------------------------------------------------------

#[test]
fn invalid_algorithm_name() {
    use fibcalc_core::CalculatorFactory;
    let factory = fibcalc_core::registry::DefaultFactory::new();
    let result = factory.get("nonexistent");
    assert!(result.is_err());
}
