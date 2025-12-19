//! Integration tests for rational number support in Kleis
//!
//! Tests parsing, type inference, and axiom verification for ℚ

use kleis::kleis_parser::{parse_kleis_program, KleisParser};
use kleis::type_context::TypeContextBuilder;
use kleis::type_inference::{Type, TypeInference};

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
