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
        "fro" | "frobenius" => {
            // Frobenius norm: sqrt(sum_{i,j} a_ij^2)
            // Note: ndarray-linalg's norm_l2() is vector 2-norm, not matrix Frobenius
            let sum_sq: f64 = arr.iter().map(|x| x * x).sum();
            Ok(sum_sq.sqrt())
        }
        "1" | "one" => Ok(arr.norm_l1()),
        "inf" | "infinity" => Ok(arr.norm_max()),
        _ => Err(NumericalError::ComputationFailed(format!(
            "Unknown norm type: {}",
            norm_type
        ))),
    }
}

/// Compute determinant of a square matrix.
///
/// ## Implementation Notes
/// - n ≤ 3: Direct formula (exact, fast)
/// - n > 3: Uses eigenvalue product (det = ∏λᵢ)
///
/// **Warning**: For n > 3, the eigenvalue-based method may be numerically
/// ill-conditioned when eigenvalues have widely varying magnitudes.
/// For critical applications, consider LU-based determinant.
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

    // For larger matrices, use eigenvalue product: det(A) = ∏λᵢ
    // Note: This can be ill-conditioned for matrices with widely varying eigenvalues.
    // For better stability, LU-based determinant could be used.
    let arr = Array2::from_shape_vec((n, n), matrix.to_vec())
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    let (eigvals, _) = arr
        .eig()
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    let det: c64 = eigvals.iter().product();

    // Real matrices have real determinants (imaginary part is numerical noise)
    Ok(det.re)
}

/// Matrix exponential exp(A) using scaling and squaring with Padé approximation.
///
/// This is the standard algorithm for computing matrix exponentials:
/// 1. Scale the matrix so the norm is small: A_s = A / 2^s
/// 2. Apply Padé approximation to exp(A_s)
/// 3. Square the result s times: exp(A) = (exp(A_s))^(2^s)
///
/// Uses [6,6] Padé approximant for high accuracy.
///
/// ## Example
/// ```ignore
/// use kleis::numerical::expm;
/// let a = vec![0.0, 1.0, -1.0, 0.0]; // rotation matrix generator
/// let exp_a = expm(&a, 2).unwrap();
/// // exp_a ≈ [[cos(1), sin(1)], [-sin(1), cos(1)]]
/// ```
pub fn expm(matrix: &[f64], n: usize) -> Result<Vec<f64>, NumericalError> {
    if matrix.len() != n * n {
        return Err(NumericalError::DimensionMismatch {
            expected: (n, n),
            got: (matrix.len() / n, matrix.len() % n),
        });
    }

    let a = Array2::from_shape_vec((n, n), matrix.to_vec())
        .map_err(|e| NumericalError::ComputationFailed(e.to_string()))?;

    // Compute the infinity-norm of the matrix (max row sum)
    let mut norm: f64 = 0.0;
    for i in 0..n {
        let mut row_sum: f64 = 0.0;
        for j in 0..n {
            row_sum += a[(i, j)].abs();
        }
        norm = norm.max(row_sum);
    }

    // Determine scaling factor s such that ||A/2^s|| < 0.5
    // This ensures good convergence of the Padé approximant
    let s = if norm > 0.5 {
        ((2.0 * norm).ln() / std::f64::consts::LN_2).ceil().max(0.0) as u32
    } else {
        0
    };
    let scale = 2.0_f64.powi(s as i32);

    // Scale the matrix
    let a_scaled = &a / scale;

    // Compute powers of A
    let a2 = a_scaled.dot(&a_scaled);
    let a4 = a2.dot(&a2);
    let a6 = a2.dot(&a4);

    // Identity matrix
    let eye = Array2::eye(n);

    // Padé [6,6] coefficients (from Higham's book)
    // These are b_k = (2p-k)! * p! / ((2p)! * k! * (p-k)!) for p=6
    let b = [
        1.0,
        1.0 / 2.0,
        1.0 / 9.0,
        1.0 / 72.0,
        1.0 / 1008.0,
        1.0 / 30240.0,
        1.0 / 665280.0,
    ];

    // U = A * (b[1]*I + b[3]*A^2 + b[5]*A^4)
    // V = b[0]*I + b[2]*A^2 + b[4]*A^4 + b[6]*A^6
    // exp(A) ≈ (V - U)^(-1) * (V + U)

    let u = a_scaled.dot(&(&eye * b[1] + &(&a2 * b[3]) + &(&a4 * b[5])));
    let v = &eye * b[0] + &(&a2 * b[2]) + &(&a4 * b[4]) + &(&a6 * b[6]);

    let v_minus_u = &v - &u;
    let v_plus_u = &v + &u;

    // Solve (V - U) * result = (V + U)  =>  result = (V - U)^(-1) * (V + U)
    let v_minus_u_inv = v_minus_u
        .inv()
        .map_err(|e| NumericalError::ComputationFailed(format!("Matrix inverse failed: {}", e)))?;
    let mut result = v_minus_u_inv.dot(&v_plus_u);

    // Squaring: result = result^(2^s)
    for _ in 0..s {
        result = result.dot(&result);
    }

    // Convert to row-major Vec
    Ok(result.into_raw_vec())
}

/// Schur decomposition via nalgebra (fallback/testing).
/// Returns (U, T) where A = U * T * U^H, T is upper quasi-triangular.
///
/// ## ⚠️ WARNING: This is NOT LAPACK Schur
///
/// Uses nalgebra's Rust implementation. **Numerical behavior may differ from LAPACK.**
///
/// For production control theory (CARE/DARE/Lyapunov):
/// - Use `schur_lapack()` for LAPACK dgees
/// - Use `schur_reorder_stable_*()` for eigenvalue ordering
///
/// This function is suitable for:
/// - Small matrices (< 100×100)
/// - Testing and development
/// - Environments where LAPACK is unavailable
///
/// Note: nalgebra may use LAPACK internally depending on feature flags.
/// Do not rely on this being "pure Rust" without checking your Cargo configuration.
pub fn schur_nalgebra(matrix: &[f64], n: usize) -> Result<(Vec<f64>, Vec<f64>), NumericalError> {
    use nalgebra::DMatrix;

    if matrix.len() != n * n {
        return Err(NumericalError::DimensionMismatch {
            expected: (n, n),
            got: (matrix.len() / n, matrix.len() % n),
        });
    }

    // nalgebra uses column-major order, so we need to transpose
    let mut col_major = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            col_major[j * n + i] = matrix[i * n + j];
        }
    }

    let mat = DMatrix::from_vec(n, n, col_major);
    let schur_result = mat.schur();
    let (q, t) = schur_result.unpack();

    // Convert back to row-major
    let mut u_row_major = vec![0.0; n * n];
    let mut t_row_major = vec![0.0; n * n];

    for i in 0..n {
        for j in 0..n {
            u_row_major[i * n + j] = q[(i, j)];
            t_row_major[i * n + j] = t[(i, j)];
        }
    }

    Ok((u_row_major, t_row_major))
}

/// Alias for `schur_nalgebra()` - provided for backwards compatibility.
/// See `schur_nalgebra()` for details and warnings.
#[inline]
pub fn schur(matrix: &[f64], n: usize) -> Result<(Vec<f64>, Vec<f64>), NumericalError> {
    schur_nalgebra(matrix, n)
}

/// Result of a real Schur decomposition A = U*T*U^T.
/// T is quasi-upper-triangular (1x1 and 2x2 blocks).
#[derive(Debug, Clone)]
pub struct SchurResult {
    /// Schur vectors (orthogonal matrix U)
    pub u: Vec<f64>,
    /// Schur form (quasi-upper-triangular matrix T)
    pub t: Vec<f64>,
    /// Real parts of eigenvalues
    pub wr: Vec<f64>,
    /// Imaginary parts of eigenvalues
    pub wi: Vec<f64>,
    /// Matrix dimension
    pub n: usize,
}

/// LAPACK Schur decomposition (dgees).
///
/// **This is what you need for control theory:**
/// - CARE/DARE solvers
/// - Lyapunov equation solvers
/// - Pole placement
/// - Stability analysis
///
/// Returns SchurResult with U, T, and eigenvalues.
/// Uses LAPACK dgees with no eigenvalue ordering (sort='N').
pub fn schur_lapack(matrix: &[f64], n: usize) -> Result<SchurResult, NumericalError> {
    use lapack::dgees;

    if matrix.len() != n * n {
        return Err(NumericalError::DimensionMismatch {
            expected: (n, n),
            got: (matrix.len() / n, matrix.len() % n),
        });
    }

    let n_i32 = n as i32;

    // Convert to column-major for LAPACK (dgees overwrites with T)
    let mut a_cm = vec![0.0f64; n * n];
    for i in 0..n {
        for j in 0..n {
            a_cm[j * n + i] = matrix[i * n + j];
        }
    }

    let lda = n_i32;

    // Outputs
    let mut wr = vec![0.0f64; n];
    let mut wi = vec![0.0f64; n];

    // Schur vectors (VS) in column-major
    let mut vs = vec![0.0f64; n * n];
    let ldvs = n_i32;

    // sdim = number of selected eigenvalues (not used with sort='N')
    let mut sdim: i32 = 0;

    // Workspace query
    let mut work = vec![0.0f64; 1];
    let mut bwork = vec![0_i32; n]; // Logical array for LAPACK (only used if sort='S')
    let mut info: i32 = 0;

    // Workspace query: lwork = -1
    unsafe {
        dgees(
            b'V', // jobvs: compute Schur vectors
            b'N', // sort: no ordering
            None, // select: not needed with sort='N'
            n_i32, &mut a_cm, lda, &mut sdim, &mut wr, &mut wi, &mut vs, ldvs, &mut work,
            -1, // lwork query
            &mut bwork, &mut info,
        );
    }

    if info != 0 {
        return Err(NumericalError::ComputationFailed(format!(
            "dgees workspace query failed, info={}",
            info
        )));
    }

    let lwork = work[0].max(1.0) as i32;
    let mut work = vec![0.0f64; lwork as usize];
    info = 0;

    // Actual computation
    unsafe {
        dgees(
            b'V', b'N', None, n_i32, &mut a_cm, lda, &mut sdim, &mut wr, &mut wi, &mut vs, ldvs,
            &mut work, lwork, &mut bwork, &mut info,
        );
    }

    if info != 0 {
        return Err(NumericalError::ComputationFailed(format!(
            "dgees failed, info={}",
            info
        )));
    }

    // Convert back to row-major
    let mut t_row = vec![0.0f64; n * n];
    let mut u_row = vec![0.0f64; n * n];

    for i in 0..n {
        for j in 0..n {
            t_row[i * n + j] = a_cm[j * n + i];
            u_row[i * n + j] = vs[j * n + i];
        }
    }

    Ok(SchurResult {
        u: u_row,
        t: t_row,
        wr,
        wi,
        n,
    })
}

/// Compute eigenvalues from Schur form.
/// This is useful when you need both Schur form and eigenvalues.
pub fn schur_eigenvalues(t_matrix: &[f64], n: usize) -> Result<Vec<(f64, f64)>, NumericalError> {
    if t_matrix.len() != n * n {
        return Err(NumericalError::DimensionMismatch {
            expected: (n, n),
            got: (t_matrix.len() / n, t_matrix.len() % n),
        });
    }

    // In real Schur form, eigenvalues are on diagonal (1x1 blocks) or
    // come from 2x2 blocks on the diagonal
    let mut eigvals = Vec::with_capacity(n);
    let mut i = 0;

    while i < n {
        if i + 1 < n {
            // Check if there's a 2x2 block (subdiagonal element is non-zero)
            let subdiag = t_matrix[(i + 1) * n + i]; // T[i+1, i]
            if subdiag.abs() > 1e-14 {
                // 2x2 block: compute complex eigenvalues
                let a11 = t_matrix[i * n + i];
                let a12 = t_matrix[i * n + i + 1];
                let a21 = subdiag;
                let a22 = t_matrix[(i + 1) * n + i + 1];

                let trace = a11 + a22;
                let det = a11 * a22 - a12 * a21;
                let disc = trace * trace - 4.0 * det;

                if disc < 0.0 {
                    // Complex conjugate pair
                    let re = trace / 2.0;
                    let im = (-disc).sqrt() / 2.0;
                    eigvals.push((re, im));
                    eigvals.push((re, -im));
                } else {
                    // Two real eigenvalues (shouldn't happen in proper Schur form)
                    eigvals.push(((trace + disc.sqrt()) / 2.0, 0.0));
                    eigvals.push(((trace - disc.sqrt()) / 2.0, 0.0));
                }
                i += 2;
                continue;
            }
        }
        // 1x1 block: real eigenvalue
        eigvals.push((t_matrix[i * n + i], 0.0));
        i += 1;
    }

    Ok(eigvals)
}

// Note: QZ decomposition (generalized Schur) requires direct LAPACK calls via lax
// These are placeholders for future implementation when control toolkit needs them

/// QZ decomposition (generalized Schur).
/// For matrix pencil (A, B), finds Q, Z such that:
/// - Q^H * A * Z = S (upper triangular)
/// - Q^H * B * Z = T (upper triangular)
///
/// ## Implementation Notes
///
/// - **LAPACK routine**: `dgges` (real), `zgges` (complex)
/// - **Use case**: Generalized eigenvalue problems, descriptor systems
/// - **Rust crates**: `lax` or `lapack` crate for direct access
///
/// Planned for Control Toolkit (pole placement, descriptor systems).
pub fn qz(
    _a: &[f64],
    _b: &[f64],
    _n: usize,
) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>), NumericalError> {
    // TODO: Implement via lax::dgges when control toolkit is built
    Err(NumericalError::ComputationFailed(
        "QZ decomposition not yet implemented - requires direct LAPACK dgges call".to_string(),
    ))
}

/// Reorder Schur form to move selected eigenvalues to top-left.
///
/// Uses LAPACK `dtrsen` directly.
///
/// ## Arguments
/// - `schur_result`: Result from `schur_lapack()`
/// - `select`: Boolean array indicating which eigenvalues to move to top-left
///
/// ## Returns
/// Updated SchurResult with reordered T and U.
pub fn schur_reorder(
    mut result: SchurResult,
    select: &[bool],
) -> Result<SchurResult, NumericalError> {
    use lapack::dtrsen;

    let n = result.n;
    if select.len() != n {
        return Err(NumericalError::DimensionMismatch {
            expected: (n, 1),
            got: (select.len(), 1),
        });
    }

    let n_i32 = n as i32;

    // Convert select to i32 for LAPACK
    let select_i32: Vec<i32> = select.iter().map(|&b| if b { 1 } else { 0 }).collect();

    // Convert to column-major for LAPACK
    let mut t_cm = vec![0.0f64; n * n];
    let mut u_cm = vec![0.0f64; n * n];
    for i in 0..n {
        for j in 0..n {
            t_cm[j * n + i] = result.t[i * n + j];
            u_cm[j * n + i] = result.u[i * n + j];
        }
    }

    let ldt = n_i32;
    let ldq = n_i32;

    // dtrsen outputs
    let mut wr = vec![0.0f64; n];
    let mut wi = vec![0.0f64; n];
    let mut m: i32 = 0;
    let mut s = vec![0.0f64; 1]; // reciprocal condition number
    let mut sep = vec![0.0f64; 1]; // separation
    let mut info: i32 = 0;

    // Workspace query
    let mut work = vec![0.0f64; 1];
    let mut iwork = vec![0_i32; 1];

    unsafe {
        dtrsen(
            b'N', // job: no condition numbers
            b'V', // compq: update Q
            &select_i32,
            n_i32,
            &mut t_cm,
            ldt,
            &mut u_cm,
            ldq,
            &mut wr,
            &mut wi,
            &mut m,
            &mut s,
            &mut sep,
            &mut work,
            -1,
            &mut iwork,
            -1,
            &mut info,
        );
    }

    if info != 0 {
        return Err(NumericalError::ComputationFailed(format!(
            "dtrsen workspace query failed, info={}",
            info
        )));
    }

    let lwork = work[0].max(1.0) as i32;
    let liwork = iwork[0].max(1) as i32;
    let mut work = vec![0.0f64; lwork as usize];
    let mut iwork = vec![0_i32; liwork as usize];
    info = 0;

    unsafe {
        dtrsen(
            b'N',
            b'V',
            &select_i32,
            n_i32,
            &mut t_cm,
            ldt,
            &mut u_cm,
            ldq,
            &mut wr,
            &mut wi,
            &mut m,
            &mut s,
            &mut sep,
            &mut work,
            lwork,
            &mut iwork,
            liwork,
            &mut info,
        );
    }

    if info != 0 {
        return Err(NumericalError::ComputationFailed(format!(
            "dtrsen failed, info={}",
            info
        )));
    }

    // Convert back to row-major
    for i in 0..n {
        for j in 0..n {
            result.t[i * n + j] = t_cm[j * n + i];
            result.u[i * n + j] = u_cm[j * n + i];
        }
    }
    result.wr = wr;
    result.wi = wi;

    Ok(result)
}

/// Default tolerance for stability boundary decisions.
/// Eigenvalues within this tolerance of the boundary are treated as on the boundary.
pub const STABILITY_TOLERANCE: f64 = 1e-12;

/// Validate complex conjugate pair structure in eigenvalue arrays.
/// Returns error if wi[i] != 0 but wi[i+1] != -wi[i].
fn validate_conjugate_pair(wi: &[f64], i: usize, n: usize) -> Result<(), NumericalError> {
    if i + 1 >= n {
        return Err(NumericalError::ComputationFailed(
            "unexpected complex eigenvalue at last index".to_string(),
        ));
    }
    // Complex conjugate pairs should have wi[i] = -wi[i+1]
    let tol = 1e-10;
    if (wi[i] + wi[i + 1]).abs() > tol {
        return Err(NumericalError::ComputationFailed(format!(
            "malformed complex conjugate pair: wi[{}]={}, wi[{}]={}",
            i,
            wi[i],
            i + 1,
            wi[i + 1]
        )));
    }
    Ok(())
}

/// Reorder Schur to move stable eigenvalues to top-left (continuous-time).
///
/// Continuous-time stability: Re(λ) < -eps
///
/// Uses LAPACK `dtrsen`.
///
/// ## Arguments
/// - `result`: Schur decomposition from `schur_lapack()`
/// - `eps`: Tolerance for stability boundary (default: use `STABILITY_TOLERANCE`)
pub fn schur_reorder_stable_continuous(result: SchurResult) -> Result<SchurResult, NumericalError> {
    schur_reorder_stable_continuous_with_tolerance(result, STABILITY_TOLERANCE)
}

/// Reorder Schur with custom tolerance for continuous-time stability.
pub fn schur_reorder_stable_continuous_with_tolerance(
    result: SchurResult,
    eps: f64,
) -> Result<SchurResult, NumericalError> {
    let n = result.n;

    // Build select array: true for stable eigenvalues (Re(λ) < -eps)
    let mut select = vec![false; n];
    let mut i = 0;
    while i < n {
        let stable = result.wr[i] < -eps;
        if result.wi[i] != 0.0 {
            // Complex conjugate pair: validate and select both
            validate_conjugate_pair(&result.wi, i, n)?;
            select[i] = stable;
            select[i + 1] = stable;
            i += 2;
        } else {
            select[i] = stable;
            i += 1;
        }
    }

    schur_reorder(result, &select)
}

/// Reorder Schur to move stable eigenvalues to top-left (discrete-time).
///
/// Discrete-time stability: |λ| < 1 - eps
///
/// Uses LAPACK `dtrsen`.
///
/// ## Arguments
/// - `result`: Schur decomposition from `schur_lapack()`
/// - `eps`: Tolerance for stability boundary (default: use `STABILITY_TOLERANCE`)
pub fn schur_reorder_stable_discrete(result: SchurResult) -> Result<SchurResult, NumericalError> {
    schur_reorder_stable_discrete_with_tolerance(result, STABILITY_TOLERANCE)
}

/// Reorder Schur with custom tolerance for discrete-time stability.
pub fn schur_reorder_stable_discrete_with_tolerance(
    result: SchurResult,
    eps: f64,
) -> Result<SchurResult, NumericalError> {
    let n = result.n;

    // Build select array: true for stable eigenvalues (|λ| < 1 - eps)
    let mut select = vec![false; n];
    let mut i = 0;
    while i < n {
        let magnitude = (result.wr[i].powi(2) + result.wi[i].powi(2)).sqrt();
        let stable = magnitude < 1.0 - eps;
        if result.wi[i] != 0.0 {
            // Complex conjugate pair: validate and select both
            validate_conjugate_pair(&result.wi, i, n)?;
            select[i] = stable;
            select[i + 1] = stable;
            i += 2;
        } else {
            select[i] = stable;
            i += 1;
        }
    }

    schur_reorder(result, &select)
}

/// Reorder QZ form to move selected eigenvalues to top-left.
///
/// ## Implementation Notes
///
/// - **LAPACK routine**: `dtgsen` (real), `ztgsen` (complex)
/// - **Use case**: Separate finite/infinite eigenvalues, pole placement
pub fn qz_reorder(
    _q: &[f64],
    _z: &[f64],
    _s: &[f64],
    _t: &[f64],
    _n: usize,
    _select: &[bool],
) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>), NumericalError> {
    Err(NumericalError::ComputationFailed(
        "QZ reordering requires LAPACK dtgsen - not yet implemented".to_string(),
    ))
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

    // === Helper functions for Schur tests ===

    fn frob_norm(a: &[f64], n: usize) -> f64 {
        a.iter().map(|x| x * x).sum::<f64>().sqrt()
    }

    fn matmul(a: &[f64], b: &[f64], n: usize) -> Vec<f64> {
        let mut c = vec![0.0; n * n];
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    c[i * n + j] += a[i * n + k] * b[k * n + j];
                }
            }
        }
        c
    }

    fn transpose(a: &[f64], n: usize) -> Vec<f64> {
        let mut t = vec![0.0; n * n];
        for i in 0..n {
            for j in 0..n {
                t[j * n + i] = a[i * n + j];
            }
        }
        t
    }

    fn eye(n: usize) -> Vec<f64> {
        let mut i = vec![0.0; n * n];
        for k in 0..n {
            i[k * n + k] = 1.0;
        }
        i
    }

    fn sub(a: &[f64], b: &[f64]) -> Vec<f64> {
        a.iter().zip(b.iter()).map(|(x, y)| x - y).collect()
    }

    #[test]
    fn test_schur_identity() {
        // Schur of identity: U = I, T = I
        let m = vec![1.0, 0.0, 0.0, 1.0];
        let (u, t) = schur(&m, 2).unwrap();

        // T should be close to identity
        assert!((t[0] - 1.0).abs() < 1e-10);
        assert!((t[3] - 1.0).abs() < 1e-10);
        assert!(t[1].abs() < 1e-10);
        assert!(t[2].abs() < 1e-10);

        // U should be orthogonal
        let prod00 = u[0] * u[0] + u[1] * u[1];
        assert!((prod00 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_schur_diagonal() {
        let m = vec![2.0, 0.0, 0.0, 3.0];
        let (_, t) = schur(&m, 2).unwrap();

        let mut diag = vec![t[0], t[3]];
        diag.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert!((diag[0] - 2.0).abs() < 1e-10);
        assert!((diag[1] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_schur_eigenvalues_from_matrix() {
        let m = vec![2.0, 1.0, 0.0, 3.0];
        let (_, t) = schur(&m, 2).unwrap();
        let eigvals = schur_eigenvalues(&t, 2).unwrap();

        let mut reals: Vec<f64> = eigvals.iter().map(|(re, _)| *re).collect();
        reals.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert!((reals[0] - 2.0).abs() < 1e-10);
        assert!((reals[1] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_schur_lapack_reconstruction_and_orthogonality() {
        // Test LAPACK Schur with reconstruction and orthogonality checks
        let n = 4;
        // Build deterministic test matrix
        let mut a = vec![0.0; n * n];
        for i in 0..n {
            for j in 0..n {
                a[i * n + j] = ((i as f64 + 1.0) * 0.7) - ((j as f64 + 1.0) * 0.3);
                if i == j {
                    a[i * n + j] += 2.0;
                }
            }
        }

        let sd = schur_lapack(&a, n).expect("schur_lapack failed");

        // Check reconstruction: A ≈ U*T*U^T
        let ut = matmul(&sd.u, &sd.t, n);
        let u_t = transpose(&sd.u, n);
        let utu_t = matmul(&ut, &u_t, n);
        let residual = sub(&a, &utu_t);

        let rel = frob_norm(&residual, n) / frob_norm(&a, n).max(1.0);
        assert!(
            rel < 1e-10,
            "relative reconstruction error too big: {}",
            rel
        );

        // Check orthogonality: U^T * U ≈ I
        let utu = matmul(&transpose(&sd.u, n), &sd.u, n);
        let ortho_err = frob_norm(&sub(&utu, &eye(n)), n);
        assert!(
            ortho_err < 1e-10,
            "orthogonality error too big: {}",
            ortho_err
        );
    }

    #[test]
    fn test_schur_reorder_stable_continuous() {
        // Test reordering with mixed stable/unstable eigenvalues
        let n = 4;
        // Build matrix with known stable/unstable modes
        let mut a = vec![0.0; n * n];
        // Diagonal: first 2 stable (negative), last 2 unstable (positive)
        a[0] = -2.0;
        a[5] = -1.0;
        a[10] = 0.5;
        a[15] = 1.0;
        // Add some off-diagonal coupling
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    a[i * n + j] = 0.01 * ((i as f64) - (j as f64));
                }
            }
        }

        let sd = schur_lapack(&a, n).expect("schur_lapack failed");
        let sd2 = schur_reorder_stable_continuous(sd).expect("reorder failed");

        // Verify decomposition still valid
        let ut = matmul(&sd2.u, &sd2.t, n);
        let u_t = transpose(&sd2.u, n);
        let utu_t = matmul(&ut, &u_t, n);
        let residual = sub(&a, &utu_t);

        let rel = frob_norm(&residual, n) / frob_norm(&a, n).max(1.0);
        assert!(
            rel < 1e-10,
            "relative reconstruction error after reorder: {}",
            rel
        );

        // Verify U still orthogonal
        let utu = matmul(&transpose(&sd2.u, n), &sd2.u, n);
        let ortho_err = frob_norm(&sub(&utu, &eye(n)), n);
        assert!(
            ortho_err < 1e-10,
            "orthogonality error after reorder: {}",
            ortho_err
        );

        // Verify stable eigenvalues come first (using same tolerance as reorder)
        let mut found_unstable = false;
        for i in 0..n {
            if sd2.wr[i] >= -STABILITY_TOLERANCE {
                found_unstable = true;
            }
            if found_unstable && sd2.wr[i] < -STABILITY_TOLERANCE {
                panic!(
                    "found stable eigenvalue after unstable: wr[{}] = {}",
                    i, sd2.wr[i]
                );
            }
        }
    }

    #[test]
    fn test_schur_reorder_stable_discrete() {
        // Test discrete-time reordering: stable = |λ| < 1
        let n = 4;
        // Build matrix with eigenvalues both inside and outside unit circle
        // Using a diagonal matrix for predictable eigenvalues
        let mut a = vec![0.0; n * n];
        a[0] = 0.5; // |0.5| < 1: stable
        a[5] = 0.8; // |0.8| < 1: stable
        a[10] = 1.2; // |1.2| > 1: unstable
        a[15] = 2.0; // |2.0| > 1: unstable

        let sd = schur_lapack(&a, n).expect("schur_lapack failed");
        let sd2 = schur_reorder_stable_discrete(sd).expect("reorder failed");

        // Verify decomposition still valid
        let ut = matmul(&sd2.u, &sd2.t, n);
        let u_t = transpose(&sd2.u, n);
        let utu_t = matmul(&ut, &u_t, n);
        let residual = sub(&a, &utu_t);

        let rel = frob_norm(&residual, n) / frob_norm(&a, n).max(1.0);
        assert!(
            rel < 1e-10,
            "relative reconstruction error after discrete reorder: {}",
            rel
        );

        // Verify stable eigenvalues (|λ| < 1) come first
        let mut found_unstable = false;
        for i in 0..n {
            let mag = (sd2.wr[i].powi(2) + sd2.wi[i].powi(2)).sqrt();
            if mag >= 1.0 - STABILITY_TOLERANCE {
                found_unstable = true;
            }
            if found_unstable && mag < 1.0 - STABILITY_TOLERANCE {
                panic!(
                    "found stable eigenvalue after unstable: |λ[{}]| = {}",
                    i, mag
                );
            }
        }
    }

    #[test]
    fn test_qz_not_implemented() {
        let a = vec![1.0, 0.0, 0.0, 1.0];
        let b = vec![1.0, 0.0, 0.0, 1.0];
        let result = qz(&a, &b, 2);
        assert!(result.is_err());
    }
}
