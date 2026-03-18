//! Integration tests for LAPACK-backed numerical operations.
//!
//! These tests verify that the LAPACK operations work correctly
//! when invoked through the Kleis evaluator.

#![cfg(feature = "numerical")]

use kleis::evaluator::Evaluator;
use kleis::kleis_parser::parse_kleis_program;

fn eval_expr(code: &str) -> String {
    let mut eval = Evaluator::new();

    // Parse the full program
    let full_code = format!("define result = {}", code);
    let program = parse_kleis_program(&full_code).expect("Failed to parse");
    eval.load_program(&program).expect("Failed to load");

    // Get result via function closure (defines are stored as zero-arg functions)
    let closure = eval.get_function("result").expect("No result definition");
    let body = closure.body.clone();
    let evaluated = eval.eval_concrete(&body).expect("Failed to evaluate");
    format!("{:?}", evaluated)
}

#[test]
fn test_eigenvalues_identity() {
    // Identity matrix has all eigenvalues = 1
    let result = eval_expr("eigenvalues(Matrix(2, 2, [1, 0, 0, 1]))");
    assert!(
        result.contains("1"),
        "Expected eigenvalue 1, got: {}",
        result
    );
}

#[test]
fn test_eigenvalues_diagonal() {
    // Diagonal matrix [[2, 0], [0, 3]] has eigenvalues 2 and 3
    let result = eval_expr("eigenvalues(Matrix(2, 2, [2, 0, 0, 3]))");
    assert!(
        result.contains("2") && result.contains("3"),
        "Expected eigenvalues 2 and 3, got: {}",
        result
    );
}

#[test]
fn test_solve_identity() {
    // Solving Ix = b gives x = b
    let result = eval_expr("solve(Matrix(2, 2, [1, 0, 0, 1]), [3, 4])");
    assert!(
        result.contains("3") && result.contains("4"),
        "Expected solution [3, 4], got: {}",
        result
    );
}

#[test]
fn test_solve_simple() {
    // Solve [[2, 1], [1, 3]] x = [8, 13]
    // Solution: x = [1, 4] (check: 2*1 + 1*4 = 6... let me recalculate)
    // Actually: [[2, 1], [1, 3]] * [1, 4] = [6, 13] - doesn't work
    // Use: [[2, 0], [0, 3]] * [1, 2] = [2, 6]
    let result = eval_expr("solve(Matrix(2, 2, [2, 0, 0, 3]), [2, 6])");
    assert!(
        result.contains("1") && result.contains("2"),
        "Expected solution [1, 2], got: {}",
        result
    );
}

#[test]
fn test_inv_identity() {
    // Inverse of identity is identity
    let result = eval_expr("inv(Matrix(2, 2, [1, 0, 0, 1]))");
    // Should contain 1s and 0s
    assert!(
        result.contains("1") && result.contains("0"),
        "Expected identity inverse, got: {}",
        result
    );
}

#[test]
fn test_inv_2x2() {
    // [[1, 2], [3, 4]] inverse is [[-2, 1], [1.5, -0.5]]
    let result = eval_expr("inv(Matrix(2, 2, [1, 2, 3, 4]))");
    assert!(
        result.contains("-2") || result.contains("1.5"),
        "Expected inverse, got: {}",
        result
    );
}

#[test]
fn test_singular_values_identity() {
    // SVD of identity: all singular values = 1
    let result = eval_expr("singular_values(Matrix(2, 2, [1, 0, 0, 1]))");
    assert!(
        result.contains("1"),
        "Expected singular values 1, got: {}",
        result
    );
}

#[test]
fn test_rank_full() {
    // Full rank matrix
    let result = eval_expr("rank(Matrix(2, 2, [1, 0, 0, 1]))");
    assert!(result.contains("2"), "Expected rank 2, got: {}", result);
}

#[test]
fn test_rank_deficient() {
    // Rank-deficient matrix: [[1, 2], [2, 4]] has rank 1
    let result = eval_expr("rank(Matrix(2, 2, [1, 2, 2, 4]))");
    assert!(result.contains("1"), "Expected rank 1, got: {}", result);
}

#[test]
fn test_cond_identity() {
    // Condition number of identity is 1
    let result = eval_expr("cond(Matrix(2, 2, [1, 0, 0, 1]))");
    assert!(
        result.contains("1"),
        "Expected condition number 1, got: {}",
        result
    );
}

#[test]
fn test_det_lapack() {
    // Determinant via LAPACK
    let result = eval_expr("det_lapack(Matrix(2, 2, [1, 2, 3, 4]))");
    // det = 1*4 - 2*3 = -2
    assert!(result.contains("-2"), "Expected det = -2, got: {}", result);
}

#[test]
fn test_qr_decomposition() {
    // QR decomposition returns [Q, R]
    let result = eval_expr("qr(Matrix(2, 2, [1, 0, 0, 1]))");
    assert!(
        result.contains("List"),
        "Expected QR to return [Q, R], got: {}",
        result
    );
}

#[test]
fn test_cholesky_positive_definite() {
    // Cholesky of [[4, 2], [2, 2]] = [[2, 0], [1, 1]]
    let result = eval_expr("cholesky(Matrix(2, 2, [4, 2, 2, 2]))");
    assert!(
        result.contains("2") && result.contains("1"),
        "Expected Cholesky factor, got: {}",
        result
    );
}

#[test]
fn test_norm_frobenius() {
    // Frobenius norm of [[1, 0], [0, 1]] = sqrt(2) ≈ 1.414
    let result = eval_expr("norm(Matrix(2, 2, [1, 0, 0, 1]))");
    assert!(
        result.contains("1.41") || result.contains("1.414"),
        "Expected Frobenius norm ~1.414, got: {}",
        result
    );
}

#[test]
fn test_svd_decomposition() {
    // SVD returns [U, S, Vt]
    let result = eval_expr("svd(Matrix(2, 2, [1, 0, 0, 1]))");
    assert!(
        result.contains("List"),
        "Expected SVD to return [U, S, Vt], got: {}",
        result
    );
}

#[test]
fn test_eig_full() {
    // Full eigendecomposition returns [eigenvalues, eigenvectors]
    let result = eval_expr("eig(Matrix(2, 2, [1, 0, 0, 1]))");
    assert!(
        result.contains("List"),
        "Expected eig to return [vals, vecs], got: {}",
        result
    );
}

#[test]
fn test_eigenvalues_3x3() {
    // Test larger matrix
    let result = eval_expr("eigenvalues(Matrix(3, 3, [1, 0, 0, 0, 2, 0, 0, 0, 3]))");
    assert!(
        result.contains("1") && result.contains("2") && result.contains("3"),
        "Expected eigenvalues 1, 2, 3, got: {}",
        result
    );
}

#[test]
fn test_solve_3x3() {
    // Solve a 3x3 system
    let result = eval_expr("solve(Matrix(3, 3, [1, 0, 0, 0, 1, 0, 0, 0, 1]), [1, 2, 3])");
    assert!(
        result.contains("1") && result.contains("2") && result.contains("3"),
        "Expected solution [1, 2, 3], got: {}",
        result
    );
}
