#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use kleis::ast::Expression;
use kleis::render::{build_default_context, render_expression, RenderTarget};

fn main() {
    let expr = Expression::operation(
        "equals",
        vec![
            Expression::operation("trace", vec![Expression::Object("\u{03c1}".to_string())]),
            Expression::Const("1".to_string()),
        ],
    );
    let ctx = build_default_context();
    let typst = render_expression(&expr, &ctx, &RenderTarget::Typst);
    println!("Typst: {}", typst);
}
