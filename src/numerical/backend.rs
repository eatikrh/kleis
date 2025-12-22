//! BLAS/LAPACK backend for numerical linear algebra.
//!
//! Uses ndarray-linalg with platform-specific backends:
//! - macOS: Apple Accelerate
//! - Linux/Windows: OpenBLAS

#![allow(clippy::type_complexity)]

use ndarray::{Array1, Array2};
use ndarray_linalg::types::c64;
use ndarray_linalg::{Cholesky, Eig, Inverse, Norm, Solve, QR, SVD, UPLO};
use std::fmt;

/// Error type for numerical operations
#[derive(Debug, Clone)]
pub enum NumericalError {
    /// Matrix is singular (non-invertible)
    Singular,
    /// Matrix dimensions don't match
    DimensionMismatch {
        expected: (usize, usize),
        got: (usize, usize),
    },
    /// Matrix must be square for this operation
    NotSquare { rows: usize, cols: usize },
    /// Matrix must be positive definite
    NotPositiveDefinite,
    /// Computation failed
    ComputationFailed(String),
}

impl fmt::Display for NumericalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumericalError::Singular => write!(f, "Matrix is singular"),
            NumericalError::DimensionMismatch { expected, got } => {
                write!(
                    f,
                    "Dimension mismatch: expected {:?}, got {:?}",
                    expected, got
                )
            }
            NumericalError::NotSquare { rows, cols } => {
                write!(f, "Matrix must be square, got {}×{}", rows, cols)
            }
            NumericalError::NotPositiveDefinite => {
                write!(f, "Matrix must be positive definite")
            }
            NumericalError::ComputationFailed(msg) => write!(f, "Computation failed: {}", msg),
        }
    }
}

impl std::error::Error for NumericalError {}

/// Compute eigenvalues of a square matrix.
/// Returns complex eigenvalues as (real, imag) pairs.
pub fn eigenvalues(matrix: &[f64], n: usize) -> Result<Vec<(f64, f64)>, NumericalError> {
    if matrix.len() != n * n {
        return Err(NumericalError::DimensionMismatch {
            expected: (n, n),
            got: (matrix.len() / n, matrix.len() % n),
        });
    }

    let arr = Array2::from_shape_vec((n, n), matrix.to_vec())
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    let (eigvals, _) = arr
        .eig()
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    Ok(eigvals.iter().map(|c| (c.re, c.im)).collect())
}

/// Compute eigenvalues and eigenvectors of a square matrix.
/// Returns (eigenvalues, eigenvectors) where eigenvectors are column vectors.
pub fn eig(
    matrix: &[f64],
    n: usize,
) -> Result<(Vec<(f64, f64)>, Vec<Vec<(f64, f64)>>), NumericalError> {
    if matrix.len() != n * n {
        return Err(NumericalError::DimensionMismatch {
            expected: (n, n),
            got: (matrix.len() / n, matrix.len() % n),
        });
    }

    let arr = Array2::from_shape_vec((n, n), matrix.to_vec())
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    let (eigvals, eigvecs) = arr
        .eig()
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    let vals: Vec<(f64, f64)> = eigvals.iter().map(|c| (c.re, c.im)).collect();

    // Extract eigenvector columns
    let mut vecs = Vec::with_capacity(n);
    for col in 0..n {
        let mut v = Vec::with_capacity(n);
        for row in 0..n {
            let c = eigvecs[[row, col]];
            v.push((c.re, c.im));
        }
        vecs.push(v);
    }

    Ok((vals, vecs))
}

/// Compute SVD (Singular Value Decomposition) of a matrix.
/// Returns (U, S, Vt) where A = U * diag(S) * Vt
pub fn svd(
    matrix: &[f64],
    m: usize,
    n: usize,
) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>), NumericalError> {
    if matrix.len() != m * n {
        return Err(NumericalError::DimensionMismatch {
            expected: (m, n),
            got: (matrix.len() / n, matrix.len() % n),
        });
    }

    let arr = Array2::from_shape_vec((m, n), matrix.to_vec())
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    let (u_opt, s, vt_opt) = arr
        .svd(true, true)
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    let u = u_opt
        .map(|u| u.into_raw_vec_and_offset().0)
        .unwrap_or_default();
    let vt = vt_opt
        .map(|vt| vt.into_raw_vec_and_offset().0)
        .unwrap_or_default();

    Ok((u, s.to_vec(), vt))
}

/// Compute singular values only.
pub fn singular_values(matrix: &[f64], m: usize, n: usize) -> Result<Vec<f64>, NumericalError> {
    if matrix.len() != m * n {
        return Err(NumericalError::DimensionMismatch {
            expected: (m, n),
            got: (matrix.len() / n, matrix.len() % n),
        });
    }

    let arr = Array2::from_shape_vec((m, n), matrix.to_vec())
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    let (_, s, _) = arr
        .svd(false, false)
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    Ok(s.to_vec())
}

/// Solve linear system Ax = b.
pub fn solve(a: &[f64], b: &[f64], n: usize) -> Result<Vec<f64>, NumericalError> {
    if a.len() != n * n {
        return Err(NumericalError::DimensionMismatch {
            expected: (n, n),
            got: (a.len() / n, a.len() % n),
        });
    }
    if b.len() != n {
        return Err(NumericalError::DimensionMismatch {
            expected: (n, 1),
            got: (b.len(), 1),
        });
    }

    let a_arr = Array2::from_shape_vec((n, n), a.to_vec())
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;
    let b_arr = Array1::from_vec(b.to_vec());

    let x = a_arr.solve(&b_arr).map_err(|_| NumericalError::Singular)?;

    Ok(x.to_vec())
}

/// Compute matrix inverse.
pub fn inv(matrix: &[f64], n: usize) -> Result<Vec<f64>, NumericalError> {
    if matrix.len() != n * n {
        return Err(NumericalError::DimensionMismatch {
            expected: (n, n),
            got: (matrix.len() / n, matrix.len() % n),
        });
    }

    let arr = Array2::from_shape_vec((n, n), matrix.to_vec())
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    let inv = arr.inv().map_err(|_| NumericalError::Singular)?;

    Ok(inv.into_raw_vec_and_offset().0)
}

/// Compute QR decomposition.
/// Returns (Q, R) where A = Q * R
pub fn qr(matrix: &[f64], m: usize, n: usize) -> Result<(Vec<f64>, Vec<f64>), NumericalError> {
    if matrix.len() != m * n {
        return Err(NumericalError::DimensionMismatch {
            expected: (m, n),
            got: (matrix.len() / n, matrix.len() % n),
        });
    }

    let arr = Array2::from_shape_vec((m, n), matrix.to_vec())
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    let (q, r) = arr
        .qr()
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    Ok((q.into_raw_vec_and_offset().0, r.into_raw_vec_and_offset().0))
}

/// Compute Cholesky decomposition of a positive-definite matrix.
/// Returns L where A = L * L^T
pub fn cholesky(matrix: &[f64], n: usize) -> Result<Vec<f64>, NumericalError> {
    if matrix.len() != n * n {
        return Err(NumericalError::DimensionMismatch {
            expected: (n, n),
            got: (matrix.len() / n, matrix.len() % n),
        });
    }

    let arr = Array2::from_shape_vec((n, n), matrix.to_vec())
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    let l = arr
        .cholesky(UPLO::Lower)
        .map_err(|_| NumericalError::NotPositiveDefinite)?;

    Ok(l.into_raw_vec_and_offset().0)
}

/// Compute matrix rank via SVD.
pub fn rank(matrix: &[f64], m: usize, n: usize, tol: Option<f64>) -> Result<usize, NumericalError> {
    let s = singular_values(matrix, m, n)?;

    let tolerance = tol.unwrap_or_else(|| {
        let max_dim = m.max(n) as f64;
        let max_s = s.iter().cloned().fold(0.0_f64, f64::max);
        max_dim * max_s * f64::EPSILON
    });

    Ok(s.iter().filter(|&&v| v > tolerance).count())
}

/// Compute condition number via SVD.
pub fn cond(matrix: &[f64], m: usize, n: usize) -> Result<f64, NumericalError> {
    let s = singular_values(matrix, m, n)?;

    let max_s = s.iter().cloned().fold(0.0_f64, f64::max);
    let min_s = s.iter().cloned().fold(f64::INFINITY, f64::min);

    if min_s == 0.0 {
        Ok(f64::INFINITY)
    } else {
        Ok(max_s / min_s)
    }
}

/// Compute matrix norm.
/// norm_type: "fro" (Frobenius), "1" (1-norm), "inf" (infinity norm)
pub fn norm(matrix: &[f64], m: usize, n: usize, norm_type: &str) -> Result<f64, NumericalError> {
    if matrix.len() != m * n {
        return Err(NumericalError::DimensionMismatch {
            expected: (m, n),
            got: (matrix.len() / n, matrix.len() % n),
        });
    }

    let arr = Array2::from_shape_vec((m, n), matrix.to_vec())
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    match norm_type {
        "fro" | "frobenius" => Ok(arr.norm_l2()),
        "1" | "one" => Ok(arr.norm_l1()),
        "inf" | "infinity" => Ok(arr.norm_max()),
        _ => Err(NumericalError::ComputationFailed(format!(
            "Unknown norm type: {}",
            norm_type
        ))),
    }
}

/// Compute determinant of a square matrix using LU decomposition.
pub fn det(matrix: &[f64], n: usize) -> Result<f64, NumericalError> {
    if matrix.len() != n * n {
        return Err(NumericalError::DimensionMismatch {
            expected: (n, n),
            got: (matrix.len() / n, matrix.len() % n),
        });
    }

    // For small matrices, use direct computation (faster and more accurate)
    if n == 1 {
        return Ok(matrix[0]);
    }
    if n == 2 {
        return Ok(matrix[0] * matrix[3] - matrix[1] * matrix[2]);
    }
    if n == 3 {
        return Ok(matrix[0] * (matrix[4] * matrix[8] - matrix[5] * matrix[7])
            - matrix[1] * (matrix[3] * matrix[8] - matrix[5] * matrix[6])
            + matrix[2] * (matrix[3] * matrix[7] - matrix[4] * matrix[6]));
    }

    // For larger matrices, use LU decomposition via solving
    // det(A) = product of diagonal of U (with sign from permutation)
    // We compute via: det(A) = 1 / det(A^-1) if A is invertible
    let arr = Array2::from_shape_vec((n, n), matrix.to_vec())
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    // Use the fact that det can be computed during solve
    // For now, use eigenvalues: det = product of eigenvalues
    let (eigvals, _) = arr
        .eig()
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    let det: c64 = eigvals.iter().product();

    // Real matrices have real determinants (imaginary part is numerical noise)
    Ok(det.re)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eigenvalues_2x2() {
        // [[1, 2], [3, 4]] has eigenvalues ≈ 5.372 and -0.372
        let m = vec![1.0, 2.0, 3.0, 4.0];
        let eigs = eigenvalues(&m, 2).unwrap();

        // Sort by real part
        let mut reals: Vec<f64> = eigs.iter().map(|(re, _)| *re).collect();
        reals.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert!((reals[0] - (-0.372)).abs() < 0.01);
        assert!((reals[1] - 5.372).abs() < 0.01);
    }

    #[test]
    fn test_solve_2x2() {
        // [[1, 0], [0, 1]] * x = [3, 4] => x = [3, 4]
        let a = vec![1.0, 0.0, 0.0, 1.0];
        let b = vec![3.0, 4.0];
        let x = solve(&a, &b, 2).unwrap();

        assert!((x[0] - 3.0).abs() < 1e-10);
        assert!((x[1] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_inv_2x2() {
        // [[1, 2], [3, 4]] inverse is [[-2, 1], [1.5, -0.5]]
        let m = vec![1.0, 2.0, 3.0, 4.0];
        let inv_m = inv(&m, 2).unwrap();

        assert!((inv_m[0] - (-2.0)).abs() < 1e-10);
        assert!((inv_m[1] - 1.0).abs() < 1e-10);
        assert!((inv_m[2] - 1.5).abs() < 1e-10);
        assert!((inv_m[3] - (-0.5)).abs() < 1e-10);
    }

    #[test]
    fn test_det_3x3() {
        // [[1, 2, 3], [4, 5, 6], [7, 8, 9]] has det = 0 (singular)
        let m = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let d = det(&m, 3).unwrap();
        assert!(d.abs() < 1e-10);
    }

    #[test]
    fn test_svd_2x2() {
        let m = vec![1.0, 0.0, 0.0, 1.0]; // Identity
        let (_, s, _) = svd(&m, 2, 2).unwrap();

        assert!((s[0] - 1.0).abs() < 1e-10);
        assert!((s[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rank() {
        // [[1, 0], [0, 1]] has rank 2
        let m = vec![1.0, 0.0, 0.0, 1.0];
        let r = rank(&m, 2, 2, None).unwrap();
        assert_eq!(r, 2);

        // [[1, 2], [2, 4]] has rank 1 (row 2 = 2 * row 1)
        let m2 = vec![1.0, 2.0, 2.0, 4.0];
        let r2 = rank(&m2, 2, 2, None).unwrap();
        assert_eq!(r2, 1);
    }
}
