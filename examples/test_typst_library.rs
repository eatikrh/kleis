// Test: Verify Typst library integration works
//
// This tests the new in-process Typst compilation

use kleis::math_layout::compile_math_to_svg_with_ids;

fn main() {
    println!("=== Typst Library Integration Test ===\n");

    // Test 1: Simple fraction with placeholders
    let markup1 = "(#sym.square)/(#sym.square)";
    let placeholder_ids1 = vec![0, 1];

    println!("Test 1: Fraction with 2 placeholders");
    println!("  Markup: {}", markup1);
    println!("  Placeholder IDs: {:?}", placeholder_ids1);

    match compile_math_to_svg_with_ids(markup1, &placeholder_ids1, &placeholder_ids1) {
        Ok(output) => {
            println!("  ✅ Compilation successful!");
            println!("  SVG length: {} bytes", output.svg.len());
            println!(
                "  Placeholders found: {}",
                output.placeholder_positions.len()
            );
            for ph in &output.placeholder_positions {
                println!(
                    "    - ID {}: pos ({:.1}, {:.1}), size {:.1}x{:.1}",
                    ph.id, ph.x, ph.y, ph.width, ph.height
                );
            }
            println!(
                "  Bounding boxes extracted: {}",
                output.argument_bounding_boxes.len()
            );
            for (i, bbox) in output.argument_bounding_boxes.iter().enumerate().take(5) {
                println!(
                    "    {}. pos ({:.1}, {:.1}), size {:.1}x{:.1}, node: {}",
                    i + 1,
                    bbox.x,
                    bbox.y,
                    bbox.width,
                    bbox.height,
                    bbox.node_id
                );
            }
        }
        Err(e) => {
            println!("  ❌ Compilation failed: {}", e);
        }
    }

    println!("\nTest 2: Filled fraction");
    let markup2 = "(x + y)/(2)";
    let placeholder_ids2 = vec![]; // No placeholders

    println!("  Markup: {}", markup2);

    match compile_math_to_svg_with_ids(markup2, &placeholder_ids2, &placeholder_ids2) {
        Ok(output) => {
            println!("  ✅ Compilation successful!");
            println!("  SVG length: {} bytes", output.svg.len());
            println!(
                "  Bounding boxes extracted: {}",
                output.argument_bounding_boxes.len()
            );
            for (i, bbox) in output.argument_bounding_boxes.iter().enumerate() {
                println!(
                    "    {}. pos ({:.1}, {:.1}), size {:.1}x{:.1}, node: {}",
                    i + 1,
                    bbox.x,
                    bbox.y,
                    bbox.width,
                    bbox.height,
                    bbox.node_id
                );
            }
        }
        Err(e) => {
            println!("  ❌ Compilation failed: {}", e);
        }
    }

    println!("\n=== Tests Complete ===");
}
