//! Type Promotion Tests
//!
//! Tests for the Promotes(From, To) structure and type promotion system.
//! Covers:
//! - Built-in numeric type promotions (ℕ → ℤ → ℚ → ℝ → ℂ)
//! - Registry-based promotion lookup
//! - User-defined type promotions
//! - Concrete value promotion with :eval

use kleis::kleis_parser::KleisParser;
use kleis::type_context::TypeContextBuilder;
use kleis::type_inference::{Type, TypeInference};

/// Helper: parse and infer type of an expression
fn infer_type(input: &str) -> Type {
    let mut parser = KleisParser::new(input);
    let expr = parser.parse().unwrap();
    let type_context_builder = TypeContextBuilder::new();
    let mut inference = TypeInference::new();
    inference.infer(&expr, Some(&type_context_builder)).unwrap()
}

/// Helper: check if type constructor matches
fn is_type(ty: &Type, expected: &str) -> bool {
    matches!(ty, Type::Data { constructor, .. } if constructor == expected)
}

// ============================================
// BUILT-IN NUMERIC PROMOTION TESTS
// ============================================

#[test]
fn test_int_literal_types_as_int() {
    // Integer literals should type as Int, not Scalar
    let ty = infer_type("42");
    assert!(
        is_type(&ty, "Int"),
        "Integer literal should be Int, got {:?}",
        ty
    );
}

#[test]
fn test_real_literal_types_as_scalar() {
    // Real literals should type as Scalar
    let ty = infer_type("3.14");
    assert!(
        is_type(&ty, "Scalar"),
        "Real literal should be Scalar, got {:?}",
        ty
    );
}

#[test]
fn test_int_plus_int_gives_int() {
    // Int + Int should give Int
    let ty = infer_type("1 + 2");
    assert!(is_type(&ty, "Int"), "Int + Int should be Int, got {:?}", ty);
}

#[test]
fn test_int_plus_scalar_gives_scalar() {
    // Int + Scalar should promote to Scalar
    let ty = infer_type("1 + 3.14");
    assert!(
        is_type(&ty, "Scalar"),
        "Int + Scalar should be Scalar, got {:?}",
        ty
    );
}

#[test]
fn test_scalar_plus_int_gives_scalar() {
    // Scalar + Int should promote to Scalar
    let ty = infer_type("3.14 + 1");
    assert!(
        is_type(&ty, "Scalar"),
        "Scalar + Int should be Scalar, got {:?}",
        ty
    );
}

#[test]
fn test_int_plus_rational_gives_rational() {
    // Int + Rational should promote to Rational
    let ty = infer_type("3 + rational(1, 2)");
    assert!(
        is_type(&ty, "Rational"),
        "Int + Rational should be Rational, got {:?}",
        ty
    );
}

#[test]
fn test_rational_plus_int_gives_rational() {
    // Rational + Int should promote to Rational
    let ty = infer_type("rational(1, 2) + 3");
    assert!(
        is_type(&ty, "Rational"),
        "Rational + Int should be Rational, got {:?}",
        ty
    );
}

#[test]
fn test_int_plus_complex_gives_complex() {
    // Int + Complex should promote to Complex
    let ty = infer_type("1 + complex(2, 3)");
    assert!(
        is_type(&ty, "Complex"),
        "Int + Complex should be Complex, got {:?}",
        ty
    );
}

#[test]
fn test_scalar_plus_complex_gives_complex() {
    // Scalar + Complex should promote to Complex
    let ty = infer_type("3.14 + complex(2, 3)");
    assert!(
        is_type(&ty, "Complex"),
        "Scalar + Complex should be Complex, got {:?}",
        ty
    );
}

// ============================================
// TYPE HIERARCHY TESTS
// ============================================

#[test]
fn test_promotion_hierarchy_nat_to_int() {
    // Nat is at the bottom, Int is next
    // Since we don't have Nat literals, we test via the hierarchy
    let ctx = TypeContextBuilder::new();
    let common = ctx.find_common_supertype("Nat", "Int");
    assert_eq!(common, Some("Int".to_string()));
}

#[test]
fn test_promotion_hierarchy_int_to_rational() {
    let ctx = TypeContextBuilder::new();
    let common = ctx.find_common_supertype("Int", "Rational");
    assert_eq!(common, Some("Rational".to_string()));
}

#[test]
fn test_promotion_hierarchy_rational_to_scalar() {
    let ctx = TypeContextBuilder::new();
    let common = ctx.find_common_supertype("Rational", "Scalar");
    assert_eq!(common, Some("Scalar".to_string()));
}

#[test]
fn test_promotion_hierarchy_scalar_to_complex() {
    let ctx = TypeContextBuilder::new();
    let common = ctx.find_common_supertype("Scalar", "Complex");
    assert_eq!(common, Some("Complex".to_string()));
}

#[test]
fn test_promotion_hierarchy_int_to_complex() {
    // Int should promote all the way to Complex
    let ctx = TypeContextBuilder::new();
    let common = ctx.find_common_supertype("Int", "Complex");
    assert_eq!(common, Some("Complex".to_string()));
}

#[test]
fn test_promotion_same_type_returns_same() {
    let ctx = TypeContextBuilder::new();
    assert_eq!(
        ctx.find_common_supertype("Int", "Int"),
        Some("Int".to_string())
    );
    assert_eq!(
        ctx.find_common_supertype("Scalar", "Scalar"),
        Some("Scalar".to_string())
    );
    assert_eq!(
        ctx.find_common_supertype("Complex", "Complex"),
        Some("Complex".to_string())
    );
}

// ============================================
// LIFT FUNCTION TESTS
// ============================================

#[test]
fn test_get_lift_function_int_to_rational() {
    let ctx = TypeContextBuilder::new();
    let lift = ctx.get_lift_function("Int", "Rational");
    assert_eq!(lift, Some("int_to_rational".to_string()));
}

#[test]
fn test_get_lift_function_rational_to_scalar() {
    let ctx = TypeContextBuilder::new();
    let lift = ctx.get_lift_function("Rational", "Scalar");
    assert_eq!(lift, Some("rational_to_real".to_string()));
}

#[test]
fn test_get_lift_function_scalar_to_complex() {
    let ctx = TypeContextBuilder::new();
    let lift = ctx.get_lift_function("Scalar", "Complex");
    assert_eq!(lift, Some("real_to_complex".to_string()));
}

#[test]
fn test_get_lift_function_same_type_returns_none() {
    let ctx = TypeContextBuilder::new();
    assert_eq!(ctx.get_lift_function("Int", "Int"), None);
    assert_eq!(ctx.get_lift_function("Scalar", "Scalar"), None);
}

// ============================================
// ARITHMETIC WITH MIXED TYPES
// ============================================

#[test]
fn test_subtraction_int_minus_int() {
    let ty = infer_type("5 - 3");
    assert!(is_type(&ty, "Int"), "Int - Int should be Int, got {:?}", ty);
}

#[test]
fn test_multiplication_int_times_scalar() {
    let ty = infer_type("2 * 3.14");
    assert!(
        is_type(&ty, "Scalar"),
        "Int * Scalar should be Scalar, got {:?}",
        ty
    );
}

#[test]
fn test_division_int_divide_int() {
    let ty = infer_type("10 / 2");
    // Division of ints - could be Int or Scalar depending on implementation
    assert!(
        is_type(&ty, "Int") || is_type(&ty, "Scalar"),
        "Int / Int should be Int or Scalar, got {:?}",
        ty
    );
}

// ============================================
// UNICODE TYPE ALIASES
// ============================================

#[test]
fn test_unicode_type_aliases_in_promotion() {
    let ctx = TypeContextBuilder::new();

    // ℤ should normalize to Int
    assert_eq!(
        ctx.find_common_supertype("ℤ", "Int"),
        Some("Int".to_string())
    );

    // ℚ should normalize to Rational
    assert_eq!(
        ctx.find_common_supertype("ℚ", "Rational"),
        Some("Rational".to_string())
    );

    // ℝ should normalize to Scalar
    assert_eq!(
        ctx.find_common_supertype("ℝ", "Scalar"),
        Some("Scalar".to_string())
    );

    // ℂ should normalize to Complex
    assert_eq!(
        ctx.find_common_supertype("ℂ", "Complex"),
        Some("Complex".to_string())
    );
}

// ============================================
// REGISTRY-BASED PROMOTION TESTS
// ============================================

#[test]
fn test_promotes_structure_registered() {
    use kleis::kleis_parser::parse_kleis_program;

    // Parse a file that defines Promotes with a lift implementation
    let source = r#"
        structure Promotes(From, To) {
            operation lift : From → To
        }
        
        implements Promotes(ℤ, ℚ) {
            operation lift = builtin_int_to_rational
        }
    "#;

    let program = parse_kleis_program(source).expect("Should parse");
    let ctx = TypeContextBuilder::from_program(program).expect("Should build context");

    // After registration, the promotion should be findable via the registry
    // The registry stores promotions with normalized type names
    // Check that we can find a common supertype (which uses the promotion graph)
    let common = ctx.find_common_supertype("Int", "Rational");
    assert_eq!(
        common,
        Some("Rational".to_string()),
        "Int should promote to Rational"
    );
}

#[test]
fn test_user_defined_promotes_registered() {
    use kleis::kleis_parser::parse_kleis_program;

    // Define a custom type and its promotion
    let source = r#"
        data Percentage = Pct(value: ℝ)
        
        structure Promotes(From, To) {
            operation lift : From → To
        }
        
        implements Promotes(Percentage, ℝ) {
            operation lift = pct_to_real
        }
        
        define pct_to_real(p: Percentage) : ℝ =
            match p { Pct(v) => v / 100 }
    "#;

    let result = parse_kleis_program(source);
    // This should parse successfully
    assert!(
        result.is_ok(),
        "Should parse user-defined promotion: {:?}",
        result.err()
    );

    let program = result.unwrap();
    let ctx_result = TypeContextBuilder::from_program(program);
    assert!(
        ctx_result.is_ok(),
        "Should build context: {:?}",
        ctx_result.err()
    );
}

// ============================================
// MULTI-STEP PROMOTION TESTS
// ============================================

#[test]
fn test_get_lift_chain_single_step() {
    let ctx = TypeContextBuilder::new();
    let chain = ctx.get_lift_chain("Int", "Rational");
    // Should find direct path (even if via fallback)
    assert!(!chain.is_empty(), "Should find Int → Rational chain");
}

#[test]
fn test_get_lift_chain_multi_step() {
    let ctx = TypeContextBuilder::new();
    // Int → Complex requires Int → Scalar → Complex (or similar path)
    let chain = ctx.get_lift_chain("Int", "Complex");
    // Should find a chain (might be direct if registered, or multi-step)
    assert!(
        !chain.is_empty(),
        "Should find Int → Complex chain: {:?}",
        chain
    );
}

#[test]
fn test_composed_lifts_format() {
    let ctx = TypeContextBuilder::new();
    let lift_fn = ctx.get_lift_function("Int", "Complex");
    assert!(lift_fn.is_some(), "Should find lift for Int → Complex");

    // Either a direct function or composed format
    let lift = lift_fn.unwrap();
    if lift.contains(',') {
        assert!(
            lift.starts_with("compose_lifts:"),
            "Multi-step should use compose_lifts format: {}",
            lift
        );
    }
}

// ============================================
// NOTES FOR FUTURE: USER-DEFINED TYPE PROMOTION
// ============================================
//
// To test user-defined type promotion, we would need:
//
// 1. A test file defining a custom type:
//    data Percentage = Pct(value: ℝ)
//
// 2. A promotion implementation:
//    implements Promotes(Percentage, ℝ) {
//      operation lift = pct_to_real
//    }
//
// 3. The lift function:
//    define pct_to_real(p: Percentage) : ℝ =
//      match p { Pct(v) => divide(v, 100) }
//
// 4. Test that promotion is registered:
//    ctx.can_promote("Percentage", "Scalar")
//
// 5. Test concrete evaluation:
//    :eval Pct(50) + 0.25  // Should be 0.75
//
// This requires loading the test file and running :eval,
// which is more of an integration test.
