//! Tests for symbolic differentiation in stdlib/symbolic_diff.kleis
//!
//! These tests verify that the `diff` function correctly computes
//! symbolic derivatives using pattern matching on expression trees.
//!
//! Note: This is COMPUTATIONAL differentiation (Evaluator), not
//! AXIOMATIC differentiation (D/Dt for Z3 verification).

use kleis::evaluator::Evaluator;
use kleis::kleis_parser::{parse_kleis_program, KleisParser};
use kleis::pretty_print::PrettyPrinter;

/// Load the symbolic_diff.kleis stdlib and return an evaluator
fn create_evaluator_with_diff() -> Evaluator {
    let source = std::fs::read_to_string("stdlib/symbolic_diff.kleis")
        .expect("Failed to read stdlib/symbolic_diff.kleis");
    
    let program = parse_kleis_program(&source)
        .expect("Failed to parse stdlib/symbolic_diff.kleis");
    
    let mut evaluator = Evaluator::new();
    evaluator.load_program(&program)
        .expect("Failed to load program into evaluator");
    
    evaluator
}

/// Helper to evaluate an expression and return string result
fn eval(evaluator: &Evaluator, input: &str) -> Result<String, String> {
    let mut parser = KleisParser::new(input);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    let result = evaluator.eval_concrete(&expr)?;
    let pp = PrettyPrinter::new();
    Ok(pp.format_expression(&result))
}

// =============================================================================
// Basic Derivative Tests
// =============================================================================

#[test]
fn test_diff_constant() {
    // d/dx(5) = 0
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(Const(5), \"x\")");
    println!("diff(Const(5), x) = {:?}", result);
    assert!(result.is_ok());
    // Result should be Const(0)
    let res = result.unwrap();
    assert!(res.contains("Const") && res.contains("0"));
}

#[test]
fn test_diff_variable_same() {
    // d/dx(x) = 1
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(Var(\"x\"), \"x\")");
    println!("diff(Var(x), x) = {:?}", result);
    assert!(result.is_ok());
    // Result should be Const(1)
    assert!(result.unwrap().contains("1"));
}

#[test]
fn test_diff_variable_different() {
    // d/dx(y) = 0
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(Var(\"y\"), \"x\")");
    println!("diff(Var(y), x) = {:?}", result);
    assert!(result.is_ok());
    // Result should be Const(0)
    assert!(result.unwrap().contains("0"));
}

// =============================================================================
// Power Rule Tests
// =============================================================================

#[test]
fn test_diff_x_squared() {
    // d/dx(x²) = 2x
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(x_squared, \"x\")");
    println!("diff(x², x) = {:?}", result);
    assert!(result.is_ok());
    // Result should contain Mul and Const(2)
    let res = result.unwrap();
    assert!(res.contains("Mul") || res.contains("2"));
}

#[test]
fn test_diff_x_cubed() {
    // d/dx(x³) = 3x²
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(x_cubed, \"x\")");
    println!("diff(x³, x) = {:?}", result);
    assert!(result.is_ok());
    // Result should contain 3
    let res = result.unwrap();
    assert!(res.contains("3"));
}

// =============================================================================
// Sum and Product Rule Tests
// =============================================================================

#[test]
fn test_diff_sum() {
    // d/dx(x + y) = 1 + 0 = 1
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(Add(Var(\"x\"), Var(\"y\")), \"x\")");
    println!("diff(x + y, x) = {:?}", result);
    assert!(result.is_ok());
    // Result should be Add(Const(1), Const(0))
    let res = result.unwrap();
    assert!(res.contains("Add") || res.contains("1"));
}

#[test]
fn test_diff_product() {
    // d/dx(x * y) = 1*y + x*0 = y
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(Mul(Var(\"x\"), Var(\"y\")), \"x\")");
    println!("diff(x * y, x) = {:?}", result);
    assert!(result.is_ok());
    // Result should follow product rule: Add(Mul(...), Mul(...))
    let res = result.unwrap();
    assert!(res.contains("Add") && res.contains("Mul"));
}

#[test]
fn test_diff_product_x_squared() {
    // d/dx(x * x) = 1*x + x*1 = 2x (via product rule)
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(Mul(Var(\"x\"), Var(\"x\")), \"x\")");
    println!("diff(x * x, x) = {:?}", result);
    assert!(result.is_ok());
    // Result: Add(Mul(Const(1), Var("x")), Mul(Var("x"), Const(1)))
    let res = result.unwrap();
    assert!(res.contains("Add"));
}

// =============================================================================
// Trigonometric Derivative Tests
// =============================================================================

#[test]
fn test_diff_sin() {
    // d/dx(sin(x)) = cos(x) * 1 = cos(x)
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(sin_x, \"x\")");
    println!("diff(sin(x), x) = {:?}", result);
    assert!(result.is_ok());
    // Result should contain Cos
    let res = result.unwrap();
    assert!(res.contains("Cos"));
}

#[test]
fn test_diff_cos() {
    // d/dx(cos(x)) = -sin(x) * 1 = -sin(x)
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(Cos(Var(\"x\")), \"x\")");
    println!("diff(cos(x), x) = {:?}", result);
    assert!(result.is_ok());
    // Result should contain Sin and Neg
    let res = result.unwrap();
    assert!(res.contains("Sin") && res.contains("Neg"));
}

// =============================================================================
// Exponential and Logarithm Tests
// =============================================================================

#[test]
fn test_diff_exp() {
    // d/dx(e^x) = e^x * 1 = e^x
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(exp_x, \"x\")");
    println!("diff(e^x, x) = {:?}", result);
    assert!(result.is_ok());
    // Result should contain Exp
    let res = result.unwrap();
    assert!(res.contains("Exp"));
}

#[test]
fn test_diff_ln() {
    // d/dx(ln(x)) = 1/x * 1 = 1/x
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(ln_x, \"x\")");
    println!("diff(ln(x), x) = {:?}", result);
    assert!(result.is_ok());
    // Result should contain Div
    let res = result.unwrap();
    assert!(res.contains("Div") || res.contains("Mul"));
}

// =============================================================================
// Chain Rule Tests
// =============================================================================

#[test]
fn test_diff_sin_x_squared() {
    // d/dx(sin(x²)) = cos(x²) * 2x
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(sin_x_squared, \"x\")");
    println!("diff(sin(x²), x) = {:?}", result);
    assert!(result.is_ok());
    // Result should contain Cos and Mul for the chain rule
    let res = result.unwrap();
    assert!(res.contains("Cos") && res.contains("Mul"));
}

#[test]
fn test_diff_exp_x_squared() {
    // d/dx(e^(x²)) = e^(x²) * 2x
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(exp_x_squared, \"x\")");
    println!("diff(e^(x²), x) = {:?}", result);
    assert!(result.is_ok());
    // Result should contain Exp and Mul for the chain rule
    let res = result.unwrap();
    assert!(res.contains("Exp") && res.contains("Mul"));
}

// =============================================================================
// Square Root Test (Special Case of Power Rule)
// =============================================================================

#[test]
fn test_diff_sqrt() {
    // d/dx(√x) = 1/(2√x)
    // Note: √x = x^(1/2), so this is power rule with n=1/2
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(sqrt_x, \"x\")");
    println!("diff(√x, x) = {:?}", result);
    assert!(result.is_ok());
    // Result should contain Div, Sqrt, and 2
    let res = result.unwrap();
    assert!(res.contains("Div") && res.contains("Sqrt"));
}

// =============================================================================
// Quotient Rule Test
// =============================================================================

#[test]
fn test_diff_quotient() {
    // d/dx(x/y) = (1*y - x*0) / y² = y/y² = 1/y
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(x_over_y, \"x\")");
    println!("diff(x/y, x) = {:?}", result);
    assert!(result.is_ok());
    // Result should contain quotient rule structure
    let res = result.unwrap();
    assert!(res.contains("Div"));
}

// =============================================================================
// Quadratic Polynomial Test
// =============================================================================

#[test]
fn test_diff_quadratic() {
    // d/dx(x² + 2x + 1) = 2x + 2
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "diff(quadratic, \"x\")");
    println!("diff(x² + 2x + 1, x) = {:?}", result);
    assert!(result.is_ok());
    // Result should be a sum of derivatives
    let res = result.unwrap();
    assert!(res.contains("Add"));
}

// =============================================================================
// Simplification Tests
// =============================================================================

#[test]
fn test_simplify_zero_add() {
    // simplify(0 + x) = x
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "simplify(Add(Const(0), Var(\"x\")))");
    println!("simplify(0 + x) = {:?}", result);
    assert!(result.is_ok());
    // Result should be Var("x")
    let res = result.unwrap();
    assert!(res.contains("Var") && res.contains("x"));
}

#[test]
fn test_simplify_zero_mul() {
    // simplify(0 * x) = 0
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "simplify(Mul(Const(0), Var(\"x\")))");
    println!("simplify(0 * x) = {:?}", result);
    assert!(result.is_ok());
    // Result should be Const(0)
    let res = result.unwrap();
    assert!(res.contains("Const") && res.contains("0"));
}

#[test]
fn test_simplify_one_mul() {
    // simplify(1 * x) = x
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "simplify(Mul(Const(1), Var(\"x\")))");
    println!("simplify(1 * x) = {:?}", result);
    assert!(result.is_ok());
    // Result should be Var("x")
    let res = result.unwrap();
    assert!(res.contains("Var") && res.contains("x"));
}

// =============================================================================
// Combined diff + simplify Test
// =============================================================================

#[test]
fn test_d_simplified() {
    // D_simplified differentiates and simplifies
    let evaluator = create_evaluator_with_diff();
    let result = eval(&evaluator, "D_simplified(Mul(Var(\"x\"), Var(\"x\")), \"x\")");
    println!("D_simplified(x*x, x) = {:?}", result);
    assert!(result.is_ok());
    // Should get a simplified form of 2x
}

