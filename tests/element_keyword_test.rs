///! Tests for the `element` keyword in structure definitions
///!
///! The `element` keyword is semantically equivalent to a nullary operation:
///! - `element e : M` is the same as `operation e : M` (no arrows = nullary)
///! - Identity elements are just operations that take no arguments
use kleis::kleis_ast::StructureMember;
use kleis::kleis_parser::KleisParser;

#[test]
fn test_parse_element_in_structure() {
    // Monoid with identity element
    let code = r#"
        structure Monoid(M) {
            operation (•) : M → M → M
            element e : M
        }
    "#;

    println!("\n🔍 Testing element keyword in structure...\n");

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let structure = result.unwrap();
    assert_eq!(structure.name, "Monoid");
    assert_eq!(structure.members.len(), 2);

    // First member: operation (•)
    match &structure.members[0] {
        StructureMember::Operation { name, .. } => {
            assert_eq!(name, "•");
            println!("✅ Found operation: •");
        }
        _ => panic!("Expected Operation for •"),
    }

    // Second member: element e (stored as nullary operation)
    match &structure.members[1] {
        StructureMember::Operation {
            name,
            type_signature,
        } => {
            assert_eq!(name, "e");
            println!("✅ Found element e (stored as operation)");
            println!("   Type signature: {:?}", type_signature);
        }
        _ => panic!("Expected Operation for element e"),
    }
}

#[test]
fn test_parse_monoid_with_element() {
    // Full Monoid structure from prelude.kleis
    let code = r#"
        structure Monoid(M) {
            operation (•) : M → M → M
            element e : M
            
            axiom left_identity:
                ∀(x : M). e • x = x
                
            axiom right_identity:
                ∀(x : M). x • e = x
        }
    "#;

    println!("\n🔍 Testing full Monoid structure with element...\n");

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let structure = result.unwrap();
    println!("✅ Successfully parsed Monoid with element");
    println!("   Name: {}", structure.name);
    println!("   Members: {}", structure.members.len());

    assert_eq!(structure.name, "Monoid");
    assert_eq!(structure.members.len(), 4); // operation, element, 2 axioms

    // Verify we have the element
    let has_element = structure
        .members
        .iter()
        .any(|m| matches!(m, StructureMember::Operation { name, .. } if name == "e"));

    assert!(has_element, "Should have element e");
    println!("   ✅ Has element e");

    // Verify we have the axioms
    let axiom_count = structure
        .members
        .iter()
        .filter(|m| matches!(m, StructureMember::Axiom { .. }))
        .count();

    assert_eq!(axiom_count, 2, "Should have 2 axioms");
    println!("   ✅ Has 2 axioms");
}

#[test]
fn test_parse_ring_with_elements() {
    // Ring with two identity elements (zero and one)
    let code = r#"
        structure Ring(R) {
            operation (+) : R → R → R
            operation (×) : R → R → R
            element zero : R
            element one : R
        }
    "#;

    println!("\n🔍 Testing Ring with multiple elements...\n");

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let structure = result.unwrap();
    println!("✅ Successfully parsed Ring with elements");

    // Count elements (stored as operations)
    let element_names: Vec<&str> = structure
        .members
        .iter()
        .filter_map(|m| {
            match m {
                StructureMember::Operation {
                    name,
                    type_signature,
                } => {
                    // Elements are operations with type R (not R → R → R)
                    // We can distinguish by checking if the type is a simple named type
                    Some(name.as_str())
                }
                _ => None,
            }
        })
        .collect();

    println!("   Found operations/elements: {:?}", element_names);
    assert!(element_names.contains(&"zero"), "Should have zero element");
    assert!(element_names.contains(&"one"), "Should have one element");
    println!("   ✅ Has zero and one elements");
}

#[test]
fn test_element_vs_operation_syntax() {
    // Verify that element and operation are equivalent for nullary operations
    let with_element = r#"
        structure Test1(T) {
            element id : T
        }
    "#;

    let with_operation = r#"
        structure Test2(T) {
            operation id : T
        }
    "#;

    println!("\n🔍 Testing element vs operation equivalence...\n");

    let mut parser1 = KleisParser::new(with_element);
    let result1 = parser1.parse_structure().unwrap();

    let mut parser2 = KleisParser::new(with_operation);
    let result2 = parser2.parse_structure().unwrap();

    // Both should parse to the same AST structure
    match (&result1.members[0], &result2.members[0]) {
        (
            StructureMember::Operation {
                name: n1,
                type_signature: t1,
            },
            StructureMember::Operation {
                name: n2,
                type_signature: t2,
            },
        ) => {
            assert_eq!(n1, n2);
            assert_eq!(t1, t2);
            println!("✅ element and operation produce equivalent AST");
        }
        _ => panic!("Both should be Operation members"),
    }
}

#[test]
fn test_parse_group_with_element_and_extends() {
    // Group extends Monoid, which has an element
    let code = r#"
        structure Group(G) extends Monoid(G) {
            operation inv : G → G
            
            axiom left_inverse:
                ∀(x : G). inv(x) • x = e
        }
    "#;

    println!("\n🔍 Testing Group with extends (inherits element from Monoid)...\n");

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let structure = result.unwrap();
    println!("✅ Successfully parsed Group with extends");
    assert_eq!(structure.name, "Group");
    assert!(
        structure.extends_clause.is_some(),
        "Should have extends clause"
    );
    println!("   ✅ Has extends clause");
}
