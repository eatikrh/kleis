// Demo: Full pipeline from Expression → Typst → SVG with placeholder extraction
//
// This demonstrates the complete flow:
// 1. Kleis Expression (with placeholders)
// 2. Render to Typst markup (with markers)
// 3. Compile Typst → SVG (Typst does layout)
// 4. Extract placeholder positions from SVG

use kleis::ast::Expression;
use kleis::math_layout::compile_math_to_svg;
use kleis::render::{RenderTarget, build_default_context, render_expression};

fn main() {
    println!("=== Full Pipeline Demo: Expression → Typst → SVG ===\n");

    // Step 1: Create Expression with placeholder
    let expr = Expression::Operation {
        name: "scalar_divide".to_string(),
        args: vec![
            Expression::Placeholder {
                id: 0,
                hint: "numerator".to_string(),
            },
            Expression::Const("2".to_string()),
        ],
    };

    println!("Step 1: Expression created");
    println!("  Operation: scalar_divide");
    println!("  Args: [Placeholder(0), Const(\"2\")]");
    println!();

    // Step 2: Render to Typst markup
    let ctx = build_default_context();
    let typst_markup = render_expression(&expr, &ctx, &RenderTarget::Typst);

    println!("Step 2: Rendered to Typst markup");
    println!("  Typst: {}", typst_markup);
    println!("  Note: ⟨⟨PH0⟩⟩ is the placeholder marker");
    println!();

    // Step 3: Compile Typst → SVG
    println!("Step 3: Compiling with Typst...");
    match compile_math_to_svg(&typst_markup) {
        Ok(output) => {
            println!("  ✅ Compilation successful!");
            println!("  SVG length: {} bytes", output.svg.len());
            println!(
                "  Placeholders found: {}",
                output.placeholder_positions.len()
            );
            println!();

            if !output.placeholder_positions.is_empty() {
                println!("Step 4: Placeholder positions extracted");
                for (i, ph) in output.placeholder_positions.iter().enumerate() {
                    println!(
                        "  Placeholder {}: ID={}, Position=({:.1}, {:.1}), Size={:.1}x{:.1}",
                        i + 1,
                        ph.id,
                        ph.x,
                        ph.y,
                        ph.width,
                        ph.height
                    );
                }
                println!();
            }

            // Show a snippet of the SVG
            println!("SVG snippet (first 500 chars):");
            println!("{}", &output.svg[..output.svg.len().min(500)]);
            println!("...");
            println!();

            println!("✅ Pipeline complete!");
            println!("Next: Replace markers with clickable <rect> elements");
        }
        Err(e) => {
            println!("  ❌ Compilation failed: {}", e);
            println!();
            println!("Note: This requires Typst CLI to be installed.");
            println!("Install with: brew install typst (macOS)");
            println!("           or cargo install --git https://github.com/typst/typst");
        }
    }
}
