pub mod ast;
pub mod data_registry;
pub mod kleis_ast;
pub mod kleis_parser;
pub mod math_layout;
pub mod parser;
pub mod pattern_matcher;
pub mod render;
pub mod signature_interpreter;
pub mod template_inference;
pub mod templates;
pub mod type_checker;
pub mod type_context;
pub mod type_inference;

/// Convenience function: Parse LaTeX and render to Unicode
pub fn latex_to_unicode(latex: &str) -> Result<String, parser::ParseError> {
    let expr = parser::parse_latex(latex)?;
    let ctx = render::build_default_context();
    Ok(render::render_expression(
        &expr,
        &ctx,
        &render::RenderTarget::Unicode,
    ))
}

/// Convenience function: Parse LaTeX and render to LaTeX
pub fn latex_to_latex(latex: &str) -> Result<String, parser::ParseError> {
    let expr = parser::parse_latex(latex)?;
    let ctx = render::build_default_context();
    Ok(render::render_expression(
        &expr,
        &ctx,
        &render::RenderTarget::LaTeX,
    ))
}
