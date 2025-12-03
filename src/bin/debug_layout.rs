use kleis::ast::Expression;
use kleis::math_layout::compile_math_to_svg_with_ids;
use kleis::render::{RenderTarget, build_default_context, render_expression};

fn main() {
    println!("=== Debugging Typst Layout Grouping ===\n");

    // AST for (x+y+z+7) / (x + 3/x)
    // Constructing: scalar_divide( x+y+z+7, x + scalar_divide(3, x) )

    // Numerator: x+y+z+7
    let num = Expression::Operation {
        name: "plus".to_string(),
        args: vec![
            Expression::Object("x".to_string()),
            Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Object("y".to_string()),
                    Expression::Operation {
                        name: "plus".to_string(),
                        args: vec![
                            Expression::Object("z".to_string()),
                            Expression::Const("7".to_string()),
                        ],
                    },
                ],
            },
        ],
    };

    // Denominator: x + 3/x
    let den = Expression::Operation {
        name: "plus".to_string(),
        args: vec![
            Expression::Object("x".to_string()),
            Expression::Operation {
                name: "scalar_divide".to_string(),
                args: vec![
                    Expression::Const("3".to_string()),
                    Expression::Object("x".to_string()),
                ],
            },
        ],
    };

    // Main fraction
    let expr = Expression::Operation {
        name: "scalar_divide".to_string(),
        args: vec![num, den],
    };

    // 1. Render to Typst
    let ctx = build_default_context();
    let typst_markup = render_expression(&expr, &ctx, &RenderTarget::Typst);
    println!("Typst markup: {}", typst_markup);

    // 2. Compile and extract layout
    // We don't have placeholders here, so we pass empty list
    match compile_math_to_svg_with_ids(&typst_markup, &[]) {
        Ok(output) => {
            println!("\n--- Final Bounding Boxes ---");
            for (i, bbox) in output.argument_bounding_boxes.iter().enumerate() {
                println!(
                    "Group {}: x={:.2}, y={:.2}, w={:.2}, h={:.2}",
                    i, bbox.x, bbox.y, bbox.width, bbox.height
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
