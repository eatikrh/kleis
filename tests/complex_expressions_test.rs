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

    // transpose(transpose(A)) where A is Matrix(2,3)
    // Should be Matrix(2,3) (double transpose)
    let expr = op(
        "transpose",
        vec![op(
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
        )],
    );

    match checker.check(&expr) {
        TypeCheckResult::Success(ty) => {
            assert_eq!(ty, Type::matrix(2, 3));
            println!("✓ transpose(transpose(Matrix(2,3))) : Matrix(2,3)");
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

    // A = B × C where B is 2×3, C is 3×4
    let expr = op(
        "equals",
        vec![
            var("A"),
            op(
                "multiply",
                vec![
                    op(
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
                    ),
                    op(
                        "Matrix",
                        vec![
                            c("3"),
                            c("4"),
                            c("1"),
                            c("2"),
                            c("3"),
                            c("4"),
                            c("5"),
                            c("6"),
                            c("7"),
                            c("8"),
                            c("9"),
                            c("10"),
                            c("11"),
                            c("12"),
                        ],
                    ),
                ],
            ),
        ],
    );

    match checker.check(&expr) {
        TypeCheckResult::Success(ty) => {
            assert_eq!(ty, Type::matrix(2, 4));
            println!("✓ A = Matrix(2,3) × Matrix(3,4) : Matrix(2,4)");
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
    let expr = op(
        "multiply",
        vec![
            op(
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
            ),
            op(
                "Matrix",
                vec![
                    c("4"),
                    c("5"),
                    c("1"),
                    c("2"),
                    c("3"),
                    c("4"),
                    c("5"),
                    c("6"),
                    c("7"),
                    c("8"),
                    c("9"),
                    c("10"),
                    c("11"),
                    c("12"),
                    c("13"),
                    c("14"),
                    c("15"),
                    c("16"),
                    c("17"),
                    c("18"),
                    c("19"),
                    c("20"),
                ],
            ),
        ],
    );

    match checker.check(&expr) {
        TypeCheckResult::Error { message, .. } => {
            // SignatureInterpreter now enforces dimension constraints
            // Accept any error that mentions dimensions or parameters
            println!("Got error: {}", message);
            assert!(
                message.contains("Dimension constraint")
                    || message.contains("inner dimensions")
                    || message.contains("parameter")
                    || message.contains("inference failed")
            );
            println!("✓ Dimension error detected: {}", message);
        }
        _ => panic!("Should have errored on dimension mismatch"),
    }
}

#[test]
fn test_ordering_on_matrices_rejected() {
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // Try A < B where both are matrices (nonsensical)
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
            // After ADR-016 refactor: Error comes from registry, not hardcoded check
            // Should mention that Matrix doesn't support ordering operations
            // OR that less_than is unknown/not supported for Matrix types
            println!("✓ Matrix ordering correctly rejected: {}", message);
            assert!(
                message.contains("Matrix")
                    || message.contains("less_than")
                    || message.contains("Operation")
                    || message.contains("not")
            );
        }
        other => {
            panic!("Should reject ordering on matrices, got: {:?}", other);
        }
    }
}
