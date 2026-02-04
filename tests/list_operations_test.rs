//! Tests for Higher-Order List Operations
//!
//! Tests the new list operations added to evaluator.rs:
//! - reverse / builtin_reverse
//! - foldr / builtin_foldr
//! - sum / builtin_sum
//! - product / builtin_product
//! - all / builtin_all
//! - any / builtin_any
//!
//! These operations are formalized in stdlib/lists.kleis

#![allow(unused_imports)]

use kleis::ast::Expression;
use kleis::evaluator::Evaluator;
use kleis::kleis_parser::parse_kleis_program;

// ============================================================================
// reverse tests
// ============================================================================

#[test]
fn test_reverse_empty_list() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = reverse([])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "reverse([]) should work: {:?}", result);
    
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::List(ref v) if v.is_empty()),
            "reverse([]) should be []: {:?}", expr);
}

#[test]
fn test_reverse_single_element() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = reverse([42])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "reverse([42]) should work: {:?}", result);
    
    let expr = result.unwrap();
    if let Expression::List(v) = expr {
        assert_eq!(v.len(), 1);
        assert!(matches!(&v[0], Expression::Const(s) if s == "42"));
    } else {
        panic!("Expected List, got: {:?}", expr);
    }
}

#[test]
fn test_reverse_multiple_elements() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = reverse([1, 2, 3])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "reverse([1,2,3]) should work: {:?}", result);
    
    let expr = result.unwrap();
    if let Expression::List(v) = expr {
        assert_eq!(v.len(), 3);
        assert!(matches!(&v[0], Expression::Const(s) if s == "3"));
        assert!(matches!(&v[1], Expression::Const(s) if s == "2"));
        assert!(matches!(&v[2], Expression::Const(s) if s == "1"));
    } else {
        panic!("Expected List, got: {:?}", expr);
    }
}

// ============================================================================
// sum tests
// ============================================================================

#[test]
fn test_sum_empty_list() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = sum([])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "sum([]) should work: {:?}", result);
    
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Const(ref s) if s == "0"),
            "sum([]) should be 0: {:?}", expr);
}

#[test]
fn test_sum_single_element() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = sum([42])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "sum([42]) should work: {:?}", result);
    
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Const(ref s) if s == "42"),
            "sum([42]) should be 42: {:?}", expr);
}

#[test]
fn test_sum_multiple_elements() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = sum([1, 2, 3, 4])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "sum([1,2,3,4]) should work: {:?}", result);
    
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Const(ref s) if s == "10"),
            "sum([1,2,3,4]) should be 10: {:?}", expr);
}

// ============================================================================
// product tests
// ============================================================================

#[test]
fn test_product_empty_list() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = product([])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "product([]) should work: {:?}", result);
    
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Const(ref s) if s == "1"),
            "product([]) should be 1: {:?}", expr);
}

#[test]
fn test_product_single_element() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = product([7])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "product([7]) should work: {:?}", result);
    
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Const(ref s) if s == "7"),
            "product([7]) should be 7: {:?}", expr);
}

#[test]
fn test_product_multiple_elements() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = product([2, 3, 4])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "product([2,3,4]) should work: {:?}", result);
    
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Const(ref s) if s == "24"),
            "product([2,3,4]) should be 24: {:?}", expr);
}

// ============================================================================
// all tests
// ============================================================================

#[test]
fn test_all_empty_list() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = all(λ x . x > 0, [])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "all on empty list should work: {:?}", result);
    
    let expr = result.unwrap();
    // all on empty list is true (vacuous truth)
    assert!(matches!(expr, Expression::Object(ref s) if s == "true" || s == "True"),
            "all(_, []) should be true: {:?}", expr);
}

#[test]
fn test_all_true_case() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = all(λ x . x > 0, [1, 2, 3])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "all(λ x . x > 0, [1,2,3]) should work: {:?}", result);
    
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Object(ref s) if s == "true" || s == "True"),
            "all(λ x . x > 0, [1,2,3]) should be true: {:?}", expr);
}

#[test]
fn test_all_false_case() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = all(λ x . x > 0, [1, 0, 3])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "all(λ x . x > 0, [1,0,3]) should work: {:?}", result);
    
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Object(ref s) if s == "false" || s == "False"),
            "all(λ x . x > 0, [1,0,3]) should be false: {:?}", expr);
}

// ============================================================================
// any tests
// ============================================================================

#[test]
fn test_any_empty_list() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = any(λ x . x > 0, [])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "any on empty list should work: {:?}", result);
    
    let expr = result.unwrap();
    // any on empty list is false
    assert!(matches!(expr, Expression::Object(ref s) if s == "false" || s == "False"),
            "any(_, []) should be false: {:?}", expr);
}

#[test]
fn test_any_true_case() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = any(λ x . x > 0, [0, 0, 3])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "any(λ x . x > 0, [0,0,3]) should work: {:?}", result);
    
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Object(ref s) if s == "true" || s == "True"),
            "any(λ x . x > 0, [0,0,3]) should be true: {:?}", expr);
}

#[test]
fn test_any_false_case() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = any(λ x . x > 0, [0, 0, 0])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "any(λ x . x > 0, [0,0,0]) should work: {:?}", result);
    
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Object(ref s) if s == "false" || s == "False"),
            "any(λ x . x > 0, [0,0,0]) should be false: {:?}", expr);
}

// ============================================================================
// foldr tests
// ============================================================================

#[test]
fn test_foldr_empty_list() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = foldr(λ x . λ acc . x + acc, 0, [])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "foldr on empty list should work: {:?}", result);
    
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Const(ref s) if s == "0"),
            "foldr(f, 0, []) should be 0: {:?}", expr);
}

#[test]
fn test_foldr_sum() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = foldr(λ x . λ acc . x + acc, 0, [1, 2, 3])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "foldr sum should work: {:?}", result);
    
    // foldr(+, 0, [1,2,3]) = 1 + (2 + (3 + 0)) = 6
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Const(ref s) if s == "6"),
            "foldr(+, 0, [1,2,3]) should be 6: {:?}", expr);
}

// ============================================================================
// Alias tests (builtin_* variants)
// ============================================================================

#[test]
fn test_builtin_reverse_alias() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = builtin_reverse([1, 2, 3])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "builtin_reverse should work: {:?}", result);
    
    let expr = result.unwrap();
    if let Expression::List(v) = expr {
        assert_eq!(v.len(), 3);
        assert!(matches!(&v[0], Expression::Const(s) if s == "3"));
    } else {
        panic!("Expected List, got: {:?}", expr);
    }
}

#[test]
fn test_builtin_sum_alias() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = builtin_sum([1, 2, 3])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "builtin_sum should work: {:?}", result);
    
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Const(ref s) if s == "6"),
            "builtin_sum([1,2,3]) should be 6: {:?}", expr);
}

#[test]
fn test_builtin_product_alias() {
    let mut eval = Evaluator::new();
    
    let code = r#"
        define test = builtin_product([2, 3, 4])
    "#;
    let program = parse_kleis_program(code).unwrap();
    eval.load_program(&program).unwrap();
    
    let result = eval.eval_concrete(&Expression::Object("test".to_string()));
    assert!(result.is_ok(), "builtin_product should work: {:?}", result);
    
    let expr = result.unwrap();
    assert!(matches!(expr, Expression::Const(ref s) if s == "24"),
            "builtin_product([2,3,4]) should be 24: {:?}", expr);
}

