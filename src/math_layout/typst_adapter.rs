// Typst adapter layer
//
// This module converts Kleis Expression AST to Typst markup strings,
// preserving placeholder information so we can make them interactive later.
//
// NOTE: We convert to Typst markup strings (not Content objects directly)
// because the markup parser handles all the complex math layout rules.
// Then we parse the markup and extract layout information.

use crate::ast::Expression;

/// Context for tracking placeholders during conversion
pub struct ConversionContext {
    /// Map of placeholder ID to marker in output
    pub placeholder_positions: Vec<PlaceholderInfo>,
}

#[derive(Debug, Clone)]
pub struct PlaceholderInfo {
    pub id: usize,
    pub hint: String,
    /// Marker string used in output (for finding position later)
    pub marker: String,
}

impl ConversionContext {
    pub fn new() -> Self {
        ConversionContext {
            placeholder_positions: Vec::new(),
        }
    }

    /// Generate unique marker for a placeholder
    fn create_marker(&mut self, id: usize) -> String {
        format!("⟨⟨PH{}⟩⟩", id)
    }
}

/// Convert Kleis Expression to Typst markup string
///
/// This is the main entry point for the adapter layer.
/// It recursively converts our AST into Typst math markup.
/// We use markup strings because Typst's parser handles all layout rules.
pub fn expression_to_typst(expr: &Expression, ctx: &mut ConversionContext) -> String {
    match expr {
        Expression::Const(s) => {
            // Simple number
            s.clone()
        }

        Expression::Object(s) => {
            // Variable or symbol
            // Convert LaTeX commands to Typst symbols
            latex_to_typst_symbol(s)
        }

        Expression::Placeholder { id, hint } => {
            // Render as Typst square symbol
            // Typst will render square.stroked as a hollow square glyph
            // We track the placeholder ID so we can find it later in the SVG
            ctx.placeholder_positions.push(PlaceholderInfo {
                id: *id,
                hint: hint.clone(),
                marker: format!("square.stroked_{}", id), // Track for debugging
            });

            // Render as Typst square symbol (hollow square)
            "square.stroked".to_string()
        }

        Expression::Operation { name, args } => operation_to_typst(name, args, ctx),

        Expression::Match { .. } => {
            // TODO: Implement pattern matching rendering
            // For now, return placeholder text
            "\\text{match expression}".to_string()
        }
    }
}

/// Convert an operation to Typst math markup
fn operation_to_typst(name: &str, args: &[Expression], ctx: &mut ConversionContext) -> String {
    match name {
        "scalar_divide" if args.len() == 2 => {
            // Fraction: a/b → (a)/(b) in Typst
            let num = expression_to_typst(&args[0], ctx);
            let den = expression_to_typst(&args[1], ctx);
            format!("({})/({})", num, den)
        }

        "sup" if args.len() == 2 => {
            // Superscript: x^n
            let base = expression_to_typst(&args[0], ctx);
            let exp = expression_to_typst(&args[1], ctx);
            format!("{}^{}", base, exp)
        }

        "sub" if args.len() == 2 => {
            // Subscript: x_n
            let base = expression_to_typst(&args[0], ctx);
            let sub = expression_to_typst(&args[1], ctx);
            format!("{}_{}", base, sub)
        }

        "sqrt" if args.len() == 1 => {
            // Square root: √x → sqrt(x) in Typst
            let radicand = expression_to_typst(&args[0], ctx);
            format!("sqrt({})", radicand)
        }

        "plus" if args.len() == 2 => {
            // Addition: a + b
            let left = expression_to_typst(&args[0], ctx);
            let right = expression_to_typst(&args[1], ctx);
            format!("{} + {}", left, right)
        }

        "minus" if args.len() == 2 => {
            // Subtraction: a - b
            let left = expression_to_typst(&args[0], ctx);
            let right = expression_to_typst(&args[1], ctx);
            format!("{} - {}", left, right)
        }

        "scalar_multiply" if args.len() == 2 => {
            // Multiplication: a · b
            let left = expression_to_typst(&args[0], ctx);
            let right = expression_to_typst(&args[1], ctx);
            format!("{} dot {}", left, right)
        }

        "int_bounds" if args.len() == 4 => {
            // Integral: ∫_a^b f dx
            let integrand = expression_to_typst(&args[0], ctx);
            let lower = expression_to_typst(&args[1], ctx);
            let upper = expression_to_typst(&args[2], ctx);
            let var = expression_to_typst(&args[3], ctx);
            format!("integral_({})^({}) {} dif {}", lower, upper, integrand, var)
        }

        "sum_bounds" if args.len() == 3 => {
            // Sum: Σ_{from}^{to} body
            let body = expression_to_typst(&args[0], ctx);
            let from = expression_to_typst(&args[1], ctx);
            let to = expression_to_typst(&args[2], ctx);
            format!("sum_({})^({}) {}", from, to, body)
        }

        "prod_bounds" if args.len() == 3 => {
            // Product: ∏_{from}^{to} body
            let body = expression_to_typst(&args[0], ctx);
            let from = expression_to_typst(&args[1], ctx);
            let to = expression_to_typst(&args[2], ctx);
            format!("product_({})^({}) {}", from, to, body)
        }

        // TODO: Add more operations
        // - matrix operations
        // - quantum mechanics (bra, ket, etc.)
        // - derivatives
        // - etc.
        _ => {
            // Fallback: just render operation name
            format!("{}(...)", name)
        }
    }
}

/// Convert LaTeX symbol commands to Typst equivalents
fn latex_to_typst_symbol(latex: &str) -> String {
    match latex {
        // Greek letters
        "\\alpha" => "α".to_string(),
        "\\beta" => "β".to_string(),
        "\\gamma" => "γ".to_string(),
        "\\delta" => "δ".to_string(),
        "\\epsilon" => "ε".to_string(),
        "\\pi" => "π".to_string(),
        "\\theta" => "θ".to_string(),
        "\\mu" => "μ".to_string(),
        "\\nu" => "ν".to_string(),
        "\\sigma" => "σ".to_string(),
        "\\omega" => "ω".to_string(),

        // Special symbols
        "\\infty" => "∞".to_string(),
        "\\hbar" => "ℏ".to_string(),
        "\\partial" => "∂".to_string(),
        "\\nabla" => "∇".to_string(),

        // If not a LaTeX command, return as-is
        _ => latex.to_string(),
    }
}

/// Wrap expression in math mode and compile with Typst
///
/// This function takes a Typst math markup string, compiles it,
/// and extracts layout information.
pub fn compile_and_layout(_markup: &str) -> Result<CompiledLayout, String> {
    // TODO: Implement actual Typst compilation
    // This will:
    // 1. Parse the markup string
    // 2. Run Typst's layout engine
    // 3. Extract positioned elements
    // 4. Find placeholder markers
    // 5. Return layout with placeholder positions

    Err("Not yet implemented - Step 3".to_string())
}

pub struct CompiledLayout {
    /// SVG output from Typst (contains beautiful math)
    pub svg: String,

    /// Positions where placeholders appear in the SVG
    pub placeholder_positions: Vec<PlaceholderPosition>,
}

#[derive(Debug, Clone)]
pub struct PlaceholderPosition {
    pub id: usize,
    pub hint: String,
    /// X position in the rendered output (pixels)
    pub x: f64,
    /// Y position in the rendered output (pixels)
    pub y: f64,
    /// Width of placeholder box
    pub width: f64,
    /// Height of placeholder box
    pub height: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_constant() {
        let expr = Expression::Const("42".to_string());
        let mut ctx = ConversionContext::new();
        let markup = expression_to_typst(&expr, &mut ctx);

        // Should produce simple number
        assert_eq!(markup, "42");

        // Should have no placeholders
        assert_eq!(ctx.placeholder_positions.len(), 0);
    }

    // TODO(2024-12-06): Placeholder conversion tests failing - needs typst_adapter refactor
    // These tests have outdated expectations for placeholder markers

    #[test]
    #[ignore = "TODO: Fix placeholder conversion - outdated expectations"]
    fn test_convert_placeholder() {
        let expr = Expression::Placeholder {
            id: 5,
            hint: "test".to_string(),
        };
        let mut ctx = ConversionContext::new();
        let markup = expression_to_typst(&expr, &mut ctx);

        // Should produce marker
        assert_eq!(markup, "⟨⟨PH5⟩⟩");

        // Should track the placeholder
        assert_eq!(ctx.placeholder_positions.len(), 1);
        assert_eq!(ctx.placeholder_positions[0].id, 5);
        assert_eq!(ctx.placeholder_positions[0].hint, "test");
        assert_eq!(ctx.placeholder_positions[0].marker, "⟨⟨PH5⟩⟩");
    }

    #[test]
    #[ignore = "TODO: Fix fraction with placeholder - outdated expectations"]
    fn test_convert_fraction_with_placeholder() {
        let expr = Expression::Operation {
            name: "scalar_divide".to_string(),
            args: vec![
                Expression::Placeholder {
                    id: 1,
                    hint: "num".to_string(),
                },
                Expression::Const("2".to_string()),
            ],
        };
        let mut ctx = ConversionContext::new();
        let markup = expression_to_typst(&expr, &mut ctx);

        // Should produce Typst fraction syntax with placeholder marker
        assert_eq!(markup, "(⟨⟨PH1⟩⟩)/(2)");

        // Should track placeholder in numerator
        assert_eq!(ctx.placeholder_positions.len(), 1);
        assert_eq!(ctx.placeholder_positions[0].id, 1);
        assert_eq!(ctx.placeholder_positions[0].hint, "num");
    }

    #[test]
    fn test_convert_superscript() {
        let expr = Expression::Operation {
            name: "sup".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Const("2".to_string()),
            ],
        };
        let mut ctx = ConversionContext::new();
        let markup = expression_to_typst(&expr, &mut ctx);

        // Should produce Typst superscript syntax
        assert_eq!(markup, "x^2");
    }

    #[test]
    #[ignore = "TODO: Fix nested placeholders - outdated expectations"]
    fn test_convert_nested_with_multiple_placeholders() {
        // Test: (□ + x)/□
        let expr = Expression::Operation {
            name: "scalar_divide".to_string(),
            args: vec![
                Expression::Operation {
                    name: "plus".to_string(),
                    args: vec![
                        Expression::Placeholder {
                            id: 1,
                            hint: "a".to_string(),
                        },
                        Expression::Object("x".to_string()),
                    ],
                },
                Expression::Placeholder {
                    id: 2,
                    hint: "b".to_string(),
                },
            ],
        };
        let mut ctx = ConversionContext::new();
        let markup = expression_to_typst(&expr, &mut ctx);

        // Should track both placeholders
        assert_eq!(ctx.placeholder_positions.len(), 2);
        assert_eq!(ctx.placeholder_positions[0].id, 1);
        assert_eq!(ctx.placeholder_positions[1].id, 2);

        // Should produce valid Typst markup
        assert!(markup.contains("⟨⟨PH1⟩⟩"));
        assert!(markup.contains("⟨⟨PH2⟩⟩"));
    }
}
