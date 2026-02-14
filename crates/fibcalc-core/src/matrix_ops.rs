//! Matrix multiplication operations including Strassen.

use crate::matrix_types::Matrix;

/// Multiply two 2x2 matrices exploiting Fibonacci symmetry (b == c).
///
/// All powers of the Fibonacci Q matrix `[[1,1],[1,0]]` are symmetric
/// (i.e., `b == c`), so we use `multiply_symmetric` which requires
/// 5 multiplications instead of the standard 8.
#[must_use]
pub fn matrix_multiply(a: &Matrix, b: &Matrix) -> Matrix {
    a.multiply_symmetric(b)
}

/// Square a 2x2 matrix exploiting Fibonacci symmetry (b == c).
///
/// Uses `square_symmetric` which requires 3 multiplications instead of 8.
#[must_use]
pub fn matrix_square(m: &Matrix) -> Matrix {
    m.square_symmetric()
}

/// Multiply two 2x2 matrices using Strassen-like optimization.
///
/// For 2x2 Fibonacci matrices with the symmetry property (b == c),
/// we can reduce the number of multiplications.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
#[allow(dead_code)] // TODO: Phase 2 — PRD T2.4 Strassen optimization
pub fn matrix_multiply_strassen(a: &Matrix, b: &Matrix, threshold: usize) -> Matrix {
    let max_bits = a.a.bits().max(b.a.bits()) as usize;

    if max_bits < threshold {
        return matrix_multiply(a, b);
    }

    // Strassen-like optimization for symmetric Fibonacci matrices
    // Standard 2x2 Strassen uses 7 multiplications instead of 8
    // For our specific case, we can exploit b==c symmetry

    // P2 optimization: full Strassen with recursive subdivision would reduce
    // multiplications from 5 to ~4.7 for large operands, but the symmetric
    // multiply already exploits Fibonacci Q-matrix structure (5 muls vs 8).
    // Current fallback is correct and performant for all tested input sizes.
    matrix_multiply(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;

    #[test]
    fn multiply_identity() {
        let id = Matrix::identity();
        let q = Matrix::fibonacci_q();
        let result = matrix_multiply(&id, &q);
        assert_eq!(result.a, q.a);
        assert_eq!(result.b, q.b);
        assert_eq!(result.c, q.c);
        assert_eq!(result.d, q.d);
    }

    #[test]
    fn square_q_matrix() {
        let q = Matrix::fibonacci_q();
        let q2 = matrix_square(&q);
        // Q^2 = [[2,1],[1,1]]
        assert_eq!(q2.a, BigUint::from(2u32));
        assert_eq!(q2.b, BigUint::from(1u32));
        assert_eq!(q2.c, BigUint::from(1u32));
        assert_eq!(q2.d, BigUint::from(1u32));
    }

    #[test]
    fn cube_q_matrix() {
        let q = Matrix::fibonacci_q();
        let q2 = matrix_square(&q);
        let q3 = matrix_multiply(&q2, &q);
        // Q^3 = [[3,2],[2,1]]
        assert_eq!(q3.a, BigUint::from(3u32));
        assert_eq!(q3.b, BigUint::from(2u32));
    }

    #[test]
    fn multiply_q_by_identity_both_sides() {
        let id = Matrix::identity();
        let q = Matrix::fibonacci_q();

        let left = matrix_multiply(&id, &q);
        let right = matrix_multiply(&q, &id);

        assert_eq!(left.a, q.a);
        assert_eq!(left.b, q.b);
        assert_eq!(left.c, q.c);
        assert_eq!(left.d, q.d);

        assert_eq!(right.a, q.a);
        assert_eq!(right.b, q.b);
        assert_eq!(right.c, q.c);
        assert_eq!(right.d, q.d);
    }

    #[test]
    fn square_identity_is_identity() {
        let id = Matrix::identity();
        let sq = matrix_square(&id);
        assert!(sq.is_identity());
    }

    #[test]
    fn q_power_5_gives_fib_5() {
        // Q^n gives F(n) in position [0][1] (or b)
        let q = Matrix::fibonacci_q();
        let q2 = matrix_square(&q);
        let q4 = matrix_square(&q2);
        let q5 = matrix_multiply(&q4, &q);
        // Q^5: a = F(6) = 8, b = F(5) = 5
        assert_eq!(q5.a, BigUint::from(8u32));
        assert_eq!(q5.b, BigUint::from(5u32));
        assert_eq!(q5.c, BigUint::from(5u32));
        assert_eq!(q5.d, BigUint::from(3u32));
    }

    #[test]
    fn q_power_10_gives_fib_10() {
        let q = Matrix::fibonacci_q();
        let q2 = matrix_square(&q);
        let q4 = matrix_square(&q2);
        let q8 = matrix_square(&q4);
        let q10 = matrix_multiply(&q8, &q2);
        // Q^10: a = F(11) = 89, b = F(10) = 55
        assert_eq!(q10.a, BigUint::from(89u32));
        assert_eq!(q10.b, BigUint::from(55u32));
    }

    #[test]
    fn strassen_below_threshold_uses_standard() {
        let q = Matrix::fibonacci_q();
        let q2_standard = matrix_multiply(&q, &q);
        // Threshold very high -> should fall through to standard multiply
        let q2_strassen = matrix_multiply_strassen(&q, &q, 1_000_000);
        assert_eq!(q2_standard.a, q2_strassen.a);
        assert_eq!(q2_standard.b, q2_strassen.b);
        assert_eq!(q2_standard.c, q2_strassen.c);
        assert_eq!(q2_standard.d, q2_strassen.d);
    }

    #[test]
    fn strassen_above_threshold() {
        let q = Matrix::fibonacci_q();
        // Threshold 0 -> should take the "strassen" path (which currently falls back)
        let q2 = matrix_multiply_strassen(&q, &q, 0);
        assert_eq!(q2.a, BigUint::from(2u32));
        assert_eq!(q2.b, BigUint::from(1u32));
    }

    #[test]
    fn matrix_symmetry_preserved_through_operations() {
        // Fibonacci Q-matrix powers should always be symmetric (b == c)
        let q = Matrix::fibonacci_q();
        let q2 = matrix_square(&q);
        assert_eq!(q2.b, q2.c);

        let q3 = matrix_multiply(&q2, &q);
        assert_eq!(q3.b, q3.c);

        let q4 = matrix_square(&q2);
        assert_eq!(q4.b, q4.c);
    }
}
