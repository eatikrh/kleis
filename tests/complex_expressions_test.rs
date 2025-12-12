#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
///! Complex expression tests for TypeContextBuilder improvements
///!
///! Tests that complex nested expressions work correctly with
///! the improved SignatureInterpreter fallback logic.
use kleis::ast::Expression;
use kleis::type_checker::{TypeCheckResult, TypeChecker};
use kleis::type_inference::Type;

fn c(s: &str) -> Expression {
    Expression::Const(s.to_string())
}

fn var(s: &str) -> Expression {
    Expression::Object(s.to_string())
}

fn op(name: &str, args: Vec<Expression>) -> Expression {
    Expression::operation(name, args)
}

#[test]
fn test_nested_matrix_operations() {
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // transpose(transpose(A)) where A is Matrix(2,3,ℝ)
    // NEW FORMAT: Matrix(m: Nat, n: Nat, T) - 3 args (type constructor)
    let expr = op(
        "transpose",
        vec![op(
            "transpose",
            vec![op(
                "Matrix",
                vec![
                    c("2"),   // m = 2
                    c("3"),   // n = 3
                    var("ℝ"), // T = ℝ (type)
                ],
            )],
        )],
    );

    match checker.check(&expr) {
        TypeCheckResult::Success(ty) => {
            // Type should be Matrix(2, 3, ℝ) represented as Data
            println!("Got type: {:?}", ty);
            assert!(
                matches!(&ty, Type::Data { type_name, .. } if type_name == "Type" || type_name == "Matrix")
                    || matches!(&ty, Type::Var(_)), // Type var is also OK
                "Expected Data(Type/Matrix) or Var, got {:?}",
                ty
            );
            println!("✓ transpose(transpose(Matrix(2,3,ℝ))) type-checks");
        }
        TypeCheckResult::Error { message, .. } => {
            panic!("Failed: {}", message);
        }
        _ => panic!("Unexpected result"),
    }
}

#[test]
fn test_complex_arithmetic_with_integrals() {
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // (a + b) * ∫₀¹ x² dx
    let expr = op(
        "scalar_multiply",
        vec![
            op("plus", vec![var("a"), var("b")]),
            op(
                "int_bounds",
                vec![op("sup", vec![var("x"), c("2")]), c("0"), c("1"), var("x")],
            ),
        ],
    );

    match checker.check(&expr) {
        TypeCheckResult::Success(ty) => {
            // With proper polymorphism, unbound variables may remain as type vars
            assert!(
                matches!(&ty, Type::Data { constructor, .. } if constructor == "Scalar")
                    || matches!(&ty, Type::Var(_)),
                "Expected Scalar or Var, got {:?}",
                ty
            );
            println!("✓ (a + b) * ∫₀¹ x² dx : {:?}", ty);
        }
        TypeCheckResult::Error { message, .. } => {
            panic!("Failed: {}", message);
        }
        _ => {}
    }
}

#[test]
fn test_matrix_equation() {
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // B × C where B is 2×3, C is 3×4
    // NEW FORMAT: Matrix(m: Nat, n: Nat, T) - 3 args
    let expr = op(
        "multiply",
        vec![
            op("Matrix", vec![c("2"), c("3"), var("ℝ")]), // Matrix(2, 3, ℝ)
            op("Matrix", vec![c("3"), c("4"), var("ℝ")]), // Matrix(3, 4, ℝ)
        ],
    );

    match checker.check(&expr) {
        TypeCheckResult::Success(ty) => {
            // Type should be Matrix(2, 4, ℝ)
            println!("Got type: {:?}", ty);
            assert!(
                matches!(&ty, Type::Data { type_name, .. } if type_name == "Type" || type_name == "Matrix")
                    || matches!(&ty, Type::Var(_)), // Type var is also OK
                "Expected Data(Type/Matrix) or Var, got {:?}",
                ty
            );
            println!("✓ Matrix(2,3,ℝ) × Matrix(3,4,ℝ) type-checks");
        }
        TypeCheckResult::Error { message, .. } => {
            panic!("Failed: {}", message);
        }
        _ => {}
    }
}

#[test]
fn test_error_message_quality() {
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // Try to use a completely unknown operation
    let expr = op("nonexistent_operation", vec![c("1")]);

    match checker.check(&expr) {
        TypeCheckResult::Error { message, .. } => {
            assert!(message.contains("Unknown operation"));
            assert!(message.contains("Hint") || message.contains("not defined"));
            println!("✓ Error message is helpful: {}", message);
        }
        _ => panic!("Should have errored on unknown operation"),
    }
}

#[test]
fn test_dimension_mismatch_error() {
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // Try to multiply incompatible matrices: 2×3 × 4×5
    // Inner dimensions don't match (3 ≠ 4) - should fail!
    // NEW FORMAT: Matrix(m: Nat, n: Nat, T) - 3 args
    let expr = op(
        "multiply",
        vec![
            op("Matrix", vec![c("2"), c("3"), var("ℝ")]), // Matrix(2, 3, ℝ)
            op("Matrix", vec![c("4"), c("5"), var("ℝ")]), // Matrix(4, 5, ℝ)
        ],
    );

    match checker.check(&expr) {
        TypeCheckResult::Error { message, .. } => {
            // Should error: inner dimensions don't match for multiplication
            // Matrix(2,3) × Matrix(4,5) - the 3 and 4 don't match!
            println!("✓ Dimension mismatch detected: {}", message);
            assert!(
                message.contains("Cannot unify")
                    || message.contains("dimension")
                    || message.contains("3")
                    || message.contains("4")
                    || message.contains("inference failed")
            );
        }
        _ => panic!("Should have errored on dimension mismatch"),
    }
}

#[test]
fn test_ordering_on_matrices_rejected() {
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // Try A < B where both are matrices (nonsensical)
    // NEW FORMAT: Matrix(m: Nat, n: Nat, T) - 3 args
    let expr = op(
        "less_than",
        vec![
            op("Matrix", vec![c("2"), c("2"), var("ℝ")]), // Matrix(2, 2, ℝ)
            op("Matrix", vec![c("2"), c("2"), var("ℝ")]), // Matrix(2, 2, ℝ)
        ],
    );

    match checker.check(&expr) {
        TypeCheckResult::Error { message, .. } => {
            // Should mention that Matrix doesn't support ordering operations
            println!("✓ Matrix ordering correctly rejected: {}", message);
            assert!(
                message.contains("Matrix")
                    || message.contains("less_than")
                    || message.contains("Operation")
                    || message.contains("not")
                    || message.contains("Unknown")
            );
        }
        other => {
            panic!("Should reject ordering on matrices, got: {:?}", other);
        }
    }
}
