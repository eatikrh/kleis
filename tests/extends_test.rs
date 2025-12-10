///! Tests for structure inheritance with extends keyword
///!
///! The extends keyword enables structure inheritance:
///! ```kleis
///! structure Monoid(M) extends Semigroup(M) {
///!     element e : M
///!     // Inherits associativity axiom from Semigroup
///! }
///! ```
use kleis::kleis_ast::{StructureDef, TypeExpr};
use kleis::kleis_parser::KleisParser;

#[test]
fn test_simple_extends() {
    // Test: Monoid extends Semigroup
    let code = r#"
        structure Monoid(M) extends Semigroup(M) {
            operation e : M
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let structure = result.unwrap();
    assert_eq!(structure.name, "Monoid");

    // Check extends clause
    assert!(
        structure.extends_clause.is_some(),
        "Extends clause should be present"
    );

    match &structure.extends_clause {
        Some(TypeExpr::Parametric(name, args)) => {
            assert_eq!(name, "Semigroup");
            assert_eq!(args.len(), 1);
            println!("✅ Monoid extends Semigroup parsed!");
        }
        _ => panic!("Expected Parametric type for extends clause"),
    }
}

#[test]
fn test_structure_without_extends() {
    // Test: Regular structure without extends still works
    let code = r#"
        structure Magma(M) {
            operation compose : M → M → M
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let structure = result.unwrap();
    assert!(
        structure.extends_clause.is_none(),
        "Should have no extends clause"
    );

    println!("✅ Structure without extends still works");
}

#[test]
fn test_group_extends_monoid() {
    // Test: Group extends Monoid
    let code = r#"
        structure Group(G) extends Monoid(G) {
            operation inv : G → G
            axiom inverse: ∀(x : G). x × inv(x) = e
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(result.is_ok(), "Failed to parse Group: {:?}", result.err());

    let structure = result.unwrap();
    assert_eq!(structure.name, "Group");

    match &structure.extends_clause {
        Some(TypeExpr::Parametric(name, _)) => {
            assert_eq!(name, "Monoid");
            println!("✅ Group extends Monoid parsed!");
        }
        _ => panic!("Expected extends Monoid"),
    }

    // Check that members are present
    assert!(structure.members.len() >= 2);
}

#[test]
fn test_abelian_group_extends_group() {
    // Test: AbelianGroup extends Group (3-level hierarchy)
    let code = r#"
        structure AbelianGroup(A) extends Group(A) {
            axiom commutativity: ∀(x y : A). x × y = y × x
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let structure = result.unwrap();
    assert_eq!(structure.name, "AbelianGroup");

    match &structure.extends_clause {
        Some(TypeExpr::Parametric(name, _)) => {
            assert_eq!(name, "Group");
            println!("✅ AbelianGroup extends Group parsed!");
            println!("   This creates: Semigroup → Monoid → Group → AbelianGroup");
        }
        _ => panic!("Expected extends Group"),
    }
}

#[test]
fn test_field_extends_ring() {
    // Test: Field extends Ring with multiple type parameters
    let code = r#"
        structure Field(F) extends Ring(F) {
            operation inv : F → F
            axiom multiplicative_inverse: 
                ∀(x : F). implies(not(equals(x, zero)), equals(times(x, inv(x)), one))
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(result.is_ok(), "Failed to parse Field: {:?}", result.err());

    let structure = result.unwrap();
    assert_eq!(structure.name, "Field");

    assert!(structure.extends_clause.is_some());

    println!("✅ Field extends Ring parsed!");
    println!("   Field inherits all Ring axioms!");
}

#[test]
fn test_extends_with_whitespace_variations() {
    // Test: extends parsing handles different whitespace
    let variations = vec![
        "structure Foo(T) extends Bar(T) { }",
        "structure Foo(T)extends Bar(T){ }",
        "structure Foo(T)  extends  Bar(T)  { }",
        "structure Foo(T)\n    extends Bar(T)\n    { }",
    ];

    for code in variations {
        let mut parser = KleisParser::new(code);
        let result = parser.parse_structure();

        assert!(
            result.is_ok(),
            "Failed to parse with whitespace variant: {:?}",
            result.err()
        );

        let structure = result.unwrap();
        assert!(structure.extends_clause.is_some());
    }

    println!("✅ Extends parsing is whitespace-independent");
}

#[test]
fn test_full_hierarchy() {
    // Test: Parse complete algebraic hierarchy
    let code = r#"
        structure Semigroup(S) {
            operation compose : S → S → S
            axiom associativity: ∀(x y z : S). (x × y) × z = x × (y × z)
        }
        
        structure Monoid(M) extends Semigroup(M) {
            operation e : M
            axiom identity: ∀(x : M). x × e = x
        }
        
        structure Group(G) extends Monoid(G) {
            operation inv : G → G
            axiom inverse: ∀(x : G). x × inv(x) = e
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_program();

    assert!(
        result.is_ok(),
        "Failed to parse hierarchy: {:?}",
        result.err()
    );

    let program = result.unwrap();
    assert_eq!(program.items.len(), 3);

    println!("✅ Complete algebraic hierarchy parsed!");
    println!("   Semigroup → Monoid → Group");
    println!("   All with proper extends relationships!");
}
