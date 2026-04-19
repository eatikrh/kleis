#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

//! Tests for element namespace collision fix.
//!
//! Verifies that elements with the same name in different structures
//! get independent Z3 constants, preventing axiom cross-contamination.

use kleis::kleis_parser::KleisParser;
use kleis::structure_registry::StructureRegistry;

#[cfg(feature = "axiom-verification")]
use kleis::axiom_verifier::{AxiomVerifier, VerificationResult};

fn build_registry(code: &str) -> StructureRegistry {
    let mut parser = KleisParser::new(code);
    let program = parser.parse_program().expect("Failed to parse");
    let mut registry = StructureRegistry::new();
    for item in program.items {
        if let kleis::kleis_ast::TopLevel::StructureDef(s) = item {
            registry.register(s).expect("Failed to register");
        }
    }
    registry
}

// ============================================================
// TEST 1: Conflicting axioms on same-named elements
// ============================================================

/// Two structures declare `element n : ℝ` with contradictory axioms.
/// Before the fix, these merged into one Z3 constant → UNSAT.
/// After the fix, each structure gets an independent constant.
#[test]
#[cfg(feature = "axiom-verification")]
fn test_conflicting_elements_no_inconsistency() {
    let code = r#"
        structure PositiveBound {
            element n : ℝ
            axiom positive : n >= 5
        }

        structure NegativeBound {
            element n : ℝ
            axiom negative : n <= 3
        }
    "#;

    let registry = build_registry(code);

    let verifier = AxiomVerifier::new(&registry);
    assert!(
        verifier.is_ok(),
        "Verifier creation should succeed: {:?}",
        verifier.err()
    );
    let mut verifier = verifier.unwrap();

    // Verify that PositiveBound's axiom is internally consistent
    let mut parser = KleisParser::new("n >= 5");
    let prop = parser.parse_proposition().expect("parse");

    // The key test: loading axioms from structures with conflicting same-named
    // elements should NOT make the solver inconsistent.
    let result = verifier.verify_axiom(&prop);
    match &result {
        Ok(VerificationResult::InconsistentAxioms) => {
            panic!(
                "REGRESSION: Conflicting same-named elements caused axiom inconsistency. \
                 Structure scoping is not working."
            );
        }
        _ => {
            println!("✅ No axiom inconsistency from conflicting same-named elements");
        }
    }
}

// ============================================================
// TEST 2: Same-named elements with identical axioms (benign)
// ============================================================

/// Two structures declare `element x : ℤ` with identical axioms.
/// This should work regardless of scoping.
#[test]
#[cfg(feature = "axiom-verification")]
fn test_identical_elements_benign() {
    let code = r#"
        structure Config1 {
            element x : ℤ
            axiom ground : x = 1000
        }

        structure Config2 {
            element x : ℤ
            axiom ground : x = 1000
        }
    "#;

    let registry = build_registry(code);
    let verifier = AxiomVerifier::new(&registry);
    assert!(verifier.is_ok(), "Should not cause inconsistency");
    println!("✅ Identical same-named elements work correctly");
}

// ============================================================
// TEST 3: Many structures sharing an element name
// ============================================================

/// Multiple structures all declare `element n : ℝ` with different constraints.
/// Tests that scoping handles more than two colliding structures.
#[test]
#[cfg(feature = "axiom-verification")]
fn test_many_structures_same_element_name() {
    let code = r#"
        structure A {
            element n : ℝ
            axiom a1 : n = 1
        }

        structure B {
            element n : ℝ
            axiom b1 : n = 2
        }

        structure C {
            element n : ℝ
            axiom c1 : n = 3
        }

        structure D {
            element n : ℝ
            axiom d1 : n = 100
        }
    "#;

    let registry = build_registry(code);
    let verifier = AxiomVerifier::new(&registry);
    assert!(
        verifier.is_ok(),
        "Four structures with conflicting 'n' should not cause inconsistency: {:?}",
        verifier.err()
    );
    println!("✅ Four structures with colliding element 'n' coexist");
}

// ============================================================
// TEST 4: Mixed unique and colliding elements
// ============================================================

/// Some elements are unique, some collide. Unique elements should
/// still resolve correctly via the global map.
#[test]
#[cfg(feature = "axiom-verification")]
fn test_mixed_unique_and_colliding_elements() {
    let code = r#"
        structure Geometry {
            element pi_approx : ℝ
            element n : ℝ
            axiom pi_bound : pi_approx > 3 ∧ pi_approx < 4
            axiom geo_n : n = 3
        }

        structure Algebra {
            element e_approx : ℝ
            element n : ℝ
            axiom e_bound : e_approx > 2 ∧ e_approx < 3
            axiom alg_n : n = 7
        }
    "#;

    let registry = build_registry(code);
    let verifier = AxiomVerifier::new(&registry);
    assert!(
        verifier.is_ok(),
        "Mixed unique/colliding elements should work: {:?}",
        verifier.err()
    );
    println!("✅ Mixed unique and colliding elements handled correctly");
}

// ============================================================
// TEST 5: Elements with different types in different structures
// ============================================================

/// Same element name but different types across structures.
#[test]
#[cfg(feature = "axiom-verification")]
fn test_same_name_different_types() {
    let code = r#"
        structure IntContext {
            element x : ℤ
            axiom int_x : x = 42
        }

        structure RealContext {
            element x : ℝ
            axiom real_x : x > 3.14 ∧ x < 3.15
        }
    "#;

    let registry = build_registry(code);
    let verifier = AxiomVerifier::new(&registry);
    assert!(
        verifier.is_ok(),
        "Same name, different types should not collide: {:?}",
        verifier.err()
    );
    println!("✅ Same-named elements with different types handled correctly");
}

// ============================================================
// TEST 6: Structure with unique elements still works
// ============================================================

/// Regression test: structures with no collisions should work exactly
/// as before the namespace change.
#[test]
#[cfg(feature = "axiom-verification")]
fn test_unique_elements_regression() {
    let code = r#"
        structure Bounds {
            element lower : ℝ
            element upper : ℝ
            axiom ordering : lower < upper
            axiom lower_positive : lower > 0
        }
    "#;

    let registry = build_registry(code);
    let mut verifier = AxiomVerifier::new(&registry).expect("verifier");

    // Verify a consequence of the axioms
    let mut parser = KleisParser::new("upper > 0");
    let prop = parser.parse_proposition().expect("parse");
    let result = verifier.verify_axiom(&prop);

    match result {
        Ok(VerificationResult::Valid) => {
            println!("✅ Unique elements: Z3 correctly derives upper > 0 from axioms");
        }
        other => {
            panic!(
                "Expected Valid for upper > 0 (consequence of lower > 0 ∧ lower < upper), got: {:?}",
                other
            );
        }
    }
}

// ============================================================
// TEST 7: Collision warning is emitted (captured via stderr)
// ============================================================

/// Verify that a warning is printed when elements collide.
/// We can't easily capture stderr in Rust tests, so this test just
/// ensures the code path executes without panicking.
#[test]
#[cfg(feature = "axiom-verification")]
fn test_collision_warning_does_not_panic() {
    let code = r#"
        structure First {
            element val : ℝ
            axiom f1 : val = 10
        }

        structure Second {
            element val : ℝ
            axiom s1 : val = 20
        }
    "#;

    let registry = build_registry(code);
    // This should emit a warning to stderr but not panic
    let verifier = AxiomVerifier::new(&registry);
    assert!(verifier.is_ok());
    println!("✅ Collision warning path executes without panic");
}

// ============================================================
// TEST 8: Schanuel-style pattern (many prefixed elements)
// ============================================================

/// Regression test modeled after the schanuel_conjecture.kleis pattern:
/// structures with carefully prefixed elements should continue to work.
#[test]
#[cfg(feature = "axiom-verification")]
fn test_prefixed_elements_still_work() {
    let code = r#"
        structure FrameworkA {
            element fa_n : ℝ
            element fa_rank : ℝ
            axiom fa_positive : fa_n >= 1
            axiom fa_rank_bound : fa_rank <= 2 * fa_n
        }

        structure FrameworkB {
            element fb_n : ℝ
            element fb_rank : ℝ
            axiom fb_positive : fb_n >= 1
            axiom fb_rank_bound : fb_rank <= 2 * fb_n
        }
    "#;

    let registry = build_registry(code);
    let verifier = AxiomVerifier::new(&registry);
    assert!(
        verifier.is_ok(),
        "Prefixed elements should work: {:?}",
        verifier.err()
    );
    println!("✅ Prefixed elements (schanuel-style) work correctly");
}
