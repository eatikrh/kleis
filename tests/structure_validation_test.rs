// Tests for structure implementation validation (ADR-016)
//
// These tests verify that the generic validation system works correctly:
// - Operations only work on types that implement the required structure
// - Helpful error messages when validation fails
// - Polymorphic types (type variables) skip validation
// - All ordering operations are validated consistently

use kleis::ast::Expression;
use kleis::type_checker::{TypeCheckResult, TypeChecker};
use kleis::type_inference::Type;

// Helper to create constants
fn c(s: &str) -> Expression {
    Expression::Const(s.to_string())
}

// Helper to create operations
fn op(name: &str, args: Vec<Expression>) -> Expression {
    Expression::Operation {
        name: name.to_string(),
        args,
    }
}

// Helper to create variables (objects in AST)
fn var(name: &str) -> Expression {
    Expression::Object(name.to_string())
}

#[test]
fn test_ordering_works_for_scalars() {
    // CRITICAL: Test that ordering SUCCEEDS for types that implement Ordered
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // Test all ordering operations on scalars (ℝ implements Ordered)
    let operations = vec!["less_than", "greater_than", "less_equal", "greater_equal"];

    for op_name in operations {
        let expr = op(op_name, vec![c("1"), c("2")]);

        match checker.check(&expr) {
            TypeCheckResult::Success(ty) => {
                // Should infer Bool (now properly represented as Type::Data)
                let is_bool = match &ty {
                    Type::Data { type_name, .. } => type_name == "Bool",
                    _ => false,
                };
                assert!(
                    ty == Type::scalar() || is_bool,
                    "Expected Scalar or Bool, got {:?}",
                    ty
                );
                println!("✓ {} on scalars works: {:?}", op_name, ty);
            }
            TypeCheckResult::Error { message, .. } => {
                panic!("{} should work on scalars but failed: {}", op_name, message);
            }
            other => {
                panic!("{} unexpected result: {:?}", op_name, other);
            }
        }
    }
}

#[test]
fn test_ordering_rejected_for_matrices() {
    // Test that ordering FAILS for types that DON'T implement Ordered
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    let operations = vec!["less_than", "greater_than", "less_equal", "greater_equal"];

    for op_name in operations {
        let expr = op(
            op_name,
            vec![
                op(
                    "Matrix",
                    vec![c("2"), c("2"), c("1"), c("2"), c("3"), c("4")],
                ),
                op(
                    "Matrix",
                    vec![c("2"), c("2"), c("5"), c("6"), c("7"), c("8")],
                ),
            ],
        );

        match checker.check(&expr) {
            TypeCheckResult::Error { message, .. } => {
                // Should mention that Matrix doesn't support ordering
                assert!(
                    message.contains("Matrix")
                        && (message.contains("does not support")
                            || message.contains("not defined")),
                    "Error message should mention Matrix doesn't support {}: {}",
                    op_name,
                    message
                );
                println!("✓ {} correctly rejected for matrices", op_name);
            }
            other => {
                panic!("{} should fail on matrices but got: {:?}", op_name, other);
            }
        }
    }
}

#[test]
fn test_ordering_with_type_variables() {
    // Polymorphic types (type variables) should skip validation
    // This is correct because validation happens at instantiation
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // x < y where x, y are unknown variables
    let expr = op("less_than", vec![var("x"), var("y")]);

    match checker.check(&expr) {
        TypeCheckResult::Success(_) | TypeCheckResult::Polymorphic { .. } => {
            println!("✓ Ordering with type variables works (polymorphic)");
        }
        TypeCheckResult::Error { message, .. } => {
            // This is also acceptable if the error is about unknown variables
            // not about structure validation
            if message.contains("Unknown variable") {
                println!("✓ Unknown variable error (acceptable)");
            } else {
                panic!("Should not fail validation for type variables: {}", message);
            }
        }
    }
}

#[test]
fn test_mixed_types_ordering() {
    // Ordering between different types should fail
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // Matrix < Scalar (nonsensical)
    let expr = op(
        "less_than",
        vec![
            op(
                "Matrix",
                vec![c("2"), c("2"), c("1"), c("2"), c("3"), c("4")],
            ),
            c("5"), // scalar
        ],
    );

    match checker.check(&expr) {
        TypeCheckResult::Error { message, .. } => {
            // Should fail - either because Matrix doesn't support ordering
            // or because types don't match
            println!("✓ Mixed type ordering rejected: {}", message);
        }
        other => {
            panic!("Mixed type ordering should fail but got: {:?}", other);
        }
    }
}

#[test]
fn test_error_message_quality() {
    // Verify error messages are helpful
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    let expr = op(
        "less_than",
        vec![
            op(
                "Matrix",
                vec![c("2"), c("2"), c("1"), c("2"), c("3"), c("4")],
            ),
            op(
                "Matrix",
                vec![c("2"), c("2"), c("5"), c("6"), c("7"), c("8")],
            ),
        ],
    );

    match checker.check(&expr) {
        TypeCheckResult::Error { message, .. } => {
            // Error message should be informative
            assert!(message.contains("Matrix"), "Should mention Matrix type");
            assert!(
                message.contains("less_than") || message.contains("Ordered"),
                "Should mention operation or structure"
            );
            assert!(
                message.contains("does not") || message.contains("not defined"),
                "Should clearly state the problem"
            );

            println!("✓ Error message is informative:\n{}", message);
        }
        other => {
            panic!("Should produce error but got: {:?}", other);
        }
    }
}

#[test]
fn test_arithmetic_still_works() {
    // Sanity check: Make sure we didn't break non-ordering operations
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    let operations = vec![
        ("plus", vec![c("1"), c("2")]),
        ("minus", vec![c("5"), c("3")]),
        ("abs", vec![c("-5")]),
        ("sqrt", vec![c("16")]),
    ];

    for (op_name, args) in operations {
        let expr = op(op_name, args);

        match checker.check(&expr) {
            TypeCheckResult::Success(_) => {
                println!("✓ {} still works", op_name);
            }
            TypeCheckResult::Error { message, .. } => {
                panic!("{} should work but failed: {}", op_name, message);
            }
            _ => {}
        }
    }
}

#[test]
fn test_matrix_operations_still_work() {
    // Sanity check: Matrix operations should still work
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // Matrix transpose should work
    let expr = op(
        "transpose",
        vec![op(
            "Matrix",
            vec![
                c("2"),
                c("3"),
                c("1"),
                c("2"),
                c("3"),
                c("4"),
                c("5"),
                c("6"),
            ],
        )],
    );

    match checker.check(&expr) {
        TypeCheckResult::Success(ty) if ty == Type::matrix(3, 2, Type::scalar()) => {
            println!("✓ Matrix transpose works and infers correct dimensions");
        }
        TypeCheckResult::Success(ty) => {
            println!("✓ Matrix transpose works (got type: {:?})", ty);
        }
        TypeCheckResult::Error { message, .. } => {
            panic!("Matrix transpose should work but failed: {}", message);
        }
        _ => {}
    }
}
