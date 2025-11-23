use kleis::ast::Expression;
use kleis::render::{build_default_context, render_expression, RenderTarget};
use kleis::math_layout::compile_math_to_svg_with_ids;

fn main() {
    // Simple fraction test
    let frac = Expression::Operation {
        name: "frac".to_string(),
        args: vec![
            Expression::Object("a".to_string()),
            Expression::Object("b".to_string()),
        ],
    };
    
    let ctx = build_default_context();
    let typst = render_expression(&frac, &ctx, &RenderTarget::Typst);
    println!("Typst markup: {}", typst);
    
    match compile_math_to_svg_with_ids(&typst, &[]) {
        Ok(output) => {
            println!("\n=== Bounding Boxes from Backend ===");
            for (i, bbox) in output.argument_bounding_boxes.iter().enumerate() {
                println!("Arg {}: x={:.1}, y={:.1}, w={:.1}, h={:.1}", 
                    i, bbox.x, bbox.y, bbox.width, bbox.height);
            }
            
            println!("\n=== SVG Structure ===");
            // Extract first few lines of SVG to see viewBox and transforms
            let svg_lines: Vec<&str> = output.svg.lines().take(10).collect();
            for line in svg_lines {
                if line.contains("viewBox") || line.contains("transform") || line.contains("<g") {
                    println!("{}", line.trim());
                }
            }
            
            // Look for actual text positions
            println!("\n=== Text Element Positions in SVG ===");
            for line in output.svg.lines() {
                if line.contains("<text") || (line.contains("transform") && line.contains("translate")) {
                    println!("{}", line.trim());
                    break;
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
