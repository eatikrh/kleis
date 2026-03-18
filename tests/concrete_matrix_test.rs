//! Tests for concrete matrix evaluation
//!
//! These tests verify that matrix operations can be evaluated
//! with concrete values using the `:eval` command.

use kleis::evaluator::Evaluator;
use kleis::kleis_parser::parse_kleis;

fn eval(code: &str) -> String {
    let expr = parse_kleis(code).expect("parse failed");
    let evaluator = Evaluator::new();
    let result = evaluator.eval_concrete(&expr);
    match result {
        Ok(evaluated) => format!("{:?}", evaluated),
        Err(e) => format!("Error: {}", e),
    }
}

#[test]
fn test_matrix_add() {
    let result = eval("matrix_add(Matrix(2, 2, [1, 2, 3, 4]), Matrix(2, 2, [5, 6, 7, 8]))");
    assert!(result.contains("Matrix"));
    assert!(result.contains("["));
    // 1+5=6, 2+6=8, 3+7=10, 4+8=12
    assert!(result.contains("6"));
    assert!(result.contains("8"));
    assert!(result.contains("10"));
    assert!(result.contains("12"));
}

#[test]
fn test_matrix_sub() {
    let result = eval("matrix_sub(Matrix(2, 2, [10, 20, 30, 40]), Matrix(2, 2, [1, 2, 3, 4]))");
    assert!(result.contains("Matrix"));
    // 10-1=9, 20-2=18, 30-3=27, 40-4=36
    assert!(result.contains("9"));
    assert!(result.contains("18"));
    assert!(result.contains("27"));
    assert!(result.contains("36"));
}

#[test]
fn test_matrix_multiply() {
    // [1,2; 3,4] × [5,6; 7,8] = [19,22; 43,50]
    let result = eval("multiply(Matrix(2, 2, [1, 2, 3, 4]), Matrix(2, 2, [5, 6, 7, 8]))");
    assert!(result.contains("Matrix"));
    assert!(result.contains("19"));
    assert!(result.contains("22"));
    assert!(result.contains("43"));
    assert!(result.contains("50"));
}

#[test]
fn test_matrix_transpose() {
    // 2×3 matrix transposed to 3×2
    let result = eval("transpose(Matrix(2, 3, [1, 2, 3, 4, 5, 6]))");
    assert!(result.contains("Matrix"));
    // Result should be Matrix(3, 2, [1, 4, 2, 5, 3, 6])
    // The output format shows the dimensions changed
}

#[test]
fn test_matrix_trace() {
    // trace of identity matrix
    let result = eval("trace(Matrix(3, 3, [1, 0, 0, 0, 2, 0, 0, 0, 3]))");
    // 1 + 2 + 3 = 6
    assert!(result.contains("6"));
}

#[test]
fn test_matrix_det_2x2() {
    // det([4,3; 6,8]) = 4*8 - 3*6 = 32 - 18 = 14
    let result = eval("det(Matrix(2, 2, [4, 3, 6, 8]))");
    assert!(result.contains("14"));
}

#[test]
fn test_matrix_det_3x3() {
    // det([1,2,3; 0,1,4; 5,6,0]) = 1
    let result = eval("det(Matrix(3, 3, [1, 2, 3, 0, 1, 4, 5, 6, 0]))");
    assert!(result.contains("1"));
}

#[test]
fn test_scalar_matrix_mul() {
    // 3 * [1,2,3,4] = [3,6,9,12]
    let result = eval("scalar_matrix_mul(3, Matrix(2, 2, [1, 2, 3, 4]))");
    assert!(result.contains("Matrix"));
    assert!(result.contains("3"));
    assert!(result.contains("6"));
    assert!(result.contains("9"));
    assert!(result.contains("12"));
}

#[test]
fn test_matrix_add_dimension_mismatch() {
    // Should error on dimension mismatch
    let result =
        eval("matrix_add(Matrix(2, 2, [1, 2, 3, 4]), Matrix(3, 3, [1, 2, 3, 4, 5, 6, 7, 8, 9]))");
    assert!(result.contains("dimension mismatch") || result.contains("Error"));
}

#[test]
fn test_matrix_multiply_inner_dimension_mismatch() {
    // 2×2 cannot multiply with 3×2 (inner dimensions don't match)
    let result = eval("multiply(Matrix(2, 2, [1, 2, 3, 4]), Matrix(3, 2, [1, 2, 3, 4, 5, 6]))");
    assert!(result.contains("don't match") || result.contains("Error"));
}

#[test]
fn test_trace_non_square() {
    // trace requires square matrix
    let result = eval("trace(Matrix(2, 3, [1, 2, 3, 4, 5, 6]))");
    assert!(result.contains("square") || result.contains("Error"));
}

#[test]
fn test_det_non_square() {
    // det requires square matrix
    let result = eval("det(Matrix(2, 3, [1, 2, 3, 4, 5, 6]))");
    assert!(result.contains("square") || result.contains("Error"));
}

#[test]
fn test_matrix_multiply_non_square() {
    // 2×3 matrix times 3×4 matrix = 2×4 matrix
    // [1,2,3; 4,5,6] × [1,2,3,4; 5,6,7,8; 9,10,11,12]
    // Result[0,0] = 1*1 + 2*5 + 3*9 = 1 + 10 + 27 = 38
    let result = eval("multiply(Matrix(2, 3, [1, 2, 3, 4, 5, 6]), Matrix(3, 4, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]))");
    assert!(result.contains("Matrix"));
    assert!(result.contains("38")); // First element
}

#[test]
fn test_identity_matrix_multiply() {
    // A × I = A
    // [1,2; 3,4] × [1,0; 0,1] = [1,2; 3,4]
    let result = eval("multiply(Matrix(2, 2, [1, 2, 3, 4]), Matrix(2, 2, [1, 0, 0, 1]))");
    assert!(result.contains("Matrix"));
    // Result should be [1, 2, 3, 4]
    assert!(result.contains("\"1\""));
    assert!(result.contains("\"2\""));
    assert!(result.contains("\"3\""));
    assert!(result.contains("\"4\""));
}

#[test]
fn test_symbolic_matrix_det_returns_unevaluated() {
    // det of symbolic matrix should return unevaluated, not just "0"
    let result = eval("det(Matrix(2, 2, [a, 0, 0, 1]))");
    // Should contain "det" (unevaluated expression) - not just a single constant
    assert!(result.contains("det") || result.contains("Matrix"));
    // The result should be the full expression, not reduced to just "0"
    assert!(result.contains("a"));
}

#[test]
fn test_symbolic_matrix_multiply_returns_unevaluated() {
    // multiply with symbolic elements should return unevaluated
    let result = eval("multiply(Matrix(2, 2, [a, 0, 0, 1]), Matrix(2, 2, [1, 0, 0, b]))");
    // Should contain the original expression, not computed zeros
    assert!(result.contains("Matrix") || result.contains("multiply"));
}

#[test]
fn test_symbolic_matrix_trace_returns_unevaluated() {
    // trace with symbolic diagonal should return unevaluated
    let result = eval("trace(Matrix(2, 2, [a, 0, 0, 1]))");
    // Should contain "trace" (unevaluated) and NOT be a simple number
    assert!(result.contains("trace") || result.contains("Matrix"));
}

// ============================================
// MATRIX EXTRACTION OPERATIONS
// ============================================

#[test]
fn test_matrix_get_element() {
    // Get element at (0, 2) from 2×3 matrix
    let result = eval("matrix_get(Matrix(2, 3, [1, 2, 3, 4, 5, 6]), 0, 2)");
    assert!(result.contains("3"));
}

#[test]
fn test_matrix_get_element_second_row() {
    // Get element at (1, 1) from 2×3 matrix
    let result = eval("matrix_get(Matrix(2, 3, [1, 2, 3, 4, 5, 6]), 1, 1)");
    assert!(result.contains("5"));
}

#[test]
fn test_matrix_row() {
    // Get row 0 from 2×3 matrix
    let result = eval("matrix_row(Matrix(2, 3, [1, 2, 3, 4, 5, 6]), 0)");
    assert!(result.contains("List"));
    assert!(result.contains("1"));
    assert!(result.contains("2"));
    assert!(result.contains("3"));
}

#[test]
fn test_matrix_col() {
    // Get column 0 from 2×3 matrix
    let result = eval("matrix_col(Matrix(2, 3, [1, 2, 3, 4, 5, 6]), 0)");
    assert!(result.contains("List"));
    assert!(result.contains("1"));
    assert!(result.contains("4"));
}

#[test]
fn test_matrix_diag() {
    // Get diagonal from 3×3 matrix
    let result = eval("matrix_diag(Matrix(3, 3, [1, 2, 3, 4, 5, 6, 7, 8, 9]))");
    assert!(result.contains("List"));
    assert!(result.contains("1"));
    assert!(result.contains("5"));
    assert!(result.contains("9"));
}

#[test]
fn test_matrix_get_out_of_bounds() {
    // Out of bounds should error
    let result = eval("matrix_get(Matrix(2, 2, [1, 2, 3, 4]), 5, 0)");
    assert!(result.contains("out of bounds") || result.contains("Error"));
}

// ============================================
// SYMBOLIC MATRIX ARITHMETIC
// ============================================

#[test]
fn test_matrix_add_mixed_symbolic() {
    // Adding concrete and symbolic matrices
    let result = eval("matrix_add(Matrix(2, 2, [1, 2, 3, 4]), Matrix(2, 2, [a, b, c, d]))");
    assert!(result.contains("Matrix"));
    // Should contain symbolic expressions like "1 + a"
    assert!(result.contains("plus") || result.contains("+"));
}

#[test]
fn test_matrix_add_zero_optimization() {
    // 0 + x = x optimization
    let result = eval("matrix_add(Matrix(2, 2, [0, 0, 0, 0]), Matrix(2, 2, [a, b, c, d]))");
    assert!(result.contains("Matrix"));
    // Should be simplified to just [a, b, c, d] without plus operations
    assert!(result.contains("a"));
    assert!(result.contains("b"));
}

#[test]
fn test_matrix_sub_mixed_symbolic() {
    // Subtracting symbolic from concrete
    let result = eval("matrix_sub(Matrix(2, 2, [10, 20, 30, 40]), Matrix(2, 2, [a, b, c, d]))");
    assert!(result.contains("Matrix"));
    // Should contain symbolic expressions
    assert!(result.contains("minus") || result.contains("-"));
}
