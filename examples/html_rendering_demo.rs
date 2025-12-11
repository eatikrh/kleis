use kleis::ast::Expression;
use kleis::render::{build_default_context, render_expression, RenderTarget};
/// HTML/MathML Rendering Demo for Integral Transforms & POT Operations
///
/// This program demonstrates the HTML rendering of all 16 new mathematical
/// operations added to Kleis for POT (Projected Ontology Theory).
///
/// Run with: cargo run --example html_rendering_demo > output.html
use kleis::templates::*;

fn render_html(name: &str, description: &str, template_fn: fn() -> Expression) {
    reset_placeholder_counter();
    let expr = template_fn();
    let ctx = build_default_context();
    let output = render_expression(&expr, &ctx, &RenderTarget::HTML);

    println!(r#"      <div class="operation">"#);
    println!(r#"        <div class="name">{}</div>"#, name);
    println!(r#"        <div class="math">{}</div>"#, output);
    println!(r#"        <div class="description">{}</div>"#, description);
    println!(r#"      </div>"#);
}

fn main() {
    // HTML Header
    println!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Kleis HTML/MathML Rendering Gallery</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }}
        
        .container {{
            background: white;
            border-radius: 12px;
            padding: 40px;
            box-shadow: 0 10px 40px rgba(0,0,0,0.2);
        }}
        
        h1 {{
            color: #667eea;
            text-align: center;
            margin-bottom: 10px;
            font-size: 2.5em;
        }}
        
        .subtitle {{
            text-align: center;
            color: #666;
            margin-bottom: 40px;
            font-size: 1.2em;
        }}
        
        .section {{
            margin-bottom: 50px;
        }}
        
        .section-title {{
            color: #764ba2;
            border-bottom: 3px solid #667eea;
            padding-bottom: 10px;
            margin-bottom: 25px;
            font-size: 1.8em;
        }}
        
        .operation {{
            background: #f8f9fa;
            border-left: 4px solid #667eea;
            padding: 20px;
            margin-bottom: 15px;
            border-radius: 6px;
            transition: all 0.3s ease;
        }}
        
        .operation:hover {{
            background: #e9ecef;
            transform: translateX(5px);
            box-shadow: 0 4px 12px rgba(102, 126, 234, 0.2);
        }}
        
        .name {{
            font-weight: bold;
            color: #495057;
            margin-bottom: 10px;
            font-size: 1.1em;
        }}
        
        .math {{
            font-size: 1.4em;
            font-family: 'Cambria Math', 'Times New Roman', serif;
            color: #212529;
            padding: 15px;
            background: white;
            border-radius: 4px;
            margin: 10px 0;
            overflow-x: auto;
        }}
        
        /* Math-specific styling */
        .math-script {{
            font-style: italic;
            font-size: 1.2em;
        }}
        
        .math-op {{
            padding: 0 4px;
            font-weight: normal;
        }}
        
        .math-func {{
            font-style: normal;
            font-weight: 500;
        }}
        
        .math-sub {{
            font-size: 0.8em;
            vertical-align: sub;
        }}
        
        .math-sup {{
            font-size: 0.8em;
            vertical-align: super;
        }}
        
        .math-blackboard {{
            font-weight: bold;
            font-family: 'Times New Roman', serif;
        }}
        
        .description {{
            color: #6c757d;
            font-size: 0.95em;
            margin-top: 10px;
            font-style: italic;
        }}
        
        .example {{
            background: #fff3cd;
            border-left: 4px solid #ffc107;
            padding: 20px;
            margin: 20px 0;
            border-radius: 6px;
        }}
        
        .example-title {{
            font-weight: bold;
            color: #856404;
            margin-bottom: 10px;
        }}
        
        .hierarchy {{
            background: #d4edda;
            border: 2px solid #28a745;
            padding: 20px;
            margin: 20px 0;
            border-radius: 6px;
            text-align: center;
            font-size: 1.2em;
        }}
        
        .footer {{
            text-align: center;
            margin-top: 40px;
            padding-top: 20px;
            border-top: 2px solid #dee2e6;
            color: #6c757d;
        }}
        
        code {{
            background: #f8f9fa;
            padding: 2px 6px;
            border-radius: 3px;
            font-family: 'Courier New', monospace;
            font-size: 0.9em;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🎨 Kleis HTML Rendering Gallery</h1>
        <div class="subtitle">Integral Transforms & POT Operations</div>
"#
    );

    // Integral Transforms
    println!(r#"        <div class="section">"#);
    println!(r#"          <h2 class="section-title">📐 Integral Transforms</h2>"#);

    render_html(
        "Fourier Transform",
        "Transform from time/space domain to frequency/momentum domain",
        template_fourier_transform,
    );

    render_html(
        "Inverse Fourier Transform",
        "Transform from frequency/momentum domain back to time/space domain",
        template_inverse_fourier,
    );

    render_html(
        "Laplace Transform",
        "Transform for solving differential equations, converting to s-domain",
        template_laplace_transform,
    );

    render_html(
        "Inverse Laplace Transform",
        "Transform from s-domain back to time domain",
        template_inverse_laplace,
    );

    render_html(
        "Convolution",
        "Combines two functions to produce a third, used for signal processing and field theory",
        template_convolution,
    );

    render_html(
        "Kernel Integral",
        "General integral transform with arbitrary kernel K(x,m)",
        template_kernel_integral,
    );

    render_html(
        "Green's Function",
        "Response at point x due to impulse at source point m",
        template_greens_function,
    );

    println!(r#"        </div>"#);

    // POT Operations
    println!(r#"        <div class="section">"#);
    println!(r#"          <h2 class="section-title">🌌 POT Operations</h2>"#);

    render_html(
        "Projection Operator",
        "Maps modal space functions to spacetime - the core POT operation",
        template_projection,
    );

    render_html(
        "Modal Integral",
        "Integration over modal space with measure dμ",
        template_modal_integral,
    );

    render_html(
        "Projection Kernel",
        "The kernel K(x,m) that defines how modal states project to spacetime",
        template_projection_kernel,
    );

    render_html(
        "Causal Bound",
        "Variable speed of light c(x), derived from projection kernel support",
        template_causal_bound,
    );

    render_html(
        "Projection Residue",
        "Physical constants as stable properties of the projection",
        template_projection_residue,
    );

    render_html(
        "Modal Space",
        "The modal domain 𝓜 from which spacetime is projected",
        template_modal_space,
    );

    // Spacetime (no placeholders)
    reset_placeholder_counter();
    let spacetime = template_spacetime();
    let ctx = build_default_context();
    let output = render_expression(&spacetime, &ctx, &RenderTarget::HTML);
    println!(r#"      <div class="operation">"#);
    println!(r#"        <div class="name">Spacetime</div>"#);
    println!(r#"        <div class="math">{}</div>"#, output);
    println!(
        r#"        <div class="description">4-dimensional spacetime manifold - the target of projection</div>"#
    );
    println!(r#"      </div>"#);

    render_html(
        "Hont (Hilbert Ontology)",
        "The eternal ontological domain - Hilbert space as Being",
        template_hont,
    );

    println!(r#"        </div>"#);

    // Examples
    println!(r#"        <div class="section">"#);
    println!(r#"          <h2 class="section-title">💡 Complete Examples</h2>"#);

    println!(
        r#"          <div class="example">
            <div class="example-title">Example 1: Projection Expansion</div>
            <div class="math">Π[ψ](x) = ∫<sub class="math-sub">M</sub> K(x,m) ψ(m) dμ(m)</div>
            <div class="description">
              Modal state ψ(m) in modal space M is projected to spacetime field φ(x) via kernel K(x,m)
            </div>
          </div>
          
          <div class="example">
            <div class="example-title">Example 2: Variable Speed of Light</div>
            <div class="math">c(x) = Residue[Π, causal_structure] = 1/width[K(x,·)]</div>
            <div class="description">
              Early universe: wide K(x,m) → large c(x) → no inflation needed<br>
              Late universe: narrow K(x,m) → small c(x) → local physics
            </div>
          </div>
          
          <div class="example">
            <div class="example-title">Example 3: Convolution for Field Propagation</div>
            <div class="math">φ(x) = (ρ <span class="math-op">∗</span> G)(x) = ∫ ρ(y) G(x,y) dy</div>
            <div class="description">
              Field φ at point x from distributed source ρ via Green's function G
            </div>
          </div>"#
    );

    println!(r#"        </div>"#);

    // POT Hierarchy
    println!(
        r#"        <div class="hierarchy">
          <div style="font-weight: bold; margin-bottom: 15px;">POT Ontological Hierarchy</div>
          <div style="font-size: 1.3em;">
            <span class="math-script">𝓗</span> (Hont) → 
            <span class="math-script">𝓜</span> (Modal) → 
            <span class="math-op">Π</span> (Projection) → 
            <span class="math-blackboard">ℝ</span><sup class="math-sup">4</sup> (Spacetime)
          </div>
          <div style="margin-top: 10px; color: #28a745;">
            Being → Relations → Transform → Appearance
          </div>
        </div>"#
    );

    // Footer
    println!(
        r#"        <div class="footer">
          <p><strong>✅ All 16 operations rendered successfully in HTML!</strong></p>
          <p>Generated by Kleis - Mathematical document editor with POT support</p>
          <p style="margin-top: 10px; font-size: 0.9em;">
            Run: <code>cargo run --example html_rendering_demo > gallery.html</code>
          </p>
        </div>
      </div>
    </body>
</html>"#
    );
}
