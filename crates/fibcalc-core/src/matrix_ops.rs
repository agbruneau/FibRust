//! Matrix multiplication operations including Strassen.

use crate::matrix_types::Matrix;

/// Multiply two 2x2 matrices using standard algorithm.
#[must_use]
pub fn matrix_multiply(a: &Matrix, b: &Matrix) -> Matrix {
    Matrix {
        a: &a.a * &b.a + &a.b * &b.c,
        b: &a.a * &b.b + &a.b * &b.d,
        c: &a.c * &b.a + &a.d * &b.c,
        d: &a.c * &b.b + &a.d * &b.d,
    }
}

/// Square a 2x2 matrix.
#[must_use]
pub fn matrix_square(m: &Matrix) -> Matrix {
    matrix_multiply(m, m)
}

/// Multiply two 2x2 matrices using Strassen-like optimization.
///
/// For 2x2 Fibonacci matrices with the symmetry property (b == c),
/// we can reduce the number of multiplications.
#[must_use]
pub fn matrix_multiply_strassen(a: &Matrix, b: &Matrix, threshold: usize) -> Matrix {
    let max_bits = a.a.bits().max(b.a.bits()) as usize;

    if max_bits < threshold {
        return matrix_multiply(a, b);
    }

    // Strassen-like optimization for symmetric Fibonacci matrices
    // Standard 2x2 Strassen uses 7 multiplications instead of 8
    // For our specific case, we can exploit b==c symmetry

    // Fall back to standard for now
    // TODO: Implement full Strassen with recursive subdivision
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
}
