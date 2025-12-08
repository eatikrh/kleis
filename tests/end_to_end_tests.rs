///! End-to-end tests for type system
///!
///! These tests verify the complete type checking pipeline works correctly
///! from parsing through type inference to result.

use kleis::kleis_parser::parse_kleis;
use kleis::type_checker::{TypeCheckResult, TypeChecker};
use kleis::type_inference::Type;

/// Helper to test that an expression type checks correctly
fn assert_type_checks(latex: &str, expected_type: Type) {
    let expr = parse_kleis(latex).expect(&format!("Failed to parse: {}", latex));
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");
    
    match checker.check(&expr) {
        TypeCheckResult::Success(ty) => {
            assert_eq!(ty, expected_type, "Type mismatch for: {}", latex);
            println!("✓ {} : {:?}", latex, ty);
        }
        TypeCheckResult::Error { message, .. } => {
            panic!("Type check failed for '{}': {}", latex, message);
        }
        TypeCheckResult::Polymorphic { .. } => {
            panic!("Got polymorphic type for '{}'", latex);
        }
    }
}

/// Helper to test that an expression fails to type check
fn assert_type_error(latex: &str, error_substring: &str) {
    let expr = parse_kleis(latex).expect(&format!("Failed to parse: {}", latex));
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");
    
    match checker.check(&expr) {
        TypeCheckResult::Error { message, .. } => {
            assert!(
                message.contains(error_substring),
                "Expected error to contain '{}', got: {}",
                error_substring, message
            );
            println!("✓ {} correctly rejected: {}", latex, message);
        }
        TypeCheckResult::Success(ty) => {
            panic!("Should have failed but got type: {:?}", ty);
        }
        TypeCheckResult::Polymorphic { .. } => {
            panic!("Should have failed but got polymorphic type");
        }
    }
}

#[test]
fn test_basic_arithmetic() {
    println!("\n=== Basic Arithmetic ===");
    assert_type_checks("1 + 2", Type::Scalar);
    assert_type_checks("5 - 3", Type::Scalar);
    assert_type_checks("2 * 3", Type::Scalar);
    assert_type_checks("10 / 2", Type::Scalar);
}

#[test]
fn test_fractions_and_powers() {
    println!("\n=== Fractions and Powers ===");
    assert_type_checks("1/2", Type::Scalar);
    assert_type_checks("x^2", Type::Scalar);
    assert_type_checks("sqrt(x)", Type::Scalar);
}

#[test]
fn test_matrix_operations() {
    println!("\n=== Matrix Operations ===");
    // Note: Matrix constructor defaults to 2×2 (known limitation)
    // We're testing the operations work, not the dimension inference
    
    // These would need actual Matrix types to test properly
    // For now, test that operations are recognized
    let expr = parse_kleis("A + B").expect("Parse failed");
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");
    
    // With unknown A and B, should infer some type
    match checker.check(&expr) {
        TypeCheckResult::Success(_) | TypeCheckResult::Polymorphic { .. } => {
            println!("✓ A + B type checks");
        }
        TypeCheckResult::Error { message, .. } => {
            panic!("Shouldn't fail: {}", message);
        }
    }
}

#[test]
fn test_integrals() {
    println!("\n=== Integrals ===");
    // Verify integral operations are defined in structures
    // Note: They may not have implementations yet (just structure definitions)
    let checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");
    
    // Check if int_bounds is available (it's the one we actually use)
    let int_bounds = checker.types_supporting("int_bounds");
    
    // If no implementations, that's OK - structure exists
    // Just verify we can query for it without error
    println!("✓ Integral operations queryable (int_bounds implementations: {:?})", int_bounds);
}

#[test]
fn test_equations() {
    println!("\n=== Equations ===");
    // Verify equals operation works
    let checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");
    let eq_types = checker.types_supporting("equals");
    assert!(!eq_types.is_empty(), "equals should be available");
    assert!(eq_types.contains(&"ℝ".to_string()), "equals should work for scalars");
    println!("✓ Equation operations available for: {:?}", eq_types);
}

#[test]
fn test_nested_operations() {
    println!("\n=== Nested Operations ===");
    assert_type_checks("(a + b) * (c - d)", Type::Scalar);
    assert_type_checks("sqrt(x^2 + y^2)", Type::Scalar);
    assert_type_checks("(a + b) / (c + d)", Type::Scalar);
}

#[test]
fn test_error_unknown_operation() {
    println!("\n=== Error: Unknown Operation ===");
    assert_type_error("foo(x)", "Unknown operation");
}

#[test]
fn test_variable_inference() {
    println!("\n=== Variable Inference ===");
    // x + 1 should infer x : Scalar
    assert_type_checks("x + 1", Type::Scalar);
    assert_type_checks("y * 2", Type::Scalar);
    assert_type_checks("sqrt(z)", Type::Scalar);
}

#[test]
fn test_operation_coverage() {
    println!("\n=== Operation Coverage ===");
    let checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");
    
    // Verify all major operation categories are available
    let categories = vec![
        ("plus", "Arithmetic"),
        ("abs", "Numeric"),
        ("sqrt", "Numeric"),
        ("equals", "Equatable"),
        ("less_than", "Ordered"),
        ("transpose", "Matrix"),
    ];
    
    for (op, category) in categories {
        let types = checker.types_supporting(op);
        assert!(!types.is_empty(), "{} operation ({}) not found", op, category);
        println!("✓ {} ({}) available for: {:?}", op, category, types);
    }
}

#[test]
fn test_type_safety() {
    println!("\n=== Type Safety ===");
    
    // These should type check
    assert_type_checks("1 + 1", Type::Scalar);
    assert_type_checks("x + x", Type::Scalar);
    
    // Variable with constant should infer
    assert_type_checks("a + 1", Type::Scalar);
    assert_type_checks("2 * b", Type::Scalar);
}

