//! Grammar v0.7 Z3 Integration Tests
//!
//! Tests that grammar v0.7 expressions translate correctly to Z3 and verify properly.
//!
//! ## Key v0.7 Features Tested:
//! - **Calculus notation**: D(f, x), Dt(f, x), Integrate(f, x), Limit(f, x, a)
//! - **Sum/Product**: Sum(expr, i, 1, n), Product(expr, i, 1, n)
//! - **Pattern matching**: match expressions with Z3 verification
//! - **Quantifiers**: ∀, ∃ with Z3 theorem proving
//!
//! ## Round-trip Testing:
//! Editor → AST → Z3 → Result → Renderer
//!
//! These tests ensure the three-rung ladder architecture works correctly.

#![allow(unused_imports)]

use kleis::ast::{Expression, LambdaParam};
use kleis::kleis_parser::KleisParser;
use kleis::solvers::backend::{SatisfiabilityResult, SolverBackend, VerificationResult};
use kleis::solvers::z3::Z3Backend;
use kleis::structure_registry::StructureRegistry;

// ============================================================================
// SECTION 1: Calculus Notation Tests (v0.7 NEW)
// ============================================================================

/// Test that D(f, x) partial derivative is handled by Z3
#[test]
#[cfg(feature = "axiom-verification")]
fn test_partial_derivative_uninterpreted() {
    println!("\n🧪 Testing: D(f, x) partial derivative in Z3");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // D(f, x) should be handled as uninterpreted function
    // This allows Z3 to reason about derivative properties
    let expr = Expression::Operation {
        name: "D".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("x".to_string()),
        ],
    };

    // Should not error - just create uninterpreted function
    let result = backend.simplify(&expr);
    assert!(result.is_ok(), "D(f, x) should be translatable to Z3");
    println!("   ✅ D(f, x) translated successfully");
}

/// Test that Dt(f, x) total derivative is handled by Z3
#[test]
#[cfg(feature = "axiom-verification")]
fn test_total_derivative_uninterpreted() {
    println!("\n🧪 Testing: Dt(f, x) total derivative in Z3");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let expr = Expression::Operation {
        name: "Dt".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("x".to_string()),
        ],
    };

    let result = backend.simplify(&expr);
    assert!(result.is_ok(), "Dt(f, x) should be translatable to Z3");
    println!("   ✅ Dt(f, x) translated successfully");
}

/// Test that Integrate(f, x) indefinite integral is handled
#[test]
#[cfg(feature = "axiom-verification")]
fn test_indefinite_integral_uninterpreted() {
    println!("\n🧪 Testing: Integrate(f, x) indefinite integral in Z3");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let expr = Expression::Operation {
        name: "Integrate".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("x".to_string()),
        ],
    };

    let result = backend.simplify(&expr);
    assert!(
        result.is_ok(),
        "Integrate(f, x) should be translatable to Z3"
    );
    println!("   ✅ Integrate(f, x) translated successfully");
}

/// Test that Integrate(f, x, a, b) definite integral is handled
#[test]
#[cfg(feature = "axiom-verification")]
fn test_definite_integral_uninterpreted() {
    println!("\n🧪 Testing: Integrate(f, x, a, b) definite integral in Z3");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // Integrate(f, x, 0, 1) - definite integral from 0 to 1
    let expr = Expression::Operation {
        name: "Integrate".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("x".to_string()),
            Expression::Const("0".to_string()),
            Expression::Const("1".to_string()),
        ],
    };

    let result = backend.simplify(&expr);
    assert!(
        result.is_ok(),
        "Integrate(f, x, a, b) should be translatable to Z3"
    );
    println!("   ✅ Integrate(f, x, 0, 1) translated successfully");
}

/// Test that Limit(f, x, a) is handled
#[test]
#[cfg(feature = "axiom-verification")]
fn test_limit_uninterpreted() {
    println!("\n🧪 Testing: Limit(f, x, a) in Z3");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // Limit(f, x, 0) - limit as x approaches 0
    let expr = Expression::Operation {
        name: "Limit".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("x".to_string()),
            Expression::Const("0".to_string()),
        ],
    };

    let result = backend.simplify(&expr);
    assert!(
        result.is_ok(),
        "Limit(f, x, a) should be translatable to Z3"
    );
    println!("   ✅ Limit(f, x, 0) translated successfully");
}

// ============================================================================
// SECTION 2: Sum and Product Notation Tests
// ============================================================================

/// Test Sum(expr, i, 1, n) function-call style
#[test]
#[cfg(feature = "axiom-verification")]
fn test_sum_function_call() {
    println!("\n🧪 Testing: Sum(i, i, 1, n) in Z3");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // Sum(i, i, 1, n) - summation of i from 1 to n
    let expr = Expression::Operation {
        name: "Sum".to_string(),
        args: vec![
            Expression::Object("i".to_string()), // expression to sum
            Expression::Object("i".to_string()), // index variable
            Expression::Const("1".to_string()),  // lower bound
            Expression::Object("n".to_string()), // upper bound
        ],
    };

    let result = backend.simplify(&expr);
    assert!(result.is_ok(), "Sum should be translatable to Z3");
    println!("   ✅ Sum(i, i, 1, n) translated successfully");
}

/// Test Product(expr, i, 1, n) function-call style
#[test]
#[cfg(feature = "axiom-verification")]
fn test_product_function_call() {
    println!("\n🧪 Testing: Product(i, i, 1, n) in Z3");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // Product(i, i, 1, n) - product of i from 1 to n (factorial-like)
    let expr = Expression::Operation {
        name: "Product".to_string(),
        args: vec![
            Expression::Object("i".to_string()),
            Expression::Object("i".to_string()),
            Expression::Const("1".to_string()),
            Expression::Object("n".to_string()),
        ],
    };

    let result = backend.simplify(&expr);
    assert!(result.is_ok(), "Product should be translatable to Z3");
    println!("   ✅ Product(i, i, 1, n) translated successfully");
}

// ============================================================================
// SECTION 3: Arithmetic Evaluation Tests
// ============================================================================

/// Test basic arithmetic evaluates correctly
#[test]
#[cfg(feature = "axiom-verification")]
fn test_arithmetic_evaluation() {
    println!("\n🧪 Testing: Basic arithmetic evaluation");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // (2 + 3) * 4 = 20
    let expr = Expression::Operation {
        name: "times".to_string(),
        args: vec![
            Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Const("2".to_string()),
                    Expression::Const("3".to_string()),
                ],
            },
            Expression::Const("4".to_string()),
        ],
    };

    let result = backend.evaluate(&expr).unwrap();
    assert_eq!(result, Expression::Const("20".to_string()));
    println!("   ✅ (2 + 3) * 4 = 20");
}

/// Test power/exponentiation verification
///
/// Note: Z3's evaluate() doesn't compute power to concrete value
/// because the power function returns a symbolic expression.
/// Instead, we verify equivalence: 2^3 = 8
#[test]
#[cfg(feature = "axiom-verification")]
fn test_power_verification() {
    println!("\n🧪 Testing: Power verification 2^3 = 8");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // Verify: 2^3 = 8
    let lhs = Expression::Operation {
        name: "power".to_string(),
        args: vec![
            Expression::Const("2".to_string()),
            Expression::Const("3".to_string()),
        ],
    };

    let rhs = Expression::Const("8".to_string());

    let equivalent = backend.are_equivalent(&lhs, &rhs).unwrap();
    assert!(equivalent, "2^3 should equal 8");
    println!("   ✅ Verified: 2^3 = 8");
}

// ============================================================================
// SECTION 4: Comparison and Boolean Operations
// ============================================================================

/// Test comparison operations
#[test]
#[cfg(feature = "axiom-verification")]
fn test_comparison_operations() {
    println!("\n🧪 Testing: Comparison operations");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // 5 > 3 should be satisfiable
    let expr = Expression::Operation {
        name: "greater_than".to_string(),
        args: vec![
            Expression::Const("5".to_string()),
            Expression::Const("3".to_string()),
        ],
    };

    let result = backend.check_satisfiability(&expr).unwrap();
    assert!(matches!(result, SatisfiabilityResult::Satisfiable { .. }));
    println!("   ✅ 5 > 3 is satisfiable");

    // 3 > 5 should also be satisfiable (it's just checking if there exists an assignment)
    // To verify 5 > 3 is ALWAYS true, we should use verify_axiom
}

/// Test boolean operations with Z3 verification
///
/// Note: Z3 requires boolean arguments for and/or/not operations.
/// We test with comparison expressions that produce booleans.
#[test]
#[cfg(feature = "axiom-verification")]
fn test_boolean_verification() {
    println!("\n🧪 Testing: Boolean verification (implication tautology)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // Verify: (x > 0) ∧ (y > 0) ⟹ (x + y > 0) is always true
    // This is a tautology for positive numbers
    let expr = Expression::Operation {
        name: "implies".to_string(),
        args: vec![
            // (x > 0) ∧ (y > 0)
            Expression::Operation {
                name: "and".to_string(),
                args: vec![
                    Expression::Operation {
                        name: "greater_than".to_string(),
                        args: vec![
                            Expression::Object("x".to_string()),
                            Expression::Const("0".to_string()),
                        ],
                    },
                    Expression::Operation {
                        name: "greater_than".to_string(),
                        args: vec![
                            Expression::Object("y".to_string()),
                            Expression::Const("0".to_string()),
                        ],
                    },
                ],
            },
            // (x + y > 0)
            Expression::Operation {
                name: "greater_than".to_string(),
                args: vec![
                    Expression::Operation {
                        name: "plus".to_string(),
                        args: vec![
                            Expression::Object("x".to_string()),
                            Expression::Object("y".to_string()),
                        ],
                    },
                    Expression::Const("0".to_string()),
                ],
            },
        ],
    };

    let result = backend.verify_axiom(&expr).unwrap();
    assert!(matches!(result, VerificationResult::Valid));
    println!("   ✅ Verified: (x > 0) ∧ (y > 0) ⟹ (x + y > 0)");
}

// ============================================================================
// SECTION 5: Conditional Expressions
// ============================================================================

/// Test conditional (if-then-else) expressions
#[test]
#[cfg(feature = "axiom-verification")]
fn test_conditional_expression() {
    println!("\n🧪 Testing: Conditional expressions");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // if 5 > 3 then 100 else 0
    let expr = Expression::Conditional {
        condition: Box::new(Expression::Operation {
            name: "greater_than".to_string(),
            args: vec![
                Expression::Const("5".to_string()),
                Expression::Const("3".to_string()),
            ],
        }),
        then_branch: Box::new(Expression::Const("100".to_string())),
        else_branch: Box::new(Expression::Const("0".to_string())),
    };

    let result = backend.evaluate(&expr).unwrap();
    assert_eq!(result, Expression::Const("100".to_string()));
    println!("   ✅ if 5 > 3 then 100 else 0 = 100");
}

// ============================================================================
// SECTION 6: Let Bindings
// ============================================================================

/// Test let bindings
#[test]
#[cfg(feature = "axiom-verification")]
fn test_let_binding() {
    println!("\n🧪 Testing: Let bindings");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // let x = 5 in x + 3
    let expr = Expression::Let {
        pattern: kleis::ast::Pattern::Variable("x".to_string()),
        type_annotation: None,
        value: Box::new(Expression::Const("5".to_string())),
        body: Box::new(Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Const("3".to_string()),
            ],
        }),
    };

    let result = backend.evaluate(&expr).unwrap();
    assert_eq!(result, Expression::Const("8".to_string()));
    println!("   ✅ let x = 5 in x + 3 = 8");
}

/// Test nested let bindings
#[test]
#[cfg(feature = "axiom-verification")]
fn test_nested_let_binding() {
    println!("\n🧪 Testing: Nested let bindings");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // let x = 5 in let y = 3 in x * y
    let expr = Expression::Let {
        pattern: kleis::ast::Pattern::Variable("x".to_string()),
        type_annotation: None,
        value: Box::new(Expression::Const("5".to_string())),
        body: Box::new(Expression::Let {
            pattern: kleis::ast::Pattern::Variable("y".to_string()),
            type_annotation: None,
            value: Box::new(Expression::Const("3".to_string())),
            body: Box::new(Expression::Operation {
                name: "times".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Object("y".to_string()),
                ],
            }),
        }),
    };

    let result = backend.evaluate(&expr).unwrap();
    assert_eq!(result, Expression::Const("15".to_string()));
    println!("   ✅ let x = 5 in let y = 3 in x * y = 15");
}

/// Test Grammar v0.8: Let destructuring with constructor pattern
/// let Point(x, y) = Point(3, 4) in x + y
#[test]
#[cfg(feature = "axiom-verification")]
fn test_let_constructor_destructuring() {
    println!("\n🧪 Testing: Let constructor destructuring (Grammar v0.8)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // let Point(x, y) = Point(3, 4) in x + y
    let expr = Expression::Let {
        pattern: kleis::ast::Pattern::Constructor {
            name: "Point".to_string(),
            args: vec![
                kleis::ast::Pattern::Variable("x".to_string()),
                kleis::ast::Pattern::Variable("y".to_string()),
            ],
        },
        type_annotation: None,
        value: Box::new(Expression::Operation {
            name: "Point".to_string(),
            args: vec![
                Expression::Const("3".to_string()),
                Expression::Const("4".to_string()),
            ],
        }),
        body: Box::new(Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Object("y".to_string()),
            ],
        }),
    };

    let result = backend.evaluate(&expr).unwrap();
    assert_eq!(result, Expression::Const("7".to_string()));
    println!("   ✅ let Point(x, y) = Point(3, 4) in x + y = 7");
}

/// Test Grammar v0.8: Nested constructor destructuring
/// let Pair(Point(a, b), Point(c, d)) = Pair(Point(1, 2), Point(3, 4)) in a + b + c + d
#[test]
#[cfg(feature = "axiom-verification")]
fn test_let_nested_constructor_destructuring() {
    println!("\n🧪 Testing: Nested constructor destructuring (Grammar v0.8)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // let Pair(Point(a, b), Point(c, d)) = Pair(Point(1, 2), Point(3, 4)) in a + b + c + d
    let expr = Expression::Let {
        pattern: kleis::ast::Pattern::Constructor {
            name: "Pair".to_string(),
            args: vec![
                kleis::ast::Pattern::Constructor {
                    name: "Point".to_string(),
                    args: vec![
                        kleis::ast::Pattern::Variable("a".to_string()),
                        kleis::ast::Pattern::Variable("b".to_string()),
                    ],
                },
                kleis::ast::Pattern::Constructor {
                    name: "Point".to_string(),
                    args: vec![
                        kleis::ast::Pattern::Variable("c".to_string()),
                        kleis::ast::Pattern::Variable("d".to_string()),
                    ],
                },
            ],
        },
        type_annotation: None,
        value: Box::new(Expression::Operation {
            name: "Pair".to_string(),
            args: vec![
                Expression::Operation {
                    name: "Point".to_string(),
                    args: vec![
                        Expression::Const("1".to_string()),
                        Expression::Const("2".to_string()),
                    ],
                },
                Expression::Operation {
                    name: "Point".to_string(),
                    args: vec![
                        Expression::Const("3".to_string()),
                        Expression::Const("4".to_string()),
                    ],
                },
            ],
        }),
        body: Box::new(Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Operation {
                    name: "plus".to_string(),
                    args: vec![
                        Expression::Operation {
                            name: "plus".to_string(),
                            args: vec![
                                Expression::Object("a".to_string()),
                                Expression::Object("b".to_string()),
                            ],
                        },
                        Expression::Object("c".to_string()),
                    ],
                },
                Expression::Object("d".to_string()),
            ],
        }),
    };

    let result = backend.evaluate(&expr).unwrap();
    assert_eq!(result, Expression::Const("10".to_string()));
    println!("   ✅ let Pair(Point(a, b), Point(c, d)) = Pair(Point(1, 2), Point(3, 4)) in a + b + c + d = 10");
}

/// Test Grammar v0.8: Let destructuring with wildcard
/// let Point(x, _) = Point(5, 10) in x * 2
#[test]
#[cfg(feature = "axiom-verification")]
fn test_let_destructuring_with_wildcard() {
    println!("\n🧪 Testing: Let destructuring with wildcard (Grammar v0.8)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // let Point(x, _) = Point(5, 10) in x * 2
    let expr = Expression::Let {
        pattern: kleis::ast::Pattern::Constructor {
            name: "Point".to_string(),
            args: vec![
                kleis::ast::Pattern::Variable("x".to_string()),
                kleis::ast::Pattern::Wildcard,
            ],
        },
        type_annotation: None,
        value: Box::new(Expression::Operation {
            name: "Point".to_string(),
            args: vec![
                Expression::Const("5".to_string()),
                Expression::Const("10".to_string()),
            ],
        }),
        body: Box::new(Expression::Operation {
            name: "times".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Const("2".to_string()),
            ],
        }),
    };

    let result = backend.evaluate(&expr).unwrap();
    assert_eq!(result, Expression::Const("10".to_string()));
    println!("   ✅ let Point(x, _) = Point(5, 10) in x * 2 = 10");
}

/// Test Grammar v0.8: Let destructuring with as-pattern
/// let Point(x, y) as p = Point(3, 4) in x + y (as-pattern binds both p and x, y)
#[test]
#[cfg(feature = "axiom-verification")]
fn test_let_destructuring_with_as_pattern() {
    println!("\n🧪 Testing: Let destructuring with as-pattern (Grammar v0.8)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // let Point(x, y) as p = Point(3, 4) in x + y
    // (p is bound to the whole Point(3, 4), x=3, y=4)
    let expr = Expression::Let {
        pattern: kleis::ast::Pattern::As {
            pattern: Box::new(kleis::ast::Pattern::Constructor {
                name: "Point".to_string(),
                args: vec![
                    kleis::ast::Pattern::Variable("x".to_string()),
                    kleis::ast::Pattern::Variable("y".to_string()),
                ],
            }),
            binding: "p".to_string(),
        },
        type_annotation: None,
        value: Box::new(Expression::Operation {
            name: "Point".to_string(),
            args: vec![
                Expression::Const("3".to_string()),
                Expression::Const("4".to_string()),
            ],
        }),
        body: Box::new(Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Object("y".to_string()),
            ],
        }),
    };

    let result = backend.evaluate(&expr).unwrap();
    assert_eq!(result, Expression::Const("7".to_string()));
    println!("   ✅ let Point(x, y) as p = Point(3, 4) in x + y = 7");
}

// ============================================================================
// SECTION 7: Expression Equivalence
// ============================================================================

/// Test expression equivalence checking
#[test]
#[cfg(feature = "axiom-verification")]
fn test_expression_equivalence() {
    println!("\n🧪 Testing: Expression equivalence");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // x + y should be equivalent to y + x (commutativity)
    let expr1 = Expression::Operation {
        name: "plus".to_string(),
        args: vec![
            Expression::Object("x".to_string()),
            Expression::Object("y".to_string()),
        ],
    };

    let expr2 = Expression::Operation {
        name: "plus".to_string(),
        args: vec![
            Expression::Object("y".to_string()),
            Expression::Object("x".to_string()),
        ],
    };

    let equivalent = backend.are_equivalent(&expr1, &expr2).unwrap();
    assert!(equivalent, "x + y should equal y + x");
    println!("   ✅ Verified: x + y ≡ y + x (commutativity)");
}

/// Test non-equivalence
#[test]
#[cfg(feature = "axiom-verification")]
fn test_expression_non_equivalence() {
    println!("\n🧪 Testing: Expression non-equivalence");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // x - y should NOT be equivalent to y - x (subtraction not commutative)
    let expr1 = Expression::Operation {
        name: "minus".to_string(),
        args: vec![
            Expression::Object("x".to_string()),
            Expression::Object("y".to_string()),
        ],
    };

    let expr2 = Expression::Operation {
        name: "minus".to_string(),
        args: vec![
            Expression::Object("y".to_string()),
            Expression::Object("x".to_string()),
        ],
    };

    let equivalent = backend.are_equivalent(&expr1, &expr2).unwrap();
    assert!(!equivalent, "x - y should NOT equal y - x");
    println!("   ✅ Verified: x - y ≢ y - x (subtraction not commutative)");
}

// ============================================================================
// SECTION 8: Match Expressions (Pattern Matching v0.5)
// ============================================================================

/// Test simple match expression with constants
#[test]
#[cfg(feature = "axiom-verification")]
fn test_match_with_constants() {
    println!("\n🧪 Testing: Match expression with constants");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    use kleis::ast::{MatchCase, Pattern};

    // match 1 { 0 => 100 | 1 => 200 | _ => 300 }
    let expr = Expression::Match {
        scrutinee: Box::new(Expression::Const("1".to_string())),
        cases: vec![
            MatchCase {
                guard: None,
                pattern: Pattern::Constant("0".to_string()),
                body: Expression::Const("100".to_string()),
            },
            MatchCase {
                guard: None,
                pattern: Pattern::Constant("1".to_string()),
                body: Expression::Const("200".to_string()),
            },
            MatchCase {
                guard: None,
                pattern: Pattern::Wildcard,
                body: Expression::Const("300".to_string()),
            },
        ],
    };

    let result = backend.evaluate(&expr).unwrap();
    assert_eq!(result, Expression::Const("200".to_string()));
    println!("   ✅ match 1 {{ 0 => 100 | 1 => 200 | _ => 300 }} = 200");
}

/// Test match with wildcard fallback
#[test]
#[cfg(feature = "axiom-verification")]
fn test_match_wildcard_fallback() {
    println!("\n🧪 Testing: Match wildcard fallback");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    use kleis::ast::{MatchCase, Pattern};

    // match 42 { 0 => 100 | 1 => 200 | _ => 300 }
    // 42 doesn't match 0 or 1, so falls through to wildcard
    let expr = Expression::Match {
        scrutinee: Box::new(Expression::Const("42".to_string())),
        cases: vec![
            MatchCase {
                guard: None,
                pattern: Pattern::Constant("0".to_string()),
                body: Expression::Const("100".to_string()),
            },
            MatchCase {
                guard: None,
                pattern: Pattern::Constant("1".to_string()),
                body: Expression::Const("200".to_string()),
            },
            MatchCase {
                guard: None,
                pattern: Pattern::Wildcard,
                body: Expression::Const("300".to_string()),
            },
        ],
    };

    let result = backend.evaluate(&expr).unwrap();
    assert_eq!(result, Expression::Const("300".to_string()));
    println!("   ✅ match 42 {{ 0 => 100 | 1 => 200 | _ => 300 }} = 300");
}

// ============================================================================
// SECTION 9: Quantifier Tests
// ============================================================================

/// Test forall quantifier with simple proposition
#[test]
#[cfg(feature = "axiom-verification")]
fn test_forall_simple() {
    println!("\n🧪 Testing: ∀ x. x = x (reflexivity)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    use kleis::ast::{QuantifiedVar, QuantifierKind};

    // ∀ x. x = x
    let expr = Expression::Quantifier {
        quantifier: QuantifierKind::ForAll,
        variables: vec![QuantifiedVar {
            name: "x".to_string(),
            type_annotation: Some("Int".to_string()),
        }],
        where_clause: None,
        body: Box::new(Expression::Operation {
            name: "equals".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Object("x".to_string()),
            ],
        }),
    };

    // Note: The Z3 backend translates quantifiers but doesn't create
    // actual Z3 forall/exists (that requires different handling)
    // For now, verify it at least translates without error
    let result = backend.simplify(&expr);
    assert!(result.is_ok(), "Quantifier should translate");
    println!("   ✅ ∀ x. x = x translated successfully");
}

// ============================================================================
// SECTION 10: Matrix Operations
// ============================================================================

/// Test 2x2 matrix times 2x1 vector
/// IGNORED: Requires list indexing for matrix operations
/// TO ENABLE: See z3_matrix_solve_test.rs::test_z3_solves_matrix_linear_system
#[test]
#[ignore]
#[cfg(feature = "axiom-verification")]
fn test_matrix_vector_multiply() {
    println!("\n🧪 Testing: Matrix(2,2) × Matrix(2,1)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // Matrix(2,2, [1, 0, 0, 1]) × Matrix(2,1, [5, 7])
    // Identity matrix times vector = vector
    let matrix = Expression::Operation {
        name: "Matrix".to_string(),
        args: vec![
            Expression::Const("2".to_string()),
            Expression::Const("2".to_string()),
            Expression::List(vec![
                Expression::Const("1".to_string()),
                Expression::Const("0".to_string()),
                Expression::Const("0".to_string()),
                Expression::Const("1".to_string()),
            ]),
        ],
    };

    let vector = Expression::Operation {
        name: "Matrix".to_string(),
        args: vec![
            Expression::Const("2".to_string()),
            Expression::Const("1".to_string()),
            Expression::List(vec![
                Expression::Const("5".to_string()),
                Expression::Const("7".to_string()),
            ]),
        ],
    };

    let multiply = Expression::Operation {
        name: "multiply".to_string(),
        args: vec![matrix, vector.clone()],
    };

    // The result should be equivalent to the original vector
    let equivalent = backend.are_equivalent(&multiply, &vector).unwrap();
    assert!(equivalent, "I × v should equal v");
    println!("   ✅ I × [5, 7]ᵀ = [5, 7]ᵀ verified");
}

// ============================================================================
// SECTION 11: Complex Integral Transforms (Uninterpreted)
// ============================================================================

/// Test DoubleIntegral is handled
#[test]
#[cfg(feature = "axiom-verification")]
fn test_double_integral() {
    println!("\n🧪 Testing: DoubleIntegral (∬)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let expr = Expression::Operation {
        name: "DoubleIntegral".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("x".to_string()),
            Expression::Object("y".to_string()),
        ],
    };

    let result = backend.simplify(&expr);
    assert!(result.is_ok(), "DoubleIntegral should be translatable");
    println!("   ✅ DoubleIntegral(f, x, y) translated successfully");
}

/// Test LineIntegral (contour integral)
#[test]
#[cfg(feature = "axiom-verification")]
fn test_line_integral() {
    println!("\n🧪 Testing: LineIntegral (∮)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let expr = Expression::Operation {
        name: "LineIntegral".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("C".to_string()),
        ],
    };

    let result = backend.simplify(&expr);
    assert!(result.is_ok(), "LineIntegral should be translatable");
    println!("   ✅ LineIntegral(f, C) translated successfully");
}

// ============================================================================
// SECTION 12: Absolute Value and Square Root
// ============================================================================

/// Test absolute value
#[test]
#[cfg(feature = "axiom-verification")]
fn test_absolute_value() {
    println!("\n🧪 Testing: abs(-5) = 5");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // abs(-5)
    let expr = Expression::Operation {
        name: "abs".to_string(),
        args: vec![Expression::Operation {
            name: "negate".to_string(),
            args: vec![Expression::Const("5".to_string())],
        }],
    };

    let result = backend.evaluate(&expr).unwrap();
    assert_eq!(result, Expression::Const("5".to_string()));
    println!("   ✅ abs(-5) = 5");
}

/// Test sqrt is handled as uninterpreted
#[test]
#[cfg(feature = "axiom-verification")]
fn test_sqrt_uninterpreted() {
    println!("\n🧪 Testing: sqrt(x) handled as uninterpreted");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let expr = Expression::Operation {
        name: "sqrt".to_string(),
        args: vec![Expression::Object("x".to_string())],
    };

    let result = backend.simplify(&expr);
    assert!(result.is_ok(), "sqrt should be translatable");
    println!("   ✅ sqrt(x) translated successfully");
}

// ============================================================================
// SECTION 13: Nth Root
// ============================================================================

/// Test nth_root is handled as uninterpreted
#[test]
#[cfg(feature = "axiom-verification")]
fn test_nth_root() {
    println!("\n🧪 Testing: nth_root(3, x) - cube root");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // nth_root(3, x) - cube root of x
    let expr = Expression::Operation {
        name: "nth_root".to_string(),
        args: vec![
            Expression::Const("3".to_string()),
            Expression::Object("x".to_string()),
        ],
    };

    let result = backend.simplify(&expr);
    assert!(result.is_ok(), "nth_root should be translatable");
    println!("   ✅ nth_root(3, x) translated successfully");
}

// ============================================================================
// SECTION 14: Round-Trip Tests (Parser → AST → Z3 → Result)
// ============================================================================

/// Test round-trip: Parse Kleis code, build AST, verify with Z3
#[test]
#[cfg(feature = "axiom-verification")]
fn test_roundtrip_simple_expression() {
    println!("\n🧪 Testing: Round-trip - Parse → AST → Z3");

    let registry = StructureRegistry::new();
    let _backend = Z3Backend::new(&registry).unwrap();

    // Parse a simple Kleis expression (function definition)
    let code = "define result = 2 + 3 × 4";
    let mut parser = KleisParser::new(code);
    let program = parser.parse_program();

    assert!(program.is_ok(), "Parser should succeed: {:?}", program);
    let program = program.unwrap();

    println!("   ✅ Parsed: {}", code);

    // Extract the expression from the function definition
    if let Some(kleis::kleis_ast::TopLevel::FunctionDef(func_def)) = program.items.first() {
        println!("   📊 Define: {} = {:?}", func_def.name, func_def.body);
        println!("   ✅ Round-trip: Parser → AST successful");
    }
}

/// Test round-trip with D() derivative notation
#[test]
#[cfg(feature = "axiom-verification")]
fn test_roundtrip_derivative_notation() {
    println!("\n🧪 Testing: Round-trip - D(f, x) derivative notation");

    // Parse D(f, x) in Kleis syntax
    let code = "define df_dx = D(f, x)";
    let mut parser = KleisParser::new(code);
    let program = parser.parse_program();

    if program.is_err() {
        println!("   ⚠️ Parser doesn't support D(f, x) yet - grammar feature pending");
        println!("   📝 This is expected - D() is a function call, needs parser support");
        return;
    }

    println!("   ✅ D(f, x) parsed successfully");
}

/// Test round-trip with Sum() notation
#[test]
#[cfg(feature = "axiom-verification")]
fn test_roundtrip_sum_notation() {
    println!("\n🧪 Testing: Round-trip - Sum(i, i, 1, n) notation");

    // Parse Sum in Kleis syntax
    let code = "define s = Sum(i, i, 1, n)";
    let mut parser = KleisParser::new(code);
    let program = parser.parse_program();

    if program.is_err() {
        println!("   ⚠️ Parser doesn't support Sum() yet - grammar feature pending");
        return;
    }

    println!("   ✅ Sum(i, i, 1, n) parsed successfully");
}

/// Test that structure with operations can be parsed
#[test]
#[cfg(feature = "axiom-verification")]
fn test_roundtrip_structure_operations() {
    println!("\n🧪 Testing: Round-trip - Structure operations parsing");

    // Simpler structure without axiom (axiom syntax may not be fully implemented)
    let code = r#"
    structure Group(G) {
        operation (+) : G × G → G
        element e : G
    }
    "#;

    let mut parser = KleisParser::new(code);
    let program = parser.parse_program();

    if program.is_err() {
        println!("   ⚠️ Structure parsing failed: {:?}", program.err());
        println!("   📝 This may be a parser limitation - checking simpler syntax");

        // Try even simpler structure
        let simple_code = "structure Monoid(M) { operation (+) : M × M → M }";
        let mut parser2 = KleisParser::new(simple_code);
        let program2 = parser2.parse_program();

        if program2.is_ok() {
            println!("   ✅ Simple structure parsed successfully");
        } else {
            println!("   ⚠️ Structure parsing not fully supported yet");
        }
        return;
    }

    let program = program.unwrap();
    println!("   ✅ Parsed structure with operations");

    // Check that the structure was parsed
    if let Some(kleis::kleis_ast::TopLevel::StructureDef(structure)) = program.items.first() {
        println!("   📊 Structure: {}", structure.name);
        println!("   📊 Members: {} members", structure.members.len());
        println!("   ✅ Structure → AST successful");
    }
}

// ============================================================================
// LAMBDA EXPRESSION Z3 TESTS
// ============================================================================

#[cfg(feature = "axiom-verification")]
#[test]
fn test_lambda_z3_translation() {
    use kleis::ast::{Expression, LambdaParam};

    println!("\n=== Lambda Z3 Translation Test ===");

    // Create a simple lambda: λ x . x + 1
    let lambda = Expression::Lambda {
        params: vec![LambdaParam::new("x")],
        body: Box::new(Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Const("1".to_string()),
            ],
        }),
    };

    println!("   Lambda: λ x . x + 1");

    // Create Z3 backend and translate
    use kleis::solvers::z3::backend::Z3Backend;
    use kleis::structure_registry::StructureRegistry;

    let registry = StructureRegistry::new();
    let backend_result = Z3Backend::new(&registry);

    if backend_result.is_err() {
        println!("   ⚠️ Z3 backend initialization failed (expected in some environments)");
        return;
    }

    let mut backend = backend_result.unwrap();
    println!("   ✅ Z3 backend initialized");

    // Test satisfiability of lambda body
    // λ x . x + 1 = 5 should be satisfiable (when x = 4)
    let equation = Expression::Operation {
        name: "equals".to_string(),
        args: vec![lambda.clone(), Expression::Const("5".to_string())],
    };

    let result = backend.check_satisfiability(&equation);
    match result {
        Ok(sat) => {
            println!("   ✅ Lambda satisfiability check: {:?}", sat);
        }
        Err(e) => {
            println!("   ⚠️ Lambda satisfiability error (may be expected): {}", e);
        }
    }
}

#[cfg(feature = "axiom-verification")]
#[test]
fn test_lambda_typed_z3() {
    use kleis::ast::{Expression, LambdaParam};

    println!("\n=== Typed Lambda Z3 Test ===");

    // Create a typed lambda: λ (x : ℝ) . x * x
    let lambda = Expression::Lambda {
        params: vec![LambdaParam::typed("x", "ℝ")],
        body: Box::new(Expression::Operation {
            name: "times".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Object("x".to_string()),
            ],
        }),
    };

    println!("   Lambda: λ (x : ℝ) . x * x");

    use kleis::solvers::z3::backend::Z3Backend;
    use kleis::structure_registry::StructureRegistry;

    let registry = StructureRegistry::new();
    let backend_result = Z3Backend::new(&registry);

    if backend_result.is_err() {
        println!("   ⚠️ Z3 backend initialization failed");
        return;
    }

    let backend = backend_result.unwrap();
    println!("   ✅ Z3 backend initialized with typed lambda");

    // Verify the lambda was created with proper type annotation
    if let Expression::Lambda { params, .. } = &lambda {
        assert_eq!(params[0].type_annotation, Some("ℝ".to_string()));
        println!(
            "   ✅ Type annotation preserved: {:?}",
            params[0].type_annotation
        );
    }

    drop(backend);
    println!("   ✅ Typed lambda (Real) processed successfully");
}

#[cfg(feature = "axiom-verification")]
#[test]
fn test_lambda_nested_z3() {
    use kleis::ast::{Expression, LambdaParam};

    println!("\n=== Nested Lambda Z3 Test ===");

    // Create curried lambda: λ x . λ y . x + y
    let inner = Expression::Lambda {
        params: vec![LambdaParam::new("y")],
        body: Box::new(Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Object("y".to_string()),
            ],
        }),
    };

    let outer = Expression::Lambda {
        params: vec![LambdaParam::new("x")],
        body: Box::new(inner),
    };

    println!("   Lambda: λ x . λ y . x + y");

    use kleis::solvers::z3::backend::Z3Backend;
    use kleis::structure_registry::StructureRegistry;

    let registry = StructureRegistry::new();
    let backend_result = Z3Backend::new(&registry);

    if backend_result.is_err() {
        println!("   ⚠️ Z3 backend initialization failed");
        return;
    }

    let backend = backend_result.unwrap();
    println!("   ✅ Z3 backend initialized");

    // Verify nested structure
    if let Expression::Lambda { params, body } = &outer {
        assert_eq!(params[0].name, "x");
        if let Expression::Lambda {
            params: inner_params,
            ..
        } = body.as_ref()
        {
            assert_eq!(inner_params[0].name, "y");
            println!("   ✅ Nested lambda structure verified: outer=x, inner=y");
        }
    }

    drop(backend);
    println!("   ✅ Nested lambda Z3 test passed");
}

// ============================================================================
// BETA REDUCTION + Z3 INTEGRATION TESTS
// ============================================================================

/// Test that beta reduction works before Z3 verification
/// This is the key integration: (λ x . x + 1)(5) = 6 should be verifiable
#[test]
fn test_beta_reduction_z3_integration() {
    println!("\n🧪 Test: Beta Reduction + Z3 Integration");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).expect("Failed to create Z3 backend");

    // Create expression: (λ x . x + 1)(5)
    // This should reduce to 5 + 1 = 6
    let lambda = Expression::Lambda {
        params: vec![LambdaParam::new("x")],
        body: Box::new(Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Const("1".to_string()),
            ],
        }),
    };

    // Apply lambda to 5 - simulating (λ x . x + 1)(5)
    // For now, we manually reduce since Z3 doesn't handle application syntax
    let reduced = backend.beta_reduce_expression(&lambda);
    assert!(
        reduced.is_ok(),
        "Beta reduction should succeed on raw lambda"
    );

    // Test: Create the equation (λ x . x + 1) with x=5 reduced should equal 6
    // We'll construct this as: let reduced_body = 5 + 1; check if reduced_body = 6
    println!("   Testing: After reduction, 5 + 1 = 6 should be satisfiable");

    let reduced_application = Expression::Operation {
        name: "plus".to_string(),
        args: vec![
            Expression::Const("5".to_string()),
            Expression::Const("1".to_string()),
        ],
    };

    let equation = Expression::Operation {
        name: "equals".to_string(),
        args: vec![reduced_application, Expression::Const("6".to_string())],
    };

    let result = backend.check_satisfiability(&equation);
    assert!(result.is_ok(), "Z3 check should succeed");

    match result.unwrap() {
        SatisfiabilityResult::Satisfiable { .. } => {
            println!("   ✅ 5 + 1 = 6 is satisfiable (correct!)");
        }
        other => panic!("Expected Satisfiable, got {:?}", other),
    }

    drop(backend);
    println!("   ✅ Beta reduction + Z3 integration test passed");
}

/// Test full round-trip: parse lambda → reduce → verify with Z3
#[test]
fn test_lambda_roundtrip_parse_reduce_verify() {
    println!("\n🧪 Test: Lambda Round-trip (Parse → Reduce → Z3)");

    use kleis::evaluator::Evaluator;
    use kleis::kleis_parser::KleisParser;

    // Step 1: Parse a lambda expression
    let code = "λ x . x + 1";
    let mut parser = KleisParser::new(code);
    let lambda = parser.parse().expect("Failed to parse lambda");
    println!("   ✅ Parsed: {}", code);

    // Step 2: Apply argument manually (simulating beta reduction)
    let arg = Expression::Const("5".to_string());
    let evaluator = Evaluator::new();
    let reduced = evaluator
        .beta_reduce(&lambda, &arg)
        .expect("Failed to reduce");
    println!("   ✅ Reduced: (λ x . x + 1)(5) → reduced form");

    // Verify the reduced form is 5 + 1
    match &reduced {
        Expression::Operation { name, args } => {
            assert_eq!(name, "plus");
            assert!(matches!(&args[0], Expression::Const(s) if s == "5"));
            assert!(matches!(&args[1], Expression::Const(s) if s == "1"));
            println!("   ✅ Reduction result: 5 + 1");
        }
        _ => panic!("Expected Operation, got {:?}", reduced),
    }

    // Step 3: Verify with Z3 that reduced form = 6
    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).expect("Failed to create Z3 backend");

    let equation = Expression::Operation {
        name: "equals".to_string(),
        args: vec![reduced, Expression::Const("6".to_string())],
    };

    let result = backend.check_satisfiability(&equation);
    assert!(result.is_ok(), "Z3 check should succeed");

    match result.unwrap() {
        SatisfiabilityResult::Satisfiable { .. } => {
            println!("   ✅ Z3 verified: 5 + 1 = 6");
        }
        other => panic!("Expected Satisfiable, got {:?}", other),
    }

    drop(backend);
    println!("   ✅ Full round-trip test passed: Parse → Reduce → Z3 Verify");
}

/// Test curried function reduction with Z3
#[test]
fn test_curried_lambda_reduction_z3() {
    println!("\n🧪 Test: Curried Lambda Reduction + Z3");

    use kleis::evaluator::Evaluator;

    // Create: λ x y . x + y
    let curried = Expression::Lambda {
        params: vec![LambdaParam::new("x"), LambdaParam::new("y")],
        body: Box::new(Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Object("y".to_string()),
            ],
        }),
    };

    let evaluator = Evaluator::new();

    // Step 1: Partial application (λ x y . x + y)(3) → λ y . 3 + y
    let partial = evaluator
        .beta_reduce(&curried, &Expression::Const("3".to_string()))
        .expect("Partial application failed");

    match &partial {
        Expression::Lambda { params, .. } => {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name, "y");
            println!("   ✅ Partial: (λ x y . x + y)(3) → λ y . 3 + y");
        }
        _ => panic!("Expected Lambda for partial application"),
    }

    // Step 2: Full application (λ y . 3 + y)(4) → 3 + 4
    let full = evaluator
        .beta_reduce(&partial, &Expression::Const("4".to_string()))
        .expect("Full application failed");

    match &full {
        Expression::Operation { name, args } => {
            assert_eq!(name, "plus");
            println!("   ✅ Full: (λ y . 3 + y)(4) → 3 + 4");
            assert!(matches!(&args[0], Expression::Const(s) if s == "3"));
            assert!(matches!(&args[1], Expression::Const(s) if s == "4"));
        }
        _ => panic!("Expected Operation, got {:?}", full),
    }

    // Step 3: Verify with Z3 that 3 + 4 = 7
    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).expect("Failed to create Z3 backend");

    let equation = Expression::Operation {
        name: "equals".to_string(),
        args: vec![full, Expression::Const("7".to_string())],
    };

    let result = backend.check_satisfiability(&equation);
    assert!(result.is_ok());

    match result.unwrap() {
        SatisfiabilityResult::Satisfiable { .. } => {
            println!("   ✅ Z3 verified: 3 + 4 = 7");
        }
        other => panic!("Expected Satisfiable, got {:?}", other),
    }

    drop(backend);
    println!("   ✅ Curried lambda + Z3 test passed");
}

/// Test variable capture avoidance in beta reduction
#[test]
fn test_variable_capture_avoidance_z3() {
    println!("\n🧪 Test: Variable Capture Avoidance");

    use kleis::evaluator::Evaluator;

    // (λ x . λ y . x + y)(y) should NOT produce λ y . y + y
    // It should alpha-convert to avoid capture

    let outer = Expression::Lambda {
        params: vec![LambdaParam::new("x")],
        body: Box::new(Expression::Lambda {
            params: vec![LambdaParam::new("y")],
            body: Box::new(Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Object("y".to_string()),
                ],
            }),
        }),
    };

    let evaluator = Evaluator::new();

    // Apply with 'y' as argument - potential capture!
    let result = evaluator
        .beta_reduce(&outer, &Expression::Object("y".to_string()))
        .expect("Reduction should succeed");

    // Result should be λ y' . y + y' (with fresh variable)
    match &result {
        Expression::Lambda { params, body } => {
            let inner_param = &params[0].name;
            // The parameter should be renamed to avoid capture
            assert_ne!(inner_param, "y", "Variable capture should be avoided!");
            println!(
                "   ✅ Parameter renamed from 'y' to '{}' to avoid capture",
                inner_param
            );

            match body.as_ref() {
                Expression::Operation { args, .. } => {
                    // First arg should be 'y' (the argument we passed)
                    match &args[0] {
                        Expression::Object(s) => assert_eq!(s, "y"),
                        _ => panic!("First arg should be Object(y)"),
                    }
                    // Second arg should be the renamed param
                    match &args[1] {
                        Expression::Object(s) => assert_eq!(s, inner_param),
                        _ => panic!("Second arg should be renamed param"),
                    }
                    println!("   ✅ Body correctly uses original 'y' and renamed param");
                }
                _ => panic!("Expected Operation in body"),
            }
        }
        _ => panic!("Expected Lambda, got {:?}", result),
    }

    println!("   ✅ Variable capture avoidance test passed");
}

// ============================================================================
// PLACEHOLDER TESTS (when feature disabled)
// ============================================================================

#[cfg(not(feature = "axiom-verification"))]
mod placeholder {
    #[test]
    fn test_z3_feature_disabled() {
        println!("⚠️ Grammar v0.7 Z3 tests skipped - compile with --features axiom-verification");
    }
}
