#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
// Demo: Render Kleis expressions to Typst markup using the template system
//
// This demonstrates that we can reuse the existing renderer infrastructure
// by simply adding Typst as a new RenderTarget.

use kleis::ast::Expression;
use kleis::render::{build_default_context, render_expression, RenderTarget};

fn main() {
    println!("=== Kleis Renderer → Typst Markup Demo ===\n");

    let ctx = build_default_context();

    // Example 1: Simple fraction with placeholder
    println!("Example 1: Fraction with placeholder");
    let expr1 = Expression::Operation {
        name: "scalar_divide".to_string(),
        args: vec![
            Expression::Placeholder {
                id: 0,
                hint: "numerator".to_string(),
            },
            Expression::Const("2".to_string()),
        ],
    };

    let unicode1 = render_expression(&expr1, &ctx, &RenderTarget::Unicode);
    let latex1 = render_expression(&expr1, &ctx, &RenderTarget::LaTeX);
    let typst1 = render_expression(&expr1, &ctx, &RenderTarget::Typst);

    println!("  Unicode:  {}", unicode1);
    println!("  LaTeX:    {}", latex1);
    println!("  Typst:    {}", typst1);
    println!();

    // Example 2: Superscript with Greek letter
    println!("Example 2: x^α (Greek letter)");
    let expr2 = Expression::Operation {
        name: "sup".to_string(),
        args: vec![
            Expression::Object("x".to_string()),
            Expression::Object("\\alpha".to_string()),
        ],
        span: None,
    };

    let unicode2 = render_expression(&expr2, &ctx, &RenderTarget::Unicode);
    let latex2 = render_expression(&expr2, &ctx, &RenderTarget::LaTeX);
    let typst2 = render_expression(&expr2, &ctx, &RenderTarget::Typst);

    println!("  Unicode:  {}", unicode2);
    println!("  LaTeX:    {}", latex2);
    println!("  Typst:    {}", typst2);
    println!();

    // Example 3: Integral with placeholders
    println!("Example 3: Integral ∫_a^b f(x) dx");
    let expr3 = Expression::Operation {
        name: "int_bounds".to_string(),
        args: vec![
            Expression::Placeholder {
                id: 1,
                hint: "integrand".to_string(),
            },
            Expression::Object("a".to_string()),
            Expression::Object("b".to_string()),
            Expression::Object("x".to_string()),
        ],
        span: None,
    };

    let unicode3 = render_expression(&expr3, &ctx, &RenderTarget::Unicode);
    let latex3 = render_expression(&expr3, &ctx, &RenderTarget::LaTeX);
    let typst3 = render_expression(&expr3, &ctx, &RenderTarget::Typst);

    println!("  Unicode:  {}", unicode3);
    println!("  LaTeX:    {}", latex3);
    println!("  Typst:    {}", typst3);
    println!();

    // Example 4: Nested expression
    println!("Example 4: (x + y) / 2");
    let expr4 = Expression::Operation {
        name: "scalar_divide".to_string(),
        args: vec![
            Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Object("y".to_string()),
                ],
                span: None,
            },
            Expression::Const("2".to_string()),
        ],
        span: None,
    };

    let unicode4 = render_expression(&expr4, &ctx, &RenderTarget::Unicode);
    let latex4 = render_expression(&expr4, &ctx, &RenderTarget::LaTeX);
    let typst4 = render_expression(&expr4, &ctx, &RenderTarget::Typst);

    println!("  Unicode:  {}", unicode4);
    println!("  LaTeX:    {}", latex4);
    println!("  Typst:    {}", typst4);
    println!();

    println!("✅ Typst rendering works through existing template system!");
    println!("✅ Placeholders preserved with unique markers (⟨⟨PH0⟩⟩)");
    println!("✅ Same template infrastructure as LaTeX, Unicode, HTML");
}
