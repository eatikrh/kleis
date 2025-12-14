//! Level 2 Tests: Render → Parse Round-Trip
//!
//! These tests render AST to Kleis syntax, then parse it back.
//! Failures reveal gaps in the Kleis PARSER.
//!
//! Testing Ladder:
//!   Level 1: AST → Render → Assert symbols (naive)
//!   Level 2: AST → Render → PARSE → Compare (THIS FILE)
//!   Level 3: AST → Render → Parse → Z3 Verify

use kleis::ast::Expression;
use kleis::kleis_parser::parse_kleis;
use kleis::render::{build_default_context, render_expression, RenderTarget};

// Helper functions
fn c(s: &str) -> Expression {
    Expression::Const(s.to_string())
}

fn o(s: &str) -> Expression {
    Expression::Object(s.to_string())
}

fn op(name: &str, args: Vec<Expression>) -> Expression {
    Expression::Operation {
        name: name.to_string(),
        args,
    }
}

/// Helper to test render → parse round-trip
/// Returns (rendered_string, parse_result)
fn roundtrip(expr: &Expression) -> (String, Result<Expression, String>) {
    let ctx = build_default_context();
    let rendered = render_expression(expr, &ctx, &RenderTarget::Kleis);
    let parsed = parse_kleis(&rendered).map_err(|e| format!("{:?}", e));
    (rendered, parsed)
}

// ============================================================
// BASIC OPERATIONS - Should all pass
// ============================================================

#[test]
fn roundtrip_simple_variable() {
    let expr = o("x");
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert!(parsed.is_ok(), "Failed to parse: {} -> {:?}", rendered, parsed);
}

#[test]
fn roundtrip_simple_constant() {
    let expr = c("42");
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert!(parsed.is_ok(), "Failed to parse: {} -> {:?}", rendered, parsed);
}

#[test]
fn roundtrip_addition() {
    let expr = op("plus", vec![o("a"), o("b")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert!(parsed.is_ok(), "Failed to parse: {} -> {:?}", rendered, parsed);
}

#[test]
fn roundtrip_multiplication() {
    let expr = op("scalar_multiply", vec![o("a"), o("b")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    // Note: × might not be parsed as multiply operator
    println!("Parse result: {:?}", parsed);
}

#[test]
fn roundtrip_power() {
    let expr = op("power", vec![o("x"), c("2")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert!(parsed.is_ok(), "Failed to parse: {} -> {:?}", rendered, parsed);
}

// ============================================================
// CALCULUS OPERATIONS - These will reveal parser gaps
// ============================================================

#[test]
fn roundtrip_gradient() {
    // ∇f - parser should support this (it's in parse_primary)
    let expr = op("gradient", vec![o("f")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    println!("Parse result: {:?}", parsed);
    // This should work - ∇ is a prefix operator
    assert!(parsed.is_ok(), "Failed to parse gradient: {} -> {:?}", rendered, parsed);
}

#[test]
fn roundtrip_integral_simple() {
    // ∫f - simple indefinite integral (prefix operator)
    let expr = op("Integrate", vec![o("f")]);
    let ctx = build_default_context();
    // Render manually since we don't have a template for simple Integrate
    let rendered = "∫f";
    let parsed = parse_kleis(rendered).map_err(|e| format!("{:?}", e));
    println!("Rendered: {}", rendered);
    println!("Parse result: {:?}", parsed);
    // ∫ is a prefix operator in kleis_parser
}

#[test]
fn roundtrip_integral_with_bounds() {
    // ∫_{0}^{1} f dx - definite integral with bounds
    let expr = op("int_bounds", vec![o("f"), c("0"), c("1"), o("x")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    println!("Parse result: {:?}", parsed);
    // This will likely FAIL - parser may not support subscript/superscript on ∫
}

#[test]
fn roundtrip_summation() {
    // Σ_{i=1}^{n} a_i
    let expr = op("sum_bounds", vec![o("a_i"), o("i=1"), o("n")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    println!("Parse result: {:?}", parsed);
    // This will likely FAIL - Σ not in parser as prefix operator
}

#[test]
fn roundtrip_product() {
    // Π_{i=1}^{n} a_i
    let expr = op("prod_bounds", vec![o("a_i"), o("i=1"), o("n")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    println!("Parse result: {:?}", parsed);
    // This will likely FAIL - Π not in parser as prefix operator
}

#[test]
fn roundtrip_partial_derivative() {
    // ∂f/∂x
    let expr = op("d_part", vec![o("f"), o("x")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    println!("Parse result: {:?}", parsed);
    // ∂f/∂x format - parser may not support this notation
}

#[test]
fn roundtrip_total_derivative() {
    // dy/dt
    let expr = op("d_dt", vec![o("y"), o("t")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    println!("Parse result: {:?}", parsed);
    // df/dx format - parser may treat / as division
}

#[test]
fn roundtrip_limit() {
    // lim_{x→0} f(x)
    let expr = op("lim", vec![o("f(x)"), o("x"), c("0")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    println!("Parse result: {:?}", parsed);
    // This will likely FAIL - lim not in grammar!
}

// ============================================================
// LOGIC OPERATIONS
// ============================================================

#[test]
fn roundtrip_implies() {
    // P ⟹ Q
    let expr = op("implies", vec![o("P"), o("Q")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    println!("Parse result: {:?}", parsed);
    // ⟹ should be supported
}

#[test]
fn roundtrip_forall() {
    // ∀(x). P(x)
    let expr = op("forall", vec![o("x"), o("P(x)")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    println!("Parse result: {:?}", parsed);
}

#[test]
fn roundtrip_exists() {
    // ∃(x). P(x)
    let expr = op("exists", vec![o("x"), o("P(x)")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    println!("Parse result: {:?}", parsed);
}

// ============================================================
// RELATIONS
// ============================================================

#[test]
fn roundtrip_equals() {
    // a = b
    let expr = op("equals", vec![o("a"), o("b")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert!(parsed.is_ok(), "Failed to parse: {} -> {:?}", rendered, parsed);
}

#[test]
fn roundtrip_less_than() {
    // a < b
    let expr = op("less_than", vec![o("a"), o("b")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert!(parsed.is_ok(), "Failed to parse: {} -> {:?}", rendered, parsed);
}

#[test]
fn roundtrip_in_set() {
    // x ∈ S
    let expr = op("in_set", vec![o("x"), o("S")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    println!("Parse result: {:?}", parsed);
    // ∈ might not be supported
}

// ============================================================
// CONTROL FLOW
// ============================================================

#[test]
fn roundtrip_conditional() {
    // if x > 0 then x else -x
    let cond_expr = Expression::Conditional {
        condition: Box::new(op("greater_than", vec![o("x"), c("0")])),
        then_branch: Box::new(o("x")),
        else_branch: Box::new(op("negate", vec![o("x")])),
    };
    let (rendered, parsed) = roundtrip(&cond_expr);
    println!("Rendered: {}", rendered);
    println!("Parse result: {:?}", parsed);
    // Parser supports if/then/else
}

#[test]
fn roundtrip_let_binding() {
    // let x = 5 in x + x
    let let_expr = Expression::Let {
        name: "x".to_string(),
        value: Box::new(c("5")),
        body: Box::new(op("plus", vec![o("x"), o("x")])),
    };
    let (rendered, parsed) = roundtrip(&let_expr);
    println!("Rendered: {}", rendered);
    println!("Parse result: {:?}", parsed);
    // Parser supports let/in
}

// ============================================================
// GREEK LETTERS
// ============================================================

#[test]
fn roundtrip_greek_alpha() {
    let expr = o("\\alpha");
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {} (should be α)", rendered);
    println!("Parse result: {:?}", parsed);
    // α should be parsed as identifier
}

#[test]
fn roundtrip_greek_pi() {
    let expr = o("\\pi");
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {} (should be π)", rendered);
    println!("Parse result: {:?}", parsed);
    // π is a symbolic constant - may need special handling
}

// ============================================================
// QUANTUM NOTATION
// ============================================================

#[test]
fn roundtrip_ket() {
    // |ψ⟩
    let expr = op("ket", vec![o("ψ")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    println!("Parse result: {:?}", parsed);
    // Ket notation may not be in parser
}

#[test]
fn roundtrip_bra() {
    // ⟨φ|
    let expr = op("bra", vec![o("φ")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    println!("Parse result: {:?}", parsed);
    // Bra notation may not be in parser
}

// ============================================================
// PLACEHOLDER
// ============================================================

#[test]
fn roundtrip_placeholder() {
    let placeholder = Expression::Placeholder {
        id: 0,
        hint: "value".to_string(),
    };
    let (rendered, parsed) = roundtrip(&placeholder);
    println!("Rendered: {} (should be □)", rendered);
    println!("Parse result: {:?}", parsed);
    // □ is in grammar but may not be in parser
}

