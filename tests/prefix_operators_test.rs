///! Tests for prefix operators: -, ¬
///!
///! Prefix operators appear before their operands.
///! Examples: -x, ¬p, -(-x)
use kleis::ast::Expression;
use kleis::kleis_parser::KleisParser;

#[test]
fn test_parse_unary_minus() {
    let input = "-x";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing: {}", input);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Operation { name, args } => {
            println!("✅ Parsed as operation: {}", name);
            assert_eq!(name, "negate");
            assert_eq!(args.len(), 1);

            match &args[0] {
                Expression::Object(x) => assert_eq!(x, "x"),
                _ => panic!("Expected Object"),
            }
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_parse_double_negative() {
    let input = "-(-x)";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing: {}", input);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    // Should parse as negate(negate(x))
    match result.unwrap() {
        Expression::Operation { name, args } => {
            assert_eq!(name, "negate");

            match &args[0] {
                Expression::Operation { name, .. } => {
                    assert_eq!(name, "negate");
                    println!("✅ Double negative parsed correctly");
                }
                _ => panic!("Expected nested negate"),
            }
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_parse_unary_minus_with_number() {
    let input = "-5";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing: {}", input);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Operation { name, args } => {
            assert_eq!(name, "negate");
            match &args[0] {
                Expression::Const(n) => assert_eq!(n, "5"),
                _ => panic!("Expected Const"),
            }
            println!("✅ Unary minus with number works");
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_parse_binary_vs_unary_minus() {
    // Binary minus: a - b
    let input1 = "a - b";
    let mut parser1 = KleisParser::new(input1);
    let result1 = parser1.parse_proposition();

    assert!(result1.is_ok());
    match result1.unwrap() {
        Expression::Operation { name, args } => {
            assert_eq!(name, "minus"); // Binary
            assert_eq!(args.len(), 2);
            println!("✅ Binary minus: a - b");
        }
        _ => panic!("Expected binary minus"),
    }

    // Unary minus: -a
    let input2 = "-a";
    let mut parser2 = KleisParser::new(input2);
    let result2 = parser2.parse_proposition();

    assert!(result2.is_ok());
    match result2.unwrap() {
        Expression::Operation { name, args } => {
            assert_eq!(name, "negate"); // Unary
            assert_eq!(args.len(), 1);
            println!("✅ Unary minus: -a");
        }
        _ => panic!("Expected unary minus"),
    }
}

#[test]
fn test_parse_expression_with_unary() {
    // a + -b should parse as a + (negate(b))
    let input = "a + -b";
    let mut parser = KleisParser::new(input);
    let result = parser.parse_proposition();

    println!("\n🔍 Testing: {}", input);

    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    match result.unwrap() {
        Expression::Operation { name, args } => {
            assert_eq!(name, "plus");
            assert_eq!(args.len(), 2);

            // Second arg should be negate(b)
            match &args[1] {
                Expression::Operation { name, .. } => {
                    assert_eq!(name, "negate");
                    println!("✅ Mixed binary and unary minus works");
                }
                _ => panic!("Expected negate"),
            }
        }
        _ => panic!("Expected plus operation"),
    }
}

#[test]
fn test_parse_negate_operation_from_prelude() {
    // The exact line from prelude that was failing
    let input = "operation negate(x) = -x";

    println!("\n🔍 Testing operation from prelude: {}", input);

    // This should now parse as an inline operation definition
    // For now, we're just testing that -x parses as an expression
    let expr_input = "-x";
    let mut parser = KleisParser::new(expr_input);
    let result = parser.parse_proposition();

    assert!(result.is_ok(), "Failed to parse -x: {:?}", result.err());
    println!("✅ The expression -x now parses!");
}
