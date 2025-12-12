#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
///! Tests for nested structure definitions
///!
///! Nested structures enable compositional algebra:
///! ```kleis
///! structure Ring(R) {
///!     structure additive : AbelianGroup(R) { ... }
///!     structure multiplicative : Monoid(R) { ... }
///! }
///! ```
///!
///! This is how mathematicians actually define algebraic structures!
use kleis::kleis_ast::{StructureMember, TypeExpr};
use kleis::kleis_parser::KleisParser;

#[test]
fn test_simple_nested_structure() {
    // Test: Basic nested structure without body
    let code = r#"
        structure Ring(R) {
            structure additive : AbelianGroup(R)
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let structure = result.unwrap();
    assert_eq!(structure.name, "Ring");
    assert_eq!(structure.members.len(), 1);

    match &structure.members[0] {
        StructureMember::NestedStructure {
            name,
            structure_type,
            members,
        } => {
            assert_eq!(name, "additive");
            assert!(matches!(structure_type, TypeExpr::Parametric(..)));
            assert_eq!(members.len(), 0, "No body = empty members");

            if let TypeExpr::Parametric(type_name, _) = structure_type {
                assert_eq!(type_name, "AbelianGroup");
            }

            println!("✅ Simple nested structure parsed");
        }
        _ => panic!("Expected NestedStructure"),
    }
}

#[test]
fn test_nested_structure_with_body() {
    // Test: Nested structure with operations in body
    let code = r#"
        structure Ring(R) {
            structure additive : AbelianGroup(R) {
                operation plus : R → R → R
                operation zero : R
            }
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let structure = result.unwrap();
    assert_eq!(structure.members.len(), 1);

    match &structure.members[0] {
        StructureMember::NestedStructure { name, members, .. } => {
            assert_eq!(name, "additive");
            assert_eq!(members.len(), 2, "Should have 2 operations");

            // Check first operation
            match &members[0] {
                StructureMember::Operation { name, .. } => {
                    assert_eq!(name, "plus");
                }
                _ => panic!("Expected Operation"),
            }

            println!("✅ Nested structure with body parsed");
        }
        _ => panic!("Expected NestedStructure"),
    }
}

#[test]
fn test_ring_with_two_nested_structures() {
    // Test: Ring with both additive and multiplicative structures
    let code = r#"
        structure Ring(R) {
            structure additive : AbelianGroup(R) {
                operation plus : R → R → R
                operation negate : R → R
                operation zero : R
            }
            
            structure multiplicative : Monoid(R) {
                operation times : R → R → R
                operation one : R
            }
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(result.is_ok(), "Failed to parse Ring: {:?}", result.err());

    let structure = result.unwrap();
    assert_eq!(structure.name, "Ring");
    assert_eq!(
        structure.members.len(),
        2,
        "Should have 2 nested structures"
    );

    // Check first nested structure (additive)
    match &structure.members[0] {
        StructureMember::NestedStructure { name, members, .. } => {
            assert_eq!(name, "additive");
            assert_eq!(members.len(), 3, "Additive has 3 members");
        }
        _ => panic!("Expected NestedStructure"),
    }

    // Check second nested structure (multiplicative)
    match &structure.members[1] {
        StructureMember::NestedStructure { name, members, .. } => {
            assert_eq!(name, "multiplicative");
            assert_eq!(members.len(), 2, "Multiplicative has 2 members");
        }
        _ => panic!("Expected NestedStructure"),
    }

    println!("✅ Ring with additive + multiplicative structures parsed!");
    println!("   This is exactly how mathematicians define Ring!");
}

#[test]
fn test_nested_structure_with_axiom() {
    // Test: Nested structure containing an axiom
    let code = r#"
        structure VectorSpace(V, F) {
            structure vectors : AbelianGroup(V) {
                operation plus : V → V → V
                axiom commutativity: ∀(v w : V). v + w = w + v
            }
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let structure = result.unwrap();

    match &structure.members[0] {
        StructureMember::NestedStructure { name, members, .. } => {
            assert_eq!(name, "vectors");
            assert_eq!(members.len(), 2);

            // Check axiom is present
            match &members[1] {
                StructureMember::Axiom { name, .. } => {
                    assert_eq!(name, "commutativity");
                }
                _ => panic!("Expected Axiom"),
            }

            println!("✅ Nested structure with axiom parsed");
        }
        _ => panic!("Expected NestedStructure"),
    }
}

#[test]
fn test_mixed_nested_and_regular_members() {
    // Test: Structure with both nested structures and regular operations
    let code = r#"
        structure Ring(R) {
            structure additive : AbelianGroup(R) {
                operation plus : R → R → R
            }
            
            operation times : R → R → R
            
            axiom distributivity: ∀(x y z : R). x × (y + z) = (x × y) + (x × z)
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let structure = result.unwrap();
    assert_eq!(structure.members.len(), 3);

    // Check member types
    assert!(matches!(
        structure.members[0],
        StructureMember::NestedStructure { .. }
    ));
    assert!(matches!(
        structure.members[1],
        StructureMember::Operation { .. }
    ));
    assert!(matches!(
        structure.members[2],
        StructureMember::Axiom { .. }
    ));

    println!("✅ Mixed nested and regular members parsed correctly");
}

#[test]
fn test_deeply_nested_structures() {
    // Test: Nested structures within nested structures
    let code = r#"
        structure ComplexAlgebra(C) {
            structure additive : Group(C) {
                structure monoid : Monoid(C) {
                    operation compose : C → C → C
                }
            }
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(
        result.is_ok(),
        "Failed to parse deeply nested: {:?}",
        result.err()
    );

    let structure = result.unwrap();

    // Check nested within nested
    match &structure.members[0] {
        StructureMember::NestedStructure { name, members, .. } => {
            assert_eq!(name, "additive");
            assert_eq!(members.len(), 1);

            match &members[0] {
                StructureMember::NestedStructure { name, .. } => {
                    assert_eq!(name, "monoid");
                    println!("✅ Deeply nested structures parsed!");
                    println!("   Three levels of nesting work!");
                }
                _ => panic!("Expected nested NestedStructure"),
            }
        }
        _ => panic!("Expected NestedStructure"),
    }
}

#[test]
fn test_vector_space_real_world_example() {
    // Test: Real-world Vector Space definition
    let code = r#"
        structure VectorSpace(V, F) {
            structure vectors : AbelianGroup(V) {
                operation plus : V → V → V
                operation zero : V
            }
            
            structure scalars : Field(F) {
                operation plus : F → F → F
                operation times : F → F → F
                operation zero : F
                operation one : F
            }
            
            operation scale : F → V → V
            
            axiom scale_distributivity: 
                ∀(a b : F). ∀(v : V). scale(plus(a, b), v) = plus(scale(a, v), scale(b, v))
        }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(
        result.is_ok(),
        "Failed to parse VectorSpace: {:?}",
        result.err()
    );

    let structure = result.unwrap();
    assert_eq!(structure.name, "VectorSpace");
    assert_eq!(structure.type_params.len(), 2); // V and F
    assert_eq!(structure.members.len(), 4); // vectors, scalars, scale, axiom

    // Verify nested structures
    assert!(matches!(
        structure.members[0],
        StructureMember::NestedStructure { .. }
    ));
    assert!(matches!(
        structure.members[1],
        StructureMember::NestedStructure { .. }
    ));

    println!("✅ Vector Space definition parsed!");
    println!("   This is a real mathematical structure!");
    println!("   Nested structures enable compositional algebra!");
}
