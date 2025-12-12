#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
///! Tests for custom Unicode mathematical operators
///!
///! Custom operators like •, ⊗, ⊕, ∘ should parse as infix operations.
///! This enables loading prelude.kleis and user-defined algebraic structures.
use kleis::ast::Expression;
use kleis::kleis_parser::KleisParser;

#[test]
fn test_parse_bullet_operator() {
    // The operator that blocks prelude.kleis: •
    let input = "(x • y)";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing: {}", input);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let expr = result.unwrap();
    match expr {
        Expression::Operation { name, args } => {
            println!("✅ Parsed as operation: {}", name);
            assert_eq!(name, "•");
            assert_eq!(args.len(), 2);

            // Check operands
            match (&args[0], &args[1]) {
                (Expression::Object(x), Expression::Object(y)) => {
                    assert_eq!(x, "x");
                    assert_eq!(y, "y");
                }
                _ => panic!("Expected Object operands"),
            }
        }
        _ => panic!("Expected Operation, got {:?}", expr),
    }
}

#[test]
fn test_parse_tensor_product() {
    let input = "a ⊗ b";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing: {}", input);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Operation { name, args } => {
            println!("✅ Parsed as operation: {}", name);
            assert_eq!(name, "⊗");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_parse_direct_sum() {
    let input = "V ⊕ W";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing: {}", input);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Operation { name, .. } => {
            println!("✅ Parsed as operation: {}", name);
            assert_eq!(name, "⊕");
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_parse_composition() {
    let input = "f ∘ g";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing: {}", input);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Operation { name, .. } => {
            println!("✅ Parsed as operation: {}", name);
            assert_eq!(name, "∘");
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_parse_nested_custom_operators() {
    // (x • y) • z
    let input = "(x • y) • z";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing: {}", input);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    // Should parse as •(•(x, y), z)
    match result.unwrap() {
        Expression::Operation { name, args } => {
            println!("✅ Parsed as operation: {}", name);
            assert_eq!(name, "•");
            assert_eq!(args.len(), 2);

            // First arg should be •(x, y)
            match &args[0] {
                Expression::Operation { name, args } => {
                    assert_eq!(name, "•");
                    assert_eq!(args.len(), 2);
                }
                _ => panic!("Expected nested operation"),
            }
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_parse_custom_operator_with_equality() {
    // x • y = y • x
    let input = "x • y = y • x";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing: {}", input);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    // Should parse as =(•(x, y), •(y, x))
    match result.unwrap() {
        Expression::Operation { name, args } => {
            println!("✅ Parsed as operation: {}", name);
            assert_eq!(name, "equals");
            assert_eq!(args.len(), 2);

            // Both sides should be • operations
            match (&args[0], &args[1]) {
                (
                    Expression::Operation { name: n1, .. },
                    Expression::Operation { name: n2, .. },
                ) => {
                    assert_eq!(n1, "•");
                    assert_eq!(n2, "•");
                }
                _ => panic!("Expected operations on both sides"),
            }
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_parse_associativity_axiom() {
    // The exact axiom from prelude.kleis that was failing
    let input = "∀(x y z : S). (x • y) • z = x • (y • z)";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing full associativity axiom");

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Quantifier { body, .. } => {
            println!("✅ Parsed quantifier with custom operators in body");

            // Body should be an equality
            match *body {
                Expression::Operation { ref name, .. } => {
                    assert_eq!(name, "equals");
                    println!("✅ Body is equality expression");
                }
                _ => panic!("Expected equality in body"),
            }
        }
        _ => panic!("Expected Quantifier"),
    }
}

#[test]
fn test_custom_operators_with_precedence() {
    // a + b • c should parse as a + (b • c)
    // because • has same precedence as +, they associate left-to-right
    let input = "a + b • c";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing precedence: {}", input);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    println!("✅ Parsed with mixed operators");
}

#[test]
fn test_multiple_custom_operators() {
    // a ⊗ b ⊕ c
    let input = "a ⊗ b ⊕ c";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing: {}", input);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    // Should parse as ⊕(⊗(a, b), c) due to left-to-right associativity
    match result.unwrap() {
        Expression::Operation { name, args } => {
            println!("✅ Parsed with multiple custom operators");
            assert_eq!(name, "⊕");
            assert_eq!(args.len(), 2);

            // First arg should be ⊗(a, b)
            match &args[0] {
                Expression::Operation { name, .. } => {
                    assert_eq!(name, "⊗");
                }
                _ => panic!("Expected nested operation"),
            }
        }
        _ => panic!("Expected Operation"),
    }
}
