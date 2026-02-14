//! Benchmark runner for calibration.

use std::time::{Duration, Instant};

/// Run a benchmark function and return its average duration.
pub fn benchmark<F>(iterations: u32, mut f: F) -> Duration
where
    F: FnMut(),
{
    // Warm up
    f();

    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    start.elapsed() / iterations
}

/// Run a benchmark with a warmup phase, returning (median, min, max).
pub fn benchmark_detailed<F>(warmup_iters: u32, measure_iters: u32, mut f: F) -> BenchmarkResult
where
    F: FnMut(),
{
    // Warmup
    for _ in 0..warmup_iters {
        f();
    }

    // Measure
    let mut durations = Vec::with_capacity(measure_iters as usize);
    for _ in 0..measure_iters {
        let start = Instant::now();
        f();
        durations.push(start.elapsed());
    }

    durations.sort();
    let min = durations.first().copied().unwrap_or_default();
    let max = durations.last().copied().unwrap_or_default();
    let median = if durations.len() % 2 == 1 {
        durations[durations.len() / 2]
    } else {
        let mid = durations.len() / 2;
        (durations[mid - 1] + durations[mid]) / 2
    };
    let total: Duration = durations.iter().sum();
    let mean = total / measure_iters;

    BenchmarkResult {
        name: String::new(),
        mean,
        median,
        min,
        max,
        iterations: measure_iters,
    }
}

/// Result of a single benchmark run.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub mean: Duration,
    pub median: Duration,
    pub min: Duration,
    pub max: Duration,
    pub iterations: u32,
}

impl BenchmarkResult {
    /// Create a named result.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_runs() {
        let duration = benchmark(10, || {
            let _ = 2 + 2;
        });
        assert!(duration.as_nanos() < 1_000_000);
    }

    #[test]
    fn benchmark_detailed_runs() {
        let result = benchmark_detailed(2, 5, || {
            let _ = 2 + 2;
        });
        assert_eq!(result.iterations, 5);
        assert!(result.min <= result.median);
        assert!(result.median <= result.max);
        assert!(result.mean.as_nanos() < 1_000_000);
    }

    #[test]
    fn benchmark_result_with_name() {
        let result = benchmark_detailed(1, 3, || {}).with_name("test_bench");
        assert_eq!(result.name, "test_bench");
    }
}
