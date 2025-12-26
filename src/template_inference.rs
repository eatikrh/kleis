//! Template-Based Semantic Inference
//!
//! Post-processes flat parsed ASTs to infer structured operations
//! by pattern-matching against known template outputs.
//!
//! If inference fails, the original flat AST is preserved (graceful degradation).

use crate::ast::Expression;

/// Apply template inference to a parsed AST
///
/// Attempts to recognize patterns in flat multiplication chains
/// that correspond to structured operations (integrals, limits, etc.)
///
/// Returns the inferred structured AST if successful, otherwise returns
/// the original flat AST unchanged.
pub fn infer_templates(expr: Expression) -> Expression {
    // First recursively infer within child expressions
    let expr = match expr {
        Expression::Operation { name, args, .. } => {
            let new_args = args.into_iter().map(infer_templates).collect();
            Expression::Operation {
                name,
                args: new_args,
                span: None,
            }
        }
        other => other,
    };

    // Then attempt to infer a higher-level template for the current node
    try_infer_double_integral(&expr)
        .or_else(|| try_infer_triple_integral(&expr))
        .or_else(|| try_infer_logical_implication(&expr))
        .or_else(|| try_infer_quantifier(&expr))
        .or_else(|| try_infer_modular_congruence(&expr))
        .or_else(|| try_infer_statistics_functions(&expr))
        .or_else(|| try_infer_curl(&expr))
        // Add more pattern matchers here as needed:
        // .or_else(|| try_infer_limit(&expr))
        // .or_else(|| try_infer_sum_bounds(&expr))
        .unwrap_or(expr)
}

/// Attempt to infer double_integral structure
///
/// Pattern: sub(\iint, region) * integrand * mathrm(d) * var1 * mathrm(d) * var2
fn try_infer_double_integral(expr: &Expression) -> Option<Expression> {
    let terms = flatten_multiply(expr);

    if terms.len() < 6 {
        return None; // Need at least 6 terms for the pattern
    }

    // 1. Check first term is sub(\iint, region)
    let region = match &terms[0] {
        Expression::Operation { name, args, .. } if name == "sub" && args.len() == 2 => {
            match &args[0] {
                Expression::Object(s) if s == "\\iint" => args[1].clone(),
                _ => return None,
            }
        }
        _ => return None,
    };

    // 2. Integrand is term 1
    let integrand = terms[1].clone();

    // 3. Extract variables: look for mathrm(d) * var pattern starting at term 2
    let variables = extract_differential_vars(&terms, 2);

    // 4. For double integral, we need exactly 2 variables
    if variables.len() == 2 {
        Some(Expression::Operation {
            name: "double_integral".to_string(),
            args: vec![
                integrand,
                region,
                variables[0].clone(),
                variables[1].clone(),
            ],
            span: None,
        })
    } else {
        None
    }
}

/// Attempt to infer logical implication structure
///
/// Pattern: left * \Rightarrow * right
/// Also handles: \Leftarrow, \Leftrightarrow
fn try_infer_logical_implication(expr: &Expression) -> Option<Expression> {
    let terms = flatten_multiply(expr);

    // Need at least 3 terms: left, arrow, right
    if terms.len() < 3 {
        return None;
    }

    // Look for \Rightarrow, \Leftarrow, or \Leftrightarrow in the chain
    for i in 1..terms.len() - 1 {
        let (op_name, _arrow_symbol) = match &terms[i] {
            Expression::Object(s) if s == "\\Rightarrow" => ("implies", "\\Rightarrow"),
            Expression::Object(s) if s == "\\Leftarrow" => ("implied_by", "\\Leftarrow"),
            Expression::Object(s) if s == "\\Leftrightarrow" => ("iff", "\\Leftrightarrow"),
            _ => continue,
        };

        // Found an arrow! Everything before is left, everything after is right
        let left_terms = &terms[0..i];
        let right_terms = &terms[i + 1..];

        if left_terms.is_empty() || right_terms.is_empty() {
            continue; // Invalid pattern
        }

        // Reconstruct left and right from their term chains
        let left = rebuild_multiply(left_terms);
        let right = rebuild_multiply(right_terms);

        return Some(Expression::Operation {
            name: op_name.to_string(),
            args: vec![left, right],
            span: None,
        });
    }

    None
}

/// Attempt to infer quantifier structure
///
/// Pattern: The quantifier might be nested in the left operand of a relational operation
/// Example: in_set((\exists * var * separator * ...), ...)
///
/// Strategy: Check if a relational operation has a quantifier in its left operand
fn try_infer_quantifier(expr: &Expression) -> Option<Expression> {
    // Check if top-level is a relational operation (in_set, equals, etc.)
    match expr {
        Expression::Operation { name, args, .. } if is_relational_op(name) && args.len() == 2 => {
            // Check if left operand contains quantifier pattern
            let left_terms = flatten_multiply(&args[0]);

            if left_terms.len() < 2 {
                return None;
            }

            // Check if first term is \forall or \exists
            let op_name = match &left_terms[0] {
                Expression::Object(s) if s == "\\forall" => "forall",
                Expression::Object(s) if s == "\\exists" => "exists",
                _ => return None,
            };

            // Second term is the bound variable
            let var = left_terms[1].clone();

            // Skip optional separator (__SPACE__ from \colon)
            let body_start = if left_terms.len() > 2 {
                match &left_terms[2] {
                    Expression::Object(s) if s == "__SPACE__" => 3,
                    _ => 2,
                }
            } else {
                2
            };

            // Body is: remaining left terms + the original relational operation
            // Example: [\exists, x, __SPACE__, x] in in_set(..., S)
            // Body should be: in_set(x, S)

            if body_start < left_terms.len() {
                // Reconstruct the body: remaining left terms + relational op
                let remaining_left = rebuild_multiply(&left_terms[body_start..]);

                // Reconstruct the relational operation with cleaned left side
                let body = Expression::Operation {
                    name: name.clone(),
                    args: vec![remaining_left, args[1].clone()],
                    span: None,
                };

                return Some(Expression::Operation {
                    name: op_name.to_string(),
                    args: vec![var, body],
                    span: None,
                });
            }
        }
        _ => {}
    }

    None
}

/// Check if an operation name is a relational operator
fn is_relational_op(name: &str) -> bool {
    matches!(
        name,
        "equals"
            | "not_equal"
            | "less_than"
            | "greater_than"
            | "leq"
            | "geq"
            | "approx"
            | "equiv"
            | "in_set"
            | "subset"
            | "subseteq"
            | "union"
            | "intersection"
    )
}

/// Attempt to infer triple_integral structure
///
/// Pattern: sub(\iiint, region) * integrand * mathrm(d) * var1 * mathrm(d) * var2 * mathrm(d) * var3
fn try_infer_triple_integral(expr: &Expression) -> Option<Expression> {
    let terms = flatten_multiply(expr);

    if terms.len() < 8 {
        return None; // Need at least 8 terms for the pattern
    }

    // 1. Check first term is sub(\iiint, region)
    let region = match &terms[0] {
        Expression::Operation { name, args, .. } if name == "sub" && args.len() == 2 => {
            match &args[0] {
                Expression::Object(s) if s == "\\iiint" => args[1].clone(),
                _ => return None,
            }
        }
        _ => return None,
    };

    // 2. Integrand is term 1
    let integrand = terms[1].clone();

    // 3. Extract variables: look for mathrm(d) * var pattern starting at term 2
    let variables = extract_differential_vars(&terms, 2);

    // 4. For triple integral, we need exactly 3 variables
    if variables.len() == 3 {
        Some(Expression::Operation {
            name: "triple_integral".to_string(),
            args: vec![
                integrand,
                region,
                variables[0].clone(),
                variables[1].clone(),
                variables[2].clone(),
            ],
            span: None,
        })
    } else {
        None
    }
}

/// Attempt to infer modular congruence structure
///
/// Pattern: equiv(a, b * \pmod * n) -> congruent_mod(a, b, n)
fn try_infer_modular_congruence(expr: &Expression) -> Option<Expression> {
    // Check if top-level is equiv operation
    match expr {
        Expression::Operation { name, args, .. } if name == "equiv" && args.len() == 2 => {
            let left = &args[0];
            let right = &args[1];

            // Check if right side is a multiplication chain containing \pmod
            let right_terms = flatten_multiply(right);

            // Look for \pmod in the chain
            let pmod_pos = right_terms
                .iter()
                .position(|term| matches!(term, Expression::Object(s) if s == "\\pmod"))?;

            // Pattern: value * \pmod * modulus
            // Everything before \pmod is the value (should be just b)
            // Everything after \pmod is the modulus (should be just n)

            if pmod_pos == 0 || pmod_pos >= right_terms.len() - 1 {
                return None; // Invalid pattern
            }

            let value_terms = &right_terms[0..pmod_pos];
            let modulus_terms = &right_terms[pmod_pos + 1..];

            if modulus_terms.is_empty() {
                return None;
            }

            let value = rebuild_multiply(value_terms);
            let modulus = rebuild_multiply(modulus_terms);

            return Some(Expression::Operation {
                name: "congruent_mod".to_string(),
                args: vec![left.clone(), value, modulus],
                span: None,
            });
        }
        _ => {}
    }

    None
}

/// Attempt to infer statistics functions
///
/// Pattern: mathrm(Var) * function_call(X) -> variance(X)
/// Pattern: mathrm(Cov) * function_call(X, Y) -> covariance(X, Y)
/// Pattern: mathrm(Tr) * function_call(A) -> trace(A)
fn try_infer_statistics_functions(expr: &Expression) -> Option<Expression> {
    // Check if this is a multiplication of mathrm and function_call
    match expr {
        Expression::Operation { name, args, .. }
            if name == "scalar_multiply" && args.len() == 2 =>
        {
            // Check if left is mathrm(function_name)
            let func_name = match &args[0] {
                Expression::Operation {
                    name,
                    args: inner_args,
                    ..
                } if name == "mathrm" && inner_args.len() == 1 => match &inner_args[0] {
                    Expression::Object(s) => s.clone(),
                    _ => return None,
                },
                _ => return None,
            };

            // Check if right is function_call or just an argument
            let (op_name, new_args) = match func_name.as_str() {
                "Var" => {
                    // variance(X)
                    ("variance", vec![args[1].clone()])
                }
                "Cov" => {
                    // covariance(X, Y) - right side should be function_call(X, Y)
                    match &args[1] {
                        Expression::Operation {
                            name,
                            args: func_args,
                            ..
                        } if name == "function_call" && func_args.len() == 2 => {
                            // func_args[0] is X (treated as func name), [1] is Y
                            // This is actually (X, Y) parsed as function call
                            (
                                "covariance",
                                vec![func_args[0].clone(), func_args[1].clone()],
                            )
                        }
                        _ => return None,
                    }
                }
                "Tr" => {
                    // trace(A)
                    ("trace", vec![args[1].clone()])
                }
                "Re" => {
                    // real part: re(z)
                    ("re", vec![args[1].clone()])
                }
                "Im" => {
                    // imaginary part: im(z)
                    ("im", vec![args[1].clone()])
                }
                _ => return None,
            };

            return Some(Expression::Operation {
                name: op_name.to_string(),
                args: new_args,
                span: None,
            });
        }
        _ => {}
    }

    None
}

/// Attempt to infer curl structure from ∇ × vector patterns
///
/// Pattern: scalar_multiply(\nabla, vector_like) => curl(vector_like)
fn try_infer_curl(expr: &Expression) -> Option<Expression> {
    match expr {
        Expression::Operation { name, args, .. }
            if name == "scalar_multiply" && args.len() == 2 =>
        {
            if matches!(&args[0], Expression::Object(s) if s == "\\nabla" || s == "∇")
                && is_vector_like(&args[1])
            {
                return Some(Expression::Operation {
                    name: "curl".to_string(),
                    args: vec![args[1].clone()],
                    span: None,
                });
            }
        }
        _ => {}
    }

    None
}

// === Helper Functions ===

/// Rebuild a multiplication chain from a list of terms
///
/// Inverse of flatten_multiply: [a, b, c] -> scalar_multiply(scalar_multiply(a, b), c)
fn rebuild_multiply(terms: &[Expression]) -> Expression {
    if terms.is_empty() {
        return Expression::Object("".to_string());
    }

    if terms.len() == 1 {
        return terms[0].clone();
    }

    // Build left-associative chain: ((a * b) * c) * d
    let mut result = terms[0].clone();
    for term in &terms[1..] {
        result = Expression::Operation {
            name: "scalar_multiply".to_string(),
            args: vec![result, term.clone()],
            span: None,
        };
    }
    result
}

/// Flatten a nested scalar_multiply chain into a list of terms
///
/// Example: scalar_multiply(scalar_multiply(a, b), c) -> [a, b, c]
fn flatten_multiply(expr: &Expression) -> Vec<Expression> {
    match expr {
        Expression::Operation { name, args, .. }
            if name == "scalar_multiply" && args.len() == 2 =>
        {
            let mut result = flatten_multiply(&args[0]);
            result.extend(flatten_multiply(&args[1]));
            result
        }
        _ => vec![expr.clone()],
    }
}

/// Extract differential variables from a term sequence
///
/// Looks for pattern: mathrm(d) * var, mathrm(d) * var, ...
/// Returns the list of variables found
fn extract_differential_vars(terms: &[Expression], start: usize) -> Vec<Expression> {
    let mut variables = Vec::new();
    let mut i = start;

    while i + 1 < terms.len() {
        // Check if terms[i] is mathrm(d)
        let is_diff = is_mathrm_d(&terms[i]);

        if is_diff {
            // Next term is the variable
            variables.push(terms[i + 1].clone());
            i += 2;
        } else {
            // Pattern broken, stop
            break;
        }
    }

    variables
}

/// Check if an expression is mathrm(d)
fn is_mathrm_d(expr: &Expression) -> bool {
    match expr {
        Expression::Operation { name, args, .. } if name == "mathrm" && args.len() == 1 => {
            matches!(&args[0], Expression::Object(s) if s == "d")
        }
        _ => false,
    }
}

/// Heuristic check for vector-like expressions (bold, arrow, etc.)
fn is_vector_like(expr: &Expression) -> bool {
    match expr {
        Expression::Operation { name, .. }
            if matches!(
                name.as_str(),
                "vector_bold"
                    | "vector_arrow"
                    | "vector"
                    | "vector_hat"
                    | "vector_over"
                    | "vector_notation"
            ) =>
        {
            true
        }
        // Allow grouped vector expressions, e.g., parentheses wrapping a vector
        Expression::Operation { name, args, .. } if name == "group" && args.len() == 1 => {
            is_vector_like(&args[0])
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_latex;
    use crate::render::{build_default_context, render_expression, RenderTarget};

    #[test]
    fn test_infer_double_integral() {
        let latex = r"\iint_{D} f(x,y) \, \mathrm{d}x \, \mathrm{d}y";
        let flat_ast = parse_latex(latex).unwrap();
        let inferred = infer_templates(flat_ast);

        // Check it's now a double_integral operation
        match &inferred {
            Expression::Operation { name, args, .. } => {
                assert_eq!(name, "double_integral");
                assert_eq!(args.len(), 4);
            }
            _ => panic!("Expected double_integral operation"),
        }

        // Verify it renders correctly
        let ctx = build_default_context();
        let latex_out = render_expression(&inferred, &ctx, &RenderTarget::LaTeX);
        assert!(latex_out.contains("\\iint"));
        assert!(latex_out.contains("\\mathrm{d}x"));
        assert!(latex_out.contains("\\mathrm{d}y"));
    }

    #[test]
    fn test_infer_triple_integral() {
        let latex = r"\iiint_{V} f(x,y,z) \, \mathrm{d}x \, \mathrm{d}y \, \mathrm{d}z";
        let flat_ast = parse_latex(latex).unwrap();
        let inferred = infer_templates(flat_ast);

        match &inferred {
            Expression::Operation { name, args, .. } => {
                assert_eq!(name, "triple_integral");
                assert_eq!(args.len(), 5);
            }
            _ => panic!("Expected triple_integral operation"),
        }
    }

    #[test]
    fn test_infer_logical_implication() {
        let latex = r"P \Rightarrow Q";
        let flat_ast = parse_latex(latex).unwrap();
        let inferred = infer_templates(flat_ast);

        match &inferred {
            Expression::Operation { name, args, .. } => {
                assert_eq!(name, "implies");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected implies operation"),
        }

        // Verify it renders correctly
        let ctx = build_default_context();
        let latex_out = render_expression(&inferred, &ctx, &RenderTarget::LaTeX);
        assert!(latex_out.contains("\\Rightarrow"));
    }

    #[test]
    fn test_infer_iff() {
        let latex = r"P \Leftrightarrow Q";
        let flat_ast = parse_latex(latex).unwrap();
        let inferred = infer_templates(flat_ast);

        match &inferred {
            Expression::Operation { name, args, .. } => {
                assert_eq!(name, "iff");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected iff operation"),
        }
    }

    #[test]
    fn test_infer_exists_quantifier() {
        let latex = r"\exists x \colon x \in S";
        let flat_ast = parse_latex(latex).unwrap();
        let inferred = infer_templates(flat_ast);

        match &inferred {
            Expression::Operation { name, args, .. } => {
                assert_eq!(name, "exists");
                assert_eq!(args.len(), 2);
                // args[0] should be the variable
                // args[1] should be the body (in_set operation)
            }
            _ => panic!("Expected exists operation"),
        }
    }

    #[test]
    fn test_infer_forall_quantifier() {
        let latex = r"\forall x \colon x \in S";
        let flat_ast = parse_latex(latex).unwrap();
        let inferred = infer_templates(flat_ast);

        match &inferred {
            Expression::Operation { name, args, .. } => {
                assert_eq!(name, "forall");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected forall operation"),
        }
    }

    #[test]
    fn test_infer_modular_congruence() {
        let latex = r"a \equiv b \pmod{n}";
        let flat_ast = parse_latex(latex).unwrap();
        let inferred = infer_templates(flat_ast);

        match &inferred {
            Expression::Operation { name, args, .. } => {
                assert_eq!(name, "congruent_mod");
                assert_eq!(args.len(), 3); // left, right, modulus
            }
            _ => panic!("Expected congruent_mod operation"),
        }

        // Verify it renders correctly
        let ctx = build_default_context();
        let latex_out = render_expression(&inferred, &ctx, &RenderTarget::LaTeX);
        assert!(latex_out.contains("\\equiv"));
        assert!(latex_out.contains("\\pmod"));
    }

    #[test]
    fn test_infer_variance() {
        let latex = r"\mathrm{Var}(X)";
        let flat_ast = parse_latex(latex).unwrap();
        let inferred = infer_templates(flat_ast);

        match &inferred {
            Expression::Operation { name, args, .. } => {
                assert_eq!(name, "variance");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected variance operation"),
        }
    }

    #[test]
    fn test_infer_covariance() {
        let latex = r"\mathrm{Cov}(X, Y)";
        let flat_ast = parse_latex(latex).unwrap();
        let inferred = infer_templates(flat_ast);

        match &inferred {
            Expression::Operation { name, args, .. } => {
                assert_eq!(name, "covariance");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected covariance operation"),
        }
    }

    #[test]
    fn test_infer_curl() {
        let latex = r"\nabla \times \mathbf{B}";
        let flat_ast = parse_latex(latex).unwrap();
        let inferred = infer_templates(flat_ast);

        match &inferred {
            Expression::Operation { name, args, .. } => {
                assert_eq!(name, "curl");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected curl operation"),
        }
    }

    #[test]
    fn test_no_inference_fallback() {
        // Expression that doesn't match any pattern
        let latex = r"a + b";
        let flat_ast = parse_latex(latex).unwrap();
        let result = infer_templates(flat_ast.clone());

        // Should return unchanged
        // (Can't easily compare Expression equality, so just check it doesn't panic)
        assert!(matches!(result, Expression::Operation { name, .. } if name == "plus"));
    }
}
