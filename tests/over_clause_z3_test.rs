///! Test that over clauses are properly connected to Z3
///!
///! When a structure uses "over Field(F)", the field axioms should be
///! available as background assumptions when verifying vector space axioms.

#[cfg(feature = "axiom-verification")]
#[test]
fn test_over_clause_loads_field_axioms() {
    use kleis::axiom_verifier::AxiomVerifier;
    use kleis::kleis_parser::parse_kleis_program;
    use kleis::structure_registry::StructureRegistry;

    // Define Field with axiom
    let code = r#"
        structure Field(F) {
            operation (×) : F → F → F
            element one : F
            
            axiom multiplicative_identity:
                ∀(x : F). one × x = x
        }
        
        structure VectorSpace(V) over Field(F) {
            operation (·) : F × V → V
            
            axiom scalar_identity:
                ∀(v : V). one · v = v
        }
    "#;

    println!("\n🔍 Testing that over clause loads field axioms...\n");

    let program = parse_kleis_program(code).expect("Failed to parse");
    let mut registry = StructureRegistry::new();

    // Register structures
    for item in program.items {
        if let kleis::kleis_ast::TopLevel::StructureDef(structure) = item {
            println!("   Registering structure: {}", structure.name);

            // Check if VectorSpace has over clause
            if structure.name == "VectorSpace" {
                assert!(
                    structure.over_clause.is_some(),
                    "VectorSpace should have over clause"
                );
                println!("   ✅ VectorSpace has over clause");
            }

            registry
                .register(structure)
                .expect("Failed to register structure");
        }
    }

    // Create verifier
    let mut verifier = AxiomVerifier::new(&registry).expect("Failed to create verifier");

    // When we load VectorSpace, it should also load Field
    println!("\n   Loading VectorSpace...");

    // Get VectorSpace axioms
    let vs_axioms = registry.get_axioms("VectorSpace");
    assert!(!vs_axioms.is_empty(), "Should have VectorSpace axioms");

    println!("   Found {} VectorSpace axiom(s)", vs_axioms.len());

    // Verify a VectorSpace axiom - this should trigger loading Field
    let (axiom_name, axiom_expr) = &vs_axioms[0];
    println!("   Verifying axiom: {}", axiom_name);

    let result = verifier.verify_axiom(axiom_expr);

    // Check stats - both structures should be loaded
    let stats = verifier.stats();
    println!("\n   📊 Verifier stats:");
    println!("      Loaded structures: {}", stats.loaded_structures);

    // Should have loaded both VectorSpace AND Field (due to over clause)
    assert!(
        stats.loaded_structures >= 2,
        "Should have loaded both VectorSpace and Field, got {}",
        stats.loaded_structures
    );

    println!("\n✅ Over clause successfully loaded Field axioms!");
    println!("   This means Z3 has field properties available when verifying vector space axioms");

    assert!(result.is_ok(), "Verification should not error");
}

#[cfg(not(feature = "axiom-verification"))]
#[test]
fn test_over_clause_parsing_only() {
    use kleis::kleis_parser::parse_kleis_program;

    let code = r#"
        structure VectorSpace(V) over Field(F) {
            operation (·) : F → V → V
        }
    "#;

    let program = parse_kleis_program(code).expect("Should parse");

    // Just verify it parses - Z3 test requires feature flag
    println!("✅ Over clause parses (Z3 test requires axiom-verification feature)");
}
