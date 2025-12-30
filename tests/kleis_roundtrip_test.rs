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
        span: None,
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
    assert!(
        parsed.is_ok(),
        "Failed to parse: {} -> {:?}",
        rendered,
        parsed
    );
}

#[test]
fn roundtrip_simple_constant() {
    let expr = c("42");
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert!(
        parsed.is_ok(),
        "Failed to parse: {} -> {:?}",
        rendered,
        parsed
    );
}

#[test]
fn roundtrip_addition() {
    let expr = op("plus", vec![o("a"), o("b")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert!(
        parsed.is_ok(),
        "Failed to parse: {} -> {:?}",
        rendered,
        parsed
    );
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
    assert!(
        parsed.is_ok(),
        "Failed to parse: {} -> {:?}",
        rendered,
        parsed
    );
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
    assert!(
        parsed.is_ok(),
        "Failed to parse gradient: {} -> {:?}",
        rendered,
        parsed
    );
}

#[test]
fn roundtrip_integral_simple() {
    // ∫f - simple indefinite integral (prefix operator)
    let _expr = op("Integrate", vec![o("f")]);
    let _ctx = build_default_context();
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
    // Grammar v0.7: D(f, x) - Mathematica-style partial derivative
    let expr = op("d_part", vec![o("f"), o("x")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert_eq!(
        rendered, "D(f, x)",
        "Should render as Mathematica-style D()"
    );
    println!("Parse result: {:?}", parsed);
    // Parser should handle D(f, x) as function call
}

#[test]
fn roundtrip_total_derivative() {
    // Grammar v0.7: Dt(y, t) - Mathematica-style total derivative
    let expr = op("d_dt", vec![o("y"), o("t")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert_eq!(
        rendered, "Dt(y, t)",
        "Should render as Mathematica-style Dt()"
    );
    println!("Parse result: {:?}", parsed);
    // Parser should handle Dt(y, t) as function call
}

#[test]
fn roundtrip_limit() {
    // v0.95: lim(var, target, body) - parseable sugar syntax
    let expr = op("lim", vec![o("f(x)"), o("x"), c("0")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert!(
        rendered.contains("lim("),
        "Should render as lim function call"
    );
    println!("Parse result: {:?}", parsed);
    // Parser should handle lim(x, 0, f(x)) as sugar for limit
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
    assert!(
        parsed.is_ok(),
        "Failed to parse: {} -> {:?}",
        rendered,
        parsed
    );
}

#[test]
fn roundtrip_less_than() {
    // a < b
    let expr = op("less_than", vec![o("a"), o("b")]);
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert!(
        parsed.is_ok(),
        "Failed to parse: {} -> {:?}",
        rendered,
        parsed
    );
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
        span: None,
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
        pattern: kleis::ast::Pattern::Variable("x".to_string()),
        type_annotation: None,
        value: Box::new(c("5")),
        body: Box::new(op("plus", vec![o("x"), o("x")])),
        span: None,
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

// ============================================================
// LAMBDA EXPRESSIONS
// ============================================================

#[test]
fn roundtrip_lambda_simple() {
    // λ x . x
    let expr = Expression::Lambda {
        params: vec![kleis::ast::LambdaParam::new("x")],
        body: Box::new(o("x")),
        span: None,
    };
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert!(parsed.is_ok(), "Failed to parse: {}", rendered);
    if let Ok(Expression::Lambda { params, body, .. }) = parsed {
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "x");
        assert!(matches!(*body, Expression::Object(ref n) if n == "x"));
    } else {
        panic!("Expected Lambda, got {:?}", parsed);
    }
}

#[test]
fn roundtrip_lambda_arithmetic() {
    // λ x . x + 1
    let expr = Expression::Lambda {
        params: vec![kleis::ast::LambdaParam::new("x")],
        body: Box::new(op("plus", vec![o("x"), c("1")])),
        span: None,
    };
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert!(parsed.is_ok(), "Failed to parse: {}", rendered);
}

#[test]
fn roundtrip_lambda_multiple_params() {
    // λ x y . x + y
    let expr = Expression::Lambda {
        params: vec![
            kleis::ast::LambdaParam::new("x"),
            kleis::ast::LambdaParam::new("y"),
        ],
        body: Box::new(op("plus", vec![o("x"), o("y")])),
        span: None,
    };
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert!(parsed.is_ok(), "Failed to parse: {}", rendered);
    if let Ok(Expression::Lambda { params, .. }) = parsed {
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "x");
        assert_eq!(params[1].name, "y");
    }
}

#[test]
fn roundtrip_lambda_typed_param() {
    // λ (x : ℝ) . x * x
    let expr = Expression::Lambda {
        params: vec![kleis::ast::LambdaParam::typed("x", "ℝ")],
        body: Box::new(op("times", vec![o("x"), o("x")])),
        span: None,
    };
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert!(parsed.is_ok(), "Failed to parse: {}", rendered);
    if let Ok(Expression::Lambda { params, .. }) = parsed {
        assert_eq!(params[0].type_annotation, Some("ℝ".to_string()));
    }
}

#[test]
fn roundtrip_lambda_nested() {
    // λ x . λ y . x + y (curried)
    let inner = Expression::Lambda {
        params: vec![kleis::ast::LambdaParam::new("y")],
        body: Box::new(op("plus", vec![o("x"), o("y")])),
        span: None,
    };
    let expr = Expression::Lambda {
        params: vec![kleis::ast::LambdaParam::new("x")],
        body: Box::new(inner),
        span: None,
    };
    let (rendered, parsed) = roundtrip(&expr);
    println!("Rendered: {}", rendered);
    assert!(parsed.is_ok(), "Failed to parse: {}", rendered);
    if let Ok(Expression::Lambda { body, .. }) = parsed {
        assert!(matches!(*body, Expression::Lambda { .. }));
    }
}
