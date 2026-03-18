//! Tests for type promotion (Lift) functionality
//!
//! These tests verify that the type checker correctly promotes types
//! through the numeric tower: Nat → Int → Rational → Scalar → Complex

use kleis::kleis_parser::KleisParser;
use kleis::type_checker::{TypeCheckResult, TypeChecker};
use kleis::type_inference::Type;

/// Helper to create the test type checker with stdlib
fn create_checker() -> TypeChecker {
    TypeChecker::with_stdlib().expect("Failed to load stdlib")
}

/// Helper to get type as string from an expression
fn infer_type(checker: &mut TypeChecker, expr_str: &str) -> Result<Type, String> {
    let mut parser = KleisParser::new(expr_str);
    let expr = parser.parse().map_err(|e| format!("Parse error: {}", e))?;

    match checker.check(&expr) {
        TypeCheckResult::Success(ty) => Ok(ty),
        TypeCheckResult::Polymorphic { type_var, .. } => Ok(type_var),
        TypeCheckResult::Error { message, .. } => Err(message),
    }
}

/// Helper to extract constructor name from Type
fn get_constructor(ty: &Type) -> Option<String> {
    match ty {
        Type::Data { constructor, .. } => Some(constructor.clone()),
        _ => None,
    }
}

#[test]
fn test_promotions_are_registered() {
    let checker = create_checker();
    let ctx = checker.context_builder();

    // Check that Promotes implementations are registered
    // From stdlib/prelude.kleis:
    // implements Promotes(ℕ, ℤ) { operation lift = nat_to_int }
    // implements Promotes(ℤ, ℚ) { operation lift = int_to_rational }
    // etc.

    // Note: ℕ = Nat, ℤ = Int, ℚ = Rational, ℝ = Scalar, ℂ = Complex

    let has_nat_to_int = ctx.registry().has_promotion("Nat", "Int");
    let has_int_to_rational = ctx.registry().has_promotion("Int", "Rational");
    let has_rational_to_scalar = ctx.registry().has_promotion("Rational", "Scalar");
    let has_scalar_to_complex = ctx.registry().has_promotion("Scalar", "Complex");

    // Also check common direct lifts
    let has_int_to_scalar = ctx.registry().has_promotion("Int", "Scalar");
    let has_nat_to_scalar = ctx.registry().has_promotion("Nat", "Scalar");

    println!("Promotions registered:");
    println!("  Nat -> Int: {}", has_nat_to_int);
    println!("  Int -> Rational: {}", has_int_to_rational);
    println!("  Rational -> Scalar: {}", has_rational_to_scalar);
    println!("  Scalar -> Complex: {}", has_scalar_to_complex);
    println!("  Int -> Scalar: {}", has_int_to_scalar);
    println!("  Nat -> Scalar: {}", has_nat_to_scalar);

    // At least the direct chain should be registered
    assert!(
        has_nat_to_int || has_int_to_rational || has_rational_to_scalar,
        "At least some promotions should be registered from stdlib"
    );
}

#[test]
fn test_find_common_supertype_int_scalar() {
    let checker = create_checker();
    let ctx = checker.context_builder();

    // Int + Scalar should find Scalar as common supertype
    let common = ctx.find_common_supertype("Int", "Scalar");

    println!("find_common_supertype(Int, Scalar) = {:?}", common);

    // Should find Scalar (the larger type)
    assert_eq!(
        common,
        Some("Scalar".to_string()),
        "Int + Scalar should promote to Scalar"
    );
}

#[test]
fn test_type_of_integer_literal() {
    let mut checker = create_checker();

    // Integer literal should be Int
    let ty = infer_type(&mut checker, "1").expect("type inference failed");
    println!(":type 1 = {:?}", ty);

    let con = get_constructor(&ty);
    // Should be Int (not Scalar)
    assert!(
        con.as_deref() == Some("Int"),
        "Integer literal should be Int, got {:?}",
        ty
    );
}

#[test]
fn test_type_of_real_literal() {
    let mut checker = create_checker();

    // Real literal should be Scalar
    let ty = infer_type(&mut checker, "3.14").expect("type inference failed");
    println!(":type 3.14 = {:?}", ty);

    let con = get_constructor(&ty);
    // Should be Scalar
    assert!(
        con.as_deref() == Some("Scalar"),
        "Real literal should be Scalar, got {:?}",
        ty
    );
}

#[test]
fn test_type_of_sin() {
    let mut checker = create_checker();

    // sin(x) should return Scalar (since sin : ℝ → ℝ)
    let ty = infer_type(&mut checker, "sin(x)").expect("type inference failed");
    println!(":type sin(x) = {:?}", ty);

    let con = get_constructor(&ty);
    // Should be Scalar (or Real)
    assert!(
        con.as_deref() == Some("Scalar") || con.as_deref() == Some("Real"),
        "sin(x) should be Scalar or Real, got {:?}",
        ty
    );
}

#[test]
fn test_type_promotion_int_plus_scalar() {
    let mut checker = create_checker();

    // THE KEY TEST: 1 + sin(x) should be Scalar, not Int
    // Because:
    // - 1 is Int
    // - sin(x) is Scalar
    // - Promotes(Int, Scalar) exists
    // - Common supertype is Scalar

    let ty = infer_type(&mut checker, "1 + sin(x)").expect("type inference failed");
    println!(":type 1 + sin(x) = {:?}", ty);

    let con = get_constructor(&ty);
    // Should be Scalar (promoted from Int)
    assert!(
        con.as_deref() == Some("Scalar"),
        "1 + sin(x) should be Scalar (not Int!), got {:?}",
        ty
    );
}

#[test]
fn test_type_promotion_int_plus_real_literal() {
    let mut checker = create_checker();

    // 1 + 3.14 should be Scalar
    let ty = infer_type(&mut checker, "1 + 3.14").expect("type inference failed");
    println!(":type 1 + 3.14 = {:?}", ty);

    let con = get_constructor(&ty);
    assert!(
        con.as_deref() == Some("Scalar"),
        "1 + 3.14 should be Scalar, got {:?}",
        ty
    );
}

#[test]
fn test_type_promotion_in_division() {
    let mut checker = create_checker();

    // (1 + sin(x)) / 2 should still be Scalar
    let ty = infer_type(&mut checker, "(1 + sin(x)) / 2").expect("type inference failed");
    println!(":type (1 + sin(x)) / 2 = {:?}", ty);

    let con = get_constructor(&ty);
    assert!(
        con.as_deref() == Some("Scalar"),
        "(1 + sin(x)) / 2 should be Scalar, got {:?}",
        ty
    );
}
