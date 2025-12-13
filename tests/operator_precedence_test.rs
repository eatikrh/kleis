//! Operator Precedence Tests
//!
//! Tests that operators are parsed with correct precedence:
//!
//! Precedence (lowest to highest):
//! 1. ⟹ (implication)
//! 2. ∨ || (disjunction)
//! 3. ∧ && (conjunction)
//! 4. = < > <= >= != ≤ ≥ ≠ (comparison)
//! 5. + - (additive)
//! 6. * / (multiplicative)
//! 7. ^ (exponentiation)
//! 8. unary - ¬ (prefix operators)
//! 9. function calls, parentheses, atoms

use kleis::ast::Expression;
use kleis::kleis_parser::KleisParser;

// =============================================================================
// ARITHMETIC PRECEDENCE TESTS
// =============================================================================

#[test]
fn test_multiplication_binds_tighter_than_addition() {
    // a + b * c should parse as a + (b * c)
    let input = "a + b * c";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing: {} -> plus(a, times(b, c))", input);

    match result {
        Expression::Operation { name, args } => {
            assert_eq!(name, "plus", "Top-level should be plus");
            assert_eq!(args.len(), 2);

            // Left arg should be 'a'
            match &args[0] {
                Expression::Object(name) => assert_eq!(name, "a"),
                _ => panic!("Left should be object 'a'"),
            }

            // Right arg should be times(b, c)
            match &args[1] {
                Expression::Operation { name, args } => {
                    assert_eq!(name, "times");
                    assert_eq!(args.len(), 2);
                }
                _ => panic!("Right should be times(b, c)"),
            }
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: * binds tighter than +");
}

#[test]
fn test_multiplication_binds_tighter_than_subtraction() {
    // a - b * c should parse as a - (b * c)
    let input = "a - b * c";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing: {} -> minus(a, times(b, c))", input);

    match result {
        Expression::Operation { name, args } => {
            assert_eq!(name, "minus", "Top-level should be minus");

            // Right arg should be times(b, c)
            match &args[1] {
                Expression::Operation { name, .. } => {
                    assert_eq!(name, "times");
                }
                _ => panic!("Right should be times(b, c)"),
            }
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: * binds tighter than -");
}

#[test]
fn test_left_to_right_associativity_addition() {
    // a + b + c should parse as (a + b) + c
    let input = "a + b + c";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing: {} -> plus(plus(a, b), c)", input);

    match result {
        Expression::Operation { name, args } => {
            assert_eq!(name, "plus", "Top-level should be plus");

            // Left arg should be plus(a, b)
            match &args[0] {
                Expression::Operation { name, .. } => {
                    assert_eq!(name, "plus", "Left should be plus(a, b)");
                }
                _ => panic!("Left should be plus(a, b)"),
            }

            // Right arg should be 'c'
            match &args[1] {
                Expression::Object(name) => assert_eq!(name, "c"),
                _ => panic!("Right should be 'c'"),
            }
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: + is left-associative");
}

#[test]
fn test_left_to_right_associativity_multiplication() {
    // a * b * c should parse as (a * b) * c
    let input = "a * b * c";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing: {} -> times(times(a, b), c)", input);

    match result {
        Expression::Operation { name, args } => {
            assert_eq!(name, "times", "Top-level should be times");

            // Left arg should be times(a, b)
            match &args[0] {
                Expression::Operation { name, .. } => {
                    assert_eq!(name, "times", "Left should be times(a, b)");
                }
                _ => panic!("Left should be times(a, b)"),
            }
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: * is left-associative");
}

#[test]
fn test_division_binds_like_multiplication() {
    // a + b / c should parse as a + (b / c)
    let input = "a + b / c";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing: {} -> plus(a, divide(b, c))", input);

    match result {
        Expression::Operation { name, args } => {
            assert_eq!(name, "plus", "Top-level should be plus");

            // Right arg should be divide(b, c)
            match &args[1] {
                Expression::Operation { name, .. } => {
                    assert_eq!(name, "divide");
                }
                _ => panic!("Right should be divide(b, c)"),
            }
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: / binds tighter than +");
}

// =============================================================================
// COMPARISON WITH ARITHMETIC TESTS
// =============================================================================

#[test]
fn test_comparison_binds_looser_than_arithmetic() {
    // a * b < c * d should parse as (a * b) < (c * d)
    let input = "a * b < c * d";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing: {} -> lt(times(a,b), times(c,d))", input);

    match result {
        Expression::Operation { name, args } => {
            assert_eq!(name, "less_than", "Top-level should be less_than");
            assert_eq!(args.len(), 2);

            // Left arg should be times(a, b)
            match &args[0] {
                Expression::Operation { name, .. } => {
                    assert_eq!(name, "times");
                }
                _ => panic!("Left should be times(a, b)"),
            }

            // Right arg should be times(c, d)
            match &args[1] {
                Expression::Operation { name, .. } => {
                    assert_eq!(name, "times");
                }
                _ => panic!("Right should be times(c, d)"),
            }
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: < binds looser than *");
}

#[test]
fn test_geq_with_arithmetic_on_both_sides() {
    // a * 100 >= b * 80 should parse as (a * 100) >= (b * 80)
    let input = "a * 100 >= b * 80";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing: {} -> geq(times(a,100), times(b,80))", input);

    match result {
        Expression::Operation { name, args } => {
            assert_eq!(name, "geq", "Top-level should be geq");
            assert_eq!(args.len(), 2);

            // Both args should be times
            match &args[0] {
                Expression::Operation { name, .. } => {
                    assert_eq!(name, "times", "Left should be times");
                }
                _ => panic!("Left should be times(a, 100)"),
            }

            match &args[1] {
                Expression::Operation { name, .. } => {
                    assert_eq!(name, "times", "Right should be times");
                }
                _ => panic!("Right should be times(b, 80)"),
            }
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: >= correctly parses arithmetic on both sides");
}

#[test]
fn test_leq_with_arithmetic() {
    // a + b <= c - d
    let input = "a + b <= c - d";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing: {} -> leq(plus(a,b), minus(c,d))", input);

    match result {
        Expression::Operation { name, args } => {
            assert_eq!(name, "leq", "Top-level should be leq");

            match &args[0] {
                Expression::Operation { name, .. } => assert_eq!(name, "plus"),
                _ => panic!("Left should be plus"),
            }

            match &args[1] {
                Expression::Operation { name, .. } => assert_eq!(name, "minus"),
                _ => panic!("Right should be minus"),
            }
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: <= with arithmetic on both sides");
}

#[test]
fn test_neq_with_arithmetic() {
    // a * 2 != b / 3
    let input = "a * 2 != b / 3";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing: {} -> neq(times(a,2), divide(b,3))", input);

    match result {
        Expression::Operation { name, args } => {
            assert_eq!(name, "neq", "Top-level should be neq");

            match &args[0] {
                Expression::Operation { name, .. } => assert_eq!(name, "times"),
                _ => panic!("Left should be times"),
            }

            match &args[1] {
                Expression::Operation { name, .. } => assert_eq!(name, "divide"),
                _ => panic!("Right should be divide"),
            }
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: != with arithmetic");
}

// =============================================================================
// LOGICAL OPERATOR PRECEDENCE
// =============================================================================

#[test]
fn test_conjunction_binds_tighter_than_disjunction() {
    // a ∨ b ∧ c should parse as a ∨ (b ∧ c)
    let input = "a ∨ b ∧ c";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing: {} -> or(a, and(b, c))", input);

    match result {
        Expression::Operation { name, args } => {
            assert_eq!(name, "logical_or", "Top-level should be logical_or");

            // Right arg should be logical_and(b, c)
            match &args[1] {
                Expression::Operation { name, .. } => {
                    assert_eq!(name, "logical_and");
                }
                _ => panic!("Right should be logical_and(b, c)"),
            }
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: ∧ binds tighter than ∨");
}

#[test]
fn test_comparison_binds_tighter_than_conjunction() {
    // a < b ∧ c > d should parse as (a < b) ∧ (c > d)
    let input = "a < b ∧ c > d";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing: {} -> and(lt(a,b), gt(c,d))", input);

    match result {
        Expression::Operation { name, args } => {
            assert_eq!(name, "logical_and", "Top-level should be logical_and");

            match &args[0] {
                Expression::Operation { name, .. } => assert_eq!(name, "less_than"),
                _ => panic!("Left should be less_than"),
            }

            match &args[1] {
                Expression::Operation { name, .. } => assert_eq!(name, "greater_than"),
                _ => panic!("Right should be greater_than"),
            }
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: comparison binds tighter than ∧");
}

#[test]
fn test_implication_binds_loosest() {
    // a ∧ b ⟹ c ∨ d should parse as (a ∧ b) ⟹ (c ∨ d)
    let input = "a ∧ b ⟹ c ∨ d";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing: {} -> implies(and(a,b), or(c,d))", input);

    match result {
        Expression::Operation { name, args } => {
            assert_eq!(name, "implies", "Top-level should be implies");

            match &args[0] {
                Expression::Operation { name, .. } => assert_eq!(name, "logical_and"),
                _ => panic!("Left should be logical_and"),
            }

            match &args[1] {
                Expression::Operation { name, .. } => assert_eq!(name, "logical_or"),
                _ => panic!("Right should be logical_or"),
            }
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: ⟹ binds loosest");
}

// =============================================================================
// MIXED PRECEDENCE TESTS
// =============================================================================

#[test]
fn test_complex_mixed_precedence() {
    // a + b * c >= d - e / f should parse as ((a + (b*c)) >= (d - (e/f)))
    let input = "a + b * c >= d - e / f";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing complex expression: {}", input);

    match result {
        Expression::Operation { name, args } => {
            assert_eq!(name, "geq", "Top-level should be geq");

            // Left: a + (b * c)
            match &args[0] {
                Expression::Operation { name, args } => {
                    assert_eq!(name, "plus");
                    match &args[1] {
                        Expression::Operation { name, .. } => assert_eq!(name, "times"),
                        _ => panic!("Expected times"),
                    }
                }
                _ => panic!("Left should be plus"),
            }

            // Right: d - (e / f)
            match &args[1] {
                Expression::Operation { name, args } => {
                    assert_eq!(name, "minus");
                    match &args[1] {
                        Expression::Operation { name, .. } => assert_eq!(name, "divide"),
                        _ => panic!("Expected divide"),
                    }
                }
                _ => panic!("Right should be minus"),
            }
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: complex mixed precedence");
}

#[test]
fn test_arithmetic_comparison_logical_combined() {
    // a * 2 > b ∧ c + 1 < d should parse as (a*2 > b) ∧ (c+1 < d)
    // Note: using Unicode ∧ instead of && which is only recognized in conditionals
    let input = "a * 2 > b ∧ c + 1 < d";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing: {}", input);

    match result {
        Expression::Operation { name, args } => {
            assert_eq!(name, "logical_and", "Top-level should be logical_and");

            // Both children should be comparisons
            match &args[0] {
                Expression::Operation { name, .. } => assert_eq!(name, "greater_than"),
                _ => panic!("Left should be greater_than"),
            }

            match &args[1] {
                Expression::Operation { name, .. } => assert_eq!(name, "less_than"),
                _ => panic!("Right should be less_than"),
            }
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: arithmetic, comparison, logical combined");
}

// =============================================================================
// CONDITIONAL PRECEDENCE TESTS
// =============================================================================

#[test]
fn test_conditional_condition_precedence() {
    // if a * b >= c then 1 else 0
    let input = "if a * b >= c then 1 else 0";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing conditional: {}", input);

    match result {
        Expression::Conditional { condition, .. } => {
            // Condition should be geq(times(a, b), c)
            match condition.as_ref() {
                Expression::Operation { name, args } => {
                    assert_eq!(name, "geq", "Condition should be geq");
                    match &args[0] {
                        Expression::Operation { name, .. } => assert_eq!(name, "times"),
                        _ => panic!("Left of geq should be times"),
                    }
                }
                _ => panic!("Condition should be Operation"),
            }
        }
        _ => panic!("Expected Conditional"),
    }
    println!("✅ Passed: conditional respects precedence in condition");
}

#[test]
fn test_nested_conditional_precedence() {
    // if a >= b then 1 else if c * 2 <= d then 2 else 0
    let input = "if a >= b then 1 else if c * 2 <= d then 2 else 0";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing nested conditional: {}", input);

    match result {
        Expression::Conditional { else_branch, .. } => {
            // Else branch should be another conditional
            match else_branch.as_ref() {
                Expression::Conditional { condition, .. } => {
                    // Inner condition: c * 2 <= d
                    match condition.as_ref() {
                        Expression::Operation { name, args } => {
                            assert_eq!(name, "leq");
                            match &args[0] {
                                Expression::Operation { name, .. } => assert_eq!(name, "times"),
                                _ => panic!("Left of leq should be times"),
                            }
                        }
                        _ => panic!("Inner condition should be Operation"),
                    }
                }
                _ => panic!("Else branch should be Conditional"),
            }
        }
        _ => panic!("Expected Conditional"),
    }
    println!("✅ Passed: nested conditional precedence");
}

// =============================================================================
// PARENTHESES OVERRIDE TESTS
// =============================================================================

#[test]
fn test_parentheses_override_precedence() {
    // (a + b) * c should parse as times(plus(a, b), c)
    let input = "(a + b) * c";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing: {} -> times(plus(a,b), c)", input);

    match result {
        Expression::Operation { name, args } => {
            assert_eq!(name, "times", "Top-level should be times");

            // Left should be plus(a, b)
            match &args[0] {
                Expression::Operation { name, .. } => assert_eq!(name, "plus"),
                _ => panic!("Left should be plus(a, b)"),
            }
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: parentheses override * over + precedence");
}

#[test]
fn test_parentheses_in_comparison() {
    // a * (b + c) >= d should parse as geq(times(a, plus(b, c)), d)
    let input = "a * (b + c) >= d";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing: {}", input);

    match result {
        Expression::Operation { name, args } => {
            assert_eq!(name, "geq");

            match &args[0] {
                Expression::Operation { name, args } => {
                    assert_eq!(name, "times");
                    match &args[1] {
                        Expression::Operation { name, .. } => assert_eq!(name, "plus"),
                        _ => panic!("Should be plus inside parens"),
                    }
                }
                _ => panic!("Left should be times"),
            }
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: parentheses in comparison");
}

// =============================================================================
// FUNCTION CALL PRECEDENCE
// =============================================================================

#[test]
fn test_function_call_binds_tightest() {
    // f(x) + g(y) * h(z) should parse as f(x) + (g(y) * h(z))
    let input = "f(x) + g(y) * h(z)";
    let mut parser = KleisParser::new(input);
    let result = parser.parse().unwrap();

    println!("\n🔍 Testing: {}", input);

    match result {
        Expression::Operation { name, .. } => {
            assert_eq!(name, "plus", "Top-level should be plus");
            // The structure should be plus(f(x), times(g(y), h(z)))
        }
        _ => panic!("Expected Operation"),
    }
    println!("✅ Passed: function calls bind tightest");
}

// =============================================================================
// UNICODE VS ASCII OPERATOR EQUIVALENCE
// =============================================================================

#[test]
fn test_unicode_leq_same_as_ascii() {
    let input1 = "a ≤ b";
    let input2 = "a <= b";

    let mut parser1 = KleisParser::new(input1);
    let mut parser2 = KleisParser::new(input2);

    let result1 = parser1.parse().unwrap();
    let result2 = parser2.parse().unwrap();

    match (&result1, &result2) {
        (
            Expression::Operation {
                name: name1,
                args: args1,
            },
            Expression::Operation {
                name: name2,
                args: args2,
            },
        ) => {
            assert_eq!(name1, name2, "≤ and <= should produce same operator");
            assert_eq!(args1.len(), args2.len());
        }
        _ => panic!("Both should be Operations"),
    }
    println!("✅ Passed: ≤ and <= are equivalent");
}

#[test]
fn test_unicode_geq_same_as_ascii() {
    let input1 = "a ≥ b";
    let input2 = "a >= b";

    let mut parser1 = KleisParser::new(input1);
    let mut parser2 = KleisParser::new(input2);

    let result1 = parser1.parse().unwrap();
    let result2 = parser2.parse().unwrap();

    match (&result1, &result2) {
        (
            Expression::Operation {
                name: name1,
                args: args1,
            },
            Expression::Operation {
                name: name2,
                args: args2,
            },
        ) => {
            assert_eq!(name1, name2, "≥ and >= should produce same operator");
            assert_eq!(args1.len(), args2.len());
        }
        _ => panic!("Both should be Operations"),
    }
    println!("✅ Passed: ≥ and >= are equivalent");
}
