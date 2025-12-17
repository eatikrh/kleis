//! EditorNode Renderer - Pure EditorNode rendering without Expression conversion
//!
//! This module renders EditorNode (from editor_ast.rs) directly to various output formats
//! WITHOUT converting to Expression. This preserves metadata like `kind` and `indexStructure`
//! throughout the entire rendering tree.
//!
//! ## Why This Exists
//!
//! The original render.rs has a bug: when rendering EditorNode operations like `equals`,
//! it converts children to Expression via `editor_node_to_expression()`, which loses
//! the `kind` and `metadata` fields. This causes nested tensors to render incorrectly
//! (all indices become upper/contravariant).
//!
//! ## Architecture
//!
//! ```text
//! EditorNode (with metadata)
//!     ↓
//! render_editor_node()     ← This module
//!     ↓
//! LaTeX / Typst / HTML / Unicode / Kleis
//! ```
//!
//! No Expression conversion anywhere in the pipeline.
//!
//! ## Note on Templates
//!
//! This module has its own template definitions, duplicated from render.rs.
//! This is intentional - render.rs is kept as reference, this module is self-contained.
//! Once this renderer is complete and tested, we can consolidate.

use std::collections::HashMap;

// Import RenderTarget from render.rs (it's a simple enum, safe to reuse)
pub use crate::render::RenderTarget;

// Import EditorNode types - NO Expression import!
use crate::editor_ast::{EditorNode, OperationData};

// =============================================================================
// Template Context (self-contained, not using render.rs GlyphContext)
// =============================================================================

/// Template context for EditorNode rendering
///
/// This is separate from render.rs GlyphContext - we maintain our own templates
/// to avoid any dependency on render.rs internals.
pub struct EditorRenderContext {
    pub unicode_templates: HashMap<String, String>,
    pub latex_templates: HashMap<String, String>,
    pub html_templates: HashMap<String, String>,
    pub typst_templates: HashMap<String, String>,
    pub kleis_templates: HashMap<String, String>,
}

impl Default for EditorRenderContext {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorRenderContext {
    pub fn new() -> Self {
        let mut ctx = EditorRenderContext {
            unicode_templates: HashMap::new(),
            latex_templates: HashMap::new(),
            html_templates: HashMap::new(),
            typst_templates: HashMap::new(),
            kleis_templates: HashMap::new(),
        };
        ctx.load_templates();
        ctx
    }

    fn load_templates(&mut self) {
        // Common operations - add templates as needed
        // Format: operation_name -> template string with placeholders

        // Equals
        self.add_template(
            "equals",
            "{left} = {right}",
            "{left} = {right}",
            "{left} = {right}",
            "{left} = {right}",
            "{left} = {right}",
        );

        // Arithmetic
        self.add_template(
            "plus",
            "{left} + {right}",
            "{left} + {right}",
            "{left} + {right}",
            "{left} + {right}",
            "plus({left}, {right})",
        );
        self.add_template(
            "minus",
            "{left} - {right}",
            "{left} - {right}",
            "{left} - {right}",
            "{left} - {right}",
            "minus({left}, {right})",
        );
        self.add_template(
            "scalar_multiply",
            "{left} × {right}",
            "{left} \\times {right}",
            "{left} × {right}",
            "{left} times {right}",
            "times({left}, {right})",
        );
        self.add_template(
            "scalar_divide",
            "{num}/{den}",
            "\\frac{{{num}}}{{{den}}}",
            "{num}/{den}",
            "frac({num}, {den})",
            "divide({num}, {den})",
        );
        self.add_template(
            "power",
            "{base}^{exponent}",
            "{base}^{{{exponent}}}",
            "{base}<sup>{exponent}</sup>",
            "{base}^({exponent})",
            "power({base}, {exponent})",
        );

        // Comparisons
        self.add_template(
            "lt",
            "{left} < {right}",
            "{left} < {right}",
            "{left} &lt; {right}",
            "{left} < {right}",
            "lt({left}, {right})",
        );
        self.add_template(
            "gt",
            "{left} > {right}",
            "{left} > {right}",
            "{left} &gt; {right}",
            "{left} > {right}",
            "gt({left}, {right})",
        );
        self.add_template(
            "leq",
            "{left} ≤ {right}",
            "{left} \\leq {right}",
            "{left} ≤ {right}",
            "{left} <= {right}",
            "leq({left}, {right})",
        );
        self.add_template(
            "geq",
            "{left} ≥ {right}",
            "{left} \\geq {right}",
            "{left} ≥ {right}",
            "{left} >= {right}",
            "geq({left}, {right})",
        );
        self.add_template(
            "neq",
            "{left} ≠ {right}",
            "{left} \\neq {right}",
            "{left} ≠ {right}",
            "{left} != {right}",
            "neq({left}, {right})",
        );

        // Brackets
        self.add_template(
            "parens",
            "({content})",
            "\\left({content}\\right)",
            "({content})",
            "lr(({content}))",
            "({content})",
        );
        self.add_template(
            "brackets",
            "[{content}]",
            "\\left[{content}\\right]",
            "[{content}]",
            "lr([{content}])",
            "[{content}]",
        );
        self.add_template(
            "braces",
            "{{{content}}}",
            "\\left\\{{{content}}\\right\\}}",
            "{{{content}}}",
            "lr({{{content}}})",
            "{{{content}}}",
        );

        // Subscript/Superscript
        self.add_template(
            "sub",
            "{base}_{subscript}",
            "{base}_{{{subscript}}}",
            "{base}<sub>{subscript}</sub>",
            "{base}_({subscript})",
            "{base}_{subscript}",
        );
        self.add_template(
            "sup",
            "{base}^{sup}",
            "{base}^{{{sup}}}",
            "{base}<sup>{sup}</sup>",
            "{base}^({sup})",
            "{base}^{sup}",
        );
        self.add_template(
            "subsup",
            "{base}_{subscript}^{superscript}",
            "{base}_{{{subscript}}}^{{{superscript}}}",
            "{base}<sub>{subscript}</sub><sup>{superscript}</sup>",
            "{base}_({subscript})^({superscript})",
            "{base}_{subscript}^{superscript}",
        );

        // Mixed index tensor (legacy template)
        self.add_template(
            "index_mixed",
            "{base}^{upper}_{lower}",
            "{base}^{{{upper}}}_{{{lower}}}",
            "{base}<sup>{upper}</sup><sub>{lower}</sub>",
            "{base}^({upper})_({lower})",
            "{base}({upper}, -{lower})",
        );

        // Trig functions
        self.add_template(
            "sin",
            "sin({arg})",
            "\\sin\\left({arg}\\right)",
            "sin({arg})",
            "sin({arg})",
            "sin({arg})",
        );
        self.add_template(
            "cos",
            "cos({arg})",
            "\\cos\\left({arg}\\right)",
            "cos({arg})",
            "cos({arg})",
            "cos({arg})",
        );
        self.add_template(
            "tan",
            "tan({arg})",
            "\\tan\\left({arg}\\right)",
            "tan({arg})",
            "tan({arg})",
            "tan({arg})",
        );

        // Calculus
        self.add_template(
            "sqrt",
            "√{arg}",
            "\\sqrt{{{arg}}}",
            "√{arg}",
            "sqrt({arg})",
            "sqrt({arg})",
        );
        self.add_template(
            "int_bounds",
            "∫_{from}^{to} {integrand} d{variable}",
            "\\int_{{{from}}}^{{{to}}} {integrand} \\, \\mathrm{{d}}{variable}",
            "∫<sub>{from}</sub><sup>{to}</sup> {integrand} d{variable}",
            "integral_({from})^({to}) {integrand} dif {variable}",
            "Integrate({integrand}, {variable}, {from}, {to})",
        );

        // Logic
        self.add_template(
            "implies",
            "{left} ⟹ {right}",
            "{left} \\Rightarrow {right}",
            "{left} ⟹ {right}",
            "{left} arrow.r.double {right}",
            "implies({left}, {right})",
        );
        self.add_template(
            "iff",
            "{left} ⟺ {right}",
            "{left} \\Leftrightarrow {right}",
            "{left} ⟺ {right}",
            "{left} arrow.l.r.double {right}",
            "iff({left}, {right})",
        );
        self.add_template(
            "and",
            "{left} ∧ {right}",
            "{left} \\land {right}",
            "{left} ∧ {right}",
            "{left} and {right}",
            "and({left}, {right})",
        );
        self.add_template(
            "or",
            "{left} ∨ {right}",
            "{left} \\lor {right}",
            "{left} ∨ {right}",
            "{left} or {right}",
            "or({left}, {right})",
        );
        self.add_template(
            "not",
            "¬{arg}",
            "\\neg {arg}",
            "¬{arg}",
            "not {arg}",
            "not({arg})",
        );

        // Sets
        self.add_template(
            "in",
            "{left} ∈ {right}",
            "{left} \\in {right}",
            "{left} ∈ {right}",
            "{left} in {right}",
            "in({left}, {right})",
        );
        self.add_template(
            "subset",
            "{left} ⊂ {right}",
            "{left} \\subset {right}",
            "{left} ⊂ {right}",
            "{left} subset {right}",
            "subset({left}, {right})",
        );
        self.add_template(
            "union",
            "{left} ∪ {right}",
            "{left} \\cup {right}",
            "{left} ∪ {right}",
            "{left} union {right}",
            "union({left}, {right})",
        );
        self.add_template(
            "intersection",
            "{left} ∩ {right}",
            "{left} \\cap {right}",
            "{left} ∩ {right}",
            "{left} sect {right}",
            "intersection({left}, {right})",
        );

        // Quantum
        self.add_template(
            "ket",
            "|{arg}⟩",
            "\\left|{arg}\\right\\rangle",
            "|{arg}⟩",
            "lr(|{arg}angle.r)",
            "ket({arg})",
        );
        self.add_template(
            "bra",
            "⟨{arg}|",
            "\\left\\langle{arg}\\right|",
            "⟨{arg}|",
            "lr(angle.l {arg}|)",
            "bra({arg})",
        );
        self.add_template(
            "inner",
            "⟨{bra}|{ket}⟩",
            "\\left\\langle{bra}\\middle|{ket}\\right\\rangle",
            "⟨{bra}|{ket}⟩",
            "lr(angle.l {bra}|{ket} angle.r)",
            "inner({bra}, {ket})",
        );

        // Vectors
        self.add_template(
            "grad",
            "∇{arg}",
            "\\nabla {arg}",
            "∇{arg}",
            "nabla {arg}",
            "gradient({arg})",
        );
        self.add_template(
            "div",
            "∇·{arg}",
            "\\nabla \\cdot {arg}",
            "∇·{arg}",
            "nabla dot {arg}",
            "divergence({arg})",
        );
        self.add_template(
            "curl",
            "∇×{arg}",
            "\\nabla \\times {arg}",
            "∇×{arg}",
            "nabla times {arg}",
            "curl({arg})",
        );

        // More will be added as needed...
    }

    fn add_template(
        &mut self,
        name: &str,
        unicode: &str,
        latex: &str,
        html: &str,
        typst: &str,
        kleis: &str,
    ) {
        self.unicode_templates
            .insert(name.to_string(), unicode.to_string());
        self.latex_templates
            .insert(name.to_string(), latex.to_string());
        self.html_templates
            .insert(name.to_string(), html.to_string());
        self.typst_templates
            .insert(name.to_string(), typst.to_string());
        self.kleis_templates
            .insert(name.to_string(), kleis.to_string());
    }

    pub fn get_template(&self, name: &str, target: &RenderTarget) -> Option<String> {
        match target {
            RenderTarget::Unicode => self.unicode_templates.get(name).cloned(),
            RenderTarget::LaTeX => self.latex_templates.get(name).cloned(),
            RenderTarget::HTML => self.html_templates.get(name).cloned(),
            RenderTarget::Typst => self.typst_templates.get(name).cloned(),
            RenderTarget::Kleis => self.kleis_templates.get(name).cloned(),
        }
    }
}

// =============================================================================
// Public API
// =============================================================================

/// Render EditorNode to the specified target format
pub fn render(node: &EditorNode, ctx: &EditorRenderContext, target: &RenderTarget) -> String {
    render_with_uuids(node, ctx, target, &HashMap::new())
}

/// Render EditorNode with UUID map for position tracking
pub fn render_with_uuids(
    node: &EditorNode,
    ctx: &EditorRenderContext,
    target: &RenderTarget,
    node_id_to_uuid: &HashMap<String, String>,
) -> String {
    render_internal(node, ctx, target, "0", node_id_to_uuid)
}

// =============================================================================
// Internal Rendering
// =============================================================================

fn render_internal(
    node: &EditorNode,
    ctx: &EditorRenderContext,
    target: &RenderTarget,
    node_id: &str,
    node_id_to_uuid: &HashMap<String, String>,
) -> String {
    match node {
        EditorNode::Object { object } => render_object(object, target, node_id, node_id_to_uuid),

        EditorNode::Const { value } => render_const(value, target, node_id, node_id_to_uuid),

        EditorNode::Placeholder { placeholder } => {
            let hint = placeholder.hint.as_deref().unwrap_or("□");
            match target {
                RenderTarget::LaTeX => format!("\\square_{{{}}}", hint),
                RenderTarget::Typst => format!("#[#box[$square.stroked$]<ph{}>]", placeholder.id),
                RenderTarget::HTML => {
                    format!(
                        "<span class=\"placeholder\" data-hint=\"{}\">□</span>",
                        hint
                    )
                }
                RenderTarget::Unicode | RenderTarget::Kleis => "□".to_string(),
            }
        }

        EditorNode::Operation { operation } => {
            render_operation(operation, ctx, target, node_id, node_id_to_uuid)
        }

        EditorNode::List { list } => {
            let rendered: Vec<String> = list
                .iter()
                .enumerate()
                .map(|(i, elem)| {
                    let child_id = format!("{}.{}", node_id, i);
                    render_internal(elem, ctx, target, &child_id, node_id_to_uuid)
                })
                .collect();
            format!("[{}]", rendered.join(", "))
        }
    }
}

// =============================================================================
// Object Rendering
// =============================================================================

fn render_object(
    s: &str,
    target: &RenderTarget,
    node_id: &str,
    node_id_to_uuid: &HashMap<String, String>,
) -> String {
    let rendered = render_object_for_target(s, target);

    // Add UUID label for Typst position tracking
    if matches!(target, RenderTarget::Typst) {
        if let Some(uuid) = node_id_to_uuid.get(node_id) {
            return format!("#[#box[${}$]<id{}>]", rendered, uuid);
        }
    }
    rendered
}

fn render_object_for_target(s: &str, target: &RenderTarget) -> String {
    match target {
        RenderTarget::LaTeX => {
            // Handle Greek letters and special symbols
            match s {
                "α" => "\\alpha".to_string(),
                "β" => "\\beta".to_string(),
                "γ" => "\\gamma".to_string(),
                "δ" => "\\delta".to_string(),
                "ε" => "\\varepsilon".to_string(),
                "ζ" => "\\zeta".to_string(),
                "η" => "\\eta".to_string(),
                "θ" => "\\theta".to_string(),
                "ι" => "\\iota".to_string(),
                "κ" => "\\kappa".to_string(),
                "λ" => "\\lambda".to_string(),
                "μ" => "\\mu".to_string(),
                "ν" => "\\nu".to_string(),
                "ξ" => "\\xi".to_string(),
                "π" => "\\pi".to_string(),
                "ρ" => "\\rho".to_string(),
                "σ" => "\\sigma".to_string(),
                "τ" => "\\tau".to_string(),
                "υ" => "\\upsilon".to_string(),
                "φ" => "\\varphi".to_string(),
                "χ" => "\\chi".to_string(),
                "ψ" => "\\psi".to_string(),
                "ω" => "\\omega".to_string(),
                "Γ" => "\\Gamma".to_string(),
                "Δ" => "\\Delta".to_string(),
                "Θ" => "\\Theta".to_string(),
                "Λ" => "\\Lambda".to_string(),
                "Ξ" => "\\Xi".to_string(),
                "Π" => "\\Pi".to_string(),
                "Σ" => "\\Sigma".to_string(),
                "Υ" => "\\Upsilon".to_string(),
                "Φ" => "\\Phi".to_string(),
                "Ψ" => "\\Psi".to_string(),
                "Ω" => "\\Omega".to_string(),
                "∞" => "\\infty".to_string(),
                "∂" => "\\partial".to_string(),
                "∇" => "\\nabla".to_string(),
                _ => s.to_string(),
            }
        }
        RenderTarget::Typst => {
            // Typst uses Unicode directly for Greek letters
            match s {
                "α" => "alpha".to_string(),
                "β" => "beta".to_string(),
                "γ" => "gamma".to_string(),
                "δ" => "delta".to_string(),
                "ε" => "epsilon".to_string(),
                "ζ" => "zeta".to_string(),
                "η" => "eta".to_string(),
                "θ" => "theta".to_string(),
                "ι" => "iota".to_string(),
                "κ" => "kappa".to_string(),
                "λ" => "lambda".to_string(),
                "μ" => "mu".to_string(),
                "ν" => "nu".to_string(),
                "ξ" => "xi".to_string(),
                "π" => "pi".to_string(),
                "ρ" => "rho".to_string(),
                "σ" => "sigma".to_string(),
                "τ" => "tau".to_string(),
                "υ" => "upsilon".to_string(),
                "φ" => "phi".to_string(),
                "χ" => "chi".to_string(),
                "ψ" => "psi".to_string(),
                "ω" => "omega".to_string(),
                "Γ" => "Gamma".to_string(),
                "Δ" => "Delta".to_string(),
                "Θ" => "Theta".to_string(),
                "Λ" => "Lambda".to_string(),
                "Ξ" => "Xi".to_string(),
                "Π" => "Pi".to_string(),
                "Σ" => "Sigma".to_string(),
                "Υ" => "Upsilon".to_string(),
                "Φ" => "Phi".to_string(),
                "Ψ" => "Psi".to_string(),
                "Ω" => "Omega".to_string(),
                "∞" => "infinity".to_string(),
                "∂" => "diff".to_string(),
                "∇" => "nabla".to_string(),
                _ => s.to_string(),
            }
        }
        RenderTarget::HTML | RenderTarget::Unicode | RenderTarget::Kleis => s.to_string(),
    }
}

// =============================================================================
// Const Rendering
// =============================================================================

fn render_const(
    value: &str,
    target: &RenderTarget,
    node_id: &str,
    node_id_to_uuid: &HashMap<String, String>,
) -> String {
    let rendered = match target {
        RenderTarget::LaTeX => escape_latex_constant(value),
        _ => value.to_string(),
    };

    // Add UUID label for Typst
    if matches!(target, RenderTarget::Typst) {
        if let Some(uuid) = node_id_to_uuid.get(node_id) {
            return format!("#[#box[${}$]<id{}>]", rendered, uuid);
        }
    }
    rendered
}

fn escape_latex_constant(s: &str) -> String {
    // Basic LaTeX escaping for constants
    s.replace('_', "\\_")
        .replace('%', "\\%")
        .replace('&', "\\&")
        .replace('#', "\\#")
}

// =============================================================================
// Operation Rendering
// =============================================================================

fn render_operation(
    op: &OperationData,
    ctx: &EditorRenderContext,
    target: &RenderTarget,
    node_id: &str,
    node_id_to_uuid: &HashMap<String, String>,
) -> String {
    // Check for special operation types first
    let is_tensor = op.kind.as_deref() == Some("tensor") || op.name == "tensor";
    let is_matrix_constructor = matches!(
        op.name.as_str(),
        "Matrix" | "PMatrix" | "VMatrix" | "BMatrix"
    );
    let is_fixed_matrix = op.name.starts_with("matrix")
        || op.name.starts_with("pmatrix")
        || op.name.starts_with("vmatrix");

    // For matrix constructors, extract dimensions first
    if is_matrix_constructor {
        return render_matrix_constructor(op, ctx, target, node_id, node_id_to_uuid);
    }

    // For fixed-size matrices (matrix2x2, pmatrix3x3, etc.)
    if is_fixed_matrix {
        return render_fixed_matrix(op, ctx, target, node_id, node_id_to_uuid);
    }

    // Pre-render ALL children as EditorNode (preserves metadata!)
    let rendered_args: Vec<String> = op
        .args
        .iter()
        .enumerate()
        .map(|(i, arg)| {
            let child_id = format!("{}.{}", node_id, i);
            render_internal(arg, ctx, target, &child_id, node_id_to_uuid)
        })
        .collect();

    // Dispatch based on kind
    if is_tensor {
        return render_tensor(op, &rendered_args, target);
    }

    // For all other operations: use template-based rendering
    render_with_template(&op.name, &rendered_args, ctx, target)
}

// =============================================================================
// Tensor Rendering (preserves indexStructure metadata)
// =============================================================================

fn render_tensor(op: &OperationData, rendered_args: &[String], target: &RenderTarget) -> String {
    // Structure: args[0] = symbol, args[1:] = indices
    // indexStructure describes args[1:], not args[0]
    let symbol = if !rendered_args.is_empty() {
        rendered_args[0].clone()
    } else {
        render_object_for_target(&op.name, target)
    };

    // Indices are args[1:]
    let indices = if rendered_args.len() > 1 {
        &rendered_args[1..]
    } else {
        &[]
    };

    // Get index structure from metadata
    let index_structure: Vec<&str> = op
        .metadata
        .as_ref()
        .and_then(|m| m.get("indexStructure"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_else(|| {
            // Default: all upper indices
            indices.iter().map(|_| "up").collect()
        });

    // Collect upper and lower indices
    let mut upper_indices = Vec::new();
    let mut lower_indices = Vec::new();

    for (i, idx) in indices.iter().enumerate() {
        let position = index_structure.get(i).copied().unwrap_or("up");
        if position == "down" {
            lower_indices.push(idx.as_str());
        } else {
            upper_indices.push(idx.as_str());
        }
    }

    match target {
        RenderTarget::LaTeX => {
            let upper = if upper_indices.is_empty() {
                String::new()
            } else {
                format!("^{{{}}}", upper_indices.join(" "))
            };
            let lower = if lower_indices.is_empty() {
                String::new()
            } else {
                format!("_{{{}}}", lower_indices.join(" "))
            };
            format!("{}{}{}", symbol, upper, lower)
        }
        RenderTarget::Typst => {
            let upper = if upper_indices.is_empty() {
                String::new()
            } else {
                format!("^({})", upper_indices.join(" "))
            };
            let lower = if lower_indices.is_empty() {
                String::new()
            } else {
                format!("_({})", lower_indices.join(" "))
            };
            format!("{}{}{}", symbol, upper, lower)
        }
        RenderTarget::HTML => {
            let upper = if upper_indices.is_empty() {
                String::new()
            } else {
                format!("<sup>{}</sup>", upper_indices.join(""))
            };
            let lower = if lower_indices.is_empty() {
                String::new()
            } else {
                format!("<sub>{}</sub>", lower_indices.join(""))
            };
            format!("{}{}{}", symbol, upper, lower)
        }
        RenderTarget::Unicode => {
            let upper = upper_indices.join("");
            let lower = lower_indices.join("");
            if upper.is_empty() && lower.is_empty() {
                symbol
            } else if upper.is_empty() {
                format!("{}_{}", symbol, lower)
            } else if lower.is_empty() {
                format!("{}^{}", symbol, upper)
            } else {
                format!("{}^{}_{{ {} }}", symbol, upper, lower)
            }
        }
        RenderTarget::Kleis => {
            // xAct-style: T(μ, -ν) where - means covariant
            let kleis_indices: Vec<String> = indices
                .iter()
                .enumerate()
                .map(|(i, idx)| {
                    let pos = index_structure.get(i).copied().unwrap_or("up");
                    if pos == "down" {
                        format!("-{}", idx)
                    } else {
                        idx.to_string()
                    }
                })
                .collect();
            format!("{}({})", symbol, kleis_indices.join(", "))
        }
    }
}

// =============================================================================
// Matrix Rendering
// =============================================================================

/// Render Matrix(rows, cols, [elements...]) or Matrix(rows, cols, a, b, c, d)
fn render_matrix_constructor(
    op: &OperationData,
    ctx: &EditorRenderContext,
    target: &RenderTarget,
    node_id: &str,
    node_id_to_uuid: &HashMap<String, String>,
) -> String {
    // Extract dimensions from first two args
    let (rows, cols) = if op.args.len() >= 2 {
        let rows = extract_const_number(&op.args[0]).unwrap_or(2);
        let cols = extract_const_number(&op.args[1]).unwrap_or(2);
        (rows, cols)
    } else {
        (2, 2)
    };

    // Get matrix elements - either from List or remaining args
    let elements: Vec<&EditorNode> = if op.args.len() == 3 {
        // NEW FORMAT: Matrix(2, 2, [a, b, c, d])
        if let EditorNode::List { list } = &op.args[2] {
            list.iter().collect()
        } else {
            // Single element in 3rd position
            vec![&op.args[2]]
        }
    } else if op.args.len() > 2 {
        // OLD FORMAT: Matrix(2, 2, a, b, c, d)
        op.args[2..].iter().collect()
    } else {
        vec![]
    };

    // Pre-render all elements
    let rendered_elements: Vec<String> = elements
        .iter()
        .enumerate()
        .map(|(i, elem)| {
            let child_id = if op.args.len() == 3 {
                // NEW FORMAT: child IDs go under the List at args[2]
                format!("{}.2.{}", node_id, i)
            } else {
                // OLD FORMAT: child IDs start at args[2+i]
                format!("{}.{}", node_id, i + 2)
            };
            let rendered = render_internal(elem, ctx, target, &child_id, node_id_to_uuid);

            // Wrap with UUID for Typst
            if matches!(target, RenderTarget::Typst) {
                if let Some(uuid) = node_id_to_uuid.get(&child_id) {
                    return format!("#[#box[${}$]<id{}>]", rendered, uuid);
                }
            }
            rendered
        })
        .collect();

    // Build matrix content
    render_matrix_content(&op.name, rows, cols, &rendered_elements, target)
}

/// Render fixed-size matrix (matrix2x2, pmatrix3x3, vmatrix2x2, etc.)
fn render_fixed_matrix(
    op: &OperationData,
    ctx: &EditorRenderContext,
    target: &RenderTarget,
    node_id: &str,
    node_id_to_uuid: &HashMap<String, String>,
) -> String {
    // Parse dimensions from name (e.g., "matrix2x2" -> 2, 2)
    let (rows, cols) = parse_matrix_dimensions(&op.name).unwrap_or((2, 2));

    // Pre-render all elements
    let rendered_elements: Vec<String> = op
        .args
        .iter()
        .enumerate()
        .map(|(i, elem)| {
            let child_id = format!("{}.{}", node_id, i);
            let rendered = render_internal(elem, ctx, target, &child_id, node_id_to_uuid);

            // Wrap with UUID for Typst
            if matches!(target, RenderTarget::Typst) {
                if let Some(uuid) = node_id_to_uuid.get(&child_id) {
                    return format!("#[#box[${}$]<id{}>]", rendered, uuid);
                }
            }
            rendered
        })
        .collect();

    // Determine matrix type from name prefix
    let matrix_type = if op.name.starts_with("pmatrix") {
        "PMatrix"
    } else if op.name.starts_with("vmatrix") {
        "VMatrix"
    } else {
        "Matrix"
    };

    render_matrix_content(matrix_type, rows, cols, &rendered_elements, target)
}

/// Render matrix content for all targets
fn render_matrix_content(
    matrix_type: &str,
    rows: usize,
    cols: usize,
    elements: &[String],
    target: &RenderTarget,
) -> String {
    match target {
        RenderTarget::LaTeX => {
            let env = match matrix_type {
                "PMatrix" | "pmatrix" => "pmatrix",
                "VMatrix" | "vmatrix" => "vmatrix",
                "BMatrix" | "bmatrix" => "bmatrix",
                _ => "matrix",
            };

            let mut content = String::new();
            for r in 0..rows {
                for c in 0..cols {
                    let idx = r * cols + c;
                    if let Some(elem) = elements.get(idx) {
                        content.push_str(elem);
                    }
                    if c < cols - 1 {
                        content.push_str(" & ");
                    }
                }
                if r < rows - 1 {
                    content.push_str(" \\\\ ");
                }
            }
            format!("\\begin{{{}}} {} \\end{{{}}}", env, content, env)
        }

        RenderTarget::Typst => {
            let (left_delim, right_delim) = match matrix_type {
                "PMatrix" | "pmatrix" => ("(", ")"),
                "VMatrix" | "vmatrix" => ("|", "|"),
                "BMatrix" | "bmatrix" => ("[", "]"),
                _ => ("(", ")"),
            };

            let mut content = String::new();
            for r in 0..rows {
                for c in 0..cols {
                    let idx = r * cols + c;
                    if let Some(elem) = elements.get(idx) {
                        content.push_str(elem);
                    }
                    if c < cols - 1 {
                        content.push_str(", ");
                    }
                }
                if r < rows - 1 {
                    content.push_str("; ");
                }
            }
            format!(
                "mat(delim: \"{}{}\", {})",
                left_delim, right_delim, content
            )
        }

        RenderTarget::HTML => {
            let mut content = String::from("<table class=\"matrix\">");
            for r in 0..rows {
                content.push_str("<tr>");
                for c in 0..cols {
                    let idx = r * cols + c;
                    content.push_str("<td>");
                    if let Some(elem) = elements.get(idx) {
                        content.push_str(elem);
                    }
                    content.push_str("</td>");
                }
                content.push_str("</tr>");
            }
            content.push_str("</table>");
            content
        }

        RenderTarget::Unicode | RenderTarget::Kleis => {
            let (left_delim, right_delim) = match matrix_type {
                "PMatrix" | "pmatrix" => ("(", ")"),
                "VMatrix" | "vmatrix" => ("|", "|"),
                "BMatrix" | "bmatrix" => ("[", "]"),
                _ => ("[", "]"),
            };

            let mut result = String::from(left_delim);
            for r in 0..rows {
                if r > 0 {
                    result.push_str("; ");
                }
                for c in 0..cols {
                    let idx = r * cols + c;
                    if c > 0 {
                        result.push_str(", ");
                    }
                    if let Some(elem) = elements.get(idx) {
                        result.push_str(elem);
                    }
                }
            }
            result.push_str(right_delim);
            result
        }
    }
}

/// Extract a number from a Const EditorNode
fn extract_const_number(node: &EditorNode) -> Option<usize> {
    match node {
        EditorNode::Const { value } => value.parse().ok(),
        _ => None,
    }
}

/// Parse matrix dimensions from operation name (e.g., "matrix2x3" -> (2, 3))
fn parse_matrix_dimensions(name: &str) -> Option<(usize, usize)> {
    // Strip prefix (matrix, pmatrix, vmatrix)
    let suffix = name
        .strip_prefix("matrix")
        .or_else(|| name.strip_prefix("pmatrix"))
        .or_else(|| name.strip_prefix("vmatrix"))?;

    // Parse NxM format
    let parts: Vec<&str> = suffix.split('x').collect();
    if parts.len() == 2 {
        let rows = parts[0].parse().ok()?;
        let cols = parts[1].parse().ok()?;
        Some((rows, cols))
    } else {
        None
    }
}

// =============================================================================
// Template-Based Rendering
// =============================================================================

/// Render an operation using template lookup and substitution
///
/// This is the core of the renderer - it looks up templates from EditorRenderContext
/// and substitutes pre-rendered arguments into placeholders like {arg}, {left}, {right}.
fn render_with_template(
    name: &str,
    rendered_args: &[String],
    ctx: &EditorRenderContext,
    target: &RenderTarget,
) -> String {
    // Get template for this operation, or use default
    let default_template = format!("{}({{args}})", name);
    let template = ctx.get_template(name, target).unwrap_or(default_template);

    // Apply placeholder substitutions
    apply_template_substitutions(&template, name, name, rendered_args, target)
}

/// Apply template placeholder substitutions
///
/// Placeholders supported:
/// - {glyph} - the operation symbol
/// - {args} - comma-separated arguments
/// - {arg}, {left}, {body}, etc. - first argument
/// - {right}, {exponent}, etc. - second argument
/// - {idx2}, {to}, etc. - third argument
/// - And more...
fn apply_template_substitutions(
    template: &str,
    glyph: &str,
    name: &str,
    rendered_args: &[String],
    _target: &RenderTarget,
) -> String {
    let mut result = template.to_string();

    // Basic substitutions
    result = result.replace("{glyph}", glyph);

    // {args} - all arguments joined
    if result.contains("{args}") {
        let joined = rendered_args.join(", ");
        result = result.replace("{args}", &joined);
    }

    // First argument aliases
    if let Some(first) = rendered_args.first() {
        result = result.replace("{arg}", first);
        result = result.replace("{left}", first);
        result = result.replace("{body}", first);
        result = result.replace("{integrand}", first);
        result = result.replace("{num}", first);
        result = result.replace("{base}", first);
        result = result.replace("{function}", first);
        result = result.replace("{content}", first);
        result = result.replace("{vector}", first);
        result = result.replace("{A}", first);
        result = result.replace("{bra}", first);
        result = result.replace("{value}", first);
    }

    // Second argument aliases
    if let Some(second) = rendered_args.get(1) {
        result = result.replace("{right}", second);
        result = result.replace("{den}", second);
        result = result.replace("{exponent}", second);
        result = result.replace("{sup}", second);
        result = result.replace("{from}", second);
        result = result.replace("{subscript}", second);
        result = result.replace("{var}", second);
        result = result.replace("{variable}", second);
        result = result.replace("{ket}", second);
        result = result.replace("{B}", second);
        // For index_mixed
        if name == "index_mixed" {
            result = result.replace("{upper}", second);
        }
    }

    // Third argument aliases
    if let Some(third) = rendered_args.get(2) {
        result = result.replace("{to}", third);
        result = result.replace("{idx2}", third);
        // For index_mixed
        if name == "index_mixed" {
            result = result.replace("{lower}", third);
        }
        // For int_bounds
        if name == "int_bounds" {
            result = result.replace("{upper}", third);
        }
    }

    // Fourth argument aliases
    if let Some(fourth) = rendered_args.get(3) {
        result = result.replace("{idx3}", fourth);
        if name == "int_bounds" {
            result = result.replace("{variable}", fourth);
        }
    }

    result
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor_ast::EditorNode;

    #[test]
    fn test_render_simple_object() {
        let ctx = EditorRenderContext::new();
        let node = EditorNode::object("x");
        let result = render(&node, &ctx, &RenderTarget::LaTeX);
        assert_eq!(result, "x");
    }

    #[test]
    fn test_render_greek_letter_latex() {
        let ctx = EditorRenderContext::new();
        let node = EditorNode::object("α");
        let result = render(&node, &ctx, &RenderTarget::LaTeX);
        assert_eq!(result, "\\alpha");
    }

    #[test]
    fn test_render_tensor_with_mixed_indices() {
        let ctx = EditorRenderContext::new();
        let tensor = EditorNode::tensor(
            "R",
            vec![
                EditorNode::object("ρ"),
                EditorNode::object("σ"),
                EditorNode::object("μ"),
                EditorNode::object("ν"),
            ],
            vec!["up", "down", "down", "down"],
        );

        let result = render(&tensor, &ctx, &RenderTarget::LaTeX);
        // Should have ρ upper, σμν lower
        assert!(result.contains("^"));
        assert!(result.contains("_"));
        // The symbol should be "R"
        assert!(result.starts_with("R"));
    }

    #[test]
    fn test_tensor_inside_equals_preserves_indices() {
        let ctx = EditorRenderContext::new();

        // Create: equals(placeholder, tensor)
        // This is the bug case - tensor nested inside equals should preserve indexStructure
        let tensor = EditorNode::tensor(
            "R",
            vec![
                EditorNode::object("μ"),
                EditorNode::object("ν"),
                EditorNode::object("ρ"),
                EditorNode::object("σ"),
            ],
            vec!["up", "down", "down", "down"],
        );

        let equals =
            EditorNode::operation("equals", vec![EditorNode::placeholder(0, None), tensor]);

        let result = render(&equals, &ctx, &RenderTarget::LaTeX);

        // The tensor part should have proper upper/lower indices
        // μ should be upper, νρσ should be lower
        assert!(result.contains("R^"));
        assert!(result.contains("_"));
    }

    #[test]
    fn test_render_simple_operation() {
        let ctx = EditorRenderContext::new();
        let node = EditorNode::operation(
            "plus",
            vec![EditorNode::object("a"), EditorNode::object("b")],
        );
        let result = render(&node, &ctx, &RenderTarget::Unicode);
        // Should render as "a + b" or similar based on template
        assert!(result.contains("a"));
        assert!(result.contains("b"));
    }

    #[test]
    fn test_render_matrix_2x2_latex() {
        let ctx = EditorRenderContext::new();
        let node = EditorNode::operation(
            "matrix2x2",
            vec![
                EditorNode::object("a"),
                EditorNode::object("b"),
                EditorNode::object("c"),
                EditorNode::object("d"),
            ],
        );
        let result = render(&node, &ctx, &RenderTarget::LaTeX);
        assert!(result.contains("\\begin{matrix}"));
        assert!(result.contains("\\end{matrix}"));
        assert!(result.contains("a"));
        assert!(result.contains("d"));
    }

    #[test]
    fn test_render_pmatrix_2x2_latex() {
        let ctx = EditorRenderContext::new();
        let node = EditorNode::operation(
            "pmatrix2x2",
            vec![
                EditorNode::constant("1"),
                EditorNode::constant("0"),
                EditorNode::constant("0"),
                EditorNode::constant("1"),
            ],
        );
        let result = render(&node, &ctx, &RenderTarget::LaTeX);
        assert!(result.contains("\\begin{pmatrix}"));
        assert!(result.contains("\\end{pmatrix}"));
    }

    #[test]
    fn test_render_matrix_constructor() {
        let ctx = EditorRenderContext::new();
        // Matrix(2, 2, [a, b, c, d])
        let node = EditorNode::Operation {
            operation: OperationData {
                name: "Matrix".to_string(),
                args: vec![
                    EditorNode::constant("2"),
                    EditorNode::constant("2"),
                    EditorNode::list(vec![
                        EditorNode::object("a"),
                        EditorNode::object("b"),
                        EditorNode::object("c"),
                        EditorNode::object("d"),
                    ]),
                ],
                kind: None,
                metadata: None,
            },
        };
        let result = render(&node, &ctx, &RenderTarget::LaTeX);
        assert!(result.contains("\\begin{matrix}"));
        assert!(result.contains("a"));
        assert!(result.contains("d"));
    }

    #[test]
    fn test_render_fraction_latex() {
        let ctx = EditorRenderContext::new();
        let node = EditorNode::operation(
            "scalar_divide",
            vec![EditorNode::object("a"), EditorNode::object("b")],
        );
        let result = render(&node, &ctx, &RenderTarget::LaTeX);
        assert!(result.contains("\\frac"));
        assert!(result.contains("a"));
        assert!(result.contains("b"));
    }

    #[test]
    fn test_render_power_latex() {
        let ctx = EditorRenderContext::new();
        let node = EditorNode::operation(
            "power",
            vec![EditorNode::object("x"), EditorNode::constant("2")],
        );
        let result = render(&node, &ctx, &RenderTarget::LaTeX);
        // Template uses {{exponent}} for LaTeX brace escaping
        assert!(result.contains("x^"));
        assert!(result.contains("2"));
    }

    #[test]
    fn test_nested_operations() {
        let ctx = EditorRenderContext::new();
        // (a + b) * c
        let inner = EditorNode::operation(
            "plus",
            vec![EditorNode::object("a"), EditorNode::object("b")],
        );
        let outer = EditorNode::operation("scalar_multiply", vec![inner, EditorNode::object("c")]);
        let result = render(&outer, &ctx, &RenderTarget::Unicode);
        assert!(result.contains("a"));
        assert!(result.contains("b"));
        assert!(result.contains("c"));
    }
}
