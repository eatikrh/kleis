//! Integration tests for bit-vector support in Kleis
//!
//! Tests parsing, type inference, and Z3 BitVec operations
//! Based on Bourbaki-style formalization: BitVec(n) as x : [0,n-1] → {0,1}

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
fn test_parse_bitvector_stdlib() {
    let stdlib = include_str!("../stdlib/bitvector.kleis");
    assert!(
        parses_ok(stdlib),
        "Failed to parse stdlib/bitvector.kleis: {}",
        parse_error(stdlib)
    );
}

#[test]
fn test_parse_bitvector_xor_axioms() {
    let input = r#"
        structure XorAxioms {
            axiom xor_commutative : ∀(n : ℕ)(x y : BitVec(n)).
                bvxor(x, y) = bvxor(y, x)
            
            axiom xor_inverse : ∀(n : ℕ)(x : BitVec(n)).
                bvxor(x, x) = bvzero(n)
        }
    "#;
    assert!(parses_ok(input), "Failed to parse: {}", parse_error(input));
}

#[test]
fn test_parse_bitvector_boolean_algebra() {
    let input = r#"
        structure BooleanAlgebraAxioms {
            axiom demorgan_and : ∀(n : ℕ)(x y : BitVec(n)).
                bvnot(bvand(x, y)) = bvor(bvnot(x), bvnot(y))
            
            axiom demorgan_or : ∀(n : ℕ)(x y : BitVec(n)).
                bvnot(bvor(x, y)) = bvand(bvnot(x), bvnot(y))
        }
    "#;
    assert!(parses_ok(input), "Failed to parse: {}", parse_error(input));
}

#[test]
fn test_parse_bitvector_arithmetic() {
    let input = r#"
        structure ArithmeticAxioms {
            axiom add_commutative : ∀(n : ℕ)(x y : BitVec(n)).
                bvadd(x, y) = bvadd(y, x)
            
            axiom neg_inverse : ∀(n : ℕ)(x : BitVec(n)).
                bvadd(x, bvneg(x)) = bvzero(n)
        }
    "#;
    assert!(parses_ok(input), "Failed to parse: {}", parse_error(input));
}

#[test]
fn test_parse_bitvector_order() {
    let input = r#"
        structure OrderAxioms {
            axiom ult_irreflexive : ∀(n : ℕ)(x : BitVec(n)).
                ¬bvult(x, x)
            
            axiom ult_transitive : ∀(n : ℕ)(x y z : BitVec(n)).
                bvult(x, y) ∧ bvult(y, z) → bvult(x, z)
        }
    "#;
    assert!(parses_ok(input), "Failed to parse: {}", parse_error(input));
}

#[test]
fn test_parse_bitvector_shift() {
    let input = r#"
        structure ShiftAxioms {
            axiom shl_zero : ∀(n : ℕ)(x : BitVec(n)).
                bvshl(x, bvzero(n)) = x
            
            axiom lshr_zero : ∀(n : ℕ)(x : BitVec(n)).
                bvlshr(x, bvzero(n)) = x
        }
    "#;
    assert!(parses_ok(input), "Failed to parse: {}", parse_error(input));
}

// ============================================
// TYPE INFERENCE TESTS
// ============================================

#[test]
fn test_type_bvand() {
    let ty = infer_type("bvand(x, y)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "BitVec"),
        "Expected bvand to return BitVec, got {:?}",
        ty
    );
}

#[test]
fn test_type_bvor() {
    let ty = infer_type("bvor(x, y)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "BitVec"),
        "Expected bvor to return BitVec, got {:?}",
        ty
    );
}

#[test]
fn test_type_bvxor() {
    let ty = infer_type("bvxor(x, y)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "BitVec"),
        "Expected bvxor to return BitVec, got {:?}",
        ty
    );
}

#[test]
fn test_type_bvnot() {
    let ty = infer_type("bvnot(x)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "BitVec"),
        "Expected bvnot to return BitVec, got {:?}",
        ty
    );
}

#[test]
fn test_type_bvadd() {
    let ty = infer_type("bvadd(x, y)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "BitVec"),
        "Expected bvadd to return BitVec, got {:?}",
        ty
    );
}

#[test]
fn test_type_bvsub() {
    let ty = infer_type("bvsub(x, y)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "BitVec"),
        "Expected bvsub to return BitVec, got {:?}",
        ty
    );
}

#[test]
fn test_type_bvmul() {
    let ty = infer_type("bvmul(x, y)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "BitVec"),
        "Expected bvmul to return BitVec, got {:?}",
        ty
    );
}

#[test]
fn test_type_bvneg() {
    let ty = infer_type("bvneg(x)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "BitVec"),
        "Expected bvneg to return BitVec, got {:?}",
        ty
    );
}

#[test]
fn test_type_bvshl() {
    let ty = infer_type("bvshl(x, y)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "BitVec"),
        "Expected bvshl to return BitVec, got {:?}",
        ty
    );
}

#[test]
fn test_type_bvlshr() {
    let ty = infer_type("bvlshr(x, y)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "BitVec"),
        "Expected bvlshr to return BitVec, got {:?}",
        ty
    );
}

#[test]
fn test_type_bvashr() {
    let ty = infer_type("bvashr(x, y)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "BitVec"),
        "Expected bvashr to return BitVec, got {:?}",
        ty
    );
}

#[test]
fn test_type_bvult() {
    let ty = infer_type("bvult(x, y)");
    assert!(
        matches!(&ty, Type::Bool),
        "Expected bvult to return Bool, got {:?}",
        ty
    );
}

#[test]
fn test_type_bvule() {
    let ty = infer_type("bvule(x, y)");
    assert!(
        matches!(&ty, Type::Bool),
        "Expected bvule to return Bool, got {:?}",
        ty
    );
}

#[test]
fn test_type_bvslt() {
    let ty = infer_type("bvslt(x, y)");
    assert!(
        matches!(&ty, Type::Bool),
        "Expected bvslt to return Bool, got {:?}",
        ty
    );
}

#[test]
fn test_type_bvsle() {
    let ty = infer_type("bvsle(x, y)");
    assert!(
        matches!(&ty, Type::Bool),
        "Expected bvsle to return Bool, got {:?}",
        ty
    );
}

#[test]
fn test_type_bvzero() {
    let ty = infer_type("bvzero(8)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "BitVec"),
        "Expected bvzero to return BitVec, got {:?}",
        ty
    );
}

#[test]
fn test_type_bvones() {
    let ty = infer_type("bvones(8)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "BitVec"),
        "Expected bvones to return BitVec, got {:?}",
        ty
    );
}

// ============================================
// BOURBAKI STRUCTURE TESTS
// ============================================

#[test]
fn test_parse_vector_space_structure() {
    // BitVec(n) as a vector space over 𝔽₂
    let input = r#"
        structure VectorSpaceF2 {
            // XOR is the addition operation
            axiom add_commutative : ∀(n : ℕ)(x y : BitVec(n)).
                bvxor(x, y) = bvxor(y, x)
            
            axiom add_associative : ∀(n : ℕ)(x y z : BitVec(n)).
                bvxor(bvxor(x, y), z) = bvxor(x, bvxor(y, z))
            
            axiom add_identity : ∀(n : ℕ)(x : BitVec(n)).
                bvxor(x, bvzero(n)) = x
            
            // Every element is its own additive inverse
            axiom add_inverse : ∀(n : ℕ)(x : BitVec(n)).
                bvxor(x, x) = bvzero(n)
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse vector space axioms: {}",
        parse_error(input)
    );
}

#[test]
fn test_parse_boolean_algebra_structure() {
    // BitVec(n) as a Boolean algebra
    let input = r#"
        structure BooleanAlgebra {
            // Complement laws
            axiom complement_and : ∀(n : ℕ)(x : BitVec(n)).
                bvand(x, bvnot(x)) = bvzero(n)
            
            axiom complement_or : ∀(n : ℕ)(x : BitVec(n)).
                bvor(x, bvnot(x)) = bvones(n)
            
            // Distributive law
            axiom distribute : ∀(n : ℕ)(x y z : BitVec(n)).
                bvand(x, bvor(y, z)) = bvor(bvand(x, y), bvand(x, z))
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse Boolean algebra axioms: {}",
        parse_error(input)
    );
}

#[test]
fn test_parse_ordered_set_structure() {
    // BitVec(n) as a totally ordered set
    let input = r#"
        structure TotalOrder {
            axiom trichotomy : ∀(n : ℕ)(x y : BitVec(n)).
                bvult(x, y) ∨ x = y ∨ bvult(y, x)
            
            axiom transitivity : ∀(n : ℕ)(x y z : BitVec(n)).
                bvult(x, y) ∧ bvult(y, z) → bvult(x, z)
            
            axiom antisymmetry : ∀(n : ℕ)(x y : BitVec(n)).
                bvule(x, y) ∧ bvule(y, x) → x = y
        }
    "#;
    assert!(
        parses_ok(input),
        "Failed to parse total order axioms: {}",
        parse_error(input)
    );
}
