#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
// Demo: Convert Kleis AST to Typst markup
//
// This demonstrates the first step of the layout pipeline:
// Expression → Typst markup (with placeholder markers)

use kleis::ast::Expression;
use kleis::math_layout::typst_adapter::{expression_to_typst, ConversionContext};

fn main() {
    println!("=== Kleis → Typst Adapter Demo ===\n");

    // Example 1: Simple fraction with placeholder
    let expr1 = Expression::Operation {
        name: "scalar_divide".to_string(),
        args: vec![
            Expression::Placeholder {
                id: 0,
                hint: "numerator".to_string(),
            },
            Expression::Const("2".to_string()),
        ],
        span: None,
    };

    let mut ctx1 = ConversionContext::new();
    let typst1 = expression_to_typst(&expr1, &mut ctx1);

    println!("Example 1: Fraction with placeholder");
    println!("  AST:          scalar_divide(Placeholder(0), Const(\"2\"))");
    println!("  Typst markup: {}", typst1);
    println!(
        "  Placeholders: {} tracked",
        ctx1.placeholder_positions.len()
    );
    println!(
        "    - ID: {}, Hint: {}, Marker: {}",
        ctx1.placeholder_positions[0].id,
        ctx1.placeholder_positions[0].hint,
        ctx1.placeholder_positions[0].marker
    );
    println!();

    // Example 2: Quadratic formula with multiple placeholders
    // x = (-b ± √(b² - 4ac)) / 2a
    let expr2 = Expression::Operation {
        name: "scalar_divide".to_string(),
        args: vec![
            Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Operation {
                        name: "minus".to_string(),
                        args: vec![
                            Expression::Const("0".to_string()),
                            Expression::Placeholder {
                                id: 1,
                                hint: "b".to_string(),
                            },
                        ],
                        span: None,
                    },
                    Expression::Operation {
                        name: "sqrt".to_string(),
                        args: vec![Expression::Operation {
                            name: "minus".to_string(),
                            args: vec![
                                Expression::Operation {
                                    name: "sup".to_string(),
                                    args: vec![
                                        Expression::Placeholder {
                                            id: 2,
                                            hint: "b".to_string(),
                                        },
                                        Expression::Const("2".to_string()),
                                    ],
                                    span: None,
                                },
                                Expression::Operation {
                                    name: "scalar_multiply".to_string(),
                                    args: vec![
                                        Expression::Const("4".to_string()),
                                        Expression::Placeholder {
                                            id: 3,
                                            hint: "a".to_string(),
                                        },
                                    ],
                                    span: None,
                                },
                            ],
                            span: None,
                        }],
                        span: None,
                    },
                ],
                span: None,
            },
            Expression::Operation {
                name: "scalar_multiply".to_string(),
                args: vec![
                    Expression::Const("2".to_string()),
                    Expression::Placeholder {
                        id: 4,
                        hint: "a".to_string(),
                    },
                ],
                span: None,
            },
        ],
        span: None,
    };

    let mut ctx2 = ConversionContext::new();
    let typst2 = expression_to_typst(&expr2, &mut ctx2);

    println!("Example 2: Quadratic formula (simplified)");
    println!("  Typst markup: {}", typst2);
    println!(
        "  Placeholders: {} tracked",
        ctx2.placeholder_positions.len()
    );
    for (i, ph) in ctx2.placeholder_positions.iter().enumerate() {
        println!(
            "    {}. ID: {}, Hint: {}, Marker: {}",
            i + 1,
            ph.id,
            ph.hint,
            ph.marker
        );
    }
    println!();

    // Example 3: Simple superscript
    let expr3 = Expression::Operation {
        name: "sup".to_string(),
        args: vec![
            Expression::Object("x".to_string()),
            Expression::Placeholder {
                id: 5,
                hint: "exponent".to_string(),
            },
        ],
        span: None,
    };

    let mut ctx3 = ConversionContext::new();
    let typst3 = expression_to_typst(&expr3, &mut ctx3);

    println!("Example 3: Superscript with placeholder");
    println!("  AST:          sup(Object(\"x\"), Placeholder(5))");
    println!("  Typst markup: {}", typst3);
    println!("  Visual:       x^(placeholder)");
    println!();

    println!("=== Next Steps ===");
    println!("1. Compile Typst markup → SVG");
    println!("2. Find placeholder markers in SVG");
    println!("3. Replace markers with clickable rectangles");
    println!("4. Wire up click handlers to editor state");
}
