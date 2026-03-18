#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use kleis::ast::Expression;
use kleis::render::{build_default_context, render_expression, RenderTarget};

fn main() {
    let placeholder = Expression::Placeholder {
        id: 0,
        hint: "test".to_string(),
    };

    let ctx = build_default_context();
    let typst = render_expression(&placeholder, &ctx, &RenderTarget::Typst);

    println!("Placeholder renders as: '{}'", typst);
    println!("Length: {}", typst.len());
}
