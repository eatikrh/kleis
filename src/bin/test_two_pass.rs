use kleis::ast::Expression;
use kleis::math_layout::compile_with_semantic_boxes;

fn main() {
    // Test fraction
    let frac = Expression::Operation {
        name: "frac".to_string(),
        args: vec![
            Expression::Object("a".to_string()),
            Expression::Object("b".to_string()),
        ],
    };

    println!("Testing two-pass rendering for: frac(a, b)\n");

    match compile_with_semantic_boxes(&frac, &[]) {
        Ok(output) => {
            println!("\n✓ Compilation successful");
            println!(
                "Argument bounding boxes: {}",
                output.argument_bounding_boxes.len()
            );
            for bbox in &output.argument_bounding_boxes {
                println!(
                    "  Arg {}: node_id={}, x={:.1}, y={:.1}, w={:.1}, h={:.1}",
                    bbox.arg_index, bbox.node_id, bbox.x, bbox.y, bbox.width, bbox.height
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
