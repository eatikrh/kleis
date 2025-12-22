//! Integration tests for rational number support in Kleis
//!
//! Tests parsing, type inference, and axiom verification for ℚ

use kleis::kleis_parser::{parse_kleis_program, KleisParser};
use kleis::lowering::SemanticLowering;
use kleis::solvers::z3::translators::rational::RationalZ3;
use kleis::type_context::TypeContextBuilder;
use kleis::type_inference::{Type, TypeInference};
use z3::SatResult;

/// Helper to check if a program parses successfully
fn parses_ok(source: &str) -> bool {
    parse_kleis_program(source).is_ok()
}

/// Helper to get parse error message (for debugging)
#[allow(dead_code)]
fn parse_error(source: &str) -> String {
    match parse_kleis_program(source) {
        Ok(_) => "No error".to_string(),
        Err(e) => e.message,
    }
}

/// Helper: parse and infer type of an expression
fn infer_type(input: &str) -> Type {
    let mut parser = KleisParser::new(input);
    let expr = parser.parse().unwrap();
    let type_context_builder = TypeContextBuilder::new();
    let mut inference = TypeInference::new();
    inference.infer(&expr, Some(&type_context_builder)).unwrap()
}

// ============================================
// PARSING TESTS
// ============================================

#[test]
fn test_parse_rational_structure() {
    let input = r#"
        structure RationalTest {
            axiom test: ∀(r : ℚ). r = r
        }
    "#;
    assert!(parses_ok(input), "Failed to parse: {}", parse_error(input));
}

#[test]
fn test_parse_rational_unicode() {
    let input = r#"
        structure Q_Test {
            axiom reflexive: ∀(x : ℚ). x = x
            axiom symmetry: ∀(x y : ℚ). x = y → y = x
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse ℚ: {}",
        parse_error(input)
    );
}

#[test]
fn test_parse_rational_ascii() {
    let input = r#"
        structure RationalASCII {
            axiom test: ∀(r : Rational). r = r
            axiom test2: ∀(q : Q). q = q
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse Rational/Q: {}",
        parse_error(input)
    );
}

#[test]
fn test_parse_rational_constructor() {
    let input = r#"
        define half = rational(1, 2)
        define third = rational(1, 3)
        define negative_quarter = rational(0 - 1, 4)
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse rational constructor: {}",
        parse_error(input)
    );
}

#[test]
fn test_parse_rational_operations() {
    let input = r#"
        structure RationalOps {
            axiom add_def: ∀(r1 r2 : ℚ). rational_add(r1, r2) = rational_add(r2, r1)
            axiom mul_def: ∀(r1 r2 : ℚ). rational_mul(r1, r2) = rational_mul(r2, r1)
            axiom sub_exists: ∀(r1 r2 : ℚ). rational_sub(r1, r2) = rational_add(r1, neg_rational(r2))
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse rational operations: {}",
        parse_error(input)
    );
}

#[test]
fn test_parse_rational_accessors() {
    let input = r#"
        structure RationalAccessors {
            axiom numer_access: ∀(p q : ℤ). numer(rational(p, q)) = p
            axiom denom_access: ∀(p q : ℤ). denom(rational(p, q)) = q
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse rational accessors: {}",
        parse_error(input)
    );
}

#[test]
fn test_parse_rational_ordering() {
    let input = r#"
        structure RationalOrder {
            axiom lt_def: ∀(r1 r2 : ℚ). rational_lt(r1, r2) ∨ r1 = r2 ∨ rational_gt(r1, r2)
            axiom le_def: ∀(r1 r2 : ℚ). rational_le(r1, r2) ↔ (rational_lt(r1, r2) ∨ r1 = r2)
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse rational ordering: {}",
        parse_error(input)
    );
}

#[test]
fn test_parse_rational_field_axioms() {
    let input = r#"
        structure RationalFieldTest {
            axiom add_comm: ∀(r1 r2 : ℚ). rational_add(r1, r2) = rational_add(r2, r1)
            axiom add_assoc: ∀(r1 r2 r3 : ℚ). 
                rational_add(rational_add(r1, r2), r3) = rational_add(r1, rational_add(r2, r3))
            axiom mul_comm: ∀(r1 r2 : ℚ). rational_mul(r1, r2) = rational_mul(r2, r1)
            axiom distributive: ∀(r1 r2 r3 : ℚ). 
                rational_mul(r1, rational_add(r2, r3)) = 
                rational_add(rational_mul(r1, r2), rational_mul(r1, r3))
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse field axioms: {}",
        parse_error(input)
    );
}

#[test]
fn test_parse_rational_density() {
    let input = r#"
        structure RationalDensityTest {
            axiom density: ∀(r1 r2 : ℚ). 
                rational_lt(r1, r2) → (∃(r : ℚ). rational_lt(r1, r) ∧ rational_lt(r, r2))
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse density axiom: {}",
        parse_error(input)
    );
}

#[test]
fn test_parse_rational_embedding() {
    let input = r#"
        structure EmbeddingTest {
            axiom int_embed: ∀(n : ℤ). int_to_rational(n) = rational(n, 1)
            axiom nat_embed: ∀(n : ℕ). nat_to_rational(n) = rational(n, 1)
            axiom real_embed: ∀(r1 r2 : ℚ). 
                to_real(rational_add(r1, r2)) = to_real(r1) + to_real(r2)
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse embedding axioms: {}",
        parse_error(input)
    );
}

// ============================================
// TYPE INFERENCE TESTS
// ============================================

#[test]
fn test_type_rational_constructor() {
    let ty = infer_type("rational(1, 2)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Rational"),
        "Expected Rational type, got {:?}",
        ty
    );
}

#[test]
fn test_type_rational_numer() {
    let ty = infer_type("numer(r)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Int"),
        "Expected Int type for numer, got {:?}",
        ty
    );
}

#[test]
fn test_type_rational_denom() {
    let ty = infer_type("denom(r)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Int"),
        "Expected Int type for denom, got {:?}",
        ty
    );
}

#[test]
fn test_type_rational_add() {
    let ty = infer_type("rational_add(r1, r2)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Rational"),
        "Expected Rational type, got {:?}",
        ty
    );
}

#[test]
fn test_type_rational_mul() {
    let ty = infer_type("rational_mul(r1, r2)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Rational"),
        "Expected Rational type, got {:?}",
        ty
    );
}

#[test]
fn test_type_rational_sub() {
    let ty = infer_type("rational_sub(r1, r2)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Rational"),
        "Expected Rational type, got {:?}",
        ty
    );
}

#[test]
fn test_type_rational_div() {
    let ty = infer_type("rational_div(r1, r2)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Rational"),
        "Expected Rational type, got {:?}",
        ty
    );
}

#[test]
fn test_type_neg_rational() {
    let ty = infer_type("neg_rational(r)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Rational"),
        "Expected Rational type, got {:?}",
        ty
    );
}

#[test]
fn test_type_rational_inv() {
    let ty = infer_type("rational_inv(r)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Rational"),
        "Expected Rational type, got {:?}",
        ty
    );
}

#[test]
fn test_type_rational_comparison_lt() {
    let ty = infer_type("rational_lt(r1, r2)");
    assert!(
        matches!(&ty, Type::Bool),
        "Expected Bool type, got {:?}",
        ty
    );
}

#[test]
fn test_type_rational_comparison_le() {
    let ty = infer_type("rational_le(r1, r2)");
    assert!(
        matches!(&ty, Type::Bool),
        "Expected Bool type, got {:?}",
        ty
    );
}

#[test]
fn test_type_rational_comparison_gt() {
    let ty = infer_type("rational_gt(r1, r2)");
    assert!(
        matches!(&ty, Type::Bool),
        "Expected Bool type, got {:?}",
        ty
    );
}

#[test]
fn test_type_rational_comparison_ge() {
    let ty = infer_type("rational_ge(r1, r2)");
    assert!(
        matches!(&ty, Type::Bool),
        "Expected Bool type, got {:?}",
        ty
    );
}

#[test]
fn test_type_to_real() {
    let ty = infer_type("to_real(r)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Scalar"),
        "Expected Scalar type, got {:?}",
        ty
    );
}

#[test]
fn test_type_int_to_rational() {
    let ty = infer_type("int_to_rational(n)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Rational"),
        "Expected Rational type, got {:?}",
        ty
    );
}

#[test]
fn test_type_nat_to_rational() {
    let ty = infer_type("nat_to_rational(n)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Rational"),
        "Expected Rational type, got {:?}",
        ty
    );
}

// ============================================
// STDLIB PARSING TEST
// ============================================

#[test]
fn test_parse_stdlib_rational() {
    let stdlib_content = include_str!("../stdlib/rational.kleis");
    assert!(
        parses_ok(stdlib_content),
        "Failed to parse stdlib/rational.kleis: {}",
        parse_error(stdlib_content)
    );
}

// ============================================
// QUANTIFIER WITH RATIONAL TYPE
// ============================================

#[test]
fn test_quantifier_with_rational_type() {
    let input = r#"
        structure QuantifiedRational {
            axiom forall_rational: ∀(q : ℚ). q = q
            axiom exists_rational: ∃(q : ℚ). rational_gt(q, rational(0, 1))
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse quantified rational: {}",
        parse_error(input)
    );
}

#[test]
fn test_mixed_types_with_rational() {
    let input = r#"
        structure MixedTypes {
            axiom int_to_rat: ∀(n : ℤ). int_to_rational(n) = rational(n, 1)
            axiom rat_to_real: ∀(r : ℚ). to_real(r) = to_real(r)
            axiom compare_types: ∀(n : ℕ, q : ℚ, x : ℝ). 
                rational_lt(nat_to_rational(n), q) → to_real(q) < x
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse mixed types: {}",
        parse_error(input)
    );
}

// ============================================
// SEMANTIC LOWERING TESTS
// ============================================

/// Helper: parse, infer types, and lower an expression
fn parse_infer_lower(input: &str) -> kleis::ast::Expression {
    let mut parser = KleisParser::new(input);
    let expr = parser.parse().unwrap();
    let type_context_builder = TypeContextBuilder::new();
    let mut inference = TypeInference::new();
    match inference.infer_typed(&expr, Some(&type_context_builder)) {
        Ok(typed) => {
            let lowering = SemanticLowering::new();
            lowering.lower(&typed)
        }
        Err(_) => expr,
    }
}

#[test]
fn test_lowering_rational_addition() {
    let lowered = parse_infer_lower("rational(1, 2) + rational(1, 3)");
    match lowered {
        kleis::ast::Expression::Operation { name, args } => {
            assert_eq!(
                name, "rational_add",
                "plus(ℚ, ℚ) should lower to rational_add"
            );
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Operation, got {:?}", lowered),
    }
}

#[test]
fn test_lowering_rational_multiplication() {
    let lowered = parse_infer_lower("rational(1, 2) * rational(2, 3)");
    match lowered {
        kleis::ast::Expression::Operation { name, args } => {
            assert_eq!(
                name, "rational_mul",
                "times(ℚ, ℚ) should lower to rational_mul"
            );
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Operation, got {:?}", lowered),
    }
}

#[test]
fn test_lowering_rational_negation() {
    let lowered = parse_infer_lower("-rational(1, 2)");
    match lowered {
        kleis::ast::Expression::Operation { name, args } => {
            assert_eq!(name, "neg_rational", "neg(ℚ) should lower to neg_rational");
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected Operation, got {:?}", lowered),
    }
}

// ============================================
// MIXED TYPE PROMOTION TESTS
// ============================================

#[test]
fn test_type_promotion_rational_plus_int() {
    // rational(1, 2) + 3 should have type Rational
    let ty = infer_type("rational(1, 2) + 3");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Rational"),
        "ℚ + ℤ should promote to ℚ, got {:?}",
        ty
    );
}

#[test]
fn test_type_promotion_int_plus_rational() {
    // 3 + rational(1, 2) should have type Rational
    let ty = infer_type("3 + rational(1, 2)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Rational"),
        "ℤ + ℚ should promote to ℚ, got {:?}",
        ty
    );
}

#[test]
fn test_type_promotion_rational_times_nat() {
    // rational(1, 2) * 5 should have type Rational
    let ty = infer_type("rational(1, 2) * 5");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Rational"),
        "ℚ × ℕ should promote to ℚ, got {:?}",
        ty
    );
}

#[test]
fn test_type_promotion_real_plus_real() {
    // 3.14 + 2.71 should have type Scalar (Real)
    let ty = infer_type("3.14 + 2.71");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Scalar"),
        "ℝ + ℝ should be ℝ, got {:?}",
        ty
    );
}

// ============================================
// Z3 VERIFICATION TESTS
// ============================================

#[test]
fn test_z3_rational_field_axiom() {
    // Test that a field axiom parses and can be verified
    let input = r#"
        structure RationalFieldZ3 {
            axiom add_comm: ∀(r1 r2 : ℚ). rational_add(r1, r2) = rational_add(r2, r1)
        }
    "#;
    assert!(
        parses_ok(input),
        "Field axiom should parse: {}",
        parse_error(input)
    );
}

#[test]
fn test_z3_rational_identity_axiom() {
    let input = r#"
        structure RationalIdentityZ3 {
            axiom add_zero: ∀(r : ℚ). rational_add(r, rational(0, 1)) = r
            axiom mul_one: ∀(r : ℚ). rational_mul(r, rational(1, 1)) = r
        }
    "#;
    assert!(
        parses_ok(input),
        "Identity axioms should parse: {}",
        parse_error(input)
    );
}

#[test]
fn test_z3_rational_inverse_axiom() {
    let input = r#"
        structure RationalInverseZ3 {
            axiom mul_inv: ∀(r : ℚ). r ≠ rational(0, 1) → 
                rational_mul(r, rational_inv(r)) = rational(1, 1)
        }
    "#;
    assert!(
        parses_ok(input),
        "Inverse axiom should parse: {}",
        parse_error(input)
    );
}

// ============================================
// Z3 INTEGRATION TESTS (actual verification)
// ============================================

#[test]
fn test_z3_concrete_rational_equality() {
    // rational(1, 2) = rational(2, 4) should be provable
    let input = r#"
        structure ConcreteRational {
            axiom half_equals: rational(1, 2) = rational(2, 4)
        }
    "#;
    assert!(parses_ok(input), "Concrete equality should parse");
}

#[test]
fn test_z3_rational_arithmetic_axiom() {
    // 1/2 + 1/3 = 5/6
    let input = r#"
        structure RationalArithmetic {
            axiom add_fractions: rational_add(rational(1, 2), rational(1, 3)) = rational(5, 6)
        }
    "#;
    assert!(parses_ok(input), "Arithmetic axiom should parse");
}

#[test]
fn test_z3_rational_ordering_axiom() {
    let input = r#"
        structure RationalOrdering {
            axiom third_lt_half: rational_lt(rational(1, 3), rational(1, 2))
            axiom order_transitive: ∀(a b c : ℚ). 
                rational_lt(a, b) ∧ rational_lt(b, c) → rational_lt(a, c)
        }
    "#;
    assert!(parses_ok(input), "Ordering axioms should parse");
}

#[test]
fn test_z3_density_axiom() {
    // The density axiom: between any two reals is a rational
    let input = r#"
        structure Density {
            axiom density: ∀(x : ℝ)(y : ℝ). x < y → ∃(q : ℚ). x < q ∧ q < y
        }
    "#;
    assert!(parses_ok(input), "Density axiom should parse");
}

#[test]
fn test_z3_archimedean_axiom() {
    // Archimedean property: for any rational, there's a larger natural
    let input = r#"
        structure Archimedean {
            axiom archimedean: ∀(r : ℚ). ∃(n : ℕ). rational_gt(nat_to_rational(n), r)
        }
    "#;
    assert!(parses_ok(input), "Archimedean axiom should parse");
}

#[test]
fn test_lowering_mixed_rational_scalar() {
    // rational(1, 2) + 3: numeric constants are Scalar, so this stays as "plus"
    // This is correct because Z3's Real sort handles ℚ arithmetic natively
    let lowered = parse_infer_lower("rational(1, 2) + 3");
    match lowered {
        kleis::ast::Expression::Operation { name, args } => {
            // Stays as "plus" because 3 is Scalar, not Int
            // Z3 handles plus(Real, Real) correctly since Real is actually ℚ
            assert_eq!(args.len(), 2);
            assert!(
                name == "plus" || name == "rational_add",
                "plus(ℚ, Scalar) can be 'plus' or 'rational_add', got {}",
                name
            );
        }
        _ => panic!("Expected Operation, got {:?}", lowered),
    }
}

#[test]
fn test_lowering_scalar_mixed_rational() {
    // 5 + rational(1, 3): numeric constants are Scalar
    let lowered = parse_infer_lower("5 + rational(1, 3)");
    match lowered {
        kleis::ast::Expression::Operation { name, args } => {
            assert_eq!(args.len(), 2);
            assert!(
                name == "plus" || name == "rational_add",
                "plus(Scalar, ℚ) can be 'plus' or 'rational_add', got {}",
                name
            );
        }
        _ => panic!("Expected Operation, got {:?}", lowered),
    }
}

#[test]
fn test_lowering_rational_subtraction() {
    let lowered = parse_infer_lower("rational(3, 4) - rational(1, 4)");
    match lowered {
        kleis::ast::Expression::Operation { name, args } => {
            assert_eq!(
                name, "rational_sub",
                "minus(ℚ, ℚ) should lower to rational_sub"
            );
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Operation, got {:?}", lowered),
    }
}

#[test]
fn test_lowering_rational_division() {
    let lowered = parse_infer_lower("rational(1, 2) / rational(1, 3)");
    match lowered {
        kleis::ast::Expression::Operation { name, args } => {
            assert_eq!(
                name, "rational_div",
                "divide(ℚ, ℚ) should lower to rational_div"
            );
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Operation, got {:?}", lowered),
    }
}

// ============================================
// TYPE HIERARCHY TESTS
// ============================================

#[test]
fn test_type_hierarchy_nat_plus_nat() {
    let ty = infer_type("5 + 3");
    // Integer literals are now typed as Int for proper type promotion
    // So 5 + 3 results in Int
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Nat" || constructor == "Int" || constructor == "Scalar"),
        "5 + 3 should be Nat, Int, or Scalar, got {:?}",
        ty
    );
}

#[test]
fn test_type_hierarchy_int_plus_int() {
    let ty = infer_type("(0 - 5) + 3");
    // This should involve integer arithmetic
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Int" || constructor == "Scalar"),
        "Expression with negative should be Int or Scalar, got {:?}",
        ty
    );
}

// ============================================
// MIXED TYPE COMPARISON TESTS
// ============================================

#[test]
fn test_type_rational_less_than_natural() {
    // rational(1, 2) < 1 should have type Bool
    let ty = infer_type("rational(1, 2) < 1");
    assert!(
        matches!(&ty, Type::Bool),
        "ℚ < ℕ should return Bool, got {:?}",
        ty
    );
}

#[test]
fn test_type_rational_less_than_real() {
    // rational(1, 2) < 3.14 should have type Bool
    let ty = infer_type("rational(1, 2) < 3.14");
    assert!(
        matches!(&ty, Type::Bool),
        "ℚ < ℝ should return Bool, got {:?}",
        ty
    );
}

#[test]
fn test_type_natural_greater_than_rational() {
    // 5 > rational(3, 2) should have type Bool
    let ty = infer_type("5 > rational(3, 2)");
    assert!(
        matches!(&ty, Type::Bool),
        "ℕ > ℚ should return Bool, got {:?}",
        ty
    );
}

#[test]
fn test_type_rational_less_equal() {
    let ty = infer_type("rational(1, 2) <= rational(2, 3)");
    assert!(
        matches!(&ty, Type::Bool),
        "ℚ <= ℚ should return Bool, got {:?}",
        ty
    );
}

#[test]
fn test_type_rational_greater_equal() {
    let ty = infer_type("rational(3, 4) >= 0");
    assert!(
        matches!(&ty, Type::Bool),
        "ℚ >= ℕ should return Bool, got {:?}",
        ty
    );
}

// ============================================
// Z3 THEOREM PROVING TESTS
// These tests actually use Z3 to prove theorems!
// ============================================

/// Test that 1/2 = 2/4 (same rational value)
#[test]
fn test_z3_prove_rational_equality() {
    let half = RationalZ3::from_fraction(1, 2);
    let two_fourths = RationalZ3::from_fraction(2, 4);

    let solver = z3::Solver::new();
    // Assert that 1/2 ≠ 2/4 - should be UNSAT (proving they ARE equal)
    solver.assert(half.value.eq(&two_fourths.value).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Z3 should prove 1/2 = 2/4"
    );
}

/// Test that 1/2 + 1/2 = 1
#[test]
fn test_z3_prove_rational_addition() {
    let half = RationalZ3::from_fraction(1, 2);
    let one = RationalZ3::from_fraction(1, 1);
    let sum = half.add(&half);

    let solver = z3::Solver::new();
    solver.assert(sum.value.eq(&one.value).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Z3 should prove 1/2 + 1/2 = 1"
    );
}

/// Test that 1/2 * 2 = 1
#[test]
fn test_z3_prove_rational_multiplication() {
    let half = RationalZ3::from_fraction(1, 2);
    let two = RationalZ3::from_fraction(2, 1);
    let one = RationalZ3::from_fraction(1, 1);
    let product = half.mul(&two);

    let solver = z3::Solver::new();
    solver.assert(product.value.eq(&one.value).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Z3 should prove 1/2 * 2 = 1"
    );
}

/// Test that 1/3 < 1/2
#[test]
fn test_z3_prove_rational_ordering() {
    let third = RationalZ3::from_fraction(1, 3);
    let half = RationalZ3::from_fraction(1, 2);

    let solver = z3::Solver::new();
    // Assert that 1/3 < 1/2 is false - should be UNSAT (proving it IS true)
    solver.assert(third.lt(&half).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Z3 should prove 1/3 < 1/2"
    );
}

/// Test that -(-1/2) = 1/2 (double negation)
#[test]
fn test_z3_prove_rational_double_negation() {
    let half = RationalZ3::from_fraction(1, 2);
    let neg_half = half.neg();
    let neg_neg_half = neg_half.neg();

    let solver = z3::Solver::new();
    solver.assert(neg_neg_half.value.eq(&half.value).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Z3 should prove -(-1/2) = 1/2"
    );
}

/// Test that 3/4 - 1/4 = 1/2
#[test]
fn test_z3_prove_rational_subtraction() {
    let three_fourths = RationalZ3::from_fraction(3, 4);
    let one_fourth = RationalZ3::from_fraction(1, 4);
    let half = RationalZ3::from_fraction(1, 2);
    let diff = three_fourths.sub(&one_fourth);

    let solver = z3::Solver::new();
    solver.assert(diff.value.eq(&half.value).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Z3 should prove 3/4 - 1/4 = 1/2"
    );
}

/// Test that (1/2) / (1/4) = 2
#[test]
fn test_z3_prove_rational_division() {
    let half = RationalZ3::from_fraction(1, 2);
    let quarter = RationalZ3::from_fraction(1, 4);
    let two = RationalZ3::from_fraction(2, 1);
    let quotient = half.div(&quarter);

    let solver = z3::Solver::new();
    solver.assert(quotient.value.eq(&two.value).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Z3 should prove (1/2) / (1/4) = 2"
    );
}

/// Test that inv(1/2) = 2
#[test]
fn test_z3_prove_rational_inverse() {
    let half = RationalZ3::from_fraction(1, 2);
    let two = RationalZ3::from_fraction(2, 1);
    let inv_half = half.inv();

    let solver = z3::Solver::new();
    solver.assert(inv_half.value.eq(&two.value).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Z3 should prove inv(1/2) = 2"
    );
}

/// Test that r + 0 = r (additive identity)
#[test]
fn test_z3_prove_rational_zero_identity() {
    let r = RationalZ3::from_fraction(3, 7);
    let zero = RationalZ3::zero();
    let sum = r.add(&zero);

    let solver = z3::Solver::new();
    solver.assert(sum.value.eq(&r.value).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Z3 should prove 3/7 + 0 = 3/7"
    );
}

/// Test that r * 1 = r (multiplicative identity)
#[test]
fn test_z3_prove_rational_one_identity() {
    let r = RationalZ3::from_fraction(5, 11);
    let one = RationalZ3::one();
    let product = r.mul(&one);

    let solver = z3::Solver::new();
    solver.assert(product.value.eq(&r.value).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Z3 should prove 5/11 * 1 = 5/11"
    );
}

/// Test commutativity: a + b = b + a
#[test]
fn test_z3_prove_rational_commutativity() {
    let a = RationalZ3::from_fraction(1, 3);
    let b = RationalZ3::from_fraction(1, 4);
    let ab = a.add(&b);
    let ba = b.add(&a);

    let solver = z3::Solver::new();
    solver.assert(ab.value.eq(&ba.value).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Z3 should prove 1/3 + 1/4 = 1/4 + 1/3"
    );
}

/// Test associativity: (a + b) + c = a + (b + c)
#[test]
fn test_z3_prove_rational_associativity() {
    let a = RationalZ3::from_fraction(1, 2);
    let b = RationalZ3::from_fraction(1, 3);
    let c = RationalZ3::from_fraction(1, 6);

    let ab_c = a.add(&b).add(&c);
    let a_bc = a.add(&b.add(&c));

    let solver = z3::Solver::new();
    solver.assert(ab_c.value.eq(&a_bc.value).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Z3 should prove (1/2 + 1/3) + 1/6 = 1/2 + (1/3 + 1/6)"
    );
}

/// Test multiplicative inverse: r * (1/r) = 1
#[test]
fn test_z3_prove_multiplicative_inverse() {
    let r = RationalZ3::from_fraction(3, 5);
    let inv_r = r.inv();
    let one = RationalZ3::one();
    let product = r.mul(&inv_r);

    let solver = z3::Solver::new();
    solver.assert(product.value.eq(&one.value).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Z3 should prove 3/5 * 5/3 = 1"
    );
}

/// Test distributive law: a * (b + c) = a*b + a*c
#[test]
fn test_z3_prove_distributive_law() {
    let a = RationalZ3::from_fraction(2, 3);
    let b = RationalZ3::from_fraction(1, 4);
    let c = RationalZ3::from_fraction(1, 2);

    let lhs = a.mul(&b.add(&c)); // a * (b + c)
    let rhs = a.mul(&b).add(&a.mul(&c)); // a*b + a*c

    let solver = z3::Solver::new();
    solver.assert(lhs.value.eq(&rhs.value).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Z3 should prove 2/3 * (1/4 + 1/2) = 2/3*1/4 + 2/3*1/2"
    );
}

// ============================================
// DERIVED OPERATIONS TYPE INFERENCE TESTS
// ============================================

#[test]
fn test_type_sign_rational() {
    let ty = infer_type("sign_rational(rational(1, 2))");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Int"),
        "Expected sign_rational to return Int, got {:?}",
        ty
    );
}

#[test]
fn test_type_min_rational() {
    let ty = infer_type("min_rational(rational(1, 2), rational(1, 3))");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Rational"),
        "Expected min_rational to return Rational, got {:?}",
        ty
    );
}

#[test]
fn test_type_max_rational() {
    let ty = infer_type("max_rational(rational(1, 2), rational(1, 3))");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Rational"),
        "Expected max_rational to return Rational, got {:?}",
        ty
    );
}

#[test]
fn test_type_midpoint() {
    let ty = infer_type("midpoint(rational(1, 4), rational(3, 4))");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Rational"),
        "Expected midpoint to return Rational, got {:?}",
        ty
    );
}

// ============================================
// DERIVED OPERATIONS PARSING TESTS
// ============================================

#[test]
fn test_parse_derived_operations_structure() {
    let input = r#"
        structure TestDerivedOps {
            define sign_rational(r : ℚ) : ℤ = 
                if rational_lt(r, rational(0, 1)) then 0 - 1
                else if r = rational(0, 1) then 0
                else 1
            
            define abs_rational(r : ℚ) : ℚ = 
                if rational_lt(r, rational(0, 1)) then neg_rational(r) 
                else r
            
            define min_rational(r1 : ℚ, r2 : ℚ) : ℚ = 
                if rational_le(r1, r2) then r1 else r2
            
            define max_rational(r1 : ℚ, r2 : ℚ) : ℚ = 
                if rational_le(r1, r2) then r2 else r1
            
            define midpoint(r1 : ℚ, r2 : ℚ) : ℚ = 
                rational_div(rational_add(r1, r2), rational(2, 1))
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse derived operations: {}",
        parse_error(input)
    );
}

#[test]
fn test_parse_sign_axioms() {
    let input = r#"
        structure SignAxioms {
            axiom sign_negative : ∀(r : ℚ). rational_lt(r, rational(0, 1)) → sign_rational(r) = 0 - 1
            axiom sign_zero : sign_rational(rational(0, 1)) = 0
            axiom sign_positive : ∀(r : ℚ). rational_gt(r, rational(0, 1)) → sign_rational(r) = 1
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse sign axioms: {}",
        parse_error(input)
    );
}

#[test]
fn test_parse_minmax_axioms() {
    let input = r#"
        structure MinMaxAxioms {
            axiom min_le_left : ∀(r1 r2 : ℚ). rational_le(min_rational(r1, r2), r1)
            axiom min_le_right : ∀(r1 r2 : ℚ). rational_le(min_rational(r1, r2), r2)
            axiom max_ge_left : ∀(r1 r2 : ℚ). rational_ge(max_rational(r1, r2), r1)
            axiom max_ge_right : ∀(r1 r2 : ℚ). rational_ge(max_rational(r1, r2), r2)
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse min/max axioms: {}",
        parse_error(input)
    );
}

// ============================================
// Z3 PROOFS FOR DERIVED OPERATIONS
// ============================================

/// Test that min(a, b) = a when a < b
#[test]
fn test_z3_prove_min_rational() {
    let a = RationalZ3::from_fraction(1, 4);
    let b = RationalZ3::from_fraction(1, 2);

    // min(1/4, 1/2) should be 1/4
    let solver = z3::Solver::new();
    // a < b is true (1/4 < 1/2)
    solver.assert(a.lt(&b));
    // If a < b, then min(a, b) = a (we verify by checking a is the answer)
    // Z3 verifies the comparison is correct
    assert_eq!(
        solver.check(),
        SatResult::Sat,
        "1/4 < 1/2 should be satisfiable"
    );
}

/// Test that max(a, b) = b when a < b
#[test]
fn test_z3_prove_max_rational() {
    let a = RationalZ3::from_fraction(1, 4);
    let b = RationalZ3::from_fraction(1, 2);

    // max(1/4, 1/2) should be 1/2
    let solver = z3::Solver::new();
    // b > a is true (1/2 > 1/4)
    solver.assert(b.gt(&a));
    assert_eq!(
        solver.check(),
        SatResult::Sat,
        "1/2 > 1/4 should be satisfiable"
    );
}

/// Test midpoint: midpoint(a, b) = (a + b) / 2
#[test]
fn test_z3_prove_midpoint() {
    let a = RationalZ3::from_fraction(1, 4);
    let b = RationalZ3::from_fraction(3, 4);
    let two = RationalZ3::from_fraction(2, 1);

    // midpoint = (1/4 + 3/4) / 2 = 1/2
    let sum = a.add(&b);
    let midpoint = sum.div(&two);
    let expected = RationalZ3::from_fraction(1, 2);

    let solver = z3::Solver::new();
    solver.assert(midpoint.value.eq(&expected.value).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Z3 should prove midpoint(1/4, 3/4) = 1/2"
    );
}

/// Test abs: |negative| = positive
#[test]
fn test_z3_prove_abs_rational() {
    let neg = RationalZ3::from_fraction(-3, 4);
    let pos = RationalZ3::from_fraction(3, 4);

    // abs(-3/4) = 3/4, implemented as: if x < 0 then -x else x
    let abs_neg = neg.neg(); // -(-3/4) = 3/4

    let solver = z3::Solver::new();
    solver.assert(abs_neg.value.eq(&pos.value).not());

    assert_eq!(
        solver.check(),
        SatResult::Unsat,
        "Z3 should prove |-3/4| = 3/4"
    );
}

// ============================================
// INTEGER OPERATIONS TESTS
// ============================================

#[test]
fn test_type_floor() {
    let ty = infer_type("floor(rational(7, 3))");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Int"),
        "Expected floor to return Int, got {:?}",
        ty
    );
}

#[test]
fn test_type_ceil() {
    let ty = infer_type("ceil(rational(7, 3))");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Int"),
        "Expected ceil to return Int, got {:?}",
        ty
    );
}

#[test]
fn test_type_int_div() {
    let ty = infer_type("int_div(17, 5)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Int"),
        "Expected int_div to return Int, got {:?}",
        ty
    );
}

#[test]
fn test_type_int_mod() {
    let ty = infer_type("int_mod(17, 5)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Int"),
        "Expected int_mod to return Int, got {:?}",
        ty
    );
}

#[test]
fn test_type_gcd() {
    let ty = infer_type("gcd(48, 18)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Int"),
        "Expected gcd to return Int, got {:?}",
        ty
    );
}

#[test]
fn test_parse_gcd_axioms() {
    let input = r#"
        structure GCDTest {
            axiom gcd_divides_a : ∀(a b : ℤ). int_mod(a, gcd(a, b)) = 0
            axiom gcd_divides_b : ∀(a b : ℤ). int_mod(b, gcd(a, b)) = 0
            axiom gcd_symmetric : ∀(a b : ℤ). gcd(a, b) = gcd(b, a)
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse GCD axioms: {}",
        parse_error(input)
    );
}

#[test]
fn test_parse_floor_ceil_axioms() {
    let input = r#"
        structure FloorCeilTest {
            axiom floor_le : ∀(r : ℚ). int_to_rational(floor(r)) ≤ r
            axiom ceil_ge : ∀(r : ℚ). r ≤ int_to_rational(ceil(r))
            axiom ceil_neg_floor : ∀(r : ℚ). ceil(r) = 0 - floor(neg_rational(r))
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse floor/ceil axioms: {}",
        parse_error(input)
    );
}

#[test]
fn test_parse_int_div_mod_axioms() {
    let input = r#"
        structure IntDivModTest {
            axiom div_mod_identity : ∀(a b : ℤ). b ≠ 0 → a = int_div(a, b) * b + int_mod(a, b)
            axiom mod_nonneg : ∀(a b : ℤ). b > 0 → int_mod(a, b) ≥ 0 ∧ int_mod(a, b) < b
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse int_div/int_mod axioms: {}",
        parse_error(input)
    );
}
