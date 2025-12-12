#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use kleis::math_layout::compile_math_to_svg_with_ids;
use kleis::parser::parse_latex;
use kleis::render::{
    build_default_context, collect_samples_for_gallery, render_expression, RenderTarget,
};
use kleis::templates::get_all_templates;
use std::fs::File;
use std::io::Write;

fn main() {
    println!("=== Generating Typst vs MathJax Comparison Report ===\n");

    let mut html_output = String::from(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Typst vs MathJax Comparison</title>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/mathjax/3.2.2/es5/tex-mml-chtml.min.js"></script>
    <style>
        body { font-family: sans-serif; padding: 20px; background: #f5f5f5; }
        h1 { text-align: center; color: #333; }
        table { width: 100%; border-collapse: collapse; background: white; box-shadow: 0 2px 8px rgba(0,0,0,0.1); }
        th, td { border: 1px solid #ddd; padding: 15px; text-align: center; vertical-align: middle; }
        th { background: #667eea; color: white; position: sticky; top: 0; }
        .typst-col { width: 40%; background: #fafafa; }
        .mathjax-col { width: 40%; background: #fff; }
        .name-col { width: 20%; font-weight: bold; color: #555; text-align: left; }
        .error { color: #dc3545; background: #ffe6e6; padding: 10px; border-radius: 4px; font-family: monospace; font-size: 0.8em; text-align: left; }
        .section-header { background: #eee; text-align: left; font-size: 1.2em; padding: 10px 20px; font-weight: bold; }
    </style>
</head>
<body>
    <h1>Typst vs MathJax Comparison</h1>
    <table>
        <thead>
            <tr>
                <th>Name / Input</th>
                <th>Typst SVG (New)</th>
                <th>MathJax (Reference)</th>
            </tr>
        </thead>
        <tbody>
"#,
    );

    let ctx = build_default_context();

    // 1. Test Templates
    html_output
        .push_str(r#"<tr><td colspan="3" class="section-header">Templates (Palette)</td></tr>"#);

    let templates = get_all_templates();
    for (name, template_fn) in templates {
        println!("Processing template: {}", name);
        let expr = template_fn();

        // Render to Typst
        let typst_markup = render_expression(&expr, &ctx, &RenderTarget::Typst);
        // Render to LaTeX (for MathJax)
        let latex_markup = render_expression(&expr, &ctx, &RenderTarget::LaTeX);

        let typst_cell = match compile_math_to_svg_with_ids(&typst_markup, &[], &[]) {
            Ok(output) => output.svg,
            Err(e) => format!(
                r#"<div class="error">Typst Error: {}<br>Markup: {}</div>"#,
                e, typst_markup
            ),
        };

        html_output.push_str(&format!(
            r#"
            <tr>
                <td class="name-col">
                    {}<br>
                    <span style="font-weight:normal; font-size:0.8em; color:#999">{}</span>
                </td>
                <td class="typst-col">{}</td>
                <td class="mathjax-col">\[{}\]</td>
            </tr>
            "#,
            name, latex_markup, typst_cell, latex_markup
        ));
    }

    // 2. Test Gallery Examples
    html_output
        .push_str(r#"<tr><td colspan="3" class="section-header">Gallery Examples</td></tr>"#);

    let gallery = collect_samples_for_gallery();
    for (title, latex) in gallery {
        println!("Processing gallery: {}", title);

        let typst_cell = match parse_latex(&latex) {
            Ok(expr) => {
                let typst_markup = render_expression(&expr, &ctx, &RenderTarget::Typst);
                match compile_math_to_svg_with_ids(&typst_markup, &[], &[]) {
                    Ok(output) => output.svg,
                    Err(e) => format!(
                        r#"<div class="error">Typst Compilation Error: {}<br>Markup: {}</div>"#,
                        e, typst_markup
                    ),
                }
            }
            Err(e) => format!(r#"<div class="error">Parse Error: {:?}</div>"#, e),
        };

        html_output.push_str(&format!(
            r#"
            <tr>
                <td class="name-col">
                    {}<br>
                    <span style="font-weight:normal; font-size:0.8em; color:#999; font-family:monospace">{}</span>
                </td>
                <td class="typst-col">{}</td>
                <td class="mathjax-col">\[{}\]</td>
            </tr>
            "#,
            title, latex, typst_cell, latex
        ));
    }

    html_output.push_str(
        r#"
        </tbody>
    </table>
</body>
</html>
"#,
    );

    let mut file = File::create("comparison_report.html").expect("Unable to create file");
    file.write_all(html_output.as_bytes())
        .expect("Unable to write data");
    println!("\nDone! Report written to comparison_report.html");
}
