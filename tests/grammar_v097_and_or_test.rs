//! Tests for Grammar v0.97: ASCII logical operators (and, or, not)
//!
//! This test file verifies that:
//! 1. 'and' works as an infix operator (same as ∧)
//! 2. 'or' works as an infix operator (same as ∨)
//! 3. 'not' works as a prefix operator (same as ¬)
//! 4. These work in all contexts: axioms, example blocks, assertions

use kleis::kleis_parser::{parse_kleis, parse_kleis_program};

#[test]
fn test_and_keyword_parses() {
    // 'and' should parse as logical_and
    let result = parse_kleis("P and Q").unwrap();
    match result {
        kleis::ast::Expression::Operation { name, args, .. } => {
            assert_eq!(name, "logical_and");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Operation, got {:?}", result),
    }
}

#[test]
fn test_or_keyword_parses() {
    // 'or' should parse as logical_or
    let result = parse_kleis("P or Q").unwrap();
    match result {
        kleis::ast::Expression::Operation { name, args, .. } => {
            assert_eq!(name, "logical_or");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Operation, got {:?}", result),
    }
}

#[test]
fn test_not_keyword_parses() {
    // 'not' should parse as logical_not
    let result = parse_kleis("not P").unwrap();
    match result {
        kleis::ast::Expression::Operation { name, args, .. } => {
            assert_eq!(name, "logical_not");
            assert_eq!(args.len(), 1);
        }
        _ => panic!("Expected Operation, got {:?}", result),
    }
}

#[test]
fn test_unicode_still_works() {
    // Unicode operators should still work
    let and_result = parse_kleis("P ∧ Q").unwrap();
    let or_result = parse_kleis("P ∨ Q").unwrap();
    let not_result = parse_kleis("¬P").unwrap();

    match and_result {
        kleis::ast::Expression::Operation { name, .. } => assert_eq!(name, "logical_and"),
        _ => panic!("Expected logical_and"),
    }
    match or_result {
        kleis::ast::Expression::Operation { name, .. } => assert_eq!(name, "logical_or"),
        _ => panic!("Expected logical_or"),
    }
    match not_result {
        kleis::ast::Expression::Operation { name, .. } => assert_eq!(name, "logical_not"),
        _ => panic!("Expected logical_not"),
    }
}

#[test]
fn test_chained_and() {
    // P and Q and R should parse correctly (left-associative)
    let result = parse_kleis("P and Q and R").unwrap();
    match result {
        kleis::ast::Expression::Operation { name, args, .. } => {
            assert_eq!(name, "logical_and");
            assert_eq!(args.len(), 2);
            // First arg should be (P and Q)
            match &args[0] {
                kleis::ast::Expression::Operation { name, .. } => {
                    assert_eq!(name, "logical_and");
                }
                _ => panic!("Expected nested logical_and"),
            }
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_chained_or() {
    // P or Q or R should parse correctly (left-associative)
    let result = parse_kleis("P or Q or R").unwrap();
    match result {
        kleis::ast::Expression::Operation { name, args, .. } => {
            assert_eq!(name, "logical_or");
            assert_eq!(args.len(), 2);
            // First arg should be (P or Q)
            match &args[0] {
                kleis::ast::Expression::Operation { name, .. } => {
                    assert_eq!(name, "logical_or");
                }
                _ => panic!("Expected nested logical_or"),
            }
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_mixed_and_or_precedence() {
    // P or Q and R should parse as P or (Q and R)
    // because 'and' has higher precedence than 'or'
    let result = parse_kleis("P or Q and R").unwrap();
    match result {
        kleis::ast::Expression::Operation { name, args, .. } => {
            assert_eq!(name, "logical_or");
            assert_eq!(args.len(), 2);
            // Second arg should be (Q and R)
            match &args[1] {
                kleis::ast::Expression::Operation { name, .. } => {
                    assert_eq!(name, "logical_and");
                }
                _ => panic!("Expected Q and R as second arg"),
            }
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_not_has_highest_precedence() {
    // not P and Q should parse as (not P) and Q
    let result = parse_kleis("not P and Q").unwrap();
    match result {
        kleis::ast::Expression::Operation { name, args, .. } => {
            assert_eq!(name, "logical_and");
            assert_eq!(args.len(), 2);
            // First arg should be (not P)
            match &args[0] {
                kleis::ast::Expression::Operation { name, .. } => {
                    assert_eq!(name, "logical_not");
                }
                _ => panic!("Expected not P as first arg"),
            }
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_mixed_unicode_and_ascii() {
    // Mixed usage: P and Q ∨ R
    let result = parse_kleis("P and Q ∨ R").unwrap();
    match result {
        kleis::ast::Expression::Operation { name, .. } => {
            assert_eq!(name, "logical_or");
        }
        _ => panic!("Expected logical_or"),
    }
}

#[test]
fn test_comparison_with_and() {
    // x > 0 and y < 10 should parse correctly
    let result = parse_kleis("x > 0 and y < 10").unwrap();
    match result {
        kleis::ast::Expression::Operation { name, args, .. } => {
            assert_eq!(name, "logical_and");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected logical_and"),
    }
}

#[test]
fn test_comparison_with_or() {
    // x = 0 or y = 0 should parse correctly
    let result = parse_kleis("x = 0 or y = 0").unwrap();
    match result {
        kleis::ast::Expression::Operation { name, args, .. } => {
            assert_eq!(name, "logical_or");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected logical_or"),
    }
}

#[test]
fn test_parenthesized_expressions() {
    // (P and Q) or R
    let result = parse_kleis("(P and Q) or R").unwrap();
    match result {
        kleis::ast::Expression::Operation { name, args, .. } => {
            assert_eq!(name, "logical_or");
            // First arg should be (P and Q)
            match &args[0] {
                kleis::ast::Expression::Operation { name, .. } => {
                    assert_eq!(name, "logical_and");
                }
                _ => panic!("Expected P and Q as first arg"),
            }
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_double_negation() {
    // not not P
    let result = parse_kleis("not not P").unwrap();
    match result {
        kleis::ast::Expression::Operation { name, args, .. } => {
            assert_eq!(name, "logical_not");
            // Arg should be (not P)
            match &args[0] {
                kleis::ast::Expression::Operation { name, .. } => {
                    assert_eq!(name, "logical_not");
                }
                _ => panic!("Expected not P as arg"),
            }
        }
        _ => panic!("Expected Operation"),
    }
}

// ============================================================
// Program-level tests (define, structure, axiom)
// ============================================================

#[test]
fn test_and_in_function_def() {
    // 'and' in a function definition body
    let code = "define test = True and False";
    let result = parse_kleis_program(code);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    let program = result.unwrap();
    assert_eq!(program.functions().len(), 1);
}

#[test]
fn test_or_in_function_def() {
    // 'or' in a function definition body
    let code = "define test = True or False";
    let result = parse_kleis_program(code);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_not_in_function_def() {
    // 'not' in a function definition body
    let code = "define test = not False";
    let result = parse_kleis_program(code);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_and_in_axiom() {
    // 'and' in a structure axiom
    let code = r#"
structure Test {
    axiom a: forall P : Bool . forall Q : Bool . (P and Q) = (Q and P)
}
"#;
    let result = parse_kleis_program(code);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_or_in_axiom() {
    // 'or' in a structure axiom
    let code = r#"
structure Test {
    axiom a: forall P : Bool . forall Q : Bool . (P or Q) = (Q or P)
}
"#;
    let result = parse_kleis_program(code);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_not_in_axiom() {
    // 'not' in a structure axiom
    let code = r#"
structure Test {
    axiom a: forall P : Bool . not (not P) = P
}
"#;
    let result = parse_kleis_program(code);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_and_in_example_block() {
    // 'and' in an example block
    let code = r#"
example "test and" {
    let P = True in
    let Q = True in
    assert(P and Q = True)
}
"#;
    let result = parse_kleis_program(code);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
