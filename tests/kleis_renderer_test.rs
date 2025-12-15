//! Tests for the Kleis syntax renderer
//!
//! Verifies that RenderTarget::Kleis produces valid Kleis syntax
//! conforming to the grammar in docs/grammar/kleis_grammar_v05.ebnf

use kleis::ast::Expression;
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

// ============================================================
// Calculus Operations
// ============================================================

#[test]
fn kleis_render_integral_with_bounds() {
    let ctx = build_default_context();
    // int_bounds(integrand, lower, upper, variable)
    let expr = op("int_bounds", vec![o("f"), c("0"), c("1"), o("x")]);
    let out = render_expression(&expr, &ctx, &RenderTarget::Kleis);

    // Grammar: integral ::= "∫" [ subscript ] [ superscript ] expression [ "d" identifier ]
    assert!(out.contains("∫"), "Should contain integral symbol");
    assert!(out.contains("0"), "Should contain lower bound");
    assert!(out.contains("1"), "Should contain upper bound");
    assert!(out.contains("f"), "Should contain integrand");
    assert!(out.contains("dx"), "Should contain differential");
    println!("Kleis integral: {}", out);
}

#[test]
fn kleis_render_summation() {
    let ctx = build_default_context();
    // sum_bounds(body, from, to)
    let expr = op("sum_bounds", vec![o("a_i"), o("i=1"), o("n")]);
    let out = render_expression(&expr, &ctx, &RenderTarget::Kleis);

    // Grammar: summation ::= "Σ" [ subscript ] [ superscript ] expression
    assert!(out.contains("Σ"), "Should contain sigma symbol");
    assert!(out.contains("a_i"), "Should contain body");
    println!("Kleis summation: {}", out);
}

#[test]
fn kleis_render_product() {
    let ctx = build_default_context();
    // prod_bounds(body, from, to)
    let expr = op("prod_bounds", vec![o("a_i"), o("i=1"), o("n")]);
    let out = render_expression(&expr, &ctx, &RenderTarget::Kleis);

    // Grammar: product ::= "Π" [ subscript ] [ superscript ] expression
    assert!(out.contains("Π"), "Should contain pi product symbol");
    println!("Kleis product: {}", out);
}

#[test]
fn kleis_render_partial_derivative() {
    let ctx = build_default_context();
    // d_part(numerator, denominator)
    let expr = op("d_part", vec![o("f"), o("x")]);
    let out = render_expression(&expr, &ctx, &RenderTarget::Kleis);

    // Grammar v0.7: Mathematica-style D(f, x) for partial derivative
    assert!(out.contains("D("), "Should contain D function call");
    assert!(out.contains("f"), "Should contain function");
    assert!(out.contains("x"), "Should contain variable");
    assert_eq!(out, "D(f, x)");
    println!("Kleis partial derivative: {}", out);
}

#[test]
fn kleis_render_total_derivative() {
    let ctx = build_default_context();
    // d_dt(numerator, denominator)
    let expr = op("d_dt", vec![o("y"), o("t")]);
    let out = render_expression(&expr, &ctx, &RenderTarget::Kleis);

    // Grammar v0.7: Mathematica-style Dt(y, t) for total derivative
    assert!(out.contains("Dt("), "Should contain Dt function call");
    assert!(out.contains("y"), "Should contain function");
    assert!(out.contains("t"), "Should contain variable");
    assert_eq!(out, "Dt(y, t)");
    println!("Kleis total derivative: {}", out);
}

#[test]
fn kleis_render_limit() {
    let ctx = build_default_context();
    // lim(body, var, target)
    let expr = op("lim", vec![o("f(x)"), o("x"), c("0")]);
    let out = render_expression(&expr, &ctx, &RenderTarget::Kleis);

    // Grammar v0.7: Limit(body, var, target)
    assert!(out.contains("Limit("), "Should contain Limit function call");
    assert!(out.contains("f(x)"), "Should contain body");
    assert!(out.contains("x"), "Should contain variable");
    assert!(out.contains("0"), "Should contain target");
    println!("Kleis limit: {}", out);
}

#[test]
fn kleis_render_gradient() {
    let ctx = build_default_context();
    // gradient(f)
    let expr = op("gradient", vec![o("f")]);
    let out = render_expression(&expr, &ctx, &RenderTarget::Kleis);

    // Grammar: prefixOp includes ∇
    assert!(out.contains("∇"), "Should contain nabla symbol");
    assert!(out.contains("f"), "Should contain function");
    println!("Kleis gradient: {}", out);
}

// ============================================================
// Arithmetic Operations
// ============================================================

#[test]
fn kleis_render_arithmetic() {
    let ctx = build_default_context();

    // Plus
    let plus_expr = op("plus", vec![o("a"), o("b")]);
    let plus_out = render_expression(&plus_expr, &ctx, &RenderTarget::Kleis);
    assert!(plus_out.contains("+"), "Plus should contain +");

    // Minus
    let minus_expr = op("minus", vec![o("a"), o("b")]);
    let minus_out = render_expression(&minus_expr, &ctx, &RenderTarget::Kleis);
    assert!(minus_out.contains("-"), "Minus should contain -");

    // Multiply
    let mult_expr = op("scalar_multiply", vec![o("a"), o("b")]);
    let mult_out = render_expression(&mult_expr, &ctx, &RenderTarget::Kleis);
    assert!(mult_out.contains("×"), "Multiply should contain ×");

    // Divide
    let div_expr = op("scalar_divide", vec![o("a"), o("b")]);
    let div_out = render_expression(&div_expr, &ctx, &RenderTarget::Kleis);
    assert!(div_out.contains("/"), "Divide should contain /");

    // Power
    let pow_expr = op("power", vec![o("x"), c("2")]);
    let pow_out = render_expression(&pow_expr, &ctx, &RenderTarget::Kleis);
    assert!(pow_out.contains("^"), "Power should contain ^");

    println!("Plus: {}", plus_out);
    println!("Minus: {}", minus_out);
    println!("Multiply: {}", mult_out);
    println!("Divide: {}", div_out);
    println!("Power: {}", pow_out);
}

// ============================================================
// Logic and Quantifiers
// ============================================================

#[test]
fn kleis_render_quantifiers() {
    let ctx = build_default_context();

    // Forall
    let forall_expr = op("forall", vec![o("x"), o("P(x)")]);
    let forall_out = render_expression(&forall_expr, &ctx, &RenderTarget::Kleis);
    assert!(forall_out.contains("∀"), "Forall should contain ∀");

    // Exists
    let exists_expr = op("exists", vec![o("x"), o("P(x)")]);
    let exists_out = render_expression(&exists_expr, &ctx, &RenderTarget::Kleis);
    assert!(exists_out.contains("∃"), "Exists should contain ∃");

    println!("Forall: {}", forall_out);
    println!("Exists: {}", exists_out);
}

#[test]
fn kleis_render_logic_operators() {
    let ctx = build_default_context();

    // Implies
    let implies_expr = op("implies", vec![o("P"), o("Q")]);
    let implies_out = render_expression(&implies_expr, &ctx, &RenderTarget::Kleis);
    assert!(implies_out.contains("⟹"), "Implies should contain ⟹");

    // Iff
    let iff_expr = op("iff", vec![o("P"), o("Q")]);
    let iff_out = render_expression(&iff_expr, &ctx, &RenderTarget::Kleis);
    assert!(iff_out.contains("⟺"), "Iff should contain ⟺");

    // And
    let and_expr = op("logical_and", vec![o("P"), o("Q")]);
    let and_out = render_expression(&and_expr, &ctx, &RenderTarget::Kleis);
    assert!(and_out.contains("∧"), "And should contain ∧");

    // Or
    let or_expr = op("logical_or", vec![o("P"), o("Q")]);
    let or_out = render_expression(&or_expr, &ctx, &RenderTarget::Kleis);
    assert!(or_out.contains("∨"), "Or should contain ∨");

    println!("Implies: {}", implies_out);
    println!("Iff: {}", iff_out);
    println!("And: {}", and_out);
    println!("Or: {}", or_out);
}

// ============================================================
// Relations
// ============================================================

#[test]
fn kleis_render_relations() {
    let ctx = build_default_context();

    // Equals
    let eq_expr = op("equals", vec![o("a"), o("b")]);
    let eq_out = render_expression(&eq_expr, &ctx, &RenderTarget::Kleis);
    assert!(eq_out.contains("="), "Equals should contain =");

    // Less than
    let lt_expr = op("less_than", vec![o("a"), o("b")]);
    let lt_out = render_expression(&lt_expr, &ctx, &RenderTarget::Kleis);
    assert!(lt_out.contains("<"), "Less than should contain <");

    // In set
    let in_expr = op("in_set", vec![o("x"), o("S")]);
    let in_out = render_expression(&in_expr, &ctx, &RenderTarget::Kleis);
    assert!(in_out.contains("∈"), "In set should contain ∈");

    println!("Equals: {}", eq_out);
    println!("Less than: {}", lt_out);
    println!("In set: {}", in_out);
}

// ============================================================
// Control Flow
// ============================================================

#[test]
fn kleis_render_conditional() {
    let ctx = build_default_context();

    // Conditional expression
    let cond_expr = Expression::Conditional {
        condition: Box::new(op("greater_than", vec![o("x"), c("0")])),
        then_branch: Box::new(o("x")),
        else_branch: Box::new(op("negate", vec![o("x")])),
    };
    let out = render_expression(&cond_expr, &ctx, &RenderTarget::Kleis);

    // Grammar: conditional ::= "if" expression "then" expression "else" expression
    assert!(out.contains("if"), "Should contain if");
    assert!(out.contains("then"), "Should contain then");
    assert!(out.contains("else"), "Should contain else");
    println!("Kleis conditional: {}", out);
}

#[test]
fn kleis_render_let_binding() {
    let ctx = build_default_context();

    // Let binding
    let let_expr = Expression::Let {
        name: "x".to_string(),
        value: Box::new(c("5")),
        body: Box::new(op("plus", vec![o("x"), o("x")])),
    };
    let out = render_expression(&let_expr, &ctx, &RenderTarget::Kleis);

    // Grammar: letBinding ::= "let" identifier ... "=" expression "in" expression
    assert!(out.contains("let"), "Should contain let");
    assert!(out.contains("="), "Should contain =");
    assert!(out.contains("in"), "Should contain in");
    println!("Kleis let binding: {}", out);
}

// ============================================================
// Greek Letters and Symbols
// ============================================================

#[test]
fn kleis_render_greek_letters() {
    let ctx = build_default_context();

    // Greek letters should be converted to Unicode
    let alpha = render_expression(&o("\\alpha"), &ctx, &RenderTarget::Kleis);
    let beta = render_expression(&o("\\beta"), &ctx, &RenderTarget::Kleis);
    let pi = render_expression(&o("\\pi"), &ctx, &RenderTarget::Kleis);
    let omega = render_expression(&o("\\Omega"), &ctx, &RenderTarget::Kleis);

    assert_eq!(alpha, "α", "Alpha should be α");
    assert_eq!(beta, "β", "Beta should be β");
    assert_eq!(pi, "π", "Pi should be π");
    assert_eq!(omega, "Ω", "Omega should be Ω");

    println!("Alpha: {}", alpha);
    println!("Beta: {}", beta);
    println!("Pi: {}", pi);
    println!("Omega: {}", omega);
}

// ============================================================
// Quantum Notation
// ============================================================

#[test]
fn kleis_render_quantum_notation() {
    let ctx = build_default_context();

    // Ket
    let ket_expr = op("ket", vec![o("ψ")]);
    let ket_out = render_expression(&ket_expr, &ctx, &RenderTarget::Kleis);
    assert!(ket_out.contains("|"), "Ket should contain |");
    assert!(ket_out.contains("⟩"), "Ket should contain ⟩");

    // Bra
    let bra_expr = op("bra", vec![o("φ")]);
    let bra_out = render_expression(&bra_expr, &ctx, &RenderTarget::Kleis);
    assert!(bra_out.contains("⟨"), "Bra should contain ⟨");
    assert!(bra_out.contains("|"), "Bra should contain |");

    println!("Ket: {}", ket_out);
    println!("Bra: {}", bra_out);
}

// ============================================================
// Placeholder
// ============================================================

#[test]
fn kleis_render_placeholder() {
    let ctx = build_default_context();

    let placeholder = Expression::Placeholder {
        id: 0,
        hint: "value".to_string(),
    };
    let out = render_expression(&placeholder, &ctx, &RenderTarget::Kleis);

    // Grammar: placeholder ::= "□"
    assert_eq!(out, "□", "Placeholder should be □");
    println!("Placeholder: {}", out);
}
