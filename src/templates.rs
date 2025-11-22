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
    Expression::operation("scalar_divide", vec![
        Expression::placeholder(next_id(), "numerator"),
        Expression::placeholder(next_id(), "denominator"),
    ])
}

/// Power: base^exponent
pub fn template_power() -> Expression {
    Expression::operation("sup", vec![
        Expression::placeholder(next_id(), "base"),
        Expression::placeholder(next_id(), "exponent"),
    ])
}

/// Square root: √x
pub fn template_sqrt() -> Expression {
    Expression::operation("sqrt", vec![
        Expression::placeholder(next_id(), "radicand"),
    ])
}

/// Subscript: base_sub
pub fn template_subscript() -> Expression {
    Expression::operation("sub", vec![
        Expression::placeholder(next_id(), "base"),
        Expression::placeholder(next_id(), "subscript"),
    ])
}

/// Sum (addition): a + b
pub fn template_plus() -> Expression {
    Expression::operation("plus", vec![
        Expression::placeholder(next_id(), "left"),
        Expression::placeholder(next_id(), "right"),
    ])
}

/// Difference (subtraction): a - b
pub fn template_minus() -> Expression {
    Expression::operation("minus", vec![
        Expression::placeholder(next_id(), "left"),
        Expression::placeholder(next_id(), "right"),
    ])
}

/// Product: a × b
pub fn template_times() -> Expression {
    Expression::operation("scalar_multiply", vec![
        Expression::placeholder(next_id(), "left"),
        Expression::placeholder(next_id(), "right"),
    ])
}

/// Equals: a = b
pub fn template_equals() -> Expression {
    Expression::operation("equals", vec![
        Expression::placeholder(next_id(), "left"),
        Expression::placeholder(next_id(), "right"),
    ])
}

// === Calculus ===

/// Integral with bounds: ∫ₐᵇ f(x) dx
pub fn template_integral() -> Expression {
    Expression::operation("int_bounds", vec![
        Expression::placeholder(next_id(), "integrand"),
        Expression::placeholder(next_id(), "lower"),
        Expression::placeholder(next_id(), "upper"),
        Expression::placeholder(next_id(), "variable"),
    ])
}

/// Summation with bounds: Σᵢ₌ₙᵐ expr
pub fn template_sum() -> Expression {
    Expression::operation("sum_bounds", vec![
        Expression::placeholder(next_id(), "body"),
        Expression::placeholder(next_id(), "from"),
        Expression::placeholder(next_id(), "to"),
    ])
}

/// Product with bounds: Πᵢ₌ₙᵐ expr
pub fn template_product() -> Expression {
    Expression::operation("prod_bounds", vec![
        Expression::placeholder(next_id(), "body"),
        Expression::placeholder(next_id(), "from"),
        Expression::placeholder(next_id(), "to"),
    ])
}

/// Partial derivative: ∂f/∂x
pub fn template_partial() -> Expression {
    Expression::operation("d_part", vec![
        Expression::placeholder(next_id(), "function"),
        Expression::placeholder(next_id(), "variable"),
    ])
}

/// Time derivative: df/dt
pub fn template_derivative_time() -> Expression {
    Expression::operation("d_dt", vec![
        Expression::placeholder(next_id(), "function"),
        Expression::placeholder(next_id(), "variable"),
    ])
}

/// Gradient: ∇f
pub fn template_gradient() -> Expression {
    Expression::operation("grad", vec![
        Expression::placeholder(next_id(), "function"),
    ])
}

// === Linear Algebra ===

/// 2×2 Matrix
pub fn template_matrix_2x2() -> Expression {
    Expression::operation("matrix2x2", vec![
        Expression::placeholder(next_id(), "a11"),
        Expression::placeholder(next_id(), "a12"),
        Expression::placeholder(next_id(), "a21"),
        Expression::placeholder(next_id(), "a22"),
    ])
}

/// 3×3 Matrix
pub fn template_matrix_3x3() -> Expression {
    Expression::operation("matrix3x3", vec![
        Expression::placeholder(next_id(), "a11"),
        Expression::placeholder(next_id(), "a12"),
        Expression::placeholder(next_id(), "a13"),
        Expression::placeholder(next_id(), "a21"),
        Expression::placeholder(next_id(), "a22"),
        Expression::placeholder(next_id(), "a23"),
        Expression::placeholder(next_id(), "a31"),
        Expression::placeholder(next_id(), "a32"),
        Expression::placeholder(next_id(), "a33"),
    ])
}

/// Vector (bold): 𝐯
pub fn template_vector_bold() -> Expression {
    Expression::operation("vector_bold", vec![
        Expression::placeholder(next_id(), "vector"),
    ])
}

/// Vector (arrow): v⃗
pub fn template_vector_arrow() -> Expression {
    Expression::operation("vector_arrow", vec![
        Expression::placeholder(next_id(), "vector"),
    ])
}

/// Dot product: a · b
pub fn template_dot_product() -> Expression {
    Expression::operation("dot", vec![
        Expression::placeholder(next_id(), "left"),
        Expression::placeholder(next_id(), "right"),
    ])
}

/// Cross product: a × b
pub fn template_cross_product() -> Expression {
    Expression::operation("cross", vec![
        Expression::placeholder(next_id(), "left"),
        Expression::placeholder(next_id(), "right"),
    ])
}

/// Norm: ‖v‖
pub fn template_norm() -> Expression {
    Expression::operation("norm", vec![
        Expression::placeholder(next_id(), "vector"),
    ])
}

/// Absolute value: |x|
pub fn template_abs() -> Expression {
    Expression::operation("abs", vec![
        Expression::placeholder(next_id(), "value"),
    ])
}

// === Quantum Mechanics ===

/// Ket vector: |ψ⟩
pub fn template_ket() -> Expression {
    Expression::operation("ket", vec![
        Expression::placeholder(next_id(), "state"),
    ])
}

/// Bra vector: ⟨ψ|
pub fn template_bra() -> Expression {
    Expression::operation("bra", vec![
        Expression::placeholder(next_id(), "state"),
    ])
}

/// Inner product: ⟨ψ|φ⟩
pub fn template_inner() -> Expression {
    Expression::operation("inner", vec![
        Expression::placeholder(next_id(), "bra"),
        Expression::placeholder(next_id(), "ket"),
    ])
}

/// Outer product: |ψ⟩⟨φ|
pub fn template_outer() -> Expression {
    Expression::operation("outer", vec![
        Expression::placeholder(next_id(), "ket"),
        Expression::placeholder(next_id(), "bra"),
    ])
}

/// Commutator: [A, B]
pub fn template_commutator() -> Expression {
    Expression::operation("commutator", vec![
        Expression::placeholder(next_id(), "A"),
        Expression::placeholder(next_id(), "B"),
    ])
}

/// Expectation value: ⟨Â⟩
pub fn template_expectation() -> Expression {
    Expression::operation("expectation", vec![
        Expression::placeholder(next_id(), "operator"),
    ])
}

// === Tensor Operations ===

/// Tensor with mixed indices: T^μ_ν
pub fn template_tensor_mixed() -> Expression {
    Expression::operation("index_mixed", vec![
        Expression::placeholder(next_id(), "base"),
        Expression::placeholder(next_id(), "upper"),
        Expression::placeholder(next_id(), "lower"),
    ])
}

/// Tensor with double upper indices: T^μν
pub fn template_tensor_upper_pair() -> Expression {
    Expression::operation("index_pair", vec![
        Expression::placeholder(next_id(), "base"),
        Expression::placeholder(next_id(), "idx1"),
        Expression::placeholder(next_id(), "idx2"),
    ])
}

// === Trigonometry ===

/// Sine: sin(x)
pub fn template_sin() -> Expression {
    Expression::operation("sin", vec![
        Expression::placeholder(next_id(), "argument"),
    ])
}

/// Cosine: cos(x)
pub fn template_cos() -> Expression {
    Expression::operation("cos", vec![
        Expression::placeholder(next_id(), "argument"),
    ])
}

/// Tangent: tan(x)
pub fn template_tan() -> Expression {
    Expression::operation("tan", vec![
        Expression::placeholder(next_id(), "argument"),
    ])
}

// === Limits ===

/// Limit: lim_{x→a} f(x)
pub fn template_limit() -> Expression {
    Expression::operation("lim", vec![
        Expression::placeholder(next_id(), "body"),
        Expression::placeholder(next_id(), "var"),
        Expression::placeholder(next_id(), "target"),
    ])
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
        ("vector_bold", template_vector_bold),
        ("vector_arrow", template_vector_arrow),
        ("dot", template_dot_product),
        ("cross", template_cross_product),
        ("norm", template_norm),
        ("abs", template_abs),
        // Quantum
        ("ket", template_ket),
        ("bra", template_bra),
        ("inner", template_inner),
        ("outer", template_outer),
        ("commutator", template_commutator),
        ("expectation", template_expectation),
        // Tensors
        ("tensor_mixed", template_tensor_mixed),
        ("tensor_upper_pair", template_tensor_upper_pair),
        // Trig
        ("sin", template_sin),
        ("cos", template_cos),
        ("tan", template_tan),
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
                assert_eq!(name, "matrix2x2");
                assert_eq!(args.len(), 4);
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
}

