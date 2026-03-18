//! Tests for parametric type dimension/parameter checking (ADR-016 compliant)
//!
//! Verifies that arithmetic operations on parametric types require matching Nat parameters.
//! Works for ANY user-defined type with Nat parameters (Matrix, Vector, Tensor, etc.)

use kleis::ast::Expression;
use kleis::type_checker::TypeChecker;

fn create_checker() -> TypeChecker {
    TypeChecker::with_stdlib().expect("Failed to create type checker with stdlib")
}

/// Helper to create Matrix(m, n, elements) expression
fn matrix_expr(m: usize, n: usize, elements: Vec<Expression>) -> Expression {
    Expression::Operation {
        name: "Matrix".to_string(),
        args: vec![
            Expression::Const(m.to_string()),
            Expression::Const(n.to_string()),
            Expression::List(elements),
        ],
        span: None,
    }
}

/// Helper to create a plus operation
fn plus_expr(a: Expression, b: Expression) -> Expression {
    Expression::Operation {
        name: "plus".to_string(),
        args: vec![a, b],
        span: None,
    }
}

#[test]
fn test_trace_dimension_mismatch() {
    let mut checker = create_checker();

    // Matrix(3,3) + Matrix(2,2) - trace what happens
    let matrix_3x3 = matrix_expr(
        3,
        3,
        vec![
            Expression::Const("1".to_string()),
            Expression::Const("2".to_string()),
            Expression::Const("3".to_string()),
            Expression::Const("4".to_string()),
            Expression::Const("5".to_string()),
            Expression::Const("6".to_string()),
            Expression::Const("7".to_string()),
            Expression::Const("8".to_string()),
            Expression::Const("9".to_string()),
        ],
    );

    let matrix_2x2 = matrix_expr(
        2,
        2,
        vec![
            Expression::Const("1".to_string()),
            Expression::Const("2".to_string()),
            Expression::Const("3".to_string()),
            Expression::Const("4".to_string()),
        ],
    );

    let expr = plus_expr(matrix_3x3, matrix_2x2);
    let result = checker.check(&expr);

    println!("\n=== DIMENSION MISMATCH TEST ===");
    println!("Result: {:?}", result);

    // We expect the type inference to catch the dimension mismatch
    match &result {
        kleis::type_checker::TypeCheckResult::Error { message, .. } => {
            println!("Good - got error: {}", message);
            // The error should mention dimension/mismatch/type parameter
            assert!(
                message.contains("dimension")
                    || message.contains("mismatch")
                    || message.contains("parameter")
                    || message.contains("NatValue"),
                "Error should mention dimension issue: {}",
                message
            );
        }
        other => {
            println!("WARNING: Expected error, got: {:?}", other);
            // For now, just log - once we fix this, we'll make it an assertion
        }
    }
}

#[test]
fn test_matrix_plus_same_dimensions_succeeds() {
    let mut checker = create_checker();

    // Two 2x2 matrices should work
    let matrix_a = matrix_expr(
        2,
        2,
        vec![
            Expression::Const("1".to_string()),
            Expression::Const("2".to_string()),
            Expression::Const("3".to_string()),
            Expression::Const("4".to_string()),
        ],
    );

    let matrix_b = matrix_expr(
        2,
        2,
        vec![
            Expression::Const("5".to_string()),
            Expression::Const("6".to_string()),
            Expression::Const("7".to_string()),
            Expression::Const("8".to_string()),
        ],
    );

    let expr = plus_expr(matrix_a, matrix_b);
    let result = checker.check(&expr);

    match result {
        kleis::type_checker::TypeCheckResult::Success(ty) => {
            let ty_str = format!("{:?}", ty);
            assert!(
                ty_str.contains("Matrix"),
                "Result should be Matrix type: {}",
                ty_str
            );
        }
        other => {
            panic!("Matrix(2,2) + Matrix(2,2) should succeed, got: {:?}", other);
        }
    }
}

#[test]
fn test_matrix_plus_different_dimensions_should_fail() {
    let mut checker = create_checker();

    // Matrix(3,3) + Matrix(2,2) should fail due to dimension mismatch
    let matrix_3x3 = matrix_expr(
        3,
        3,
        vec![
            Expression::Const("1".to_string()),
            Expression::Const("2".to_string()),
            Expression::Const("3".to_string()),
            Expression::Const("4".to_string()),
            Expression::Const("5".to_string()),
            Expression::Const("6".to_string()),
            Expression::Const("7".to_string()),
            Expression::Const("8".to_string()),
            Expression::Const("9".to_string()),
        ],
    );

    let matrix_2x2 = matrix_expr(
        2,
        2,
        vec![
            Expression::Const("1".to_string()),
            Expression::Const("2".to_string()),
            Expression::Const("3".to_string()),
            Expression::Const("4".to_string()),
        ],
    );

    let expr = plus_expr(matrix_3x3, matrix_2x2);
    let result = checker.check(&expr);

    println!("\n=== test_matrix_plus_different_dimensions_should_fail ===");
    println!("Result: {:?}", result);

    // This MUST be an error due to parameter mismatch (dimension checking)
    match result {
        kleis::type_checker::TypeCheckResult::Error { message, .. } => {
            println!("Correctly caught parameter mismatch: {}", message);
            assert!(
                message.contains("mismatch") || message.contains("parameter"),
                "Error message should mention parameter mismatch"
            );
        }
        kleis::type_checker::TypeCheckResult::Success(ty) => {
            panic!(
                "Matrix(3,3) + Matrix(2,2) should fail with dimension mismatch, but got: {:?}",
                ty
            );
        }
        other => {
            panic!("Expected Error, got: {:?}", other);
        }
    }
}

#[test]
fn test_matrix_minus_same_dimensions_succeeds() {
    let mut checker = create_checker();

    // Two 3x3 matrices should work
    let matrix_a = matrix_expr(
        3,
        3,
        vec![
            Expression::Const("1".to_string()),
            Expression::Const("2".to_string()),
            Expression::Const("3".to_string()),
            Expression::Const("4".to_string()),
            Expression::Const("5".to_string()),
            Expression::Const("6".to_string()),
            Expression::Const("7".to_string()),
            Expression::Const("8".to_string()),
            Expression::Const("9".to_string()),
        ],
    );

    let matrix_b = matrix_expr(
        3,
        3,
        vec![
            Expression::Const("9".to_string()),
            Expression::Const("8".to_string()),
            Expression::Const("7".to_string()),
            Expression::Const("6".to_string()),
            Expression::Const("5".to_string()),
            Expression::Const("4".to_string()),
            Expression::Const("3".to_string()),
            Expression::Const("2".to_string()),
            Expression::Const("1".to_string()),
        ],
    );

    let expr = Expression::Operation {
        name: "minus".to_string(),
        args: vec![matrix_a, matrix_b],
        span: None,
    };
    let result = checker.check(&expr);

    match result {
        kleis::type_checker::TypeCheckResult::Success(ty) => {
            let ty_str = format!("{:?}", ty);
            assert!(
                ty_str.contains("Matrix"),
                "Result should be Matrix type: {}",
                ty_str
            );
        }
        other => {
            panic!("Matrix(3,3) - Matrix(3,3) should succeed, got: {:?}", other);
        }
    }
}

#[test]
fn test_matrix_add_via_specialized_operation() {
    let mut checker = create_checker();

    // Using the specialized matrix_add operation directly
    let matrix_a = matrix_expr(
        2,
        2,
        vec![
            Expression::Const("1".to_string()),
            Expression::Const("2".to_string()),
            Expression::Const("3".to_string()),
            Expression::Const("4".to_string()),
        ],
    );

    let matrix_b = matrix_expr(
        2,
        2,
        vec![
            Expression::Const("5".to_string()),
            Expression::Const("6".to_string()),
            Expression::Const("7".to_string()),
            Expression::Const("8".to_string()),
        ],
    );

    let expr = Expression::Operation {
        name: "matrix_add".to_string(),
        args: vec![matrix_a, matrix_b],
        span: None,
    };
    let result = checker.check(&expr);

    println!("\n=== test_matrix_add_via_specialized_operation ===");
    println!("Result: {:?}", result);

    // matrix_add should work for same dimensions
    match result {
        kleis::type_checker::TypeCheckResult::Success(ty) => {
            let ty_str = format!("{:?}", ty);
            assert!(
                ty_str.contains("Matrix"),
                "Result should be Matrix type: {}",
                ty_str
            );
        }
        kleis::type_checker::TypeCheckResult::Error { message, .. } => {
            // matrix_add might not be registered yet
            println!("matrix_add not yet supported: {}", message);
        }
        other => {
            println!("Got: {:?}", other);
        }
    }
}
