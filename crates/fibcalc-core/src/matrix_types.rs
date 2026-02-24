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
    #[allow(dead_code)]
    pub fn is_identity(&self) -> bool {
        self.a == BigUint::from(1u32)
            && self.b == BigUint::ZERO
            && self.c == BigUint::ZERO
            && self.d == BigUint::from(1u32)
    }

    /// Optimized squaring for symmetric matrices (where b == c).
    ///
    /// For `[[a,b],[b,d]]`, the square is:
    /// `[[a*a + b*b, b*(a+d)], [b*(a+d), b*b + d*d]]`
    ///
    /// This requires 3 multiplications + 2 additions instead of the
    /// 8 multiplications + 4 additions of standard 2x2 matrix squaring.
    #[must_use]
    pub fn square_symmetric(&self) -> Self {
        let b_sq = &self.b * &self.b;
        let new_a = &self.a * &self.a + &b_sq;
        let new_b = &self.b * (&self.a + &self.d);
        let new_d = &b_sq + &self.d * &self.d;
        Self {
            a: new_a,
            b: new_b.clone(),
            c: new_b,
            d: new_d,
        }
    }

    /// In-place squaring for symmetric matrices.
    ///
    /// Mutates `self` to contain `self * self`, reusing buffer capacity.
    pub fn square_symmetric_into(&mut self) {
        let b_sq = &self.b * &self.b;
        let new_a = &self.a * &self.a + &b_sq;
        let new_b = &self.b * (&self.a + &self.d);
        let new_d = &b_sq + &self.d * &self.d;
        self.a = new_a;
        self.c.clone_from(&new_b);
        self.b = new_b;
        self.d = new_d;
    }

    /// Optimized multiplication for symmetric Fibonacci matrices.
    ///
    /// For symmetric `A=[[a1,b1],[b1,d1]]` and `B=[[a2,b2],[b2,d2]]`:
    /// The result is also symmetric, requiring 5 multiplications + 2 additions
    /// instead of the 8 multiplications + 4 additions of standard multiply.
    #[must_use]
    pub fn multiply_symmetric(&self, other: &Self) -> Self {
        let b1_b2 = &self.b * &other.b;
        let new_a = &self.a * &other.a + &b1_b2;
        let new_b = &self.a * &other.b + &self.b * &other.d;
        let new_d = &b1_b2 + &self.d * &other.d;
        Self {
            a: new_a,
            b: new_b.clone(),
            c: new_b,
            d: new_d,
        }
    }

    /// In-place multiplication for symmetric matrices.
    ///
    /// Mutates `self` to contain `self * other`, reusing buffer capacity.
    pub fn multiply_symmetric_into(&mut self, other: &Self) {
        let b1_b2 = &self.b * &other.b;
        let new_a = &self.a * &other.a + &b1_b2;
        let new_b = &self.a * &other.b + &self.b * &other.d;
        let new_d = &b1_b2 + &self.d * &other.d;
        self.a = new_a;
        self.c.clone_from(&new_b);
        self.b = new_b;
        self.d = new_d;
    }
}

/// State for matrix exponentiation computation.
pub struct MatrixState {
    pub result: Matrix,
    pub base: Matrix,
}

impl MatrixState {
    /// Create a new matrix state for computing Q^n.
    #[must_use]
    pub fn new() -> Self {
        Self {
            result: Matrix::identity(),
            base: Matrix::fibonacci_q(),
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
    fn square_symmetric_into_matches_immutable() {
        let q = Matrix::fibonacci_q();
        let expected = q.square_symmetric();
        let mut m = q.clone();
        m.square_symmetric_into();
        assert_eq!(m.a, expected.a);
        assert_eq!(m.b, expected.b);
        assert_eq!(m.d, expected.d);
    }

    #[test]
    fn multiply_symmetric_into_matches_immutable() {
        let q = Matrix::fibonacci_q();
        let q2 = q.square_symmetric();
        let expected = q2.multiply_symmetric(&q);
        let mut m = q2.clone();
        m.multiply_symmetric_into(&q);
        assert_eq!(m.a, expected.a);
        assert_eq!(m.b, expected.b);
        assert_eq!(m.d, expected.d);
    }

    #[test]
    fn matrix_state_new() {
        let state = MatrixState::new();
        assert!(state.result.is_identity());
    }
}
