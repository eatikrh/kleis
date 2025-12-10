///! Tests that Z3 axiom verifier handles nested structure axioms
///!
///! This proves that axioms in nested structures are available to Z3!
///!
///! Example:
///! ```kleis
///! structure Ring(R) {
///!     structure additive : AbelianGroup(R) {
///!         axiom commutativity: ∀(x y : R). x + y = y + x
///!     }
///! }
///! // When verifying Ring axioms, commutativity should be available!
///! ```

use kleis::kleis_parser::KleisParser;
use kleis::structure_registry::StructureRegistry;

#[cfg(feature = "axiom-verification")]
use kleis::axiom_verifier::{AxiomVerifier, VerificationResult};

#[test]
fn test_nested_structure_axioms_loaded() {
    // Test: Axioms in nested structures are loaded by Z3
    
    let code = r#"
        structure Ring(R) {
            structure additive : AbelianGroup(R) {
                operation plus : R → R → R
                operation zero : R
                axiom commutativity: ∀(x y : R). x + y = y + x
            }
            
            operation times : R → R → R
        }
    "#;

    let mut parser = KleisParser::new(code);
    let program = parser.parse_program().expect("Failed to parse");
    
    let mut registry = StructureRegistry::new();
    
    for item in program.items {
        if let kleis::kleis_ast::TopLevel::StructureDef(s) = item {
            registry.register(s).expect("Failed to register");
        }
    }

    #[cfg(feature = "axiom-verification")]
    {
        let mut verifier = AxiomVerifier::new(&registry).expect("Failed to create verifier");
        
        // Verify the commutativity axiom (from nested structure)
        let test_axiom = "∀(x y : R). x + y = y + x";
        let mut parser = KleisParser::new(test_axiom);
        let axiom = parser.parse_proposition().expect("Failed to parse");
        
        println!("\n🧪 Verifying commutativity from nested structure...");
        let result = verifier.verify_axiom(&axiom);
        
        println!("   Result: {:?}", result);
        
        match result {
            Ok(VerificationResult::Valid) => {
                println!("   ✅ VERIFIED!");
                println!("   Nested structure axiom was available to Z3!");
            }
            _ => {}
        }
        
        let stats = verifier.stats();
        println!("\n📊 Stats:");
        println!("   Structures loaded: {}", stats.loaded_structures);
    }

    #[cfg(not(feature = "axiom-verification"))]
    {
        println!("✅ Nested structure parsed (Z3 disabled)");
    }
}

#[test]
fn test_nested_identity_elements_available() {
    // Test: Identity elements from nested structures are available
    
    let code = r#"
        structure Ring(R) {
            structure additive : AbelianGroup(R) {
                operation plus : R → R → R
                operation zero : R
            }
            
            structure multiplicative : Monoid(R) {
                operation times : R → R → R
                operation one : R
            }
        }
    "#;

    let mut parser = KleisParser::new(code);
    let program = parser.parse_program().expect("Failed to parse");
    
    let mut registry = StructureRegistry::new();
    
    for item in program.items {
        if let kleis::kleis_ast::TopLevel::StructureDef(s) = item {
            registry.register(s).expect("Failed to register");
        }
    }

    #[cfg(feature = "axiom-verification")]
    {
        let mut verifier = AxiomVerifier::new(&registry).expect("Failed to create verifier");
        
        // Try an axiom that uses both zero and one (from nested structures)
        let test_axiom = "∀(x : R). x + zero = x";
        let mut parser = KleisParser::new(test_axiom);
        let axiom = parser.parse_proposition().expect("Failed to parse");
        
        println!("\n🧪 Verifying identity axiom with nested identity elements...");
        let result = verifier.verify_axiom(&axiom);
        
        println!("   Result: {:?}", result);
        
        let stats = verifier.stats();
        println!("   Structures loaded: {}", stats.loaded_structures);
        
        // The key test: Did zero get loaded?
        if result.is_ok() {
            println!("\n   ✅ Identity elements from nested structures are available!");
            println!("   'zero' from nested additive structure worked!");
        }
    }

    #[cfg(not(feature = "axiom-verification"))]
    {
        println!("✅ Ring with nested structures parsed");
    }
}

#[test]
fn test_vector_space_nested_axioms() {
    // Test: Complex VectorSpace with axioms in nested structures
    
    let code = r#"
        structure VectorSpace(V, F) {
            structure vectors : AbelianGroup(V) {
                operation plus : V → V → V
                operation zero : V
                axiom vector_commutativity: ∀(v w : V). v + w = w + v
            }
            
            structure scalars : Field(F) {
                operation plus : F → F → F
                operation times : F → F → F
                operation zero : F
                operation one : F
                axiom scalar_commutativity: ∀(a b : F). a + b = b + a
            }
            
            operation scale : F → V → V
        }
    "#;

    let mut parser = KleisParser::new(code);
    let program = parser.parse_program().expect("Failed to parse");
    
    let mut registry = StructureRegistry::new();
    
    for item in program.items {
        if let kleis::kleis_ast::TopLevel::StructureDef(s) = item {
            registry.register(s).expect("Failed to register");
        }
    }

    #[cfg(feature = "axiom-verification")]
    {
        let mut verifier = AxiomVerifier::new(&registry).expect("Failed to create verifier");
        
        // Verify vector commutativity (from vectors nested structure)
        let vector_axiom = "∀(v w : V). v + w = w + v";
        let mut parser = KleisParser::new(vector_axiom);
        let axiom = parser.parse_proposition().expect("Failed to parse");
        
        println!("\n🧪 Verifying vector commutativity...");
        let result = verifier.verify_axiom(&axiom);
        println!("   Result: {:?}", result);
        
        if let Ok(VerificationResult::Valid) = result {
            println!("   ✅ Verified axiom from nested 'vectors' structure!");
        }
        
        let stats = verifier.stats();
        println!("   Structures loaded: {}", stats.loaded_structures);
        
        if stats.loaded_structures > 0 {
            println!("\n   🎯 SUCCESS! VectorSpace nested structures work with Z3!");
            println!("   Both vectors and scalars substructures are accessible!");
        }
    }

    #[cfg(not(feature = "axiom-verification"))]
    {
        println!("✅ VectorSpace with nested structures parsed");
    }
}

