use std::collections::HashMap;

// === Symbolic Model ===
#[derive(Debug)]
enum Expression {
    Const(String),
    Object(String),
    Operation { name: String, args: Vec<Expression> },
}
enum RenderTarget {
    Unicode,
    LaTeX,
}


// === Glyph + Template Context ===
#[derive(Debug)]
struct GlyphContext {
    unicode_glyphs: HashMap<String, String>,
    unicode_templates: HashMap<String, String>,
    latex_glyphs: HashMap<String, String>,
    latex_templates: HashMap<String, String>,
}


// === Renderer ===
fn render_expression(expr: &Expression, ctx: &GlyphContext, target: &RenderTarget) -> String {
    match expr {
        Expression::Const(name) => {
            match target {
                RenderTarget::Unicode => name.clone(),
                RenderTarget::LaTeX => escape_latex_constant(name),
            }
        }
        Expression::Object(name) => name.clone(),
        Expression::Operation { name, args } => {
            let (template, glyph) = match target {
                RenderTarget::Unicode => {
                    let template = ctx.unicode_templates.get(name)
                        .cloned()
                        .unwrap_or_else(|| format!("{}({})", name, "{args}"));
                    let glyph = ctx.unicode_glyphs.get(name)
                        .unwrap_or(name);
                    (template, glyph)
                },
                RenderTarget::LaTeX => {
                    let template = ctx.latex_templates.get(name)
                        .cloned()
                        .unwrap_or_else(|| format!("{}({})", name, "{args}"));
                    let glyph = ctx.latex_glyphs.get(name)
                        .unwrap_or(name);
                    (template, glyph)
                },
            };

            let rendered_args: Vec<String> = args.iter()
                .map(|arg| render_expression(arg, ctx, target)) // RECURSION
                .collect();

            let mut result = template.clone();
            // Simple placeholder substitution
            result = result.replace("{glyph}", glyph);
            if let Some(first) = rendered_args.get(0) {
                result = result.replace("{arg}", first);
                result = result.replace("{left}", first);
                result = result.replace("{field}", first);
            }
            if let Some(second) = rendered_args.get(1) {
                result = result.replace("{right}", second);
                result = result.replace("{surface}", second);
            }
            result
        }
    }
}

fn escape_latex_constant(constant: &str) -> String {
    constant.replace("π", "\\pi")
}

fn main() {
    // === Set up GlyphContext ===
    let mut unicode_glyphs = HashMap::new();
    unicode_glyphs.insert("grad".to_string(), "∇".to_string());
    unicode_glyphs.insert("surface_integral_over".to_string(), "∮".to_string());
    unicode_glyphs.insert("scalar_multiply".to_string(), "×".to_string());
    unicode_glyphs.insert("scalar_divide".to_string(), "/".to_string());

    let mut unicode_templates = HashMap::new();
    unicode_templates.insert("grad".to_string(), "{glyph}{arg}".to_string());
    unicode_templates.insert("surface_integral_over".to_string(), "{glyph}_{surface} {field} dS".to_string());
    unicode_templates.insert("scalar_multiply".to_string(),  "{left} \\, {right}".to_string()); // thin space
    unicode_templates.insert("scalar_divide".to_string(), "({left}) / ({right})".to_string());



    let mut latex_glyphs = HashMap::new();
    latex_glyphs.insert("grad".to_string(), "\\nabla".to_string());
    latex_glyphs.insert("surface_integral_over".to_string(), "\\oint".to_string());
    //latex_glyphs.insert("scalar_multiply".to_string(), "\\times".to_string());
    // scalar_divide does not need glyph separately (uses \frac)

    let mut latex_templates = HashMap::new();
    latex_templates.insert("grad".to_string(), "{glyph} {arg}".to_string());
    latex_templates.insert("surface_integral_over".to_string(), "{glyph}_{ {surface} } {field} \\, dS".to_string());
    latex_templates.insert("scalar_multiply".to_string(), "{left} \\, {right}".to_string());
    latex_templates.insert("scalar_divide".to_string(), "\\frac{{left}}{{right}}".to_string());



    let ctx = GlyphContext {
        unicode_glyphs,
        unicode_templates,
        latex_glyphs,
        latex_templates,
    };

    // === Build Expression Tree ===
    let phi = Expression::Object("Φ".to_string());
    let grad_phi = Expression::Operation {
        name: "grad".to_string(),
        args: vec![phi]
    };

    let surface = Expression::Object("S".to_string());
    let surface_integral = Expression::Operation {
        name: "surface_integral_over".to_string(),
        args: vec![grad_phi, surface],
    };

    // Represent -1 / (4π) symbolically
    let minus_one = Expression::Const("-1".to_string());
    let four = Expression::Const("4".to_string());
    let pi = Expression::Const("π".to_string());

    let four_pi = Expression::Operation {
        name: "scalar_multiply".to_string(),
        args: vec![four, pi],
    };

    let negative_one_over_four_pi = Expression::Operation {
        name: "scalar_divide".to_string(),
        args: vec![minus_one, four_pi],
    };

    // Multiply (-1/4π) × (surface integral)
    let residue = Expression::Operation {
        name: "scalar_multiply".to_string(),
        args: vec![negative_one_over_four_pi, surface_integral],
    };

    let g_c = Expression::Const("G_c".to_string());

    let mass = Expression::Operation {
        name: "scalar_divide".to_string(),
        args: vec![residue, g_c],
    };


    // === Render Unicode===
    let output = render_expression(&mass, &ctx, &RenderTarget::Unicode);
    println!("Rendered Expression: {}", output);

    // === Render Latex ===
    let output = render_expression(&mass, &ctx, &RenderTarget::LaTeX);
    println!("Rendered Expression: {}", output);

}
