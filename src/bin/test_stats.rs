#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use kleis::parser::parse_latex;
use kleis::render::{build_default_context, render_expression, RenderTarget};

fn main() {
    let tests = vec![r"\mathrm{Var}(X)", r"\mathrm{Cov}(X, Y)", r"\mathrm{Tr}(A)"];

    for latex in tests {
        println!("Input: {}", latex);
        match parse_latex(latex) {
            Ok(expr) => {
                let ctx = build_default_context();
                let typst = render_expression(&expr, &ctx, &RenderTarget::Typst);
                println!("  Typst: {}", typst);
            }
            Err(e) => println!("  Error: {:?}", e),
        }
        println!();
    }
}
