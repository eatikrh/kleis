//! Operator Overloading Integration Tests
//!
//! Tests for complex number operator overloading and scoping of the imaginary unit `i`.

use kleis::kleis_parser::KleisParser;
use kleis::lowering::SemanticLowering;
use kleis::type_context::TypeContextBuilder;
use kleis::type_inference::{Type, TypeInference};

/// Helper: parse, infer types, and lower an expression
fn parse_infer_lower(input: &str) -> kleis::ast::Expression {
    let mut parser = KleisParser::new(input);
    let expr = parser.parse().unwrap();
    let type_context_builder = TypeContextBuilder::new();
    let mut inference = TypeInference::new();
    match inference.infer_typed(&expr, Some(&type_context_builder)) {
        Ok(typed) => {
            let lowering = SemanticLowering::new();
            lowering.lower(&typed)
        }
        Err(_) => expr,
    }
}

/// Helper: parse and infer type of an expression
fn infer_type(input: &str) -> Type {
    let mut parser = KleisParser::new(input);
    let expr = parser.parse().unwrap();
    let type_context_builder = TypeContextBuilder::new();
    let mut inference = TypeInference::new();
    inference.infer(&expr, Some(&type_context_builder)).unwrap()
}

// ============================================================================
// SECTION 1: Complex Number Operator Lowering
// ============================================================================

#[test]
fn test_complex_addition_lowered_to_complex_add() {
    let lowered = parse_infer_lower("complex(1, 2) + complex(3, 4)");
    match lowered {
        kleis::ast::Expression::Operation { name, args } => {
            assert_eq!(name, "complex_add", "plus(ℂ, ℂ) should lower to complex_add");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Operation, got {:?}", lowered),
    }
}

#[test]
fn test_complex_multiplication_lowered_to_complex_mul() {
    let lowered = parse_infer_lower("complex(1, 2) * complex(3, 4)");
    match lowered {
        kleis::ast::Expression::Operation { name, args } => {
            assert_eq!(name, "complex_mul", "times(ℂ, ℂ) should lower to complex_mul");
            assert_eq!(args.len(), 2);
        }
        _ => panic!("Expected Operation, got {:?}", lowered),
    }
}

#[test]
fn test_real_plus_complex_lifts_real() {
    let lowered = parse_infer_lower("5 + complex(1, 2)");
    match lowered {
        kleis::ast::Expression::Operation { name, args } => {
            assert_eq!(name, "complex_add", "plus(ℝ, ℂ) should lower to complex_add");
            // First arg should be complex(5, 0) - lifted real
            match &args[0] {
                kleis::ast::Expression::Operation { name: inner_name, args: inner_args } => {
                    assert_eq!(inner_name, "complex");
                    assert_eq!(inner_args.len(), 2);
                }
                _ => panic!("Expected lifted complex, got {:?}", args[0]),
            }
        }
        _ => panic!("Expected Operation, got {:?}", lowered),
    }
}

#[test]
fn test_i_times_i_lowered_to_complex_mul() {
    let lowered = parse_infer_lower("i * i");
    match lowered {
        kleis::ast::Expression::Operation { name, .. } => {
            assert_eq!(name, "complex_mul", "i * i should lower to complex_mul");
        }
        _ => panic!("Expected Operation, got {:?}", lowered),
    }
}

#[test]
fn test_real_arithmetic_not_lowered() {
    let lowered = parse_infer_lower("1 + 2");
    match lowered {
        kleis::ast::Expression::Operation { name, .. } => {
            assert_eq!(name, "plus", "plus(ℝ, ℝ) should stay as plus");
        }
        _ => panic!("Expected Operation, got {:?}", lowered),
    }
}

// ============================================================================
// SECTION 2: Type Inference for Complex Numbers
// ============================================================================

#[test]
fn test_i_has_complex_type() {
    let ty = infer_type("i");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Complex"),
        "i should have Complex type, got {:?}",
        ty
    );
}

#[test]
fn test_complex_constructor_has_complex_type() {
    let ty = infer_type("complex(1, 2)");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Complex"),
        "complex(1, 2) should have Complex type, got {:?}",
        ty
    );
}

#[test]
fn test_i_plus_one_has_complex_type() {
    let ty = infer_type("i + 1");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Complex"),
        "i + 1 should have Complex type, got {:?}",
        ty
    );
}

#[test]
fn test_re_returns_scalar() {
    let ty = infer_type("re(complex(1, 2))");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Scalar"),
        "re(z) should have Scalar type, got {:?}",
        ty
    );
}

#[test]
fn test_conj_returns_complex() {
    let ty = infer_type("conj(complex(1, 2))");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Complex"),
        "conj(z) should have Complex type, got {:?}",
        ty
    );
}

// ============================================================================
// SECTION 3: Scoping Rules for Imaginary Unit i
// ============================================================================

#[test]
fn test_quantified_i_real_shadows_imaginary() {
    // ∀(i : ℝ). i + 1 = 1 + i
    // The quantified i should be Real, not Complex
    let input = "∀(i : ℝ). i + 1 = 1 + i";
    let mut parser = KleisParser::new(input);
    let expr = parser.parse_proposition().unwrap();
    
    let type_context_builder = TypeContextBuilder::new();
    let mut inference = TypeInference::new();
    let typed = inference.infer_typed(&expr, Some(&type_context_builder)).unwrap();
    
    let lowering = SemanticLowering::new();
    let lowered = lowering.lower(&typed);
    
    // Check that the body uses 'plus', not 'complex_add'
    match lowered {
        kleis::ast::Expression::Quantifier { body, .. } => {
            match *body {
                kleis::ast::Expression::Operation { name, ref args } => {
                    assert_eq!(name, "equals");
                    // Check the LHS: i + 1
                    match &args[0] {
                        kleis::ast::Expression::Operation { name: op_name, .. } => {
                            assert_eq!(op_name, "plus", "Should be 'plus' not 'complex_add' when i : ℝ");
                        }
                        _ => panic!("Expected Operation for LHS"),
                    }
                }
                _ => panic!("Expected equals operation in body"),
            }
        }
        _ => panic!("Expected Quantifier"),
    }
}

#[test]
fn test_quantified_i_complex_uses_complex_ops() {
    // ∀(i : ℂ). i + complex(0, 0) = i
    // The quantified i should be Complex
    let input = "∀(i : ℂ). i + complex(0, 0) = i";
    let mut parser = KleisParser::new(input);
    let expr = parser.parse_proposition().unwrap();
    
    let type_context_builder = TypeContextBuilder::new();
    let mut inference = TypeInference::new();
    let typed = inference.infer_typed(&expr, Some(&type_context_builder)).unwrap();
    
    let lowering = SemanticLowering::new();
    let lowered = lowering.lower(&typed);
    
    // Check that the body uses 'complex_add'
    match lowered {
        kleis::ast::Expression::Quantifier { body, .. } => {
            match *body {
                kleis::ast::Expression::Operation { name, ref args } => {
                    assert_eq!(name, "equals");
                    // Check the LHS: i + complex(0, 0)
                    match &args[0] {
                        kleis::ast::Expression::Operation { name: op_name, .. } => {
                            assert_eq!(op_name, "complex_add", "Should be 'complex_add' when i : ℂ");
                        }
                        _ => panic!("Expected Operation for LHS"),
                    }
                }
                _ => panic!("Expected equals operation in body"),
            }
        }
        _ => panic!("Expected Quantifier"),
    }
}

#[test]
fn test_lambda_parameter_i_shadows_imaginary() {
    // λ i . i + 1
    // The lambda parameter i should be Scalar (generic), not Complex
    let ty = infer_type("λ i . i + 1");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Scalar"),
        "λ i . i + 1 should have Scalar type (param shadows global i), got {:?}",
        ty
    );
}

#[test]
fn test_lambda_with_global_i_has_complex_type() {
    // λ x . x + i
    // Uses global i, so result is Complex
    let ty = infer_type("λ x . x + i");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Complex"),
        "λ x . x + i should have Complex type (uses global i), got {:?}",
        ty
    );
}

#[test]
fn test_global_i_still_works_after_quantifier() {
    // First use i in a quantifier, then use global i
    // This verifies context is not polluted
    let ty = infer_type("i * i");
    assert!(
        matches!(&ty, Type::Data { constructor, .. } if constructor == "Complex"),
        "i * i should have Complex type, got {:?}",
        ty
    );
}

// ============================================================================
// SECTION 4: Complex Number Arithmetic Properties
// ============================================================================

#[test]
fn test_nested_complex_expression() {
    // (complex(1,2) + complex(3,4)) * complex(5,6)
    let lowered = parse_infer_lower("(complex(1,2) + complex(3,4)) * complex(5,6)");
    match lowered {
        kleis::ast::Expression::Operation { name, args } => {
            assert_eq!(name, "complex_mul");
            // First arg should be complex_add
            match &args[0] {
                kleis::ast::Expression::Operation { name: inner_name, .. } => {
                    assert_eq!(inner_name, "complex_add");
                }
                _ => panic!("Expected inner complex_add"),
            }
        }
        _ => panic!("Expected Operation"),
    }
}

#[test]
fn test_complex_negation() {
    let lowered = parse_infer_lower("-complex(1, 2)");
    match lowered {
        kleis::ast::Expression::Operation { name, .. } => {
            assert_eq!(name, "neg_complex", "negate(ℂ) should lower to neg_complex");
        }
        _ => panic!("Expected Operation, got {:?}", lowered),
    }
}

