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
//! ## Migration Guide
//!
//! To switch from render.rs to render_editor.rs in your code:
//!
//! ### Before (using render.rs):
//! ```ignore
//! use kleis::render::{build_default_context, render_editor_node, RenderTarget};
//!
//! let ctx = build_default_context();
//! let output = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
//! ```
//!
//! ### After (using render_editor.rs):
//! ```ignore
//! use kleis::render_editor::{render_editor_node, RenderTarget};
//!
//! let output = render_editor_node(&node, &RenderTarget::LaTeX);
//! ```
//!
//! The new API is simpler - no need to build a context, it uses an internal default.
//!
//! ### For server.rs specifically:
//! ```ignore
//! // Change line 551 from:
//! let output = kleis::render::render_editor_node(&node, &ctx, &target);
//!
//! // To:
//! let output = kleis::render_editor::render_editor_node(&node, &target);
//!
//! // And remove the ctx = build_default_context() line
//! ```
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
        // Implicit multiply uses juxtaposition (no operator symbol)
        self.add_template(
            "multiply",
            "{left} {right}",
            "{left} \\, {right}",
            "{left} × {right}",
            "{left} {right}", // Typst: juxtaposition, not *
            "multiply({left}, {right})",
        );
        // divide: render.rs falls back to function notation
        // so we match that for compatibility
        self.add_template(
            "divide",
            "{left}/{right}",
            "divide({left}, {right})",
            "{left}/{right}",
            "{left}/{right}",
            "divide({left}, {right})",
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
        // Alias for lt used by some AST generators
        self.add_template(
            "less_than",
            "{left} < {right}",
            "{left} < {right}",
            "{left} &lt; {right}",
            "{left} < {right}",
            "less_than({left}, {right})",
        );
        self.add_template(
            "gt",
            "{left} > {right}",
            "{left} > {right}",
            "{left} &gt; {right}",
            "{left} > {right}",
            "gt({left}, {right})",
        );
        // Alias for gt used by some AST generators
        self.add_template(
            "greater_than",
            "{left} > {right}",
            "{left} > {right}",
            "{left} &gt; {right}",
            "{left} > {right}",
            "greater_than({left}, {right})",
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
            "\\left\\{ {content} \\right\\}",
            "{{{content}}}",
            "lr(\\{ {content} \\})", // Typst: \{ and \} for literal braces
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

        // Tensor with two lower indices: args = [base, lower1, lower2]
        // Uses {subscript} for second arg, {superscript} for third arg
        self.add_template(
            "tensor_lower_pair",
            "{base}_{subscript} {superscript}",
            "{base}_{{{subscript} {superscript}}}",
            r#"{base}<sub>{subscript} {superscript}</sub>"#,
            "{base}_({subscript} {superscript})",
            "tensor_lower_pair({base}, {subscript}, {superscript})",
        );

        // Tensor with two upper and two lower indices: args = [base, upper1, upper2, lower1, lower2]
        self.add_template(
            "tensor_2up_2down",
            "{base}^{subscript} {superscript}_{idx2} {idx3}",
            "{base}^{{{subscript} {superscript}}}_{{{idx2} {idx3}}}",
            r#"{base}<sup>{subscript} {superscript}</sup><sub>{idx2} {idx3}</sub>"#,
            "{base}^({subscript} {superscript})_({idx2} {idx3})",
            "tensor_2up_2down({base}, {subscript}, {superscript}, {idx2}, {idx3})",
        );

        // Trig functions (LaTeX uses \! for small space before parens)
        self.add_template(
            "sin",
            "sin({arg})",
            "\\sin\\!({arg})",
            "sin({arg})",
            "sin({arg})",
            "sin({arg})",
        );
        self.add_template(
            "cos",
            "cos({arg})",
            "\\cos\\!({arg})",
            "cos({arg})",
            "cos({arg})",
            "cos({arg})",
        );
        self.add_template(
            "tan",
            "tan({arg})",
            "\\tan\\!({arg})",
            "tan({arg})",
            "tan({arg})",
            "tan({arg})",
        );

        // Calculus (LaTeX sqrt: {{arg}} contains {arg} for substitution)
        self.add_template(
            "sqrt",
            "√{arg}",
            "\\sqrt{{arg}}",
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
            "|{arg}\\rangle",
            "|{arg}⟩",
            "lr(| {arg} angle.r)",
            "ket({arg})",
        );
        self.add_template(
            "bra",
            "⟨{arg}|",
            "\\langle{arg}|",
            "⟨{arg}|",
            "lr(angle.l {arg} |)",
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

        // Unary operations
        self.add_template(
            "negate",
            "-{arg}",
            "-{arg}",
            "-{arg}",
            "-{arg}",
            "negate({arg})",
        );
        self.add_template(
            "abs",
            "|{arg}|",
            "\\left|{arg}\\right|",
            "|{arg}|",
            "abs({arg})",
            "abs({arg})",
        );
        self.add_template(
            "norm",
            "‖{arg}‖",
            "\\left\\|{arg}\\right\\|",
            "‖{arg}‖",
            "norm({arg})",
            "norm({arg})",
        );
        self.add_template(
            "factorial",
            "{arg}!",
            "{arg}!",
            "{arg}!",
            "{arg}!",
            "factorial({arg})",
        );

        // Derivatives - args: [function, variable]
        self.add_template(
            "d_dt",
            "d{arg}/d{right}",
            "\\frac{{d\\,{arg}}}{{d{right}}}",
            "d{arg}/d{right}",
            "(d {arg})/(d {right})",
            "Dt({arg}, {right})",
        );
        self.add_template(
            "d_part",
            "∂{arg}/∂{right}",
            "\\frac{{\\partial\\,{arg}}}{{\\partial {right}}}",
            "∂{arg}/∂{right}",
            "(diff {arg})/(diff {right})",
            "D({arg}, {right})",
        );

        // Limits
        self.add_template(
            "lim",
            "lim_{var}→{target} {body}",
            "\\lim_{{var} \\to {target}} {body}",
            "lim<sub>{var}→{target}</sub> {body}",
            "lim_({var} arrow {target}) {body}",
            "Limit({body}, {var}, {target})",
        );

        // Summation and product
        self.add_template(
            "sum_bounds",
            "Σ_{from}^{to} {body}",
            "\\sum_{{{from}}}^{{{to}}} {body}",
            "Σ<sub>{from}</sub><sup>{to}</sup> {body}",
            "sum_({from})^({to}) {body}",
            "Sum({body}, {from}, {to})",
        );
        self.add_template(
            "prod_bounds",
            "Π_{from}^{to} {body}",
            "\\prod_{{{from}}}^{{{to}}} {body}",
            "Π<sub>{from}</sub><sup>{to}</sup> {body}",
            "product_({from})^({to}) {body}",
            "Product({body}, {from}, {to})",
        );

        // More trig
        self.add_template(
            "arcsin",
            "arcsin({arg})",
            "\\arcsin\\left({arg}\\right)",
            "arcsin({arg})",
            "arcsin({arg})",
            "arcsin({arg})",
        );
        self.add_template(
            "arccos",
            "arccos({arg})",
            "\\arccos\\left({arg}\\right)",
            "arccos({arg})",
            "arccos({arg})",
            "arccos({arg})",
        );
        self.add_template(
            "arctan",
            "arctan({arg})",
            "\\arctan\\left({arg}\\right)",
            "arctan({arg})",
            "arctan({arg})",
            "arctan({arg})",
        );
        self.add_template(
            "sinh",
            "sinh({arg})",
            "\\sinh\\left({arg}\\right)",
            "sinh({arg})",
            "sinh({arg})",
            "sinh({arg})",
        );
        self.add_template(
            "cosh",
            "cosh({arg})",
            "\\cosh\\left({arg}\\right)",
            "cosh({arg})",
            "cosh({arg})",
            "cosh({arg})",
        );
        self.add_template(
            "tanh",
            "tanh({arg})",
            "\\tanh\\left({arg}\\right)",
            "tanh({arg})",
            "tanh({arg})",
            "tanh({arg})",
        );

        // Logarithms
        self.add_template(
            "log",
            "log({arg})",
            "\\log\\left({arg}\\right)",
            "log({arg})",
            "log({arg})",
            "log({arg})",
        );
        self.add_template(
            "ln",
            "ln({arg})",
            "\\ln\\left({arg}\\right)",
            "ln({arg})",
            "ln({arg})",
            "ln({arg})",
        );
        self.add_template(
            "exp",
            "exp({arg})",
            "e^{{{arg}}}",
            "e<sup>{arg}</sup>",
            "e^({arg})",
            "exp({arg})",
        );

        // Additional operations from palette
        // nth_root: args[0] = index, args[1] = radicand (matches render.rs: {left}=index, {right}=radicand)
        // But render.rs uses: LaTeX \sqrt[{right}]{{left}}, Typst root({right}, {left})
        // This means render.rs has args swapped: left=radicand, right=index
        // Our AST has: args[0]=index, args[1]=radicand, so {left}=index, {right}=radicand
        self.add_template(
            "nth_root",
            "ⁿ√({right})",
            "\\sqrt[{left}]{{{right}}}",
            "<sup>{left}</sup>√{right}",
            "root({left}, {right})",
            "nth_root({left}, {right})",
        );
        self.add_template(
            "binomial",
            "C({left},{right})",
            "\\binom{{{left}}}{{{right}}}",
            "C({left},{right})",
            "binom({left}, {right})",
            "binomial({left}, {right})",
        );
        self.add_template(
            "floor",
            "⌊{arg}⌋",
            "\\lfloor {arg} \\rfloor",
            "⌊{arg}⌋",
            "floor({arg})",
            "floor({arg})",
        );
        self.add_template(
            "ceiling",
            "⌈{arg}⌉",
            "\\lceil {arg} \\rceil",
            "⌈{arg}⌉",
            "ceil({arg})",
            "ceiling({arg})",
        );
        self.add_template(
            "approx",
            "{left} ≈ {right}",
            "{left} \\approx {right}",
            "{left} ≈ {right}",
            "{left} approx {right}",
            "approx({left}, {right})",
        );
        // Logical operators (aliases for and/or/not)
        self.add_template(
            "logical_and",
            "{left} ∧ {right}",
            "{left} \\land {right}",
            "{left} ∧ {right}",
            "{left} and {right}",
            "and({left}, {right})",
        );
        self.add_template(
            "logical_or",
            "{left} ∨ {right}",
            "{left} \\lor {right}",
            "{left} ∨ {right}",
            "{left} or {right}",
            "or({left}, {right})",
        );
        self.add_template(
            "logical_not",
            "¬{arg}",
            "\\lnot {arg}",
            "¬{arg}",
            "not {arg}",
            "not({arg})",
        );
        // Quantum operations
        self.add_template(
            "outer",
            "|{ket}⟩⟨{bra}|",
            "|{ket}\\rangle\\langle{bra}|",
            "|{ket}⟩⟨{bra}|",
            "ket({ket}) bra({bra})",
            "outer({ket}, {bra})",
        );
        self.add_template(
            "commutator",
            "[{A}, {B}]",
            "[{A}, {B}]",
            "[{A}, {B}]",
            "[{A}, {B}]",
            "commutator({A}, {B})",
        );
        self.add_template(
            "expectation",
            "⟨{operator}⟩",
            "\\langle {operator} \\rangle",
            "⟨{operator}⟩",
            "lr(angle.l {operator} angle.r)",
            "expectation({operator})",
        );
        // Vector operations
        self.add_template(
            "vector_bold",
            "{vector}",
            "\\mathbf{{{vector}}}",
            "<b>{vector}</b>",
            "bold({vector})",
            "vector({vector})",
        );
        self.add_template(
            "vector_arrow",
            "{vector}⃗",
            "\\vec{{{vector}}}",
            "{vector}⃗",
            "arrow({vector})",
            "vector({vector})",
        );
        self.add_template(
            "dot",
            "{left} · {right}",
            "{left} \\cdot {right}",
            "{left} · {right}",
            "{left} dot {right}",
            "dot({left}, {right})",
        );
        self.add_template(
            "cross",
            "{left} × {right}",
            "{left} \\times {right}",
            "{left} × {right}",
            "{left} times {right}",
            "cross({left}, {right})",
        );
        // Bracket types
        self.add_template(
            "angle_brackets",
            "⟨{content}⟩",
            "\\langle {content} \\rangle",
            "⟨{content}⟩",
            "angle.l {content} angle.r",
            "angle({content})",
        );
        // Accents
        self.add_template(
            "dot_accent",
            "{arg}̇",
            "\\dot{{{arg}}}",
            "{arg}̇",
            "dot({arg})",
            "dot({arg})",
        );
        self.add_template(
            "ddot_accent",
            "{arg}̈",
            "\\ddot{{{arg}}}",
            "{arg}̈",
            "dot.double({arg})",
            "ddot({arg})",
        );
        self.add_template(
            "bar",
            "{arg}̄",
            "\\bar{{{arg}}}",
            "{arg}̄",
            "macron({arg})",
            "bar({arg})",
        );
        self.add_template(
            "hat",
            "{arg}̂",
            "\\hat{{{arg}}}",
            "{arg}̂",
            "hat({arg})",
            "hat({arg})",
        );
        self.add_template(
            "tilde",
            "{arg}̃",
            "\\tilde{{{arg}}}",
            "{arg}̃",
            "tilde({arg})",
            "tilde({arg})",
        );
        self.add_template(
            "overline",
            "{arg}̅",
            "\\overline{{{arg}}}",
            "{arg}̅",
            "overline({arg})",
            "overline({arg})",
        );
        // Transforms (placeholder templates - can be enhanced)
        self.add_template(
            "fourier_transform",
            "ℱ[{function}]",
            "\\mathcal{{F}}[{function}]",
            "ℱ[{function}]",
            "cal(F)[{function}]",
            "fourier({function})",
        );
        self.add_template(
            "inverse_fourier",
            "ℱ⁻¹[{function}]",
            "\\mathcal{{F}}^{{-1}}[{function}]",
            "ℱ⁻¹[{function}]",
            "cal(F)^(-1)[{function}]",
            "inverse_fourier({function})",
        );
        self.add_template(
            "laplace_transform",
            "ℒ[{function}]",
            "\\mathcal{{L}}[{function}]",
            "ℒ[{function}]",
            "cal(L)[{function}]",
            "laplace({function})",
        );
        self.add_template(
            "inverse_laplace",
            "ℒ⁻¹[{function}]",
            "\\mathcal{{L}}^{{-1}}[{function}]",
            "ℒ⁻¹[{function}]",
            "cal(L)^(-1)[{function}]",
            "inverse_laplace({function})",
        );
        // convolution: args = [f, g, variable]
        self.add_template(
            "convolution",
            "({arg} ∗ {right})({superscript})",
            "({arg} \\ast {right})({superscript})",
            "({arg} ∗ {right})({superscript})",
            "({arg} ast {right})({superscript})",
            "Convolve({arg}, {right})",
        );

        // kernel_integral: args = [kernel, function, domain, variable]
        self.add_template(
            "kernel_integral",
            "∫_{idx2} {arg} {right} d{idx3}",
            "\\int_{{{idx2}}} {arg} \\, {right} \\, \\mathrm{{d}}{idx3}",
            "∫<sub>{idx2}</sub> {arg} {right} d{idx3}",
            "integral _({idx2}) {arg} {right} dif {idx3}",
            "KernelIntegral({arg}, {right}, {idx2}, {idx3})",
        );

        // greens_function: args = [point_x, source_m]
        self.add_template(
            "greens_function",
            "G({arg}, {right})",
            "G({arg}, {right})",
            "G({arg}, {right})",
            "G({arg}, {right})",
            "GreensFunction({arg}, {right})",
        );

        // =====================================================================
        // POT (Projected Ontology Theory) Operations
        // =====================================================================

        // projection: args = [function, variable]
        self.add_template(
            "projection",
            "Π[{arg}]({right})",
            "\\Pi[{arg}]({right})",
            "<span class=\"math-op\">Π</span>[{arg}]({right})",
            "Pi[{arg}]({right})",
            "Projection({arg}, {right})",
        );

        // modal_integral: args = [function, modal_space, variable]
        self.add_template(
            "modal_integral",
            "∫_{right} {arg} dμ({idx2})",
            "\\int_{{{right}}} {arg} \\, \\mathrm{{d}}\\mu({idx2})",
            "∫<sub>{right}</sub> {arg} dμ({idx2})",
            "integral _({right}) {arg} dif mu({idx2})",
            "ModalIntegral({arg}, {right}, {idx2})",
        );

        // projection_kernel: args = [spacetime_point, modal_state]
        self.add_template(
            "projection_kernel",
            "K({arg}, {right})",
            "K({arg}, {right})",
            "<span class=\"math-func\">K</span>({arg}, {right})",
            "K({arg}, {right})",
            "ProjectionKernel({arg}, {right})",
        );

        // causal_bound: args = [point]
        self.add_template(
            "causal_bound",
            "c({arg})",
            "c({arg})",
            "<span class=\"math-func\">c</span>({arg})",
            "c({arg})",
            "CausalBound({arg})",
        );

        // projection_residue: args = [projection, structure]
        self.add_template(
            "projection_residue",
            "Residue[{arg}, {right}]",
            "\\mathrm{{Residue}}[{arg}, {right}]",
            "<span class=\"math-func\">Residue</span>[{arg}, {right}]",
            "op(\"Residue\")[{arg}, {right}]",
            "ProjectionResidue({arg}, {right})",
        );

        // modal_space: args = [name]
        self.add_template(
            "modal_space",
            "𝓜_{arg}",
            "\\mathcal{{M}}_{{{arg}}}",
            "<span class=\"math-script\">𝓜</span><sub>{arg}</sub>",
            "cal(M)_({arg})",
            "ModalSpace({arg})",
        );

        // spacetime: args = [] (no arguments)
        self.add_template(
            "spacetime",
            "ℝ⁴",
            "\\mathbb{{R}}^4",
            "ℝ<sup>4</sup>",
            "bb(R)^4",
            "Spacetime",
        );

        // hont: args = [dimension]
        self.add_template(
            "hont",
            "𝓗_{arg}",
            "\\mathcal{{H}}_{{{arg}}}",
            "<span class=\"math-script\">𝓗</span><sub>{arg}</sub>",
            "cal(H)_({arg})",
            "HONT({arg})",
        );

        // outer: args = [ket, bra] - for outer product |ψ⟩⟨φ|
        self.add_template(
            "outer",
            "|{arg}⟩⟨{right}|",
            "|{arg}\\rangle\\langle{right}|",
            "|{arg}⟩⟨{right}|",
            "lr(| {arg} angle.r angle.l {right} |)",
            "outer({arg}, {right})",
        );

        // commutator: args = [A, B] - [A, B]
        self.add_template(
            "commutator",
            "[{arg}, {right}]",
            "[{arg}, {right}]",
            "[{arg}, {right}]",
            "lr([ {arg}, {right} ])",
            "commutator({arg}, {right})",
        );

        // anticommutator: args = [A, B] - {A, B}
        self.add_template(
            "anticommutator",
            "{{arg}, {right}}",
            "\\{{{arg}, {right}\\}}",
            "{{arg}, {right}}",
            "lr(\\{ {arg}, {right} \\})",
            "anticommutator({arg}, {right})",
        );
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

// Thread-local default context for the drop-in API
thread_local! {
    static DEFAULT_CONTEXT: EditorRenderContext = EditorRenderContext::new();
}

/// Render EditorNode to the specified target format (with explicit context)
pub fn render(node: &EditorNode, ctx: &EditorRenderContext, target: &RenderTarget) -> String {
    render_with_uuids(node, ctx, target, &HashMap::new())
}

/// Render EditorNode with UUID map for position tracking (with explicit context)
pub fn render_with_uuids(
    node: &EditorNode,
    ctx: &EditorRenderContext,
    target: &RenderTarget,
    node_id_to_uuid: &HashMap<String, String>,
) -> String {
    render_internal(node, ctx, target, "0", node_id_to_uuid)
}

// =============================================================================
// Drop-in Replacement API (matches render.rs signatures)
// =============================================================================

/// Drop-in replacement for render.rs::render_editor_node
///
/// Uses internal EditorRenderContext instead of GlyphContext.
/// Callers can switch from:
///   `render::render_editor_node(&node, &ctx, &target)`
/// To:
///   `render_editor::render_editor_node(&node, &target)`
///
/// Note: The GlyphContext parameter is ignored - we use our own templates.
pub fn render_editor_node(node: &EditorNode, target: &RenderTarget) -> String {
    DEFAULT_CONTEXT.with(|ctx| render(node, ctx, target))
}

/// Drop-in replacement for render.rs::render_editor_node_with_uuids
///
/// Uses internal EditorRenderContext instead of GlyphContext.
pub fn render_editor_node_with_uuids(
    node: &EditorNode,
    target: &RenderTarget,
    node_id_to_uuid: &HashMap<String, String>,
) -> String {
    DEFAULT_CONTEXT.with(|ctx| render_with_uuids(node, ctx, target, node_id_to_uuid))
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
            // Also handle LaTeX \text{...} conversion
            if s.starts_with("\\text{") && s.ends_with('}') {
                // Convert \text{True} to "True"
                let inner = &s[6..s.len() - 1];
                return format!("\"{}\"", inner);
            }
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
    let is_piecewise =
        op.name == "Piecewise" || op.name == "cases2" || op.name == "cases3" || op.name == "cases";

    // For matrix constructors
    if is_matrix_constructor {
        return render_matrix_constructor(op, ctx, target, node_id, node_id_to_uuid);
    }

    // For fixed-size matrices (matrix2x2, pmatrix3x3, etc.)
    if is_fixed_matrix {
        return render_fixed_matrix(op, ctx, target, node_id, node_id_to_uuid);
    }

    // For piecewise functions
    if is_piecewise {
        return render_piecewise(op, ctx, target, node_id, node_id_to_uuid);
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
            // Typst mat() uses a single delimiter character - the closing is inferred
            let delim = match matrix_type {
                "PMatrix" | "pmatrix" => "(",
                "VMatrix" | "vmatrix" => "|",
                "BMatrix" | "bmatrix" => "[",
                // Regular Matrix uses square brackets
                _ => "[",
            };

            let mut content = String::new();
            for r in 0..rows {
                for c in 0..cols {
                    let idx = r * cols + c;
                    if let Some(elem) = elements.get(idx) {
                        content.push_str(elem);
                    }
                    if c < cols - 1 {
                        // Use generous spacing to avoid parsing issues with #[#box[...]]
                        content.push_str(" , ");
                    }
                }
                if r < rows - 1 {
                    // Row separator with spacing
                    content.push_str(" ; ");
                }
            }
            format!("mat(delim: \"{}\", {})", delim, content)
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
// Piecewise Function Rendering
// =============================================================================

/// Render piecewise functions (cases2, cases3, Piecewise)
fn render_piecewise(
    op: &OperationData,
    ctx: &EditorRenderContext,
    target: &RenderTarget,
    node_id: &str,
    node_id_to_uuid: &HashMap<String, String>,
) -> String {
    match op.name.as_str() {
        "cases2" => render_cases_n(op, ctx, target, node_id, node_id_to_uuid, 2),
        "cases3" => render_cases_n(op, ctx, target, node_id, node_id_to_uuid, 3),
        "Piecewise" => render_piecewise_constructor(op, ctx, target, node_id, node_id_to_uuid),
        _ => render_cases_n(op, ctx, target, node_id, node_id_to_uuid, 2), // default
    }
}

/// Render cases2/cases3 format: (expr1, cond1, expr2, cond2, ...)
fn render_cases_n(
    op: &OperationData,
    ctx: &EditorRenderContext,
    target: &RenderTarget,
    node_id: &str,
    node_id_to_uuid: &HashMap<String, String>,
    n_cases: usize,
) -> String {
    // Args are interleaved: expr1, cond1, expr2, cond2, ...
    let mut cases: Vec<(String, String)> = Vec::new();

    for i in 0..n_cases {
        let expr_idx = i * 2;
        let cond_idx = i * 2 + 1;

        let expr = if let Some(e) = op.args.get(expr_idx) {
            let child_id = format!("{}.{}", node_id, expr_idx);
            render_internal(e, ctx, target, &child_id, node_id_to_uuid)
        } else {
            "□".to_string()
        };

        let cond = if let Some(c) = op.args.get(cond_idx) {
            let child_id = format!("{}.{}", node_id, cond_idx);
            render_internal(c, ctx, target, &child_id, node_id_to_uuid)
        } else {
            "□".to_string()
        };

        cases.push((expr, cond));
    }

    render_cases_content(&cases, target)
}

/// Render Piecewise(n, [exprs...], [conds...]) format
fn render_piecewise_constructor(
    op: &OperationData,
    ctx: &EditorRenderContext,
    target: &RenderTarget,
    node_id: &str,
    node_id_to_uuid: &HashMap<String, String>,
) -> String {
    // Format: Piecewise(n, [expr1, expr2, ...], [cond1, cond2, ...])
    let n_cases = if let Some(EditorNode::Const { value }) = op.args.first() {
        value.parse::<usize>().unwrap_or(2)
    } else {
        2
    };

    // Get expressions list
    let exprs: Vec<&EditorNode> = if let Some(EditorNode::List { list }) = op.args.get(1) {
        list.iter().collect()
    } else {
        vec![]
    };

    // Get conditions list
    let conds: Vec<&EditorNode> = if let Some(EditorNode::List { list }) = op.args.get(2) {
        list.iter().collect()
    } else {
        vec![]
    };

    let mut cases: Vec<(String, String)> = Vec::new();

    for i in 0..n_cases {
        let expr = if let Some(e) = exprs.get(i) {
            let child_id = format!("{}.1.{}", node_id, i);
            render_internal(e, ctx, target, &child_id, node_id_to_uuid)
        } else {
            "□".to_string()
        };

        let cond = if let Some(c) = conds.get(i) {
            let child_id = format!("{}.2.{}", node_id, i);
            render_internal(c, ctx, target, &child_id, node_id_to_uuid)
        } else {
            "□".to_string()
        };

        cases.push((expr, cond));
    }

    render_cases_content(&cases, target)
}

/// Render cases content for all targets
fn render_cases_content(cases: &[(String, String)], target: &RenderTarget) -> String {
    match target {
        RenderTarget::LaTeX => {
            let mut content = String::from("\\begin{cases}\n");
            for (expr, cond) in cases {
                content.push_str(&format!("  {} & \\text{{if }} {} \\\\\n", expr, cond));
            }
            content.push_str("\\end{cases}");
            content
        }

        RenderTarget::Typst => {
            let mut content = String::from("cases(\n");
            for (i, (expr, cond)) in cases.iter().enumerate() {
                if i > 0 {
                    content.push_str(",\n");
                }
                content.push_str(&format!("  {} \"if\" {}", expr, cond));
            }
            content.push_str("\n)");
            content
        }

        RenderTarget::HTML => {
            let mut content = String::from("<table class=\"cases\">");
            for (expr, cond) in cases {
                content.push_str(&format!("<tr><td>{}</td><td>if {}</td></tr>", expr, cond));
            }
            content.push_str("</table>");
            content
        }

        RenderTarget::Unicode | RenderTarget::Kleis => {
            let mut content = String::from("{ ");
            for (i, (expr, cond)) in cases.iter().enumerate() {
                if i > 0 {
                    content.push_str("; ");
                }
                content.push_str(&format!("{} if {}", expr, cond));
            }
            content.push_str(" }");
            content
        }
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
        result = result.replace("{n}", first); // binomial: n choose k
        result = result.replace("{index}", first); // nth_root: index-th root
        result = result.replace("{operator}", first); // expectation: ⟨operator⟩
        result = result.replace("{state}", first); // ket/bra: |state⟩, ⟨state|
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
        result = result.replace("{k}", second); // binomial: n choose k
        result = result.replace("{radicand}", second); // nth_root: radicand
                                                       // For index_mixed
        if name == "index_mixed" {
            result = result.replace("{upper}", second);
        }
    }

    // Third argument aliases
    if let Some(third) = rendered_args.get(2) {
        result = result.replace("{to}", third);
        result = result.replace("{idx2}", third);
        result = result.replace("{superscript}", third); // subsup: base_sub^super
        result = result.replace("{target}", third); // lim: limit target
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

    // -------------------------------------------------------------------------
    // Comparison tests: render_editor.rs vs render.rs
    // These verify our new renderer matches the original for common cases
    // -------------------------------------------------------------------------

    mod comparison_tests {
        use super::*;
        use crate::render::build_default_context;

        /// Helper to compare both renderers
        fn compare_renderers(node: &EditorNode, target: &RenderTarget) -> (String, String) {
            // New renderer (render_editor.rs) - using drop-in API
            let new_result = render_editor_node(node, target);

            // Old renderer (render.rs)
            let old_ctx = build_default_context();
            let old_result = crate::render::render_editor_node(node, &old_ctx, target);

            (new_result, old_result)
        }

        #[test]
        fn compare_simple_object() {
            let node = EditorNode::object("x");
            let (new, old) = compare_renderers(&node, &RenderTarget::LaTeX);
            assert_eq!(new, old, "Simple object should match");
        }

        #[test]
        fn compare_greek_letter() {
            let node = EditorNode::object("α");
            let (new, old) = compare_renderers(&node, &RenderTarget::LaTeX);
            assert_eq!(new, old, "Greek letter should match");
        }

        #[test]
        fn compare_plus_operation() {
            let node = EditorNode::operation(
                "plus",
                vec![EditorNode::object("a"), EditorNode::object("b")],
            );
            let (new, old) = compare_renderers(&node, &RenderTarget::LaTeX);
            assert_eq!(new, old, "Plus operation should match");
        }

        #[test]
        fn compare_equals_operation() {
            let node = EditorNode::operation(
                "equals",
                vec![EditorNode::object("x"), EditorNode::constant("5")],
            );
            let (new, old) = compare_renderers(&node, &RenderTarget::LaTeX);
            assert_eq!(new, old, "Equals operation should match");
        }

        #[test]
        fn compare_tensor_with_mixed_indices() {
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
            let (new, old) = compare_renderers(&tensor, &RenderTarget::LaTeX);
            assert_eq!(new, old, "Tensor with mixed indices should match");
        }

        #[test]
        fn compare_tensor_inside_equals() {
            // This is the BUG CASE - old renderer loses indexStructure
            // New renderer should preserve it
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

            let new_ctx = EditorRenderContext::new();
            let new_result = render(&equals, &new_ctx, &RenderTarget::LaTeX);

            // New renderer should have proper upper/lower indices
            assert!(
                new_result.contains("R^"),
                "New renderer should preserve upper index"
            );
            assert!(
                new_result.contains("_"),
                "New renderer should preserve lower indices"
            );

            // Note: We intentionally don't compare with old renderer here
            // because the old renderer has the bug we're fixing!
        }

        #[test]
        fn compare_sqrt() {
            let node = EditorNode::operation("sqrt", vec![EditorNode::object("x")]);
            let (new, old) = compare_renderers(&node, &RenderTarget::LaTeX);
            assert_eq!(new, old, "Sqrt should match");
        }

        #[test]
        fn compare_sin() {
            let node = EditorNode::operation("sin", vec![EditorNode::object("x")]);
            let (new, old) = compare_renderers(&node, &RenderTarget::LaTeX);
            assert_eq!(new, old, "Sin should match");
        }

        #[test]
        fn compare_fraction() {
            let node = EditorNode::operation(
                "frac",
                vec![EditorNode::object("a"), EditorNode::object("b")],
            );
            let (new, old) = compare_renderers(&node, &RenderTarget::LaTeX);
            assert_eq!(new, old, "Fraction should match");
        }

        #[test]
        fn compare_subscript() {
            let node = EditorNode::operation(
                "sub",
                vec![EditorNode::object("x"), EditorNode::constant("0")],
            );
            let (new, old) = compare_renderers(&node, &RenderTarget::LaTeX);
            assert_eq!(new, old, "Subscript should match");
        }

        #[test]
        fn compare_superscript() {
            let node = EditorNode::operation(
                "sup",
                vec![EditorNode::object("x"), EditorNode::constant("2")],
            );
            let (new, old) = compare_renderers(&node, &RenderTarget::LaTeX);
            assert_eq!(new, old, "Superscript should match");
        }

        #[test]
        fn compare_greek_letter_unicode() {
            let node = EditorNode::object("α");
            let (new, old) = compare_renderers(&node, &RenderTarget::Unicode);
            assert_eq!(new, old, "Greek letter unicode should match");
        }

        #[test]
        fn compare_tensor_kleis_target() {
            let tensor = EditorNode::tensor(
                "Γ",
                vec![
                    EditorNode::object("λ"),
                    EditorNode::object("μ"),
                    EditorNode::object("ν"),
                ],
                vec!["up", "down", "down"],
            );
            let (new, old) = compare_renderers(&tensor, &RenderTarget::Kleis);
            assert_eq!(new, old, "Tensor Kleis output should match");
        }

        #[test]
        fn compare_tensor_html_target() {
            let tensor = EditorNode::tensor(
                "g",
                vec![EditorNode::object("μ"), EditorNode::object("ν")],
                vec!["down", "down"],
            );
            let (new, old) = compare_renderers(&tensor, &RenderTarget::HTML);
            assert_eq!(new, old, "Tensor HTML output should match");
        }

        #[test]
        fn compare_multiply() {
            let node = EditorNode::operation(
                "multiply",
                vec![EditorNode::object("a"), EditorNode::object("b")],
            );
            let (new, old) = compare_renderers(&node, &RenderTarget::LaTeX);
            assert_eq!(new, old, "Multiply should match");
        }

        #[test]
        fn compare_divide() {
            let node = EditorNode::operation(
                "divide",
                vec![EditorNode::object("a"), EditorNode::object("b")],
            );
            let (new, old) = compare_renderers(&node, &RenderTarget::LaTeX);
            assert_eq!(new, old, "Divide should match");
        }
    }

    // -------------------------------------------------------------------------
    // Unit tests for render_editor.rs
    // -------------------------------------------------------------------------

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

    #[test]
    fn test_render_cases2_latex() {
        let ctx = EditorRenderContext::new();
        // |x| = { x if x >= 0; -x if x < 0 }
        let node = EditorNode::operation(
            "cases2",
            vec![
                EditorNode::object("x"),
                EditorNode::operation(
                    "geq",
                    vec![EditorNode::object("x"), EditorNode::constant("0")],
                ),
                EditorNode::operation("negate", vec![EditorNode::object("x")]),
                EditorNode::operation(
                    "lt",
                    vec![EditorNode::object("x"), EditorNode::constant("0")],
                ),
            ],
        );
        let result = render(&node, &ctx, &RenderTarget::LaTeX);
        assert!(result.contains("\\begin{cases}"));
        assert!(result.contains("\\end{cases}"));
    }

    #[test]
    fn test_render_cases3_unicode() {
        let ctx = EditorRenderContext::new();
        // sign(x) = { 1 if x > 0; 0 if x = 0; -1 if x < 0 }
        let node = EditorNode::operation(
            "cases3",
            vec![
                EditorNode::constant("1"),
                EditorNode::operation(
                    "gt",
                    vec![EditorNode::object("x"), EditorNode::constant("0")],
                ),
                EditorNode::constant("0"),
                EditorNode::operation(
                    "equals",
                    vec![EditorNode::object("x"), EditorNode::constant("0")],
                ),
                EditorNode::operation("negate", vec![EditorNode::constant("1")]),
                EditorNode::operation(
                    "lt",
                    vec![EditorNode::object("x"), EditorNode::constant("0")],
                ),
            ],
        );
        let result = render(&node, &ctx, &RenderTarget::Unicode);
        assert!(result.contains("1"));
        assert!(result.contains("0"));
        assert!(result.contains("if"));
    }

    #[test]
    fn test_render_sqrt_latex() {
        let ctx = EditorRenderContext::new();
        let node = EditorNode::operation("sqrt", vec![EditorNode::object("x")]);
        let result = render(&node, &ctx, &RenderTarget::LaTeX);
        assert!(result.contains("\\sqrt"));
    }

    #[test]
    fn test_render_integral_latex() {
        let ctx = EditorRenderContext::new();
        // ∫_0^1 x dx
        let node = EditorNode::operation(
            "int_bounds",
            vec![
                EditorNode::object("x"),
                EditorNode::constant("0"),
                EditorNode::constant("1"),
                EditorNode::object("x"),
            ],
        );
        let result = render(&node, &ctx, &RenderTarget::LaTeX);
        assert!(result.contains("\\int"));
    }
}
