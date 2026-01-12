//! Tests for symbolic differentiation in stdlib/symbolic_diff.kleis
//!
//! These tests verify that diff(e, x) correctly computes symbolic derivatives
//! using pattern matching on Expression AST.
//!
//! Note: This is COMPUTATIONAL differentiation (Evaluator), not
//! AXIOMATIC differentiation (calculus_hof.kleis for Z3 verification).
//!
//! We use lowercase 'diff' because uppercase names in Kleis are reserved
//! for data constructors.

use kleis::evaluator::Evaluator;
use kleis::kleis_parser::{parse_kleis_program, KleisParser};
use kleis::pretty_print::PrettyPrinter;

/// Load the symbolic_diff.kleis stdlib and return an evaluator
fn create_evaluator() -> Evaluator {
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
    // diff(5, x) = 0
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff(num(5), \"x\")");
    println!("diff(num(5), x) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.contains("ENumber") && res.contains("0"));
}

#[test]
fn test_diff_variable_same() {
    // diff(x, x) = 1
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff(var(\"x\"), \"x\")");
    println!("diff(var(x), x) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.contains("ENumber") && res.contains("1"));
}

#[test]
fn test_diff_variable_different() {
    // diff(y, x) = 0
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff(var(\"y\"), \"x\")");
    println!("diff(var(y), x) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.contains("ENumber") && res.contains("0"));
}

// =============================================================================
// Power Rule Tests
// =============================================================================

#[test]
fn test_diff_x_squared() {
    // diff(x², x) = 2x
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff(e_pow(var(\"x\"), num(2)), \"x\")");
    println!("diff(x², x) = {:?}", result);
    assert!(result.is_ok());
    // Should contain 2 and x
    let res = result.unwrap();
    assert!(res.contains("2"));
}

#[test]
fn test_diff_x_cubed() {
    // diff(x³, x) = 3x²
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff(e_pow(var(\"x\"), num(3)), \"x\")");
    println!("diff(x³, x) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.contains("3"));
}

// =============================================================================
// Sum and Product Rule Tests
// =============================================================================

#[test]
fn test_diff_sum() {
    // diff(x + y, x) = 1 + 0 = 1
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff(e_add(var(\"x\"), var(\"y\")), \"x\")");
    println!("diff(x + y, x) = {:?}", result);
    assert!(result.is_ok());
    // Should be plus of two derivatives
    assert!(result.unwrap().contains("plus"));
}

#[test]
fn test_diff_product() {
    // diff(x * y, x) = 1*y + x*0 = y
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff(e_mul(var(\"x\"), var(\"y\")), \"x\")");
    println!("diff(x * y, x) = {:?}", result);
    assert!(result.is_ok());
    // Product rule applied
    assert!(result.unwrap().contains("plus"));
}

#[test]
fn test_diff_x_times_x() {
    // diff(x * x, x) = 1*x + x*1 = 2x
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff(e_mul(var(\"x\"), var(\"x\")), \"x\")");
    println!("diff(x * x, x) = {:?}", result);
    assert!(result.is_ok());
}

// =============================================================================
// Transcendental Function Tests
// =============================================================================

#[test]
fn test_diff_sin() {
    // diff(sin(x), x) = cos(x) * 1 = cos(x)
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff(e_sin(var(\"x\")), \"x\")");
    println!("diff(sin(x), x) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.contains("cos"));
}

#[test]
fn test_diff_cos() {
    // diff(cos(x), x) = -sin(x) * 1 = -sin(x)
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff(e_cos(var(\"x\")), \"x\")");
    println!("diff(cos(x), x) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.contains("sin") && res.contains("negate"));
}

#[test]
fn test_diff_exp() {
    // diff(e^x, x) = e^x * 1 = e^x
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff(e_exp(var(\"x\")), \"x\")");
    println!("diff(e^x, x) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.contains("exp"));
}

#[test]
fn test_diff_ln() {
    // diff(ln(x), x) = 1/x * 1 = 1/x
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff(e_ln(var(\"x\")), \"x\")");
    println!("diff(ln(x), x) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.contains("divide"));
}

#[test]
fn test_diff_sqrt() {
    // diff(√x, x) = 1/(2√x)
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff(e_sqrt(var(\"x\")), \"x\")");
    println!("diff(√x, x) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.contains("divide") && res.contains("sqrt"));
}

// =============================================================================
// Chain Rule Tests
// =============================================================================

#[test]
fn test_diff_sin_x_squared() {
    // diff(sin(x²), x) = cos(x²) * 2x
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff(e_sin(e_pow(var(\"x\"), num(2))), \"x\")");
    println!("diff(sin(x²), x) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.contains("cos") && res.contains("times"));
}

#[test]
fn test_diff_exp_x_squared() {
    // diff(e^(x²), x) = e^(x²) * 2x
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff(e_exp(e_pow(var(\"x\"), num(2))), \"x\")");
    println!("diff(e^(x²), x) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.contains("exp") && res.contains("times"));
}

// =============================================================================
// Quotient Rule Test
// =============================================================================

#[test]
fn test_diff_quotient() {
    // diff(x/y, x) = (1*y - x*0) / y² = y/y² = 1/y
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff(e_div(var(\"x\"), var(\"y\")), \"x\")");
    println!("diff(x/y, x) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.contains("divide"));
}

// =============================================================================
// Simplification Tests
// =============================================================================

#[test]
fn test_simplify_zero_plus() {
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "simplify(e_add(num(0), var(\"x\")))");
    println!("simplify(0 + x) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    // Should simplify to just x
    assert!(res.contains("EVariable") && res.contains("x"));
}

#[test]
fn test_simplify_times_zero() {
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "simplify(e_mul(num(0), var(\"x\")))");
    println!("simplify(0 * x) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    // Should simplify to 0
    assert!(res.contains("ENumber") && res.contains("0"));
}

#[test]
fn test_simplify_times_one() {
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "simplify(e_mul(num(1), var(\"x\")))");
    println!("simplify(1 * x) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    // Should simplify to x
    assert!(res.contains("EVariable"));
}

#[test]
fn test_diffs_x_squared() {
    // diffs = differentiate and simplify
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diffs(e_pow(var(\"x\"), num(2)), \"x\")");
    println!("diffs(x², x) = {:?}", result);
    assert!(result.is_ok());
}

// =============================================================================
// Coordinate-Specific Derivatives
// =============================================================================

#[test]
fn test_diff_t() {
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff_t(var(\"t\"))");
    println!("diff_t(t) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.contains("1"));
}

#[test]
fn test_diff_r() {
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff_r(e_pow(var(\"r\"), num(2)))");
    println!("diff_r(r²) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.contains("2"));
}

#[test]
fn test_diff_theta() {
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff_theta(e_sin(var(\"theta\")))");
    println!("diff_θ(sin(θ)) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.contains("cos"));
}

// =============================================================================
// Schwarzschild Metric Factor
// =============================================================================

#[test]
fn test_schwarzschild_factor() {
    // f(r) = √(1 - 2M/r)
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "schw_f(num(1))");
    println!("schw_f(1) = {:?}", result);
    assert!(result.is_ok());
    let res = result.unwrap();
    assert!(res.contains("sqrt"));
}

#[test]
fn test_diff_schwarzschild_factor() {
    // diff_r(√(1 - 2M/r)) should involve M/r² terms
    let evaluator = create_evaluator();
    let result = eval(&evaluator, "diff_r(schw_f(num(1)))");
    println!("diff_r(√(1 - 2M/r)) = {:?}", result);
    assert!(result.is_ok());
}
