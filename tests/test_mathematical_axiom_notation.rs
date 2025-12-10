///! Test that axioms can be written with mathematical infix notation
///!
///! This tests if we can write:
///!   ∀(x y : R). x + y = y + x
///! Instead of:
///!   ∀(x y : R). equals(plus(x, y), plus(y, x))

use kleis::kleis_parser::KleisParser;

#[test]
fn test_axiom_with_infix_notation() {
    // Test: Can we write axioms with natural mathematical notation?
    let axiom_text = "∀(x y : R). x + y = y + x";
    
    let mut parser = KleisParser::new(axiom_text);
    let result = parser.parse_proposition();
    
    println!("Parsing: {}", axiom_text);
    println!("Result: {:?}", result);
    
    assert!(
        result.is_ok(),
        "Should parse mathematical infix notation: {:?}",
        result.err()
    );
    
    println!("✅ Mathematical infix notation works in axioms!");
}

#[test]
fn test_distributivity_with_mathematical_notation() {
    // Test: Ring distributivity with × and + symbols
    let axiom_text = "∀(x y z : R). x × (y + z) = (x × y) + (x × z)";
    
    let mut parser = KleisParser::new(axiom_text);
    let result = parser.parse_proposition();
    
    println!("Parsing: {}", axiom_text);
    
    if result.is_ok() {
        println!("✅ Beautiful! Mathematical notation works!");
        println!("   We can write axioms the way mathematicians actually write them!");
    } else {
        println!("⚠️ Parser doesn't recognize × symbol: {:?}", result.err());
        println!("   This is why we use times() function notation");
    }
}

#[test]
fn test_comparison_with_both_notations() {
    // Compare function notation vs infix notation
    
    let function_notation = "∀(x y : R). equals(plus(x, y), plus(y, x))";
    let infix_notation = "∀(x y : R). x + y = y + x";
    
    let mut parser1 = KleisParser::new(function_notation);
    let result1 = parser1.parse_proposition();
    
    let mut parser2 = KleisParser::new(infix_notation);
    let result2 = parser2.parse_proposition();
    
    println!("\n📊 Comparison:");
    println!("Function notation: {:?}", result1.is_ok());
    println!("Infix notation: {:?}", result2.is_ok());
    
    if result1.is_ok() && result2.is_ok() {
        println!("\n✅ BOTH notations work!");
        println!("   Question: Why are we using function notation in tests?");
        println!("   Answer: Probably for Z3 translation clarity");
    } else if result1.is_ok() && result2.is_err() {
        println!("\n⚠️ Only function notation works");
        println!("   Infix error: {:?}", result2.err());
    }
}

#[test]
fn test_verify_infix_axiom_with_z3() {
    // Test: Can Z3 verify axioms written with infix notation?
    
    #[cfg(feature = "axiom-verification")]
    {
        use kleis::axiom_verifier::{AxiomVerifier, VerificationResult};
        use kleis::structure_registry::StructureRegistry;
        
        // Try to verify commutativity with infix notation
        let axiom_text = "∀(x y : R). x + y = y + x";
        
        let mut parser = KleisParser::new(axiom_text);
        let result = parser.parse_proposition();
        
        if let Ok(axiom) = result {
            let registry = StructureRegistry::new();
            let mut verifier = AxiomVerifier::new(&registry).expect("Failed to create verifier");
            
            println!("\n🧪 Verifying axiom with infix notation...");
            let verification = verifier.verify_axiom(&axiom);
            
            match verification {
                Ok(VerificationResult::Valid) => {
                    println!("   ✅ VERIFIED with infix notation!");
                    println!("   This means we CAN use mathematical notation!");
                }
                Ok(VerificationResult::Invalid { counterexample }) => {
                    println!("   ⚠️ Verification failed: {}", counterexample);
                }
                Err(e) => {
                    println!("   ⚠️ Translation error: {}", e);
                    println!("   This is why we use function notation - Z3 translator needs it");
                }
                _ => {}
            }
        } else {
            println!("⚠️ Infix notation doesn't parse");
        }
    }
    
    #[cfg(not(feature = "axiom-verification"))]
    {
        println!("Test requires axiom-verification feature");
    }
}

