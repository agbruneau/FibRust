//! Criterion benchmarks for Fibonacci algorithms.

use std::sync::Arc;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use num_bigint::BigUint;

use fibcalc_core::calculator::{Calculator, FibCalculator};
use fibcalc_core::fastdoubling::OptimizedFastDoubling;
use fibcalc_core::fft_based::FFTBasedCalculator;
use fibcalc_core::matrix::MatrixExponentiation;
use fibcalc_core::observers::NoOpObserver;
use fibcalc_core::options::Options;
use fibcalc_core::progress::CancellationToken;

fn compute(calc: &dyn Calculator, n: u64) -> BigUint {
    let cancel = CancellationToken::new();
    let observer = NoOpObserver::new();
    let opts = Options::default().normalize();
    calc.calculate(&cancel, &observer, 0, n, &opts).unwrap()
}

fn bench_algorithms(c: &mut Criterion) {
    let fast: Arc<dyn Calculator> =
        Arc::new(FibCalculator::new(Arc::new(OptimizedFastDoubling::new())));
    let matrix: Arc<dyn Calculator> =
        Arc::new(FibCalculator::new(Arc::new(MatrixExponentiation::new())));
    let fft: Arc<dyn Calculator> =
        Arc::new(FibCalculator::new(Arc::new(FFTBasedCalculator::new())));

    let ns: Vec<u64> = vec![100, 1_000, 10_000, 100_000];

    let mut group = c.benchmark_group("FastDoubling");
    for &n in &ns {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| compute(fast.as_ref(), n));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("MatrixExponentiation");
    for &n in &ns {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| compute(matrix.as_ref(), n));
        });
    }
    group.finish();

    let mut group = c.benchmark_group("FFTBased");
    for &n in &ns {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| compute(fft.as_ref(), n));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_algorithms);
criterion_main!(benches);
