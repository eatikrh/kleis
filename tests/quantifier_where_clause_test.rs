#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
///! Tests for where clauses in quantifiers
///!
///! Example: ∀(x : F) where x ≠ zero. inverse(x) × x = one
///!
///! This is needed for axioms that have preconditions.
use kleis::ast::Expression;
use kleis::kleis_parser::KleisParser;

#[test]
fn test_parse_quantifier_with_where_simple() {
    // ∀(x : F) where x ≠ zero. x
    let input = "∀(x : F) where x ≠ zero. x";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing: {}", input);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Quantifier {
            where_clause, body, ..
        } => {
            println!("✅ Parsed quantifier with where clause");

            // Should have a where clause
            assert!(where_clause.is_some(), "Should have where clause");

            // Where clause should be: x ≠ zero
            match where_clause.unwrap().as_ref() {
                Expression::Operation { name, args, .. } => {
                    assert_eq!(name, "neq");
                    assert_eq!(args.len(), 2);
                    println!("   ✅ Where clause: x ≠ zero");
                }
                _ => panic!("Expected operation in where clause"),
            }

            // Body should be: x
            match body.as_ref() {
                Expression::Object(name) => {
                    assert_eq!(name, "x");
                    println!("   ✅ Body: x");
                }
                _ => panic!("Expected Object in body"),
            }
        }
        _ => panic!("Expected Quantifier"),
    }
}

#[test]
fn test_parse_multiplicative_inverse_axiom() {
    // The exact axiom from prelude.kleis that was failing
    let input = "∀(x : F) where x ≠ zero. inverse(x) × x = one";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing multiplicative inverse axiom");

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Quantifier {
            variables,
            where_clause,
            body,
            ..
        } => {
            println!("✅ Parsed quantifier with where clause");

            // Check variable
            assert_eq!(variables.len(), 1);
            assert_eq!(variables[0].name, "x");
            assert_eq!(variables[0].type_annotation, Some("F".to_string()));
            println!("   ✅ Variable: x : F");

            // Check where clause
            assert!(where_clause.is_some(), "Should have where clause");
            println!("   ✅ Has where clause: x ≠ zero");

            // Check body is equality
            match body.as_ref() {
                Expression::Operation { name, .. } => {
                    assert_eq!(name, "equals");
                    println!("   ✅ Body is equality");
                }
                _ => panic!("Expected equality in body"),
            }
        }
        _ => panic!("Expected Quantifier"),
    }
}

#[test]
fn test_parse_quantifier_without_where() {
    // Regular quantifier without where clause should still work
    let input = "∀(x : M). x • e = x";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing quantifier WITHOUT where clause");

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Quantifier { where_clause, .. } => {
            // Should NOT have a where clause
            assert!(where_clause.is_none(), "Should not have where clause");
            println!("✅ Quantifier without where clause still works");
        }
        _ => panic!("Expected Quantifier"),
    }
}

#[test]
fn test_parse_field_structure() {
    // The Field structure from prelude.kleis
    let code = r#"
        structure Field(F) extends Ring(F) {
            operation (/) : F → F → F
            operation inverse : F → F
            
            axiom multiplicative_inverse:
                ∀(x : F) where x ≠ zero. inverse(x) × x = one
        }
    "#;

    println!("\n🔍 Testing Field structure with where clause in axiom...\n");

    let mut parser = KleisParser::new(code);
    let result = parser.parse_structure();

    assert!(result.is_ok(), "Failed to parse Field: {:?}", result.err());

    let structure = result.unwrap();
    println!("✅ Successfully parsed Field structure!");
    println!("   Name: {}", structure.name);
    println!("   Members: {}", structure.members.len());

    // Check we have the multiplicative_inverse axiom
    use kleis::kleis_ast::StructureMember;
    let has_axiom = structure.members.iter().any(
        |m| matches!(m, StructureMember::Axiom { name, .. } if name == "multiplicative_inverse"),
    );

    assert!(has_axiom, "Should have multiplicative_inverse axiom");
    println!("   ✅ Has multiplicative_inverse axiom with where clause");
}

#[test]
fn test_parse_where_with_comparison() {
    // Test different comparison operators in where clause
    let inputs = vec![
        "∀(x : R) where x > 0. x",
        "∀(x : R) where x < 10. x",
        "∀(x : R) where x ≥ 0. x",
        "∀(x : R) where x ≤ 100. x",
    ];

    println!("\n🔍 Testing where clauses with different comparisons...\n");

    for input in inputs {
        let mut parser = KleisParser::new(input);
        let result = parser.parse_proposition();

        assert!(
            result.is_ok(),
            "Failed to parse: {} - {:?}",
            input,
            result.err()
        );

        match result.unwrap() {
            Expression::Quantifier { where_clause, .. } => {
                assert!(where_clause.is_some());
                println!("   ✅ Parsed: {}", input);
            }
            _ => panic!("Expected Quantifier"),
        }
    }
}

#[test]
fn test_parse_where_with_custom_operator() {
    // Where clause using custom operator
    let input = "∀(x y : G) where x • y = e. x";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing where clause with custom operator: {}", input);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Quantifier { where_clause, .. } => {
            assert!(where_clause.is_some());

            // Where clause should contain the • operator
            match where_clause.unwrap().as_ref() {
                Expression::Operation { name, .. } if name == "equals" => {
                    println!("✅ Where clause with custom operator parsed");
                }
                _ => panic!("Expected equality in where clause"),
            }
        }
        _ => panic!("Expected Quantifier"),
    }
}
