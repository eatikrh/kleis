//! Test for Grammar v0.6 - Functions in Structures
//! Verifies that function definitions inside structures are parsed correctly
//! Resolves TODO #11

use kleis::kleis_ast::{StructureMember, TopLevel};
use kleis::kleis_parser::KleisParser;

#[test]
fn test_parse_ring_with_derived_subtraction() {
    let code = r#"
    structure Ring(R) {
      operation (+) : R × R → R
      operation negate : R → R
      
      // Derived operation with default implementation
      operation (-) : R × R → R
      define (-)(x, y) = x + negate(y)
    }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(
        result.is_ok(),
        "Failed to parse Ring structure: {:?}",
        result.err()
    );

    let structure = result.unwrap();
    assert_eq!(structure.name, "Ring");
    assert_eq!(structure.members.len(), 4); // +, negate, -, and define(-)

    // Check that we have a FunctionDef member
    let has_function_def = structure
        .members
        .iter()
        .any(|m| matches!(m, StructureMember::FunctionDef(_)));
    assert!(
        has_function_def,
        "Structure should contain a FunctionDef member"
    );

    // Check that the FunctionDef is for the (-) operation
    let subtraction_func = structure.members.iter().find_map(|m| {
        if let StructureMember::FunctionDef(func_def) = m {
            if func_def.name == "-" {
                Some(func_def)
            } else {
                None
            }
        } else {
            None
        }
    });

    assert!(
        subtraction_func.is_some(),
        "Should have function definition for (-)"
    );
    let func_def = subtraction_func.unwrap();
    assert_eq!(func_def.params.len(), 2); // x and y
    assert_eq!(func_def.params[0], "x");
    assert_eq!(func_def.params[1], "y");
}

#[test]
fn test_parse_field_with_derived_division() {
    let code = r#"
    structure Field(F) {
      operation (×) : F × F → F
      operation inverse : F → F
      
      operation (/) : F × F → F
      define (/)(x, y) = x × inverse(y)
    }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(
        result.is_ok(),
        "Failed to parse Field structure: {:?}",
        result.err()
    );

    let structure = result.unwrap();
    assert_eq!(structure.name, "Field");

    // Check that we have the division function def
    let division_func = structure.members.iter().find_map(|m| {
        if let StructureMember::FunctionDef(func_def) = m {
            if func_def.name == "/" {
                Some(func_def)
            } else {
                None
            }
        } else {
            None
        }
    });

    assert!(
        division_func.is_some(),
        "Should have function definition for (/)"
    );
}

#[test]
fn test_parse_program_with_structure_containing_define() {
    let code = r#"
    structure Monoid(M) {
      operation (•) : M × M → M
      element e : M
      
      define identity() = e
    }
    
    define test = 42
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_program();

    assert!(
        result.is_ok(),
        "Failed to parse program: {:?}",
        result.err()
    );

    let program = result.unwrap();
    assert_eq!(program.items.len(), 2); // Structure and top-level function

    // Check first item is structure
    match &program.items[0] {
        TopLevel::StructureDef(structure) => {
            assert_eq!(structure.name, "Monoid");

            // Check that structure has FunctionDef member
            let has_function = structure
                .members
                .iter()
                .any(|m| matches!(m, StructureMember::FunctionDef(_)));
            assert!(
                has_function,
                "Monoid structure should have function definition"
            );
        }
        _ => panic!("Expected StructureDef"),
    }

    // Check second item is top-level function
    assert!(matches!(program.items[1], TopLevel::FunctionDef(_)));
}

#[test]
fn test_no_regression_structures_without_define() {
    // Ensure structures without define still work
    let code = r#"
    structure Semigroup(S) {
      operation (•) : S × S → S
      
      axiom associativity:
        ∀(x y z : S). (x • y) • z = x • (y • z)
    }
    "#;

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(
        result.is_ok(),
        "Failed to parse Semigroup: {:?}",
        result.err()
    );

    let structure = result.unwrap();
    assert_eq!(structure.name, "Semigroup");

    // Should have 2 members (operation and axiom), no FunctionDef
    assert_eq!(structure.members.len(), 2);
    let has_function = structure
        .members
        .iter()
        .any(|m| matches!(m, StructureMember::FunctionDef(_)));
    assert!(
        !has_function,
        "Semigroup should not have any function definitions"
    );
}
