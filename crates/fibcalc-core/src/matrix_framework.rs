//! Generic matrix exponentiation framework.

use num_bigint::BigUint;

use crate::calculator::FibError;
use crate::matrix_ops::{matrix_multiply, matrix_square};
use crate::matrix_types::MatrixState;
use crate::observer::ProgressObserver;
use crate::progress::{CancellationToken, ProgressUpdate};

/// Execute matrix exponentiation loop to compute Q^n.
pub fn execute_matrix_loop(
    n: u64,
    cancel: &CancellationToken,
    observer: &dyn ProgressObserver,
    calc_index: usize,
    algorithm_name: &str,
) -> Result<BigUint, FibError> {
    let num_bits = 64 - n.leading_zeros();
    let mut state = MatrixState::new();
    let frozen = observer.freeze();

    for i in (0..num_bits).rev() {
        if cancel.is_cancelled() {
            return Err(FibError::Cancelled);
        }

        state.result = matrix_square(&state.result);

        if (n >> i) & 1 == 1 {
            state.result = matrix_multiply(&state.result, &state.base);
        }

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

    Ok(std::mem::take(&mut state.result.b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observers::NoOpObserver;

    #[test]
    fn matrix_framework_computes_correctly() {
        let cancel = CancellationToken::new();
        let observer = NoOpObserver::new();
        let result = execute_matrix_loop(100, &cancel, &observer, 0, "test").unwrap();
        assert_eq!(
            result,
            BigUint::parse_bytes(b"354224848179261915075", 10).unwrap()
        );
    }
}
