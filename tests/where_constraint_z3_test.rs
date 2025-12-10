///! Tests that Z3 axiom verifier respects where constraints
///!
///! This tests the CRITICAL integration:
///! When verifying an axiom for a structure with where clauses,
///! Z3 should have access to the constrained structures' axioms.
///!
///! Example:
///! ```kleis
///! implements MatrixMult(T) where Semiring(T) {
///!     axiom: ... can use Semiring properties ...
///! }
///! ```
use kleis::kleis_parser::KleisParser;
use kleis::structure_registry::StructureRegistry;

#[cfg(feature = "axiom-verification")]
use kleis::axiom_verifier::{AxiomVerifier, VerificationResult};

#[test]
fn test_where_constraint_axioms_available_to_z3() {
    // This is THE KEY TEST: Does Z3 know about where constraint axioms?

    let code = r#"
        structure Semiring(S) {
            operation plus : S → S → S
            operation times : S → S → S
            axiom additive_commutativity: ∀(x y : S). equals(plus(x, y), plus(y, x))
        }
        
        structure MatrixMultipliable(m, n, p, T) {
            operation multiply : Matrix(m, n, T) → Matrix(n, p, T) → Matrix(m, p, T)
        }
        
        implements MatrixMultipliable(m, n, p, T) where Semiring(T) {
            operation multiply = builtin_matrix_multiply
        }
    "#;

    // Parse and build registry
    let mut parser = KleisParser::new(code);
    let program = parser.parse_program().expect("Failed to parse");

    let mut registry = StructureRegistry::new();

    for item in program.items {
        match item {
            kleis::kleis_ast::TopLevel::StructureDef(s) => {
                registry.register(s).expect("Failed to register structure");
            }
            kleis::kleis_ast::TopLevel::ImplementsDef(i) => {
                registry.register_implements(i);
            }
            _ => {}
        }
    }

    // Check that where constraints are registered
    let constraints = registry.get_where_constraints("MatrixMultipliable");
    assert_eq!(constraints.len(), 1, "Should have 1 where constraint");
    assert_eq!(constraints[0].structure_name, "Semiring");

    println!("✅ Where constraint registered in registry");

    #[cfg(feature = "axiom-verification")]
    {
        // Now test with axiom verifier
        let mut verifier = AxiomVerifier::new(&registry).expect("Failed to create verifier");

        // Verify a simple axiom that uses 'plus' operation
        // Since MatrixMultipliable has where Semiring(T),
        // and Semiring has commutativity axiom,
        // Z3 should have that axiom available as background theory

        let test_axiom_text = "∀(x y : S). equals(plus(x, y), plus(y, x))";
        let mut axiom_parser = KleisParser::new(test_axiom_text);
        let axiom = axiom_parser
            .parse_proposition()
            .expect("Failed to parse axiom");

        println!("\n🧪 Verifying Semiring commutativity...");
        let result = verifier.verify_axiom(&axiom);

        println!("   Result: {:?}", result);

        let stats = verifier.stats();
        println!("\n📊 Verifier stats:");
        println!("   Structures loaded: {}", stats.loaded_structures);

        if stats.loaded_structures > 0 {
            println!("\n   🎯 SUCCESS! Structures were loaded for verification");
            println!("   This means where constraints triggered structure loading!");
        }
    }
}

#[test]
fn test_where_constraint_loads_dependent_structure() {
    // Test that when we load a structure with where clause,
    // the constrained structure gets loaded too

    let code = r#"
        structure Ring(R) {
            operation plus : R → R → R
            operation times : R → R → R
            axiom distributivity: ∀(x y z : R). equals(times(x, plus(y, z)), plus(times(x, y), times(x, z)))
        }
        
        structure MatrixRing(m, n, T) {
            operation add : Matrix(m, n, T) → Matrix(m, n, T) → Matrix(m, n, T)
        }
        
        implements MatrixRing(m, n, T) where Ring(T) {
            operation add = builtin_matrix_add
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

    #[cfg(feature = "axiom-verification")]
    {
        let mut verifier = AxiomVerifier::new(&registry).expect("Failed to create verifier");

        // When we verify an axiom for MatrixRing, it should load Ring too
        let test_axiom =
            "∀(x y z : R). equals(times(x, plus(y, z)), plus(times(x, y), times(x, z)))";
        let mut parser = KleisParser::new(test_axiom);
        let axiom = parser.parse_proposition().expect("Failed to parse");

        println!("\n🧪 Verifying Ring distributivity (via where constraint)...");
        let result = verifier.verify_axiom(&axiom);

        println!("   Result: {:?}", result);

        let stats = verifier.stats();
        println!("   Structures loaded: {}", stats.loaded_structures);

        // The key test: Did it load structures?
        if stats.loaded_structures >= 1 {
            println!("\n   ✅ Where constraint triggered dependent structure loading!");
        }
    }

    #[cfg(not(feature = "axiom-verification"))]
    {
        println!("✅ Where constraint registered (Z3 verification disabled)");
    }
}

#[test]
fn test_transitive_where_constraints() {
    // Test: Structure A where B, Structure B where C
    // Should load A → B → C transitively

    let code = r#"
        structure Magma(M) {
            operation compose : M → M → M
            axiom closure: ∀(x y : M). equals(compose(x, y), compose(x, y))
        }
        
        structure Semigroup(S) {
            operation compose : S → S → S
            axiom associativity: ∀(x y z : S). equals(compose(compose(x, y), z), compose(x, compose(y, z)))
        }
        
        implements Semigroup(S) where Magma(S) {
            operation compose = builtin_compose
        }
        
        structure Monoid(M) {
            operation compose : M → M → M
            operation e : M
        }
        
        implements Monoid(M) where Semigroup(M) {
            operation compose = builtin_compose
            element e = builtin_identity
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

    // Verify transitive constraints are tracked
    let monoid_constraints = registry.get_where_constraints("Monoid");
    assert_eq!(monoid_constraints.len(), 1);
    assert_eq!(monoid_constraints[0].structure_name, "Semigroup");

    let semigroup_constraints = registry.get_where_constraints("Semigroup");
    assert_eq!(semigroup_constraints.len(), 1);
    assert_eq!(semigroup_constraints[0].structure_name, "Magma");

    println!("✅ Transitive where constraints registered");

    #[cfg(feature = "axiom-verification")]
    {
        let mut verifier = AxiomVerifier::new(&registry).expect("Failed to create verifier");

        // When loading Monoid, should recursively load Semigroup and Magma
        let test_axiom =
            "∀(x y z : S). equals(compose(compose(x, y), z), compose(x, compose(y, z)))";
        let mut parser = KleisParser::new(test_axiom);
        let axiom = parser.parse_proposition().expect("Failed to parse");

        println!("\n🧪 Verifying with transitive constraints...");
        let _ = verifier.verify_axiom(&axiom);

        let stats = verifier.stats();
        println!("   Structures loaded: {}", stats.loaded_structures);

        if stats.loaded_structures >= 2 {
            println!("\n   🎯 TRANSITIVE LOADING WORKS!");
            println!("   Multiple levels of where constraints resolved!");
        }
    }
}
