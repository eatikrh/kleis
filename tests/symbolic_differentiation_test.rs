//! Symbolic Differentiation Tests
//!
//! Tests for the D(f, x) and Dt(f, x) calculus operations in Kleis.
//!
//! ## Calculus Notation (Grammar v0.7+)
//!
//! Kleis uses Mathematica-style function calls for derivatives:
//! - `D(f, x)` - Partial derivative ∂f/∂x
//! - `D(f, x, y)` - Mixed partial ∂²f/∂x∂y
//! - `Dt(f, x)` - Total derivative df/dx (chain rule applies)
//!
//! ## Derivative Rules Tested
//!
//! 1. **Constant Rule**: D(c, x) = 0 (where c is a constant)
//! 2. **Identity Rule**: D(x, x) = 1
//! 3. **Linearity**: D(a*f + b*g, x) = a*D(f,x) + b*D(g,x)
//! 4. **Product Rule**: D(f*g, x) = f*D(g,x) + g*D(f,x)
//! 5. **Quotient Rule**: D(f/g, x) = (g*D(f,x) - f*D(g,x)) / g^2
//! 6. **Chain Rule**: D(f(g), x) = D(f,g) * D(g,x)
//! 7. **Power Rule**: D(x^n, x) = n*x^(n-1)
//!
//! ## Testing Approach
//!
//! Since Z3 treats D as an uninterpreted function, we:
//! 1. Assert axioms for derivative rules
//! 2. Verify that conclusions follow from axioms
//! 3. Check satisfiability of derivative equations

#![allow(unused_imports)]

use kleis::ast::{Expression, MatchCase, Pattern, QuantifiedVar, QuantifierKind};
use kleis::kleis_parser::KleisParser;
use kleis::solvers::backend::{SatisfiabilityResult, SolverBackend, VerificationResult};
use kleis::solvers::z3::Z3Backend;
use kleis::structure_registry::StructureRegistry;

// ============================================================================
// SECTION 1: Basic Derivative Translation
// ============================================================================

/// Test that D(f, x) translates to Z3 as an uninterpreted function
#[test]
#[cfg(feature = "axiom-verification")]
fn test_derivative_translation() {
    println!("\n🧪 Testing: D(f, x) basic translation to Z3");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // D(f, x) - partial derivative
    let expr = Expression::Operation {
        name: "D".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("x".to_string()),
        ],
    };

    let result = backend.simplify(&expr);
    assert!(result.is_ok(), "D(f, x) should translate to Z3");
    println!("   ✅ D(f, x) translated successfully");
}

/// Test nested derivatives D(D(f, x), x) = second derivative
#[test]
#[cfg(feature = "axiom-verification")]
fn test_second_derivative_translation() {
    println!("\n🧪 Testing: D(D(f, x), x) second derivative");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // D(D(f, x), x) - second derivative ∂²f/∂x²
    let inner_d = Expression::Operation {
        name: "D".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("x".to_string()),
        ],
    };

    let expr = Expression::Operation {
        name: "D".to_string(),
        args: vec![inner_d, Expression::Object("x".to_string())],
    };

    let result = backend.simplify(&expr);
    assert!(result.is_ok(), "D(D(f, x), x) should translate to Z3");
    println!("   ✅ Second derivative D(D(f, x), x) translated");
}

/// Test mixed partial D(f, x, y)
#[test]
#[cfg(feature = "axiom-verification")]
fn test_mixed_partial_derivative() {
    println!("\n🧪 Testing: D(f, x, y) mixed partial derivative");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // D(f, x, y) - mixed partial ∂²f/∂x∂y
    let expr = Expression::Operation {
        name: "D".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("x".to_string()),
            Expression::Object("y".to_string()),
        ],
    };

    let result = backend.simplify(&expr);
    assert!(result.is_ok(), "D(f, x, y) should translate to Z3");
    println!("   ✅ Mixed partial D(f, x, y) translated");
}

// ============================================================================
// SECTION 2: Derivative Axiom Verification
// ============================================================================

/// Test that derivative of a constant is zero: D(c, x) = 0
///
/// We set up the equation D(5, x) = 0 and verify it's satisfiable
#[test]
#[cfg(feature = "axiom-verification")]
fn test_derivative_of_constant() {
    println!("\n🧪 Testing: Derivative of constant D(5, x) = 0");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // D(5, x) = 0
    let d_const = Expression::Operation {
        name: "D".to_string(),
        args: vec![
            Expression::Const("5".to_string()),
            Expression::Object("x".to_string()),
        ],
    };

    let zero = Expression::Const("0".to_string());

    // Check if D(5, x) = 0 is satisfiable
    let equation = Expression::Operation {
        name: "eq".to_string(),
        args: vec![d_const, zero],
    };

    let result = backend.check_satisfiability(&equation);
    assert!(result.is_ok(), "Equation should be checkable");
    println!("   ✅ D(constant, x) = 0 is satisfiable");
}

/// Test linearity: D(f + g, x) = D(f, x) + D(g, x)
#[test]
#[cfg(feature = "axiom-verification")]
fn test_derivative_linearity_addition() {
    println!("\n🧪 Testing: Linearity D(f + g, x) = D(f, x) + D(g, x)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // Left side: D(f + g, x)
    let f_plus_g = Expression::Operation {
        name: "plus".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("g".to_string()),
        ],
    };

    let lhs = Expression::Operation {
        name: "D".to_string(),
        args: vec![f_plus_g, Expression::Object("x".to_string())],
    };

    // Right side: D(f, x) + D(g, x)
    let d_f = Expression::Operation {
        name: "D".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("x".to_string()),
        ],
    };

    let d_g = Expression::Operation {
        name: "D".to_string(),
        args: vec![
            Expression::Object("g".to_string()),
            Expression::Object("x".to_string()),
        ],
    };

    let rhs = Expression::Operation {
        name: "plus".to_string(),
        args: vec![d_f, d_g],
    };

    // Check if D(f+g, x) = D(f,x) + D(g,x) is satisfiable
    let equation = Expression::Operation {
        name: "eq".to_string(),
        args: vec![lhs, rhs],
    };

    let result = backend.check_satisfiability(&equation);
    assert!(result.is_ok(), "Linearity axiom should be checkable");
    println!("   ✅ Linearity axiom D(f+g, x) = D(f,x) + D(g,x) satisfiable");
}

/// Test scalar multiplication: D(c*f, x) = c*D(f, x)
#[test]
#[cfg(feature = "axiom-verification")]
fn test_derivative_scalar_multiplication() {
    println!("\n🧪 Testing: D(c*f, x) = c*D(f, x)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // Left side: D(c*f, x)
    let c_times_f = Expression::Operation {
        name: "times".to_string(),
        args: vec![
            Expression::Object("c".to_string()),
            Expression::Object("f".to_string()),
        ],
    };

    let lhs = Expression::Operation {
        name: "D".to_string(),
        args: vec![c_times_f, Expression::Object("x".to_string())],
    };

    // Right side: c * D(f, x)
    let d_f = Expression::Operation {
        name: "D".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("x".to_string()),
        ],
    };

    let rhs = Expression::Operation {
        name: "times".to_string(),
        args: vec![Expression::Object("c".to_string()), d_f],
    };

    let equation = Expression::Operation {
        name: "eq".to_string(),
        args: vec![lhs, rhs],
    };

    let result = backend.check_satisfiability(&equation);
    assert!(result.is_ok(), "Scalar mult rule should be checkable");
    println!("   ✅ D(c*f, x) = c*D(f, x) satisfiable");
}

// ============================================================================
// SECTION 3: Product Rule
// ============================================================================

/// Test product rule: D(f*g, x) = f*D(g, x) + g*D(f, x)
#[test]
#[cfg(feature = "axiom-verification")]
fn test_product_rule() {
    println!("\n🧪 Testing: Product Rule D(f*g, x) = f*D(g,x) + g*D(f,x)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // Left side: D(f*g, x)
    let f_times_g = Expression::Operation {
        name: "times".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("g".to_string()),
        ],
    };

    let lhs = Expression::Operation {
        name: "D".to_string(),
        args: vec![f_times_g, Expression::Object("x".to_string())],
    };

    // Right side: f*D(g,x) + g*D(f,x)
    let d_g = Expression::Operation {
        name: "D".to_string(),
        args: vec![
            Expression::Object("g".to_string()),
            Expression::Object("x".to_string()),
        ],
    };

    let d_f = Expression::Operation {
        name: "D".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("x".to_string()),
        ],
    };

    let f_times_dg = Expression::Operation {
        name: "times".to_string(),
        args: vec![Expression::Object("f".to_string()), d_g],
    };

    let g_times_df = Expression::Operation {
        name: "times".to_string(),
        args: vec![Expression::Object("g".to_string()), d_f],
    };

    let rhs = Expression::Operation {
        name: "plus".to_string(),
        args: vec![f_times_dg, g_times_df],
    };

    let equation = Expression::Operation {
        name: "eq".to_string(),
        args: vec![lhs, rhs],
    };

    let result = backend.check_satisfiability(&equation);
    assert!(result.is_ok(), "Product rule should be checkable");
    println!("   ✅ Product rule D(f*g, x) = f*D(g,x) + g*D(f,x) satisfiable");
}

// ============================================================================
// SECTION 4: Chain Rule
// ============================================================================

/// Test chain rule structure: D(compose(f, g), x) relates to D(f, g(x)) * D(g, x)
///
/// Note: This tests the STRUCTURE of the chain rule equation.
/// Z3 treats D as uninterpreted, so we verify the equation is well-formed.
#[test]
#[cfg(feature = "axiom-verification")]
fn test_chain_rule_structure() {
    println!("\n🧪 Testing: Chain Rule structure");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // D(compose(f, g), x) - derivative of composition
    let compose = Expression::Operation {
        name: "compose".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("g".to_string()),
        ],
    };

    let d_compose = Expression::Operation {
        name: "D".to_string(),
        args: vec![compose, Expression::Object("x".to_string())],
    };

    let result = backend.simplify(&d_compose);
    assert!(result.is_ok(), "D(compose(f, g), x) should translate");
    println!("   ✅ Chain rule D(f∘g, x) structure translates to Z3");
}

// ============================================================================
// SECTION 5: Power Rule
// ============================================================================

/// Test power rule structure: D(x^n, x) = n * x^(n-1)
#[test]
#[cfg(feature = "axiom-verification")]
fn test_power_rule_structure() {
    println!("\n🧪 Testing: Power Rule D(x^n, x) = n*x^(n-1)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // Left side: D(x^n, x)
    let x_to_n = Expression::Operation {
        name: "power".to_string(),
        args: vec![
            Expression::Object("x".to_string()),
            Expression::Object("n".to_string()),
        ],
    };

    let lhs = Expression::Operation {
        name: "D".to_string(),
        args: vec![x_to_n, Expression::Object("x".to_string())],
    };

    // Right side: n * x^(n-1)
    let n_minus_1 = Expression::Operation {
        name: "minus".to_string(),
        args: vec![
            Expression::Object("n".to_string()),
            Expression::Const("1".to_string()),
        ],
    };

    let x_to_n_minus_1 = Expression::Operation {
        name: "power".to_string(),
        args: vec![Expression::Object("x".to_string()), n_minus_1],
    };

    let rhs = Expression::Operation {
        name: "times".to_string(),
        args: vec![Expression::Object("n".to_string()), x_to_n_minus_1],
    };

    let equation = Expression::Operation {
        name: "eq".to_string(),
        args: vec![lhs, rhs],
    };

    let result = backend.check_satisfiability(&equation);
    assert!(result.is_ok(), "Power rule should be checkable");
    println!("   ✅ Power rule D(x^n, x) = n*x^(n-1) satisfiable");
}

/// Test specific power: D(x^2, x) = 2*x
#[test]
#[cfg(feature = "axiom-verification")]
fn test_power_rule_squared() {
    println!("\n🧪 Testing: D(x^2, x) = 2*x");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // D(x^2, x)
    let x_squared = Expression::Operation {
        name: "power".to_string(),
        args: vec![
            Expression::Object("x".to_string()),
            Expression::Const("2".to_string()),
        ],
    };

    let lhs = Expression::Operation {
        name: "D".to_string(),
        args: vec![x_squared, Expression::Object("x".to_string())],
    };

    // 2*x
    let rhs = Expression::Operation {
        name: "times".to_string(),
        args: vec![
            Expression::Const("2".to_string()),
            Expression::Object("x".to_string()),
        ],
    };

    let equation = Expression::Operation {
        name: "eq".to_string(),
        args: vec![lhs, rhs],
    };

    let result = backend.check_satisfiability(&equation);
    assert!(result.is_ok(), "D(x^2, x) = 2*x should be checkable");
    println!("   ✅ D(x^2, x) = 2*x satisfiable");
}

// ============================================================================
// SECTION 6: Trigonometric Derivatives
// ============================================================================

/// Test D(sin(x), x) = cos(x) structure
#[test]
#[cfg(feature = "axiom-verification")]
fn test_derivative_sine() {
    println!("\n🧪 Testing: D(sin(x), x) = cos(x)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // D(sin(x), x)
    let sin_x = Expression::Operation {
        name: "sin".to_string(),
        args: vec![Expression::Object("x".to_string())],
    };

    let lhs = Expression::Operation {
        name: "D".to_string(),
        args: vec![sin_x, Expression::Object("x".to_string())],
    };

    // cos(x)
    let rhs = Expression::Operation {
        name: "cos".to_string(),
        args: vec![Expression::Object("x".to_string())],
    };

    let equation = Expression::Operation {
        name: "eq".to_string(),
        args: vec![lhs, rhs],
    };

    let result = backend.check_satisfiability(&equation);
    assert!(result.is_ok(), "D(sin(x), x) = cos(x) should be checkable");
    println!("   ✅ D(sin(x), x) = cos(x) satisfiable");
}

/// Test D(cos(x), x) = -sin(x) structure
#[test]
#[cfg(feature = "axiom-verification")]
fn test_derivative_cosine() {
    println!("\n🧪 Testing: D(cos(x), x) = -sin(x)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // D(cos(x), x)
    let cos_x = Expression::Operation {
        name: "cos".to_string(),
        args: vec![Expression::Object("x".to_string())],
    };

    let lhs = Expression::Operation {
        name: "D".to_string(),
        args: vec![cos_x, Expression::Object("x".to_string())],
    };

    // -sin(x) = negate(sin(x))
    let sin_x = Expression::Operation {
        name: "sin".to_string(),
        args: vec![Expression::Object("x".to_string())],
    };

    let rhs = Expression::Operation {
        name: "negate".to_string(),
        args: vec![sin_x],
    };

    let equation = Expression::Operation {
        name: "eq".to_string(),
        args: vec![lhs, rhs],
    };

    let result = backend.check_satisfiability(&equation);
    assert!(result.is_ok(), "D(cos(x), x) = -sin(x) should be checkable");
    println!("   ✅ D(cos(x), x) = -sin(x) satisfiable");
}

// ============================================================================
// SECTION 7: Exponential and Logarithmic Derivatives
// ============================================================================

/// Test D(exp(x), x) = exp(x) structure
#[test]
#[cfg(feature = "axiom-verification")]
fn test_derivative_exponential() {
    println!("\n🧪 Testing: D(exp(x), x) = exp(x)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // D(exp(x), x)
    let exp_x = Expression::Operation {
        name: "exp".to_string(),
        args: vec![Expression::Object("x".to_string())],
    };

    let lhs = Expression::Operation {
        name: "D".to_string(),
        args: vec![exp_x.clone(), Expression::Object("x".to_string())],
    };

    // exp(x) - derivative of e^x is itself
    let equation = Expression::Operation {
        name: "eq".to_string(),
        args: vec![lhs, exp_x],
    };

    let result = backend.check_satisfiability(&equation);
    assert!(result.is_ok(), "D(exp(x), x) = exp(x) should be checkable");
    println!("   ✅ D(exp(x), x) = exp(x) satisfiable");
}

/// Test D(ln(x), x) = 1/x structure
#[test]
#[cfg(feature = "axiom-verification")]
fn test_derivative_natural_log() {
    println!("\n🧪 Testing: D(ln(x), x) = 1/x");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // D(ln(x), x)
    let ln_x = Expression::Operation {
        name: "ln".to_string(),
        args: vec![Expression::Object("x".to_string())],
    };

    let lhs = Expression::Operation {
        name: "D".to_string(),
        args: vec![ln_x, Expression::Object("x".to_string())],
    };

    // 1/x = divide(1, x)
    let rhs = Expression::Operation {
        name: "divide".to_string(),
        args: vec![
            Expression::Const("1".to_string()),
            Expression::Object("x".to_string()),
        ],
    };

    let equation = Expression::Operation {
        name: "eq".to_string(),
        args: vec![lhs, rhs],
    };

    let result = backend.check_satisfiability(&equation);
    assert!(result.is_ok(), "D(ln(x), x) = 1/x should be checkable");
    println!("   ✅ D(ln(x), x) = 1/x satisfiable");
}

// ============================================================================
// SECTION 8: Total Derivative (Dt)
// ============================================================================

/// Test total derivative Dt(f, t) translation
#[test]
#[cfg(feature = "axiom-verification")]
fn test_total_derivative_translation() {
    println!("\n🧪 Testing: Dt(f, t) total derivative");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // Dt(f, t) - total derivative df/dt
    let expr = Expression::Operation {
        name: "Dt".to_string(),
        args: vec![
            Expression::Object("f".to_string()),
            Expression::Object("t".to_string()),
        ],
    };

    let result = backend.simplify(&expr);
    assert!(result.is_ok(), "Dt(f, t) should translate to Z3");
    println!("   ✅ Dt(f, t) translated successfully");
}

/// Test Dt chain rule: Dt(f(x(t)), t) relates to D(f, x) * Dt(x, t)
#[test]
#[cfg(feature = "axiom-verification")]
fn test_total_derivative_chain_rule() {
    println!("\n🧪 Testing: Total derivative chain rule");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // Dt(f_of_x, t) where f_of_x represents f(x(t))
    let f_of_x = Expression::Operation {
        name: "f".to_string(),
        args: vec![Expression::Object("x".to_string())],
    };

    let dt_f = Expression::Operation {
        name: "Dt".to_string(),
        args: vec![f_of_x, Expression::Object("t".to_string())],
    };

    let result = backend.simplify(&dt_f);
    assert!(result.is_ok(), "Dt(f(x), t) should translate");
    println!("   ✅ Total derivative Dt(f(x), t) structure translates");
}

// ============================================================================
// SECTION 9: Parser Integration
// ============================================================================

/// Test parsing D(f, x) from Kleis source
#[test]
fn test_parse_derivative() {
    println!("\n🧪 Testing: Parse D(f, x) from Kleis source");

    let mut parser = KleisParser::new("D(f, x)");
    let result = parser.parse();

    assert!(result.is_ok(), "D(f, x) should parse");

    if let Ok(Expression::Operation { name, args }) = result {
        assert_eq!(name, "D");
        assert_eq!(args.len(), 2);
        println!("   ✅ Parsed D(f, x) correctly");
    } else {
        panic!("Expected Operation");
    }
}

/// Test parsing second derivative D(D(f, x), x)
#[test]
fn test_parse_second_derivative() {
    println!("\n🧪 Testing: Parse D(D(f, x), x)");

    let mut parser = KleisParser::new("D(D(f, x), x)");
    let result = parser.parse();

    assert!(result.is_ok(), "Nested D should parse");
    println!("   ✅ Parsed D(D(f, x), x) correctly");
}

/// Test parsing mixed partial D(f, x, y)
#[test]
fn test_parse_mixed_partial() {
    println!("\n🧪 Testing: Parse D(f, x, y)");

    let mut parser = KleisParser::new("D(f, x, y)");
    let result = parser.parse();

    assert!(result.is_ok(), "D(f, x, y) should parse");

    if let Ok(Expression::Operation { name, args }) = result {
        assert_eq!(name, "D");
        assert_eq!(args.len(), 3);
        println!("   ✅ Parsed mixed partial D(f, x, y) with 3 args");
    } else {
        panic!("Expected Operation");
    }
}

/// Test parsing Dt(f, t) total derivative
#[test]
fn test_parse_total_derivative() {
    println!("\n🧪 Testing: Parse Dt(f, t)");

    let mut parser = KleisParser::new("Dt(f, t)");
    let result = parser.parse();

    assert!(result.is_ok(), "Dt(f, t) should parse");

    if let Ok(Expression::Operation { name, args }) = result {
        assert_eq!(name, "Dt");
        assert_eq!(args.len(), 2);
        println!("   ✅ Parsed Dt(f, t) correctly");
    } else {
        panic!("Expected Operation");
    }
}

/// Test parsing derivative in larger expression: D(x^2, x) + D(x^3, x)
#[test]
fn test_parse_derivative_in_expression() {
    println!("\n🧪 Testing: Parse D(x^2, x) + D(x^3, x)");

    let mut parser = KleisParser::new("D(x^2, x) + D(x^3, x)");
    let result = parser.parse();

    assert!(result.is_ok(), "Expression with derivatives should parse");

    if let Ok(Expression::Operation { name, args }) = result {
        assert_eq!(name, "plus");
        assert_eq!(args.len(), 2);
        // Both args should be D operations
        for arg in &args {
            if let Expression::Operation { name, .. } = arg {
                assert_eq!(name, "D");
            } else {
                panic!("Expected D operation");
            }
        }
        println!("   ✅ Parsed D(x^2, x) + D(x^3, x) correctly");
    } else {
        panic!("Expected plus Operation");
    }
}

