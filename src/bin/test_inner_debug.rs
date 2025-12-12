#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use kleis::ast::Expression;

fn main() {
    fn o(s: &str) -> Expression {
        Expression::Object(s.to_string())
    }
    fn op(name: &str, args: Vec<Expression>) -> Expression {
        Expression::Operation {
            name: name.to_string(),
            args,
        }
    }

    let inner = op("inner", vec![o("u"), o("v")]);
    println!("AST: {:#?}", inner);
    println!(
        "Args count: {}",
        match &inner {
            Expression::Operation { args, .. } => args.len(),
            _ => 0,
        }
    );

    let ctx = kleis::render::build_default_context();
    let typst = kleis::render::render_expression(&inner, &ctx, &kleis::render::RenderTarget::Typst);
    println!("\nTypst: {}", typst);
}
