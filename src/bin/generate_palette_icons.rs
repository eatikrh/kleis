#!/usr/bin/env rust
#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
//! Generate SVG icons for palette buttons
//!
//! Creates small, clean SVG images for each template that can be used
//! as button backgrounds or inline images in the palette.

use kleis::math_layout::typst_compiler::compile_math_to_svg;
use std::fs;
use std::path::Path;

/// Template definition with visual example
struct TemplateIcon {
    name: &'static str,
    typst_markup: &'static str,
    category: &'static str,
}

fn get_template_icons() -> Vec<TemplateIcon> {
    vec![
        // Basic Operations
        TemplateIcon {
            name: "fraction",
            typst_markup: "$a/b$",
            category: "basic",
        },
        TemplateIcon {
            name: "sqrt",
            typst_markup: "$sqrt(x)$",
            category: "basic",
        },
        TemplateIcon {
            name: "nthroot",
            typst_markup: "$root(n, x)$",
            category: "basic",
        },
        TemplateIcon {
            name: "power",
            typst_markup: "$x^n$",
            category: "basic",
        },
        TemplateIcon {
            name: "subscript",
            typst_markup: "$x_i$",
            category: "basic",
        },
        TemplateIcon {
            name: "tensor_mixed",
            typst_markup: "$T^i_j$",
            category: "basic",
        },
        TemplateIcon {
            name: "subsup",
            typst_markup: "$T_j^i$",
            category: "basic",
        },
        // Brackets
        TemplateIcon {
            name: "parens",
            typst_markup: "$(x)$",
            category: "brackets",
        },
        TemplateIcon {
            name: "brackets",
            typst_markup: "$[x]$",
            category: "brackets",
        },
        TemplateIcon {
            name: "braces",
            typst_markup: "${x}$",
            category: "brackets",
        },
        TemplateIcon {
            name: "angle_brackets",
            typst_markup: "$angle.l x angle.r$",
            category: "brackets",
        },
        TemplateIcon {
            name: "abs",
            typst_markup: "$|x|$",
            category: "brackets",
        },
        TemplateIcon {
            name: "norm",
            typst_markup: "$norm(v)$",
            category: "brackets",
        },
        TemplateIcon {
            name: "floor",
            typst_markup: "$floor(x)$",
            category: "brackets",
        },
        TemplateIcon {
            name: "ceiling",
            typst_markup: "$ceil(x)$",
            category: "brackets",
        },
        // Calculus
        TemplateIcon {
            name: "integral",
            typst_markup: "$integral_a^b f dif x$",
            category: "calculus",
        },
        TemplateIcon {
            name: "sum",
            typst_markup: "$sum_(i=1)^n a_i$",
            category: "calculus",
        },
        TemplateIcon {
            name: "product",
            typst_markup: "$product_(i=1)^n a_i$",
            category: "calculus",
        },
        TemplateIcon {
            name: "limit",
            typst_markup: "$lim_(x arrow 0) f(x)$",
            category: "calculus",
        },
        TemplateIcon {
            name: "partial",
            typst_markup: "$(diff f)/(diff x)$",
            category: "calculus",
        },
        TemplateIcon {
            name: "gradient",
            typst_markup: "$nabla f$",
            category: "calculus",
        },
        // Tensors
        TemplateIcon {
            name: "tensor_upper_pair",
            typst_markup: "$T^(mu nu)$",
            category: "tensors",
        },
        TemplateIcon {
            name: "tensor_lower_pair",
            typst_markup: "$g_(mu nu)$",
            category: "tensors",
        },
        TemplateIcon {
            name: "tensor_1up_3down",
            typst_markup: "$R^rho_(sigma mu nu)$",
            category: "tensors",
        },
        TemplateIcon {
            name: "tensor_2up_2down",
            typst_markup: "$R^(mu nu)_(rho sigma)$",
            category: "tensors",
        },
        // Matrices
        TemplateIcon {
            name: "matrix2x2",
            typst_markup: "$mat(a, b; c, d)$",
            category: "matrices",
        },
        TemplateIcon {
            name: "matrix3x3",
            typst_markup: "$mat(a, b, c; d, e, f; g, h, i)$",
            category: "matrices",
        },
        TemplateIcon {
            name: "pmatrix2x2",
            typst_markup: "$(mat(a, b; c, d))$",
            category: "matrices",
        },
        TemplateIcon {
            name: "vmatrix2x2",
            typst_markup: "$det(mat(a, b; c, d))$",
            category: "matrices",
        },
        // Quantum
        TemplateIcon {
            name: "ket",
            typst_markup: "$|psi angle.r$",
            category: "quantum",
        },
        TemplateIcon {
            name: "bra",
            typst_markup: "$angle.l phi|$",
            category: "quantum",
        },
        TemplateIcon {
            name: "inner",
            typst_markup: "$angle.l phi|psi angle.r$",
            category: "quantum",
        },
        TemplateIcon {
            name: "outer",
            typst_markup: "$|psi angle.r angle.l phi|$",
            category: "quantum",
        },
        // Functions
        TemplateIcon {
            name: "sin",
            typst_markup: "$sin(x)$",
            category: "functions",
        },
        TemplateIcon {
            name: "cos",
            typst_markup: "$cos(x)$",
            category: "functions",
        },
        TemplateIcon {
            name: "ln",
            typst_markup: "$ln(x)$",
            category: "functions",
        },
        TemplateIcon {
            name: "exp",
            typst_markup: "$e^x$",
            category: "functions",
        },
    ]
}

fn main() {
    println!("🎨 Generating palette icon SVGs...\n");

    let output_dir = Path::new("static/palette_icons");
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    let templates = get_template_icons();
    let mut success_count = 0;
    let mut failed = Vec::new();

    for template in &templates {
        print!("  Rendering {}... ", template.name);

        match compile_math_to_svg(template.typst_markup) {
            Ok(compiled) => {
                let filename = format!("{}.svg", template.name);
                let filepath = output_dir.join(&filename);

                // Optimize SVG for small size
                let optimized_svg = optimize_svg_for_button(&compiled.svg);

                match fs::write(&filepath, optimized_svg) {
                    Ok(_) => {
                        println!("✓");
                        success_count += 1;
                    }
                    Err(e) => {
                        println!("✗ (write failed: {})", e);
                        failed.push(template.name);
                    }
                }
            }
            Err(e) => {
                println!("✗ (render failed: {})", e);
                failed.push(template.name);
            }
        }
    }

    println!("\n📊 Summary:");
    println!("  ✓ Success: {}/{}", success_count, templates.len());
    if !failed.is_empty() {
        println!("  ✗ Failed: {:?}", failed);
    }
    println!("\n📁 Icons saved to: {}", output_dir.display());

    // Generate CSS helper
    generate_css_helper(output_dir);

    // Generate HTML example
    generate_html_example(output_dir, &templates);
}

/// Optimize SVG for use as small button icon
fn optimize_svg_for_button(svg: &str) -> String {
    // Remove width/height to make it scale-able
    let mut optimized = svg.replace("width=\"", "data-width=\"");
    optimized = optimized.replace("height=\"", "data-height=\"");

    // Add viewBox if missing (extract from original width/height)
    if !optimized.contains("viewBox") {
        // Extract dimensions and add viewBox
        optimized = optimized.replace(
            "<svg ",
            "<svg viewBox=\"0 0 100 50\" preserveAspectRatio=\"xMidYMid meet\" ",
        );
    }

    // Make background transparent
    optimized = optimized.replace("fill=\"#ffffff\"", "fill=\"none\"");
    optimized = optimized.replace("fill=\"white\"", "fill=\"none\"");

    optimized
}

/// Generate CSS helper file
fn generate_css_helper(output_dir: &Path) {
    let css = r#"/* Palette Icon Styles */

.palette-icon-btn {
    width: 80px;
    height: 40px;
    padding: 4px;
    border: 1px solid #ddd;
    border-radius: 4px;
    background: white;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
}

.palette-icon-btn:hover {
    border-color: #4CAF50;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    transform: translateY(-1px);
}

.palette-icon-btn:active {
    background: #f0f0f0;
    transform: translateY(0);
}

.palette-icon-btn svg {
    max-width: 100%;
    max-height: 100%;
    display: block;
}

/* Tooltip for template names */
.palette-icon-btn[data-tooltip]::after {
    content: attr(data-tooltip);
    position: absolute;
    bottom: -30px;
    left: 50%;
    transform: translateX(-50%);
    padding: 4px 8px;
    background: rgba(0,0,0,0.8);
    color: white;
    font-size: 11px;
    border-radius: 3px;
    white-space: nowrap;
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.2s;
}

.palette-icon-btn:hover[data-tooltip]::after {
    opacity: 1;
}

/* Category sections */
.palette-section {
    margin-bottom: 20px;
}

.palette-section-title {
    font-size: 13px;
    font-weight: 600;
    color: #555;
    margin-bottom: 8px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.palette-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
    gap: 8px;
}
"#;

    let css_path = output_dir.join("palette_icons.css");
    fs::write(css_path, css).expect("Failed to write CSS");
    println!("📝 Generated CSS helper");
}

/// Generate HTML example page
fn generate_html_example(output_dir: &Path, templates: &[TemplateIcon]) {
    let mut html = String::from(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Kleis Palette Icons</title>
    <link rel="stylesheet" href="palette_icons.css">
    <style>
        body { 
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
            padding: 40px;
            background: #f5f5f5;
        }
        .container { 
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }
        h1 { color: #333; margin-bottom: 10px; }
        .subtitle { color: #666; margin-bottom: 30px; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎨 Kleis Palette Icons</h1>
        <p class="subtitle">Click any icon to copy its template code</p>
"#,
    );

    // Group by category
    let mut categories: std::collections::HashMap<&str, Vec<&TemplateIcon>> =
        std::collections::HashMap::new();

    for template in templates {
        categories
            .entry(template.category)
            .or_insert_with(Vec::new)
            .push(template);
    }

    for (category, templates) in categories.iter() {
        html.push_str(&format!(
            r#"        <div class="palette-section">
            <div class="palette-section-title">{}</div>
            <div class="palette-grid">
"#,
            category
        ));

        for template in templates {
            html.push_str(&format!(
                r#"                <button class="palette-icon-btn" 
                        data-tooltip="{}"
                        onclick="navigator.clipboard.writeText('{}')">
                    <img src="{}.svg" alt="{}">
                </button>
"#,
                template.name, template.typst_markup, template.name, template.name
            ));
        }

        html.push_str("            </div>\n        </div>\n\n");
    }

    html.push_str(
        r#"    </div>
    <script>
        document.querySelectorAll('.palette-icon-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                btn.style.background = '#4CAF50';
                btn.style.color = 'white';
                setTimeout(() => {
                    btn.style.background = '';
                    btn.style.color = '';
                }, 200);
            });
        });
    </script>
</body>
</html>
"#,
    );

    let html_path = output_dir.join("index.html");
    fs::write(html_path, html).expect("Failed to write HTML");
    println!("📝 Generated HTML example");
}
