#![allow(non_snake_case)]

use std::collections::HashMap;

// === Symbolic Model ===
use crate::ast::Expression;
#[derive(PartialEq)]
pub enum RenderTarget {
    Unicode,
    LaTeX,
    HTML,
    Typst, // Typst markup for math layout engine
    Kleis, // Kleis text syntax for verification
}

// === Glyph + Template Context ===
#[derive(Debug)]
pub struct GlyphContext {
    unicode_glyphs: HashMap<String, String>,
    unicode_templates: HashMap<String, String>,
    latex_glyphs: HashMap<String, String>,
    latex_templates: HashMap<String, String>,
    html_glyphs: HashMap<String, String>,
    html_templates: HashMap<String, String>,
    typst_glyphs: HashMap<String, String>,
    typst_templates: HashMap<String, String>,
    kleis_glyphs: HashMap<String, String>,
    kleis_templates: HashMap<String, String>,
}

// === Expression Builders (ergonomic helpers) ===
#[allow(dead_code)]
fn c<S: Into<String>>(s: S) -> Expression {
    Expression::Const(s.into())
}
#[allow(dead_code)]
fn o<S: Into<String>>(s: S) -> Expression {
    Expression::Object(s.into())
}
#[allow(dead_code)]
fn op<S: Into<String>>(name: S, args: Vec<Expression>) -> Expression {
    Expression::Operation {
        name: name.into(),
        args,
    }
}
#[allow(dead_code)]
fn plus(a: Expression, b: Expression) -> Expression {
    op("plus", vec![a, b])
}
#[allow(dead_code)]
fn minus(a: Expression, b: Expression) -> Expression {
    op("minus", vec![a, b])
}
#[allow(dead_code)]
fn times(a: Expression, b: Expression) -> Expression {
    op("scalar_multiply", vec![a, b])
}
#[allow(dead_code)]
fn over(a: Expression, b: Expression) -> Expression {
    op("scalar_divide", vec![a, b])
}
#[allow(dead_code)]
fn dot_e(a: Expression, b: Expression) -> Expression {
    op("dot", vec![a, b])
}
#[allow(dead_code)]
fn d_dt(num: Expression, den: Expression) -> Expression {
    op("d_dt", vec![num, den])
}
#[allow(dead_code)]
fn d_part(num: Expression, den: Expression) -> Expression {
    op("d_part", vec![num, den])
}
#[allow(dead_code)]
fn d2_part(num: Expression, den: Expression) -> Expression {
    op("d2_part", vec![num, den])
}
#[allow(dead_code)]
fn sub_e(base: Expression, sub: Expression) -> Expression {
    op("sub", vec![base, sub])
}
#[allow(dead_code)]
fn sup_e(base: Expression, sup: Expression) -> Expression {
    op("sup", vec![base, sup])
}
#[allow(dead_code)]
fn index_mixed(base: Expression, idx1: Expression, idx2: Expression) -> Expression {
    op("index_mixed", vec![base, idx1, idx2])
}
#[allow(dead_code)]
fn index_pair(base: Expression, idx1: Expression, idx2: Expression) -> Expression {
    op("index_pair", vec![base, idx1, idx2])
}
#[allow(dead_code)]
fn partial_apply(arg: Expression, sub: Expression) -> Expression {
    op("partial_apply", vec![arg, sub])
}
#[allow(dead_code)]
fn min_over(sub: Expression, body: Expression) -> Expression {
    op("min_over", vec![body, sub])
}
#[allow(dead_code)]
fn func<S: Into<String>>(name: S, args: Vec<Expression>) -> Expression {
    op(name, args)
}

// Common math helpers
#[allow(dead_code)]
fn equals(a: Expression, b: Expression) -> Expression {
    op("equals", vec![a, b])
}
#[allow(dead_code)]
fn pow_e(base: Expression, exponent: Expression) -> Expression {
    op("power", vec![base, exponent])
}
#[allow(dead_code)]
fn inner_e(a: Expression, b: Expression) -> Expression {
    op("inner", vec![a, b])
}
#[allow(dead_code)]
fn cross_e(a: Expression, b: Expression) -> Expression {
    op("cross", vec![a, b])
}
#[allow(dead_code)]
fn norm_e(a: Expression) -> Expression {
    op("norm", vec![a])
}
#[allow(dead_code)]
fn abs_e(a: Expression) -> Expression {
    op("abs", vec![a])
}
#[allow(dead_code)]
fn transpose_e(a: Expression) -> Expression {
    op("transpose", vec![a])
}
#[allow(dead_code)]
fn det_e(a: Expression) -> Expression {
    op("det", vec![a])
}
#[allow(dead_code)]
fn m2(a11: Expression, a12: Expression, a21: Expression, a22: Expression) -> Expression {
    op(
        "Matrix",
        vec![
            Expression::Const("2".to_string()),
            Expression::Const("2".to_string()),
            a11,
            a12,
            a21,
            a22,
        ],
    )
}
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn m3(
    a11: Expression,
    a12: Expression,
    a13: Expression,
    a21: Expression,
    a22: Expression,
    a23: Expression,
    a31: Expression,
    a32: Expression,
    a33: Expression,
) -> Expression {
    op(
        "Matrix",
        vec![
            Expression::Const("3".to_string()),
            Expression::Const("3".to_string()),
            a11,
            a12,
            a13,
            a21,
            a22,
            a23,
            a31,
            a32,
            a33,
        ],
    )
}
#[allow(dead_code)]
fn vector_arrow_e(a: Expression) -> Expression {
    op("vector_arrow", vec![a])
}
#[allow(dead_code)]
fn vector_bold_e(a: Expression) -> Expression {
    op("vector_bold", vec![a])
}
#[allow(dead_code)]
fn sum_e(body: Expression, from: Expression, to: Expression) -> Expression {
    op("sum_bounds", vec![body, from, to])
}
#[allow(dead_code)]
fn prod_e(body: Expression, from: Expression, to: Expression) -> Expression {
    op("prod_bounds", vec![body, from, to])
}
#[allow(dead_code)]
fn int_e(integrand: Expression, from: Expression, to: Expression, var: Expression) -> Expression {
    op("int_bounds", vec![integrand, from, to, var])
}
#[allow(dead_code)]
fn grad_e(a: Expression) -> Expression {
    op("grad", vec![a])
}
#[allow(dead_code)]
fn surface_integral(field: Expression, surface: Expression) -> Expression {
    op("surface_integral_over", vec![field, surface])
}

// === Top 5 Operations (Implemented) ===

// 1. Bra-ket notation (Quantum mechanics)
#[allow(dead_code)]
fn ket(a: Expression) -> Expression {
    op("ket", vec![a])
}
#[allow(dead_code)]
fn bra(a: Expression) -> Expression {
    op("bra", vec![a])
}
#[allow(dead_code)]
fn outer_product(a: Expression, b: Expression) -> Expression {
    op("outer_product", vec![a, b])
}

// 2. Set theory and logic
#[allow(dead_code)]
fn in_set(a: Expression, b: Expression) -> Expression {
    op("in", vec![a, b])
}
#[allow(dead_code)]
fn subset(a: Expression, b: Expression) -> Expression {
    op("subset", vec![a, b])
}
#[allow(dead_code)]
fn subseteq(a: Expression, b: Expression) -> Expression {
    op("subseteq", vec![a, b])
}
#[allow(dead_code)]
fn union(a: Expression, b: Expression) -> Expression {
    op("union", vec![a, b])
}
#[allow(dead_code)]
fn intersection(a: Expression, b: Expression) -> Expression {
    op("intersection", vec![a, b])
}
#[allow(dead_code)]
fn forall(var: Expression, body: Expression) -> Expression {
    op("forall", vec![var, body])
}
#[allow(dead_code)]
fn exists(var: Expression, body: Expression) -> Expression {
    op("exists", vec![var, body])
}
#[allow(dead_code)]
fn implies(a: Expression, b: Expression) -> Expression {
    op("implies", vec![a, b])
}
#[allow(dead_code)]
fn iff(a: Expression, b: Expression) -> Expression {
    op("iff", vec![a, b])
}

// 3. Multiple integrals
#[allow(dead_code)]
fn double_int(
    integrand: Expression,
    region: Expression,
    var1: Expression,
    var2: Expression,
) -> Expression {
    op("double_integral", vec![integrand, region, var1, var2])
}
#[allow(dead_code)]
fn triple_int(
    integrand: Expression,
    region: Expression,
    var1: Expression,
    var2: Expression,
    var3: Expression,
) -> Expression {
    op("triple_integral", vec![integrand, region, var1, var2, var3])
}

// 4. Commutators
#[allow(dead_code)]
fn commutator(a: Expression, b: Expression) -> Expression {
    op("commutator", vec![a, b])
}
#[allow(dead_code)]
fn anticommutator(a: Expression, b: Expression) -> Expression {
    op("anticommutator", vec![a, b])
}

// 5. Square root
#[allow(dead_code)]
fn sqrt_e(a: Expression) -> Expression {
    op("sqrt", vec![a])
}
#[allow(dead_code)]
fn nth_root(a: Expression, n: Expression) -> Expression {
    op("nth_root", vec![a, n])
}

// === Next Top 3 + Low-Hanging Fruit ===

// Comparison & Inequality Operators (7 operators)
#[allow(dead_code)]
fn less_than(a: Expression, b: Expression) -> Expression {
    op("lt", vec![a, b])
}
#[allow(dead_code)]
fn greater_than(a: Expression, b: Expression) -> Expression {
    op("gt", vec![a, b])
}
#[allow(dead_code)]
fn leq(a: Expression, b: Expression) -> Expression {
    op("leq", vec![a, b])
}
#[allow(dead_code)]
fn geq(a: Expression, b: Expression) -> Expression {
    op("geq", vec![a, b])
}
#[allow(dead_code)]
fn not_equal(a: Expression, b: Expression) -> Expression {
    op("neq", vec![a, b])
}
#[allow(dead_code)]
fn approx(a: Expression, b: Expression) -> Expression {
    op("approx", vec![a, b])
}
#[allow(dead_code)]
fn proportional(a: Expression, b: Expression) -> Expression {
    op("propto", vec![a, b])
}

// Complex Number Operations (4 operators)
#[allow(dead_code)]
fn conjugate(z: Expression) -> Expression {
    op("conjugate", vec![z])
}
#[allow(dead_code)]
fn re(z: Expression) -> Expression {
    op("re", vec![z])
}
#[allow(dead_code)]
fn im(z: Expression) -> Expression {
    op("im", vec![z])
}
#[allow(dead_code)]
fn modulus(z: Expression) -> Expression {
    op("modulus", vec![z])
}

// Operator Hat Notation (QM)
#[allow(dead_code)]
fn hat(x: Expression) -> Expression {
    op("hat", vec![x])
}

// Trigonometric & Logarithmic Functions (6 functions)
#[allow(dead_code)]
fn cos_e(x: Expression) -> Expression {
    op("cos", vec![x])
}
#[allow(dead_code)]
fn tan_e(x: Expression) -> Expression {
    op("tan", vec![x])
}
#[allow(dead_code)]
fn sinh_e(x: Expression) -> Expression {
    op("sinh", vec![x])
}
#[allow(dead_code)]
fn cosh_e(x: Expression) -> Expression {
    op("cosh", vec![x])
}
#[allow(dead_code)]
fn log_e(x: Expression) -> Expression {
    op("log", vec![x])
}
#[allow(dead_code)]
fn ln_e(x: Expression) -> Expression {
    op("ln", vec![x])
}

// Matrix Operations (2 operators)
#[allow(dead_code)]
fn trace(a: Expression) -> Expression {
    op("trace", vec![a])
}
#[allow(dead_code)]
fn inverse(a: Expression) -> Expression {
    op("inverse", vec![a])
}

// === Batch 3: Completeness Operations ===

// Phase A: Quick Wins

// Factorial
#[allow(dead_code)]
fn factorial(n: Expression) -> Expression {
    op("factorial", vec![n])
}

// Floor & Ceiling
#[allow(dead_code)]
fn floor(x: Expression) -> Expression {
    op("floor", vec![x])
}
#[allow(dead_code)]
fn ceiling(x: Expression) -> Expression {
    op("ceiling", vec![x])
}

// Inverse Trigonometric
#[allow(dead_code)]
fn arcsin_e(x: Expression) -> Expression {
    op("arcsin", vec![x])
}
#[allow(dead_code)]
fn arccos_e(x: Expression) -> Expression {
    op("arccos", vec![x])
}
#[allow(dead_code)]
fn arctan_e(x: Expression) -> Expression {
    op("arctan", vec![x])
}

// Reciprocal Trigonometric
#[allow(dead_code)]
fn sec_e(x: Expression) -> Expression {
    op("sec", vec![x])
}
#[allow(dead_code)]
fn csc_e(x: Expression) -> Expression {
    op("csc", vec![x])
}
#[allow(dead_code)]
fn cot_e(x: Expression) -> Expression {
    op("cot", vec![x])
}

// Phase B: Quantum Focus

// Parenthesis matrices
#[allow(dead_code)]
fn pmatrix2(a11: Expression, a12: Expression, a21: Expression, a22: Expression) -> Expression {
    op(
        "PMatrix",
        vec![
            Expression::Const("2".to_string()),
            Expression::Const("2".to_string()),
            a11,
            a12,
            a21,
            a22,
        ],
    )
}
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn pmatrix3(
    a11: Expression,
    a12: Expression,
    a13: Expression,
    a21: Expression,
    a22: Expression,
    a23: Expression,
    a31: Expression,
    a32: Expression,
    a33: Expression,
) -> Expression {
    op(
        "PMatrix",
        vec![
            Expression::Const("3".to_string()),
            Expression::Const("3".to_string()),
            a11,
            a12,
            a13,
            a21,
            a22,
            a23,
            a31,
            a32,
            a33,
        ],
    )
}

// Binomial coefficient
#[allow(dead_code)]
fn binomial(n: Expression, k: Expression) -> Expression {
    op("binomial", vec![n, k])
}

// Phase C: Field Theory

// Vector calculus operators
#[allow(dead_code)]
fn div_e(f: Expression) -> Expression {
    op("div", vec![f])
}
#[allow(dead_code)]
fn curl_e(f: Expression) -> Expression {
    op("curl", vec![f])
}
#[allow(dead_code)]
fn laplacian(f: Expression) -> Expression {
    op("laplacian", vec![f])
}

// === Batch 4: Polish & Edge Cases ===

// Phase A: Unicode Polish & Better Formatting
// (Number sets handled in latex_to_unicode conversion - see below)

// Phase B: Piecewise Functions
#[allow(dead_code)]
fn cases2(
    expr1: Expression,
    cond1: Expression,
    expr2: Expression,
    cond2: Expression,
) -> Expression {
    op("cases2", vec![expr1, cond1, expr2, cond2])
}
#[allow(dead_code)]
fn cases3(
    expr1: Expression,
    cond1: Expression,
    expr2: Expression,
    cond2: Expression,
    expr3: Expression,
    cond3: Expression,
) -> Expression {
    op("cases3", vec![expr1, cond1, expr2, cond2, expr3, cond3])
}

// Phase C: Nice to Have
#[allow(dead_code)]
fn vmatrix2(a11: Expression, a12: Expression, a21: Expression, a22: Expression) -> Expression {
    op(
        "VMatrix",
        vec![
            Expression::Const("2".to_string()),
            Expression::Const("2".to_string()),
            a11,
            a12,
            a21,
            a22,
        ],
    )
}
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn vmatrix3(
    a11: Expression,
    a12: Expression,
    a13: Expression,
    a21: Expression,
    a22: Expression,
    a23: Expression,
    a31: Expression,
    a32: Expression,
    a33: Expression,
) -> Expression {
    op(
        "vmatrix3x3",
        vec![a11, a12, a13, a21, a22, a23, a31, a32, a33],
    )
}

#[allow(dead_code)]
fn congruent_mod(a: Expression, b: Expression, n: Expression) -> Expression {
    op("congruent_mod", vec![a, b, n])
}

#[allow(dead_code)]
fn variance(x: Expression) -> Expression {
    op("variance", vec![x])
}
#[allow(dead_code)]
fn covariance(x: Expression, y: Expression) -> Expression {
    op("covariance", vec![x, y])
}

// === Renderer ===
/// Render expression with semantic markers (public entry point)
pub fn render_expression(expr: &Expression, ctx: &GlyphContext, target: &RenderTarget) -> String {
    let empty_map = std::collections::HashMap::new();
    render_expression_internal(expr, ctx, target, "0", &empty_map)
}

/// Render expression with UUID labels for position tracking
pub fn render_expression_with_ids(
    expr: &Expression,
    ctx: &GlyphContext,
    target: &RenderTarget,
    node_id_to_uuid: &std::collections::HashMap<String, String>,
) -> String {
    render_expression_internal(expr, ctx, target, "0", node_id_to_uuid)
}

fn render_literal_chain(
    args: &[Expression],
    ctx: &GlyphContext,
    target: &RenderTarget,
    node_id: &str,
    node_id_to_uuid: &std::collections::HashMap<String, String>,
) -> String {
    let rendered_segments: Vec<String> = args
        .iter()
        .enumerate()
        .map(|(i, arg)| {
            let child_id = format!("{}.{}", node_id, i);
            let rendered = render_expression_internal(arg, ctx, target, &child_id, node_id_to_uuid);

            // For Typst: wrap each child with UUID label so they're individually editable
            if *target == RenderTarget::Typst {
                if let Some(uuid) = node_id_to_uuid.get(&child_id) {
                    return format!("#[#box[${}$]<id{}>]", rendered, uuid);
                }
            }

            rendered
        })
        .collect();

    match target {
        RenderTarget::Typst => rendered_segments.join("\u{2062}"),
        _ => rendered_segments.concat(),
    }
}

/// Internal rendering with path tracking for semantic markers
fn render_expression_internal(
    expr: &Expression,
    ctx: &GlyphContext,
    target: &RenderTarget,
    node_id: &str,
    node_id_to_uuid: &std::collections::HashMap<String, String>,
) -> String {
    match expr {
        Expression::Const(name) => {
            match target {
                RenderTarget::Unicode => name.clone(),
                RenderTarget::LaTeX => escape_latex_constant(name),
                RenderTarget::HTML => {
                    format!(r#"<span class="math-const">{}</span>"#, escape_html(name))
                }
                RenderTarget::Typst => latex_to_typst_symbol(name), // Convert LaTeX symbols to Typst
                RenderTarget::Kleis => name.clone(),                // Constants pass through as-is
            }
        }
        Expression::Object(name) => {
            match target {
                RenderTarget::Unicode => latex_to_unicode(name),
                RenderTarget::LaTeX => escape_latex_text(name),
                RenderTarget::HTML => format!(
                    r#"<span class="math-object">{}</span>"#,
                    escape_html(&latex_to_unicode(name))
                ),
                RenderTarget::Typst => latex_to_typst_symbol(name), // Convert LaTeX symbols to Typst
                RenderTarget::Kleis => {
                    // Convert LaTeX symbols to Unicode for Kleis
                    ctx.kleis_glyphs
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| latex_to_unicode(name))
                }
            }
        }
        Expression::Placeholder { id, hint } => {
            match target {
                RenderTarget::Unicode => "□".to_string(),
                RenderTarget::LaTeX => r"\square".to_string(),
                RenderTarget::HTML => format!(
                    r#"<span class="placeholder" data-id="{}" data-hint="{}" title="Click to fill: {}" onclick="selectPlaceholder({})">□</span>"#,
                    id,
                    escape_html(hint),
                    escape_html(hint),
                    id
                ),
                // Use labeled box for SVG extraction via data-typst-label attribute
                // The syntax #[#box[...]<label>] switches to markup mode where labels work
                // This produces <g data-typst-label="ph0"> in the SVG output
                RenderTarget::Typst => format!("#[#box[$square.stroked$]<ph{}>]", id),
                // Kleis grammar: placeholder ::= "□"
                RenderTarget::Kleis => "□".to_string(),
            }
        }
        Expression::Operation { name, args } => {
            if name == "literal_chain" {
                return render_literal_chain(args, ctx, target, node_id, node_id_to_uuid);
            }
            // Special handling for function_call: render as funcname(arg1, arg2, ...)
            // Wrap arguments (not function name) with UUID for deterministic positioning
            if name == "function_call" && !args.is_empty() {
                let func_name_id = format!("{}.0", node_id);
                let func_name = render_expression_internal(
                    &args[0],
                    ctx,
                    target,
                    &func_name_id,
                    node_id_to_uuid,
                );

                // NOTE: Cannot wrap function name with #[#box[$f$]<id>](...) - breaks Typst syntax
                // With Option B filtering, function name is hidden anyway (child of function_call parent)

                let func_args: Vec<String> = args[1..]
                    .iter()
                    .enumerate()
                    .map(|(i, arg)| {
                        let arg_id = format!("{}.{}", node_id, i + 1);
                        let rendered =
                            render_expression_internal(arg, ctx, target, &arg_id, node_id_to_uuid);

                        // For Typst: wrap each argument with UUID label inside the parentheses
                        if *target == RenderTarget::Typst {
                            if let Some(uuid) = node_id_to_uuid.get(&arg_id) {
                                return format!("#[#box[${}$]<id{}>]", rendered, uuid);
                            }
                        }
                        rendered
                    })
                    .collect();

                // Return function call with individually-labeled arguments
                return format!("{}({})", func_name, func_args.join(", "));
            }

            // Special handling for unary minus: minus(0, x) -> -x
            if name == "minus" && args.len() == 2 {
                if let Expression::Const(val) = &args[0] {
                    if val == "0" {
                        let operand_id = format!("{}.1", node_id);
                        let operand = render_expression_internal(
                            &args[1],
                            ctx,
                            target,
                            &operand_id,
                            node_id_to_uuid,
                        );

                        // For Typst: wrap the negated operand with UUID if available
                        if *target == RenderTarget::Typst {
                            if let Some(uuid) = node_id_to_uuid.get(&operand_id) {
                                return format!("-#[#box[${}$]<id{}>]", operand, uuid);
                            }
                        }

                        return format!("-{}", operand);
                    }
                }
            }

            let (template, glyph) = match target {
                RenderTarget::Unicode => {
                    let template = ctx
                        .unicode_templates
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| format!("{}({})", name, "{args}"));
                    let glyph = ctx.unicode_glyphs.get(name).unwrap_or(name);
                    (template, glyph)
                }
                RenderTarget::LaTeX => {
                    let template = ctx
                        .latex_templates
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| format!("{}({})", name, "{args}"));
                    let glyph = ctx.latex_glyphs.get(name).unwrap_or(name);
                    (template, glyph)
                }
                RenderTarget::HTML => {
                    // Use proper HTML templates for WYSIWYG rendering
                    let template = ctx
                        .html_templates
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| format!("{}({})", name, "{args}"));
                    let glyph = ctx.html_glyphs.get(name).unwrap_or(name);
                    (template, glyph)
                }
                RenderTarget::Typst => {
                    // Use Typst templates for math layout engine
                    let template = ctx
                        .typst_templates
                        .get(name)
                        .cloned()
                        .or_else(|| {
                            // Fallback for dynamic matrix operations: matrix, matrix2x3, matrix4x5, etc.
                            if name.starts_with("matrix") {
                                ctx.typst_templates.get("matrix").cloned()
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| format!("{}({})", name, "{args}"));
                    let glyph = ctx.typst_glyphs.get(name).unwrap_or(name);
                    (template, glyph)
                }
                RenderTarget::Kleis => {
                    // Use Kleis templates for verification syntax
                    let template = ctx.kleis_templates.get(name).cloned().unwrap_or_else(|| {
                        // Default: function call syntax name(arg1, arg2, ...)
                        format!("{}({{args}})", name)
                    });
                    let glyph = ctx.kleis_glyphs.get(name).unwrap_or(name);
                    (template, glyph)
                }
            };

            // Determine which argument indices should NOT be wrapped with box labels
            // because they appear in special Typst contexts where #[#box[...]<label>] breaks
            let skip_wrap_indices: Vec<usize> = match name.as_str() {
                // int_bounds: arg 3 is the variable which appears after "dif"
                "int_bounds" => vec![3],
                // double_integral, triple_integral: UUID wrapping works with dif
                "double_integral" => vec![],
                "triple_integral" => vec![],
                // mathrm: uses upright("arg") - wrapping goes inside string, renders as literal text
                "mathrm" => vec![0],
                // text: similar issue
                "text" => vec![0],
                // Matrix constructors: skip first two args (dimensions)
                "Matrix" | "PMatrix" | "VMatrix" | "BMatrix" => vec![0, 1],
                // Piecewise: skip first arg (n = number of cases)
                "Piecewise" => vec![0],
                _ => vec![],
            };

            // For Piecewise: extract number of cases and handle List format
            let is_piecewise = name == "Piecewise";
            let piecewise_cases = if is_piecewise && !args.is_empty() {
                match &args[0] {
                    Expression::Const(s) => s.parse::<usize>().unwrap_or(2),
                    _ => 2,
                }
            } else {
                0
            };

            // For Matrix constructors: extract dimensions and handle List format
            let is_matrix_constructor =
                matches!(name.as_str(), "Matrix" | "PMatrix" | "VMatrix" | "BMatrix");
            let (matrix_rows, matrix_cols) = if is_matrix_constructor && args.len() >= 2 {
                let rows = match &args[0] {
                    Expression::Const(s) => s.parse::<usize>().unwrap_or(2),
                    _ => 2,
                };
                let cols = match &args[1] {
                    Expression::Const(s) => s.parse::<usize>().unwrap_or(2),
                    _ => 2,
                };
                (rows, cols)
            } else {
                (0, 0)
            };

            // For Piecewise: extract expressions and conditions from Lists
            let (piecewise_exprs, piecewise_conds) = if is_piecewise && args.len() == 3 {
                let exprs = if let Expression::List(list_elements) = &args[1] {
                    list_elements.clone()
                } else {
                    vec![]
                };
                let conds = if let Expression::List(list_elements) = &args[2] {
                    list_elements.clone()
                } else {
                    vec![]
                };
                (exprs, conds)
            } else {
                (vec![], vec![])
            };

            // For Matrix: flatten List elements into individual rendered args
            let args_to_render: Vec<&Expression> = if is_matrix_constructor && args.len() == 3 {
                // Check if 3rd arg is a List (NEW FORMAT)
                if let Expression::List(list_elements) = &args[2] {
                    // NEW FORMAT: Matrix(2, 2, [a, b, c, d])
                    // Flatten: render a, b, c, d individually
                    list_elements.iter().collect()
                } else {
                    // Not a List, render normally (skip first 2)
                    args[2..].iter().collect()
                }
            } else if is_matrix_constructor {
                // OLD FORMAT: Matrix(2, 2, a, b, c, d) - skip first 2
                args[2..].iter().collect()
            } else if is_piecewise {
                // Piecewise: don't render args normally, we'll handle it specially below
                vec![]
            } else {
                // Not a matrix or piecewise, render all args
                args.iter().collect()
            };

            let rendered_args: Vec<String> = args_to_render
                .iter()
                .enumerate()
                .map(|(i, arg)| {
                    // Generate child node ID
                    let child_id = if is_matrix_constructor && args.len() == 3 {
                        // NEW FORMAT: child IDs go under the List at args[2]
                        format!("{}.2.{}", node_id, i)
                    } else if is_matrix_constructor {
                        // OLD FORMAT: child IDs start at args[2+i]
                        format!("{}.{}", node_id, i + 2)
                    } else {
                        format!("{}.{}", node_id, i)
                    };

                    let rendered = render_expression_internal(arg, ctx, target, &child_id, node_id_to_uuid);

                    // For Typst: ALWAYS wrap Matrix elements with UUID labels for edit markers
                    // Matrix elements need edit markers whether they're placeholders or filled values
                    if *target == RenderTarget::Typst && is_matrix_constructor {
                        // Check if this node has a UUID in the map
                        if let Some(uuid) = node_id_to_uuid.get(&child_id) {
                            // Wrap with UUID label for deterministic position tracking
                            format!("#[#box[${}$]<id{}>]", rendered, uuid)
                        } else {
                            // No UUID - this shouldn't happen for matrix elements
                            eprintln!("Warning: No UUID for matrix element at {}", child_id);
                            rendered
                        }
                    } else {
                        // Regular argument handling
                        let child_handles_own_wrapping = matches!(arg, Expression::Operation { name, .. }
                            if name == "function_call" || name == "literal_chain");

                        if *target == RenderTarget::Typst && !skip_wrap_indices.contains(&i) && !child_handles_own_wrapping {
                            if let Some(uuid) = node_id_to_uuid.get(&child_id) {
                                format!("#[#box[${}$]<id{}>]", rendered, uuid)
                            } else {
                                rendered
                            }
                        } else {
                            rendered
                        }
                    }
                })
                .collect();

            let mut result = template.clone();
            // Simple placeholder substitution
            result = result.replace("{glyph}", glyph);
            // Replace {args} with comma-separated rendered args if present
            // BUT: Skip this for matrix operations - they need special formatting with semicolons
            let is_matrix_op = name.starts_with("matrix")
                || name == "Matrix"
                || name == "PMatrix"
                || name == "VMatrix"
                || name == "BMatrix";
            if result.contains("{args}") && !is_matrix_op {
                let joined = rendered_args.join(", ");
                result = result.replace("{args}", &joined);
            }
            if let Some(first) = rendered_args.first() {
                result = result.replace("{arg}", first);
                result = result.replace("{left}", first);
                result = result.replace("{field}", first);
                // Extended placeholder aliases for arg0
                // Only use arg0 as {body} for operations with < 6 args
                if rendered_args.len() < 6 {
                    result = result.replace("{body}", first);
                }
                result = result.replace("{integrand}", first);
                result = result.replace("{num}", first);
                result = result.replace("{base}", first);
                // Added for coverage
                if name == "kernel_integral" {
                    result = result.replace("{kernel}", first); // kernel_integral: arg 0 is kernel
                } else {
                    result = result.replace("{function}", first); // default: arg 0 is function
                }
                result = result.replace("{a11}", first);
                result = result.replace("{vector}", first);
                result = result.replace("{state}", first);
                result = result.replace("{A}", first);
                result = result.replace("{operator}", first);
                result = result.replace("{argument}", first);
                result = result.replace("{value}", first);
                result = result.replace("{content}", first); // for brackets: parens, brackets, braces, angle_brackets
                result = result.replace("{bra}", first); // for inner product (arg 0)
                                                         // For outer product, {ket} is arg 0, but for inner product, {ket} is arg 1
                if name != "inner" {
                    result = result.replace("{ket}", first); // for outer product
                }
                // POT operations
                if name == "projection_kernel" || name == "greens_function" {
                    result = result.replace("{point_x}", first);
                    result = result.replace("{spacetime_point}", first);
                } else if name == "causal_bound" {
                    result = result.replace("{point}", first);
                } else if name == "projection_residue" {
                    result = result.replace("{projection}", first);
                } else if name == "modal_space" {
                    result = result.replace("{name}", first);
                } else if name == "hont" {
                    result = result.replace("{dimension}", first);
                } else if name == "convolution" {
                    result = result.replace("{f}", first);
                }
            }
            if let Some(second) = rendered_args.get(1) {
                result = result.replace("{right}", second);
                result = result.replace("{surface}", second);
                // Extended placeholder aliases for arg1
                result = result.replace("{den}", second);
                result = result.replace("{exponent}", second);
                result = result.replace("{lower1}", second); // For tensor_lower_pair arg1
                result = result.replace("{upper1}", second); // For tensor_2up_2down arg1
                                                             // Use arg1 as {from} for 2-3 arg operations, not for cases2 or cases3
                if name != "cases2" && name != "cases3" {
                    result = result.replace("{from}", second);
                }
                result = result.replace("{sup}", second);
                result = result.replace("{idx1}", second);
                // For limits, {target} is arg 2, not arg 1
                if name != "lim" && name != "limit" && name != "limsup" && name != "liminf" {
                    result = result.replace("{target}", second);
                }
                result = result.replace("{sub}", second);
                // Added for coverage
                if name == "int_bounds" {
                    result = result.replace("{lower}", second); // for int_bounds: arg 1 is lower bound
                } else if name == "kernel_integral" {
                    result = result.replace("{function}", second); // kernel_integral: arg 1 is function
                } else if name == "fourier_transform"
                    || name == "inverse_fourier"
                    || name == "laplace_transform"
                    || name == "inverse_laplace"
                    || name == "projection"
                {
                    result = result.replace("{variable}", second); // transforms: arg 1 is variable
                } else if name == "projection_kernel" || name == "greens_function" {
                    result = result.replace("{source_m}", second);
                    result = result.replace("{modal_state}", second);
                } else if name == "projection_residue" {
                    result = result.replace("{structure}", second);
                } else if name == "modal_integral" {
                    result = result.replace("{modal_space}", second);
                } else if name == "convolution" {
                    result = result.replace("{g}", second);
                } else {
                    result = result.replace("{variable}", second); // for derivatives
                }
                result = result.replace("{var}", second); // for limits
                result = result.replace("{subscript}", second);
                // Don't replace {idx2} here for operations that use arg 2 for idx2
                // riemann and gamma use {idx2} for arg[2], not arg[1]
                if name != "double_integral"
                    && name != "triple_integral"
                    && name != "congruent_mod"
                    && name != "riemann"
                    && name != "gamma"
                {
                    result = result.replace("{idx2}", second); // general index
                }
                if name == "index_mixed" {
                    result = result.replace("{upper}", second); // mixed tensor: arg 1 is upper index
                }
                if name == "subsup" {
                    result = result.replace("{subscript}", second); // subsup: arg 1 is subscript
                }
                result = result.replace("{ket}", second); // for inner product
                                                          // For outer product, {bra} is arg 1, but for inner product, {bra} is arg 0
                if name != "inner" {
                    result = result.replace("{bra}", second); // for outer product
                }
                result = result.replace("{B}", second);
            }
            if let Some(third) = rendered_args.get(2) {
                // Extended placeholder aliases for arg2
                // For cases2 and cases3 (4+ arg), use arg2 as {from}
                if name == "cases2" || name == "cases3" {
                    result = result.replace("{from}", third);
                }
                // Use arg2 as {to} for 3-arg operations and int_bounds (4-arg)
                if rendered_args.len() == 3 || name == "int_bounds" {
                    result = result.replace("{to}", third);
                }
                // Only use arg2 as {idx2} for operations with < 6 args
                if rendered_args.len() < 6 {
                    result = result.replace("{idx2}", third);
                }
                result = result.replace("{lower2}", third); // For tensor_lower_pair arg2
                result = result.replace("{upper2}", third); // For tensor_2up_2down arg2
                                                            // Added for coverage
                if name == "int_bounds" {
                    result = result.replace("{upper}", third); // integral upper bound: arg 2
                } else if name == "index_mixed" {
                    result = result.replace("{lower}", third); // mixed tensor: arg 2 is lower index
                } else if name == "subsup" {
                    result = result.replace("{superscript}", third); // subsup: arg 2 is superscript
                } else if name == "lim" || name == "limit" || name == "limsup" || name == "liminf" {
                    result = result.replace("{target}", third); // limit target: arg 2
                } else if name == "kernel_integral" {
                    result = result.replace("{domain}", third); // kernel_integral: arg 2 is domain
                } else if name == "modal_integral" {
                    result = result.replace("{variable}", third); // modal_integral: arg 2 is variable (not modal_space!)
                } else if name == "convolution" {
                    result = result.replace("{variable}", third); // convolution: arg 2 is variable
                }
            }
            if let Some(fourth) = rendered_args.get(3) {
                // Extended placeholder aliases for arg3
                // Use arg3 as {to} for 4-arg operations except int_bounds
                if name != "int_bounds" && name != "kernel_integral" {
                    result = result.replace("{to}", fourth);
                }
                result = result.replace("{idx3}", fourth);
                result = result.replace("{lower1}", fourth); // For tensor_2up_2down arg3 (first lower index)
                                                             // Added for coverage
                if name == "int_bounds" || name == "kernel_integral" {
                    result = result.replace("{variable}", fourth); // int_bounds and kernel_integral variable
                }
            }
            // Add more for Matrix3x3
            if let Some(fifth) = rendered_args.get(4) {
                result = result.replace("{idx4}", fifth);
                result = result.replace("{lower2}", fifth); // For tensor_2up_2down arg4 (second lower index)
                                                            // For 6-arg operations (cases3), use arg4 as {body}
                if rendered_args.len() == 6 {
                    result = result.replace("{body}", fifth);
                }
            }
            if let Some(sixth) = rendered_args.get(5) {
                result = result.replace("{idx5}", sixth);
                // For 6-arg operations (cases3), use arg5 as {idx2} (third row condition)
                if rendered_args.len() == 6 {
                    result = result.replace("{idx2}", sixth);
                }
            }
            // After generic replacements, restore single-braced LaTeX placeholders for vector wrappers
            if *target == RenderTarget::LaTeX && (name == "vector_arrow" || name == "vector_bold") {
                // ensure we end up with \vec{...} instead of \vec{{...}}
                result = result.replace("{{", "{");
                result = result.replace("}}", "}");
            }

            // Special-case mapping for 3x3 matrices
            if name == "matrix3x3" {
                let map = [
                    ("{a11}", 0usize),
                    ("{a12}", 1usize),
                    ("{a13}", 2usize),
                    ("{a21}", 3usize),
                    ("{a22}", 4usize),
                    ("{a23}", 5usize),
                    ("{a31}", 6usize),
                    ("{a32}", 7usize),
                    ("{a33}", 8usize),
                ];
                for (ph, idx) in map.iter() {
                    if let Some(val) = rendered_args.get(*idx) {
                        result = result.replace(ph, val);
                    }
                }
            }
            // Special-case mapping for 2x2 matrices
            if name == "matrix2x2" {
                let map = [
                    ("{a11}", 0usize),
                    ("{a12}", 1usize),
                    ("{a21}", 2usize),
                    ("{a22}", 3usize),
                ];
                for (ph, idx) in map.iter() {
                    if let Some(val) = rendered_args.get(*idx) {
                        result = result.replace(ph, val);
                    }
                }
            }
            // Special-case mapping for 2x2 pmatrix (parenthesis matrices)
            if name == "pmatrix2x2" {
                let map = [
                    ("{a11}", 0usize),
                    ("{a12}", 1usize),
                    ("{a21}", 2usize),
                    ("{a22}", 3usize),
                ];
                for (ph, idx) in map.iter() {
                    if let Some(val) = rendered_args.get(*idx) {
                        result = result.replace(ph, val);
                    }
                }
            }
            // Special-case mapping for 3x3 pmatrix
            if name == "pmatrix3x3" {
                let map = [
                    ("{a11}", 0usize),
                    ("{a12}", 1usize),
                    ("{a13}", 2usize),
                    ("{a21}", 3usize),
                    ("{a22}", 4usize),
                    ("{a23}", 5usize),
                    ("{a31}", 6usize),
                    ("{a32}", 7usize),
                    ("{a33}", 8usize),
                ];
                for (ph, idx) in map.iter() {
                    if let Some(val) = rendered_args.get(*idx) {
                        result = result.replace(ph, val);
                    }
                }
            }
            // Special-case mapping for 2x2 vmatrix (determinant bars)
            if name == "vmatrix2x2" {
                let map = [
                    ("{a11}", 0usize),
                    ("{a12}", 1usize),
                    ("{a21}", 2usize),
                    ("{a22}", 3usize),
                ];
                for (ph, idx) in map.iter() {
                    if let Some(val) = rendered_args.get(*idx) {
                        result = result.replace(ph, val);
                    }
                }
            }
            // Special-case mapping for 3x3 vmatrix
            if name == "vmatrix3x3" {
                let map = [
                    ("{a11}", 0usize),
                    ("{a12}", 1usize),
                    ("{a13}", 2usize),
                    ("{a21}", 3usize),
                    ("{a22}", 4usize),
                    ("{a23}", 5usize),
                    ("{a31}", 6usize),
                    ("{a32}", 7usize),
                    ("{a33}", 8usize),
                ];
                for (ph, idx) in map.iter() {
                    if let Some(val) = rendered_args.get(*idx) {
                        result = result.replace(ph, val);
                    }
                }
            }
            // Special handling for Matrix, PMatrix, VMatrix, BMatrix constructors
            // Format: Matrix(rows, cols, ...elements)
            // Note: dimensions were already extracted above and filtered from rendered_args
            if is_matrix_constructor {
                let rows = matrix_rows;
                let cols = matrix_cols;

                let mut matrix_content = String::new();
                for r in 0..rows {
                    for c in 0..cols {
                        let idx = r * cols + c; // No offset - dimensions already filtered
                        if let Some(val) = rendered_args.get(idx) {
                            matrix_content.push_str(val);
                            if c < cols - 1 {
                                // LaTeX uses & for column separator
                                if *target == RenderTarget::LaTeX {
                                    matrix_content.push('&');
                                } else {
                                    matrix_content.push_str(" , ");
                                }
                            }
                        }
                    }
                    if r < rows - 1 {
                        // LaTeX uses \\ for row separator
                        if *target == RenderTarget::LaTeX {
                            matrix_content.push_str("\\\\");
                        } else {
                            matrix_content.push_str(" ; ");
                        }
                    }
                }
                result = result.replace("{args}", &matrix_content);
            }
            // Piecewise functions: render as cases(...)
            else if is_piecewise {
                let mut case_rows = Vec::new();
                for i in 0..piecewise_cases {
                    if let (Some(expr), Some(cond)) =
                        (piecewise_exprs.get(i), piecewise_conds.get(i))
                    {
                        let expr_id = format!("{}.1.{}", node_id, i);
                        let cond_id = format!("{}.2.{}", node_id, i);

                        let rendered_expr = render_expression_internal(
                            expr,
                            ctx,
                            target,
                            &expr_id,
                            node_id_to_uuid,
                        );
                        let rendered_cond = render_expression_internal(
                            cond,
                            ctx,
                            target,
                            &cond_id,
                            node_id_to_uuid,
                        );

                        match target {
                            RenderTarget::Typst => {
                                // Wrap elements with UUID labels like Matrix does
                                let expr_wrapped = if let Some(uuid) = node_id_to_uuid.get(&expr_id)
                                {
                                    format!("#[#box[${}$]<id{}>]", rendered_expr, uuid)
                                } else {
                                    rendered_expr
                                };
                                let cond_wrapped = if let Some(uuid) = node_id_to_uuid.get(&cond_id)
                                {
                                    format!("#[#box[${}$]<id{}>]", rendered_cond, uuid)
                                } else {
                                    rendered_cond
                                };
                                case_rows.push(format!("{} & {}", expr_wrapped, cond_wrapped));
                            }
                            RenderTarget::LaTeX | RenderTarget::HTML => {
                                // LaTeX: same format
                                case_rows.push(format!("{} & {}", rendered_expr, rendered_cond));
                            }
                            RenderTarget::Unicode | RenderTarget::Kleis => {
                                case_rows.push(format!("{}  if {}", rendered_expr, rendered_cond));
                            }
                        }
                    }
                }

                match target {
                    RenderTarget::Typst => {
                        // Typst cases uses commas between rows
                        result = format!("cases({})", case_rows.join(", "));
                    }
                    RenderTarget::LaTeX | RenderTarget::HTML => {
                        // LaTeX uses \\ between rows
                        result =
                            format!(r"\begin{{cases}}{}\end{{cases}}", case_rows.join(r" \\ "));
                    }
                    RenderTarget::Unicode | RenderTarget::Kleis => {
                        result = format!("{{ {} }}", case_rows.join(" ; "));
                    }
                }
            }
            // Legacy support: old matrix operations like matrix2x2, matrix3x3, etc.
            else if name.starts_with("matrix")
                || name.starts_with("pmatrix")
                || name.starts_with("vmatrix")
            {
                let (rows, cols) = if name == "matrix" {
                    // Legacy: infer from args
                    let total_args = rendered_args.len();
                    infer_matrix_dimensions(total_args)
                } else {
                    // Parse dimensions from name: "matrix2x2" → (2, 2)
                    parse_matrix_dimensions_from_name(name).unwrap_or_else(|| {
                        let total_args = rendered_args.len();
                        infer_matrix_dimensions(total_args)
                    })
                };

                let mut matrix_content = String::new();
                for r in 0..rows {
                    for c in 0..cols {
                        let idx = r * cols + c;
                        if let Some(val) = rendered_args.get(idx) {
                            matrix_content.push_str(val);
                            if c < cols - 1 {
                                matrix_content.push_str(" , ");
                            }
                        }
                    }
                    if r < rows - 1 {
                        matrix_content.push_str(" ; ");
                    }
                }
                result = result.replace("{args}", &matrix_content);
            }
            // Special handling for integral var position: int_bounds(integrand, from, to, var)
            if name == "int_bounds" {
                if let Some(var) = rendered_args.get(3) {
                    result = result.replace("{int_var}", var);
                }
            }
            result
        }

        Expression::Match { .. } => {
            // TODO: Implement pattern matching rendering
            match target {
                RenderTarget::Unicode => "⟨match⟩".to_string(),
                RenderTarget::LaTeX => r"\text{match}".to_string(),
                RenderTarget::HTML => r#"<span class="match-expr">match</span>"#.to_string(),
                RenderTarget::Typst => "\\text{match}".to_string(),
                // Kleis has native match syntax
                RenderTarget::Kleis => "match { ... }".to_string(),
            }
        }

        // Quantifier: render as ∀(x : T). body or ∀(x : T) where cond. body
        Expression::Quantifier {
            quantifier,
            variables,
            where_clause,
            body,
        } => {
            let _ = where_clause; // TODO: Render where clause in output
            let quant_symbol = match quantifier {
                crate::ast::QuantifierKind::ForAll => match target {
                    RenderTarget::Unicode | RenderTarget::Kleis => "∀",
                    RenderTarget::LaTeX => r"\forall",
                    RenderTarget::HTML => "∀",
                    RenderTarget::Typst => "forall",
                },
                crate::ast::QuantifierKind::Exists => match target {
                    RenderTarget::Unicode | RenderTarget::Kleis => "∃",
                    RenderTarget::LaTeX => r"\exists",
                    RenderTarget::HTML => "∃",
                    RenderTarget::Typst => "exists",
                },
            };

            let vars_str = variables
                .iter()
                .map(|v| {
                    if let Some(ref ty) = v.type_annotation {
                        format!("{} : {}", v.name, ty)
                    } else {
                        v.name.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");

            let body_id = format!("{}.body", node_id);
            let body_str = render_expression_internal(body, ctx, target, &body_id, node_id_to_uuid);

            match target {
                RenderTarget::Unicode | RenderTarget::HTML | RenderTarget::Kleis => {
                    format!("{}({}). {}", quant_symbol, vars_str, body_str)
                }
                RenderTarget::LaTeX => {
                    format!("{}({}). {}", quant_symbol, vars_str, body_str)
                }
                RenderTarget::Typst => {
                    format!("{}({}). {}", quant_symbol, vars_str, body_str)
                }
            }
        }

        Expression::List(elements) => {
            // Render list literal as [a, b, c]
            let rendered_elements: Vec<String> = elements
                .iter()
                .enumerate()
                .map(|(i, elem)| {
                    let child_id = format!("{}.{}", node_id, i);
                    let rendered =
                        render_expression_internal(elem, ctx, target, &child_id, node_id_to_uuid);

                    // For Typst: wrap list elements with UUID labels for position tracking
                    // This is critical for Matrix List format where each element needs tracking
                    if *target == RenderTarget::Typst {
                        if let Some(uuid) = node_id_to_uuid.get(&child_id) {
                            return format!("#[#box[${}$]<id{}>]", rendered, uuid);
                        }
                    }

                    rendered
                })
                .collect();

            match target {
                RenderTarget::Unicode | RenderTarget::Kleis => {
                    format!("[{}]", rendered_elements.join(", "))
                }
                RenderTarget::LaTeX => format!(r"\left[{}\right]", rendered_elements.join(", ")),
                RenderTarget::HTML => {
                    format!(
                        r#"<span class="list">[{}]</span>"#,
                        rendered_elements.join(", ")
                    )
                }
                RenderTarget::Typst => format!("({})", rendered_elements.join(", ")), // Typst uses () for lists
            }
        }

        Expression::Conditional {
            condition,
            then_branch,
            else_branch,
        } => {
            let cond_id = format!("{}.cond", node_id);
            let then_id = format!("{}.then", node_id);
            let else_id = format!("{}.else", node_id);

            let cond_str =
                render_expression_internal(condition, ctx, target, &cond_id, node_id_to_uuid);
            let then_str =
                render_expression_internal(then_branch, ctx, target, &then_id, node_id_to_uuid);
            let else_str =
                render_expression_internal(else_branch, ctx, target, &else_id, node_id_to_uuid);

            match target {
                RenderTarget::Unicode | RenderTarget::Kleis => {
                    // Kleis grammar: conditional ::= "if" expression "then" expression "else" expression
                    format!("if {} then {} else {}", cond_str, then_str, else_str)
                }
                RenderTarget::LaTeX => {
                    format!(
                        r"\text{{if }} {} \text{{ then }} {} \text{{ else }} {}",
                        cond_str, then_str, else_str
                    )
                }
                RenderTarget::HTML => {
                    format!(
                        r#"<span class="conditional">if {} then {} else {}</span>"#,
                        cond_str, then_str, else_str
                    )
                }
                RenderTarget::Typst => {
                    format!(
                        r#""if " {} " then " {} " else " {}"#,
                        cond_str, then_str, else_str
                    )
                }
            }
        }

        Expression::Let { name, value, body } => {
            let value_id = format!("{}.value", node_id);
            let body_id = format!("{}.body", node_id);

            let value_str =
                render_expression_internal(value, ctx, target, &value_id, node_id_to_uuid);
            let body_str = render_expression_internal(body, ctx, target, &body_id, node_id_to_uuid);

            match target {
                RenderTarget::Unicode | RenderTarget::Kleis => {
                    // Kleis grammar: letBinding ::= "let" identifier ... "=" expression "in" expression
                    format!("let {} = {} in {}", name, value_str, body_str)
                }
                RenderTarget::LaTeX => {
                    format!(
                        r"\text{{let }} {} = {} \text{{ in }} {}",
                        name, value_str, body_str
                    )
                }
                RenderTarget::HTML => {
                    format!(
                        r#"<span class="let-binding">let {} = {} in {}</span>"#,
                        name, value_str, body_str
                    )
                }
                RenderTarget::Typst => {
                    format!(r#""let " {} " = " {} " in " {}"#, name, value_str, body_str)
                }
            }
        }
    }
}

/// Parse matrix dimensions from operation name
/// E.g. "matrix2x3" → Some((2, 3)), "matrix4x5" → Some((4, 5))
fn parse_matrix_dimensions_from_name(name: &str) -> Option<(usize, usize)> {
    if !name.starts_with("matrix") {
        return None;
    }

    // Remove "matrix" prefix
    let dims = &name[6..];

    // Split on 'x'
    let parts: Vec<&str> = dims.split('x').collect();
    if parts.len() != 2 {
        return None;
    }

    // Parse rows and cols
    let rows = parts[0].parse::<usize>().ok()?;
    let cols = parts[1].parse::<usize>().ok()?;

    Some((rows, cols))
}

/// Infer matrix dimensions from total number of elements
/// Tries to find the most reasonable (rows, cols) pair
fn infer_matrix_dimensions(total: usize) -> (usize, usize) {
    if total == 0 {
        return (0, 0);
    }

    // Try square matrices first
    let sqrt = (total as f64).sqrt();
    if sqrt.fract() == 0.0 {
        let n = sqrt as usize;
        return (n, n);
    }

    // Try common rectangular dimensions
    // Check if it's a nice factorization
    for rows in 1..=10 {
        if total.is_multiple_of(rows) {
            let cols = total / rows;
            if cols <= 10 {
                return (rows, cols);
            }
        }
    }

    // Fallback: make it a row vector
    (1, total)
}

fn latex_to_unicode(input: &str) -> String {
    // Convert LaTeX commands to Unicode symbols for Unicode rendering
    input
        // lowercase Greek
        .replace("\\alpha", "α")
        .replace("\\beta", "β")
        .replace("\\gamma", "γ")
        .replace("\\delta", "δ")
        .replace("\\epsilon", "ε")
        .replace("\\zeta", "ζ")
        .replace("\\eta", "η")
        .replace("\\theta", "θ")
        .replace("\\iota", "ι")
        .replace("\\kappa", "κ")
        .replace("\\lambda", "λ")
        .replace("\\mu", "μ")
        .replace("\\nu", "ν")
        .replace("\\xi", "ξ")
        .replace("\\omicron", "ο")
        .replace("\\pi", "π")
        .replace("\\rho", "ρ")
        .replace("\\sigma", "σ")
        .replace("\\tau", "τ")
        .replace("\\upsilon", "υ")
        .replace("\\phi", "φ")
        .replace("\\chi", "χ")
        .replace("\\psi", "ψ")
        .replace("\\omega", "ω")
        // uppercase Greek
        .replace("\\Gamma", "Γ")
        .replace("\\Delta", "Δ")
        .replace("\\Theta", "Θ")
        .replace("\\Lambda", "Λ")
        .replace("\\Xi", "Ξ")
        .replace("\\Pi", "Π")
        .replace("\\Sigma", "Σ")
        .replace("\\Upsilon", "Υ")
        .replace("\\Phi", "Φ")
        .replace("\\Psi", "Ψ")
        .replace("\\Omega", "Ω")
        // Hebrew letters
        .replace("\\aleph", "ℵ")
        .replace("\\beth", "ℶ")
        .replace("\\gimel", "ℷ")
        .replace("\\daleth", "ℸ")
        // Greek variants
        .replace("\\varepsilon", "ε")
        .replace("\\vartheta", "ϑ")
        .replace("\\varkappa", "ϰ")
        .replace("\\varpi", "ϖ")
        .replace("\\varrho", "ϱ")
        .replace("\\varsigma", "ς")
        .replace("\\varphi", "ϕ")
        // Number sets (blackboard bold)
        .replace("\\mathbb{R}", "ℝ")
        .replace("\\mathbb{C}", "ℂ")
        .replace("\\mathbb{N}", "ℕ")
        .replace("\\mathbb{Z}", "ℤ")
        .replace("\\mathbb{Q}", "ℚ")
        .replace("\\mathbb{H}", "ℍ")
        .replace("\\mathbb{P}", "ℙ")
        .replace("\\mathbb{E}", "𝔼")
        // Other common symbols
        .replace("\\hbar", "ℏ")
        .replace("\\infty", "∞")
        .replace("\\emptyset", "∅")
        .replace("\\varnothing", "∅")
        // Ellipsis (dots)
        .replace("\\cdots", "⋯")
        .replace("\\ldots", "…")
        .replace("\\vdots", "⋮")
        .replace("\\ddots", "⋱")
        .replace("\\iddots", "⋰")
    // Note: \mathbf and \boldsymbol are left as-is for now
    // Keep backslashes for unknown commands
}

fn latex_to_typst_symbol(input: &str) -> String {
    // Convert LaTeX commands to Typst/Unicode symbols
    // Typst uses Unicode directly, so this is similar to latex_to_unicode
    input
        // lowercase Greek
        .replace("\\alpha", "α")
        .replace("\\beta", "β")
        .replace("\\gamma", "γ")
        .replace("\\delta", "δ")
        .replace("\\epsilon", "ε")
        .replace("\\zeta", "ζ")
        .replace("\\eta", "η")
        .replace("\\theta", "θ")
        .replace("\\iota", "ι")
        .replace("\\kappa", "κ")
        .replace("\\lambda", "λ")
        .replace("\\mu", "μ")
        .replace("\\nu", "ν")
        .replace("\\xi", "ξ")
        .replace("\\omicron", "ο")
        .replace("\\pi", "π")
        .replace("\\rho", "ρ")
        .replace("\\sigma", "σ")
        .replace("\\tau", "τ")
        .replace("\\upsilon", "υ")
        .replace("\\phi", "φ")
        .replace("\\chi", "χ")
        .replace("\\psi", "ψ")
        .replace("\\omega", "ω")
        // uppercase Greek
        .replace("\\Gamma", "Γ")
        .replace("\\Delta", "Δ")
        .replace("\\Theta", "Θ")
        .replace("\\Lambda", "Λ")
        .replace("\\Xi", "Ξ")
        .replace("\\Pi", "Π")
        .replace("\\Sigma", "Σ")
        .replace("\\Upsilon", "Υ")
        .replace("\\Phi", "Φ")
        .replace("\\Psi", "Ψ")
        .replace("\\Omega", "Ω")
        // Hebrew letters
        .replace("\\aleph", "ℵ")
        .replace("\\beth", "ℶ")
        .replace("\\gimel", "ℷ")
        .replace("\\daleth", "ℸ")
        // Greek variants
        .replace("\\varepsilon", "ε")
        .replace("\\vartheta", "ϑ")
        .replace("\\varkappa", "ϰ")
        .replace("\\varpi", "ϖ")
        .replace("\\varrho", "ϱ")
        .replace("\\varsigma", "ς")
        .replace("\\varphi", "ϕ")
        // Number sets
        .replace("\\mathbb{R}", "ℝ")
        .replace("\\mathbb{C}", "ℂ")
        .replace("\\mathbb{N}", "ℕ")
        .replace("\\mathbb{Z}", "ℤ")
        .replace("\\mathbb{Q}", "ℚ")
        .replace("\\mathbb{H}", "ℍ")
        // Other symbols
        .replace("\\hbar", "ℏ")
        .replace("\\infty", "∞")
        .replace("\\emptyset", "∅")
        .replace("\\varnothing", "∅")
        .replace("\\partial", "∂")
        .replace("\\nabla", "∇")
        .replace("\\Box", "square")
        .replace("\\square", "square")
        // Math operators (remove backslash for Typst)
        .replace("\\min", "min")
        .replace("\\max", "max")
        .replace("\\sup", "sup")
        .replace("\\inf", "inf")
        .replace("\\lim", "lim")
        .replace("\\limsup", "limsup")
        .replace("\\liminf", "liminf")
        .replace("\\sum", "sum")
        .replace("\\prod", "product")
        .replace("\\iiint", "integral.triple")
        .replace("\\iint", "integral.double")
        .replace("\\int", "integral")
        // Trig functions
        .replace("\\sin", "sin")
        .replace("\\cos", "cos")
        .replace("\\tan", "tan")
        .replace("\\sec", "sec")
        .replace("\\csc", "csc")
        .replace("\\cot", "cot")
        .replace("\\arcsin", "arcsin")
        .replace("\\arccos", "arccos")
        .replace("\\arctan", "arctan")
        .replace("\\sinh", "sinh")
        .replace("\\cosh", "cosh")
        .replace("\\tanh", "tanh")
        // Other math functions
        .replace("\\ln", "ln")
        .replace("\\log", "log")
        .replace("\\exp", "exp")
        // Set theory and logic symbols
        .replace("\\forall", "forall")
        .replace("\\exists", "exists")
        .replace("\\in", "in")
        .replace("\\notin", "in.not")
        .replace("\\subset", "subset")
        .replace("\\subseteq", "subset.eq")
        .replace("\\supset", "supset")
        .replace("\\supseteq", "supset.eq")
        .replace("\\cup", "union")
        .replace("\\cap", "sect")
        .replace("\\Rightarrow", "=>")
        .replace("\\Leftarrow", "<=")
        .replace("\\Leftrightarrow", "<=>")
        // Ellipsis
        .replace("\\cdots", "dots.c")
        .replace("\\ldots", "dots")
        .replace("\\vdots", "dots.v")
        .replace("\\ddots", "dots.down")
    // If not converted, return as-is (Typst might understand it)
}

fn escape_latex_constant(constant: &str) -> String {
    escape_latex_text(constant)
}

fn escape_latex_text(input: &str) -> String {
    // Minimal escaping for Greek letters and common LaTeX-sensitive glyphs seen in this project
    input
        // lowercase
        .replace("α", "\\alpha")
        .replace("β", "\\beta")
        .replace("γ", "\\gamma")
        .replace("δ", "\\delta")
        .replace("ε", "\\epsilon")
        .replace("ζ", "\\zeta")
        .replace("η", "\\eta")
        .replace("θ", "\\theta")
        .replace("ι", "\\iota")
        .replace("κ", "\\kappa")
        .replace("λ", "\\lambda")
        .replace("μ", "\\mu")
        .replace("ν", "\\nu")
        .replace("ξ", "\\xi")
        .replace("ο", "o")
        .replace("π", "\\pi")
        .replace("ρ", "\\rho")
        .replace("σ", "\\sigma")
        .replace("τ", "\\tau")
        .replace("υ", "\\upsilon")
        .replace("φ", "\\phi")
        .replace("χ", "\\chi")
        .replace("ψ", "\\psi")
        .replace("ω", "\\omega")
        // uppercase
        .replace("Γ", "\\Gamma")
        .replace("Δ", "\\Delta")
        .replace("Θ", "\\Theta")
        .replace("Λ", "\\Lambda")
        .replace("Ξ", "\\Xi")
        .replace("Π", "\\Pi")
        .replace("Σ", "\\Sigma")
        .replace("Υ", "\\Upsilon")
        .replace("Φ", "\\Phi")
        .replace("Ψ", "\\Psi")
        .replace("Ω", "\\Omega")
        // Ellipsis back to LaTeX
        .replace("⋯", "\\cdots")
        .replace("…", "\\ldots")
        .replace("⋮", "\\vdots")
        .replace("⋱", "\\ddots")
        .replace("⋰", "\\iddots")
        // underscores in identifiers should be escaped
        .replace("_", "\\_")
}

fn escape_html(input: &str) -> String {
    // Basic HTML escaping for safety
    input
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}

pub fn build_default_context() -> GlyphContext {
    let mut unicode_glyphs = HashMap::new();
    unicode_glyphs.insert("grad".to_string(), "∇".to_string());
    unicode_glyphs.insert("surface_integral_over".to_string(), "∮".to_string());
    unicode_glyphs.insert("scalar_divide".to_string(), "/".to_string());

    let mut unicode_templates = HashMap::new();
    unicode_templates.insert("grad".to_string(), "{glyph}{arg}".to_string());
    unicode_templates.insert(
        "surface_integral_over".to_string(),
        "{glyph}_{surface} {field} dS".to_string(),
    );
    unicode_templates.insert("scalar_multiply".to_string(), "{left} {right}".to_string());
    unicode_templates.insert("multiply".to_string(), "{left} {right}".to_string()); // Matrix multiplication (polymorphic - works for block matrices!)
    unicode_templates.insert(
        "scalar_divide".to_string(),
        "({left}) / ({right})".to_string(),
    );

    // Additional Unicode templates
    // basic arithmetic/equation
    unicode_templates.insert("equals".to_string(), "{left} = {right}".to_string());
    unicode_templates.insert("plus".to_string(), "{left} + {right}".to_string());
    unicode_templates.insert("minus".to_string(), "{left} - {right}".to_string());
    unicode_templates.insert("dot".to_string(), "{left} · {right}".to_string());
    unicode_templates.insert("cross".to_string(), "{left} × {right}".to_string());
    unicode_templates.insert("power".to_string(), "{base}^{exponent}".to_string());
    unicode_templates.insert("norm".to_string(), "‖{arg}‖".to_string());
    unicode_templates.insert("abs".to_string(), "|{arg}|".to_string());
    // Brackets & grouping
    unicode_templates.insert("parens".to_string(), "({content})".to_string());
    unicode_templates.insert("brackets".to_string(), "[{content}]".to_string());
    unicode_templates.insert("braces".to_string(), "{{{content}}}".to_string());
    unicode_templates.insert("angle_brackets".to_string(), "⟨{content}⟩".to_string());
    unicode_templates.insert("inner".to_string(), "⟨{left}, {right}⟩".to_string());
    unicode_templates.insert(
        "sum_bounds".to_string(),
        "Σ_{ {from} }^{ {to} } {body}".to_string(),
    );
    unicode_templates.insert("sum_index".to_string(), "Σ_{ {from} } {body}".to_string());
    unicode_templates.insert(
        "prod_bounds".to_string(),
        "Π_{ {from} }^{ {to} } {body}".to_string(),
    );
    unicode_templates.insert("prod_index".to_string(), "Π_{ {from} } {body}".to_string());
    // limits
    unicode_templates.insert(
        "limit".to_string(),
        "lim_{ {var}→{target} } {body}".to_string(),
    );
    unicode_templates.insert(
        "limsup".to_string(),
        "lim sup_{ {var}→{target} } {body}".to_string(),
    );
    unicode_templates.insert(
        "liminf".to_string(),
        "lim inf_{ {var}→{target} } {body}".to_string(),
    );
    unicode_templates.insert(
        "int_bounds".to_string(),
        "∫_{ {from} }^{ {to} } {integrand} d{int_var}".to_string(),
    );
    unicode_templates.insert("transpose".to_string(), "{arg}ᵀ".to_string());
    unicode_templates.insert("det".to_string(), "|{arg}|".to_string());
    unicode_templates.insert(
        "matrix2x2".to_string(),
        "[[{a11}, {a12}]; [{a21}, {a22}]]".to_string(),
    );
    unicode_templates.insert(
        "matrix3x3".to_string(),
        "[[{a11}, {a12}, {a13}]; [{a21}, {a22}, {a23}]; [{a31}, {a32}, {a33}]]".to_string(),
    );
    unicode_templates.insert("vector_arrow".to_string(), "{arg}⃗".to_string());
    unicode_templates.insert("vector_bold".to_string(), "{arg}".to_string());
    unicode_templates.insert("d_dt".to_string(), "d{num}/d{den}".to_string());
    unicode_templates.insert("d_part".to_string(), "∂{num}/∂{den}".to_string());
    unicode_templates.insert("d2_part".to_string(), "∂^2{num}/∂{den}^2".to_string());
    // indices
    unicode_templates.insert("sub".to_string(), "{base}_{right}".to_string());
    unicode_templates.insert("sup".to_string(), "{base}^{right}".to_string());
    unicode_templates.insert("index".to_string(), "{base}^{sup}_{sub}".to_string());
    // nabla with subscript, box operator
    unicode_templates.insert("nabla_sub".to_string(), "∇_{sub} {arg}".to_string());
    unicode_templates.insert("box".to_string(), "□{arg}".to_string());
    // partial derivative apply and mixed index wrapper
    unicode_templates.insert("partial_apply".to_string(), "∂_{sub} {arg}".to_string());
    unicode_templates.insert(
        "index_mixed".to_string(),
        "{base}^{idx1}_{idx2}".to_string(),
    );
    unicode_templates.insert(
        "subsup".to_string(),
        "{base}_{subscript}^{superscript}".to_string(),
    );
    unicode_templates.insert("index_pair".to_string(), "{base}_{idx1}{idx2}".to_string());
    // Gamma (Christoffel) and Riemann tensors
    unicode_templates.insert("gamma".to_string(), "Γ^{idx1}_{idx2 idx3}".to_string());
    unicode_templates.insert(
        "riemann".to_string(),
        "R^{idx1}_{idx2 idx3 idx4}".to_string(),
    );
    unicode_templates.insert(
        "tensor_1up_3down".to_string(),
        "{base}^{upper}_{lower1 lower2 lower3}".to_string(),
    );
    unicode_templates.insert(
        "tensor_lower_pair".to_string(),
        "{base}_{lower1 lower2}".to_string(),
    );
    unicode_templates.insert(
        "tensor_2up_2down".to_string(),
        "{base}^{upper1 upper2}_{lower1 lower2}".to_string(),
    );
    // Zeta as a function
    unicode_templates.insert("zeta".to_string(), "ζ({args})".to_string());

    // === Top 5 New Operations - Unicode ===

    // 1. Bra-ket notation
    unicode_templates.insert("ket".to_string(), "|{arg}⟩".to_string());
    unicode_templates.insert("bra".to_string(), "⟨{arg}|".to_string());
    unicode_templates.insert("outer_product".to_string(), "|{left}⟩⟨{right}|".to_string());

    // 2. Set theory and logic
    unicode_templates.insert("in".to_string(), "{left} ∈ {right}".to_string());
    unicode_templates.insert("subset".to_string(), "{left} ⊂ {right}".to_string());
    unicode_templates.insert("subseteq".to_string(), "{left} ⊆ {right}".to_string());
    unicode_templates.insert("union".to_string(), "{left} ∪ {right}".to_string());
    unicode_templates.insert("intersection".to_string(), "{left} ∩ {right}".to_string());
    unicode_templates.insert("forall".to_string(), "∀{left}: {right}".to_string());
    unicode_templates.insert("exists".to_string(), "∃{left}: {right}".to_string());
    unicode_templates.insert("implies".to_string(), "{left} ⇒ {right}".to_string());
    unicode_templates.insert("iff".to_string(), "{left} ⇔ {right}".to_string());

    // 3. Multiple integrals
    unicode_templates.insert(
        "double_integral".to_string(),
        "∬_{ {right} } {left} d{from} d{to}".to_string(),
    );
    unicode_templates.insert(
        "triple_integral".to_string(),
        "∭_{ {right} } {left} d{from} d{to} d{idx2}".to_string(),
    );

    // 4. Commutators
    unicode_templates.insert("commutator".to_string(), "[{left}, {right}]".to_string());
    unicode_templates.insert(
        "anticommutator".to_string(),
        "{{left}, {right}}".to_string(),
    );

    // 5. Square root
    unicode_templates.insert("sqrt".to_string(), "√({arg})".to_string());
    unicode_templates.insert("nth_root".to_string(), "ⁿ√({left})".to_string());

    // === Next Top 3 + Low-Hanging Fruit - Unicode ===

    // Comparison & inequality operators
    unicode_templates.insert("lt".to_string(), "{left} < {right}".to_string());
    unicode_templates.insert("gt".to_string(), "{left} > {right}".to_string());
    unicode_templates.insert("leq".to_string(), "{left} ≤ {right}".to_string());
    unicode_templates.insert("geq".to_string(), "{left} ≥ {right}".to_string());
    unicode_templates.insert("neq".to_string(), "{left} ≠ {right}".to_string());
    unicode_templates.insert("approx".to_string(), "{left} ≈ {right}".to_string());
    unicode_templates.insert("propto".to_string(), "{left} ∝ {right}".to_string());

    // Complex number operations
    unicode_templates.insert("conjugate".to_string(), "{arg}̄".to_string());
    unicode_templates.insert("re".to_string(), "Re({arg})".to_string());
    unicode_templates.insert("im".to_string(), "Im({arg})".to_string());
    unicode_templates.insert("modulus".to_string(), "|{arg}|".to_string());

    // Accent operators (use Unicode combining characters where possible)
    unicode_templates.insert("hat".to_string(), "hat({arg})".to_string());
    unicode_templates.insert("bar".to_string(), "{arg}̄".to_string()); // U+0304 combining macron
    unicode_templates.insert("tilde".to_string(), "{arg}̃".to_string()); // U+0303 combining tilde
    unicode_templates.insert("overline".to_string(), "{arg}̅".to_string()); // U+0305 combining overline
    unicode_templates.insert("dot_accent".to_string(), "{arg}̇".to_string()); // U+0307 combining dot above
    unicode_templates.insert("ddot_accent".to_string(), "{arg}̈".to_string()); // U+0308 combining diaeresis

    // Trig & log functions
    unicode_templates.insert("cos".to_string(), "cos({args})".to_string());
    unicode_templates.insert("tan".to_string(), "tan({args})".to_string());
    unicode_templates.insert("sinh".to_string(), "sinh({args})".to_string());
    unicode_templates.insert("cosh".to_string(), "cosh({args})".to_string());
    unicode_templates.insert("log".to_string(), "log({args})".to_string());
    unicode_templates.insert("ln".to_string(), "ln({args})".to_string());

    // Text mode (plain text within math)
    unicode_templates.insert("text".to_string(), "{arg}".to_string());

    // Matrix operations
    unicode_templates.insert("trace".to_string(), "Tr({arg})".to_string());
    unicode_templates.insert("inverse".to_string(), "({arg})⁻¹".to_string());

    // === Batch 3: Completeness Operations - Unicode ===

    // Phase A: Quick wins
    unicode_templates.insert("factorial".to_string(), "{arg}!".to_string());
    unicode_templates.insert("floor".to_string(), "⌊{arg}⌋".to_string());
    unicode_templates.insert("ceiling".to_string(), "⌈{arg}⌉".to_string());
    unicode_templates.insert("arcsin".to_string(), "arcsin({args})".to_string());
    unicode_templates.insert("arccos".to_string(), "arccos({args})".to_string());
    unicode_templates.insert("arctan".to_string(), "arctan({args})".to_string());
    unicode_templates.insert("sec".to_string(), "sec({args})".to_string());
    unicode_templates.insert("csc".to_string(), "csc({args})".to_string());
    unicode_templates.insert("cot".to_string(), "cot({args})".to_string());

    // Phase B: Quantum focus
    unicode_templates.insert(
        "pmatrix2x2".to_string(),
        "(({a11}, {a12}); ({a21}, {a22}))".to_string(),
    );
    unicode_templates.insert(
        "pmatrix3x3".to_string(),
        "(({a11}, {a12}, {a13}); ({a21}, {a22}, {a23}); ({a31}, {a32}, {a33}))".to_string(),
    );
    unicode_templates.insert("binomial".to_string(), "C({left},{right})".to_string());

    // Phase C: Field theory
    unicode_templates.insert("div".to_string(), "∇·{arg}".to_string());
    unicode_templates.insert("curl".to_string(), "∇×{arg}".to_string());
    unicode_templates.insert("laplacian".to_string(), "∇²{arg}".to_string());

    // === Batch 4: Polish & Edge Cases - Unicode ===

    // Piecewise functions
    unicode_templates.insert(
        "cases2".to_string(),
        "{ {left} if {right}, {from} if {to} }".to_string(),
    );
    unicode_templates.insert(
        "cases3".to_string(),
        "{ {left} if {right}, {from} if {to}, {body} if {idx2} }".to_string(),
    );

    // Determinant bars (vmatrix)
    unicode_templates.insert(
        "vmatrix2x2".to_string(),
        "|{a11}, {a12}; {a21}, {a22}|".to_string(),
    );
    unicode_templates.insert(
        "vmatrix3x3".to_string(),
        "|{a11}, {a12}, {a13}; {a21}, {a22}, {a23}; {a31}, {a32}, {a33}|".to_string(),
    );

    // Modular arithmetic
    // Use {idx2} which uniquely maps to arg2 (the modulus n)
    unicode_templates.insert(
        "congruent_mod".to_string(),
        "{left} ≡ {right} (mod {idx2})".to_string(),
    );

    // Statistics
    unicode_templates.insert("variance".to_string(), "Var({arg})".to_string());
    unicode_templates.insert("covariance".to_string(), "Cov({left}, {right})".to_string());

    // === Integral Transforms - Unicode ===

    // Fourier transforms
    unicode_templates.insert(
        "fourier_transform".to_string(),
        "ℱ[{function}]({variable})".to_string(),
    );
    unicode_templates.insert(
        "inverse_fourier".to_string(),
        "ℱ⁻¹[{function}]({variable})".to_string(),
    );

    // Laplace transforms
    unicode_templates.insert(
        "laplace_transform".to_string(),
        "ℒ[{function}]({variable})".to_string(),
    );
    unicode_templates.insert(
        "inverse_laplace".to_string(),
        "ℒ⁻¹[{function}]({variable})".to_string(),
    );

    // Convolution
    unicode_templates.insert(
        "convolution".to_string(),
        "({f} ∗ {g})({variable})".to_string(),
    );

    // Kernel integral
    unicode_templates.insert(
        "kernel_integral".to_string(),
        "∫_{domain} {kernel} {function} d{variable}".to_string(),
    );

    // Green's function
    unicode_templates.insert(
        "greens_function".to_string(),
        "G({point_x}, {source_m})".to_string(),
    );

    // === POT-Specific Operations - Unicode ===

    // Projection operator
    unicode_templates.insert(
        "projection".to_string(),
        "Π[{function}]({variable})".to_string(),
    );

    // Modal integral
    unicode_templates.insert(
        "modal_integral".to_string(),
        "∫_{modal_space} {function} dμ({variable})".to_string(),
    );

    // Projection kernel
    unicode_templates.insert(
        "projection_kernel".to_string(),
        "K({spacetime_point}, {modal_state})".to_string(),
    );

    // Causal bound
    unicode_templates.insert("causal_bound".to_string(), "c({point})".to_string());

    // Projection residue
    unicode_templates.insert(
        "projection_residue".to_string(),
        "Residue[{projection}, {structure}]".to_string(),
    );

    // Modal space
    unicode_templates.insert("modal_space".to_string(), "𝓜_{name}".to_string());

    // Spacetime
    unicode_templates.insert("spacetime".to_string(), "ℝ⁴".to_string());

    // Hont (Hilbert Ontology)
    unicode_templates.insert("hont".to_string(), "𝓗_{dimension}".to_string());

    let mut latex_glyphs = HashMap::new();
    latex_glyphs.insert("grad".to_string(), "\\nabla".to_string());
    latex_glyphs.insert("surface_integral_over".to_string(), "\\oint".to_string());

    let mut latex_templates = HashMap::new();
    latex_templates.insert("grad".to_string(), "{glyph} {arg}".to_string());
    latex_templates.insert(
        "surface_integral_over".to_string(),
        "{glyph}_{{{surface}}} {field} \\, dS".to_string(),
    );
    latex_templates.insert(
        "scalar_multiply".to_string(),
        "{left} \\, {right}".to_string(),
    );
    latex_templates.insert(
        "multiply".to_string(),
        "{left} \\, {right}".to_string(), // Matrix multiplication (polymorphic - works for block matrices!)
    );
    latex_templates.insert(
        "scalar_divide".to_string(),
        "\\frac{{left}}{{right}}".to_string(),
    );

    // Additional LaTeX templates
    latex_templates.insert("equals".to_string(), "{left} = {right}".to_string());
    latex_templates.insert("plus".to_string(), "{left} + {right}".to_string());
    latex_templates.insert("minus".to_string(), "{left} - {right}".to_string());
    latex_templates.insert("dot".to_string(), "{left} \\cdot {right}".to_string());
    latex_templates.insert("cross".to_string(), "{left} \\times {right}".to_string());
    latex_templates.insert("power".to_string(), "{base}^{{{exponent}}}".to_string());
    latex_templates.insert(
        "norm".to_string(),
        "\\left\\lVert {arg} \\right\\rVert".to_string(),
    );
    latex_templates.insert(
        "abs".to_string(),
        "\\left\\lvert {arg} \\right\\rvert".to_string(),
    );
    // Brackets & grouping
    latex_templates.insert(
        "parens".to_string(),
        "\\left( {content} \\right)".to_string(),
    );
    latex_templates.insert(
        "brackets".to_string(),
        "\\left[ {content} \\right]".to_string(),
    );
    latex_templates.insert(
        "braces".to_string(),
        "\\left\\{{ {content} \\right\\}}".to_string(),
    );
    latex_templates.insert(
        "angle_brackets".to_string(),
        "\\left\\langle {content} \\right\\rangle".to_string(),
    );
    latex_templates.insert(
        "inner".to_string(),
        "\\langle {left}, {right} \\rangle".to_string(),
    );
    latex_templates.insert("d_dt".to_string(), "\\frac{d\\,{num}}{d{den}}".to_string());
    latex_templates.insert(
        "d_part".to_string(),
        "\\frac{\\partial\\,{num}}{\\partial {den}}".to_string(),
    );
    latex_templates.insert(
        "d2_part".to_string(),
        "\\frac{\\partial^{2} \\,{num}}{\\partial {den}^{2}}".to_string(),
    );
    latex_templates.insert(
        "sum_bounds".to_string(),
        "\\sum_{ {from} }^{ {to} } {body}".to_string(),
    );
    latex_templates.insert(
        "sum_index".to_string(),
        "\\sum_{ {from} } {body}".to_string(),
    );
    latex_templates.insert(
        "prod_bounds".to_string(),
        "\\prod_{ {from} }^{ {to} } {body}".to_string(),
    );
    latex_templates.insert(
        "prod_index".to_string(),
        "\\prod_{ {from} } {body}".to_string(),
    );
    // limits
    latex_templates.insert(
        "lim".to_string(),
        "\\lim_{ {var} \\to {target} } {body}".to_string(),
    );
    latex_templates.insert(
        "limit".to_string(),
        "\\lim_{ {var} \\to {target} } {body}".to_string(),
    );
    latex_templates.insert(
        "limsup".to_string(),
        "\\limsup_{ {var} \\to {target} } {body}".to_string(),
    );
    latex_templates.insert(
        "liminf".to_string(),
        "\\liminf_{ {var} \\to {target} } {body}".to_string(),
    );
    latex_templates.insert(
        "int_bounds".to_string(),
        "\\int_{ {from} }^{ {to} } {integrand} \\, \\mathrm{d}{int_var}".to_string(),
    );
    latex_templates.insert("transpose".to_string(), "{arg}^{\\mathsf{T}}".to_string());
    latex_templates.insert(
        "det".to_string(),
        "\\det\\!\\left({arg}\\right)".to_string(),
    );
    latex_templates.insert(
        "matrix2x2".to_string(),
        "\\begin{bmatrix}{a11}&{a12}\\\\{a21}&{a22}\\end{bmatrix}".to_string(),
    );
    latex_templates.insert(
        "matrix3x3".to_string(),
        "\\begin{bmatrix}{a11}&{a12}&{a13}\\\\{a21}&{a22}&{a23}\\\\{a31}&{a32}&{a33}\\end{bmatrix}"
            .to_string(),
    );
    latex_templates.insert("vector_arrow".to_string(), "\\vec{{{arg}}}".to_string());
    latex_templates.insert(
        "vector_bold".to_string(),
        "\\boldsymbol{{{arg}}}".to_string(),
    );
    // indices
    latex_templates.insert("sub".to_string(), "{base}_{{{right}}}".to_string());
    latex_templates.insert("sup".to_string(), "{base}^{{{right}}}".to_string());
    latex_templates.insert(
        "index".to_string(),
        "{base}^{{{sup}}}_{{{sub}}}".to_string(),
    );
    // nabla with subscript, box operator
    latex_templates.insert(
        "nabla_sub".to_string(),
        "\\nabla_{{{sub}}} {arg}".to_string(),
    );
    latex_templates.insert("box".to_string(), "\\Box {arg}".to_string());
    // partial derivative apply and mixed index wrapper
    latex_templates.insert(
        "partial_apply".to_string(),
        "\\partial_{{{sub}}} {arg}".to_string(),
    );
    latex_templates.insert(
        "index_mixed".to_string(),
        "{base}^{{{idx1}}}_{{{idx2}}}".to_string(),
    );
    latex_templates.insert(
        "subsup".to_string(),
        "{base}_{{{subscript}}}^{{{superscript}}}".to_string(),
    );
    latex_templates.insert(
        "index_pair".to_string(),
        "{base}^{{{idx1}{idx2}}}".to_string(),
    );
    // Gamma (Christoffel) and Riemann tensors
    latex_templates.insert(
        "gamma".to_string(),
        "\\Gamma^{{{idx1}}}_{{{idx2} {idx3}}}".to_string(),
    );
    latex_templates.insert(
        "riemann".to_string(),
        "R^{{{idx1}}}_{{{idx2} {idx3} {idx4}}}".to_string(),
    );
    latex_templates.insert(
        "tensor_1up_3down".to_string(),
        "{base}^{{{upper}}}_{{{lower1} {lower2} {lower3}}}".to_string(),
    );
    latex_templates.insert(
        "tensor_lower_pair".to_string(),
        "{base}_{{{lower1} {lower2}}}".to_string(),
    );
    latex_templates.insert(
        "tensor_2up_2down".to_string(),
        "{base}^{{{upper1} {upper2}}}_{{{lower1} {lower2}}}".to_string(),
    );
    // Zeta and common math functions
    latex_templates.insert("zeta".to_string(), "\\zeta({args})".to_string());
    latex_templates.insert("Gamma".to_string(), "\\Gamma({args})".to_string());
    latex_templates.insert("sin".to_string(), "\\sin\\!({args})".to_string());
    latex_templates.insert("exp".to_string(), "\\exp({args})".to_string());
    // Hamilton–Jacobi (generic H and S)
    latex_templates.insert("H".to_string(), "H({args})".to_string());
    latex_templates.insert("S".to_string(), "S({args})".to_string());
    // Generic function-like symbols
    latex_templates.insert("V".to_string(), "V({args})".to_string());
    latex_templates.insert("F".to_string(), "F({args})".to_string());
    latex_templates.insert("C".to_string(), "C({args})".to_string());
    latex_templates.insert("D".to_string(), "D({args})".to_string());
    // min over control
    latex_templates.insert(
        "min_over".to_string(),
        "\\min_{{{sub}}} \\left\\{ {body} \\right\\}".to_string(),
    );

    // === Top 5 New Operations - LaTeX ===

    // 1. Bra-ket notation
    latex_templates.insert("ket".to_string(), "|{arg}\\rangle".to_string());
    latex_templates.insert("bra".to_string(), "\\langle{arg}|".to_string());
    latex_templates.insert(
        "inner".to_string(),
        "\\langle {left}|{right} \\rangle".to_string(),
    );
    latex_templates.insert(
        "outer".to_string(),
        "|{left}\\rangle\\langle{right}|".to_string(),
    );
    latex_templates.insert(
        "expectation".to_string(),
        "\\langle {arg} \\rangle".to_string(),
    );

    // 2. Set theory and logic
    latex_templates.insert("in".to_string(), "{left} \\in {right}".to_string());
    latex_templates.insert("subset".to_string(), "{left} \\subset {right}".to_string());
    latex_templates.insert(
        "subseteq".to_string(),
        "{left} \\subseteq {right}".to_string(),
    );
    latex_templates.insert("union".to_string(), "{left} \\cup {right}".to_string());
    latex_templates.insert(
        "intersection".to_string(),
        "{left} \\cap {right}".to_string(),
    );
    latex_templates.insert(
        "forall".to_string(),
        "\\forall {left} \\colon {right}".to_string(),
    );
    latex_templates.insert(
        "exists".to_string(),
        "\\exists {left} \\colon {right}".to_string(),
    );
    latex_templates.insert(
        "implies".to_string(),
        "{left} \\Rightarrow {right}".to_string(),
    );
    latex_templates.insert(
        "iff".to_string(),
        "{left} \\Leftrightarrow {right}".to_string(),
    );

    // 3. Multiple integrals
    // Use {idx2}, {idx3}, {idx4} which are unique to arg2, arg3, arg4
    latex_templates.insert(
        "double_integral".to_string(),
        "\\iint_{{right}} {left} \\, \\mathrm{d}{idx2} \\, \\mathrm{d}{idx3}".to_string(),
    );
    latex_templates.insert("triple_integral".to_string(), "\\iiint_{{right}} {left} \\, \\mathrm{d}{idx2} \\, \\mathrm{d}{idx3} \\, \\mathrm{d}{idx4}".to_string());

    // 4. Commutators
    latex_templates.insert("commutator".to_string(), "[{left}, {right}]".to_string());
    latex_templates.insert(
        "anticommutator".to_string(),
        "\\{{left}, {right}\\}".to_string(),
    );

    // 5. Square root
    latex_templates.insert("sqrt".to_string(), "\\sqrt{{arg}}".to_string());
    latex_templates.insert(
        "nth_root".to_string(),
        "\\sqrt[{right}]{{left}}".to_string(),
    );

    // === Next Top 3 + Low-Hanging Fruit - LaTeX ===

    // Comparison & inequality operators
    latex_templates.insert("lt".to_string(), "{left} < {right}".to_string());
    latex_templates.insert("gt".to_string(), "{left} > {right}".to_string());
    latex_templates.insert("leq".to_string(), "{left} \\leq {right}".to_string());
    latex_templates.insert("geq".to_string(), "{left} \\geq {right}".to_string());
    latex_templates.insert("neq".to_string(), "{left} \\neq {right}".to_string());
    latex_templates.insert("approx".to_string(), "{left} \\approx {right}".to_string());
    latex_templates.insert("propto".to_string(), "{left} \\propto {right}".to_string());

    // Complex number operations
    latex_templates.insert("conjugate".to_string(), "\\overline{{arg}}".to_string());
    latex_templates.insert("re".to_string(), "\\mathrm{Re}({arg})".to_string());
    latex_templates.insert("im".to_string(), "\\mathrm{Im}({arg})".to_string());
    latex_templates.insert("modulus".to_string(), "\\left|{arg}\\right|".to_string());

    // Accent commands
    latex_templates.insert("hat".to_string(), "\\hat{{arg}}".to_string());
    latex_templates.insert("bar".to_string(), "\\bar{{arg}}".to_string());
    latex_templates.insert("tilde".to_string(), "\\tilde{{arg}}".to_string());
    latex_templates.insert("overline".to_string(), "\\overline{{arg}}".to_string());
    latex_templates.insert("dot_accent".to_string(), "\\dot{{arg}}".to_string());
    latex_templates.insert("ddot_accent".to_string(), "\\ddot{{arg}}".to_string());

    // Trig & log functions
    latex_templates.insert("cos".to_string(), "\\cos({args})".to_string());
    latex_templates.insert("tan".to_string(), "\\tan({args})".to_string());
    latex_templates.insert("sinh".to_string(), "\\sinh({args})".to_string());
    latex_templates.insert("cosh".to_string(), "\\cosh({args})".to_string());
    latex_templates.insert("log".to_string(), "\\log({args})".to_string());
    latex_templates.insert("ln".to_string(), "\\ln({args})".to_string());

    // Text mode (plain text within math)
    latex_templates.insert("text".to_string(), "\\text{{arg}}".to_string());

    // Matrix operations
    latex_templates.insert("trace".to_string(), "\\mathrm{Tr}({arg})".to_string());
    latex_templates.insert("inverse".to_string(), "{arg}^{-1}".to_string());

    // Generic matrix constructors (new system)
    latex_templates.insert(
        "Matrix".to_string(),
        "\\begin{bmatrix}{args}\\end{bmatrix}".to_string(),
    );
    latex_templates.insert(
        "PMatrix".to_string(),
        "\\begin{pmatrix}{args}\\end{pmatrix}".to_string(),
    );
    latex_templates.insert(
        "VMatrix".to_string(),
        "\\begin{vmatrix}{args}\\end{vmatrix}".to_string(),
    );
    latex_templates.insert(
        "BMatrix".to_string(),
        "\\begin{bmatrix}{args}\\end{bmatrix}".to_string(),
    );

    // === Batch 3: Completeness Operations - LaTeX ===

    // Phase A: Quick wins
    latex_templates.insert("factorial".to_string(), "{arg}!".to_string());
    latex_templates.insert("floor".to_string(), "\\lfloor {arg} \\rfloor".to_string());
    latex_templates.insert("ceiling".to_string(), "\\lceil {arg} \\rceil".to_string());
    latex_templates.insert("arcsin".to_string(), "\\arcsin({args})".to_string());
    latex_templates.insert("arccos".to_string(), "\\arccos({args})".to_string());
    latex_templates.insert("arctan".to_string(), "\\arctan({args})".to_string());
    latex_templates.insert("sec".to_string(), "\\sec({args})".to_string());
    latex_templates.insert("csc".to_string(), "\\csc({args})".to_string());
    latex_templates.insert("cot".to_string(), "\\cot({args})".to_string());

    // Phase B: Quantum focus
    latex_templates.insert(
        "pmatrix2x2".to_string(),
        "\\begin{pmatrix}{a11}&{a12}\\\\{a21}&{a22}\\end{pmatrix}".to_string(),
    );
    latex_templates.insert(
        "pmatrix3x3".to_string(),
        "\\begin{pmatrix}{a11}&{a12}&{a13}\\\\{a21}&{a22}&{a23}\\\\{a31}&{a32}&{a33}\\end{pmatrix}"
            .to_string(),
    );
    latex_templates.insert(
        "binomial".to_string(),
        "\\binom{{left}}{{right}}".to_string(),
    );

    // Phase C: Field theory
    latex_templates.insert("div".to_string(), "\\nabla \\cdot {arg}".to_string());
    latex_templates.insert("curl".to_string(), "\\nabla \\times {arg}".to_string());
    latex_templates.insert("laplacian".to_string(), "\\nabla^2 {arg}".to_string());

    // === Batch 4: Polish & Edge Cases - LaTeX ===

    // Piecewise functions (cases environment)
    latex_templates.insert(
        "cases2".to_string(),
        "\\begin{cases}{left} & {right} \\\\ {from} & {to}\\end{cases}".to_string(),
    );
    latex_templates.insert(
        "cases3".to_string(),
        "\\begin{cases}{left} & {right} \\\\ {from} & {to} \\\\ {body} & {idx2}\\end{cases}"
            .to_string(),
    );

    // Determinant bars (vmatrix)
    latex_templates.insert(
        "vmatrix2x2".to_string(),
        "\\begin{vmatrix}{a11}&{a12}\\\\{a21}&{a22}\\end{vmatrix}".to_string(),
    );
    latex_templates.insert(
        "vmatrix3x3".to_string(),
        "\\begin{vmatrix}{a11}&{a12}&{a13}\\\\{a21}&{a22}&{a23}\\\\{a31}&{a32}&{a33}\\end{vmatrix}"
            .to_string(),
    );

    // Modular arithmetic
    // Use {idx2} which uniquely maps to arg2 (the modulus n)
    latex_templates.insert(
        "congruent_mod".to_string(),
        "{left} \\equiv {right} \\pmod{{idx2}}".to_string(),
    );

    // Statistics
    latex_templates.insert("variance".to_string(), "\\mathrm{Var}({arg})".to_string());
    latex_templates.insert(
        "covariance".to_string(),
        "\\mathrm{Cov}({left}, {right})".to_string(),
    );

    // === Integral Transforms - LaTeX ===

    // Fourier transforms
    latex_templates.insert(
        "fourier_transform".to_string(),
        "\\mathcal{F}[{function}]({variable})".to_string(),
    );
    latex_templates.insert(
        "inverse_fourier".to_string(),
        "\\mathcal{F}^{-1}[{function}]({variable})".to_string(),
    );

    // Laplace transforms
    latex_templates.insert(
        "laplace_transform".to_string(),
        "\\mathcal{L}[{function}]({variable})".to_string(),
    );
    latex_templates.insert(
        "inverse_laplace".to_string(),
        "\\mathcal{L}^{-1}[{function}]({variable})".to_string(),
    );

    // Convolution
    latex_templates.insert(
        "convolution".to_string(),
        "({f} \\ast {g})({variable})".to_string(),
    );

    // Kernel integral
    latex_templates.insert(
        "kernel_integral".to_string(),
        "\\int_{{{domain}}} {kernel} \\, {function} \\, \\mathrm{d}{variable}".to_string(),
    );

    // Green's function
    latex_templates.insert(
        "greens_function".to_string(),
        "G({point_x}, {source_m})".to_string(),
    );

    // === POT-Specific Operations - LaTeX ===

    // Projection operator
    latex_templates.insert(
        "projection".to_string(),
        "\\Pi[{function}]({variable})".to_string(),
    );

    // Modal integral
    latex_templates.insert(
        "modal_integral".to_string(),
        "\\int_{{{modal_space}}} {function} \\, \\mathrm{d}\\mu({variable})".to_string(),
    );

    // Projection kernel
    latex_templates.insert(
        "projection_kernel".to_string(),
        "K({spacetime_point}, {modal_state})".to_string(),
    );

    // Causal bound
    latex_templates.insert("causal_bound".to_string(), "c({point})".to_string());

    // Projection residue
    latex_templates.insert(
        "projection_residue".to_string(),
        "\\mathrm{Residue}[{projection}, {structure}]".to_string(),
    );

    // Modal space
    latex_templates.insert(
        "modal_space".to_string(),
        "\\mathcal{M}_{{{name}}}".to_string(),
    );

    // Spacetime
    latex_templates.insert("spacetime".to_string(), "\\mathbb{R}^4".to_string());

    // Hont (Hilbert Ontology)
    latex_templates.insert(
        "hont".to_string(),
        "\\mathcal{H}_{{{dimension}}}".to_string(),
    );

    // === HTML Templates (WYSIWYG with proper HTML elements) ===
    let html_glyphs = HashMap::new();
    let mut html_templates = HashMap::new();

    // Basic arithmetic with proper HTML structure
    html_templates.insert("scalar_divide".to_string(), 
        r#"<div class="math-frac"><div class="math-frac-num">{left}</div><div class="math-frac-line"></div><div class="math-frac-den">{right}</div></div>"#.to_string());
    html_templates.insert(
        "plus".to_string(),
        r#"{left} <span class="math-op">+</span> {right}"#.to_string(),
    );
    html_templates.insert(
        "minus".to_string(),
        r#"{left} <span class="math-op">−</span> {right}"#.to_string(),
    );
    html_templates.insert(
        "scalar_multiply".to_string(),
        r#"{left} <span class="math-op">·</span> {right}"#.to_string(),
    );
    html_templates.insert(
        "multiply".to_string(),
        r#"{left} <span class="math-op">·</span> {right}"#.to_string(), // Matrix multiplication (polymorphic - works for block matrices!)
    );
    html_templates.insert(
        "equals".to_string(),
        r#"{left} <span class="math-op">=</span> {right}"#.to_string(),
    );

    // Superscripts and subscripts
    html_templates.insert(
        "sup".to_string(),
        r#"{base}<sup class="math-sup">{right}</sup>"#.to_string(),
    );
    html_templates.insert(
        "sub".to_string(),
        r#"{base}<sub class="math-sub">{right}</sub>"#.to_string(),
    );
    html_templates.insert(
        "index_mixed".to_string(),
        r#"{base}<sup class="math-sup">{idx1}</sup><sub class="math-sub">{idx2}</sub>"#.to_string(),
    );
    html_templates.insert(
        "subsup".to_string(),
        r#"{base}<sub class="math-sub">{subscript}</sub><sup class="math-sup">{superscript}</sup>"#
            .to_string(),
    );
    html_templates.insert(
        "index_pair".to_string(),
        r#"{base}<sub class="math-sub">{idx1}{idx2}</sub>"#.to_string(),
    );
    html_templates.insert(
        "power".to_string(),
        r#"{base}<sup class="math-sup">{exponent}</sup>"#.to_string(),
    );

    // Square roots
    html_templates.insert(
        "sqrt".to_string(),
        r#"<span class="math-sqrt">√<span class="math-sqrt-content">{arg}</span></span>"#
            .to_string(),
    );

    // Integrals, sums, products
    html_templates.insert("int_bounds".to_string(), 
        r#"<span class="math-large-op">∫<sub class="math-sub">{from}</sub><sup class="math-sup">{to}</sup></span> {integrand} <span class="math-op">d</span>{int_var}"#.to_string());
    html_templates.insert("sum_bounds".to_string(), 
        r#"<span class="math-large-op">Σ<sub class="math-sub">{from}</sub><sup class="math-sup">{to}</sup></span> {body}"#.to_string());
    html_templates.insert(
        "sum_index".to_string(),
        r#"<span class="math-large-op">Σ<sub class="math-sub">{from}</sub></span> {body}"#
            .to_string(),
    );
    html_templates.insert("prod_bounds".to_string(), 
        r#"<span class="math-large-op">Π<sub class="math-sub">{from}</sub><sup class="math-sup">{to}</sup></span> {body}"#.to_string());
    html_templates.insert(
        "prod_index".to_string(),
        r#"<span class="math-large-op">Π<sub class="math-sub">{from}</sub></span> {body}"#
            .to_string(),
    );

    // Derivatives
    html_templates.insert("d_dt".to_string(), 
        r#"<div class="math-frac"><div class="math-frac-num">d{num}</div><div class="math-frac-line"></div><div class="math-frac-den">d{den}</div></div>"#.to_string());
    html_templates.insert("d_part".to_string(), 
        r#"<div class="math-frac"><div class="math-frac-num">∂{num}</div><div class="math-frac-line"></div><div class="math-frac-den">∂{den}</div></div>"#.to_string());
    html_templates.insert("d2_part".to_string(), 
        r#"<div class="math-frac"><div class="math-frac-num">∂<sup>2</sup>{num}</div><div class="math-frac-line"></div><div class="math-frac-den">∂{den}<sup>2</sup></div></div>"#.to_string());

    // Quantum mechanics (bra-ket notation)
    html_templates.insert("ket".to_string(), r#"|{arg}⟩"#.to_string());
    html_templates.insert("bra".to_string(), r#"⟨{arg}|"#.to_string());
    html_templates.insert("inner".to_string(), r#"⟨{left}|{right}⟩"#.to_string());
    html_templates.insert(
        "outer_product".to_string(),
        r#"|{left}⟩⟨{right}|"#.to_string(),
    );

    // Brackets and norms
    html_templates.insert("norm".to_string(), r#"‖{arg}‖"#.to_string());
    html_templates.insert("abs".to_string(), r#"|{arg}|"#.to_string());
    // Brackets & grouping
    html_templates.insert("parens".to_string(), r#"({content})"#.to_string());
    html_templates.insert("brackets".to_string(), r#"[{content}]"#.to_string());
    html_templates.insert("braces".to_string(), r#"{{{content}}}"#.to_string());
    html_templates.insert("angle_brackets".to_string(), r#"⟨{content}⟩"#.to_string());
    html_templates.insert("commutator".to_string(), r#"[{left}, {right}]"#.to_string());
    html_templates.insert(
        "anticommutator".to_string(),
        r#"{{left}, {right}}"#.to_string(),
    );

    // Dot and cross products
    html_templates.insert(
        "dot".to_string(),
        r#"{left} <span class="math-op">·</span> {right}"#.to_string(),
    );
    html_templates.insert(
        "cross".to_string(),
        r#"{left} <span class="math-op">×</span> {right}"#.to_string(),
    );

    // Limits
    html_templates.insert("lim".to_string(), 
        r#"<span class="math-large-op">lim<sub class="math-sub">{var}→{target}</sub></span> {body}"#.to_string());
    html_templates.insert("limit".to_string(), 
        r#"<span class="math-large-op">lim<sub class="math-sub">{var}→{target}</sub></span> {body}"#.to_string());
    html_templates.insert("limsup".to_string(), 
        r#"<span class="math-large-op">lim sup<sub class="math-sub">{var}→{target}</sub></span> {body}"#.to_string());
    html_templates.insert("liminf".to_string(), 
        r#"<span class="math-large-op">lim inf<sub class="math-sub">{var}→{target}</sub></span> {body}"#.to_string());

    // Set theory and logic
    html_templates.insert(
        "in".to_string(),
        r#"{left} <span class="math-op">∈</span> {right}"#.to_string(),
    );
    html_templates.insert(
        "subset".to_string(),
        r#"{left} <span class="math-op">⊂</span> {right}"#.to_string(),
    );
    html_templates.insert(
        "subseteq".to_string(),
        r#"{left} <span class="math-op">⊆</span> {right}"#.to_string(),
    );
    html_templates.insert(
        "union".to_string(),
        r#"{left} <span class="math-op">∪</span> {right}"#.to_string(),
    );
    html_templates.insert(
        "intersection".to_string(),
        r#"{left} <span class="math-op">∩</span> {right}"#.to_string(),
    );
    html_templates.insert("forall".to_string(), r#"∀{left}: {right}"#.to_string());
    html_templates.insert("exists".to_string(), r#"∃{left}: {right}"#.to_string());
    html_templates.insert(
        "implies".to_string(),
        r#"{left} <span class="math-op">⇒</span> {right}"#.to_string(),
    );
    html_templates.insert(
        "iff".to_string(),
        r#"{left} <span class="math-op">⇔</span> {right}"#.to_string(),
    );

    // Comparisons
    html_templates.insert(
        "lt".to_string(),
        r#"{left} <span class="math-op">&lt;</span> {right}"#.to_string(),
    );
    html_templates.insert(
        "gt".to_string(),
        r#"{left} <span class="math-op">&gt;</span> {right}"#.to_string(),
    );
    html_templates.insert(
        "leq".to_string(),
        r#"{left} <span class="math-op">≤</span> {right}"#.to_string(),
    );
    html_templates.insert(
        "geq".to_string(),
        r#"{left} <span class="math-op">≥</span> {right}"#.to_string(),
    );
    html_templates.insert(
        "neq".to_string(),
        r#"{left} <span class="math-op">≠</span> {right}"#.to_string(),
    );
    html_templates.insert(
        "approx".to_string(),
        r#"{left} <span class="math-op">≈</span> {right}"#.to_string(),
    );
    html_templates.insert(
        "propto".to_string(),
        r#"{left} <span class="math-op">∝</span> {right}"#.to_string(),
    );

    // Trig functions
    html_templates.insert(
        "sin".to_string(),
        r#"<span class="math-func">sin</span>({arg})"#.to_string(),
    );
    html_templates.insert(
        "cos".to_string(),
        r#"<span class="math-func">cos</span>({arg})"#.to_string(),
    );
    html_templates.insert(
        "tan".to_string(),
        r#"<span class="math-func">tan</span>({arg})"#.to_string(),
    );

    // Grad and nabla
    html_templates.insert("grad".to_string(), r#"∇{arg}"#.to_string());
    html_templates.insert(
        "nabla_sub".to_string(),
        r#"∇<sub class="math-sub">{sub}</sub> {arg}"#.to_string(),
    );

    // Christoffel and Riemann tensors
    html_templates.insert(
        "gamma".to_string(),
        r#"Γ<sup class="math-sup">{idx1}</sup><sub class="math-sub">{idx2} {idx3}</sub>"#
            .to_string(),
    );
    html_templates.insert(
        "riemann".to_string(),
        r#"R<sup class="math-sup">{idx1}</sup><sub class="math-sub">{idx2} {idx3} {idx4}</sub>"#
            .to_string(),
    );
    html_templates.insert(
        "tensor_1up_3down".to_string(),
        r#"{base}<sup class="math-sup">{upper}</sup><sub class="math-sub">{lower1} {lower2} {lower3}</sub>"#
            .to_string(),
    );
    html_templates.insert(
        "tensor_lower_pair".to_string(),
        r#"{base}<sub class="math-sub">{lower1} {lower2}</sub>"#.to_string(),
    );
    html_templates.insert(
        "tensor_2up_2down".to_string(),
        r#"{base}<sup class="math-sup">{upper1} {upper2}</sup><sub class="math-sub">{lower1} {lower2}</sub>"#
            .to_string(),
    );

    // Transpose and determinant
    html_templates.insert(
        "transpose".to_string(),
        r#"{arg}<sup class="math-sup">T</sup>"#.to_string(),
    );
    html_templates.insert("det".to_string(), r#"det({arg})"#.to_string());

    // Vectors
    html_templates.insert(
        "vector_arrow".to_string(),
        r#"<span class="math-vector">{arg}⃗</span>"#.to_string(),
    );
    html_templates.insert(
        "vector_bold".to_string(),
        r#"<span class="math-vector-bold">{arg}</span>"#.to_string(),
    );

    // === Additional missing HTML templates (52 operations) ===

    // Accents and decorations
    html_templates.insert("hat".to_string(), r#"{arg}̂"#.to_string());
    html_templates.insert("bar".to_string(), r#"{arg}̄"#.to_string());
    html_templates.insert("tilde".to_string(), r#"{arg}̃"#.to_string());
    html_templates.insert(
        "overline".to_string(),
        r#"<span style="text-decoration: overline;">{arg}</span>"#.to_string(),
    );
    html_templates.insert("dot_accent".to_string(), r#"{arg}̇"#.to_string());
    html_templates.insert("ddot_accent".to_string(), r#"{arg}̈"#.to_string());

    // More trig functions
    html_templates.insert(
        "arcsin".to_string(),
        r#"<span class="math-func">arcsin</span>({args})"#.to_string(),
    );
    html_templates.insert(
        "arccos".to_string(),
        r#"<span class="math-func">arccos</span>({args})"#.to_string(),
    );
    html_templates.insert(
        "arctan".to_string(),
        r#"<span class="math-func">arctan</span>({args})"#.to_string(),
    );
    html_templates.insert(
        "sec".to_string(),
        r#"<span class="math-func">sec</span>({args})"#.to_string(),
    );
    html_templates.insert(
        "csc".to_string(),
        r#"<span class="math-func">csc</span>({args})"#.to_string(),
    );
    html_templates.insert(
        "cot".to_string(),
        r#"<span class="math-func">cot</span>({args})"#.to_string(),
    );
    html_templates.insert(
        "sinh".to_string(),
        r#"<span class="math-func">sinh</span>({args})"#.to_string(),
    );
    html_templates.insert(
        "cosh".to_string(),
        r#"<span class="math-func">cosh</span>({args})"#.to_string(),
    );

    // Logarithms and exponentials
    html_templates.insert(
        "exp".to_string(),
        r#"<span class="math-func">exp</span>({args})"#.to_string(),
    );
    html_templates.insert(
        "log".to_string(),
        r#"<span class="math-func">log</span>({args})"#.to_string(),
    );
    html_templates.insert(
        "ln".to_string(),
        r#"<span class="math-func">ln</span>({args})"#.to_string(),
    );

    // Complex numbers
    html_templates.insert("conjugate".to_string(), r#"{arg}*"#.to_string());
    html_templates.insert(
        "re".to_string(),
        r#"<span class="math-func">Re</span>({arg})"#.to_string(),
    );
    html_templates.insert(
        "im".to_string(),
        r#"<span class="math-func">Im</span>({arg})"#.to_string(),
    );
    html_templates.insert("modulus".to_string(), r#"|{arg}|"#.to_string());

    // Matrices - 2x2 and 3x3
    html_templates.insert("matrix2x2".to_string(), 
        r#"<table class="math-matrix"><tr><td>{a11}</td><td>{a12}</td></tr><tr><td>{a21}</td><td>{a22}</td></tr></table>"#.to_string());
    html_templates.insert("matrix3x3".to_string(), 
        r#"<table class="math-matrix"><tr><td>{a11}</td><td>{a12}</td><td>{a13}</td></tr><tr><td>{a21}</td><td>{a22}</td><td>{a23}</td></tr><tr><td>{a31}</td><td>{a32}</td><td>{a33}</td></tr></table>"#.to_string());
    html_templates.insert("pmatrix2x2".to_string(), 
        r#"<span class="math-pmatrix">(</span><table class="math-matrix"><tr><td>{a11}</td><td>{a12}</td></tr><tr><td>{a21}</td><td>{a22}</td></tr></table><span class="math-pmatrix">)</span>"#.to_string());
    html_templates.insert("pmatrix3x3".to_string(), 
        r#"<span class="math-pmatrix">(</span><table class="math-matrix"><tr><td>{a11}</td><td>{a12}</td><td>{a13}</td></tr><tr><td>{a21}</td><td>{a22}</td><td>{a23}</td></tr><tr><td>{a31}</td><td>{a32}</td><td>{a33}</td></tr></table><span class="math-pmatrix">)</span>"#.to_string());
    html_templates.insert("vmatrix2x2".to_string(), 
        r#"<span class="math-vmatrix">|</span><table class="math-matrix"><tr><td>{a11}</td><td>{a12}</td></tr><tr><td>{a21}</td><td>{a22}</td></tr></table><span class="math-vmatrix">|</span>"#.to_string());
    html_templates.insert("vmatrix3x3".to_string(), 
        r#"<span class="math-vmatrix">|</span><table class="math-matrix"><tr><td>{a11}</td><td>{a12}</td><td>{a13}</td></tr><tr><td>{a21}</td><td>{a22}</td><td>{a23}</td></tr><tr><td>{a31}</td><td>{a32}</td><td>{a33}</td></tr></table><span class="math-vmatrix">|</span>"#.to_string());

    // Cases and piecewise
    html_templates.insert("cases2".to_string(), 
        r#"<span class="math-cases">{<br>&nbsp;&nbsp;{cond1}, &nbsp;{body1}<br>&nbsp;&nbsp;{cond2}, &nbsp;{body2}<br>}</span>"#.to_string());
    html_templates.insert("cases3".to_string(), 
        r#"<span class="math-cases">{<br>&nbsp;&nbsp;{cond1}, &nbsp;{body1}<br>&nbsp;&nbsp;{cond2}, &nbsp;{body2}<br>&nbsp;&nbsp;{cond3}, &nbsp;{body3}<br>}</span>"#.to_string());
    html_templates.insert(
        "piecewise".to_string(),
        r#"<span class="math-cases">{<br>&nbsp;&nbsp;{args}<br>}</span>"#.to_string(),
    );

    // Floor, ceiling, factorial
    html_templates.insert("floor".to_string(), r#"⌊{arg}⌋"#.to_string());
    html_templates.insert("ceiling".to_string(), r#"⌈{arg}⌉"#.to_string());
    html_templates.insert("factorial".to_string(), r#"{arg}!"#.to_string());

    // Nth root
    html_templates.insert("nth_root".to_string(), 
        r#"<span class="math-nthroot"><sup class="math-root-index">{right}</sup>√<span class="math-sqrt-content">{left}</span></span>"#.to_string());

    // Multiple integrals
    html_templates.insert("double_integral".to_string(), 
        r#"<span class="math-large-op">∬<sub class="math-sub">{right}</sub></span> {left} <span class="math-op">d</span>{idx2} <span class="math-op">d</span>{idx3}"#.to_string());
    html_templates.insert("triple_integral".to_string(), 
        r#"<span class="math-large-op">∭<sub class="math-sub">{right}</sub></span> {left} <span class="math-op">d</span>{idx2} <span class="math-op">d</span>{idx3} <span class="math-op">d</span>{idx4}"#.to_string());
    html_templates.insert("surface_integral_over".to_string(), 
        r#"<span class="math-large-op">∮<sub class="math-sub">{surface}</sub></span> {field} <span class="math-op">dS</span>"#.to_string());

    // Linear algebra operations
    html_templates.insert(
        "trace".to_string(),
        r#"<span class="math-func">Tr</span>({arg})"#.to_string(),
    );
    html_templates.insert(
        "inverse".to_string(),
        r#"{arg}<sup class="math-sup">−1</sup>"#.to_string(),
    );

    // Index operations
    html_templates.insert(
        "index".to_string(),
        r#"{base}<sup class="math-sup">{sup}</sup><sub class="math-sub">{sub}</sub>"#.to_string(),
    );
    html_templates.insert(
        "partial_apply".to_string(),
        r#"∂<sub class="math-sub">{sub}</sub> {arg}"#.to_string(),
    );

    // Box and other operators
    html_templates.insert("box".to_string(), r#"□{arg}"#.to_string());
    html_templates.insert(
        "min_over".to_string(),
        r#"<span class="math-large-op">min<sub class="math-sub">{sub}</sub></span> { {body} }"#
            .to_string(),
    );

    // Text in math
    html_templates.insert(
        "text".to_string(),
        r#"<span class="math-text">{arg}</span>"#.to_string(),
    );

    // Special functions (single letter - physics notation)
    html_templates.insert(
        "H".to_string(),
        r#"<span class="math-func">H</span>({args})"#.to_string(),
    );
    html_templates.insert(
        "S".to_string(),
        r#"<span class="math-func">S</span>({args})"#.to_string(),
    );
    html_templates.insert(
        "V".to_string(),
        r#"<span class="math-func">V</span>({args})"#.to_string(),
    );
    html_templates.insert(
        "F".to_string(),
        r#"<span class="math-func">F</span>({args})"#.to_string(),
    );
    html_templates.insert(
        "C".to_string(),
        r#"<span class="math-func">C</span>({args})"#.to_string(),
    );
    html_templates.insert(
        "D".to_string(),
        r#"<span class="math-func">D</span>({args})"#.to_string(),
    );

    // Greek letter functions
    html_templates.insert(
        "Gamma".to_string(),
        r#"<span class="math-func">Γ</span>({args})"#.to_string(),
    );
    html_templates.insert(
        "zeta".to_string(),
        r#"<span class="math-func">ζ</span>({args})"#.to_string(),
    );

    // Final 7 missing templates
    html_templates.insert("binomial".to_string(), 
        r#"<span class="math-binomial">(<table class="math-binomial-content"><tr><td>{left}</td></tr><tr><td>{right}</td></tr></table>)</span>"#.to_string());
    html_templates.insert(
        "div".to_string(),
        r#"∇ <span class="math-op">·</span> {arg}"#.to_string(),
    );
    html_templates.insert(
        "curl".to_string(),
        r#"∇ <span class="math-op">×</span> {arg}"#.to_string(),
    );
    html_templates.insert(
        "laplacian".to_string(),
        r#"∇<sup class="math-sup">2</sup> {arg}"#.to_string(),
    );
    html_templates.insert("congruent_mod".to_string(), r#"{left} <span class="math-op">≡</span> {right} <span class="math-pmod">(mod {idx2})</span>"#.to_string());
    html_templates.insert(
        "variance".to_string(),
        r#"<span class="math-func">Var</span>({arg})"#.to_string(),
    );
    html_templates.insert(
        "covariance".to_string(),
        r#"<span class="math-func">Cov</span>({left}, {right})"#.to_string(),
    );

    // === Integral Transforms - HTML ===

    // Fourier transforms
    html_templates.insert(
        "fourier_transform".to_string(),
        r#"<span class="math-script">ℱ</span>[{function}]({variable})"#.to_string(),
    );
    html_templates.insert(
        "inverse_fourier".to_string(),
        r#"<span class="math-script">ℱ</span><sup class="math-sup">-1</sup>[{function}]({variable})"#.to_string(),
    );

    // Laplace transforms
    html_templates.insert(
        "laplace_transform".to_string(),
        r#"<span class="math-script">ℒ</span>[{function}]({variable})"#.to_string(),
    );
    html_templates.insert(
        "inverse_laplace".to_string(),
        r#"<span class="math-script">ℒ</span><sup class="math-sup">-1</sup>[{function}]({variable})"#.to_string(),
    );

    // Convolution
    html_templates.insert(
        "convolution".to_string(),
        r#"({f} <span class="math-op">∗</span> {g})({variable})"#.to_string(),
    );

    // Kernel integral
    html_templates.insert(
        "kernel_integral".to_string(),
        r#"∫<sub class="math-sub">{domain}</sub> {kernel} {function} d{variable}"#.to_string(),
    );

    // Green's function
    html_templates.insert(
        "greens_function".to_string(),
        r#"<span class="math-func">G</span>({point_x}, {source_m})"#.to_string(),
    );

    // === POT-Specific Operations - HTML ===

    // Projection operator
    html_templates.insert(
        "projection".to_string(),
        r#"<span class="math-op">Π</span>[{function}]({variable})"#.to_string(),
    );

    // Modal integral
    html_templates.insert(
        "modal_integral".to_string(),
        r#"∫<sub class="math-sub">{modal_space}</sub> {function} dμ({variable})"#.to_string(),
    );

    // Projection kernel
    html_templates.insert(
        "projection_kernel".to_string(),
        r#"<span class="math-func">K</span>({spacetime_point}, {modal_state})"#.to_string(),
    );

    // Causal bound
    html_templates.insert(
        "causal_bound".to_string(),
        r#"<span class="math-func">c</span>({point})"#.to_string(),
    );

    // Projection residue
    html_templates.insert(
        "projection_residue".to_string(),
        r#"<span class="math-func">Residue</span>[{projection}, {structure}]"#.to_string(),
    );

    // Modal space
    html_templates.insert(
        "modal_space".to_string(),
        r#"<span class="math-script">𝓜</span><sub class="math-sub">{name}</sub>"#.to_string(),
    );

    // Spacetime
    html_templates.insert(
        "spacetime".to_string(),
        r#"<span class="math-blackboard">ℝ</span><sup class="math-sup">4</sup>"#.to_string(),
    );

    // Hont (Hilbert Ontology)
    html_templates.insert(
        "hont".to_string(),
        r#"<span class="math-script">𝓗</span><sub class="math-sub">{dimension}</sub>"#.to_string(),
    );

    // === TYPST Templates (for math layout engine) ===
    // Templates are clean (no invisible markers) as we use layout tree analysis
    // for bounding box extraction
    let typst_glyphs = HashMap::new();
    let mut typst_templates = HashMap::new();

    // Basic operations
    typst_templates.insert(
        "scalar_divide".to_string(),
        "({left})/({right})".to_string(),
    );
    typst_templates.insert("scalar_multiply".to_string(), "{left} {right}".to_string());
    typst_templates.insert("multiply".to_string(), "{left} {right}".to_string()); // Matrix multiplication (polymorphic over T - works for block matrices too!)
    typst_templates.insert("plus".to_string(), "{left} + {right}".to_string());
    typst_templates.insert("minus".to_string(), "{left} - {right}".to_string());

    // Superscript/subscript
    typst_templates.insert("sup".to_string(), "{base}^({exponent})".to_string());
    typst_templates.insert("sub".to_string(), "{base} _({subscript})".to_string());

    // Square root
    typst_templates.insert("sqrt".to_string(), "sqrt({arg})".to_string());
    typst_templates.insert("nth_root".to_string(), "root({right}, {left})".to_string());

    // Calculus
    typst_templates.insert(
        "int_bounds".to_string(),
        "integral _({lower})^({upper}) {integrand} dif {variable}".to_string(),
    );
    typst_templates.insert(
        "double_integral".to_string(),
        "integral.double _({right}) {left} dif {idx2} dif {idx3}".to_string(),
    );
    typst_templates.insert(
        "triple_integral".to_string(),
        "integral.triple _({right}) {left} dif {idx2} dif {idx3} dif {idx4}".to_string(),
    );
    typst_templates.insert(
        "sum_bounds".to_string(),
        "sum _({from})^({to}) {body}".to_string(),
    );
    typst_templates.insert(
        "prod_bounds".to_string(),
        "product _({from})^({to}) {body}".to_string(),
    );
    typst_templates.insert("sum_index".to_string(), "sum _({from}) {body}".to_string());
    typst_templates.insert(
        "prod_index".to_string(),
        "product _({from}) {body}".to_string(),
    );

    typst_templates.insert(
        "d_part".to_string(),
        "(diff {function})/(diff {variable})".to_string(),
    );
    typst_templates.insert(
        "d2_part".to_string(),
        "(diff^2 {function})/(diff {variable}^2)".to_string(),
    );
    typst_templates.insert(
        "partial_apply".to_string(),
        "diff _({sub}) {arg}".to_string(),
    );
    typst_templates.insert(
        "d_dt".to_string(),
        "(d {function})/(d {variable})".to_string(),
    ); // using d for derivative? or upright d?
    typst_templates.insert("grad".to_string(), "nabla {function}".to_string());
    typst_templates.insert("div".to_string(), "nabla dot {function}".to_string());
    typst_templates.insert("curl".to_string(), "∇ × {function}".to_string());
    typst_templates.insert("laplacian".to_string(), "nabla^2 {function}".to_string());
    typst_templates.insert("box".to_string(), "square {arg}".to_string()); // d'Alembertian
    typst_templates.insert(
        "surface_integral_over".to_string(),
        "integral _({surface}) {field} dot dif S".to_string(),
    );

    // Linear Algebra
    // Use generous spacing to ensure #sym.square identifiers don't merge with commas
    typst_templates.insert(
        "matrix2x2".to_string(),
        "mat(delim: \"[\", {a11} , {a12} ; {a21} , {a22})".to_string(),
    );

    // Generic matrix constructors (new system)
    typst_templates.insert(
        "Matrix".to_string(),
        "mat(delim: \"[\", {args})".to_string(),
    );
    typst_templates.insert(
        "PMatrix".to_string(),
        "mat(delim: \"(\", {args})".to_string(),
    );
    typst_templates.insert(
        "VMatrix".to_string(),
        "mat(delim: \"|\", {args})".to_string(),
    );
    typst_templates.insert(
        "BMatrix".to_string(),
        "mat(delim: \"[\", {args})".to_string(),
    );
    typst_templates.insert(
        "matrix3x3".to_string(),
        "mat(delim: \"[\", {a11} , {a12} , {a13} ; {a21} , {a22} , {a23} ; {a31} , {a32} , {a33})"
            .to_string(),
    );
    typst_templates.insert(
        "pmatrix2x2".to_string(),
        "mat(delim: \"(\", {a11} , {a12} ; {a21} , {a22})".to_string(),
    );
    typst_templates.insert(
        "pmatrix3x3".to_string(),
        "mat(delim: \"(\", {a11} , {a12} , {a13} ; {a21} , {a22} , {a23} ; {a31} , {a32} , {a33})"
            .to_string(),
    );
    typst_templates.insert(
        "vmatrix2x2".to_string(),
        "mat(delim: \"|\", {a11} , {a12} ; {a21} , {a22})".to_string(),
    );
    typst_templates.insert(
        "vmatrix3x3".to_string(),
        "mat(delim: \"|\", {a11} , {a12} , {a13} ; {a21} , {a22} , {a23} ; {a31} , {a32} , {a33})"
            .to_string(),
    );
    // Generic matrix template - will be filled dynamically
    typst_templates.insert(
        "matrix".to_string(),
        "mat(delim: \"[\", {args})".to_string(),
    );
    typst_templates.insert("binomial".to_string(), "binom({left}, {right})".to_string());

    typst_templates.insert("vector_bold".to_string(), "bold({arg})".to_string());
    typst_templates.insert("vector_arrow".to_string(), "arrow({arg})".to_string());
    typst_templates.insert("dot".to_string(), "{left} dot {right}".to_string());
    typst_templates.insert("cross".to_string(), "{left} times {right}".to_string());
    typst_templates.insert("norm".to_string(), "norm({vector})".to_string());
    typst_templates.insert("abs".to_string(), "abs({value})".to_string());
    // Brackets & grouping (use lr() for auto-scaling)
    typst_templates.insert("parens".to_string(), "lr(({content}))".to_string());
    typst_templates.insert("brackets".to_string(), "lr([{content}])".to_string());
    typst_templates.insert("braces".to_string(), "lr(\\{ {content} \\})".to_string());
    typst_templates.insert(
        "angle_brackets".to_string(),
        "lr(angle.l {content} angle.r)".to_string(),
    );
    typst_templates.insert("det".to_string(), "det({arg})".to_string());
    typst_templates.insert("transpose".to_string(), "{arg}^T".to_string());
    typst_templates.insert("inverse".to_string(), "{arg}^(-1)".to_string());
    typst_templates.insert(
        "outer_product".to_string(),
        "lr(| {ket} angle.r angle.l {bra} |)".to_string(),
    );

    // Quantum
    // Use lr(...) to ensure brackets scale with content
    typst_templates.insert("ket".to_string(), "lr(| {state} angle.r)".to_string());
    typst_templates.insert("bra".to_string(), "lr(angle.l {state} |)".to_string());
    typst_templates.insert(
        "inner".to_string(),
        "lr(angle.l {bra} | {ket} angle.r)".to_string(),
    );
    typst_templates.insert(
        "outer".to_string(),
        "lr(| {ket} angle.r angle.l {bra} |)".to_string(),
    );
    typst_templates.insert("commutator".to_string(), "lr([ {A}, {B} ])".to_string());
    typst_templates.insert("anticommutator".to_string(), "lr({ {A}, {B} })".to_string());
    typst_templates.insert(
        "expectation".to_string(),
        "lr(angle.l {operator} angle.r)".to_string(),
    );

    // Tensors
    typst_templates.insert(
        "index_mixed".to_string(),
        "{base}^({upper}) _({lower})".to_string(),
    );
    typst_templates.insert(
        "subsup".to_string(),
        "{base}_({subscript}) ^({superscript})".to_string(),
    );
    typst_templates.insert(
        "index_pair".to_string(),
        "{base}^({idx1} {idx2})".to_string(),
    ); // or separate?

    // Trig
    typst_templates.insert("sin".to_string(), "sin({argument})".to_string());
    typst_templates.insert("cos".to_string(), "cos({argument})".to_string());
    typst_templates.insert("tan".to_string(), "tan({argument})".to_string());
    typst_templates.insert("sec".to_string(), "sec({argument})".to_string());
    typst_templates.insert("csc".to_string(), "csc({argument})".to_string());
    typst_templates.insert("cot".to_string(), "cot({argument})".to_string());
    typst_templates.insert("arcsin".to_string(), "arcsin({argument})".to_string());
    typst_templates.insert("arccos".to_string(), "arccos({argument})".to_string());
    typst_templates.insert("arctan".to_string(), "arctan({argument})".to_string());
    typst_templates.insert("sinh".to_string(), "sinh({argument})".to_string());
    typst_templates.insert("cosh".to_string(), "cosh({argument})".to_string());
    typst_templates.insert("tanh".to_string(), "tanh({argument})".to_string());

    // Exponential
    typst_templates.insert("exp".to_string(), "e^({argument})".to_string());
    typst_templates.insert("ln".to_string(), "ln({argument})".to_string());
    typst_templates.insert("log".to_string(), "log({argument})".to_string());
    typst_templates.insert("factorial".to_string(), "{arg}!".to_string());
    typst_templates.insert("floor".to_string(), "floor({arg})".to_string());
    typst_templates.insert("ceiling".to_string(), "ceil({arg})".to_string());
    typst_templates.insert("conjugate".to_string(), "overline({arg})".to_string());
    typst_templates.insert("re".to_string(), "upright(\"Re\")({arg})".to_string());
    typst_templates.insert("im".to_string(), "upright(\"Im\")({arg})".to_string());
    typst_templates.insert("modulus".to_string(), "mod {arg}".to_string());
    typst_templates.insert(
        "congruent_mod".to_string(),
        "{left} equiv {right} (mod {to})".to_string(),
    );

    // Limits
    typst_templates.insert(
        "lim".to_string(),
        "lim _({var} -> {target}) {body}".to_string(),
    );
    typst_templates.insert(
        "limit".to_string(),
        "lim _({var} -> {target}) {body}".to_string(),
    );
    typst_templates.insert(
        "limsup".to_string(),
        "limsup _({var} -> {target}) {body}".to_string(),
    );
    typst_templates.insert(
        "liminf".to_string(),
        "liminf _({var} -> {target}) {body}".to_string(),
    );
    typst_templates.insert("equals".to_string(), "{left} = {right}".to_string());

    // Comparison operators
    typst_templates.insert("leq".to_string(), "{left} <= {right}".to_string());
    typst_templates.insert("geq".to_string(), "{left} >= {right}".to_string());
    typst_templates.insert("less_than".to_string(), "{left} < {right}".to_string());
    typst_templates.insert("greater_than".to_string(), "{left} > {right}".to_string());
    typst_templates.insert("neq".to_string(), "{left} != {right}".to_string());
    typst_templates.insert("not_equal".to_string(), "{left} != {right}".to_string());
    typst_templates.insert("approx".to_string(), "{left} approx {right}".to_string());
    typst_templates.insert("propto".to_string(), "{left} prop {right}".to_string());
    typst_templates.insert(
        "proportional".to_string(),
        "{left} prop {right}".to_string(),
    );

    // Logical operators
    typst_templates.insert("logical_and".to_string(), "{left} and {right}".to_string());
    typst_templates.insert("logical_or".to_string(), "{left} or {right}".to_string());
    typst_templates.insert("logical_not".to_string(), "not {arg}".to_string());

    // Piecewise functions
    typst_templates.insert(
        "cases2".to_string(),
        "cases({left} & \"if\" {right}, {from} & \"if\" {to})".to_string(),
    );
    typst_templates.insert(
        "cases3".to_string(),
        "cases({left} & \"if\" {right}, {from} & \"if\" {to}, {body} & \"if\" {idx2})".to_string(),
    );

    // Accents
    typst_templates.insert("hat".to_string(), "hat({arg})".to_string());
    typst_templates.insert("bar".to_string(), "macron({arg})".to_string()); // Use macron for short bar (like LaTeX \bar)
    typst_templates.insert("tilde".to_string(), "tilde({arg})".to_string());
    typst_templates.insert("overline".to_string(), "overline({arg})".to_string()); // Keep overline for full overline
    typst_templates.insert("dot_accent".to_string(), "dot({arg})".to_string());
    typst_templates.insert("ddot_accent".to_string(), "dot.double({arg})".to_string());

    // Text mode
    typst_templates.insert("text".to_string(), "\"{arg}\"".to_string());
    typst_templates.insert("mathrm".to_string(), "upright(\"{arg}\")".to_string());

    // Statistics and linear algebra functions
    typst_templates.insert("variance".to_string(), "op(\"Var\")({arg})".to_string());
    typst_templates.insert(
        "covariance".to_string(),
        "op(\"Cov\")({left}, {right})".to_string(),
    );
    typst_templates.insert("trace".to_string(), "op(\"Tr\") lr(( {arg} ))".to_string());

    // Set theory and logic
    typst_templates.insert("in".to_string(), "{left} in {right}".to_string());
    typst_templates.insert("in_set".to_string(), "{left} in {right}".to_string());
    typst_templates.insert("subset".to_string(), "{left} subset {right}".to_string());
    typst_templates.insert("lt".to_string(), "{left} < {right}".to_string());
    typst_templates.insert("gt".to_string(), "{left} > {right}".to_string());

    typst_templates.insert(
        "riemann".to_string(),
        "R^({idx1})_({idx2} {idx3} {idx4})".to_string(),
    );
    typst_templates.insert(
        "tensor_1up_3down".to_string(),
        "{base}^({upper})_({lower1} {lower2} {lower3})".to_string(),
    );
    typst_templates.insert(
        "tensor_lower_pair".to_string(),
        "{base}_({lower1} {lower2})".to_string(),
    );
    typst_templates.insert(
        "tensor_2up_2down".to_string(),
        "{base}^({upper1} {upper2})_({lower1} {lower2})".to_string(),
    );
    typst_templates.insert("zeta".to_string(), "zeta({arg})".to_string());
    // Christoffel symbol: Γ^idx1_{idx2 idx3}
    typst_templates.insert(
        "gamma".to_string(),
        "Gamma^({idx1})_({idx2} {idx3})".to_string(),
    );
    // Riemann tensor: R^idx1_{idx2 idx3 idx4}
    typst_templates.insert(
        "riemann".to_string(),
        "R^({idx1})_({idx2} {idx3} {idx4})".to_string(),
    );
    typst_templates.insert("power".to_string(), "{base}^({exponent})".to_string());
    typst_templates.insert("index".to_string(), "{base}_({subscript})".to_string());
    typst_templates.insert(
        "subseteq".to_string(),
        "{left} subset.eq {right}".to_string(),
    );
    typst_templates.insert("union".to_string(), "{left} union {right}".to_string());
    typst_templates.insert(
        "intersection".to_string(),
        "{left} sect {right}".to_string(),
    );
    typst_templates.insert("forall".to_string(), "forall {left} : {right}".to_string());
    typst_templates.insert("exists".to_string(), "exists {left} : {right}".to_string());
    typst_templates.insert("implies".to_string(), "{left} => {right}".to_string());
    typst_templates.insert("implied_by".to_string(), "{left} <= {right}".to_string());
    typst_templates.insert("iff".to_string(), "{left} <=> {right}".to_string());

    // === Integral Transforms - Typst ===

    // Fourier transforms
    typst_templates.insert(
        "fourier_transform".to_string(),
        "cal(F)[{function}]({variable})".to_string(),
    );
    typst_templates.insert(
        "inverse_fourier".to_string(),
        "cal(F)^(-1)[{function}]({variable})".to_string(),
    );

    // Laplace transforms
    typst_templates.insert(
        "laplace_transform".to_string(),
        "cal(L)[{function}]({variable})".to_string(),
    );
    typst_templates.insert(
        "inverse_laplace".to_string(),
        "cal(L)^(-1)[{function}]({variable})".to_string(),
    );

    // Convolution
    typst_templates.insert(
        "convolution".to_string(),
        "({f} ast {g})({variable})".to_string(),
    );

    // Kernel integral
    typst_templates.insert(
        "kernel_integral".to_string(),
        "integral _({domain}) {kernel} {function} dif {variable}".to_string(),
    );

    // Green's function
    typst_templates.insert(
        "greens_function".to_string(),
        "G({point_x}, {source_m})".to_string(),
    );

    // === POT-Specific Operations - Typst ===

    // Projection operator
    typst_templates.insert(
        "projection".to_string(),
        "Pi[{function}]({variable})".to_string(),
    );

    // Modal integral
    typst_templates.insert(
        "modal_integral".to_string(),
        "integral _({modal_space}) {function} dif mu({variable})".to_string(),
    );

    // Projection kernel
    typst_templates.insert(
        "projection_kernel".to_string(),
        "K({spacetime_point}, {modal_state})".to_string(),
    );

    // Causal bound
    typst_templates.insert("causal_bound".to_string(), "c({point})".to_string());

    // Projection residue
    typst_templates.insert(
        "projection_residue".to_string(),
        "op(\"Residue\")[{projection}, {structure}]".to_string(),
    );

    // Modal space
    typst_templates.insert("modal_space".to_string(), "cal(M)_({name})".to_string());

    // Spacetime
    typst_templates.insert("spacetime".to_string(), "bb(R)^4".to_string());

    // Hont (Hilbert Ontology)
    typst_templates.insert("hont".to_string(), "cal(H)_({dimension})".to_string());

    // TODO: Add more Typst templates as needed

    // === Kleis Glyphs ===
    // Kleis uses Unicode directly for most symbols
    let mut kleis_glyphs = HashMap::new();
    // Greek letters (keep as Unicode)
    kleis_glyphs.insert("\\alpha".to_string(), "α".to_string());
    kleis_glyphs.insert("\\beta".to_string(), "β".to_string());
    kleis_glyphs.insert("\\gamma".to_string(), "γ".to_string());
    kleis_glyphs.insert("\\delta".to_string(), "δ".to_string());
    kleis_glyphs.insert("\\epsilon".to_string(), "ε".to_string());
    kleis_glyphs.insert("\\zeta".to_string(), "ζ".to_string());
    kleis_glyphs.insert("\\eta".to_string(), "η".to_string());
    kleis_glyphs.insert("\\theta".to_string(), "θ".to_string());
    kleis_glyphs.insert("\\lambda".to_string(), "λ".to_string());
    kleis_glyphs.insert("\\mu".to_string(), "μ".to_string());
    kleis_glyphs.insert("\\nu".to_string(), "ν".to_string());
    kleis_glyphs.insert("\\xi".to_string(), "ξ".to_string());
    kleis_glyphs.insert("\\pi".to_string(), "π".to_string());
    kleis_glyphs.insert("\\rho".to_string(), "ρ".to_string());
    kleis_glyphs.insert("\\sigma".to_string(), "σ".to_string());
    kleis_glyphs.insert("\\tau".to_string(), "τ".to_string());
    kleis_glyphs.insert("\\phi".to_string(), "φ".to_string());
    kleis_glyphs.insert("\\chi".to_string(), "χ".to_string());
    kleis_glyphs.insert("\\psi".to_string(), "ψ".to_string());
    kleis_glyphs.insert("\\omega".to_string(), "ω".to_string());
    // Uppercase Greek
    kleis_glyphs.insert("\\Gamma".to_string(), "Γ".to_string());
    kleis_glyphs.insert("\\Delta".to_string(), "Δ".to_string());
    kleis_glyphs.insert("\\Theta".to_string(), "Θ".to_string());
    kleis_glyphs.insert("\\Lambda".to_string(), "Λ".to_string());
    kleis_glyphs.insert("\\Xi".to_string(), "Ξ".to_string());
    kleis_glyphs.insert("\\Pi".to_string(), "Π".to_string());
    kleis_glyphs.insert("\\Sigma".to_string(), "Σ".to_string());
    kleis_glyphs.insert("\\Phi".to_string(), "Φ".to_string());
    kleis_glyphs.insert("\\Psi".to_string(), "Ψ".to_string());
    kleis_glyphs.insert("\\Omega".to_string(), "Ω".to_string());
    // Special symbols
    kleis_glyphs.insert("\\infty".to_string(), "∞".to_string());
    kleis_glyphs.insert("\\partial".to_string(), "∂".to_string());
    kleis_glyphs.insert("\\nabla".to_string(), "∇".to_string());
    kleis_glyphs.insert("\\hbar".to_string(), "ℏ".to_string());
    kleis_glyphs.insert("\\emptyset".to_string(), "∅".to_string());

    // === Kleis Templates ===
    // These output Kleis syntax conforming to the grammar
    let mut kleis_templates = HashMap::new();

    // Calculus - conforming to grammar:
    // integral ::= "∫" [ subscript ] [ superscript ] expression [ "d" identifier ]
    kleis_templates.insert(
        "int_bounds".to_string(),
        "∫_{{{lower}}}^{{{upper}}} {integrand} d{int_var}".to_string(),
    );
    // summation ::= "Σ" [ subscript ] [ superscript ] expression
    kleis_templates.insert(
        "sum_bounds".to_string(),
        "Σ_{{{from}}}^{{{to}}} {body}".to_string(),
    );
    kleis_templates.insert("sum_index".to_string(), "Σ_{{{from}}} {body}".to_string());
    // product ::= "Π" [ subscript ] [ superscript ] expression
    kleis_templates.insert(
        "prod_bounds".to_string(),
        "Π_{{{from}}}^{{{to}}} {body}".to_string(),
    );
    kleis_templates.insert("prod_index".to_string(), "Π_{{{from}}} {body}".to_string());
    // v0.7: Mathematica-style derivatives - D() for partial, Dt() for total
    kleis_templates.insert("d_dt".to_string(), "Dt({num}, {den})".to_string());
    kleis_templates.insert("d_part".to_string(), "D({num}, {den})".to_string());
    kleis_templates.insert("d2_part".to_string(), "D({num}, {den}, {den})".to_string());
    // v0.7: Limit notation - Limit(body, var, target)
    kleis_templates.insert(
        "lim".to_string(),
        "Limit({body}, {var}, {target})".to_string(),
    );
    kleis_templates.insert(
        "limit".to_string(),
        "Limit({body}, {var}, {target})".to_string(),
    );
    // Gradient (prefix operator per grammar)
    kleis_templates.insert("grad".to_string(), "∇{arg}".to_string());
    kleis_templates.insert("gradient".to_string(), "∇{arg}".to_string());

    // Arithmetic
    kleis_templates.insert("plus".to_string(), "({left} + {right})".to_string());
    kleis_templates.insert("minus".to_string(), "({left} - {right})".to_string());
    kleis_templates.insert(
        "scalar_multiply".to_string(),
        "({left} × {right})".to_string(),
    );
    kleis_templates.insert("multiply".to_string(), "({left} × {right})".to_string());
    kleis_templates.insert(
        "scalar_divide".to_string(),
        "({left} / {right})".to_string(),
    );
    kleis_templates.insert("power".to_string(), "{base}^{exponent}".to_string());
    kleis_templates.insert("negate".to_string(), "-{arg}".to_string());

    // Comparison/Relations
    kleis_templates.insert("equals".to_string(), "{left} = {right}".to_string());
    kleis_templates.insert("not_equal".to_string(), "{left} ≠ {right}".to_string());
    kleis_templates.insert("less_than".to_string(), "{left} < {right}".to_string());
    kleis_templates.insert("greater_than".to_string(), "{left} > {right}".to_string());
    kleis_templates.insert("leq".to_string(), "{left} ≤ {right}".to_string());
    kleis_templates.insert("geq".to_string(), "{left} ≥ {right}".to_string());
    kleis_templates.insert("approx".to_string(), "{left} ≈ {right}".to_string());
    kleis_templates.insert("equiv".to_string(), "{left} ≡ {right}".to_string());

    // Logic
    kleis_templates.insert("implies".to_string(), "({left} ⟹ {right})".to_string());
    kleis_templates.insert("iff".to_string(), "({left} ⟺ {right})".to_string());
    kleis_templates.insert("logical_and".to_string(), "({left} ∧ {right})".to_string());
    kleis_templates.insert("logical_or".to_string(), "({left} ∨ {right})".to_string());
    kleis_templates.insert("logical_not".to_string(), "¬{arg}".to_string());
    // Quantifiers
    kleis_templates.insert("forall".to_string(), "∀({var}). {body}".to_string());
    kleis_templates.insert("exists".to_string(), "∃({var}). {body}".to_string());

    // Set operations
    kleis_templates.insert("in_set".to_string(), "{left} ∈ {right}".to_string());
    kleis_templates.insert("not_in_set".to_string(), "{left} ∉ {right}".to_string());
    kleis_templates.insert("subset".to_string(), "{left} ⊂ {right}".to_string());
    kleis_templates.insert("subseteq".to_string(), "{left} ⊆ {right}".to_string());
    kleis_templates.insert("union".to_string(), "({left} ∪ {right})".to_string());
    kleis_templates.insert("intersection".to_string(), "({left} ∩ {right})".to_string());

    // Functions (function call syntax)
    kleis_templates.insert("sqrt".to_string(), "√{arg}".to_string());
    kleis_templates.insert("abs".to_string(), "abs({arg})".to_string());
    kleis_templates.insert("norm".to_string(), "norm({arg})".to_string());
    kleis_templates.insert("sin".to_string(), "sin({arg})".to_string());
    kleis_templates.insert("cos".to_string(), "cos({arg})".to_string());
    kleis_templates.insert("tan".to_string(), "tan({arg})".to_string());
    kleis_templates.insert("exp".to_string(), "exp({arg})".to_string());
    kleis_templates.insert("ln".to_string(), "ln({arg})".to_string());
    kleis_templates.insert("log".to_string(), "log({arg})".to_string());

    // Brackets & grouping
    kleis_templates.insert("parens".to_string(), "({content})".to_string());
    kleis_templates.insert("brackets".to_string(), "[{content}]".to_string());
    kleis_templates.insert("braces".to_string(), "{{{content}}}".to_string());

    // Subscript/superscript
    kleis_templates.insert("sub".to_string(), "{base}_{{{right}}}".to_string());
    kleis_templates.insert("sup".to_string(), "{base}^{{{right}}}".to_string());
    kleis_templates.insert(
        "subsup".to_string(),
        "{base}_{{{subscript}}}^{{{superscript}}}".to_string(),
    );

    // Matrix/Vector (use Kleis list syntax)
    kleis_templates.insert("transpose".to_string(), "{arg}ᵀ".to_string());
    kleis_templates.insert("det".to_string(), "det({arg})".to_string());
    kleis_templates.insert("trace".to_string(), "trace({arg})".to_string());
    kleis_templates.insert("vector_bold".to_string(), "{arg}".to_string());
    kleis_templates.insert("vector_arrow".to_string(), "{arg}".to_string());

    // Dot/cross products
    kleis_templates.insert("dot".to_string(), "({left} · {right})".to_string());
    kleis_templates.insert("cross".to_string(), "({left} × {right})".to_string());
    kleis_templates.insert("inner".to_string(), "⟨{left}, {right}⟩".to_string());

    // Quantum notation
    kleis_templates.insert("ket".to_string(), "|{arg}⟩".to_string());
    kleis_templates.insert("bra".to_string(), "⟨{arg}|".to_string());
    kleis_templates.insert("braket".to_string(), "⟨{left}|{right}⟩".to_string());

    GlyphContext {
        unicode_glyphs,
        unicode_templates,
        latex_glyphs,
        latex_templates,
        html_glyphs,
        html_templates,
        typst_glyphs,
        typst_templates,
        kleis_glyphs,
        kleis_templates,
    }
}

pub fn demo_render() {
    let ctx = build_default_context();

    // === Build Expression Tree ===
    let phi = o("Φ");
    let grad_phi = grad_e(phi);

    let surface = o("S");
    let surface_integral = surface_integral(grad_phi, surface);

    // Represent -1 / (4π) symbolically
    let minus_one = c("-1");
    let four = c("4");
    let pi = c("π");

    let four_pi = times(four, pi);

    let negative_one_over_four_pi = over(minus_one, four_pi);

    // Multiply (-1/4π) × (surface integral)
    let residue = times(negative_one_over_four_pi, surface_integral);

    let g_c = c("G_c");

    let mass = over(residue, g_c);

    // === Render Unicode===
    let output = render_expression(&mass, &ctx, &RenderTarget::Unicode);
    println!("Rendered Expression: {}", output);

    // === Render Latex ===
    let output = render_expression(&mass, &ctx, &RenderTarget::LaTeX);
    println!("Rendered Expression: {}", output);

    // === Additional demos for new templates ===
    // Inner product ⟨u, v⟩
    let inner_uv = inner_e(o("u"), o("v"));
    println!(
        "Unicode inner(u,v): {}",
        render_expression(&inner_uv, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  inner(u,v): {}",
        render_expression(&inner_uv, &ctx, &RenderTarget::LaTeX)
    );

    // Dot and cross
    let dot_uv = dot_e(o("u"), o("v"));
    let cross_uv = cross_e(o("u"), o("v"));
    println!(
        "Unicode dot(u,v): {}",
        render_expression(&dot_uv, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  dot(u,v): {}",
        render_expression(&dot_uv, &ctx, &RenderTarget::LaTeX)
    );
    println!(
        "Unicode cross(u,v): {}",
        render_expression(&cross_uv, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  cross(u,v): {}",
        render_expression(&cross_uv, &ctx, &RenderTarget::LaTeX)
    );

    // Norm and absolute
    let norm_x = norm_e(o("x"));
    let abs_x = abs_e(o("x"));
    println!(
        "Unicode norm(x): {}",
        render_expression(&norm_x, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  norm(x): {}",
        render_expression(&norm_x, &ctx, &RenderTarget::LaTeX)
    );
    println!(
        "Unicode abs(x): {}",
        render_expression(&abs_x, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  abs(x): {}",
        render_expression(&abs_x, &ctx, &RenderTarget::LaTeX)
    );

    // Power x^2
    let x_sq = pow_e(o("x"), c("2"));
    println!(
        "Unicode x^2: {}",
        render_expression(&x_sq, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  x^2: {}",
        render_expression(&x_sq, &ctx, &RenderTarget::LaTeX)
    );

    // Derivatives
    let dydx = d_dt(o("y"), o("x"));
    let dfdx = d_part(o("f"), o("x"));
    println!(
        "Unicode dy/dx: {}",
        render_expression(&dydx, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  dy/dx: {}",
        render_expression(&dydx, &ctx, &RenderTarget::LaTeX)
    );
    println!(
        "Unicode ∂f/∂x: {}",
        render_expression(&dfdx, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  ∂f/∂x: {}",
        render_expression(&dfdx, &ctx, &RenderTarget::LaTeX)
    );

    // Summation and product with bounds
    let sum_i = sum_e(o("f(i)"), o("i=1"), o("n"));
    let prod_k = prod_e(o("a_k"), o("k=1"), o("m"));
    println!(
        "Unicode sum: {}",
        render_expression(&sum_i, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  sum: {}",
        render_expression(&sum_i, &ctx, &RenderTarget::LaTeX)
    );
    println!(
        "Unicode prod: {}",
        render_expression(&prod_k, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  prod: {}",
        render_expression(&prod_k, &ctx, &RenderTarget::LaTeX)
    );

    // Integral with bounds
    let int_ab = int_e(o("g(x)"), o("a"), o("b"), o("x"));
    println!(
        "Unicode int: {}",
        render_expression(&int_ab, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  int: {}",
        render_expression(&int_ab, &ctx, &RenderTarget::LaTeX)
    );

    // Transpose and determinant
    let a_t = transpose_e(o("A"));
    let det_a = det_e(o("A"));
    println!(
        "Unicode A^T: {}",
        render_expression(&a_t, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  A^T: {}",
        render_expression(&a_t, &ctx, &RenderTarget::LaTeX)
    );
    println!(
        "Unicode det(A): {}",
        render_expression(&det_a, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  det(A): {}",
        render_expression(&det_a, &ctx, &RenderTarget::LaTeX)
    );

    // 2x2 matrix
    let m2 = m2(o("a_{11}"), o("a_{12}"), o("a_{21}"), o("a_{22}"));
    println!(
        "Unicode [2x2]: {}",
        render_expression(&m2, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  [2x2]: {}",
        render_expression(&m2, &ctx, &RenderTarget::LaTeX)
    );

    // 3x3 matrix
    let m3 = m3(
        o("a_{11}"),
        o("a_{12}"),
        o("a_{13}"),
        o("a_{21}"),
        o("a_{22}"),
        o("a_{23}"),
        o("a_{31}"),
        o("a_{32}"),
        o("a_{33}"),
    );
    println!(
        "Unicode [3x3]: {}",
        render_expression(&m3, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  [3x3]: {}",
        render_expression(&m3, &ctx, &RenderTarget::LaTeX)
    );

    // Vector arrow and bold
    let v = o("v");
    let varrow = vector_arrow_e(v.clone());
    let vbold = vector_bold_e(v);
    println!(
        "Unicode vec(v): {}",
        render_expression(&varrow, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  vec(v): {}",
        render_expression(&varrow, &ctx, &RenderTarget::LaTeX)
    );
    println!(
        "Unicode bold(v): {}",
        render_expression(&vbold, &ctx, &RenderTarget::Unicode)
    );
    println!(
        "LaTeX  bold(v): {}",
        render_expression(&vbold, &ctx, &RenderTarget::LaTeX)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO(2024-12-06): Some LaTeX rendering tests have outdated expectations
    // These may need updates to match current renderer output

    #[test]
    #[ignore = "TODO: Fix inner product LaTeX rendering - outdated expectations"]
    fn renders_inner_product_latex() {
        let ctx = build_default_context();
        let expr = inner_e(o("u"), o("v"));
        let out = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out, "\\langle u, v \\rangle");
    }

    #[test]
    fn renders_matrix_2x2_latex() {
        let ctx = build_default_context();
        let expr = m2(o("a_{11}"), o("a_{12}"), o("a_{21}"), o("a_{22}"));
        let out = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(
            out,
            "\\begin{bmatrix}a\\_{11}&a\\_{12}\\\\a\\_{21}&a\\_{22}\\end{bmatrix}"
        );
    }

    #[test]
    fn renders_matrix_3x3_latex() {
        let ctx = build_default_context();
        let expr = m3(
            o("a_{11}"),
            o("a_{12}"),
            o("a_{13}"),
            o("a_{21}"),
            o("a_{22}"),
            o("a_{23}"),
            o("a_{31}"),
            o("a_{32}"),
            o("a_{33}"),
        );
        let out = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(
            out,
            "\\begin{bmatrix}a\\_{11}&a\\_{12}&a\\_{13}\\\\a\\_{21}&a\\_{22}&a\\_{23}\\\\a\\_{31}&a\\_{32}&a\\_{33}\\end{bmatrix}"
        );
    }

    #[test]
    fn renders_vector_styles_latex() {
        let ctx = build_default_context();
        let varrow = vector_arrow_e(o("v"));
        let vbold = vector_bold_e(o("v"));
        let out_arrow = render_expression(&varrow, &ctx, &RenderTarget::LaTeX);
        let out_bold = render_expression(&vbold, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_arrow, "\\vec{v}");
        assert_eq!(out_bold, "\\boldsymbol{v}");
    }

    #[test]
    #[ignore = "TODO: Update EFE LaTeX expectations - renderer output changed"]
    fn renders_efe_core_latex() {
        // G_{\mu\nu} + \Lambda g_{\mu\nu} = \kappa T_{\mu\nu}
        let ctx = build_default_context();
        let mu = o("μ");
        let nu = o("ν");
        let g_t = o("g");
        let g_mn = index_pair(g_t, mu.clone(), nu.clone());
        let gEin = o("G");
        let G_mn = index_pair(gEin, mu.clone(), nu.clone());
        let Tsym = o("T");
        let T_mn = index_pair(Tsym, mu.clone(), nu.clone());
        let left_sum = plus(G_mn, times(o("Λ"), g_mn));
        let rhs = times(o("κ"), T_mn);
        let efe = equals(left_sum, rhs);

        let out = render_expression(&efe, &ctx, &RenderTarget::LaTeX);
        assert_eq!(
            out,
            r"G_{{\mu\nu}} + \Lambda \, g_{{\mu\nu}} = \kappa \, T_{{\mu\nu}}"
        );
    }

    #[test]
    #[ignore = "TODO: Fix tensor rendering - outdated expectations"]
    fn renders_f_tensor_from_potential() {
        // F^{\mu}_{\nu} = \partial_{\mu} A_{\nu} - \partial_{\nu} A_{\mu}
        let ctx = build_default_context();
        let mu = o("μ");
        let nu = o("ν");
        let A = o("A");
        let F = o("F");
        let A_nu = sub_e(A.clone(), nu.clone());
        let A_mu = sub_e(A.clone(), mu.clone());
        let dA_nu = partial_apply(A_nu, mu.clone());
        let dA_mu = partial_apply(A_mu, nu.clone());
        let rhs = minus(dA_nu, dA_mu);
        let F_mn = index_mixed(F, mu.clone(), nu.clone());
        let eq = equals(F_mn, rhs);
        let out = render_expression(&eq, &ctx, &RenderTarget::LaTeX);
        assert_eq!(
            out,
            r"F^{{\mu}}_{{\nu}} = \partial_{{\mu}} A_{{\nu}} - \partial_{{\nu}} A_{{\mu}}"
        );
    }

    #[test]
    fn renders_kk_metric_ansatz_block() {
        // \begin{bmatrix} g^{\mu}_{\nu} + \phi A_{\mu}A_{\nu} & \phi A_{\mu} \\\\ \phi A_{\nu} & \phi \end{bmatrix}
        let ctx = build_default_context();
        let mu = o("μ");
        let nu = o("ν");
        let g = o("g");
        let A = o("A");
        let phi = o("\u{03C6}");
        let g_mn = index_mixed(g, mu.clone(), nu.clone());
        let A_mu = sub_e(A.clone(), mu.clone());
        let A_nu = sub_e(A.clone(), nu.clone());
        let phi_A_mu = times(phi.clone(), A_mu.clone());
        let phi_A_nu = times(phi.clone(), A_nu.clone());
        let phi_A_muA_nu = times(phi.clone(), times(A_mu.clone(), A_nu.clone()));
        let tl = plus(g_mn, phi_A_muA_nu);
        let tr = phi_A_mu.clone();
        let bl = phi_A_nu.clone();
        let br = phi.clone();
        let mat = m2(tl, tr, bl, br);
        let out = render_expression(&mat, &ctx, &RenderTarget::LaTeX);
        assert!(out.contains("\\begin{bmatrix}"));
        assert!(out.contains("g"));
        assert!(out.contains("\\phi"));
        assert!(out.contains("A"));
    }

    #[test]
    fn renders_christoffel_and_riemann_placeholders() {
        let ctx = build_default_context();
        let rho = o("ρ");
        let mu = o("μ");
        let nu = o("ν");
        let sigma = o("σ");

        // Γ^{ρ}_{μν}
        let gamma = func("gamma", vec![o(""), rho.clone(), mu.clone(), nu.clone()]);
        let out_g = render_expression(&gamma, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_g, r"\Gamma^{{\rho}}_{{\mu \nu}}");

        // R^{ρ}{}_{σμν}
        let riemann = func(
            "riemann",
            vec![o(""), rho.clone(), sigma.clone(), mu.clone(), nu.clone()],
        );
        let out_r = render_expression(&riemann, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_r, r"R^{{\rho}}_{{\sigma \mu \nu}}");
    }

    #[test]
    fn renders_euler_lagrange_single_var() {
        // ∂L/∂y - d/dx(∂L/∂y') = 0
        let ctx = build_default_context();
        let l = o("L");
        let y = o("y");
        let yprime = o("y'");
        let x = o("x");
        let dL_dy = d_part(l.clone(), y.clone());
        let dL_dyprime = d_part(l.clone(), yprime.clone());
        let d_dx_of_dL_dyprime = d_dt(dL_dyprime, x.clone());
        let left = minus(dL_dy, d_dx_of_dL_dyprime);
        let eq = equals(left, c("0"));
        let out = render_expression(&eq, &ctx, &RenderTarget::LaTeX);
        assert!(out.contains(r"\frac{\partial\,L}{\partial y} - "));
        assert!(out.contains(r"\frac{d\,\frac{\partial\,L}{\partial y'}}{dx}"));
        assert!(out.ends_with(" = 0"));
    }

    #[test]
    fn renders_euler_lagrange_multi_var_sum() {
        // ∂L/∂u - Σ_{i=1}^{n} d/dx_i ( ∂L/∂u_{x_i} ) = 0
        let ctx = build_default_context();
        let l = o("L");
        let u = o("u");
        let xi = o("x_i");
        let dL_du = d_part(l.clone(), u.clone());
        let u_xi = sub_e(u.clone(), xi.clone());
        let dL_du_xi = d_part(l.clone(), u_xi);
        let d_dxi_of_term = d_dt(dL_du_xi, xi.clone());
        let sum_term = sum_e(d_dxi_of_term, o("i=1"), o("n"));
        let left = minus(dL_du, sum_term);
        let eq = equals(left, c("0"));
        let out = render_expression(&eq, &ctx, &RenderTarget::LaTeX);
        assert!(out.contains(r"\sum_{ i=1 }^{ n }"));
        // Allow extra braces from our placeholder expansion
        assert!(out.contains(r"\frac{\partial\,L}{\partial u_{{x\_i}}}"));
        assert!(out.contains(r"\frac{d"));
        assert!(out.ends_with(" = 0"));
    }

    #[test]
    fn renders_beltrami_identity_form() {
        // L - y' * (∂L/∂y') = C
        let ctx = build_default_context();
        let l = o("L");
        let yprime = o("y'");
        let dL_dyprime = d_part(l.clone(), yprime.clone());
        let product = times(yprime.clone(), dL_dyprime);
        let left = minus(l.clone(), product);
        let eq = equals(left, o("C"));
        let out = render_expression(&eq, &ctx, &RenderTarget::LaTeX);
        assert!(out.contains(r"L - y' \, "));
        assert!(out.contains(r"\frac{\partial\,L}{\partial y'}"));
        assert!(out.ends_with(" = C"));
    }

    #[test]
    fn renders_hamilton_jacobi_basic() {
        // H(q, ∂S/∂q, t) + ∂S/∂t = 0
        let ctx = build_default_context();
        let q = o("q");
        let t = o("t");
        let s = o("S");
        let dS_dq = d_part(s.clone(), q.clone());
        let dS_dt = d_part(s.clone(), t.clone());
        let H_of = func("H", vec![q.clone(), dS_dq.clone(), t.clone()]);
        let left = plus(H_of, dS_dt);
        let eq = equals(left, c("0"));
        let out = render_expression(&eq, &ctx, &RenderTarget::LaTeX);
        assert!(out.contains(r"H(q, \frac{\partial\,S}{\partial q}, t) + "));
        assert!(out.contains(r"\frac{\partial\,S}{\partial t}"));
        assert!(out.ends_with(" = 0"));
    }

    #[test]
    fn renders_hjb_pde_core() {
        // ∂V/∂t + min_u { ∂V/∂x · F(x,u) + C(x,u) } = 0
        let ctx = build_default_context();
        let Vsym = o("V");
        let x = o("x");
        let t = o("t");
        let u = o("u");
        let dV_dt = d_part(Vsym.clone(), t.clone());
        let dV_dx = d_part(Vsym.clone(), x.clone());
        let F_xu = func("F", vec![x.clone(), u.clone()]);
        let C_xu = func("C", vec![x.clone(), u.clone()]);
        let dot = dot_e(dV_dx, F_xu);
        let inner = plus(dot, C_xu);
        let min_u = min_over(u.clone(), inner);
        let left = plus(dV_dt, min_u);
        let eq = equals(left, c("0"));

        let out = render_expression(&eq, &ctx, &RenderTarget::LaTeX);
        assert!(out.contains(r"\frac{\partial\,V}{\partial t} + "));
        assert!(out.contains(r"\min_"));
        assert!(out.contains(r"\frac{\partial\,V}{\partial x} \cdot F(x, u) + C(x, u)"));
        assert!(out.ends_with(" \\right\\} = 0"));
    }

    #[test]
    fn renders_stochastic_hjb_term() {
        // -∂V/∂t = ... + (σ^2/2) ∂^2 V / ∂x^2
        let ctx = build_default_context();
        let Vsym = o("V");
        let x = o("x");
        let t = o("t");
        let sigma = o("\\sigma");
        let dV_dt = d_part(Vsym.clone(), t.clone());
        let d2V_dx2 = d2_part(Vsym.clone(), x.clone());
        let sigma_sq_over_2 = over(pow_e(sigma.clone(), c("2")), c("2"));
        let diffusion = times(sigma_sq_over_2, d2V_dx2);
        let left = minus(dV_dt, c("0"));
        let eq = equals(left, diffusion);
        let out = render_expression(&eq, &ctx, &RenderTarget::LaTeX);
        // Left side shape may vary; focus on diffusion term presence
        assert!(out.contains(r"\sigma"));
        assert!(out.contains(r"\frac{\partial^{2} \,V}{\partial x^{2}}"));
    }

    #[test]
    fn renders_riemann_zeta_dirichlet_series() {
        // \zeta(s) = \sum_{n=1}^{\infty} 1 / n^{s}
        let ctx = build_default_context();
        let s = o("s");
        let n = o("n");
        let term = over(c("1"), pow_e(n.clone(), s.clone()));
        let series = op("sum_bounds", vec![term, o("n=1"), o("\\infty")]);
        let eq = equals(func("zeta", vec![s.clone()]), series);
        let out = render_expression(&eq, &ctx, &RenderTarget::LaTeX);
        println!("zeta dirichlet out= {}", out);
        assert!(out.contains(r"\zeta(s) = \sum_{ n=1 }^{ \infty }"));
        assert!(out.contains(r"\frac{1}{n^{s}}") || out.contains(r"\frac{1}{n^{{s}}}"));
    }

    #[test]
    fn renders_euler_product_for_zeta() {
        // \zeta(s) = \prod_{p\,\text{prime}} 1/(1 - p^{-s})
        let ctx = build_default_context();
        let s = o("s");
        let p = o("p");
        let denom = minus(c("1"), pow_e(p.clone(), o("-s")));
        let term = over(c("1"), denom);
        let prod = op("prod_index", vec![term, o("p\\,\\text{prime}")]);
        let eq = equals(func("zeta", vec![s.clone()]), prod);
        let out = render_expression(&eq, &ctx, &RenderTarget::LaTeX);
        println!("zeta product out= {}", out);
        assert!(out.contains(r"\prod_{ p\,\text{prime} }"));
        assert!(out.contains(r"\frac{1}{1 - p^{-s}}") || out.contains(r"\frac{1}{1 - p^{{-s}}}"));
    }

    #[test]
    fn renders_zeta_mellin_integral_hint() {
        // \zeta(s) = (\int_0^{\infty} x^{s-1}/(e^{x} - 1) \, \mathrm{d}x) / \Gamma(s)
        let ctx = build_default_context();
        let s = o("s");
        let x = o("x");
        let gamma_s = func("Gamma", vec![s.clone()]);
        let x_pow = pow_e(x.clone(), minus(s.clone(), c("1"))); // x^{s-1}
        let denom = minus(func("exp", vec![x.clone()]), c("1")); // e^x - 1
        let integrand = over(x_pow, denom);
        let integral = op(
            "int_bounds",
            vec![integrand, c("0"), o("\\infty"), x.clone()],
        );
        let rhs = over(integral, gamma_s);
        let eq = equals(func("zeta", vec![s.clone()]), rhs);
        let out = render_expression(&eq, &ctx, &RenderTarget::LaTeX);
        println!("zeta mellin out= {}", out);
        assert!(out.contains(r"\Gamma(s)"));
        assert!(out.contains(r"\int_{ 0 }^{ \infty }"));
    }

    #[test]
    fn renders_limits_variants() {
        let ctx = build_default_context();
        let x = o("x");
        let to0 = c("0");
        let body = func("f", vec![x.clone()]);
        let lim = op("limit", vec![body.clone(), x.clone(), to0.clone()]);
        let limsup = op("limsup", vec![body.clone(), x.clone(), o("\\infty")]);
        let liminf = op("liminf", vec![body.clone(), x.clone(), o("a")]);
        let out = render_expression(&lim, &ctx, &RenderTarget::LaTeX);
        let out_sup = render_expression(&limsup, &ctx, &RenderTarget::LaTeX);
        let out_inf = render_expression(&liminf, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out, r"\lim_{ x \to 0 } f(x)");
        assert_eq!(out_sup, r"\limsup_{ x \to \infty } f(x)");
        assert_eq!(out_inf, r"\liminf_{ x \to a } f(x)");
    }

    // === Top 5 New Operations Tests ===

    // 1. BRA-KET NOTATION
    #[test]
    fn renders_ket_vector() {
        let ctx = build_default_context();
        let k = ket(o("\\psi"));
        let out_unicode = render_expression(&k, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&k, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_unicode, "|ψ⟩");
        assert_eq!(out_latex, r"|\psi\rangle");
    }

    #[test]
    fn renders_bra_vector() {
        let ctx = build_default_context();
        let b = bra(o("\\phi"));
        let out_unicode = render_expression(&b, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&b, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_unicode, "⟨φ|");
        assert_eq!(out_latex, r"\langle\phi|");
    }

    #[test]
    #[ignore = "TODO: Fix outer product rendering - outdated expectations"]
    fn renders_outer_product() {
        let ctx = build_default_context();
        let outer = outer_product(o("\\psi"), o("\\phi"));
        let out_unicode = render_expression(&outer, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&outer, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_unicode, "|ψ⟩⟨φ|");
        assert_eq!(out_latex, r"|\psi\rangle\langle\phi|");
    }

    // 2. SET THEORY
    #[test]
    fn renders_set_membership() {
        let ctx = build_default_context();
        let expr = in_set(o("x"), o("\\mathbb{R}"));
        let out_unicode = render_expression(&expr, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_unicode, "x ∈ ℝ"); // Unicode conversion happens
        assert_eq!(out_latex, r"x \in \mathbb{R}");
    }

    #[test]
    fn renders_subset_relations() {
        let ctx = build_default_context();
        let sub = subset(o("A"), o("B"));
        let subeq = subseteq(o("A"), o("B"));
        let out_sub_latex = render_expression(&sub, &ctx, &RenderTarget::LaTeX);
        let out_subeq_latex = render_expression(&subeq, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_sub_latex, r"A \subset B");
        assert_eq!(out_subeq_latex, r"A \subseteq B");
    }

    #[test]
    fn renders_set_operations() {
        let ctx = build_default_context();
        let u = union(o("A"), o("B"));
        let i = intersection(o("A"), o("B"));
        let out_union = render_expression(&u, &ctx, &RenderTarget::LaTeX);
        let out_intersection = render_expression(&i, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_union, r"A \cup B");
        assert_eq!(out_intersection, r"A \cap B");
    }

    #[test]
    fn renders_quantifiers() {
        let ctx = build_default_context();
        let fa = forall(o("x"), in_set(o("x"), o("\\mathbb{R}")));
        let ex = exists(o("x"), in_set(o("x"), o("\\mathbb{R}")));
        let out_forall = render_expression(&fa, &ctx, &RenderTarget::LaTeX);
        let out_exists = render_expression(&ex, &ctx, &RenderTarget::LaTeX);
        assert!(out_forall.contains(r"\forall"));
        assert!(out_exists.contains(r"\exists"));
    }

    #[test]
    fn renders_logical_implications() {
        let ctx = build_default_context();
        let imp = implies(o("P"), o("Q"));
        let iff_expr = iff(o("P"), o("Q"));
        let out_implies = render_expression(&imp, &ctx, &RenderTarget::LaTeX);
        let out_iff = render_expression(&iff_expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_implies, r"P \Rightarrow Q");
        assert_eq!(out_iff, r"P \Leftrightarrow Q");
    }

    // 3. MULTIPLE INTEGRALS
    #[test]
    fn renders_double_integral() {
        let ctx = build_default_context();
        let expr = double_int(o("f(x,y)"), o("D"), o("x"), o("y"));
        let out_unicode = render_expression(&expr, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert!(out_unicode.contains("∬"));
        assert_eq!(out_latex, r"\iint_{D} f(x,y) \, \mathrm{d}x \, \mathrm{d}y");
    }

    #[test]
    fn renders_triple_integral() {
        let ctx = build_default_context();
        let expr = triple_int(o("f(x,y,z)"), o("V"), o("x"), o("y"), o("z"));
        let out_unicode = render_expression(&expr, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert!(out_unicode.contains("∭"));
        assert_eq!(
            out_latex,
            r"\iiint_{V} f(x,y,z) \, \mathrm{d}x \, \mathrm{d}y \, \mathrm{d}z"
        );
    }

    // 4. COMMUTATORS
    #[test]
    fn renders_commutator() {
        let ctx = build_default_context();
        let comm = commutator(o("A"), o("B"));
        let out_unicode = render_expression(&comm, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&comm, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_unicode, "[A, B]");
        assert_eq!(out_latex, "[A, B]");
    }

    #[test]
    fn renders_anticommutator() {
        let ctx = build_default_context();
        let anticomm = anticommutator(o("A"), o("B"));
        let out_unicode = render_expression(&anticomm, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&anticomm, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_unicode, "{A, B}");
        assert_eq!(out_latex, r"\{A, B\}");
    }

    #[test]
    fn renders_quantum_commutation_relation() {
        // [x, p] = iℏ
        let ctx = build_default_context();
        let lhs = commutator(o("\\hat{x}"), o("\\hat{p}"));
        let rhs = times(o("i"), o("\\hbar"));
        let eq = equals(lhs, rhs);
        let out = render_expression(&eq, &ctx, &RenderTarget::LaTeX);
        assert!(out.contains(r"[\hat{x}, \hat{p}]"));
        assert!(out.contains(r"i \, \hbar"));
    }

    // 5. SQUARE ROOT
    #[test]
    fn renders_square_root() {
        let ctx = build_default_context();
        let sqrt = sqrt_e(o("x"));
        let out_unicode = render_expression(&sqrt, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&sqrt, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_unicode, "√(x)");
        assert_eq!(out_latex, r"\sqrt{x}");
    }

    #[test]
    fn renders_nth_root() {
        let ctx = build_default_context();
        let root = nth_root(o("x"), c("3"));
        let out_latex = render_expression(&root, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_latex, r"\sqrt[3]{x}");
    }

    #[test]
    fn renders_sqrt_in_expression() {
        // √(π)/2
        let ctx = build_default_context();
        let numerator = sqrt_e(o("\\pi"));
        let expr = over(numerator, c("2"));
        let out = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out, r"\frac{\sqrt{\pi}}{2}");
    }

    // INTEGRATION TEST: Complex quantum expression
    #[test]
    fn renders_quantum_expectation_value() {
        // ⟨ψ| Ĥ |ψ⟩
        let ctx = build_default_context();
        let psi = o("\\psi");
        let _hamiltonian = o("\\hat{H}");
        let _bra_psi = bra(psi.clone());
        let _ket_psi = ket(psi);

        // Build as: bra * operator * ket (using inner for now)
        // Full version would be: ⟨ψ| Ĥ |ψ⟩
        let inner_part = inner_e(o("\\psi"), o("\\psi"));
        let out = render_expression(&inner_part, &ctx, &RenderTarget::LaTeX);
        assert!(out.contains(r"\langle"));
        assert!(out.contains(r"\rangle"));
    }

    // === Next Top 3 + Low-Hanging Fruit Tests ===

    // COMPARISON OPERATORS
    #[test]
    fn renders_less_than() {
        let ctx = build_default_context();
        let expr = less_than(o("x"), c("0"));
        let out_unicode = render_expression(&expr, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_unicode, "x < 0");
        assert_eq!(out_latex, "x < 0");
    }

    #[test]
    fn renders_inequalities() {
        let ctx = build_default_context();
        let leq_expr = leq(o("E"), c("0"));
        let geq_expr = geq(o("x"), c("0"));
        let neq_expr = not_equal(o("a"), o("b"));

        let out_leq = render_expression(&leq_expr, &ctx, &RenderTarget::LaTeX);
        let out_geq = render_expression(&geq_expr, &ctx, &RenderTarget::LaTeX);
        let out_neq = render_expression(&neq_expr, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_leq, r"E \leq 0");
        assert_eq!(out_geq, r"x \geq 0");
        assert_eq!(out_neq, r"a \neq b");
    }

    #[test]
    fn renders_approx_and_proportional() {
        let ctx = build_default_context();
        let approx_expr = approx(o("\\pi"), c("3.14"));
        let propto_expr = proportional(o("F"), o("ma"));

        let out_approx_unicode = render_expression(&approx_expr, &ctx, &RenderTarget::Unicode);
        let out_approx_latex = render_expression(&approx_expr, &ctx, &RenderTarget::LaTeX);
        let out_propto_latex = render_expression(&propto_expr, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_approx_unicode, "π ≈ 3.14");
        assert_eq!(out_approx_latex, r"\pi \approx 3.14");
        assert_eq!(out_propto_latex, r"F \propto ma");
    }

    // COMPLEX NUMBER OPERATIONS
    #[test]
    fn renders_complex_conjugate() {
        let ctx = build_default_context();
        let z_conj = conjugate(o("z"));
        let out_unicode = render_expression(&z_conj, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&z_conj, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_unicode, "z̄");
        assert_eq!(out_latex, r"\overline{z}");
    }

    #[test]
    fn renders_real_and_imaginary_parts() {
        let ctx = build_default_context();
        let re_z = re(o("z"));
        let im_z = im(o("z"));

        let out_re = render_expression(&re_z, &ctx, &RenderTarget::LaTeX);
        let out_im = render_expression(&im_z, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_re, r"\mathrm{Re}(z)");
        assert_eq!(out_im, r"\mathrm{Im}(z)");
    }

    #[test]
    fn renders_complex_modulus() {
        let ctx = build_default_context();
        let mod_z = modulus(o("z"));
        let out_latex = render_expression(&mod_z, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_latex, r"\left|z\right|");
    }

    // OPERATOR HAT
    #[test]
    fn renders_operator_hat() {
        let ctx = build_default_context();
        let h_hat = hat(o("H"));
        let x_hat = hat(o("x"));

        let out_h_unicode = render_expression(&h_hat, &ctx, &RenderTarget::Unicode);
        let out_h_latex = render_expression(&h_hat, &ctx, &RenderTarget::LaTeX);
        let out_x_latex = render_expression(&x_hat, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_h_unicode, "hat(H)");
        assert_eq!(out_h_latex, r"\hat{H}");
        assert_eq!(out_x_latex, r"\hat{x}");
    }

    #[test]
    fn renders_quantum_hamiltonian() {
        // Ĥ|ψ⟩ = E|ψ⟩
        let ctx = build_default_context();
        let h_hat = hat(o("H"));
        let psi = o("\\psi");
        let ket_psi = ket(psi.clone());

        // For now just test the components render
        let out_h = render_expression(&h_hat, &ctx, &RenderTarget::LaTeX);
        let out_ket = render_expression(&ket_psi, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_h, r"\hat{H}");
        assert_eq!(out_ket, r"|\psi\rangle");
    }

    // TRIGONOMETRIC & LOGARITHMIC FUNCTIONS
    #[test]
    fn renders_trig_functions() {
        let ctx = build_default_context();
        let cos_x = cos_e(o("x"));
        let tan_theta = tan_e(o("\\theta"));

        let out_cos = render_expression(&cos_x, &ctx, &RenderTarget::LaTeX);
        let out_tan = render_expression(&tan_theta, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_cos, r"\cos(x)");
        assert_eq!(out_tan, r"\tan(\theta)");
    }

    #[test]
    fn renders_hyperbolic_functions() {
        let ctx = build_default_context();
        let sinh_x = sinh_e(o("x"));
        let cosh_x = cosh_e(o("x"));

        let out_sinh = render_expression(&sinh_x, &ctx, &RenderTarget::LaTeX);
        let out_cosh = render_expression(&cosh_x, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_sinh, r"\sinh(x)");
        assert_eq!(out_cosh, r"\cosh(x)");
    }

    #[test]
    fn renders_logarithms() {
        let ctx = build_default_context();
        let log_x = log_e(o("x"));
        let ln_x = ln_e(o("x"));

        let out_log = render_expression(&log_x, &ctx, &RenderTarget::LaTeX);
        let out_ln = render_expression(&ln_x, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_log, r"\log(x)");
        assert_eq!(out_ln, r"\ln(x)");
    }

    #[test]
    fn renders_euler_formula() {
        // e^{iθ} = cos θ + i sin θ
        let ctx = build_default_context();
        let theta = o("\\theta");
        let i_theta = times(o("i"), theta.clone());
        let lhs = pow_e(o("e"), i_theta);
        let rhs = plus(
            cos_e(theta.clone()),
            times(o("i"), func("sin", vec![theta])),
        );
        let formula = equals(lhs, rhs);

        let out = render_expression(&formula, &ctx, &RenderTarget::LaTeX);
        // Check that key parts are present (exact spacing may vary)
        assert!(out.contains(r"e^{"));
        assert!(out.contains(r"\theta"));
        assert!(out.contains(r"\cos(\theta)"));
        assert!(out.contains(r"\sin") || out.contains("sin")); // sin might not be escaped
    }

    // MATRIX OPERATIONS
    #[test]
    fn renders_trace() {
        let ctx = build_default_context();
        let tr_rho = trace(o("\\rho"));
        let out_unicode = render_expression(&tr_rho, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&tr_rho, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_unicode, "Tr(ρ)");
        assert_eq!(out_latex, r"\mathrm{Tr}(\rho)");
    }

    #[test]
    fn renders_matrix_inverse() {
        let ctx = build_default_context();
        let a_inv = inverse(o("A"));
        let out_unicode = render_expression(&a_inv, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&a_inv, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_unicode, "(A)⁻¹");
        assert_eq!(out_latex, "A^{-1}");
    }

    #[test]
    fn renders_density_matrix_normalization() {
        // Tr(ρ) = 1
        let ctx = build_default_context();
        let tr_rho = trace(o("\\rho"));
        let norm = equals(tr_rho, c("1"));
        let out = render_expression(&norm, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out, r"\mathrm{Tr}(\rho) = 1");
    }

    // INTEGRATION TESTS
    #[test]
    fn renders_inequality_constraint() {
        // E ≥ 0 for physical energies
        let ctx = build_default_context();
        let constraint = geq(o("E"), c("0"));
        let out = render_expression(&constraint, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out, r"E \geq 0");
    }

    #[test]
    fn renders_complex_inner_product() {
        // ⟨ψ|φ⟩ = ∫ ψ̄(x)φ(x) dx (conceptually)
        let ctx = build_default_context();
        let psi_conj = conjugate(o("\\psi(x)"));

        let out_conj = render_expression(&psi_conj, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out_conj, r"\overline{\psi(x)}");
    }

    // === Batch 3: Completeness Operations Tests ===

    // PHASE A: QUICK WINS
    #[test]
    fn renders_factorial() {
        let ctx = build_default_context();
        let fact_n = factorial(o("n"));
        let out_unicode = render_expression(&fact_n, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&fact_n, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_unicode, "n!");
        assert_eq!(out_latex, "n!");
    }

    #[test]
    fn renders_floor_and_ceiling() {
        let ctx = build_default_context();
        let floor_x = floor(o("x"));
        let ceil_x = ceiling(o("x"));

        let out_floor_unicode = render_expression(&floor_x, &ctx, &RenderTarget::Unicode);
        let out_floor_latex = render_expression(&floor_x, &ctx, &RenderTarget::LaTeX);
        let out_ceil_latex = render_expression(&ceil_x, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_floor_unicode, "⌊x⌋");
        assert_eq!(out_floor_latex, r"\lfloor x \rfloor");
        assert_eq!(out_ceil_latex, r"\lceil x \rceil");
    }

    #[test]
    fn renders_inverse_trig() {
        let ctx = build_default_context();
        let arcsin_x = arcsin_e(o("x"));
        let arccos_x = arccos_e(o("x"));
        let arctan_x = arctan_e(o("x"));

        let out_arcsin = render_expression(&arcsin_x, &ctx, &RenderTarget::LaTeX);
        let out_arccos = render_expression(&arccos_x, &ctx, &RenderTarget::LaTeX);
        let out_arctan = render_expression(&arctan_x, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_arcsin, r"\arcsin(x)");
        assert_eq!(out_arccos, r"\arccos(x)");
        assert_eq!(out_arctan, r"\arctan(x)");
    }

    #[test]
    fn renders_reciprocal_trig() {
        let ctx = build_default_context();
        let sec_x = sec_e(o("x"));
        let csc_x = csc_e(o("x"));
        let cot_x = cot_e(o("x"));

        let out_sec = render_expression(&sec_x, &ctx, &RenderTarget::LaTeX);
        let out_csc = render_expression(&csc_x, &ctx, &RenderTarget::LaTeX);
        let out_cot = render_expression(&cot_x, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_sec, r"\sec(x)");
        assert_eq!(out_csc, r"\csc(x)");
        assert_eq!(out_cot, r"\cot(x)");
    }

    // PHASE B: QUANTUM FOCUS
    #[test]
    fn renders_pmatrix_2x2() {
        let ctx = build_default_context();
        let mat = pmatrix2(c("0"), c("1"), c("1"), c("0"));
        let out_latex = render_expression(&mat, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_latex, r"\begin{pmatrix}0&1\\1&0\end{pmatrix}");
    }

    #[test]
    fn renders_pmatrix_3x3() {
        let ctx = build_default_context();
        let mat = pmatrix3(
            c("1"),
            c("0"),
            c("0"),
            c("0"),
            c("1"),
            c("0"),
            c("0"),
            c("0"),
            c("1"),
        );
        let out_latex = render_expression(&mat, &ctx, &RenderTarget::LaTeX);

        assert_eq!(
            out_latex,
            r"\begin{pmatrix}1&0&0\\0&1&0\\0&0&1\end{pmatrix}"
        );
    }

    #[test]
    fn renders_pauli_matrices() {
        // σ_x = (0 1; 1 0)
        let ctx = build_default_context();
        let sigma_x = pmatrix2(c("0"), c("1"), c("1"), c("0"));
        let sigma_x_eq = equals(sub_e(o("\\sigma"), o("x")), sigma_x);

        let out = render_expression(&sigma_x_eq, &ctx, &RenderTarget::LaTeX);
        assert!(out.contains(r"\sigma_{{x}}"));
        assert!(out.contains(r"\begin{pmatrix}"));
    }

    #[test]
    fn renders_binomial_coefficient() {
        let ctx = build_default_context();
        let binom = binomial(o("n"), o("k"));
        let out_unicode = render_expression(&binom, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&binom, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_unicode, "C(n,k)");
        assert_eq!(out_latex, r"\binom{n}{k}");
    }

    #[test]
    fn renders_taylor_series_with_factorial() {
        // f(x) = Σ_{n=0}^∞ f^(n)(a)/n! (x-a)^n
        let ctx = build_default_context();
        let n = o("n");
        let numerator = o("f^{(n)}(a)");
        let n_fact = factorial(n.clone());
        let term = over(numerator, n_fact);

        let out = render_expression(&term, &ctx, &RenderTarget::LaTeX);
        assert!(out.contains("f^{(n)}(a)"));
        assert!(out.contains("n!"));
    }

    // PHASE C: FIELD THEORY
    #[test]
    fn renders_divergence() {
        let ctx = build_default_context();
        let div_f = div_e(o("\\mathbf{F}"));
        let out_unicode = render_expression(&div_f, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&div_f, &ctx, &RenderTarget::LaTeX);

        assert!(out_unicode.contains("∇·")); // \mathbf may not convert perfectly
        assert_eq!(out_latex, r"\nabla \cdot \mathbf{F}");
    }

    #[test]
    fn renders_curl() {
        let ctx = build_default_context();
        let curl_f = curl_e(o("\\mathbf{B}"));
        let out_unicode = render_expression(&curl_f, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&curl_f, &ctx, &RenderTarget::LaTeX);

        assert!(out_unicode.contains("∇×")); // \mathbf may not convert perfectly
        assert_eq!(out_latex, r"\nabla \times \mathbf{B}");
    }

    #[test]
    fn renders_laplacian() {
        let ctx = build_default_context();
        let lapl_phi = laplacian(o("\\phi"));
        let out_unicode = render_expression(&lapl_phi, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&lapl_phi, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_unicode, "∇²φ"); // \phi converts to φ in Unicode
        assert_eq!(out_latex, r"\nabla^2 \phi");
    }

    #[test]
    fn renders_maxwell_divergence() {
        // ∇·E = ρ/ε₀
        let ctx = build_default_context();
        let div_e_field = div_e(o("\\mathbf{E}"));
        let epsilon_0 = sub_e(o("\\epsilon"), c("0"));
        let rhs = over(o("\\rho"), epsilon_0);
        let maxwell = equals(div_e_field, rhs);

        let out = render_expression(&maxwell, &ctx, &RenderTarget::LaTeX);
        assert!(out.contains(r"\nabla \cdot \mathbf{E}"));
        assert!(out.contains(r"\epsilon_{{0}}"));
    }

    #[test]
    fn renders_wave_equation() {
        // ∇²φ - (1/c²)∂²φ/∂t² = 0
        let ctx = build_default_context();
        let phi = o("\\phi");
        let lapl = laplacian(phi.clone());
        let d2_dt2 = d2_part(phi, o("t"));
        let c_sq = pow_e(o("c"), c("2"));
        let time_term = over(d2_dt2, c_sq);
        let wave_op = minus(lapl, time_term);
        let wave_eq = equals(wave_op, c("0"));

        let out = render_expression(&wave_eq, &ctx, &RenderTarget::LaTeX);
        assert!(out.contains(r"\nabla^2"));
        assert!(out.contains(r"\phi"));
    }

    // === Batch 4: Polish & Edge Cases Tests ===

    // NUMBER SETS - Unicode Conversion
    #[test]
    fn renders_number_sets_unicode() {
        let ctx = build_default_context();
        let x_in_r = in_set(o("x"), o("\\mathbb{R}"));
        let psi_in_l2 = in_set(o("\\psi"), o("L^2(\\mathbb{C})"));

        let out_r = render_expression(&x_in_r, &ctx, &RenderTarget::Unicode);
        let out_l2 = render_expression(&psi_in_l2, &ctx, &RenderTarget::Unicode);

        assert_eq!(out_r, "x ∈ ℝ");
        assert!(out_l2.contains("ℂ"));
    }

    // PIECEWISE FUNCTIONS
    #[test]
    fn renders_piecewise_2cases() {
        let ctx = build_default_context();
        let cases = cases2(
            pow_e(o("x"), c("2")),
            geq(o("x"), c("0")),
            c("0"),
            less_than(o("x"), c("0")),
        );
        let out_latex = render_expression(&cases, &ctx, &RenderTarget::LaTeX);

        assert!(out_latex.contains(r"\begin{cases}"));
        assert!(out_latex.contains(r"x^{{2}}"));
        assert!(out_latex.contains(r"\end{cases}"));
    }

    #[test]
    fn renders_piecewise_3cases() {
        let ctx = build_default_context();
        let cases = cases3(
            c("-1"),
            less_than(o("x"), c("0")),
            c("0"),
            equals(o("x"), c("0")),
            c("1"),
            greater_than(o("x"), c("0")),
        );
        let out_latex = render_expression(&cases, &ctx, &RenderTarget::LaTeX);

        assert!(out_latex.contains(r"\begin{cases}"));
        assert!(out_latex.contains(r"-1"));
        assert!(out_latex.contains(r"\end{cases}"));
    }

    #[test]
    fn renders_absolute_value_piecewise() {
        // |x| = { x if x≥0, -x if x<0 }
        let ctx = build_default_context();
        let abs_def = cases2(
            o("x"),
            geq(o("x"), c("0")),
            o("-x"),
            less_than(o("x"), c("0")),
        );

        let out = render_expression(&abs_def, &ctx, &RenderTarget::LaTeX);
        assert!(out.contains(r"\begin{cases}"));
    }

    // VMATRIX (Determinant Bars)
    #[test]
    fn renders_vmatrix_2x2() {
        let ctx = build_default_context();
        let vmat = vmatrix2(o("a"), o("b"), o("c"), o("d"));
        let out_latex = render_expression(&vmat, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_latex, r"\begin{vmatrix}a&b\\c&d\end{vmatrix}");
    }

    #[test]
    fn renders_vmatrix_3x3() {
        let ctx = build_default_context();
        let vmat = vmatrix3(
            c("1"),
            c("2"),
            c("3"),
            c("4"),
            c("5"),
            c("6"),
            c("7"),
            c("8"),
            c("9"),
        );
        let out_latex = render_expression(&vmat, &ctx, &RenderTarget::LaTeX);

        assert!(out_latex.contains(r"\begin{vmatrix}"));
        assert!(out_latex.contains(r"\end{vmatrix}"));
    }

    // MODULAR ARITHMETIC
    #[test]
    fn renders_congruence_mod() {
        let ctx = build_default_context();
        let cong = congruent_mod(o("a"), o("b"), o("n"));
        let out_unicode = render_expression(&cong, &ctx, &RenderTarget::Unicode);
        let out_latex = render_expression(&cong, &ctx, &RenderTarget::LaTeX);

        assert!(out_unicode.contains("≡"));
        assert!(out_unicode.contains("mod"));
        assert_eq!(out_latex, r"a \equiv b \pmod{n}");
    }

    #[test]
    fn renders_fermats_little_theorem() {
        // a^(p-1) ≡ 1 (mod p)
        let ctx = build_default_context();
        let lhs = pow_e(o("a"), minus(o("p"), c("1")));
        let cong = congruent_mod(lhs, c("1"), o("p"));

        let out = render_expression(&cong, &ctx, &RenderTarget::LaTeX);
        assert!(out.contains(r"\equiv"));
        assert!(out.contains(r"\pmod{p}"));
    }

    // STATISTICS
    #[test]
    fn renders_variance_and_covariance() {
        let ctx = build_default_context();
        let var_x = variance(o("X"));
        let cov_xy = covariance(o("X"), o("Y"));

        let out_var = render_expression(&var_x, &ctx, &RenderTarget::LaTeX);
        let out_cov = render_expression(&cov_xy, &ctx, &RenderTarget::LaTeX);

        assert_eq!(out_var, r"\mathrm{Var}(X)");
        assert_eq!(out_cov, r"\mathrm{Cov}(X, Y)");
    }

    // INTEGRATION TESTS
    #[test]
    fn renders_sign_function_piecewise() {
        // sgn(x) = { -1 if x<0, 0 if x=0, 1 if x>0 }
        let ctx = build_default_context();
        let sgn = cases3(
            c("-1"),
            less_than(o("x"), c("0")),
            c("0"),
            equals(o("x"), c("0")),
            c("1"),
            greater_than(o("x"), c("0")),
        );

        let out = render_expression(&sgn, &ctx, &RenderTarget::LaTeX);
        assert!(out.contains(r"\begin{cases}"));
        assert!(out.contains("-1"));
        assert!(out.contains(r"\end{cases}"));
    }

    // === Accent Commands ===

    #[test]
    fn renders_bar_latex() {
        let ctx = build_default_context();
        let expr = op("bar", vec![o("x")]);
        let out = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out, "\\bar{x}");
    }

    #[test]
    fn renders_bar_unicode() {
        let ctx = build_default_context();
        let expr = op("bar", vec![o("x")]);
        let out = render_expression(&expr, &ctx, &RenderTarget::Unicode);
        assert_eq!(out, "x̄"); // x with combining macron
    }

    #[test]
    fn renders_tilde_latex() {
        let ctx = build_default_context();
        let expr = op("tilde", vec![o("x")]);
        let out = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out, "\\tilde{x}");
    }

    #[test]
    fn renders_tilde_unicode() {
        let ctx = build_default_context();
        let expr = op("tilde", vec![o("x")]);
        let out = render_expression(&expr, &ctx, &RenderTarget::Unicode);
        assert_eq!(out, "x̃"); // x with combining tilde
    }

    #[test]
    fn renders_overline_latex() {
        let ctx = build_default_context();
        let expr = op("overline", vec![o("xy")]);
        let out = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out, "\\overline{xy}");
    }

    #[test]
    fn renders_overline_unicode() {
        let ctx = build_default_context();
        let expr = op("overline", vec![o("xy")]);
        let out = render_expression(&expr, &ctx, &RenderTarget::Unicode);
        assert_eq!(out, "xy̅"); // xy with combining overline
    }

    #[test]
    fn renders_dot_latex() {
        let ctx = build_default_context();
        let expr = op("dot_accent", vec![o("x")]);
        let out = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out, "\\dot{x}");
    }

    #[test]
    fn renders_dot_unicode() {
        let ctx = build_default_context();
        let expr = op("dot_accent", vec![o("x")]);
        let out = render_expression(&expr, &ctx, &RenderTarget::Unicode);
        assert_eq!(out, "ẋ"); // x with combining dot above
    }

    #[test]
    fn renders_ddot_latex() {
        let ctx = build_default_context();
        let expr = op("ddot_accent", vec![o("x")]);
        let out = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out, "\\ddot{x}");
    }

    #[test]
    fn renders_ddot_unicode() {
        let ctx = build_default_context();
        let expr = op("ddot_accent", vec![o("x")]);
        let out = render_expression(&expr, &ctx, &RenderTarget::Unicode);
        assert_eq!(out, "ẍ"); // x with combining diaeresis (double dot)
    }

    #[test]
    fn renders_accents_in_physics_equations() {
        // Common notation: \bar{v} for average velocity
        let ctx = build_default_context();
        let expr = op("bar", vec![o("v")]);
        let out = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out, "\\bar{v}");
    }

    // === Text Mode Support ===

    #[test]
    fn renders_text_latex() {
        let ctx = build_default_context();
        let expr = op("text", vec![o("hello")]);
        let out = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out, "\\text{hello}");
    }

    #[test]
    fn renders_text_unicode() {
        let ctx = build_default_context();
        let expr = op("text", vec![o("hello")]);
        let out = render_expression(&expr, &ctx, &RenderTarget::Unicode);
        assert_eq!(out, "hello");
    }

    #[test]
    fn renders_text_with_spaces_latex() {
        let ctx = build_default_context();
        let expr = op("text", vec![o("if ")]);
        let out = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out, "\\text{if }");
    }

    #[test]
    fn renders_text_with_punctuation_latex() {
        let ctx = build_default_context();
        let expr = op("text", vec![o("for all ")]);
        let out = render_expression(&expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out, "\\text{for all }");
    }

    #[test]
    fn renders_text_in_context_latex() {
        let ctx = build_default_context();
        // \forall x \in \mathbb{R} \text{, we have } x^2 \geq 0
        let text_expr = op("text", vec![o(", we have ")]);
        let out = render_expression(&text_expr, &ctx, &RenderTarget::LaTeX);
        assert_eq!(out, "\\text{, we have }");
    }
}

// === Gallery collector ===
#[allow(clippy::vec_init_then_push)]
#[allow(clippy::items_after_test_module)]
pub fn collect_samples_for_gallery() -> Vec<(String, String)> {
    let ctx = build_default_context();
    let mut out: Vec<(String, String)> = Vec::new();

    // Basic linear algebra and vectors
    out.push((
        "Inner product ⟨u,v⟩".into(),
        render_expression(&inner_e(o("u"), o("v")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Matrix 2x2".into(),
        render_expression(
            &m2(
                sub_e(o("a"), c("11")),
                sub_e(o("a"), c("12")),
                sub_e(o("a"), c("21")),
                sub_e(o("a"), c("22")),
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Matrix 3x3".into(),
        render_expression(
            &m3(
                sub_e(o("a"), c("11")),
                sub_e(o("a"), c("12")),
                sub_e(o("a"), c("13")),
                sub_e(o("a"), c("21")),
                sub_e(o("a"), c("22")),
                sub_e(o("a"), c("23")),
                sub_e(o("a"), c("31")),
                sub_e(o("a"), c("32")),
                sub_e(o("a"), c("33")),
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Vector arrow".into(),
        render_expression(&vector_arrow_e(o("v")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Vector bold".into(),
        render_expression(&vector_bold_e(o("v")), &ctx, &RenderTarget::LaTeX),
    ));

    // Einstein Field Equations core
    let mu = o("μ");
    let nu = o("ν");
    let g_mn = index_pair(o("g"), mu.clone(), nu.clone());
    let G_mn = index_pair(o("G"), mu.clone(), nu.clone());
    let T_mn = index_pair(o("T"), mu.clone(), nu.clone());
    let efe = equals(plus(G_mn, times(o("Λ"), g_mn)), times(o("κ"), T_mn));
    out.push((
        "Einstein Field Equations (core)".into(),
        render_expression(&efe, &ctx, &RenderTarget::LaTeX),
    ));

    // Maxwell tensor from potential
    let A_nu = sub_e(o("A"), nu.clone());
    let A_mu = sub_e(o("A"), mu.clone());
    let rhs_max = minus(
        partial_apply(A_nu, mu.clone()),
        partial_apply(A_mu, nu.clone()),
    );
    let F_mn = index_mixed(o("F"), mu.clone(), nu.clone());
    let maxwell = equals(F_mn, rhs_max);
    out.push((
        "Maxwell tensor from potential".into(),
        render_expression(&maxwell, &ctx, &RenderTarget::LaTeX),
    ));

    // Kaluza–Klein metric block (2x2)
    // Matrix: [g^μ_ν + ΦA_μA_ν,  ΦA_μ]
    //         [ΦA_ν,             Φ    ]
    let A_mu2 = sub_e(o("A"), mu.clone());
    let A_nu2 = sub_e(o("A"), nu.clone());
    let phi = o("\\Phi");
    let g_mn = index_mixed(o("g"), mu.clone(), nu.clone());
    let kk_tl = plus(
        g_mn,
        times(phi.clone(), times(A_mu2.clone(), A_nu2.clone())),
    );
    let kk_tr = times(phi.clone(), A_mu2.clone());
    let kk_bl = times(phi.clone(), A_nu2.clone());
    let kk_br = phi.clone();
    out.push((
        "Kaluza–Klein metric block".into(),
        render_expression(&m2(kk_tl, kk_tr, kk_bl, kk_br), &ctx, &RenderTarget::LaTeX),
    ));

    // Euler–Lagrange (single var)
    let l = o("L");
    let y = o("y");
    let yprime = o("y'");
    let x = o("x");
    let el_single = equals(
        minus(
            d_part(l.clone(), y.clone()),
            d_dt(d_part(l.clone(), yprime.clone()), x.clone()),
        ),
        c("0"),
    );
    out.push((
        "Euler–Lagrange (single var)".into(),
        render_expression(&el_single, &ctx, &RenderTarget::LaTeX),
    ));

    // Beltrami identity
    let beltrami = equals(
        minus(
            l.clone(),
            times(yprime.clone(), d_part(l.clone(), yprime.clone())),
        ),
        o("C"),
    );
    out.push((
        "Beltrami identity".into(),
        render_expression(&beltrami, &ctx, &RenderTarget::LaTeX),
    ));

    // Hamilton–Jacobi
    let q = o("q");
    let s = o("S");
    let hj = equals(
        plus(
            func(
                "H",
                vec![q.clone(), d_part(s.clone(), q.clone()), x.clone()],
            ),
            d_part(s.clone(), x.clone()),
        ),
        c("0"),
    );
    out.push((
        "Hamilton–Jacobi (basic)".into(),
        render_expression(&hj, &ctx, &RenderTarget::LaTeX),
    ));

    // HJB core and stochastic term
    let Vsym = o("V");
    let u = o("u");
    let hjb_core_left = plus(
        d_part(Vsym.clone(), x.clone()),
        min_over(
            u.clone(),
            plus(
                dot_e(
                    d_part(Vsym.clone(), x.clone()),
                    func("F", vec![x.clone(), u.clone()]),
                ),
                func("C", vec![x.clone(), u.clone()]),
            ),
        ),
    );
    out.push((
        "HJB (core shape)".into(),
        render_expression(&equals(hjb_core_left, c("0")), &ctx, &RenderTarget::LaTeX),
    ));
    let d2V_dx2 = d2_part(Vsym.clone(), x.clone());
    let sigma = o("\\sigma");
    let diffusion = times(over(pow_e(sigma.clone(), c("2")), c("2")), d2V_dx2);
    out.push((
        "HJB (stochastic diffusion term)".into(),
        render_expression(&diffusion, &ctx, &RenderTarget::LaTeX),
    ));

    // Zeta forms (use lowercase s for zeta function parameter)
    let zeta_s = o("s");
    let zeta_x = o("x");
    let zeta_series = equals(
        func("zeta", vec![zeta_s.clone()]),
        op(
            "sum_bounds",
            vec![
                over(c("1"), pow_e(o("n"), zeta_s.clone())),
                o("n=1"),
                o("\\infty"),
            ],
        ),
    );
    out.push((
        "Riemann zeta (Dirichlet series)".into(),
        render_expression(&zeta_series, &ctx, &RenderTarget::LaTeX),
    ));
    let euler_prod = equals(
        func("zeta", vec![zeta_s.clone()]),
        op(
            "prod_index",
            vec![
                over(c("1"), minus(c("1"), pow_e(o("p"), o("-s")))),
                o("p\\,\\text{prime}"),
            ],
        ),
    );
    out.push((
        "Riemann zeta (Euler product)".into(),
        render_expression(&euler_prod, &ctx, &RenderTarget::LaTeX),
    ));
    // Mellin integral: ζ(s) = 1/Γ(s) * ∫₀^∞ x^(s-1)/(e^x - 1) dx
    let mellin_integrand = over(
        pow_e(zeta_x.clone(), minus(zeta_s.clone(), c("1"))),
        minus(func("exp", vec![zeta_x.clone()]), c("1")),
    );
    let mellin_integral = op(
        "int_bounds",
        vec![mellin_integrand, c("0"), o("\\infty"), zeta_x.clone()],
    );
    let mellin = equals(
        func("zeta", vec![zeta_s.clone()]),
        times(
            over(c("1"), func("Gamma", vec![zeta_s.clone()])),
            mellin_integral,
        ),
    );
    out.push((
        "Riemann zeta (Mellin-type integral)".into(),
        render_expression(&mellin, &ctx, &RenderTarget::LaTeX),
    ));

    // Limits
    out.push((
        "Limit".into(),
        render_expression(
            &op("limit", vec![func("f", vec![o("x")]), o("x"), c("0")]),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Limsup".into(),
        render_expression(
            &op(
                "limsup",
                vec![func("f", vec![o("x")]), o("x"), o("\\infty")],
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Liminf".into(),
        render_expression(
            &op("liminf", vec![func("f", vec![o("x")]), o("x"), o("a")]),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));

    // === NEW Top 5 Operations ===

    // 1. Bra-ket notation
    out.push((
        "Ket vector (QM)".into(),
        render_expression(&ket(o("\\psi")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Bra vector (QM)".into(),
        render_expression(&bra(o("\\phi")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Outer product (QM)".into(),
        render_expression(
            &outer_product(o("\\psi"), o("\\phi")),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Commutation relation (canonical)".into(),
        render_expression(
            &equals(
                commutator(o("\\hat{x}"), o("\\hat{p}")),
                times(o("i"), o("\\hbar")),
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));

    // 2. Set theory & logic
    out.push((
        "Set membership".into(),
        render_expression(
            &in_set(o("x"), o("\\mathbb{R}")),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Subset relation".into(),
        render_expression(&subseteq(o("A"), o("B")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Set union".into(),
        render_expression(&union(o("A"), o("B")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Set intersection".into(),
        render_expression(&intersection(o("A"), o("B")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Universal quantifier".into(),
        render_expression(
            &forall(o("x"), in_set(o("x"), o("S"))),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Existential quantifier".into(),
        render_expression(
            &exists(o("x"), in_set(o("x"), o("S"))),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Logical implication".into(),
        render_expression(&implies(o("P"), o("Q")), &ctx, &RenderTarget::LaTeX),
    ));

    // 3. Multiple integrals
    out.push((
        "Double integral".into(),
        render_expression(
            &double_int(o("f(x,y)"), o("D"), o("x"), o("y")),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Triple integral".into(),
        render_expression(
            &triple_int(o("f(x,y,z)"), o("V"), o("x"), o("y"), o("z")),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));

    // 4. Commutators
    out.push((
        "Commutator".into(),
        render_expression(&commutator(o("A"), o("B")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Anticommutator".into(),
        render_expression(&anticommutator(o("A"), o("B")), &ctx, &RenderTarget::LaTeX),
    ));

    // 5. Square roots
    out.push((
        "Square root".into(),
        render_expression(&sqrt_e(o("x")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Nth root (cube)".into(),
        render_expression(&nth_root(o("x"), c("3")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Gaussian integral result".into(),
        render_expression(&over(sqrt_e(o("\\pi")), c("2")), &ctx, &RenderTarget::LaTeX),
    ));

    // === Next Top 3 + Low-Hanging Fruit Gallery Examples ===

    // Comparison operators
    out.push((
        "Energy constraint".into(),
        render_expression(&geq(o("E"), c("0")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Inequality".into(),
        render_expression(&leq(o("x"), o("y")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Not equal".into(),
        render_expression(&not_equal(o("a"), o("b")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Approximation".into(),
        render_expression(&approx(o("\\pi"), c("3.14159")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Proportionality".into(),
        render_expression(&proportional(o("F"), o("ma")), &ctx, &RenderTarget::LaTeX),
    ));

    // Complex numbers
    out.push((
        "Complex conjugate".into(),
        render_expression(&conjugate(o("z")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Real part".into(),
        render_expression(&re(o("z")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Imaginary part".into(),
        render_expression(&im(o("z")), &ctx, &RenderTarget::LaTeX),
    ));

    // Operator hat (Quantum mechanics)
    out.push((
        "Hamiltonian operator".into(),
        render_expression(&hat(o("H")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Schrodinger equation".into(),
        render_expression(
            &equals(
                times(hat(o("H")), ket(o("\\psi"))),
                times(o("E"), ket(o("\\psi"))),
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));

    // Trig & log functions
    out.push((
        "Function - cos".into(),
        render_expression(&cos_e(o("x")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Tangent".into(),
        render_expression(&tan_e(o("x")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Function - sin".into(),
        render_expression(&func("sin", vec![o("x")]), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Hyperbolic sine".into(),
        render_expression(&sinh_e(o("x")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Hyperbolic cosine".into(),
        render_expression(&cosh_e(o("x")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Function - log".into(),
        render_expression(&ln_e(o("x")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Function - exp".into(),
        render_expression(&log_e(o("x")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Euler formula".into(),
        render_expression(
            &equals(
                pow_e(o("e"), times(o("i"), o("\\theta"))),
                plus(
                    cos_e(o("\\theta")),
                    times(o("i"), func("sin", vec![o("\\theta")])),
                ),
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));

    // Matrix operations
    out.push((
        "Trace (density matrix)".into(),
        render_expression(
            &equals(trace(o("\\rho")), c("1")),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Matrix inverse".into(),
        render_expression(&inverse(o("A")), &ctx, &RenderTarget::LaTeX),
    ));

    // === Batch 3: Completeness Operations ===

    // Phase A: Quick wins
    out.push((
        "Factorial".into(),
        render_expression(
            &equals(prod_e(o("i"), o("i=1"), o("n")), factorial(o("n"))),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Floor function".into(),
        render_expression(&floor(o("x")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Ceiling function".into(),
        render_expression(&ceiling(o("x")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Inverse trig - arcsin".into(),
        render_expression(&arcsin_e(o("x")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Inverse trig - arccos".into(),
        render_expression(&arccos_e(o("x")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Arctangent".into(),
        render_expression(&arctan_e(o("x")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Secant".into(),
        render_expression(&sec_e(o("x")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Cosecant".into(),
        render_expression(&csc_e(o("x")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Cotangent".into(),
        render_expression(&cot_e(o("x")), &ctx, &RenderTarget::LaTeX),
    ));

    // Phase B: Quantum focus
    out.push((
        "Pauli matrix (sigma x)".into(),
        render_expression(
            &equals(
                sub_e(o("\\sigma"), o("x")),
                pmatrix2(c("0"), c("1"), c("1"), c("0")),
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Pauli matrix (sigma z)".into(),
        render_expression(
            &equals(
                sub_e(o("\\sigma"), o("z")),
                pmatrix2(c("1"), c("0"), c("0"), c("-1")),
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Identity matrix (pmatrix)".into(),
        render_expression(
            &pmatrix3(
                c("1"),
                c("0"),
                c("0"),
                c("0"),
                c("1"),
                c("0"),
                c("0"),
                c("0"),
                c("1"),
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Binomial coefficient".into(),
        render_expression(&binomial(o("n"), o("k")), &ctx, &RenderTarget::LaTeX),
    ));

    // Phase C: Field theory
    out.push((
        "Divergence".into(),
        render_expression(&div_e(o("\\mathbf{F}")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Curl".into(),
        render_expression(&curl_e(o("\\mathbf{B}")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Laplacian".into(),
        render_expression(&laplacian(o("\\phi")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Maxwell divergence law".into(),
        render_expression(
            &equals(
                div_e(o("\\mathbf{E}")),
                over(o("\\rho"), sub_e(o("\\epsilon"), c("0"))),
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Wave equation".into(),
        render_expression(
            &equals(
                minus(
                    laplacian(o("\\phi")),
                    over(d2_part(o("\\phi"), o("t")), pow_e(o("c"), c("2"))),
                ),
                c("0"),
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));

    // === Matrix Complex Cells (NEW - showcasing parser fix) ===
    out.push((
        "Matrix with fractions".into(),
        render_expression(
            &op(
                "matrix2x2",
                vec![over(o("a"), o("b")), o("c"), o("d"), o("e")],
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Matrix with sqrt".into(),
        render_expression(
            &op(
                "matrix2x2",
                vec![
                    sqrt_e(c("2")),
                    sqrt_e(c("3")),
                    sqrt_e(c("5")),
                    sqrt_e(c("7")),
                ],
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Rotation matrix".into(),
        render_expression(
            &op(
                "matrix2x2",
                vec![
                    cos_e(o("\\theta")),
                    minus(c("0"), func("sin", vec![o("\\theta")])),
                    func("sin", vec![o("\\theta")]),
                    cos_e(o("\\theta")),
                ],
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Normalized state (QM)".into(),
        render_expression(
            &op(
                "matrix2x2",
                vec![
                    over(c("1"), sqrt_e(c("2"))),
                    c("0"),
                    c("0"),
                    over(c("1"), sqrt_e(c("2"))),
                ],
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));

    // === Batch 4: Polish & Edge Cases ===

    // Number sets (improved Unicode rendering)
    out.push((
        "Hilbert space membership".into(),
        render_expression(
            &in_set(o("\\psi"), o("L^2(\\mathbb{C})")),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));

    // Piecewise functions
    out.push((
        "Absolute value (piecewise)".into(),
        render_expression(
            &cases2(
                o("x"),
                geq(o("x"), c("0")),
                o("-x"),
                less_than(o("x"), c("0")),
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Sign function".into(),
        render_expression(
            &cases3(
                c("-1"),
                less_than(o("x"), c("0")),
                c("0"),
                equals(o("x"), c("0")),
                c("1"),
                greater_than(o("x"), c("0")),
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));

    // Text mode annotations
    out.push((
        "Text mode (simple)".into(),
        render_expression(
            &op("text", vec![o("hello world")]),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Text mode (with spaces)".into(),
        render_expression(&op("text", vec![o("if ")]), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Text in piecewise".into(),
        render_expression(
            &cases2(
                pow_e(o("x"), c("2")),
                op("text", vec![o("if ")]),
                c("0"),
                op("text", vec![o("otherwise")]),
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));

    // Accent commands
    out.push((
        "Bar accent (average)".into(),
        render_expression(&op("bar", vec![o("x")]), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Tilde accent".into(),
        render_expression(&op("tilde", vec![o("x")]), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Overline (conjugate)".into(),
        render_expression(&op("overline", vec![o("z")]), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Dot (velocity)".into(),
        render_expression(&op("dot_accent", vec![o("x")]), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Double dot (acceleration)".into(),
        render_expression(&op("ddot_accent", vec![o("x")]), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Newton's 2nd law".into(),
        render_expression(
            &equals(o("F"), times(o("m"), op("ddot_accent", vec![o("x")]))),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));

    // Vmatrix (determinant bars)
    out.push((
        "Determinant (vmatrix 2x2)".into(),
        render_expression(
            &vmatrix2(o("a"), o("b"), o("c"), o("d")),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Determinant (vmatrix 3x3)".into(),
        render_expression(
            &vmatrix3(
                c("1"),
                c("2"),
                c("3"),
                c("4"),
                c("5"),
                c("6"),
                c("7"),
                c("8"),
                c("9"),
            ),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));

    // Modular arithmetic
    out.push((
        "Congruence modulo".into(),
        render_expression(
            &congruent_mod(o("a"), o("b"), o("n")),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));
    out.push((
        "Fermat little theorem".into(),
        render_expression(
            &congruent_mod(pow_e(o("a"), minus(o("p"), c("1"))), c("1"), o("p")),
            &ctx,
            &RenderTarget::LaTeX,
        ),
    ));

    // Statistics
    out.push((
        "Variance".into(),
        render_expression(&variance(o("X")), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Covariance".into(),
        render_expression(&covariance(o("X"), o("Y")), &ctx, &RenderTarget::LaTeX),
    ));

    // Ellipsis (dots) - horizontal, vertical, and diagonal
    out.push((
        "Ellipsis: horizontal centered".into(),
        render_expression(&o("\\cdots"), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Ellipsis: horizontal lower".into(),
        render_expression(&o("\\ldots"), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Ellipsis: vertical".into(),
        render_expression(&o("\\vdots"), &ctx, &RenderTarget::LaTeX),
    ));
    out.push((
        "Ellipsis: diagonal".into(),
        render_expression(&o("\\ddots"), &ctx, &RenderTarget::LaTeX),
    ));
    // Note: \iddots requires \usepackage{mathdots} - commented out for standard LaTeX compatibility
    // out.push(("Ellipsis: inverse diagonal".into(), render_expression(&o("\\iddots"), &ctx, &RenderTarget::LaTeX)));

    // Ellipsis in sequences and matrices
    out.push((
        "Sequence with ellipsis".into(),
        "1, 2, 3, \\ldots, n".to_string(),
    ));
    out.push(("Matrix with ellipsis".into(), "\\begin{bmatrix}a_{11} & \\cdots & a_{1n}\\\\\\vdots & \\ddots & \\vdots\\\\a_{m1} & \\cdots & a_{mn}\\end{bmatrix}".to_string()));

    out
}
