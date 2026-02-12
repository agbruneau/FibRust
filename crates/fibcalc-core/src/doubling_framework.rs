//! Generic doubling framework for Fast Doubling algorithms.

use num_bigint::BigUint;

use crate::calculator::FibError;
use crate::observer::ProgressObserver;
use crate::progress::{CancellationToken, ProgressUpdate};
use crate::strategy::{DoublingStepExecutor, KaratsubaStrategy, ParallelKaratsubaStrategy};

/// Execute the doubling loop with a given step executor.
pub fn execute_doubling_loop(
    n: u64,
    executor: &dyn DoublingStepExecutor,
    cancel: &CancellationToken,
    observer: &dyn ProgressObserver,
    calc_index: usize,
    algorithm_name: &'static str,
) -> Result<BigUint, FibError> {
    let num_bits = 64 - n.leading_zeros();
    let mut fk = BigUint::ZERO;
    let mut fk1 = BigUint::from(1u32);

    let frozen = observer.freeze();

    for i in (0..num_bits).rev() {
        if cancel.is_cancelled() {
            return Err(FibError::Cancelled);
        }

        // Doubling step
        let (f2k, f2k1) = executor.execute_doubling_step(&fk, &fk1);
        fk = f2k;
        fk1 = f2k1;

        // Conditional addition
        if (n >> i) & 1 == 1 {
            let sum = &fk + &fk1;
            fk = std::mem::replace(&mut fk1, sum);
        }

        // Progress
        let progress = 1.0 - (f64::from(i) / f64::from(num_bits));
        if frozen.should_report(progress) {
            frozen.update(progress);
            observer.on_progress(&ProgressUpdate::new(
                calc_index,
                algorithm_name,
                progress,
                u64::from(num_bits - i),
                u64::from(num_bits),
            ));
        }
    }

    Ok(fk)
}

/// Execute the doubling loop with optional parallelism.
///
/// When `parallel_threshold > 0`, uses `ParallelKaratsubaStrategy` which
/// parallelizes the three independent multiplications via `rayon::join`
/// when operand bits exceed the threshold. When `parallel_threshold == 0`,
/// uses the sequential `KaratsubaStrategy`.
pub fn execute_doubling_loop_parallel(
    n: u64,
    cancel: &CancellationToken,
    observer: &dyn ProgressObserver,
    calc_index: usize,
    algorithm_name: &'static str,
    parallel_threshold: usize,
) -> Result<BigUint, FibError> {
    if parallel_threshold > 0 {
        let strategy = ParallelKaratsubaStrategy::new(parallel_threshold);
        execute_doubling_loop(n, &strategy, cancel, observer, calc_index, algorithm_name)
    } else {
        let strategy = KaratsubaStrategy::new();
        execute_doubling_loop(n, &strategy, cancel, observer, calc_index, algorithm_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observers::NoOpObserver;
    use crate::strategy::KaratsubaStrategy;

    #[test]
    fn framework_computes_correctly() {
        let strategy = KaratsubaStrategy::new();
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let result = execute_doubling_loop(100, &strategy, &cancel, &observer, 0, "test").unwrap();
        assert_eq!(
            result,
            BigUint::parse_bytes(b"354224848179261915075", 10).unwrap()
        );
    }

    #[test]
    fn framework_parallel_computes_correctly() {
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let expected = BigUint::parse_bytes(b"354224848179261915075", 10).unwrap();

        // With parallel (threshold = 1, always parallel)
        let result_par =
            execute_doubling_loop_parallel(100, &cancel, &observer, 0, "test", 1).unwrap();
        assert_eq!(result_par, expected);

        // Without parallel (threshold = 0, always sequential)
        let result_seq =
            execute_doubling_loop_parallel(100, &cancel, &observer, 0, "test", 0).unwrap();
        assert_eq!(result_seq, expected);
    }

    #[test]
    fn framework_parallel_matches_sequential() {
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();

        for n in [94, 100, 200, 500, 1000] {
            let seq = execute_doubling_loop_parallel(n, &cancel, &observer, 0, "test", 0).unwrap();
            let par = execute_doubling_loop_parallel(n, &cancel, &observer, 0, "test", 1).unwrap();
            assert_eq!(seq, par, "Parallel/sequential mismatch at n={n}");
        }
    }
}
