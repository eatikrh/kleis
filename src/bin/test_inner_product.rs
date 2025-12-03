use kleis::parser::parse_latex;
use kleis::render::{RenderTarget, build_default_context, render_expression};

fn main() {
    let latex = r"\langle u|v \rangle";
    println!("Input: {}", latex);

    match parse_latex(latex) {
        Ok(expr) => {
            println!("AST: {:#?}", expr);

            let ctx = build_default_context();
            let typst = render_expression(&expr, &ctx, &RenderTarget::Typst);
            println!("\nTypst: {}", typst);
        }
        Err(e) => println!("Error: {:?}", e),
    }
}
