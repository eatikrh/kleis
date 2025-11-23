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
    // Try each pattern matcher in priority order
    // If any succeeds, return the structured version
    // If all fail, return original flat structure
    
    try_infer_double_integral(&expr)
        .or_else(|| try_infer_triple_integral(&expr))
        // Add more pattern matchers here as needed:
        // .or_else(|| try_infer_limit(&expr))
        // .or_else(|| try_infer_sum_bounds(&expr))
        // .or_else(|| try_infer_quantifier(&expr))
        .unwrap_or(expr) // Fallback: keep flat if no pattern matches
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
        Expression::Operation { name, args } if name == "sub" && args.len() == 2 => {
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
            args: vec![integrand, region, variables[0].clone(), variables[1].clone()],
        })
    } else {
        None
    }
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
        Expression::Operation { name, args } if name == "sub" && args.len() == 2 => {
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
        })
    } else {
        None
    }
}

// === Helper Functions ===

/// Flatten a nested scalar_multiply chain into a list of terms
/// 
/// Example: scalar_multiply(scalar_multiply(a, b), c) -> [a, b, c]
fn flatten_multiply(expr: &Expression) -> Vec<Expression> {
    match expr {
        Expression::Operation { name, args } if name == "scalar_multiply" && args.len() == 2 => {
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
        Expression::Operation { name, args } if name == "mathrm" && args.len() == 1 => {
            matches!(&args[0], Expression::Object(s) if s == "d")
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
            Expression::Operation { name, args } => {
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
            Expression::Operation { name, args } => {
                assert_eq!(name, "triple_integral");
                assert_eq!(args.len(), 5);
            }
            _ => panic!("Expected triple_integral operation"),
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

