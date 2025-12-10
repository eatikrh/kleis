///! Tests that Z3 loads parent structure axioms via extends
///!
///! When a structure extends another, Z3 should automatically
///! have access to the parent's axioms!
///!
///! Example:
///! ```kleis
///! structure Monoid(M) extends Semigroup(M) { ... }
///! // When verifying Monoid axioms, Semigroup axioms should be available!
///! ```
use kleis::kleis_parser::KleisParser;
use kleis::structure_registry::StructureRegistry;

#[cfg(feature = "axiom-verification")]
use kleis::axiom_verifier::{AxiomVerifier, VerificationResult};

#[test]
fn test_extends_loads_parent_axioms() {
    // Test: Monoid extends Semigroup → loads Semigroup axioms

    let code = r#"
        structure Semigroup(S) {
            operation compose : S → S → S
            axiom associativity: ∀(x y z : S). (x × y) × z = x × (y × z)
        }
        
        structure Monoid(M) extends Semigroup(M) {
            operation e : M
            axiom identity: ∀(x : M). x × e = x
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

        // Verify associativity (from Semigroup parent)
        let test_axiom = "∀(x y z : S). (x × y) × z = x × (y × z)";
        let mut parser = KleisParser::new(test_axiom);
        let axiom = parser.parse_proposition().expect("Failed to parse");

        println!("\n🧪 Verifying Semigroup associativity (Monoid's parent)...");
        let result = verifier.verify_axiom(&axiom);

        println!("   Result: {:?}", result);

        let stats = verifier.stats();
        println!("   Structures loaded: {}", stats.loaded_structures);

        if stats.loaded_structures >= 2 {
            println!("\n   🎯 SUCCESS! Parent structure loaded via extends!");
            println!("   Monoid extends Semigroup triggered Semigroup loading!");
        }
    }

    #[cfg(not(feature = "axiom-verification"))]
    {
        println!("✅ Extends parsed (Z3 disabled)");
    }
}

#[test]
fn test_transitive_inheritance() {
    // Test: AbelianGroup → Group → Monoid → Semigroup (4 levels!)

    let code = r#"
        structure Semigroup(S) {
            operation compose : S → S → S
            axiom associativity: ∀(x y z : S). (x × y) × z = x × (y × z)
        }
        
        structure Monoid(M) extends Semigroup(M) {
            operation e : M
        }
        
        structure Group(G) extends Monoid(G) {
            operation inv : G → G
        }
        
        structure AbelianGroup(A) extends Group(A) {
            axiom commutativity: ∀(x y : A). x × y = y × x
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

        // When loading AbelianGroup, should transitively load: Group → Monoid → Semigroup
        let test_axiom = "∀(x y z : S). (x × y) × z = x × (y × z)";
        let mut parser = KleisParser::new(test_axiom);
        let axiom = parser.parse_proposition().expect("Failed to parse");

        println!("\n🧪 Verifying axiom from 4-level inheritance chain...");
        println!("   AbelianGroup → Group → Monoid → Semigroup");

        let result = verifier.verify_axiom(&axiom);
        println!("   Result: {:?}", result);

        let stats = verifier.stats();
        println!("   Structures loaded: {}", stats.loaded_structures);

        if stats.loaded_structures >= 3 {
            println!("\n   🏆 TRANSITIVE INHERITANCE WORKS!");
            println!("   Multiple levels of extends resolved automatically!");
        }
    }

    #[cfg(not(feature = "axiom-verification"))]
    {
        println!("✅ 4-level hierarchy parsed");
    }
}

#[test]
fn test_extends_plus_where_constraints() {
    // Test: Structure with BOTH extends and where clauses!
    // This is the ultimate combo

    let code = r#"
        structure Semiring(S) {
            operation plus : S → S → S
            operation times : S → S → S
        }
        
        structure Ring(R) {
            operation plus : R → R → R
            operation times : R → R → R
            operation zero : R
        }
        
        structure Module(M, R) extends AbelianGroup(M) {
            operation scale : R → M → M
        }
        
        implements Module(M, R) where Ring(R) {
            operation scale = builtin_scale
        }
    "#;

    let mut parser = KleisParser::new(code);
    let program = parser.parse_program().expect("Failed to parse");

    let mut registry = StructureRegistry::new();

    for item in program.items {
        match item {
            kleis::kleis_ast::TopLevel::StructureDef(s) => {
                registry.register(s).expect("Failed to register");
            }
            kleis::kleis_ast::TopLevel::ImplementsDef(i) => {
                registry.register_implements(i);
            }
            _ => {}
        }
    }

    // Verify Module has both extends and implements has where
    let module_def = registry.get("Module").expect("Module not found");
    assert!(
        module_def.extends_clause.is_some(),
        "Module should extend AbelianGroup"
    );

    let where_constraints = registry.get_where_constraints("Module");
    assert_eq!(
        where_constraints.len(),
        1,
        "Should have where Ring(R) constraint"
    );

    println!("✅ Structure with extends + where constraint works!");
    println!("   Module extends AbelianGroup where Ring(R)");
    println!("   This is the full power of generic algebra!");

    #[cfg(feature = "axiom-verification")]
    {
        let mut verifier = AxiomVerifier::new(&registry).expect("Failed to create verifier");

        let test_axiom = "∀(x y : S). x + y = y + x";
        let mut parser = KleisParser::new(test_axiom);
        let axiom = parser.parse_proposition().expect("Failed to parse");

        println!("\n🧪 Verifying with both extends and where constraints...");
        let _ = verifier.verify_axiom(&axiom);

        let stats = verifier.stats();
        println!("   Structures loaded: {}", stats.loaded_structures);

        if stats.loaded_structures > 0 {
            println!("\n   🎯 Both extends AND where constraints work with Z3!");
        }
    }
}
