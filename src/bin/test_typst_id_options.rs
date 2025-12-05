//! Test Typst markup options for embedding placeholder IDs
//!
//! The compile_math_to_svg function wraps markup in:
//!   #set page(width: auto, height: auto, margin: 0pt)
//!   #set text(size: 24pt)
//!   #box($ {markup} $)
//!
//! So we only pass MATH-MODE content, not content-mode constructs.

use kleis::math_layout::compile_math_to_svg;

fn main() {
    println!("=== Testing Typst Math-Mode ID Embedding Options ===\n");
    println!("Note: compile_math_to_svg wraps input in #box($ ... $)");
    println!("So all tests below are MATH-MODE content only.\n");

    // Baseline: what we know works
    println!("--- Baseline (known working) ---\n");

    let baseline = vec![
        ("square.stroked", "square.stroked"),
        ("frac(a,b)", "frac(a, b)"),
        ("2x2 matrix", "mat(delim: \"[\", a, b; c, d)"),
    ];

    for (name, markup) in &baseline {
        print!("{:35} ", name);
        match compile_math_to_svg(markup) {
            Ok(out) => println!("✅ {} bytes", out.svg.len()),
            Err(e) => println!("❌ {}", &e[..e.len().min(50)]),
        }
    }

    // Test: Can we use #text(...) inside math mode?
    println!("\n--- Option 1: #text() inside math mode ---\n");

    let text_tests = vec![
        // Basic #text usage in math
        ("text(a)", "#text[a]"),
        ("text size 1pt", "#text(size: 1pt)[X]"),
        ("text white fill", "#text(fill: white)[X]"),
        ("text tiny invisible", "#text(size: 0.1pt, fill: white)[ID]"),
        // Combining with square
        (
            "square + text",
            "square.stroked #text(size: 0.1pt, fill: white)[0]",
        ),
        (
            "text + square",
            "#text(size: 0.1pt, fill: white)[0] square.stroked",
        ),
    ];

    for (name, markup) in &text_tests {
        print!("{:35} ", name);
        match compile_math_to_svg(markup) {
            Ok(out) => {
                println!("✅ {} bytes", out.svg.len());
                // Check if "0" or "ID" appears in SVG
                if out.svg.contains(">0<") || out.svg.contains(">ID<") {
                    println!("    → Text content visible in SVG!");
                }
            }
            Err(e) => println!("❌ {}", &e[..e.len().min(50)]),
        }
    }

    // Test: Matrix with embedded IDs
    println!("\n--- Option 1b: Matrix with embedded text IDs ---\n");

    let matrix_tests = vec![
        (
            "matrix plain",
            "mat(delim: \"[\", square.stroked, square.stroked; square.stroked, square.stroked)",
        ),
        (
            "matrix + trailing text",
            "mat(delim: \"[\", square.stroked #text(size: 0.1pt, fill: white)[0], square.stroked #text(size: 0.1pt, fill: white)[1]; square.stroked #text(size: 0.1pt, fill: white)[2], square.stroked #text(size: 0.1pt, fill: white)[3])",
        ),
        (
            "matrix + angle brackets",
            "mat(delim: \"[\", square.stroked #text(size: 0.1pt, fill: white)[⟨0⟩], square.stroked #text(size: 0.1pt, fill: white)[⟨1⟩]; square.stroked #text(size: 0.1pt, fill: white)[⟨2⟩], square.stroked #text(size: 0.1pt, fill: white)[⟨3⟩])",
        ),
    ];

    for (name, markup) in &matrix_tests {
        print!("{:35} ", name);
        match compile_math_to_svg(markup) {
            Ok(out) => {
                println!("✅ {} bytes", out.svg.len());
                // Check for our markers
                let has_markers =
                    out.svg.contains("⟨0⟩") || out.svg.contains(">0<") || out.svg.contains(">1<");
                if has_markers {
                    println!("    → ID markers found in SVG!");
                }
            }
            Err(e) => println!("❌ {}", &e[..e.len().min(60)]),
        }
    }

    // Test: Using lr() or similar math constructs
    println!("\n--- Option 2: Math-mode grouping ---\n");

    let group_tests = vec![
        ("lr with parens", "lr((square.stroked))"),
        ("attach (sub/super)", "square.stroked^0"),
        ("limits", "limits(square.stroked)_0"),
        ("scripts", "scripts(square.stroked)_0"),
        ("upright text", "upright(\"0\") square.stroked"),
    ];

    for (name, markup) in &group_tests {
        print!("{:35} ", name);
        match compile_math_to_svg(markup) {
            Ok(out) => {
                println!("✅ {} bytes", out.svg.len());
            }
            Err(e) => println!("❌ {}", &e[..e.len().min(50)]),
        }
    }

    // Test: Can we use unique subscripts as IDs?
    println!("\n--- Option 3: Subscript-based IDs (visible but small) ---\n");

    let subscript_tests = vec![
        ("square with sub 0", "square.stroked_0"),
        ("square with sub id", "square.stroked_(\"id0\")"),
        ("tiny sub", "square.stroked_#text(size: 4pt)[0]"),
    ];

    for (name, markup) in &subscript_tests {
        print!("{:35} ", name);
        match compile_math_to_svg(markup) {
            Ok(out) => {
                println!("✅ {} bytes", out.svg.len());
            }
            Err(e) => println!("❌ {}", &e[..e.len().min(50)]),
        }
    }

    // Summary: Check which approach gives us extractable IDs
    println!("\n=== Testing ID extraction from SVG ===\n");

    let test_markup = "mat(delim: \"[\", square.stroked #text(size: 0.1pt, fill: white)[PH0], square.stroked #text(size: 0.1pt, fill: white)[PH1]; square.stroked #text(size: 0.1pt, fill: white)[PH2], square.stroked #text(size: 0.1pt, fill: white)[PH3])";

    match compile_math_to_svg(test_markup) {
        Ok(out) => {
            println!("Compiled matrix with PH0-PH3 markers");
            println!("SVG length: {} bytes", out.svg.len());

            // Search for our markers in SVG
            for id in 0..4 {
                let marker = format!("PH{}", id);
                if out.svg.contains(&marker) {
                    println!("  ✅ Found {} in SVG", marker);
                } else {
                    println!("  ❌ {} NOT in SVG", marker);
                }
            }

            // Show a snippet around any PH marker
            if let Some(pos) = out.svg.find("PH") {
                let start = pos.saturating_sub(100);
                let end = (pos + 150).min(out.svg.len());
                println!("\nSVG snippet around 'PH':");
                println!("{}", &out.svg[start..end]);
            }
        }
        Err(e) => println!("❌ Compilation failed: {}", e),
    }

    println!("\n=== Conclusion ===");
    println!("If PH markers appear in SVG, we can extract them with regex.");
    println!("The key is making them invisible (tiny + white) but still present.");
}
