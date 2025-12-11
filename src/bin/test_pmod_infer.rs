use kleis::parser::parse_latex;
use kleis::render::{build_default_context, render_expression, RenderTarget};

fn main() {
    let latex = r"a \equiv b \pmod{n}";
    println!("Input: {}", latex);

    match parse_latex(latex) {
        Ok(expr) => {
            println!("AST: {:#?}", expr);

            let ctx = build_default_context();
            let latex_out = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
            let typst_out = render_expression(&expr, &ctx, &RenderTarget::Typst);
            println!("\nLaTeX output: {}", latex_out);
            println!("Typst output: {}", typst_out);
        }
        Err(e) => println!("Error: {:?}", e),
    }
}
