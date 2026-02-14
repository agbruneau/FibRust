//! Micro-benchmarks for calibration.

use std::time::Duration;

use num_bigint::BigUint;

use crate::runner::{benchmark, benchmark_detailed, BenchmarkResult};

/// Benchmark Karatsuba multiplication at a given bit length.
#[must_use]
pub fn bench_karatsuba(bit_length: usize) -> Duration {
    let a = make_number(bit_length);
    let b = make_number(bit_length);
    benchmark(10, || {
        let _ = &a * &b;
    })
}

/// Benchmark FFT multiplication at a given bit length.
#[must_use]
pub fn bench_fft(bit_length: usize) -> Duration {
    let a = make_number(bit_length);
    let b = make_number(bit_length);
    benchmark(10, || {
        let _ = fibcalc_bigfft::mul(&a, &b);
    })
}

/// Benchmark multiplication at various bit lengths and return crossover info.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn find_fft_crossover(bit_lengths: &[usize]) -> Vec<CrossoverPoint> {
    bit_lengths
        .iter()
        .map(|&bits| {
            let karatsuba = bench_karatsuba_detailed(bits);
            let fft = bench_fft_detailed(bits);
            CrossoverPoint {
                bit_length: bits,
                karatsuba_ns: karatsuba.median.as_nanos() as u64,
                fft_ns: fft.median.as_nanos() as u64,
                fft_is_faster: fft.median < karatsuba.median,
            }
        })
        .collect()
}

/// Measure parallel overhead by comparing sequential vs parallel work.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn measure_parallel_overhead(bit_length: usize) -> ParallelOverhead {
    let a = make_number(bit_length);
    let b = make_number(bit_length);

    // Sequential: two multiplications in series
    let sequential = benchmark_detailed(2, 5, || {
        let _ = &a * &b;
        let _ = &a * &b;
    });

    // Parallel: two multiplications using rayon
    let a_clone = a.clone();
    let b_clone = b.clone();
    let parallel = benchmark_detailed(2, 5, || {
        rayon::join(|| &a * &b, || &a_clone * &b_clone);
    });

    let seq_ns = sequential.median.as_nanos() as u64;
    let par_ns = parallel.median.as_nanos() as u64;

    ParallelOverhead {
        bit_length,
        sequential_ns: seq_ns,
        parallel_ns: par_ns,
        speedup: if par_ns > 0 {
            seq_ns as f64 / par_ns as f64
        } else {
            1.0
        },
    }
}

/// Detailed Karatsuba benchmark.
fn bench_karatsuba_detailed(bit_length: usize) -> BenchmarkResult {
    let a = make_number(bit_length);
    let b = make_number(bit_length);
    benchmark_detailed(3, 10, || {
        let _ = &a * &b;
    })
    .with_name(format!("karatsuba_{bit_length}"))
}

/// Detailed FFT benchmark.
fn bench_fft_detailed(bit_length: usize) -> BenchmarkResult {
    let a = make_number(bit_length);
    let b = make_number(bit_length);
    benchmark_detailed(3, 10, || {
        let _ = fibcalc_bigfft::mul(&a, &b);
    })
    .with_name(format!("fft_{bit_length}"))
}

/// Create a `BigUint` with approximately the given number of bits.
fn make_number(bit_length: usize) -> BigUint {
    if bit_length == 0 {
        return BigUint::from(0u32);
    }
    // Create a number with the high bit set and some pattern
    let mut bytes = vec![0xABu8; bit_length.div_ceil(8)];
    bytes[0] |= 0x80; // ensure high bit is set
    BigUint::from_bytes_be(&bytes)
}

/// Result of comparing Karatsuba vs FFT at a specific bit length.
#[derive(Debug, Clone)]
pub struct CrossoverPoint {
    pub bit_length: usize,
    pub karatsuba_ns: u64,
    pub fft_ns: u64,
    pub fft_is_faster: bool,
}

/// Result of measuring parallel execution overhead.
#[derive(Debug, Clone)]
pub struct ParallelOverhead {
    pub bit_length: usize,
    pub sequential_ns: u64,
    pub parallel_ns: u64,
    pub speedup: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bench_karatsuba_small() {
        let duration = bench_karatsuba(1000);
        assert!(duration.as_nanos() > 0);
    }

    #[test]
    fn bench_fft_small() {
        let duration = bench_fft(1000);
        assert!(duration.as_nanos() > 0);
    }

    #[test]
    fn make_number_bit_length() {
        let n = make_number(1024);
        // Should have approximately 1024 bits
        assert!(n.bits() >= 1020);
        assert!(n.bits() <= 1024);
    }

    #[test]
    fn make_number_zero() {
        let n = make_number(0);
        assert_eq!(n, BigUint::from(0u32));
    }

    #[test]
    fn find_fft_crossover_runs() {
        let points = find_fft_crossover(&[512, 1024, 2048]);
        assert_eq!(points.len(), 3);
        for p in &points {
            assert!(p.karatsuba_ns > 0);
            assert!(p.fft_ns > 0);
        }
    }

    #[test]
    fn measure_parallel_overhead_runs() {
        let overhead = measure_parallel_overhead(2048);
        assert!(overhead.sequential_ns > 0);
        assert!(overhead.parallel_ns > 0);
        assert!(overhead.speedup > 0.0);
    }
}
