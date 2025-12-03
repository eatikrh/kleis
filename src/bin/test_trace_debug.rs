use kleis::ast::Expression;
use kleis::render::{RenderTarget, build_default_context, render_expression};

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
