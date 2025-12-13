//! Integration tests for Match expression translation to Z3
//!
//! Tests the translation of pattern matching to Z3's ite (if-then-else)

#[cfg(feature = "axiom-verification")]
use kleis::ast::{Expression, MatchCase, Pattern};
#[cfg(feature = "axiom-verification")]
use kleis::axiom_verifier::{AxiomVerifier, VerificationResult};
#[cfg(feature = "axiom-verification")]
use kleis::structure_registry::StructureRegistry;

/// Helper to verify an expression
#[cfg(feature = "axiom-verification")]
fn verify_expression(expr: &Expression) -> Result<VerificationResult, String> {
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry)?;
    verifier.verify_axiom(expr)
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_match_simple_wildcard() {
    println!("\n🧪 Testing: match with wildcard pattern");

    // match 5 { _ => 1 } = 1
    let scrutinee = Expression::Const("5".to_string());
    let match_expr = Expression::Match {
        scrutinee: Box::new(scrutinee),
        cases: vec![MatchCase {
            pattern: Pattern::Wildcard,
            body: Expression::Const("1".to_string()),
        }],
    };

    // The match should always produce 1
    let axiom = Expression::Operation {
        name: "equals".to_string(),
        args: vec![match_expr, Expression::Const("1".to_string())],
    };

    match verify_expression(&axiom) {
        Ok(VerificationResult::Valid) => println!("   ✅ Wildcard match works!"),
        other => panic!("Expected Valid, got: {:?}", other),
    }
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_match_variable_binding() {
    println!("\n🧪 Testing: match with variable binding");

    // match 5 { x => x } = 5
    let scrutinee = Expression::Const("5".to_string());
    let match_expr = Expression::Match {
        scrutinee: Box::new(scrutinee),
        cases: vec![MatchCase {
            pattern: Pattern::Variable("x".to_string()),
            body: Expression::Object("x".to_string()),
        }],
    };

    let axiom = Expression::Operation {
        name: "equals".to_string(),
        args: vec![match_expr, Expression::Const("5".to_string())],
    };

    match verify_expression(&axiom) {
        Ok(VerificationResult::Valid) => println!("   ✅ Variable binding works!"),
        other => panic!("Expected Valid, got: {:?}", other),
    }
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_match_constant_pattern() {
    println!("\n🧪 Testing: match with constant pattern");

    // match 3 { 3 => 10 | _ => 20 }
    // This should produce 10 when scrutinee is 3
    let scrutinee = Expression::Const("3".to_string());
    let match_expr = Expression::Match {
        scrutinee: Box::new(scrutinee),
        cases: vec![
            MatchCase {
                pattern: Pattern::Constant("3".to_string()),
                body: Expression::Const("10".to_string()),
            },
            MatchCase {
                pattern: Pattern::Wildcard,
                body: Expression::Const("20".to_string()),
            },
        ],
    };

    let axiom = Expression::Operation {
        name: "equals".to_string(),
        args: vec![match_expr, Expression::Const("10".to_string())],
    };

    match verify_expression(&axiom) {
        Ok(VerificationResult::Valid) => println!("   ✅ Constant pattern works!"),
        other => panic!("Expected Valid, got: {:?}", other),
    }
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_match_multiple_cases() {
    println!("\n🧪 Testing: match with multiple cases");

    // match 1 { 0 => 100 | 1 => 200 | _ => 300 }
    // We'll test with x = 1, should get 200
    let scrutinee = Expression::Const("1".to_string());
    let match_expr = Expression::Match {
        scrutinee: Box::new(scrutinee),
        cases: vec![
            MatchCase {
                pattern: Pattern::Constant("0".to_string()),
                body: Expression::Const("100".to_string()),
            },
            MatchCase {
                pattern: Pattern::Constant("1".to_string()),
                body: Expression::Const("200".to_string()),
            },
            MatchCase {
                pattern: Pattern::Wildcard,
                body: Expression::Const("300".to_string()),
            },
        ],
    };

    let axiom = Expression::Operation {
        name: "equals".to_string(),
        args: vec![match_expr, Expression::Const("200".to_string())],
    };

    match verify_expression(&axiom) {
        Ok(VerificationResult::Valid) => println!("   ✅ Multiple cases work!"),
        other => panic!("Expected Valid, got: {:?}", other),
    }
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_match_fallthrough_to_wildcard() {
    println!("\n🧪 Testing: match fallthrough to wildcard");

    // match 999 { 0 => 100 | 1 => 200 | _ => 300 }
    // Should get 300 (falls through to wildcard)
    let scrutinee = Expression::Const("999".to_string());
    let match_expr = Expression::Match {
        scrutinee: Box::new(scrutinee),
        cases: vec![
            MatchCase {
                pattern: Pattern::Constant("0".to_string()),
                body: Expression::Const("100".to_string()),
            },
            MatchCase {
                pattern: Pattern::Constant("1".to_string()),
                body: Expression::Const("200".to_string()),
            },
            MatchCase {
                pattern: Pattern::Wildcard,
                body: Expression::Const("300".to_string()),
            },
        ],
    };

    let axiom = Expression::Operation {
        name: "equals".to_string(),
        args: vec![match_expr, Expression::Const("300".to_string())],
    };

    match verify_expression(&axiom) {
        Ok(VerificationResult::Valid) => println!("   ✅ Fallthrough to wildcard works!"),
        other => panic!("Expected Valid, got: {:?}", other),
    }
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_match_with_arithmetic_in_body() {
    println!("\n🧪 Testing: match with arithmetic in body");

    // match 5 { y => y + 1 } = 6
    let match_expr = Expression::Match {
        scrutinee: Box::new(Expression::Const("5".to_string())),
        cases: vec![MatchCase {
            pattern: Pattern::Variable("y".to_string()),
            body: Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Object("y".to_string()),
                    Expression::Const("1".to_string()),
                ],
            },
        }],
    };

    let axiom = Expression::Operation {
        name: "equals".to_string(),
        args: vec![match_expr, Expression::Const("6".to_string())],
    };

    match verify_expression(&axiom) {
        Ok(VerificationResult::Valid) => println!("   ✅ Arithmetic in match body works!"),
        other => panic!("Expected Valid, got: {:?}", other),
    }
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_match_constructor_simple() {
    println!("\n🧪 Testing: match with constructor pattern");

    // match Some(5) { Some(x) => x | None => 0 }
    // This tests constructor pattern matching
    let scrutinee = Expression::Operation {
        name: "Some".to_string(),
        args: vec![Expression::Const("5".to_string())],
    };

    let match_expr = Expression::Match {
        scrutinee: Box::new(scrutinee),
        cases: vec![
            MatchCase {
                pattern: Pattern::Constructor {
                    name: "Some".to_string(),
                    args: vec![Pattern::Variable("x".to_string())],
                },
                body: Expression::Object("x".to_string()),
            },
            MatchCase {
                pattern: Pattern::Constructor {
                    name: "None".to_string(),
                    args: vec![],
                },
                body: Expression::Const("0".to_string()),
            },
        ],
    };

    // Should extract x = 5
    let axiom = Expression::Operation {
        name: "equals".to_string(),
        args: vec![match_expr, Expression::Const("5".to_string())],
    };

    match verify_expression(&axiom) {
        Ok(VerificationResult::Valid) => println!("   ✅ Constructor pattern works!"),
        other => panic!("Expected Valid, got: {:?}", other),
    }
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_match_nested_constructor() {
    println!("\n🧪 Testing: match with nested constructor pattern");

    // match Pair(1, 2) { Pair(a, b) => a + b } = 3
    let scrutinee = Expression::Operation {
        name: "Pair".to_string(),
        args: vec![
            Expression::Const("1".to_string()),
            Expression::Const("2".to_string()),
        ],
    };

    let match_expr = Expression::Match {
        scrutinee: Box::new(scrutinee),
        cases: vec![MatchCase {
            pattern: Pattern::Constructor {
                name: "Pair".to_string(),
                args: vec![
                    Pattern::Variable("a".to_string()),
                    Pattern::Variable("b".to_string()),
                ],
            },
            body: Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Object("a".to_string()),
                    Expression::Object("b".to_string()),
                ],
            },
        }],
    };

    let axiom = Expression::Operation {
        name: "equals".to_string(),
        args: vec![match_expr, Expression::Const("3".to_string())],
    };

    match verify_expression(&axiom) {
        Ok(VerificationResult::Valid) => println!("   ✅ Nested constructor pattern works!"),
        other => panic!("Expected Valid, got: {:?}", other),
    }
}
