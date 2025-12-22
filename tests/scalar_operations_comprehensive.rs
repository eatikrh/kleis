#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
///! Comprehensive tests for all scalar operations
///!
///! This test suite verifies that ALL scalar operations used by the parser
///! are available in the stdlib and type-check correctly.
///!
///! Purpose: Catch any missing operations before users encounter errors.
use kleis::ast::Expression;
use kleis::type_checker::{TypeCheckResult, TypeChecker};
use kleis::type_inference::Type;

/// Helper to create a const expression
fn c(s: &str) -> Expression {
    Expression::Const(s.to_string())
}

/// Helper to create a variable expression
fn var(s: &str) -> Expression {
    Expression::Object(s.to_string())
}

/// Helper to create an operation
fn op(name: &str, args: Vec<Expression>) -> Expression {
    Expression::operation(name, args)
}

/// Test a scalar operation and expect Scalar type
fn test_scalar_op(op_name: &str, args: Vec<Expression>) {
    test_op_returns_type(op_name, args, Type::scalar());
}

/// Test an operation that returns Int type
fn test_int_op(op_name: &str, args: Vec<Expression>) {
    let int_type = Type::Data {
        type_name: "Type".to_string(),
        constructor: "Int".to_string(),
        args: vec![],
    };
    test_op_returns_type(op_name, args, int_type);
}

/// Test an operation and expect a specific type
fn test_op_returns_type(op_name: &str, args: Vec<Expression>, expected: Type) {
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    let expr = op(op_name, args);
    let result = checker.check(&expr);

    match result {
        TypeCheckResult::Success(ty) => {
            // If expecting Scalar, also accept Int (integer literals now type as Int)
            let is_match = if matches!(&expected, Type::Data { constructor, .. } if constructor == "Scalar")
            {
                matches!(&ty, Type::Data { constructor, .. } if constructor == "Scalar" || constructor == "Int")
            } else {
                ty == expected
            };
            assert!(
                is_match,
                "Operation '{}' should return {:?}, got {:?}",
                op_name, expected, ty
            );
        }
        TypeCheckResult::Error {
            message,
            suggestion,
        } => {
            panic!(
                "Operation '{}' failed type checking:\n  Error: {}\n  Suggestion: {:?}",
                op_name, message, suggestion
            );
        }
        TypeCheckResult::Polymorphic {
            type_var,
            available_types,
        } => {
            // Polymorphic is OK for operations with unknowns
            println!(
                "✓ Operation '{}' returned polymorphic type: {:?}, available: {:?}",
                op_name, type_var, available_types
            );
        }
    }
}

#[test]
fn test_all_arithmetic_operations() {
    println!("\n=== Testing Arithmetic Operations ===");

    // Addition
    test_scalar_op("plus", vec![c("1"), c("2")]);
    println!("✓ plus");

    // Subtraction
    test_scalar_op("minus", vec![c("5"), c("3")]);
    println!("✓ minus");

    // Multiplication
    test_scalar_op("times", vec![c("2"), c("3")]);
    println!("✓ times");

    test_scalar_op("scalar_multiply", vec![c("2"), c("3")]);
    println!("✓ scalar_multiply");

    // Division
    test_scalar_op("divide", vec![c("10"), c("2")]);
    println!("✓ divide");

    test_scalar_op("scalar_divide", vec![c("10"), c("2")]);
    println!("✓ scalar_divide");

    test_scalar_op("frac", vec![c("1"), c("2")]);
    println!("✓ frac");
}

#[test]
fn test_all_numeric_operations() {
    println!("\n=== Testing Numeric Operations ===");

    // Square root
    test_scalar_op("sqrt", vec![c("4")]);
    println!("✓ sqrt");

    // Absolute value
    test_scalar_op("abs", vec![c("-5")]);
    println!("✓ abs");

    // Floor - returns Int (floor : ℝ → ℤ)
    test_int_op("floor", vec![c("3.7")]);
    println!("✓ floor");

    // Power
    test_scalar_op("power", vec![c("2"), c("3")]);
    println!("✓ power");

    // Superscript (same as power)
    test_scalar_op("sup", vec![c("2"), c("3")]);
    println!("✓ sup");
}

#[test]
fn test_nested_scalar_expressions() {
    println!("\n=== Testing Nested Scalar Expressions ===");

    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // (1 + 2) * 3
    let expr1 = op(
        "scalar_multiply",
        vec![op("plus", vec![c("1"), c("2")]), c("3")],
    );
    match checker.check(&expr1) {
        TypeCheckResult::Success(ty) => {
            // Accept Int (integer literals now type as Int) or Scalar or Var
            assert!(
                matches!(&ty, Type::Data { constructor, .. } if constructor == "Scalar" || constructor == "Int")
                    || matches!(&ty, Type::Var(_)),
                "Expected Scalar, Int, or Var, got {:?}",
                ty
            );
            println!("✓ (1 + 2) * 3 : {:?}", ty);
        }
        _ => panic!("Failed to type check (1 + 2) * 3"),
    }

    // √(x / (x + 1))
    let expr2 = op(
        "sqrt",
        vec![op(
            "scalar_divide",
            vec![var("x"), op("plus", vec![var("x"), c("1")])],
        )],
    );
    match checker.check(&expr2) {
        TypeCheckResult::Success(ty) => {
            // Accept Int (integer literals now type as Int) or Scalar or Var
            assert!(
                matches!(&ty, Type::Data { constructor, .. } if constructor == "Scalar" || constructor == "Int")
                    || matches!(&ty, Type::Var(_)),
                "Expected Scalar, Int, or Var, got {:?}",
                ty
            );
            println!("✓ √(x / (x + 1)) : {:?}", ty);
        }
        _ => panic!("Failed to type check √(x / (x + 1))"),
    }

    // (a + b) / (c - d)
    // Note: With all unknowns, HM returns a type variable (correct!)
    let expr3 = op(
        "scalar_divide",
        vec![
            op("plus", vec![var("a"), var("b")]),
            op("minus", vec![var("c"), var("d")]),
        ],
    );
    match checker.check(&expr3) {
        TypeCheckResult::Success(ty) => {
            // Could be Scalar or Var - both are valid
            if ty == Type::scalar() {
                println!("✓ (a + b) / (c - d) : Scalar");
            } else if matches!(ty, Type::Var(_)) {
                println!("✓ (a + b) / (c - d) : TypeVar (correct HM behavior)");
            } else {
                panic!("Unexpected type: {:?}", ty);
            }
        }
        _ => panic!("Failed to type check (a + b) / (c - d)"),
    }
}

#[test]
fn test_variable_inference_with_scalars() {
    println!("\n=== Testing Variable Inference ===");

    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // x + 1 should infer x : Scalar
    // With proper HM substitution: Var(x) + Scalar → Scalar (substitution applied!)
    let expr = op("plus", vec![var("x"), c("1")]);
    match checker.check(&expr) {
        TypeCheckResult::Success(ty) => {
            // Accept Int (integer literals now type as Int) or Scalar or Var
            assert!(
                matches!(&ty, Type::Data { constructor, .. } if constructor == "Scalar" || constructor == "Int")
                    || matches!(&ty, Type::Var(_)),
                "Expected Scalar, Int, or Var, got {:?}",
                ty
            );
            println!("✓ x + 1 infers: {:?}", ty);
        }
        _ => panic!("Failed to infer x + 1"),
    }

    // y * 2 should infer y : Scalar
    // With proper HM substitution: Var(y) * Scalar → Scalar
    let expr2 = op("scalar_multiply", vec![var("y"), c("2")]);
    match checker.check(&expr2) {
        TypeCheckResult::Success(ty) => {
            // Accept Int (integer literals now type as Int) or Scalar or Var
            assert!(
                matches!(&ty, Type::Data { constructor, .. } if constructor == "Scalar" || constructor == "Int")
                    || matches!(&ty, Type::Var(_)),
                "Expected Scalar, Int, or Var, got {:?}",
                ty
            );
            println!("✓ y * 2 infers: {:?}", ty);
        }
        _ => panic!("Failed to infer y * 2"),
    }
}

#[test]
fn test_error_cases() {
    println!("\n=== Testing Error Cases ===");

    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // Wrong number of arguments
    let expr = op("plus", vec![c("1")]);
    match checker.check(&expr) {
        TypeCheckResult::Error { message, .. } => {
            // SignatureInterpreter may give different error
            assert!(
                message.contains("requires 2 arguments")
                    || message.contains("requires both")
                    || message.contains("Argument count")
                    || message.contains("inference failed")
            );
            println!("✓ plus with 1 arg correctly errors: {}", message);
        }
        _ => panic!("Expected error for plus with wrong number of args"),
    }
}

#[test]
fn test_all_operations_exist() {
    println!("\n=== Checking All Operations Exist in Registry ===");

    let checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // List of all scalar operations the parser can generate
    let operations = vec![
        "plus",
        "minus",
        "times",
        "scalar_multiply",
        "divide",
        "scalar_divide",
        "frac",
        "sqrt",
        "abs",
        "floor",
        "power",
        "sup",
    ];

    for op_name in operations {
        let types = checker.types_supporting(op_name);
        assert!(
            !types.is_empty(),
            "❌ Operation '{}' not found in stdlib! Available operations should include this.",
            op_name
        );
        println!("✓ {} is available for types: {:?}", op_name, types);
    }
}

#[test]
fn test_trig_operations() {
    println!("\n=== Testing Trigonometric Operations ===");

    let checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // Check if trig operations are in stdlib
    // (They're declared in full prelude.kleis but not in minimal yet)
    let trig_ops = vec!["sin", "cos", "tan", "exp", "ln"];

    for op_name in trig_ops {
        let types = checker.types_supporting(op_name);
        if types.is_empty() {
            println!(
                "⚠️  {} not in minimal stdlib (will be in full prelude)",
                op_name
            );
        } else {
            println!("✓ {} available for: {:?}", op_name, types);
        }
    }
}

#[test]
fn test_complex_scalar_expression() {
    println!("\n=== Testing Complex Scalar Expression ===");

    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // ((a + b) * (c - d)) / (e + f)
    // With all unknowns, HM returns a type variable (this is correct!)
    let expr = op(
        "scalar_divide",
        vec![
            op(
                "scalar_multiply",
                vec![
                    op("plus", vec![var("a"), var("b")]),
                    op("minus", vec![var("c"), var("d")]),
                ],
            ),
            op("plus", vec![var("e"), var("f")]),
        ],
    );

    match checker.check(&expr) {
        TypeCheckResult::Success(ty) => {
            if ty == Type::scalar() {
                println!("✓ Complex expression: ((a + b) * (c - d)) / (e + f) : Scalar");
            } else if matches!(ty, Type::Var(_)) {
                println!(
                    "✓ Complex expression: ((a + b) * (c - d)) / (e + f) : TypeVar (correct HM!)"
                );
            } else {
                panic!("Unexpected type: {:?}", ty);
            }
        }
        TypeCheckResult::Error { message, .. } => {
            panic!("Failed to type check complex expression: {}", message);
        }
        TypeCheckResult::Polymorphic { .. } => {
            println!("✓ Complex expression returned polymorphic type (acceptable)");
        }
    }
}
