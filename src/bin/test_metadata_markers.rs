#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use kleis::ast::Expression;
use kleis::render::{build_default_context, render_expression, RenderTarget};

fn main() {
    // Test fraction with metadata markers
    let frac = Expression::Operation {
        name: "frac".to_string(),
        args: vec![
            Expression::Object("a".to_string()),
            Expression::Object("b".to_string()),
        ],
    };

    let ctx = build_default_context();
    let typst = render_expression(&frac, &ctx, &RenderTarget::Typst);

    println!("Typst with metadata markers:");
    println!("{}", typst);
}
