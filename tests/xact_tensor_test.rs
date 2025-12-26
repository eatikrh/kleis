//! xAct/xTensor-Style Tensor Notation Tests
//!
//! Tests that tensor expressions with signed indices render correctly:
//! - T(μ, -ν) → T^μ_ν (positive = up, negative = down)
//! - g(-μ, -ν) → g_μν (both covariant)
//! - R(ρ, -σ, -μ, -ν) → R^ρ_σμν (Riemann-style)

use kleis::ast::Expression;
use kleis::render::{build_default_context, render_expression, RenderTarget};

/// Helper to create an Object expression
fn obj(name: &str) -> Expression {
    Expression::Object(name.to_string())
}

/// Helper to create negate(Object) for covariant index
fn neg(name: &str) -> Expression {
    Expression::Operation {
        name: "negate".to_string(),
        args: vec![obj(name)],
    }
}

/// Helper to create an operation (tensor with indices)
fn tensor(name: &str, indices: Vec<Expression>) -> Expression {
    Expression::Operation {
        name: name.to_string(),
        args: indices,
    }
}

#[test]
fn test_xact_mixed_tensor() {
    // T(μ, -ν) should render as T^μ_ν
    let ctx = build_default_context();
    let t = tensor("T", vec![obj("μ"), neg("ν")]);

    let unicode = render_expression(&t, &ctx, &RenderTarget::Unicode);
    let latex = render_expression(&t, &ctx, &RenderTarget::LaTeX);

    println!("T(μ, -ν):");
    println!("  Unicode: {}", unicode);
    println!("  LaTeX: {}", latex);

    assert!(unicode.contains("μ"), "Should contain μ");
    assert!(unicode.contains("ν"), "Should contain ν");
    // Upper index before lower
    assert!(unicode.contains("^"), "Should have superscript marker");
    assert!(unicode.contains("_"), "Should have subscript marker");
}

#[test]
fn test_xact_covariant_metric() {
    // g(-μ, -ν) should render as g_μν (both lower)
    let ctx = build_default_context();
    let g = tensor("g", vec![neg("μ"), neg("ν")]);

    let unicode = render_expression(&g, &ctx, &RenderTarget::Unicode);
    let latex = render_expression(&g, &ctx, &RenderTarget::LaTeX);

    println!("g(-μ, -ν):");
    println!("  Unicode: {}", unicode);
    println!("  LaTeX: {}", latex);

    // Should have subscript but no superscript
    assert!(unicode.contains("_"), "Should have subscript");
    assert!(
        !unicode.contains("^"),
        "Should NOT have superscript (all indices covariant)"
    );
}

#[test]
fn test_xact_contravariant_only() {
    // V(μ) without covariant indices is NOT tensor notation - it's a function call
    // xAct requires at least one covariant (negated) index to distinguish from f(x)
    let ctx = build_default_context();
    let v = tensor("V", vec![obj("μ")]);

    let unicode = render_expression(&v, &ctx, &RenderTarget::Unicode);
    let latex = render_expression(&v, &ctx, &RenderTarget::LaTeX);

    println!("V(μ) [no covariant index = function call]:");
    println!("  Unicode: {}", unicode);
    println!("  LaTeX: {}", latex);

    // Without a covariant index, this renders as a function call, not tensor
    assert!(unicode.contains("V") && unicode.contains("μ"));
    // Should NOT have tensor superscript notation
    assert!(
        !unicode.contains("^{"),
        "Without covariant index, not tensor notation"
    );
}

#[test]
fn test_xact_riemann_tensor() {
    // R(ρ, -σ, -μ, -ν) should render as R^ρ_σμν
    let ctx = build_default_context();
    let r = tensor("R", vec![obj("ρ"), neg("σ"), neg("μ"), neg("ν")]);

    let unicode = render_expression(&r, &ctx, &RenderTarget::Unicode);
    let latex = render_expression(&r, &ctx, &RenderTarget::LaTeX);

    println!("R(ρ, -σ, -μ, -ν):");
    println!("  Unicode: {}", unicode);
    println!("  LaTeX: {}", latex);

    // One upper (ρ), three lower (σμν)
    assert!(unicode.contains("ρ"), "Should contain ρ");
    assert!(unicode.contains("σ"), "Should contain σ");
    assert!(unicode.contains("μ"), "Should contain μ");
    assert!(unicode.contains("ν"), "Should contain ν");
}

#[test]
fn test_xact_christoffel_symbol() {
    // Γ(λ, -μ, -ν) should render as Γ^λ_μν
    let ctx = build_default_context();
    let gamma = tensor("Γ", vec![obj("λ"), neg("μ"), neg("ν")]);

    let unicode = render_expression(&gamma, &ctx, &RenderTarget::Unicode);
    let latex = render_expression(&gamma, &ctx, &RenderTarget::LaTeX);

    println!("Γ(λ, -μ, -ν):");
    println!("  Unicode: {}", unicode);
    println!("  LaTeX: {}", latex);

    assert!(unicode.starts_with("Γ"), "Should start with Γ");
}

#[test]
fn test_xact_latex_output() {
    // Verify LaTeX output format
    let ctx = build_default_context();
    let t = tensor("T", vec![obj("\\mu"), neg("\\nu")]);

    let latex = render_expression(&t, &ctx, &RenderTarget::LaTeX);

    println!("T(\\mu, -\\nu) LaTeX: {}", latex);

    // Should produce proper LaTeX: T^{\mu}_{\nu}
    assert!(latex.contains("^{"), "Should have LaTeX superscript");
    assert!(latex.contains("_{"), "Should have LaTeX subscript");
}

#[test]
fn test_xact_kleis_roundtrip() {
    // Kleis target should preserve original notation
    let ctx = build_default_context();
    let t = tensor("T", vec![obj("μ"), neg("ν")]);

    let kleis = render_expression(&t, &ctx, &RenderTarget::Kleis);

    println!("T(μ, -ν) Kleis: {}", kleis);

    // Should be: T(μ, -ν)
    assert!(kleis.contains("T("), "Should have function call syntax");
    assert!(kleis.contains("-ν"), "Should have negative index");
}

#[test]
fn test_non_xact_falls_through() {
    // Expressions that don't match xAct pattern should render normally
    let ctx = build_default_context();

    // plus(x, y) - not xAct pattern (not all indices)
    let plus_expr = Expression::Operation {
        name: "plus".to_string(),
        args: vec![obj("x"), obj("y")],
        span: None,
    };

    let unicode = render_expression(&plus_expr, &ctx, &RenderTarget::Unicode);

    println!("plus(x, y): {}", unicode);

    // Should NOT render as tensor notation
    // (plus has a template, so it will use that)
    assert!(
        !unicode.contains("^{"),
        "plus should not have tensor superscript"
    );
}
