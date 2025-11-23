// Mathematical layout engine using Typst
//
// ARCHITECTURE DECISION (2024-11-22): Use Typst as the math layout engine
// instead of porting KaTeX or writing from scratch.
//
// Typst is a modern typesetting system written in Rust that provides:
// - Professional-quality math layout (TeX-compatible)
// - Font metrics for Computer Modern and other fonts
// - Extensible symbols (integrals, brackets, etc.)
// - Complex spacing and positioning rules
// - Active development and maintenance
//
// This module provides:
// 1. Adapter layer: Convert Kleis Expression → Typst Content
// 2. Layout extraction: Typst layout → LayoutBox (our interface)
// 3. Placeholder preservation: Mark placeholder positions for interactivity
//
// The layout engine is platform-independent - it only calculates positions
// and dimensions. The actual rendering (SVG, Canvas, native widgets) happens
// in separate renderer modules.
//
// See docs/adr-009-wysiwyg-structural-editor.md for full rationale.

pub mod font_metrics;
pub mod layout_box;
pub mod typst_adapter;
pub mod typst_compiler;

pub use font_metrics::{CharMetrics, ExtensibleChar, FontMetrics};
pub use layout_box::{
    BoundingBox, Color, ElementContent, FontFamily, GlyphPiece, LayoutBox, PositionedElement,
    Stroke, Transform,
};
pub use typst_adapter::{ConversionContext, PlaceholderInfo, expression_to_typst};
pub use typst_compiler::{
    ArgumentBoundingBox, CompiledOutput, PlaceholderPosition, compile_math_to_svg,
    compile_math_to_svg_with_ids, compile_with_semantic_boxes,
};

use crate::ast::Expression;

/// Math style affects sizing and positioning
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathStyle {
    /// Display style - large operators, generous spacing
    Display,
    /// Text style - inline math
    Text,
    /// Script style - superscripts/subscripts
    Script,
    /// ScriptScript style - nested scripts
    ScriptScript,
}

impl MathStyle {
    /// Get the next smaller style
    pub fn smaller(self) -> Self {
        match self {
            MathStyle::Display => MathStyle::Text,
            MathStyle::Text => MathStyle::Script,
            MathStyle::Script => MathStyle::ScriptScript,
            MathStyle::ScriptScript => MathStyle::ScriptScript,
        }
    }

    /// Get font size multiplier for this style
    pub fn font_scale(self) -> f64 {
        match self {
            MathStyle::Display => 1.0,
            MathStyle::Text => 1.0,
            MathStyle::Script => 0.7,
            MathStyle::ScriptScript => 0.5,
        }
    }
}

/// Layout context carries style and metrics
pub struct LayoutContext {
    pub style: MathStyle,
    pub base_font_size: f64,
    pub cramped: bool, // Affects superscript raising
    pub font_metrics: &'static FontMetrics,
}

impl Default for LayoutContext {
    fn default() -> Self {
        LayoutContext {
            style: MathStyle::Display,
            base_font_size: 1.0, // em units
            cramped: false,
            font_metrics: get_default_metrics(),
        }
    }
}

/// Main entry point: layout an expression
pub fn layout_expression(expr: &Expression, context: &LayoutContext) -> LayoutBox {
    match expr {
        Expression::Const(s) => layout_constant(s, context),
        Expression::Object(s) => layout_symbol(s, context),
        Expression::Placeholder { id, hint } => layout_placeholder(*id, hint, context),
        Expression::Operation { name, args } => layout_operation(name, args, context),
    }
}

/// Layout a constant (number)
fn layout_constant(value: &str, context: &LayoutContext) -> LayoutBox {
    // TODO: Implement
    LayoutBox::text(value, context.base_font_size, FontFamily::Main, false)
}

/// Layout a symbol (variable or Greek letter)
fn layout_symbol(symbol: &str, context: &LayoutContext) -> LayoutBox {
    // TODO: Implement proper symbol lookup
    LayoutBox::text(symbol, context.base_font_size, FontFamily::Math, true)
}

/// Layout a placeholder (empty slot to fill)
fn layout_placeholder(id: usize, hint: &str, _context: &LayoutContext) -> LayoutBox {
    const PLACEHOLDER_WIDTH: f64 = 2.0; // em
    const PLACEHOLDER_HEIGHT: f64 = 1.0; // em

    LayoutBox {
        width: PLACEHOLDER_WIDTH,
        height: PLACEHOLDER_HEIGHT * 0.8,
        depth: PLACEHOLDER_HEIGHT * 0.2,
        baseline: PLACEHOLDER_HEIGHT * 0.8,
        children: vec![PositionedElement {
            x: 0.0,
            y: 0.0,
            content: ElementContent::Placeholder {
                id,
                hint: hint.to_string(),
                width: PLACEHOLDER_WIDTH,
                height: PLACEHOLDER_HEIGHT,
            },
        }],
    }
}

/// Layout an operation (fraction, sqrt, etc.)
fn layout_operation(name: &str, args: &[Expression], context: &LayoutContext) -> LayoutBox {
    match name {
        "scalar_divide" => layout_fraction(args, context),
        "sup" => layout_superscript(args, context),
        "sub" => layout_subscript(args, context),
        "sqrt" => layout_square_root(args, context),
        "plus" => layout_binary_op("+", args, context),
        "minus" => layout_binary_op("-", args, context),
        "scalar_multiply" => layout_binary_op("·", args, context),
        _ => layout_fallback(name, args, context),
    }
}

/// Layout a fraction
fn layout_fraction(args: &[Expression], context: &LayoutContext) -> LayoutBox {
    if args.len() != 2 {
        return layout_fallback("fraction", args, context);
    }

    // TODO: Implement TeX fraction layout rules
    // For now, simple placeholder
    LayoutBox::text(
        "(fraction)",
        context.base_font_size,
        FontFamily::Main,
        false,
    )
}

/// Layout superscript
fn layout_superscript(args: &[Expression], context: &LayoutContext) -> LayoutBox {
    if args.len() != 2 {
        return layout_fallback("sup", args, context);
    }

    // TODO: Implement
    LayoutBox::text(
        "(superscript)",
        context.base_font_size,
        FontFamily::Main,
        false,
    )
}

/// Layout subscript
fn layout_subscript(args: &[Expression], context: &LayoutContext) -> LayoutBox {
    if args.len() != 2 {
        return layout_fallback("sub", args, context);
    }

    // TODO: Implement
    LayoutBox::text(
        "(subscript)",
        context.base_font_size,
        FontFamily::Main,
        false,
    )
}

/// Layout square root
fn layout_square_root(args: &[Expression], context: &LayoutContext) -> LayoutBox {
    if args.is_empty() {
        return layout_fallback("sqrt", args, context);
    }

    // TODO: Implement
    LayoutBox::text("(sqrt)", context.base_font_size, FontFamily::Main, false)
}

/// Layout binary operator
fn layout_binary_op(op: &str, args: &[Expression], context: &LayoutContext) -> LayoutBox {
    if args.len() != 2 {
        return layout_fallback(op, args, context);
    }

    // TODO: Implement with proper spacing
    LayoutBox::text(
        &format!("({} {} {})", "left", op, "right"),
        context.base_font_size,
        FontFamily::Main,
        false,
    )
}

/// Fallback for unimplemented operations
fn layout_fallback(name: &str, _args: &[Expression], context: &LayoutContext) -> LayoutBox {
    LayoutBox::text(
        &format!("{}(...)", name),
        context.base_font_size,
        FontFamily::Main,
        false,
    )
}

// Placeholder for font metrics (will be replaced with real data)
// Use lazy_static or once_cell in production
fn get_default_metrics() -> &'static FontMetrics {
    use std::sync::OnceLock;
    static METRICS: OnceLock<FontMetrics> = OnceLock::new();
    METRICS.get_or_init(|| font_metrics::load_default_metrics())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_constant() {
        let expr = Expression::Const("42".to_string());
        let context = LayoutContext::default();
        let layout = layout_expression(&expr, &context);

        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    #[test]
    fn test_layout_placeholder() {
        let expr = Expression::Placeholder {
            id: 0,
            hint: "test".to_string(),
        };
        let context = LayoutContext::default();
        let layout = layout_expression(&expr, &context);

        assert_eq!(layout.width, 2.0);
        assert_eq!(layout.children.len(), 1);

        match &layout.children[0].content {
            ElementContent::Placeholder { id, .. } => {
                assert_eq!(*id, 0);
            }
            _ => panic!("Expected placeholder content"),
        }
    }
}
