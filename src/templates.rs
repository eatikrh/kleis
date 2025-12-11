//! Template insertion functions for structural editor
//!
//! This module provides functions to create mathematical operations with placeholders.
//! Each function corresponds to a template that can be inserted via the palette.

use crate::ast::Expression;

/// Counter for generating unique placeholder IDs
static mut PLACEHOLDER_COUNTER: usize = 0;

fn next_id() -> usize {
    unsafe {
        PLACEHOLDER_COUNTER += 1;
        PLACEHOLDER_COUNTER
    }
}

/// Reset the placeholder counter (useful for testing)
pub fn reset_placeholder_counter() {
    unsafe {
        PLACEHOLDER_COUNTER = 0;
    }
}

// === Basic Operations ===

/// Fraction: numerator / denominator
pub fn template_fraction() -> Expression {
    Expression::operation(
        "scalar_divide",
        vec![
            Expression::placeholder(next_id(), "numerator"),
            Expression::placeholder(next_id(), "denominator"),
        ],
    )
}

/// Power: base^exponent
pub fn template_power() -> Expression {
    Expression::operation(
        "sup",
        vec![
            Expression::placeholder(next_id(), "base"),
            Expression::placeholder(next_id(), "exponent"),
        ],
    )
}

/// Square root: √x
pub fn template_sqrt() -> Expression {
    Expression::operation("sqrt", vec![Expression::placeholder(next_id(), "radicand")])
}

/// Subscript: base_sub
pub fn template_subscript() -> Expression {
    Expression::operation(
        "sub",
        vec![
            Expression::placeholder(next_id(), "base"),
            Expression::placeholder(next_id(), "subscript"),
        ],
    )
}

/// Sum (addition): a + b
pub fn template_plus() -> Expression {
    Expression::operation(
        "plus",
        vec![
            Expression::placeholder(next_id(), "left"),
            Expression::placeholder(next_id(), "right"),
        ],
    )
}

/// Difference (subtraction): a - b
pub fn template_minus() -> Expression {
    Expression::operation(
        "minus",
        vec![
            Expression::placeholder(next_id(), "left"),
            Expression::placeholder(next_id(), "right"),
        ],
    )
}

/// Product: a × b
pub fn template_times() -> Expression {
    Expression::operation(
        "scalar_multiply",
        vec![
            Expression::placeholder(next_id(), "left"),
            Expression::placeholder(next_id(), "right"),
        ],
    )
}

/// Equals: a = b
pub fn template_equals() -> Expression {
    Expression::operation(
        "equals",
        vec![
            Expression::placeholder(next_id(), "left"),
            Expression::placeholder(next_id(), "right"),
        ],
    )
}

// === Calculus ===

/// Integral with bounds: ∫ₐᵇ f(x) dx
pub fn template_integral() -> Expression {
    Expression::operation(
        "int_bounds",
        vec![
            Expression::placeholder(next_id(), "integrand"),
            Expression::placeholder(next_id(), "lower"),
            Expression::placeholder(next_id(), "upper"),
            Expression::placeholder(next_id(), "variable"),
        ],
    )
}

/// Summation with bounds: Σᵢ₌ₙᵐ expr
pub fn template_sum() -> Expression {
    Expression::operation(
        "sum_bounds",
        vec![
            Expression::placeholder(next_id(), "body"),
            Expression::placeholder(next_id(), "from"),
            Expression::placeholder(next_id(), "to"),
        ],
    )
}

/// Product with bounds: Πᵢ₌ₙᵐ expr
pub fn template_product() -> Expression {
    Expression::operation(
        "prod_bounds",
        vec![
            Expression::placeholder(next_id(), "body"),
            Expression::placeholder(next_id(), "from"),
            Expression::placeholder(next_id(), "to"),
        ],
    )
}

/// Partial derivative: ∂f/∂x
pub fn template_partial() -> Expression {
    Expression::operation(
        "d_part",
        vec![
            Expression::placeholder(next_id(), "function"),
            Expression::placeholder(next_id(), "variable"),
        ],
    )
}

/// Time derivative: df/dt
pub fn template_derivative_time() -> Expression {
    Expression::operation(
        "d_dt",
        vec![
            Expression::placeholder(next_id(), "function"),
            Expression::placeholder(next_id(), "variable"),
        ],
    )
}

/// Gradient: ∇f
pub fn template_gradient() -> Expression {
    Expression::operation("grad", vec![Expression::placeholder(next_id(), "function")])
}

// === Linear Algebra ===

/// 2×2 Matrix (generic constructor)
pub fn template_matrix_2x2() -> Expression {
    Expression::operation(
        "Matrix",
        vec![
            Expression::Const("2".to_string()), // rows
            Expression::Const("2".to_string()), // cols
            Expression::List(vec![
                Expression::placeholder(next_id(), "a11"),
                Expression::placeholder(next_id(), "a12"),
                Expression::placeholder(next_id(), "a21"),
                Expression::placeholder(next_id(), "a22"),
            ]),
        ],
    )
}

/// 3×3 Matrix (generic constructor)
pub fn template_matrix_3x3() -> Expression {
    Expression::operation(
        "Matrix",
        vec![
            Expression::Const("3".to_string()), // rows
            Expression::Const("3".to_string()), // cols
            Expression::List(vec![
                Expression::placeholder(next_id(), "a11"),
                Expression::placeholder(next_id(), "a12"),
                Expression::placeholder(next_id(), "a13"),
                Expression::placeholder(next_id(), "a21"),
                Expression::placeholder(next_id(), "a22"),
                Expression::placeholder(next_id(), "a23"),
                Expression::placeholder(next_id(), "a31"),
                Expression::placeholder(next_id(), "a32"),
                Expression::placeholder(next_id(), "a33"),
            ]),
        ],
    )
}

/// Vector (bold): 𝐯
pub fn template_vector_bold() -> Expression {
    Expression::operation(
        "vector_bold",
        vec![Expression::placeholder(next_id(), "vector")],
    )
}

/// Vector (arrow): v⃗
pub fn template_vector_arrow() -> Expression {
    Expression::operation(
        "vector_arrow",
        vec![Expression::placeholder(next_id(), "vector")],
    )
}

/// Dot product: a · b
pub fn template_dot_product() -> Expression {
    Expression::operation(
        "dot",
        vec![
            Expression::placeholder(next_id(), "left"),
            Expression::placeholder(next_id(), "right"),
        ],
    )
}

/// Cross product: a × b
pub fn template_cross_product() -> Expression {
    Expression::operation(
        "cross",
        vec![
            Expression::placeholder(next_id(), "left"),
            Expression::placeholder(next_id(), "right"),
        ],
    )
}

/// Norm: ‖v‖
pub fn template_norm() -> Expression {
    Expression::operation("norm", vec![Expression::placeholder(next_id(), "vector")])
}

/// Absolute value: |x|
pub fn template_abs() -> Expression {
    Expression::operation("abs", vec![Expression::placeholder(next_id(), "value")])
}

// === Brackets & Grouping ===

/// Parentheses: (x)
pub fn template_parens() -> Expression {
    Expression::operation(
        "parens",
        vec![Expression::placeholder(next_id(), "content")],
    )
}

/// Square brackets: [x]
pub fn template_brackets() -> Expression {
    Expression::operation(
        "brackets",
        vec![Expression::placeholder(next_id(), "content")],
    )
}

/// Curly braces: {x}
pub fn template_braces() -> Expression {
    Expression::operation(
        "braces",
        vec![Expression::placeholder(next_id(), "content")],
    )
}

/// Angle brackets: ⟨x⟩
pub fn template_angle_brackets() -> Expression {
    Expression::operation(
        "angle_brackets",
        vec![Expression::placeholder(next_id(), "content")],
    )
}

/// Floor function: ⌊x⌋
pub fn template_floor() -> Expression {
    Expression::operation("floor", vec![Expression::placeholder(next_id(), "arg")])
}

/// Ceiling function: ⌈x⌉
pub fn template_ceiling() -> Expression {
    Expression::operation("ceiling", vec![Expression::placeholder(next_id(), "arg")])
}

// === Quantum Mechanics ===

/// Ket vector: |ψ⟩
pub fn template_ket() -> Expression {
    Expression::operation("ket", vec![Expression::placeholder(next_id(), "state")])
}

/// Bra vector: ⟨ψ|
pub fn template_bra() -> Expression {
    Expression::operation("bra", vec![Expression::placeholder(next_id(), "state")])
}

/// Inner product: ⟨ψ|φ⟩
pub fn template_inner() -> Expression {
    Expression::operation(
        "inner",
        vec![
            Expression::placeholder(next_id(), "bra"),
            Expression::placeholder(next_id(), "ket"),
        ],
    )
}

/// Outer product: |ψ⟩⟨φ|
pub fn template_outer() -> Expression {
    Expression::operation(
        "outer",
        vec![
            Expression::placeholder(next_id(), "ket"),
            Expression::placeholder(next_id(), "bra"),
        ],
    )
}

/// Commutator: [A, B]
pub fn template_commutator() -> Expression {
    Expression::operation(
        "commutator",
        vec![
            Expression::placeholder(next_id(), "A"),
            Expression::placeholder(next_id(), "B"),
        ],
    )
}

/// Expectation value: ⟨Â⟩
pub fn template_expectation() -> Expression {
    Expression::operation(
        "expectation",
        vec![Expression::placeholder(next_id(), "operator")],
    )
}

// === Tensor Operations ===

/// Tensor with mixed indices: T^μ_ν
pub fn template_tensor_mixed() -> Expression {
    Expression::operation(
        "index_mixed",
        vec![
            Expression::placeholder(next_id(), "base"),
            Expression::placeholder(next_id(), "upper"),
            Expression::placeholder(next_id(), "lower"),
        ],
    )
}

/// Subscript-superscript: base_{sub}^{sup}
pub fn template_subsup() -> Expression {
    Expression::operation(
        "subsup",
        vec![
            Expression::placeholder(next_id(), "base"),
            Expression::placeholder(next_id(), "subscript"),
            Expression::placeholder(next_id(), "superscript"),
        ],
    )
}

/// Tensor with 1 upper and 3 lower indices: base^{upper}_{lower1 lower2 lower3}
pub fn template_tensor_1up_3down() -> Expression {
    Expression::operation(
        "tensor_1up_3down",
        vec![
            Expression::placeholder(next_id(), "base"),
            Expression::placeholder(next_id(), "upper"),
            Expression::placeholder(next_id(), "lower1"),
            Expression::placeholder(next_id(), "lower2"),
            Expression::placeholder(next_id(), "lower3"),
        ],
    )
}

/// Tensor with double lower indices: g_{μν}
pub fn template_tensor_lower_pair() -> Expression {
    Expression::operation(
        "tensor_lower_pair",
        vec![
            Expression::placeholder(next_id(), "base"),
            Expression::placeholder(next_id(), "lower1"),
            Expression::placeholder(next_id(), "lower2"),
        ],
    )
}

/// Tensor with 2 upper and 2 lower indices: R^{μν}_{ρσ}
pub fn template_tensor_2up_2down() -> Expression {
    Expression::operation(
        "tensor_2up_2down",
        vec![
            Expression::placeholder(next_id(), "base"),
            Expression::placeholder(next_id(), "upper1"),
            Expression::placeholder(next_id(), "upper2"),
            Expression::placeholder(next_id(), "lower1"),
            Expression::placeholder(next_id(), "lower2"),
        ],
    )
}

/// Tensor with double upper indices: T^μν
pub fn template_tensor_upper_pair() -> Expression {
    Expression::operation(
        "index_pair",
        vec![
            Expression::placeholder(next_id(), "base"),
            Expression::placeholder(next_id(), "idx1"),
            Expression::placeholder(next_id(), "idx2"),
        ],
    )
}

// === Trigonometry ===

/// Sine: sin(x)
pub fn template_sin() -> Expression {
    Expression::operation("sin", vec![Expression::placeholder(next_id(), "argument")])
}

/// Cosine: cos(x)
pub fn template_cos() -> Expression {
    Expression::operation("cos", vec![Expression::placeholder(next_id(), "argument")])
}

/// Tangent: tan(x)
pub fn template_tan() -> Expression {
    Expression::operation("tan", vec![Expression::placeholder(next_id(), "argument")])
}

// === Limits ===

/// Limit: lim_{x→a} f(x)
pub fn template_limit() -> Expression {
    Expression::operation(
        "lim",
        vec![
            Expression::placeholder(next_id(), "body"),
            Expression::placeholder(next_id(), "var"),
            Expression::placeholder(next_id(), "target"),
        ],
    )
}

// === Additional Functions ===

/// Arcsine: arcsin(x)
pub fn template_arcsin() -> Expression {
    Expression::operation(
        "arcsin",
        vec![Expression::placeholder(next_id(), "argument")],
    )
}

/// Arccosine: arccos(x)
pub fn template_arccos() -> Expression {
    Expression::operation(
        "arccos",
        vec![Expression::placeholder(next_id(), "argument")],
    )
}

/// Arctangent: arctan(x)
pub fn template_arctan() -> Expression {
    Expression::operation(
        "arctan",
        vec![Expression::placeholder(next_id(), "argument")],
    )
}

/// Natural logarithm: ln(x)
pub fn template_ln() -> Expression {
    Expression::operation("ln", vec![Expression::placeholder(next_id(), "argument")])
}

/// Logarithm: log(x)
pub fn template_log() -> Expression {
    Expression::operation("log", vec![Expression::placeholder(next_id(), "argument")])
}

/// Exponential: exp(x)
pub fn template_exp() -> Expression {
    Expression::operation("exp", vec![Expression::placeholder(next_id(), "argument")])
}

// === Accents ===

/// Dot accent: ẋ (velocity, time derivative)
pub fn template_dot_accent() -> Expression {
    Expression::operation(
        "dot_accent",
        vec![Expression::placeholder(next_id(), "variable")],
    )
}

/// Double dot accent: ẍ (acceleration, second derivative)
pub fn template_ddot_accent() -> Expression {
    Expression::operation(
        "ddot_accent",
        vec![Expression::placeholder(next_id(), "variable")],
    )
}

/// Hat accent: x̂
pub fn template_hat() -> Expression {
    Expression::operation("hat", vec![Expression::placeholder(next_id(), "variable")])
}

/// Bar accent: x̄
pub fn template_bar() -> Expression {
    Expression::operation("bar", vec![Expression::placeholder(next_id(), "variable")])
}

/// Tilde accent: x̃
pub fn template_tilde() -> Expression {
    Expression::operation(
        "tilde",
        vec![Expression::placeholder(next_id(), "variable")],
    )
}

// === Advanced Tensors ===

/// Christoffel symbol: Γ^μ_{νσ}
pub fn template_christoffel() -> Expression {
    Expression::operation(
        "gamma",
        vec![
            Expression::placeholder(next_id(), "dummy"), // First arg is empty in render
            Expression::placeholder(next_id(), "upper"),
            Expression::placeholder(next_id(), "lower1"),
            Expression::placeholder(next_id(), "lower2"),
        ],
    )
}

/// Riemann tensor: R^ρ_{σμν}
pub fn template_riemann() -> Expression {
    Expression::operation(
        "riemann",
        vec![
            Expression::placeholder(next_id(), "dummy"), // First arg is empty in render
            Expression::placeholder(next_id(), "upper"),
            Expression::placeholder(next_id(), "lower1"),
            Expression::placeholder(next_id(), "lower2"),
            Expression::placeholder(next_id(), "lower3"),
        ],
    )
}

// === Additional Matrix Types ===

/// 2×2 Matrix with parentheses (generic constructor)
pub fn template_pmatrix_2x2() -> Expression {
    Expression::operation(
        "PMatrix",
        vec![
            Expression::Const("2".to_string()), // rows
            Expression::Const("2".to_string()), // cols
            Expression::List(vec![
                Expression::placeholder(next_id(), "a11"),
                Expression::placeholder(next_id(), "a12"),
                Expression::placeholder(next_id(), "a21"),
                Expression::placeholder(next_id(), "a22"),
            ]),
        ],
    )
}

/// 3×3 Matrix with parentheses (generic constructor)
pub fn template_pmatrix_3x3() -> Expression {
    Expression::operation(
        "PMatrix",
        vec![
            Expression::Const("3".to_string()), // rows
            Expression::Const("3".to_string()), // cols
            Expression::List(vec![
                Expression::placeholder(next_id(), "a11"),
                Expression::placeholder(next_id(), "a12"),
                Expression::placeholder(next_id(), "a13"),
                Expression::placeholder(next_id(), "a21"),
                Expression::placeholder(next_id(), "a22"),
                Expression::placeholder(next_id(), "a23"),
                Expression::placeholder(next_id(), "a31"),
                Expression::placeholder(next_id(), "a32"),
                Expression::placeholder(next_id(), "a33"),
            ]),
        ],
    )
}

/// 2×2 Determinant matrix (vertical bars, generic constructor)
pub fn template_vmatrix_2x2() -> Expression {
    Expression::operation(
        "VMatrix",
        vec![
            Expression::Const("2".to_string()), // rows
            Expression::Const("2".to_string()), // cols
            Expression::List(vec![
                Expression::placeholder(next_id(), "a11"),
                Expression::placeholder(next_id(), "a12"),
                Expression::placeholder(next_id(), "a21"),
                Expression::placeholder(next_id(), "a22"),
            ]),
        ],
    )
}

/// 3×3 Determinant matrix (vertical bars, generic constructor)
pub fn template_vmatrix_3x3() -> Expression {
    Expression::operation(
        "VMatrix",
        vec![
            Expression::Const("3".to_string()), // rows
            Expression::Const("3".to_string()), // cols
            Expression::List(vec![
                Expression::placeholder(next_id(), "a11"),
                Expression::placeholder(next_id(), "a12"),
                Expression::placeholder(next_id(), "a13"),
                Expression::placeholder(next_id(), "a21"),
                Expression::placeholder(next_id(), "a22"),
                Expression::placeholder(next_id(), "a23"),
                Expression::placeholder(next_id(), "a31"),
                Expression::placeholder(next_id(), "a32"),
                Expression::placeholder(next_id(), "a33"),
            ]),
        ],
    )
}

// === Integral Transforms ===

/// Fourier transform: ℱ[f](ω)
pub fn template_fourier_transform() -> Expression {
    Expression::operation(
        "fourier_transform",
        vec![
            Expression::placeholder(next_id(), "function"),
            Expression::placeholder(next_id(), "variable"),
        ],
    )
}

/// Inverse Fourier transform: ℱ⁻¹[f](x)
pub fn template_inverse_fourier() -> Expression {
    Expression::operation(
        "inverse_fourier",
        vec![
            Expression::placeholder(next_id(), "function"),
            Expression::placeholder(next_id(), "variable"),
        ],
    )
}

/// Laplace transform: ℒ[f](s)
pub fn template_laplace_transform() -> Expression {
    Expression::operation(
        "laplace_transform",
        vec![
            Expression::placeholder(next_id(), "function"),
            Expression::placeholder(next_id(), "variable"),
        ],
    )
}

/// Inverse Laplace transform: ℒ⁻¹[f](t)
pub fn template_inverse_laplace() -> Expression {
    Expression::operation(
        "inverse_laplace",
        vec![
            Expression::placeholder(next_id(), "function"),
            Expression::placeholder(next_id(), "variable"),
        ],
    )
}

/// Convolution: (f ∗ g)(x)
pub fn template_convolution() -> Expression {
    Expression::operation(
        "convolution",
        vec![
            Expression::placeholder(next_id(), "f"),
            Expression::placeholder(next_id(), "g"),
            Expression::placeholder(next_id(), "variable"),
        ],
    )
}

/// Kernel integral: ∫ K(x,m) f(m) dμ
pub fn template_kernel_integral() -> Expression {
    Expression::operation(
        "kernel_integral",
        vec![
            Expression::placeholder(next_id(), "kernel"),
            Expression::placeholder(next_id(), "function"),
            Expression::placeholder(next_id(), "domain"),
            Expression::placeholder(next_id(), "variable"),
        ],
    )
}

/// Green's function: G(x, m)
pub fn template_greens_function() -> Expression {
    Expression::operation(
        "greens_function",
        vec![
            Expression::placeholder(next_id(), "point_x"),
            Expression::placeholder(next_id(), "source_m"),
        ],
    )
}

// === POT-Specific Operations ===

/// Projection operator: Π[f](x)
pub fn template_projection() -> Expression {
    Expression::operation(
        "projection",
        vec![
            Expression::placeholder(next_id(), "function"),
            Expression::placeholder(next_id(), "variable"),
        ],
    )
}

/// Modal integral: ∫_M f(m) dμ(m)
pub fn template_modal_integral() -> Expression {
    Expression::operation(
        "modal_integral",
        vec![
            Expression::placeholder(next_id(), "function"),
            Expression::placeholder(next_id(), "modal_space"),
            Expression::placeholder(next_id(), "variable"),
        ],
    )
}

/// Projection kernel: K(x, m)
pub fn template_projection_kernel() -> Expression {
    Expression::operation(
        "projection_kernel",
        vec![
            Expression::placeholder(next_id(), "spacetime_point"),
            Expression::placeholder(next_id(), "modal_state"),
        ],
    )
}

/// Causal bound: c(x)
pub fn template_causal_bound() -> Expression {
    Expression::operation(
        "causal_bound",
        vec![Expression::placeholder(next_id(), "point")],
    )
}

/// Projection residue: Residue[Π, X]
pub fn template_projection_residue() -> Expression {
    Expression::operation(
        "projection_residue",
        vec![
            Expression::placeholder(next_id(), "projection"),
            Expression::placeholder(next_id(), "structure"),
        ],
    )
}

/// Modal space: ModalSpace
pub fn template_modal_space() -> Expression {
    Expression::operation(
        "modal_space",
        vec![Expression::placeholder(next_id(), "name")],
    )
}

/// Spacetime: R⁴
pub fn template_spacetime() -> Expression {
    Expression::operation("spacetime", vec![])
}

/// Hont (Hilbert Ontology): Modal domain
pub fn template_hont() -> Expression {
    Expression::operation(
        "hont",
        vec![Expression::placeholder(next_id(), "dimension")],
    )
}

// === Template Registry ===

/// Get all available templates as (name, function) pairs
pub fn get_all_templates() -> Vec<(&'static str, fn() -> Expression)> {
    vec![
        // Basic
        ("fraction", template_fraction as fn() -> Expression),
        ("power", template_power),
        ("sqrt", template_sqrt),
        ("subscript", template_subscript),
        ("plus", template_plus),
        ("minus", template_minus),
        ("times", template_times),
        ("equals", template_equals),
        // Calculus
        ("integral", template_integral),
        ("sum", template_sum),
        ("product", template_product),
        ("partial", template_partial),
        ("derivative", template_derivative_time),
        ("gradient", template_gradient),
        // Linear Algebra
        ("matrix2x2", template_matrix_2x2),
        ("matrix3x3", template_matrix_3x3),
        ("pmatrix2x2", template_pmatrix_2x2),
        ("pmatrix3x3", template_pmatrix_3x3),
        ("vmatrix2x2", template_vmatrix_2x2),
        ("vmatrix3x3", template_vmatrix_3x3),
        ("vector_bold", template_vector_bold),
        ("vector_arrow", template_vector_arrow),
        ("dot", template_dot_product),
        ("cross", template_cross_product),
        ("norm", template_norm),
        ("abs", template_abs),
        // Integral Transforms
        ("fourier_transform", template_fourier_transform),
        ("inverse_fourier", template_inverse_fourier),
        ("laplace_transform", template_laplace_transform),
        ("inverse_laplace", template_inverse_laplace),
        ("convolution", template_convolution),
        ("kernel_integral", template_kernel_integral),
        ("greens_function", template_greens_function),
        // POT-Specific
        ("projection", template_projection),
        ("modal_integral", template_modal_integral),
        ("projection_kernel", template_projection_kernel),
        ("causal_bound", template_causal_bound),
        ("projection_residue", template_projection_residue),
        ("modal_space", template_modal_space),
        ("spacetime", template_spacetime),
        ("hont", template_hont),
        // Brackets & Grouping
        ("parens", template_parens),
        ("brackets", template_brackets),
        ("braces", template_braces),
        ("angle_brackets", template_angle_brackets),
        ("floor", template_floor),
        ("ceiling", template_ceiling),
        // Quantum
        ("ket", template_ket),
        ("bra", template_bra),
        ("inner", template_inner),
        ("outer", template_outer),
        ("commutator", template_commutator),
        ("expectation", template_expectation),
        // Tensors
        ("tensor_mixed", template_tensor_mixed),
        ("subsup", template_subsup),
        ("tensor_upper_pair", template_tensor_upper_pair),
        ("tensor_lower_pair", template_tensor_lower_pair),
        ("tensor_1up_3down", template_tensor_1up_3down),
        ("tensor_2up_2down", template_tensor_2up_2down),
        ("christoffel", template_christoffel),
        ("riemann", template_riemann),
        // Trig & Functions
        ("sin", template_sin),
        ("cos", template_cos),
        ("tan", template_tan),
        ("arcsin", template_arcsin),
        ("arccos", template_arccos),
        ("arctan", template_arctan),
        ("ln", template_ln),
        ("log", template_log),
        ("exp", template_exp),
        // Accents
        ("dot_accent", template_dot_accent),
        ("ddot_accent", template_ddot_accent),
        ("hat", template_hat),
        ("bar", template_bar),
        ("tilde", template_tilde),
        // Limits
        ("limit", template_limit),
    ]
}

/// Get a template by name
pub fn get_template(name: &str) -> Option<Expression> {
    get_all_templates()
        .into_iter()
        .find(|(n, _)| *n == name)
        .map(|(_, f)| f())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fraction_template() {
        reset_placeholder_counter();
        let frac = template_fraction();
        match frac {
            Expression::Operation { name, args } => {
                assert_eq!(name, "scalar_divide");
                assert_eq!(args.len(), 2);
                assert!(matches!(args[0], Expression::Placeholder { .. }));
                assert!(matches!(args[1], Expression::Placeholder { .. }));
            }
            _ => panic!("Expected operation"),
        }
    }

    #[test]
    fn test_integral_template() {
        reset_placeholder_counter();
        let integral = template_integral();
        match integral {
            Expression::Operation { name, args } => {
                assert_eq!(name, "int_bounds");
                assert_eq!(args.len(), 4);
                for arg in args {
                    assert!(matches!(arg, Expression::Placeholder { .. }));
                }
            }
            _ => panic!("Expected operation"),
        }
    }

    #[test]
    fn test_matrix_2x2_template() {
        reset_placeholder_counter();
        let matrix = template_matrix_2x2();
        match matrix {
            Expression::Operation { name, args } => {
                assert_eq!(name, "Matrix");
                // NEW FORMAT: 2 dimension args + 1 List = 3 total
                assert_eq!(args.len(), 3);
                // Third arg should be a List with 4 elements
                match &args[2] {
                    Expression::List(elements) => {
                        assert_eq!(elements.len(), 4, "2x2 matrix should have 4 elements");
                    }
                    _ => panic!("Expected List as third argument"),
                }
            }
            _ => panic!("Expected operation"),
        }
    }

    #[test]
    fn test_get_template_by_name() {
        reset_placeholder_counter();
        let frac = get_template("fraction");
        assert!(frac.is_some());

        let unknown = get_template("unknown_template");
        assert!(unknown.is_none());
    }

    // === Tests for Integral Transform Templates ===

    #[test]
    fn test_fourier_transform() {
        reset_placeholder_counter();
        let expr = template_fourier_transform();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "fourier_transform");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected operation"),
        }
    }

    #[test]
    fn test_laplace_transform() {
        reset_placeholder_counter();
        let expr = template_laplace_transform();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "laplace_transform");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected operation"),
        }
    }

    #[test]
    fn test_convolution() {
        reset_placeholder_counter();
        let expr = template_convolution();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "convolution");
                assert_eq!(args.len(), 3);
            }
            _ => panic!("Expected operation"),
        }
    }

    #[test]
    fn test_kernel_integral() {
        reset_placeholder_counter();
        let expr = template_kernel_integral();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "kernel_integral");
                assert_eq!(args.len(), 4);
            }
            _ => panic!("Expected operation"),
        }
    }

    #[test]
    fn test_greens_function() {
        reset_placeholder_counter();
        let expr = template_greens_function();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "greens_function");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected operation"),
        }
    }

    // === Tests for POT-Specific Templates ===

    #[test]
    fn test_projection() {
        reset_placeholder_counter();
        let expr = template_projection();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "projection");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected operation"),
        }
    }

    #[test]
    fn test_modal_integral() {
        reset_placeholder_counter();
        let expr = template_modal_integral();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "modal_integral");
                assert_eq!(args.len(), 3);
            }
            _ => panic!("Expected operation"),
        }
    }

    #[test]
    fn test_projection_kernel() {
        reset_placeholder_counter();
        let expr = template_projection_kernel();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "projection_kernel");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected operation"),
        }
    }

    #[test]
    fn test_causal_bound() {
        reset_placeholder_counter();
        let expr = template_causal_bound();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "causal_bound");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected operation"),
        }
    }

    #[test]
    fn test_spacetime() {
        let expr = template_spacetime();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "spacetime");
                assert_eq!(args.len(), 0);
            }
            _ => panic!("Expected operation"),
        }
    }

    #[test]
    fn test_hont() {
        reset_placeholder_counter();
        let expr = template_hont();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "hont");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected operation"),
        }
    }

    #[test]
    fn test_all_new_templates_registered() {
        let templates = get_all_templates();
        let names: Vec<&str> = templates.iter().map(|(name, _)| *name).collect();

        // Check integral transforms
        assert!(names.contains(&"fourier_transform"));
        assert!(names.contains(&"inverse_fourier"));
        assert!(names.contains(&"laplace_transform"));
        assert!(names.contains(&"inverse_laplace"));
        assert!(names.contains(&"convolution"));
        assert!(names.contains(&"kernel_integral"));
        assert!(names.contains(&"greens_function"));

        // Check POT operations
        assert!(names.contains(&"projection"));
        assert!(names.contains(&"modal_integral"));
        assert!(names.contains(&"projection_kernel"));
        assert!(names.contains(&"causal_bound"));
        assert!(names.contains(&"projection_residue"));
        assert!(names.contains(&"modal_space"));
        assert!(names.contains(&"spacetime"));
        assert!(names.contains(&"hont"));
    }
}
