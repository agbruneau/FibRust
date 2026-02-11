//! Common utilities: task semaphore, generic task execution, pools.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use rayon::ThreadPoolBuilder;

use crate::calculator::FibError;

/// Execute tasks in parallel with a thread pool acting as semaphore.
///
/// Limits concurrency to `max_concurrency` threads. Collects all results
/// or returns the first error encountered.
pub fn execute_tasks<T, F>(tasks: Vec<F>, max_concurrency: usize) -> Result<Vec<T>, FibError>
where
    T: Send,
    F: FnOnce() -> Result<T, FibError> + Send,
{
    let pool = ThreadPoolBuilder::new()
        .num_threads(max_concurrency)
        .build()
        .map_err(|e| FibError::Calculation(format!("failed to create thread pool: {e}")))?;

    let results: Vec<Result<T, FibError>> = pool.install(|| {
        use rayon::iter::{IntoParallelIterator, ParallelIterator};
        tasks.into_par_iter().map(|task| task()).collect()
    });

    results.into_iter().collect()
}

/// Execute tasks in parallel with first-error-wins semantics.
///
/// When the first task fails, sets a cancellation flag so remaining tasks
/// can check and bail out early. Returns either all successful results
/// or the first error encountered.
///
/// The `max_concurrency` parameter limits the number of concurrent threads
/// (semaphore pattern: typically `2 * num_cpus`).
pub fn execute_tasks_first_error<T, F>(
    tasks: Vec<F>,
    max_concurrency: usize,
) -> Result<Vec<T>, FibError>
where
    T: Send,
    F: FnOnce(&AtomicBool) -> Result<T, FibError> + Send,
{
    let pool = ThreadPoolBuilder::new()
        .num_threads(max_concurrency)
        .build()
        .map_err(|e| FibError::Calculation(format!("failed to create thread pool: {e}")))?;

    let error_flag = Arc::new(AtomicBool::new(false));

    let results: Vec<Result<T, FibError>> = pool.install(|| {
        use rayon::iter::{IntoParallelIterator, ParallelIterator};
        tasks
            .into_par_iter()
            .map(|task| {
                // Check if another task has already failed
                if error_flag.load(Ordering::Relaxed) {
                    return Err(FibError::Cancelled);
                }

                let result = task(&error_flag);

                if result.is_err() {
                    error_flag.store(true, Ordering::Relaxed);
                }

                result
            })
            .collect()
    });

    // Collect results, filtering out cancellation errors that were caused by
    // the first-error-wins pattern (keep the original error).
    let mut collected = Vec::with_capacity(results.len());
    let mut first_error: Option<FibError> = None;

    for result in results {
        match result {
            Ok(val) => collected.push(val),
            Err(FibError::Cancelled) if first_error.is_some() => {
                // Ignore secondary cancellations
            }
            Err(e) => {
                if first_error.is_none() {
                    first_error = Some(e);
                }
            }
        }
    }

    if let Some(err) = first_error {
        return Err(err);
    }

    Ok(collected)
}

/// Collect errors from parallel task results, returning all errors.
///
/// Useful when you want to report all failures instead of just the first one.
#[must_use]
pub fn collect_errors<T>(results: Vec<Result<T, FibError>>) -> (Vec<T>, Vec<FibError>) {
    let mut successes = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok(val) => successes.push(val),
            Err(e) => errors.push(e),
        }
    }

    (successes, errors)
}

/// Get the default parallelism level (2 * `num_cpus`).
#[must_use]
pub fn default_parallelism() -> usize {
    let cpus = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(4);
    cpus * 2
}

/// Get the semaphore-limited concurrency (capped at 2 * `num_cpus`).
#[must_use]
pub fn semaphore_concurrency(requested: Option<usize>) -> usize {
    let max = default_parallelism();
    match requested {
        Some(n) if n > 0 => n.min(max),
        _ => max,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_parallelism_positive() {
        assert!(default_parallelism() > 0);
    }

    #[test]
    fn execute_tasks_simple() {
        let tasks: Vec<Box<dyn FnOnce() -> Result<u32, FibError> + Send>> =
            vec![Box::new(|| Ok(1)), Box::new(|| Ok(2)), Box::new(|| Ok(3))];
        let results = execute_tasks(tasks, 2).unwrap();
        assert_eq!(results.len(), 3);
        let mut sorted = results.clone();
        sorted.sort();
        assert_eq!(sorted, vec![1, 2, 3]);
    }

    #[test]
    fn execute_tasks_error_propagation() {
        let tasks: Vec<Box<dyn FnOnce() -> Result<u32, FibError> + Send>> = vec![
            Box::new(|| Ok(1)),
            Box::new(|| Err(FibError::Calculation("test error".into()))),
            Box::new(|| Ok(3)),
        ];
        let result = execute_tasks(tasks, 2);
        assert!(result.is_err());
    }

    #[test]
    fn execute_tasks_first_error_simple() {
        let tasks: Vec<Box<dyn FnOnce(&AtomicBool) -> Result<u32, FibError> + Send>> = vec![
            Box::new(|_| Ok(1)),
            Box::new(|_| Ok(2)),
            Box::new(|_| Ok(3)),
        ];
        let results = execute_tasks_first_error(tasks, 2).unwrap();
        assert_eq!(results.len(), 3);
        let mut sorted = results.clone();
        sorted.sort();
        assert_eq!(sorted, vec![1, 2, 3]);
    }

    #[test]
    fn execute_tasks_first_error_returns_first() {
        let tasks: Vec<Box<dyn FnOnce(&AtomicBool) -> Result<u32, FibError> + Send>> = vec![
            Box::new(|_| Err(FibError::Calculation("first error".into()))),
            Box::new(|flag| {
                // Simulate checking the error flag
                if flag.load(Ordering::Relaxed) {
                    return Err(FibError::Cancelled);
                }
                Ok(2)
            }),
        ];
        let result = execute_tasks_first_error(tasks, 2);
        assert!(result.is_err());
        match result.unwrap_err() {
            FibError::Calculation(msg) => assert_eq!(msg, "first error"),
            FibError::Cancelled => {} // Also acceptable in race conditions
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn collect_errors_all_success() {
        let results: Vec<Result<u32, FibError>> = vec![Ok(1), Ok(2), Ok(3)];
        let (successes, errors) = collect_errors(results);
        assert_eq!(successes, vec![1, 2, 3]);
        assert!(errors.is_empty());
    }

    #[test]
    fn collect_errors_mixed() {
        let results: Vec<Result<u32, FibError>> = vec![
            Ok(1),
            Err(FibError::Calculation("err1".into())),
            Ok(3),
            Err(FibError::Cancelled),
        ];
        let (successes, errors) = collect_errors(results);
        assert_eq!(successes, vec![1, 3]);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn semaphore_concurrency_default() {
        let c = semaphore_concurrency(None);
        assert_eq!(c, default_parallelism());
    }

    #[test]
    fn semaphore_concurrency_capped() {
        let max = default_parallelism();
        let c = semaphore_concurrency(Some(max * 10));
        assert_eq!(c, max);
    }

    #[test]
    fn semaphore_concurrency_custom() {
        let c = semaphore_concurrency(Some(2));
        assert_eq!(c, 2);
    }

    #[test]
    fn semaphore_concurrency_zero_falls_back() {
        let c = semaphore_concurrency(Some(0));
        assert_eq!(c, default_parallelism());
    }
}
