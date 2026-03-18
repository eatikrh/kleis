//! Comprehensive type checking tests for matrix operations
//! Tests various scenarios to shake out edge cases in dimension inference

use kleis::kleis_parser::parse_kleis_program;
use kleis::type_checker::TypeChecker;

/// Helper to run type checking on code and return result
fn type_check_code(code: &str) -> Result<(), String> {
    let mut checker =
        TypeChecker::with_stdlib().map_err(|e| format!("Stdlib load failed: {}", e))?;
    let program = parse_kleis_program(code).map_err(|e| format!("Parse error: {:?}", e))?;

    for item in &program.items {
        if let kleis::kleis_ast::TopLevel::FunctionDef(func_def) = item {
            checker.check_function_def(func_def)?;
        }
    }
    Ok(())
}

#[test]
fn test_simple_matrix_multiply() {
    let code = r#"
define test1 =
  let A : Matrix(3, 3, ℝ) = matrix(1,2,3,4,5,6,7,8,9) in
  let B : Matrix(3, 3, ℝ) = matrix(9,8,7,6,5,4,3,2,1) in
  multiply(A, B)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Simple 3x3 multiply should work"
    );
}

#[test]
fn test_different_dimensions_multiply() {
    let code = r#"
define test2 =
  let A : Matrix(2, 3, ℝ) = matrix(1,2,3,4,5,6) in
  let B : Matrix(3, 4, ℝ) = matrix(1,2,3,4,5,6,7,8,9,10,11,12) in
  multiply(A, B)
"#;
    // Result should be Matrix(2, 4, ℝ)
    assert!(
        type_check_code(code).is_ok(),
        "2x3 * 3x4 multiply should work"
    );
}

#[test]
fn test_matrix_addition() {
    let code = r#"
define test3 =
  let A : Matrix(2, 2, ℝ) = matrix(1,2,3,4) in
  let B : Matrix(2, 2, ℝ) = matrix(5,6,7,8) in
  matrix_add(A, B)
"#;
    assert!(type_check_code(code).is_ok(), "Matrix addition should work");
}

#[test]
fn test_matrix_transpose() {
    let code = r#"
define test4 =
  let A : Matrix(2, 3, ℝ) = matrix(1,2,3,4,5,6) in
  transpose(A)
"#;
    // Result should be Matrix(3, 2, ℝ)
    assert!(type_check_code(code).is_ok(), "Transpose should work");
}

#[test]
fn test_chained_operations() {
    let code = r#"
define test5 =
  let A : Matrix(2, 2, ℝ) = matrix(1,2,3,4) in
  let B : Matrix(2, 2, ℝ) = matrix(5,6,7,8) in
  let C : Matrix(2, 2, ℝ) = multiply(A, B) in
  transpose(C)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Chained multiply then transpose should work"
    );
}

#[test]
fn test_complex_matrix() {
    let code = r#"
define test6 =
  let A : Matrix(2, 2, ℂ) = matrix(complex(1,0), complex(0,1), complex(0,-1), complex(1,0)) in
  A
"#;
    assert!(type_check_code(code).is_ok(), "Complex matrix should work");
}

#[test]
fn test_determinant() {
    let code = r#"
define test7 =
  let A : Matrix(3, 3, ℝ) = matrix(1,2,3,4,5,6,7,8,9) in
  det(A)
"#;
    assert!(type_check_code(code).is_ok(), "Determinant should work");
}

#[test]
fn test_trace() {
    let code = r#"
define test8 =
  let A : Matrix(4, 4, ℝ) = matrix(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1) in
  trace(A)
"#;
    assert!(type_check_code(code).is_ok(), "Trace should work");
}

#[test]
fn test_identity_matrix() {
    let code = r#"
define test9 =
  let I : Matrix(3, 3, ℝ) = identity in
  I
"#;
    assert!(type_check_code(code).is_ok(), "Identity matrix should work");
}

#[test]
fn test_vector_operations() {
    let code = r#"
define test10 =
  let v : Vector(3, ℝ) = vector(1, 2, 3) in
  let w : Vector(3, ℝ) = vector(4, 5, 6) in
  dot(v, w)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Vector dot product should work"
    );
}

#[test]
fn test_matrix_subtraction() {
    let code = r#"
define test11 =
  let A : Matrix(2, 3, ℝ) = matrix(1,2,3,4,5,6) in
  let B : Matrix(2, 3, ℝ) = matrix(6,5,4,3,2,1) in
  matrix_sub(A, B)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Matrix subtraction should work"
    );
}

#[test]
fn test_nested_let_bindings() {
    let code = r#"
define test12 =
  let A : Matrix(2, 2, ℝ) = matrix(1,0,0,1) in
  let B : Matrix(2, 2, ℝ) = matrix(2,0,0,2) in
  let C : Matrix(2, 2, ℝ) = multiply(A, B) in
  let D : Matrix(2, 2, ℝ) = multiply(B, C) in
  let E : Matrix(2, 2, ℝ) = multiply(C, D) in
  det(E)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Nested let bindings with operations should work"
    );
}

#[test]
fn test_lowercase_variable_names() {
    let code = r#"
define test13 =
  let a : Matrix(2, 2, ℝ) = matrix(1,2,3,4) in
  let b : Matrix(2, 2, ℝ) = matrix(5,6,7,8) in
  multiply(a, b)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Lowercase variable names should work"
    );
}

#[test]
fn test_mixed_case_variable_names() {
    let code = r#"
define test14 =
  let MyMatrix : Matrix(3, 3, ℝ) = matrix(1,2,3,4,5,6,7,8,9) in
  let anotherMatrix : Matrix(3, 3, ℝ) = matrix(9,8,7,6,5,4,3,2,1) in
  multiply(MyMatrix, anotherMatrix)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Mixed case variable names should work"
    );
}

#[test]
fn test_matrix_with_scalar_binding() {
    let code = r#"
define test15 =
  let A : Matrix(2, 2, ℝ) = matrix(1,2,3,4) in
  let s : ℝ = 2.0 in
  let B : Matrix(2, 2, ℝ) = matrix(s, s, s, s) in
  multiply(A, B)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Matrix with scalar bindings should work"
    );
}

#[test]
fn test_expression_in_matrix() {
    let code = r#"
define test16 =
  let x : ℝ = 3.14 in
  let A : Matrix(2, 2, ℝ) = matrix(x, x+1, x+2, x+3) in
  A
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Expressions in matrix constructor should work"
    );
}

#[test]
fn test_function_with_matrix_param() {
    // Note: This tests function definition with matrix type
    let code = r#"
define doubleMatrix(M) =
  let A : Matrix(2, 2, ℝ) = M in
  matrix_add(A, A)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Function with matrix parameter should work"
    );
}

#[test]
fn test_block_matrix_style() {
    let code = r#"
define test18 =
  let A11 : Matrix(2, 2, ℝ) = matrix(1,2,3,4) in
  let A12 : Matrix(2, 2, ℝ) = matrix(0,0,0,0) in
  let A21 : Matrix(2, 2, ℝ) = matrix(0,0,0,0) in
  let A22 : Matrix(2, 2, ℝ) = matrix(5,6,7,8) in
  multiply(A11, A22)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Block matrix style should work"
    );
}

#[test]
fn test_many_sequential_operations() {
    let code = r#"
define test19 =
  let M1 : Matrix(2, 2, ℝ) = matrix(1,0,0,1) in
  let M2 : Matrix(2, 2, ℝ) = multiply(M1, M1) in
  let M3 : Matrix(2, 2, ℝ) = multiply(M2, M2) in
  let M4 : Matrix(2, 2, ℝ) = multiply(M3, M3) in
  let M5 : Matrix(2, 2, ℝ) = multiply(M4, M4) in
  let M6 : Matrix(2, 2, ℝ) = multiply(M5, M5) in
  let M7 : Matrix(2, 2, ℝ) = multiply(M6, M6) in
  let M8 : Matrix(2, 2, ℝ) = multiply(M7, M7) in
  det(M8)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Many sequential operations should work"
    );
}

#[test]
fn test_single_element_matrix() {
    let code = r#"
define test20 =
  let A : Matrix(1, 1, ℝ) = matrix(42) in
  det(A)
"#;
    assert!(type_check_code(code).is_ok(), "1x1 matrix should work");
}

// ============= COMPLEX MATRIX TESTS =============

#[test]
fn test_complex_matrix_creation() {
    let code = r#"
define testCmx1 =
  let M : ComplexMatrix(2, 2) = cmat_zero in
  M
"#;
    assert!(
        type_check_code(code).is_ok(),
        "ComplexMatrix zero should work"
    );
}

#[test]
fn test_complex_matrix_from_real() {
    let code = r#"
define testCmx2 =
  let R : Matrix(2, 2, ℝ) = matrix(1,2,3,4) in
  let C : ComplexMatrix(2, 2) = cmat_from_real(R) in
  C
"#;
    assert!(type_check_code(code).is_ok(), "cmat_from_real should work");
}

#[test]
fn test_complex_matrix_from_imag() {
    let code = r#"
define testCmx3 =
  let I : Matrix(2, 2, ℝ) = matrix(1,0,0,1) in
  let C : ComplexMatrix(2, 2) = cmat_from_imag(I) in
  C
"#;
    assert!(type_check_code(code).is_ok(), "cmat_from_imag should work");
}

#[test]
fn test_complex_matrix_real_part() {
    let code = r#"
define testCmx4 =
  let C : ComplexMatrix(3, 3) = cmat_zero in
  let R : Matrix(3, 3, ℝ) = cmat_real(C) in
  R
"#;
    assert!(type_check_code(code).is_ok(), "cmat_real should work");
}

#[test]
fn test_complex_matrix_imag_part() {
    let code = r#"
define testCmx5 =
  let C : ComplexMatrix(3, 3) = cmat_zero in
  let I : Matrix(3, 3, ℝ) = cmat_imag(C) in
  I
"#;
    assert!(type_check_code(code).is_ok(), "cmat_imag should work");
}

#[test]
fn test_complex_matrix_addition() {
    let code = r#"
define testCmx6 =
  let A : ComplexMatrix(2, 2) = cmat_zero in
  let B : ComplexMatrix(2, 2) = cmat_zero in
  let C : ComplexMatrix(2, 2) = cmat_add(A, B) in
  C
"#;
    assert!(type_check_code(code).is_ok(), "cmat_add should work");
}

#[test]
fn test_complex_matrix_subtraction() {
    let code = r#"
define testCmx7 =
  let A : ComplexMatrix(2, 2) = cmat_zero in
  let B : ComplexMatrix(2, 2) = cmat_zero in
  let C : ComplexMatrix(2, 2) = cmat_sub(A, B) in
  C
"#;
    assert!(type_check_code(code).is_ok(), "cmat_sub should work");
}

#[test]
fn test_complex_matrix_conjugate() {
    let code = r#"
define testCmx8 =
  let A : ComplexMatrix(3, 2) = cmat_zero in
  let B : ComplexMatrix(3, 2) = cmat_conj(A) in
  B
"#;
    assert!(type_check_code(code).is_ok(), "cmat_conj should work");
}

#[test]
fn test_complex_matrix_transpose() {
    let code = r#"
define testCmx9 =
  let A : ComplexMatrix(2, 3) = cmat_zero in
  let B : ComplexMatrix(3, 2) = cmat_transpose(A) in
  B
"#;
    assert!(type_check_code(code).is_ok(), "cmat_transpose should work");
}

#[test]
fn test_complex_matrix_dagger() {
    let code = r#"
define testCmx10 =
  let A : ComplexMatrix(2, 4) = cmat_zero in
  let B : ComplexMatrix(4, 2) = cmat_dagger(A) in
  B
"#;
    assert!(type_check_code(code).is_ok(), "cmat_dagger should work");
}

#[test]
fn test_complex_matrix_scale_real() {
    let code = r#"
define testCmx11 =
  let A : ComplexMatrix(3, 3) = cmat_zero in
  let s : ℝ = 2.5 in
  let B : ComplexMatrix(3, 3) = cmat_scale_real(s, A) in
  B
"#;
    assert!(type_check_code(code).is_ok(), "cmat_scale_real should work");
}

// ============= LAPACK-STYLE OPERATIONS =============

#[test]
fn test_matrix_exponential() {
    let code = r#"
define testLapack1 =
  let A : Matrix(3, 3, ℝ) = matrix(1,0,0,0,1,0,0,0,1) in
  let ExpA : Matrix(3, 3, ℝ) = expm(A) in
  ExpA
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Matrix exponential expm should work"
    );
}

#[test]
fn test_matrix_power() {
    let code = r#"
define testLapack2 =
  let A : Matrix(2, 2, ℝ) = matrix(1,1,1,0) in
  let A5 : Matrix(2, 2, ℝ) = mpow(A, 5) in
  A5
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Matrix power mpow should work"
    );
}

#[test]
fn test_complex_matrix_exponential() {
    let code = r#"
define testLapack3 =
  let A : ComplexMatrix(2, 2) = cmat_zero in
  let ExpA : ComplexMatrix(2, 2) = cmat_expm(A) in
  ExpA
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Complex matrix exponential cmat_expm should work"
    );
}

#[test]
fn test_complex_matrix_power() {
    let code = r#"
define testLapack4 =
  let A : ComplexMatrix(2, 2) = cmat_zero in
  let A3 : ComplexMatrix(2, 2) = cmat_mpow(A, 3) in
  A3
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Complex matrix power cmat_mpow should work"
    );
}

#[test]
fn test_complex_matrix_solve() {
    let code = r#"
define testLapack5 =
  let A : ComplexMatrix(3, 3) = cmat_zero in
  let b : ComplexMatrix(3, 1) = cmat_zero in
  let x : ComplexMatrix(3, 1) = cmat_solve(A, b) in
  x
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Complex matrix solve cmat_solve should work"
    );
}

#[test]
fn test_complex_matrix_inverse() {
    let code = r#"
define testLapack6 =
  let A : ComplexMatrix(4, 4) = cmat_zero in
  let Ainv : ComplexMatrix(4, 4) = cmat_inv(A) in
  Ainv
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Complex matrix inverse cmat_inv should work"
    );
}

#[test]
fn test_complex_matrix_eigenvalues() {
    let code = r#"
define testLapack7 =
  let A : ComplexMatrix(3, 3) = cmat_zero in
  cmat_eigenvalues(A)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Complex matrix eigenvalues cmat_eigenvalues should work"
    );
}

// Note: Operations returning tuples (cmat_qr, cmat_svd, cmat_eig, cmat_schur)
// require additional SignatureInterpreter work. Testing simpler operations.

#[test]
fn test_complex_matrix_double_exponential() {
    let code = r#"
define testLapack8 =
  let A : ComplexMatrix(4, 4) = cmat_zero in
  let B : ComplexMatrix(4, 4) = cmat_expm(A) in
  let C : ComplexMatrix(4, 4) = cmat_expm(B) in
  cmat_det(C)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Double complex matrix exponential should work"
    );
}

#[test]
fn test_multiple_complex_matrices() {
    let code = r#"
define testLapack9 =
  let A : ComplexMatrix(3, 3) = cmat_zero in
  let B : ComplexMatrix(3, 3) = cmat_zero in
  let C : ComplexMatrix(3, 3) = cmat_add(A, B) in
  let D : ComplexMatrix(3, 3) = cmat_conj(C) in
  cmat_det(D)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Multiple complex matrix ops should work"
    );
}

#[test]
fn test_complex_matrix_singular_values() {
    let code = r#"
define testLapack10 =
  let A : ComplexMatrix(5, 5) = cmat_zero in
  cmat_singular_values(A)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Complex matrix singular values should work"
    );
}

// Note: cmat_schur returns a tuple, requires additional work
#[test]
fn test_large_complex_matrix() {
    let code = r#"
define testLapack11 =
  let A : ComplexMatrix(8, 8) = cmat_zero in
  let B : ComplexMatrix(8, 8) = cmat_expm(A) in
  cmat_norm(B)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Large complex matrix operations should work"
    );
}

#[test]
fn test_complex_matrix_determinant() {
    let code = r#"
define testLapack12 =
  let A : ComplexMatrix(2, 2) = cmat_zero in
  cmat_det(A)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Complex matrix determinant cmat_det should work"
    );
}

#[test]
fn test_complex_matrix_norm() {
    let code = r#"
define testLapack13 =
  let A : ComplexMatrix(3, 3) = cmat_zero in
  cmat_norm(A)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Complex matrix norm cmat_norm should work"
    );
}

#[test]
fn test_complex_matrix_condition_number() {
    let code = r#"
define testLapack14 =
  let A : ComplexMatrix(4, 4) = cmat_zero in
  cmat_cond(A)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Complex matrix condition number cmat_cond should work"
    );
}

#[test]
fn test_complex_matrix_rank() {
    let code = r#"
define testLapack15 =
  let A : ComplexMatrix(3, 3) = cmat_zero in
  cmat_rank(A)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Complex matrix rank cmat_rank should work"
    );
}

// ============= CHAINED COMPLEX OPERATIONS =============

#[test]
fn test_complex_matrix_chain() {
    let code = r#"
define testChain1 =
  let R : Matrix(2, 2, ℝ) = matrix(1,2,3,4) in
  let C : ComplexMatrix(2, 2) = cmat_from_real(R) in
  let D : ComplexMatrix(2, 2) = cmat_dagger(C) in
  let E : ComplexMatrix(2, 2) = cmat_add(C, D) in
  cmat_det(E)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Chained complex matrix operations should work"
    );
}

#[test]
fn test_hermitian_matrix_construction() {
    let code = r#"
define testHermitian =
  let R : Matrix(3, 3, ℝ) = matrix(1,0,0,0,2,0,0,0,3) in
  let C : ComplexMatrix(3, 3) = cmat_from_real(R) in
  let Cdagger : ComplexMatrix(3, 3) = cmat_dagger(C) in
  let Hermitian : ComplexMatrix(3, 3) = cmat_add(C, Cdagger) in
  cmat_eigenvalues(Hermitian)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Hermitian matrix construction should work"
    );
}

#[test]
fn test_mixed_real_complex_workflow() {
    let code = r#"
define testMixed =
  let R1 : Matrix(2, 2, ℝ) = matrix(1,0,0,1) in
  let R2 : Matrix(2, 2, ℝ) = matrix(0,1,1,0) in
  let C1 : ComplexMatrix(2, 2) = cmat_from_real(R1) in
  let C2 : ComplexMatrix(2, 2) = cmat_from_imag(R2) in
  let C3 : ComplexMatrix(2, 2) = cmat_add(C1, C2) in
  let ExpC3 : ComplexMatrix(2, 2) = cmat_expm(C3) in
  let Result : Matrix(2, 2, ℝ) = cmat_real(ExpC3) in
  det(Result)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Mixed real/complex workflow should work"
    );
}

// ============= DIMENSION EXPRESSION TESTS =============
// These test operations with dimension expressions like 2*n, m+n, etc.

#[test]
fn test_realify_dimension_doubling() {
    // realify : ComplexMatrix(n, n) → Matrix(2*n, 2*n, ℝ)
    // ComplexMatrix(2,2) → Matrix(4,4,ℝ)
    let code = r#"
define testDimExpr1 =
  let C : ComplexMatrix(2, 2) = cmat_zero in
  let R : Matrix(4, 4, ℝ) = realify(C) in
  det(R)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "realify with 2*n dimension should work"
    );
}

#[test]
fn test_realify_3x3() {
    // ComplexMatrix(3,3) → Matrix(6,6,ℝ)
    let code = r#"
define testDimExpr2 =
  let C : ComplexMatrix(3, 3) = cmat_zero in
  let R : Matrix(6, 6, ℝ) = realify(C) in
  trace(R)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "realify 3x3 → 6x6 should work"
    );
}

#[test]
fn test_realify_4x4() {
    // ComplexMatrix(4,4) → Matrix(8,8,ℝ)
    let code = r#"
define testDimExpr3 =
  let C : ComplexMatrix(4, 4) = cmat_zero in
  let R : Matrix(8, 8, ℝ) = realify(C) in
  det(R)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "realify 4x4 → 8x8 should work"
    );
}

#[test]
fn test_complexify_dimension_halving() {
    // complexify : Matrix(2*n, 2*n, ℝ) → ComplexMatrix(n, n)
    // Matrix(4,4,ℝ) → ComplexMatrix(2,2)
    let code = r#"
define testDimExpr4 =
  let R : Matrix(4, 4, ℝ) = matrix(1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1) in
  let C : ComplexMatrix(2, 2) = complexify(R) in
  cmat_det(C)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "complexify with n=2*n/2 dimension should work"
    );
}

#[test]
fn test_complexify_6x6_to_3x3() {
    // Matrix(6,6,ℝ) → ComplexMatrix(3,3)
    let code = r#"
define testDimExpr5 =
  let R : Matrix(6, 6, ℝ) = matrix(1,0,0,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,0,0,1) in
  let C : ComplexMatrix(3, 3) = complexify(R) in
  cmat_norm(C)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "complexify 6x6 → 3x3 should work"
    );
}

#[test]
fn test_realify_complexify_roundtrip() {
    // C → realify → complexify should preserve dimensions
    let code = r#"
define testDimExpr6 =
  let C1 : ComplexMatrix(2, 2) = cmat_zero in
  let R : Matrix(4, 4, ℝ) = realify(C1) in
  let C2 : ComplexMatrix(2, 2) = complexify(R) in
  cmat_det(C2)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "realify/complexify roundtrip should work"
    );
}

#[test]
fn test_realify_then_real_operations() {
    // After realify, should be able to use real matrix operations
    let code = r#"
define testDimExpr7 =
  let C : ComplexMatrix(2, 2) = cmat_zero in
  let R1 : Matrix(4, 4, ℝ) = realify(C) in
  let R2 : Matrix(4, 4, ℝ) = realify(C) in
  let R3 : Matrix(4, 4, ℝ) = multiply(R1, R2) in
  det(R3)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Real operations after realify should work"
    );
}

#[test]
fn test_realify_expm_complexify_chain() {
    // C → realify → expm → complexify
    let code = r#"
define testDimExpr8 =
  let C : ComplexMatrix(2, 2) = cmat_zero in
  let R : Matrix(4, 4, ℝ) = realify(C) in
  let ExpR : Matrix(4, 4, ℝ) = expm(R) in
  let Result : ComplexMatrix(2, 2) = complexify(ExpR) in
  cmat_det(Result)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "realify → expm → complexify chain should work"
    );
}

#[test]
fn test_multiple_realify_operations() {
    let code = r#"
define testDimExpr9 =
  let A : ComplexMatrix(2, 2) = cmat_zero in
  let B : ComplexMatrix(2, 2) = cmat_zero in
  let RA : Matrix(4, 4, ℝ) = realify(A) in
  let RB : Matrix(4, 4, ℝ) = realify(B) in
  let Sum : Matrix(4, 4, ℝ) = matrix_add(RA, RB) in
  let Product : Matrix(4, 4, ℝ) = multiply(RA, RB) in
  matrix_add(Sum, Product)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Multiple realify with real ops should work"
    );
}

#[test]
fn test_realify_with_different_sizes() {
    let code = r#"
define testDimExpr10 =
  let Small : ComplexMatrix(1, 1) = cmat_zero in
  let Medium : ComplexMatrix(2, 2) = cmat_zero in
  let Large : ComplexMatrix(3, 3) = cmat_zero in
  let RSmall : Matrix(2, 2, ℝ) = realify(Small) in
  let RMedium : Matrix(4, 4, ℝ) = realify(Medium) in
  let RLarge : Matrix(6, 6, ℝ) = realify(Large) in
  det(RSmall)
"#;
    assert!(
        type_check_code(code).is_ok(),
        "Realify with different sizes should work"
    );
}
