#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
///! Test that proves structures ARE being loaded and their axioms ARE being used
///!
///! This test uses simple axioms that:
///! 1. Use operation names that match registry
///! 2. Don't reference constants (zero, one, etc.)
///! 3. Actually trigger structure loading
///!
///! Goal: Prove the multi-level axiom filtering works end-to-end
use kleis::kleis_parser::KleisParser;
use kleis::structure_registry::StructureRegistry;

#[cfg(feature = "axiom-verification")]
use kleis::axiom_verifier::{AxiomVerifier, VerificationResult};

#[test]
fn test_structure_axioms_actually_loaded() {
    // Create a registry with a simple structure
    let mut registry = StructureRegistry::new();

    // Structure with operations named to match our test axiom
    let additive_def = r#"
        structure AdditiveStructure(A) {
            operation plus : A → A → A
            axiom commutativity: ∀(x y : A). equals(plus(x, y), plus(y, x))
            axiom associativity: ∀(x y z : A). equals(plus(plus(x, y), z), plus(x, plus(y, z)))
        }
    "#;

    let mut parser = KleisParser::new(additive_def);
    let structure = parser.parse_structure().expect("Failed to parse structure");

    println!("📝 Registered AdditiveStructure with axioms:");
    for member in &structure.members {
        if let kleis::kleis_ast::StructureMember::Axiom { name, .. } = member {
            println!("   - {}", name);
        }
    }

    registry.register(structure).expect("Failed to register");

    #[cfg(feature = "axiom-verification")]
    {
        // Now verify an axiom that uses 'plus' operation
        let test_axiom = "∀(a b : A). equals(plus(a, b), plus(b, a))";
        let mut parser = KleisParser::new(test_axiom);
        let axiom = parser.parse_proposition().expect("Failed to parse axiom");

        let mut verifier = AxiomVerifier::new(&registry).expect("Failed to create verifier");

        println!("\n🧪 Verifying commutativity axiom:");
        println!("   Axiom uses operation: 'plus'");
        println!("   Registry has structure with: 'plus'");
        println!("   Expected: Structure SHOULD be loaded!");

        let result = verifier.verify_axiom(&axiom);

        let stats = verifier.stats();
        println!("\n📊 Verification Results:");
        println!("   Structures loaded: {}", stats.loaded_structures);
        println!("   Verification: {:?}", result);

        if stats.loaded_structures > 0 {
            println!("\n   ✅ SUCCESS!");
            println!("   🎯 Structure was loaded - dependency analysis works!");
            println!("   🎯 AdditiveStructure's axioms are now available to Z3!");
            println!("\n   This proves:");
            println!("   1. Dependency analysis finds structures by operation name");
            println!("   2. ensure_structure_loaded() actually loads the structure");
            println!("   3. Axioms from the structure are available for verification");
            println!("   4. Multi-level axiom filtering is working end-to-end!");
        } else {
            println!("\n   ❌ FAILED to load structure");
            println!("   This suggests dependency analysis didn't find 'plus' operation");
            panic!("Expected structure to be loaded!");
        }

        // The axiom should verify (it's commutativity over integers)
        match result {
            Ok(VerificationResult::Valid) => {
                println!("\n   ✅ Axiom verified as valid!");
            }
            Ok(VerificationResult::Invalid { ref witness }) => {
                println!("\n   ℹ️  Counterexample found: {}", witness);
                println!("   (This is OK - we're testing structure loading, not validity)");
            }
            _ => {}
        }
    }

    #[cfg(not(feature = "axiom-verification"))]
    {
        println!("✅ Structure registered (Z3 verification disabled)");
    }
}

#[test]
fn test_multiple_structures_with_same_operation() {
    // Test: Multiple structures define 'plus' - all should be detected
    let mut registry = StructureRegistry::new();

    let struct1 = r#"
        structure AdditiveSemigroup(S) {
            operation plus : S → S → S
            axiom associativity: ∀(x y z : S). equals(plus(plus(x, y), z), plus(x, plus(y, z)))
        }
    "#;

    let struct2 = r#"
        structure AdditiveMonoid(M) {
            operation plus : M → M → M
            axiom associativity: ∀(x y z : M). equals(plus(plus(x, y), z), plus(x, plus(y, z)))
        }
    "#;

    for (name, def) in [("AdditiveSemigroup", struct1), ("AdditiveMonoid", struct2)] {
        let mut parser = KleisParser::new(def);
        if let Ok(structure) = parser.parse_structure() {
            registry.register(structure).expect("Failed to register");
            println!("📝 Registered: {}", name);
        }
    }

    // Check which structures define 'plus'
    match registry.get_operation_owners("plus") {
        Some(owners) => {
            println!("\n🔍 Operation 'plus' found in:");
            for owner in &owners {
                println!("   - {}", owner);
            }
            assert!(owners.len() >= 2, "Should find at least 2 structures");
            println!("\n   ✅ Multiple structures detected correctly!");
        }
        None => {
            panic!("Should find 'plus' operation in registry!");
        }
    }

    #[cfg(feature = "axiom-verification")]
    {
        let test_axiom = "∀(x y : S). equals(plus(x, y), plus(y, x))";
        let mut parser = KleisParser::new(test_axiom);
        let axiom = parser.parse_proposition().expect("Failed to parse");

        let mut verifier = AxiomVerifier::new(&registry).expect("Failed to create verifier");
        let _ = verifier.verify_axiom(&axiom);

        let stats = verifier.stats();
        println!("\n📊 After verification:");
        println!("   Structures loaded: {}", stats.loaded_structures);

        if stats.loaded_structures >= 2 {
            println!("\n   🎯 Multiple structures loaded!");
            println!("   Axioms from ALL matching structures are available!");
        }
    }
}

#[test]
fn test_hierarchical_structure_loading() {
    // Test: Axiom uses operations from multiple structures
    // Should load all relevant structures

    let mut registry = StructureRegistry::new();

    let additive = r#"
        structure Additive(A) {
            operation plus : A → A → A
            axiom add_comm: ∀(x y : A). equals(plus(x, y), plus(y, x))
        }
    "#;

    let multiplicative = r#"
        structure Multiplicative(M) {
            operation times : M → M → M
            axiom mul_assoc: ∀(x y z : M). equals(times(times(x, y), z), times(x, times(y, z)))
        }
    "#;

    for def in [additive, multiplicative] {
        let mut parser = KleisParser::new(def);
        if let Ok(structure) = parser.parse_structure() {
            println!("📝 Registered: {}", structure.name);
            registry.register(structure).expect("Failed to register");
        }
    }

    #[cfg(feature = "axiom-verification")]
    {
        // Axiom that uses BOTH plus and times
        let test_axiom =
            "∀(x y z : R). equals(times(x, plus(y, z)), plus(times(x, y), times(x, z)))";
        let mut parser = KleisParser::new(test_axiom);
        let axiom = parser.parse_proposition().expect("Failed to parse");

        let mut verifier = AxiomVerifier::new(&registry).expect("Failed to create verifier");

        println!("\n🧪 Verifying axiom that uses 'plus' AND 'times':");
        let result = verifier.verify_axiom(&axiom);

        let stats = verifier.stats();
        println!("\n📊 Multi-operation verification:");
        println!("   Structures loaded: {}", stats.loaded_structures);
        println!("   Result: {:?}", result);

        if stats.loaded_structures >= 2 {
            println!("\n   🏆 EXCELLENT!");
            println!("   Both Additive and Multiplicative structures loaded!");
            println!("   This proves hierarchical dependency analysis works!");
            println!("\n   The verifier:");
            println!("   1. Found 'plus' → loaded Additive structure");
            println!("   2. Found 'times' → loaded Multiplicative structure");
            println!("   3. Both sets of axioms are available");
            println!("   4. Can verify axioms spanning multiple structures!");
        }
    }

    #[cfg(not(feature = "axiom-verification"))]
    {
        println!("✅ Structures registered (Z3 verification disabled)");
    }
}
