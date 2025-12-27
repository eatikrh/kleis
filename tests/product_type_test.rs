//! Tests for first-class product types
//!
//! These tests verify that the type system can represent product types properly.

use kleis::type_inference::{Type, TypeVar};

/// Helper to create a simple Data type
fn scalar() -> Type {
    Type::Data {
        type_name: "Type".to_string(),
        constructor: "Scalar".to_string(),
        args: vec![],
    }
}

fn int_type() -> Type {
    Type::Data {
        type_name: "Type".to_string(),
        constructor: "Int".to_string(),
        args: vec![],
    }
}

fn string_type() -> Type {
    Type::String
}

fn product_type(types: Vec<Type>) -> Type {
    Type::Product(types)
}

fn type_var(n: usize) -> Type {
    Type::Var(TypeVar(n))
}

#[test]
fn test_product_type_creation() {
    // ℝ × ℝ
    let pair = product_type(vec![scalar(), scalar()]);

    match &pair {
        Type::Product(types) => {
            assert_eq!(types.len(), 2);
            println!("Product has {} elements", types.len());
        }
        _ => panic!("Expected Product type"),
    }
}

#[test]
fn test_product_type_display() {
    // ℝ × ℝ
    let pair = product_type(vec![scalar(), scalar()]);
    let display = format!("{}", pair);

    println!("pair : {}", pair);
    assert!(
        display.contains("×"),
        "Product type should display with ×, got: {}",
        display
    );
}

#[test]
fn test_triple_product_display() {
    // ℝ × Int × String
    let triple = product_type(vec![scalar(), int_type(), string_type()]);

    let display = format!("{}", triple);
    println!("triple : {}", triple);

    // Should show two × symbols
    let cross_count = display.matches("×").count();
    assert_eq!(
        cross_count, 2,
        "Triple product should have 2 × symbols, got: {}",
        display
    );
}

#[test]
fn test_product_type_equality() {
    let t1 = product_type(vec![scalar(), scalar()]);
    let t2 = product_type(vec![scalar(), scalar()]);

    assert_eq!(t1, t2, "Same product types should be equal");
}

#[test]
fn test_product_type_inequality_length() {
    let t1 = product_type(vec![scalar(), scalar()]);
    let t2 = product_type(vec![scalar(), scalar(), scalar()]);

    assert_ne!(t1, t2, "Different length products should not be equal");
}

#[test]
fn test_product_type_inequality_elements() {
    let t1 = product_type(vec![scalar(), scalar()]);
    let t2 = product_type(vec![scalar(), int_type()]);

    assert_ne!(t1, t2, "Different element types should not be equal");
}

#[test]
fn test_product_type_with_type_variables() {
    // α × β
    let poly = product_type(vec![type_var(0), type_var(1)]);
    let display = format!("{}", poly);

    println!("polymorphic : {}", poly);
    assert!(
        display.contains("α"),
        "Should contain type variable α, got: {}",
        display
    );
}

#[test]
fn test_nested_product_in_function() {
    // (ℝ × ℝ) → ℝ - like add : ℝ × ℝ → ℝ
    let domain = product_type(vec![scalar(), scalar()]);
    let add_type = Type::Function(Box::new(domain), Box::new(scalar()));

    let display = format!("{}", add_type);
    println!("add : {}", add_type);

    assert!(
        display.contains("×"),
        "Should contain × for product domain, got: {}",
        display
    );
    assert!(
        display.contains("→"),
        "Should contain → for function, got: {}",
        display
    );
}
