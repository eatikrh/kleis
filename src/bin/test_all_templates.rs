#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use kleis::math_layout::compile_math_to_svg_with_ids;
use kleis::render::{build_default_context, render_expression, RenderTarget};
use kleis::templates::get_all_templates;
use std::fs::File;
use std::io::Write;

fn main() {
    println!("=== Testing All Templates Coverage ===\n");

    let templates = get_all_templates();
    println!("Found {} templates to test.\n", templates.len());

    let mut html_output = String::from(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Typst Template Coverage Report</title>
    <style>
        body { font-family: sans-serif; padding: 20px; background: #f0f0f0; }
        .case { background: white; padding: 20px; margin-bottom: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        h2 { margin-top: 0; color: #333; font-size: 18px; }
        .success { border-left: 5px solid #28a745; }
        .failure { border-left: 5px solid #dc3545; }
        .preview { border: 1px solid #ddd; padding: 20px; display: inline-block; background: white; min-width: 200px; min-height: 100px; display: flex; align-items: center; justify-content: center;}
        pre { background: #f8f9fa; padding: 10px; border-radius: 4px; overflow-x: auto; }
        .status { float: right; font-weight: bold; }
        .status.ok { color: #28a745; }
        .status.err { color: #dc3545; }
    </style>
</head>
<body>
    <h1>Typst Template Coverage Report</h1>
"#,
    );

    let mut success_count = 0;
    let mut failure_count = 0;

    for (name, template_fn) in templates {
        print!("Testing template: {:<20} ... ", name);

        let expr = template_fn();

        // 1. Render to Typst
        let ctx = build_default_context();
        let typst_markup = render_expression(&expr, &ctx, &RenderTarget::Typst);

        // 2. Compile and extract layout
        match compile_math_to_svg_with_ids(&typst_markup, &[], &[]) {
            Ok(output) => {
                println!("OK");
                success_count += 1;

                // Inject bounding boxes for visualization
                let mut svg = output.svg.clone();
                let mut overlays = String::new();
                for (i, bbox) in output.argument_bounding_boxes.iter().enumerate() {
                    overlays.push_str(&format!(
                        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="rgba(40, 167, 69, 0.2)" stroke="green" stroke-width="1" stroke-dasharray="2,2"/>"#,
                        bbox.x, bbox.y, bbox.width, bbox.height
                    ));
                }
                svg = svg.replace("</svg>", &format!("<g>{}</g></svg>", overlays));

                html_output.push_str(&format!(
                    r#"
                    <div class="case success">
                        <span class="status ok">PASS</span>
                        <h2>{}</h2>
                        <div class="preview">
                            {}
                        </div>
                        <pre>Typst: {}</pre>
                        <p>Boxes found: {}</p>
                    </div>
                    "#,
                    name,
                    svg,
                    typst_markup,
                    output.argument_bounding_boxes.len()
                ));
            }
            Err(e) => {
                println!("FAILED: {}", e);
                failure_count += 1;

                html_output.push_str(&format!(
                    r#"
                    <div class="case failure">
                        <span class="status err">FAIL</span>
                        <h2>{}</h2>
                        <pre>Typst: {}</pre>
                        <div style="color: #dc3545; padding: 10px; background: #ffe6e6; border-radius: 4px;">
                            Error: {}
                        </div>
                    </div>
                    "#,
                    name, typst_markup, e
                ));
            }
        }
    }

    html_output.push_str(&format!(
        r#"
        <div class="summary" style="margin-top: 30px; padding: 20px; background: #fff; border-radius: 8px;">
            <h2>Summary</h2>
            <p>Total Templates: <strong>{}</strong></p>
            <p style="color: #28a745">Passed: <strong>{}</strong></p>
            <p style="color: #dc3545">Failed: <strong>{}</strong></p>
        </div>
        </body></html>
        "#, 
        success_count + failure_count, success_count, failure_count
    ));

    let mut file = File::create("template_coverage_report.html").expect("Unable to create file");
    file.write_all(html_output.as_bytes())
        .expect("Unable to write data");
    println!("\nReport written to template_coverage_report.html");
}
