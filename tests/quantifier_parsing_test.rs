#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
///! Test parsing of universal quantifiers for axioms
///!
///! Tests Phase 1 Task 1: Universal Quantifiers
use kleis::ast::{Expression, QuantifiedVar, QuantifierKind};
use kleis::kleis_parser::KleisParser;

#[test]
fn test_parse_simple_forall() {
    let input = "∀(x : M). x";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let expr = result.unwrap();
    match expr {
        Expression::Quantifier {
            quantifier,
            variables,
            body,
            where_clause: _,
        } => {
            assert!(matches!(quantifier, QuantifierKind::ForAll));
            assert_eq!(variables.len(), 1);
            assert_eq!(variables[0].name, "x");
            assert_eq!(variables[0].type_annotation, Some("M".to_string()));

            // Body should be Object("x")
            match *body {
                Expression::Object(ref name) => assert_eq!(name, "x"),
                _ => panic!("Expected Object in body, got {:?}", body),
            }
        }
        _ => panic!("Expected Quantifier, got {:?}", expr),
    }
}

#[test]
fn test_parse_forall_multiple_vars() {
    let input = "∀(x y z : R). x";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let expr = result.unwrap();
    match expr {
        Expression::Quantifier {
            quantifier,
            variables,
            ..
        } => {
            assert!(matches!(quantifier, QuantifierKind::ForAll));
            assert_eq!(variables.len(), 3);
            assert_eq!(variables[0].name, "x");
            assert_eq!(variables[1].name, "y");
            assert_eq!(variables[2].name, "z");
            assert_eq!(variables[0].type_annotation, Some("R".to_string()));
            assert_eq!(variables[1].type_annotation, Some("R".to_string()));
            assert_eq!(variables[2].type_annotation, Some("R".to_string()));
        }
        _ => panic!("Expected Quantifier, got {:?}", expr),
    }
}

#[test]
fn test_parse_forall_with_operation() {
    // Identity axiom: ∀(x : M). x • e = x
    let input = "∀(x : M). plus(x, e)";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let expr = result.unwrap();
    match expr {
        Expression::Quantifier { body, .. } => {
            // Body should be Operation("plus", [x, e])
            match *body {
                Expression::Operation {
                    ref name, ref args, ..
                } => {
                    assert_eq!(name, "plus");
                    assert_eq!(args.len(), 2);
                }
                _ => panic!("Expected Operation in body, got {:?}", body),
            }
        }
        _ => panic!("Expected Quantifier, got {:?}", expr),
    }
}

#[test]
fn test_parse_exists() {
    let input = "∃(x : M). x";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let expr = result.unwrap();
    match expr {
        Expression::Quantifier { quantifier, .. } => {
            assert!(matches!(quantifier, QuantifierKind::Exists));
        }
        _ => panic!("Expected Quantifier, got {:?}", expr),
    }
}

#[test]
fn test_parse_forall_keyword() {
    let input = "forall(x : M). x";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let expr = result.unwrap();
    match expr {
        Expression::Quantifier { quantifier, .. } => {
            assert!(matches!(quantifier, QuantifierKind::ForAll));
        }
        _ => panic!("Expected Quantifier, got {:?}", expr),
    }
}

#[test]
fn test_parse_nested_quantifiers() {
    // ∀(x : M). ∀(y : M). plus(x, y)
    let input = "∀(x : M). ∀(y : M). plus(x, y)";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let expr = result.unwrap();
    match expr {
        Expression::Quantifier { body, .. } => {
            // Body should be another Quantifier
            match *body {
                Expression::Quantifier {
                    body: inner_body, ..
                } => {
                    // Inner body should be Operation
                    match *inner_body {
                        Expression::Operation { ref name, .. } => {
                            assert_eq!(name, "plus");
                        }
                        _ => panic!("Expected Operation in inner body"),
                    }
                }
                _ => panic!("Expected nested Quantifier in body"),
            }
        }
        _ => panic!("Expected Quantifier, got {:?}", expr),
    }
}

#[test]
fn test_axiom_in_structure() {
    // Test parsing a structure with an axiom
    let input = r#"
        structure Monoid(M) {
            operation e : M
            operation (•) : M → M → M
            axiom identity: ∀(x : M). plus(x, e)
        }
    "#;

    let mut parser = KleisParser::new(input);
    let result = parser.parse_structure();

    assert!(
        result.is_ok(),
        "Failed to parse structure: {:?}",
        result.err()
    );

    let structure = result.unwrap();
    assert_eq!(structure.name, "Monoid");
    assert_eq!(structure.members.len(), 3);

    // Check the axiom member
    match &structure.members[2] {
        kleis::kleis_ast::StructureMember::Axiom { name, proposition } => {
            assert_eq!(name, "identity");

            // Proposition should be a Quantifier
            match proposition {
                Expression::Quantifier { .. } => {
                    // Success!
                }
                _ => panic!("Expected Quantifier in axiom proposition"),
            }
        }
        _ => panic!("Expected Axiom as third member"),
    }
}
