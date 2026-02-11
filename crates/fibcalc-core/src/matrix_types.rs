//! Matrix types for the Matrix Exponentiation algorithm.

use num_bigint::BigUint;

/// 2x2 matrix of `BigUint` values.
#[derive(Debug, Clone)]
pub struct Matrix {
    pub a: BigUint, // [0][0]
    pub b: BigUint, // [0][1]
    pub c: BigUint, // [1][0]
    pub d: BigUint, // [1][1]
}

impl Matrix {
    /// Create the identity matrix.
    #[must_use]
    pub fn identity() -> Self {
        Self {
            a: BigUint::from(1u32),
            b: BigUint::ZERO,
            c: BigUint::ZERO,
            d: BigUint::from(1u32),
        }
    }

    /// Create the Fibonacci Q matrix [[1,1],[1,0]].
    #[must_use]
    pub fn fibonacci_q() -> Self {
        Self {
            a: BigUint::from(1u32),
            b: BigUint::from(1u32),
            c: BigUint::from(1u32),
            d: BigUint::ZERO,
        }
    }

    /// Check if this is the identity matrix.
    #[must_use]
    pub fn is_identity(&self) -> bool {
        self.a == BigUint::from(1u32)
            && self.b == BigUint::ZERO
            && self.c == BigUint::ZERO
            && self.d == BigUint::from(1u32)
    }
}

/// State for matrix exponentiation computation.
pub struct MatrixState {
    pub result: Matrix,
    pub base: Matrix,
    pub temp: Matrix,
}

impl MatrixState {
    /// Create a new matrix state for computing Q^n.
    #[must_use]
    pub fn new() -> Self {
        Self {
            result: Matrix::identity(),
            base: Matrix::fibonacci_q(),
            temp: Matrix::identity(),
        }
    }

    /// Reset state for reuse.
    pub fn reset(&mut self) {
        self.result = Matrix::identity();
        self.base = Matrix::fibonacci_q();
    }
}

impl Default for MatrixState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_matrix() {
        let m = Matrix::identity();
        assert!(m.is_identity());
    }

    #[test]
    fn fibonacci_q_matrix() {
        let q = Matrix::fibonacci_q();
        assert_eq!(q.a, BigUint::from(1u32));
        assert_eq!(q.b, BigUint::from(1u32));
        assert_eq!(q.c, BigUint::from(1u32));
        assert_eq!(q.d, BigUint::ZERO);
    }

    #[test]
    fn matrix_state_new() {
        let state = MatrixState::new();
        assert!(state.result.is_identity());
    }
}
