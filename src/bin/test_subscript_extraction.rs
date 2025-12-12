#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
//! Test the new subscript-based placeholder ID extraction
//!
//! Verifies that square.stroked_N placeholders are correctly identified
//! using source span extraction from the Typst layout tree.

use kleis::math_layout::compile_math_to_svg;

fn main() {
    println!("=== Testing Subscript-Based Placeholder ID Extraction ===\n");

    // Test 1: Single placeholder with subscript
    println!("--- Test 1: Single placeholder ---");
    let markup1 = "square.stroked_0";
    println!("Markup: {}", markup1);

    match compile_math_to_svg(markup1) {
        Ok(output) => {
            println!("✅ Compiled successfully");
            println!(
                "   Placeholders found: {}",
                output.placeholder_positions.len()
            );
            for ph in &output.placeholder_positions {
                println!(
                    "   - ID {}: ({:.1}, {:.1}) size {:.1}x{:.1}",
                    ph.id, ph.x, ph.y, ph.width, ph.height
                );
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }

    // Test 2: Matrix with 4 subscripted placeholders
    println!("\n--- Test 2: 2x2 Matrix with subscripted placeholders ---");
    let markup2 =
        "mat(delim: \"[\", square.stroked_0, square.stroked_1; square.stroked_2, square.stroked_3)";
    println!("Markup: {}", markup2);

    match compile_math_to_svg(markup2) {
        Ok(output) => {
            println!("✅ Compiled successfully");
            println!(
                "   Placeholders found: {}",
                output.placeholder_positions.len()
            );
            for ph in &output.placeholder_positions {
                println!(
                    "   - ID {}: ({:.1}, {:.1}) size {:.1}x{:.1}",
                    ph.id, ph.x, ph.y, ph.width, ph.height
                );
            }

            // Verify IDs are correct
            let ids: Vec<usize> = output.placeholder_positions.iter().map(|p| p.id).collect();
            if ids == vec![0, 1, 2, 3] {
                println!("\n✅ All 4 placeholder IDs correctly extracted!");
            } else {
                println!("\n⚠️ Expected IDs [0, 1, 2, 3], got {:?}", ids);
            }

            // Check spatial arrangement (row-major order)
            if output.placeholder_positions.len() == 4 {
                let p0 = &output.placeholder_positions[0];
                let p1 = &output.placeholder_positions[1];
                let p2 = &output.placeholder_positions[2];
                let p3 = &output.placeholder_positions[3];

                // ID 0 should be top-left, ID 1 top-right
                // ID 2 should be bottom-left, ID 3 bottom-right
                let row0_ok = p0.y < p2.y && p1.y < p3.y; // top row above bottom
                let col0_ok = p0.x < p1.x && p2.x < p3.x; // left col before right

                if row0_ok && col0_ok {
                    println!("✅ Spatial arrangement correct (row-major order)");
                } else {
                    println!("⚠️ Spatial arrangement may be wrong");
                    println!("   p0 (0,0): ({:.1}, {:.1})", p0.x, p0.y);
                    println!("   p1 (0,1): ({:.1}, {:.1})", p1.x, p1.y);
                    println!("   p2 (1,0): ({:.1}, {:.1})", p2.x, p2.y);
                    println!("   p3 (1,1): ({:.1}, {:.1})", p3.x, p3.y);
                }
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }

    // Test 3: 3x3 Matrix
    println!("\n--- Test 3: 3x3 Matrix with subscripted placeholders ---");
    let markup3 = "mat(delim: \"[\", square.stroked_0, square.stroked_1, square.stroked_2; square.stroked_3, square.stroked_4, square.stroked_5; square.stroked_6, square.stroked_7, square.stroked_8)";
    println!("Markup: {} ...", &markup3[..60]);

    match compile_math_to_svg(markup3) {
        Ok(output) => {
            println!("✅ Compiled successfully");
            println!(
                "   Placeholders found: {}",
                output.placeholder_positions.len()
            );

            let ids: Vec<usize> = output.placeholder_positions.iter().map(|p| p.id).collect();
            let expected: Vec<usize> = (0..9).collect();

            if ids == expected {
                println!("✅ All 9 placeholder IDs correctly extracted!");
            } else {
                println!("⚠️ Expected IDs {:?}, got {:?}", expected, ids);
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }

    // Test 4: Mixed content (using render_expression would be better, but test raw markup)
    println!("\n--- Test 4: Fraction with subscripted placeholders ---");
    let markup4 = "frac(square.stroked_0, square.stroked_1)";
    println!("Markup: {}", markup4);

    match compile_math_to_svg(markup4) {
        Ok(output) => {
            println!("✅ Compiled successfully");
            println!(
                "   Placeholders found: {}",
                output.placeholder_positions.len()
            );
            for ph in &output.placeholder_positions {
                println!("   - ID {}: ({:.1}, {:.1})", ph.id, ph.x, ph.y);
            }

            // Verify numerator (ID 0) is above denominator (ID 1)
            if output.placeholder_positions.len() == 2 {
                let num = &output.placeholder_positions[0];
                let den = &output.placeholder_positions[1];
                if num.y < den.y {
                    println!("✅ Numerator (ID 0) correctly above denominator (ID 1)");
                } else {
                    println!("⚠️ Vertical order may be wrong");
                }
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }

    println!("\n=== Summary ===");
    println!("The subscript-based ID extraction should:");
    println!("1. Find each square.stroked_N in the layout tree");
    println!("2. Extract the subscript digit N using source spans");
    println!("3. Return placeholder positions with correct IDs");
}
