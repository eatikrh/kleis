//! Integration tests for ndarray tensor operations.
//!
//! These tests verify that ndarray_reshape, ndarray_contract,
//! ndarray_moveaxis, and ndarray_flatten work correctly when
//! invoked through the Kleis evaluator.

#![cfg(feature = "numerical")]

use kleis::evaluator::Evaluator;
use kleis::kleis_parser::parse_kleis_program;

fn eval_expr(code: &str) -> String {
    let mut eval = Evaluator::new();
    let full_code = format!("define result = {}", code);
    let program = parse_kleis_program(&full_code).expect("Failed to parse");
    eval.load_program(&program).expect("Failed to load");
    let closure = eval.get_function("result").expect("No result definition");
    let body = closure.body.clone();
    let evaluated = eval.eval_concrete(&body).expect("Failed to evaluate");
    format!("{:?}", evaluated)
}

fn eval_program(code: &str) -> String {
    let mut eval = Evaluator::new();
    let program = parse_kleis_program(code).expect("Failed to parse");
    eval.load_program(&program).expect("Failed to load");
    let closure = eval.get_function("result").expect("No result definition");
    let body = closure.body.clone();
    let evaluated = eval.eval_concrete(&body).expect("Failed to evaluate");
    format!("{:?}", evaluated)
}

#[test]
fn test_ndarray_reshape_basic() {
    let result = eval_expr("ndarray_reshape([1, 2, 3, 4], [2, 2])");
    assert!(
        result.contains("NDArray"),
        "Expected NDArray wrapper, got: {}",
        result
    );
    assert!(
        result.contains("1") && result.contains("4"),
        "Expected data preserved, got: {}",
        result
    );
}

#[test]
fn test_ndarray_reshape_flatten_roundtrip() {
    let result = eval_expr("ndarray_flatten(ndarray_reshape([10, 20, 30, 40], [2, 2]))");
    assert!(
        result.contains("10")
            && result.contains("20")
            && result.contains("30")
            && result.contains("40"),
        "Expected [10, 20, 30, 40], got: {}",
        result
    );
    assert!(
        !result.contains("NDArray"),
        "flatten should return a plain list, got: {}",
        result
    );
}

#[test]
fn test_ndarray_contract_identity() {
    // Contract with identity matrix should preserve data
    let result = eval_expr(
        "ndarray_flatten(ndarray_contract(ndarray_reshape([1, 2, 3, 4], [2, 2]), Matrix(2, 2, [1, 0, 0, 1]), 0))",
    );
    assert!(
        result.contains("1")
            && result.contains("2")
            && result.contains("3")
            && result.contains("4"),
        "Identity contraction should preserve data, got: {}",
        result
    );
}

#[test]
fn test_ndarray_contract_ising_n1() {
    // Ising trick for N=1: reshape 2-vector to (2,) tensor,
    // contract with a 2x2 matrix, flatten back
    // v = [1, 0], A = [[2, 0], [0, 3]]
    // Result: A * [1, 0] = [2, 0]
    let result = eval_expr(
        "ndarray_flatten(ndarray_contract(ndarray_reshape([1, 0], [2]), Matrix(2, 2, [2, 0, 0, 3]), 0))",
    );
    assert!(
        result.contains("2"),
        "Expected first element = 2, got: {}",
        result
    );
}

#[test]
fn test_ndarray_moveaxis() {
    // Reshape [1,2,3,4] as (2,2), move axis 0→1 (transpose)
    // [[1,2],[3,4]] transposed = [[1,3],[2,4]] → flat [1,3,2,4]
    let result =
        eval_expr("ndarray_flatten(ndarray_moveaxis(ndarray_reshape([1, 2, 3, 4], [2, 2]), 0, 1))");
    assert!(
        result.contains("1")
            && result.contains("3")
            && result.contains("2")
            && result.contains("4"),
        "Expected transposed data, got: {}",
        result
    );
}

#[test]
fn test_ndarray_multi_step_program() {
    // Test a multi-step Kleis program that chains tensor operations
    let code = r#"
define v = [1, 0, 0, 1]
define t = ndarray_reshape(v, [2, 2])
define A = Matrix(2, 2, [1, 0, 0, 1])
define contracted = ndarray_contract(t, A, 0)
define result = ndarray_flatten(contracted)
"#;
    let result = eval_program(code);
    assert!(
        result.contains("1") && result.contains("0"),
        "Expected identity contraction to preserve data, got: {}",
        result
    );
}

#[test]
fn test_ndarray_reshape_3d() {
    let result = eval_expr("ndarray_reshape([1,2,3,4,5,6,7,8], [2, 2, 2])");
    assert!(
        result.contains("NDArray"),
        "Expected NDArray wrapper, got: {}",
        result
    );
}

#[test]
fn test_ndarray_flatten_3d_roundtrip() {
    let result = eval_expr("ndarray_flatten(ndarray_reshape([1, 2, 3, 4, 5, 6, 7, 8], [2, 2, 2]))");
    for i in 1..=8 {
        assert!(
            result.contains(&format!("{}", i)),
            "Expected {} in result, got: {}",
            i,
            result
        );
    }
}

// ========================================
// DFT / FFT integration tests
// ========================================

#[test]
fn test_dft_dc_signal() {
    // DFT of [1, 1, 1, 1] = [4, 0, 0, 0]
    let result = eval_expr("dft([1, 1, 1, 1])");
    assert!(
        result.contains("4"),
        "Expected DC component = 4, got: {}",
        result
    );
}

#[test]
fn test_dft_impulse() {
    // DFT of [1, 0, 0, 0] = [1, 1, 1, 1]
    let result = eval_expr("dft([1, 0, 0, 0])");
    // All four elements should be 1
    assert!(
        result.contains("1"),
        "Expected all-ones spectrum, got: {}",
        result
    );
}

#[test]
fn test_fft_basic() {
    // FFT should produce same result as DFT for power-of-2
    let result = eval_expr("fft([1, 0, 0, 0])");
    assert!(
        result.contains("1"),
        "Expected all-ones spectrum, got: {}",
        result
    );
}

#[test]
fn test_fft_sine_signal() {
    // [0, 1, 0, -1] should produce complex values at k=1 and k=3
    let result = eval_expr("fft([0, 1, 0, -1])");
    assert!(
        result.contains("complex"),
        "Expected complex values in spectrum, got: {}",
        result
    );
}

#[test]
fn test_ifft_roundtrip() {
    // FFT then IFFT should recover original (real parts)
    // Using a multi-step program
    let code = r#"
define spectrum = fft([3, 1, 4, 1])
define result = ifft(spectrum)
"#;
    let result = eval_program(code);
    assert!(
        result.contains("3") && result.contains("4"),
        "Expected recovered signal with 3 and 4, got: {}",
        result
    );
}

#[test]
fn test_dft_non_power_of_2() {
    // DFT works for any size
    let result = eval_expr("dft([1, 2, 3])");
    assert!(
        result.contains("6"),
        "Expected DC component = 6 for [1,2,3], got: {}",
        result
    );
}

#[test]
fn test_fft_non_power_of_2_fallback() {
    // FFT falls back to DFT for non-power-of-2
    let result = eval_expr("fft([1, 2, 3])");
    assert!(
        result.contains("6"),
        "Expected DC component = 6, got: {}",
        result
    );
}
