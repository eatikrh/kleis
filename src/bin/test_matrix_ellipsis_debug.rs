use kleis::parser::parse_latex;
use kleis::render::{build_default_context, render_expression, RenderTarget};

fn main() {
    let latex = r"\begin{bmatrix}a_{11} & \cdots & a_{1n}\\\vdots & \ddots & \vdots\\a_{m1} & \cdots & a_{mn}\end{bmatrix}";
    println!("LaTeX: {}", latex);

    match parse_latex(latex) {
        Ok(expr) => {
            println!("AST: {:#?}", expr);
            let ctx = build_default_context();
            let typst = render_expression(&expr, &ctx, &RenderTarget::Typst);
            println!("\nTypst markup: {}", typst);
        }
        Err(e) => {
            println!("Parse error: {}", e);
        }
    }
}
