#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use kleis::ast::Expression;
use kleis::math_layout::compile_math_to_svg_with_ids;
use kleis::render::{build_default_context, render_expression, RenderTarget};
use std::fs::File;
use std::io::Write;

fn main() {
    println!("=== Testing Complex Layout Grouping ===\n");

    let cases = vec![
        ("Nested Fraction in Numerator", nested_fraction_num()),
        ("Nested Fraction in Denominator", nested_fraction_den()),
        ("Tall Square Root", tall_sqrt()),
        ("Mixed Sub/Superscript", mixed_scripts()),
    ];

    let mut html_output = String::from(
        r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        body { font-family: sans-serif; padding: 20px; background: #f0f0f0; }
        .case { background: white; padding: 20px; margin-bottom: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        h2 { margin-top: 0; color: #333; }
        .preview { border: 1px solid #ddd; padding: 20px; display: inline-block; background: white; }
    </style>
</head>
<body>
    <h1>Layout Debug Report</h1>
"#,
    );

    for (name, expr) in cases {
        println!("\n--- Test Case: {} ---", name);

        // 1. Render to Typst
        let ctx = build_default_context();
        let typst_markup = render_expression(&expr, &ctx, &RenderTarget::Typst);

        // 2. Compile and extract layout
        match compile_math_to_svg_with_ids(&typst_markup, &[], &[]) {
            Ok(output) => {
                let mut svg = output.svg.clone();

                // Inject red rectangles for bounding boxes into the SVG
                // We use the exact same logic as the frontend: absolute coordinates from backend
                // The SVG root has a viewbox, but no transform. The content inside might.
                // But our backend normalization aligns layout (0,0) to SVG (0,0).

                let mut overlays = String::new();
                for (i, bbox) in output.argument_bounding_boxes.iter().enumerate() {
                    println!(
                        "  Group {}: x={:.2}, y={:.2}, w={:.2}, h={:.2}",
                        i, bbox.x, bbox.y, bbox.width, bbox.height
                    );

                    // Green dashed box
                    overlays.push_str(&format!(
                        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="rgba(40, 167, 69, 0.2)" stroke="green" stroke-width="2" stroke-dasharray="4,2"/>"#,
                        bbox.x - 3.0, bbox.y - 3.0, bbox.width + 6.0, bbox.height + 6.0
                    ));

                    // Label
                    overlays.push_str(&format!(
                        r#"<text x="{}" y="{}" font-size="10" fill="red">Group {}</text>"#,
                        bbox.x,
                        bbox.y - 5.0,
                        i
                    ));
                }

                // Insert overlays before closing svg tag
                svg = svg.replace("</svg>", &format!("<g>{}</g></svg>", overlays));

                html_output.push_str(&format!(
                    r#"
                    <div class="case">
                        <h2>{}</h2>
                        <div class="preview">
                            {}
                        </div>
                        <pre>{}</pre>
                    </div>
                    "#,
                    name, svg, typst_markup
                ));
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    html_output.push_str("</body></html>");

    let mut file = File::create("layout_debug.html").expect("Unable to create file");
    file.write_all(html_output.as_bytes())
        .expect("Unable to write data");
    println!("\nReport written to layout_debug.html");
}

// (1 + 1/(x+y)) / 2
fn nested_fraction_num() -> Expression {
    Expression::Operation {
        name: "scalar_divide".to_string(),
        args: vec![
            Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Const("1".to_string()),
                    Expression::Operation {
                        name: "scalar_divide".to_string(),
                        args: vec![
                            Expression::Const("1".to_string()),
                            Expression::Operation {
                                name: "plus".to_string(),
                                args: vec![
                                    Expression::Object("x".to_string()),
                                    Expression::Object("y".to_string()),
                                ],
                                span: None,
                            },
                        ],
                        span: None,
                    },
                ],
                span: None,
            },
            Expression::Const("2".to_string()),
        ],
        span: None,
    }
}

// 2 / (1 + 1/(x+y))
fn nested_fraction_den() -> Expression {
    Expression::Operation {
        name: "scalar_divide".to_string(),
        args: vec![
            Expression::Const("2".to_string()),
            Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Const("1".to_string()),
                    Expression::Operation {
                        name: "scalar_divide".to_string(),
                        args: vec![
                            Expression::Const("1".to_string()),
                            Expression::Operation {
                                name: "plus".to_string(),
                                args: vec![
                                    Expression::Object("x".to_string()),
                                    Expression::Object("y".to_string()),
                                ],
                                span: None,
                            },
                        ],
                        span: None,
                    },
                ],
                span: None,
            },
        ],
        span: None,
    }
}

// sqrt( (a^2 + b^2) / c^2 )
fn tall_sqrt() -> Expression {
    Expression::Operation {
        name: "sqrt".to_string(),
        args: vec![Expression::Operation {
            name: "scalar_divide".to_string(),
            args: vec![
                Expression::Operation {
                    name: "plus".to_string(),
                    args: vec![
                        Expression::Operation {
                            name: "sup".to_string(),
                            args: vec![
                                Expression::Object("a".to_string()),
                                Expression::Const("2".to_string()),
                            ],
                            span: None,
                        },
                        Expression::Operation {
                            name: "sup".to_string(),
                            args: vec![
                                Expression::Object("b".to_string()),
                                Expression::Const("2".to_string()),
                            ],
                            span: None,
                        },
                    ],
                    span: None,
                },
                Expression::Operation {
                    name: "sup".to_string(),
                    args: vec![
                        Expression::Object("c".to_string()),
                        Expression::Const("2".to_string()),
                    ],
                    span: None,
                },
            ],
            span: None,
        }],
        span: None,
    }
}

// x_i + y^n + z_(k+1)
fn mixed_scripts() -> Expression {
    Expression::Operation {
        name: "plus".to_string(),
        args: vec![
            Expression::Operation {
                name: "sub".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Object("i".to_string()),
                ],
                span: None,
            },
            Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Operation {
                        name: "sup".to_string(),
                        args: vec![
                            Expression::Object("y".to_string()),
                            Expression::Object("n".to_string()),
                        ],
                        span: None,
                    },
                    Expression::Operation {
                        name: "sub".to_string(),
                        args: vec![
                            Expression::Object("z".to_string()),
                            Expression::Operation {
                                name: "plus".to_string(),
                                args: vec![
                                    Expression::Object("k".to_string()),
                                    Expression::Const("1".to_string()),
                                ],
                                span: None,
                            },
                        ],
                        span: None,
                    },
                ],
                span: None,
            },
        ],
        span: None,
    }
}
