#![cfg_attr(test, allow(warnings))]

pub mod ast;
pub mod axiom_verifier; // Z3 integration for axiom verification
pub mod data_registry;
pub mod evaluator; // Wire 3: Self-hosting
pub mod kleis_ast;
pub mod kleis_parser;
pub mod math_layout;
pub mod parser;
pub mod pattern_matcher;
pub mod pretty_print; // Pretty-printer for exporting Kleis source
pub mod render;
pub mod signature_interpreter;
pub mod solvers; // Pluggable solver backends (Z3, CVC5, etc.)
pub mod structure_registry;
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
