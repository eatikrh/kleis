//! Numerical linear algebra operations backed by BLAS/LAPACK.
//!
//! This module provides high-performance numerical operations:
//! - Eigenvalues/eigenvectors
//! - SVD (Singular Value Decomposition)
//! - Solve Ax = b
//! - Matrix inverse
//! - LU/QR/Cholesky factorization
//!
//! On macOS: Uses Apple Accelerate (pre-installed, M1/M2/M3 optimized)
//! On Linux/Windows: Uses OpenBLAS

// Force linking of BLAS/LAPACK backends
extern crate blas_src;
extern crate lapack_src;

pub mod backend;
pub use backend::*;
