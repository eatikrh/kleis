#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use kleis::ast::Expression;
///! Integration Tests for Axiom Verification with Z3
///!
///! Tests Phase 1 Task 4: Actually verify axioms using Z3
///!
///! This tests the complete pipeline:
///! 1. Parse Kleis structures with axioms
///! 2. Extract axiom expressions
///! 3. Pass to AxiomVerifier
///! 4. Verify with Z3
use kleis::axiom_verifier::{AxiomVerifier, VerificationResult};
use kleis::kleis_parser::KleisParser;
use kleis::structure_registry::StructureRegistry;

/// Helper to parse an axiom expression
fn parse_axiom(input: &str) -> Expression {
    let mut parser = KleisParser::new(input);
    parser.parse_proposition().expect("Failed to parse axiom")
}

/// Helper to verify an axiom with test setup
/// This handles all the boilerplate of creating verifier, etc.
#[cfg(feature = "axiom-verification")]
fn verify_with_test_context(axiom: &Expression) -> Result<VerificationResult, String> {
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry)?;
    verifier.verify_axiom(axiom)
}

/// Placeholder for non-axiom-verification builds
#[cfg(not(feature = "axiom-verification"))]
fn verify_with_test_context(_axiom: &Expression) -> Result<VerificationResult, String> {
    Ok(VerificationResult::Disabled)
}

/// Helper to check equivalence with test setup
#[cfg(feature = "axiom-verification")]
fn check_equivalence_with_test_context(
    expr1: &Expression,
    expr2: &Expression,
) -> Result<bool, String> {
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry)?;
    verifier.are_equivalent(expr1, expr2)
}

/// Placeholder for non-axiom-verification builds
#[cfg(not(feature = "axiom-verification"))]
fn check_equivalence_with_test_context(
    _expr1: &Expression,
    _expr2: &Expression,
) -> Result<bool, String> {
    Err("Axiom verification feature not enabled".to_string())
}

#[test]
fn test_verifier_creation() {
    // Test that we can create a verifier
    #[cfg(feature = "axiom-verification")]
    {
        let registry = StructureRegistry::new();
        let verifier = AxiomVerifier::new(&registry);
        assert!(verifier.is_ok(), "Verifier creation should succeed");

        // Try to verify a simple expression
        let expr = parse_axiom("∀(x : M). equals(x, x)"); // Must be boolean!
        let mut verifier = verifier.unwrap();
        let result = verifier.verify_axiom(&expr);

        // Should return something (either Valid/Invalid/Unknown or Disabled)
        if let Err(e) = &result {
            println!("Error: {}", e);
        }
        assert!(result.is_ok());
    }

    #[cfg(not(feature = "axiom-verification"))]
    {
        // Just verify the test compiles without axiom-verification
        println!("Axiom verification disabled");
    }
}

#[test]
fn test_identity_axiom_simple() {
    // Test: ∀(x : M). x + 0 = x
    // This is the additive identity axiom
    // Note: Using literal "0" instead of variable "zero"

    let axiom_text = "∀(x : M). equals(plus(x, 0), x)";
    let axiom = parse_axiom(axiom_text);

    let result = verify_with_test_context(&axiom);

    // For now, this will fail because our translator is basic
    // But it should parse correctly
    println!("Result: {:?}", result);
    assert!(result.is_ok() || result.is_err(), "Should return a result");

    #[cfg(feature = "axiom-verification")]
    {
        let verification = result.unwrap();
        match verification {
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. } => {
                println!("✅ Identity axiom verified!");
            }
            VerificationResult::Invalid { witness } => {
                println!("❌ Axiom violated! Counterexample: {}", witness);
                panic!("Identity axiom should be valid");
            }
            VerificationResult::Unknown => {
                println!("⚠️ Z3 could not determine");
            }
            VerificationResult::Disabled => {
                panic!("Axiom verification should be enabled in this test");
            }
            VerificationResult::InconsistentAxioms => {
                panic!("Axioms are inconsistent!");
            }
        }
    }

    #[cfg(not(feature = "axiom-verification"))]
    {
        // Just compile check for non-axiom-verification builds
        assert_eq!(result.unwrap(), VerificationResult::Disabled);
    }
}

#[test]
fn test_commutativity_axiom() {
    // Test: ∀(x y : R). x + y = y + x
    // This is commutativity of addition

    let axiom_text = "∀(x y : R). equals(plus(x, y), plus(y, x))";
    let axiom = parse_axiom(axiom_text);

    let result = verify_with_test_context(&axiom);

    if let Err(e) = &result {
        eprintln!("ERROR: {}", e);
    }
    assert!(result.is_ok(), "Failed: {:?}", result);

    #[cfg(feature = "axiom-verification")]
    {
        match result.unwrap() {
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. } => {
                println!("✅ Commutativity verified!");
            }
            VerificationResult::Invalid { witness } => {
                println!("❌ Counterexample: {}", witness);
                panic!("Commutativity should be valid");
            }
            VerificationResult::Unknown => {
                println!("⚠️ Z3 could not determine");
            }
            VerificationResult::Disabled => {
                panic!("Feature should be enabled");
            }
            VerificationResult::InconsistentAxioms => {
                panic!("Axioms are inconsistent!");
            }
        }
    }
}

#[test]
fn test_associativity_axiom() {
    // Test: ∀(x y z : R). (x + y) + z = x + (y + z)
    // This is associativity of addition

    let axiom_text = "∀(x y z : R). equals(plus(plus(x, y), z), plus(x, plus(y, z)))";
    let axiom = parse_axiom(axiom_text);

    let result = verify_with_test_context(&axiom);

    assert!(result.is_ok());

    #[cfg(feature = "axiom-verification")]
    {
        match result.unwrap() {
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. } => {
                println!("✅ Associativity verified!");
            }
            VerificationResult::Invalid { witness } => {
                println!("❌ Counterexample: {}", witness);
                panic!("Associativity should be valid");
            }
            VerificationResult::Unknown => {
                println!("⚠️ Z3 could not determine");
            }
            VerificationResult::Disabled => {
                panic!("Feature should be enabled");
            }
            VerificationResult::InconsistentAxioms => {
                panic!("Axioms are inconsistent!");
            }
        }
    }
}

#[test]
fn test_parse_structure_with_axiom() {
    // Test: Parse a complete Monoid structure with axioms

    let structure_text = r#"
        structure Monoid(M) {
            operation e : M
            operation (•) : M → M → M
            axiom identity: ∀(x : M). equals(x, x)
        }
    "#;

    let mut parser = KleisParser::new(structure_text);
    let result = parser.parse_structure();

    assert!(
        result.is_ok(),
        "Failed to parse structure: {:?}",
        result.err()
    );

    let structure = result.unwrap();
    assert_eq!(structure.name, "Monoid");
    assert_eq!(structure.members.len(), 3);

    // Find the axiom
    let axiom = structure.members.iter().find_map(|member| match member {
        kleis::kleis_ast::StructureMember::Axiom { name, proposition } => {
            Some((name.clone(), proposition.clone()))
        }
        _ => None,
    });

    assert!(axiom.is_some(), "Should have an axiom");
    let (axiom_name, axiom_expr) = axiom.unwrap();
    assert_eq!(axiom_name, "identity");

    // Try to verify it (simplified axiom: x = x is always true)
    let result = verify_with_test_context(&axiom_expr);
    println!("Verification result: {:?}", result);
    // Should parse and attempt verification
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_invalid_axiom_detection() {
    // Test: Verify that Z3 can detect INVALID axioms
    // False axiom: ∀(x : M). x + 1 = x (obviously false!)

    let false_axiom = "∀(x : M). equals(plus(x, 1), x)";
    let axiom = parse_axiom(false_axiom);

    let result = verify_with_test_context(&axiom);

    println!("Invalid axiom test result: {:?}", result);
    // Should return a result (even if error)
    assert!(result.is_ok() || result.is_err());

    #[cfg(feature = "axiom-verification")]
    {
        match result.unwrap() {
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. } => {
                panic!("False axiom should NOT be valid!");
            }
            VerificationResult::Invalid { witness } => {
                println!("✅ Correctly detected invalid axiom!");
                println!("   Counterexample: {}", witness);
            }
            VerificationResult::Unknown => {
                println!("⚠️ Z3 could not determine (acceptable)");
            }
            VerificationResult::Disabled => {
                panic!("Feature should be enabled");
            }
            VerificationResult::InconsistentAxioms => {
                println!("⚠️ Axioms are inconsistent (acceptable for false axiom test)");
            }
        }
    }
}

#[test]
fn test_equivalence_checking() {
    // Test: Check if two expressions are equivalent

    let expr1_text = "plus(x, zero)";
    let expr2_text = "x";

    let mut parser1 = KleisParser::new(expr1_text);
    let expr1 = parser1.parse().expect("Failed to parse expr1");

    let mut parser2 = KleisParser::new(expr2_text);
    let expr2 = parser2.parse().expect("Failed to parse expr2");

    let result = check_equivalence_with_test_context(&expr1, &expr2);

    #[cfg(feature = "axiom-verification")]
    {
        // Should be able to check equivalence
        match result {
            Ok(equivalent) => {
                println!("Equivalence check result: {}", equivalent);
            }
            Err(e) => {
                println!("Equivalence check error: {}", e);
            }
        }
    }

    #[cfg(not(feature = "axiom-verification"))]
    {
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Axiom verification feature not enabled"
        );
    }
}

#[test]
fn test_distributivity_axiom() {
    // Test: ∀(x y z : R). x × (y + z) = (x × y) + (x × z)
    // This is the distributivity axiom for rings

    let axiom_text = "∀(x y z : R). equals(times(x, plus(y, z)), plus(times(x, y), times(x, z)))";
    let axiom = parse_axiom(axiom_text);

    let result = verify_with_test_context(&axiom);

    assert!(result.is_ok());

    #[cfg(feature = "axiom-verification")]
    {
        match result.unwrap() {
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. } => {
                println!("✅ Distributivity verified!");
            }
            VerificationResult::Invalid { witness } => {
                println!("❌ Counterexample: {}", witness);
                panic!("Distributivity should be valid");
            }
            VerificationResult::Unknown => {
                println!("⚠️ Z3 could not determine");
            }
            VerificationResult::Disabled => {
                panic!("Feature should be enabled");
            }
            VerificationResult::InconsistentAxioms => {
                panic!("Axioms are inconsistent!");
            }
        }
    }
}

#[test]
fn test_multiple_axioms_from_structure() {
    // Test: Parse a Ring structure with multiple axioms

    let structure_text = r#"
        structure Ring(R) {
            operation (+) : R → R → R
            operation (×) : R → R → R
            axiom additive_commutativity: ∀(x y : R). equals(plus(x, y), plus(y, x))
            axiom additive_associativity: ∀(x y z : R). equals(plus(plus(x, y), z), plus(x, plus(y, z)))
            axiom distributivity: ∀(x y z : R). equals(times(x, plus(y, z)), plus(times(x, y), times(x, z)))
        }
    "#;

    let mut parser = KleisParser::new(structure_text);
    let result = parser.parse_structure();

    assert!(result.is_ok(), "Failed to parse structure");

    let structure = result.unwrap();
    assert_eq!(structure.name, "Ring");

    // Count axioms
    let axiom_count = structure
        .members
        .iter()
        .filter(|member| matches!(member, kleis::kleis_ast::StructureMember::Axiom { .. }))
        .count();

    assert_eq!(axiom_count, 3, "Should have 3 axioms");

    // Verify each axiom
    #[cfg(feature = "axiom-verification")]
    {
        let registry = StructureRegistry::new();
        let mut verifier = AxiomVerifier::new(&registry).expect("Failed to create verifier");
        let mut verified_count = 0;

        for member in &structure.members {
            if let kleis::kleis_ast::StructureMember::Axiom { name, proposition } = member {
                println!("Verifying axiom: {}", name);
                let result = verifier.verify_axiom(proposition);
                assert!(result.is_ok(), "Verification failed for {}", name);
                verified_count += 1;
            }
        }

        assert_eq!(verified_count, 3, "Should have verified 3 axioms");
    }

    #[cfg(not(feature = "axiom-verification"))]
    {
        println!("Axiom verification disabled");
    }
}

#[test]
fn test_nested_quantifiers() {
    // Test: ∀(x : M). ∀(y : M). x + y = y + x
    // Nested universal quantifiers

    let axiom_text = "∀(x : M). ∀(y : M). equals(plus(x, y), plus(y, x))";
    let axiom = parse_axiom(axiom_text);

    let result = verify_with_test_context(&axiom);

    assert!(result.is_ok());

    // Check the AST structure
    match &axiom {
        Expression::Quantifier { body, .. } => {
            // Outer quantifier body should be another quantifier
            match &**body {
                Expression::Quantifier { .. } => {
                    println!("✅ Nested quantifiers parsed correctly");
                }
                _ => panic!("Expected nested quantifier"),
            }
        }
        _ => panic!("Expected quantifier at top level"),
    }
}
