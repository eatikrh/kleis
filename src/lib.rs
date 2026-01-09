#![cfg_attr(test, allow(warnings))]

pub mod ast;
pub mod axiom_verifier; // Z3 integration for axiom verification
pub mod config; // Shared configuration loader
pub mod context; // Shared context for LSP/REPL/Debugger
pub mod dap; // Debug Adapter Protocol implementation
pub mod data_registry;
pub mod debug; // Debug hooks for step-through debugging
pub mod dimension_solver; // Built-in solver for type-level dimension constraints
pub mod editor_ast; // Visual Editor AST (separate from Kleis Core AST)
pub mod editor_type_translator; // Translates EditorNode types to canonical Kleis types
pub mod evaluator; // Wire 3: Self-hosting
pub mod kleis_ast;
pub mod kleis_parser;
pub mod kleist_parser; // .kleist template file parser
pub mod logging; // File-based logging (avoids stdio interference)
pub mod lowering;
pub mod math_layout;
#[cfg(feature = "numerical")]
pub mod numerical; // BLAS/LAPACK backend for numerical linear algebra
pub mod parser;
pub mod pattern_matcher;
pub mod plotting; // Lilaq/Typst plotting integration
pub mod pretty_print; // Pretty-printer for exporting Kleis source
pub mod provenance; // Track which file each definition came from
pub mod render;
pub mod render_editor; // EditorNode-only renderer (no Expression conversion)
pub mod repl; // REPL implementation
pub mod signature_interpreter;
pub mod solvers; // Pluggable solver backends (Z3, CVC5, etc.)
pub mod structure_registry;
pub mod template_inference;
pub mod templates;
pub mod type_checker;
pub mod type_context;
pub mod type_inference;
pub mod typed_ast; // Typed AST for operator overloading (semantic lowering) // Semantic lowering pass (operator overloading)

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
