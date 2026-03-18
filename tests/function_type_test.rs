//! Tests for first-class function types
//!
//! These tests verify that the type system can represent function types properly.

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

fn list_type(elem: Type) -> Type {
    Type::Data {
        type_name: "Type".to_string(),
        constructor: "List".to_string(),
        args: vec![elem],
    }
}

fn function_type(from: Type, to: Type) -> Type {
    Type::Function(Box::new(from), Box::new(to))
}

fn type_var(n: usize) -> Type {
    Type::Var(TypeVar::new(n))
}

#[test]
fn test_function_type_creation() {
    // ℝ → ℝ
    let sin_type = function_type(scalar(), scalar());

    match &sin_type {
        Type::Function(from, to) => {
            println!("Domain: {:?}", from);
            println!("Codomain: {:?}", to);
        }
        _ => panic!("Expected Function type"),
    }
}

#[test]
fn test_function_type_display() {
    // ℝ → ℝ
    let sin_type = function_type(scalar(), scalar());
    let display = format!("{}", sin_type);

    println!("sin : {}", sin_type);
    assert!(
        display.contains("→"),
        "Function type should display with arrow, got: {}",
        display
    );
    assert!(
        display.contains("Scalar"),
        "Should contain Scalar, got: {}",
        display
    );
}

#[test]
fn test_nested_function_type_display() {
    // (ℝ → ℝ) → ℝ (e.g., definite integral)
    let inner = function_type(scalar(), scalar());
    let higher_order = function_type(inner, scalar());

    let display = format!("{}", higher_order);
    println!("integral : {}", higher_order);

    // Should show two arrows
    let arrow_count = display.matches("→").count();
    assert!(
        arrow_count >= 2,
        "Higher-order function should have 2+ arrows, got: {}",
        display
    );
}

#[test]
fn test_curried_function_display() {
    // ℝ → ℝ → ℝ (curried binary function)
    let curried = function_type(scalar(), function_type(scalar(), scalar()));

    let display = format!("{}", curried);
    println!("add : {}", curried);

    let arrow_count = display.matches("→").count();
    assert_eq!(
        arrow_count, 2,
        "Curried function should have 2 arrows, got: {}",
        display
    );
}

#[test]
fn test_higher_order_function_type_display() {
    // map : (T → U) → List(T) → List(U)
    let t = type_var(0);
    let u = type_var(1);

    let f_type = function_type(t.clone(), u.clone());
    let list_t = list_type(t);
    let list_u = list_type(u);

    // (T → U) → List(T) → List(U)
    let map_type = function_type(f_type, function_type(list_t, list_u));

    let display = format!("{}", map_type);
    println!("map : {}", map_type);

    assert!(
        display.contains("→"),
        "map type should contain arrows, got: {}",
        display
    );
    assert!(
        display.contains("List"),
        "map type should contain List, got: {}",
        display
    );
}

#[test]
fn test_function_type_equality() {
    let t1 = function_type(scalar(), scalar());
    let t2 = function_type(scalar(), scalar());

    assert_eq!(t1, t2, "Same function types should be equal");
}

#[test]
fn test_function_type_inequality_domain() {
    let t1 = function_type(scalar(), scalar());
    let t2 = function_type(int_type(), scalar());

    assert_ne!(t1, t2, "Different domain types should not be equal");
}

#[test]
fn test_function_type_inequality_codomain() {
    let t1 = function_type(scalar(), scalar());
    let t2 = function_type(scalar(), int_type());

    assert_ne!(t1, t2, "Different codomain types should not be equal");
}

#[test]
fn test_function_type_with_type_variables() {
    // α → β
    let poly = function_type(type_var(0), type_var(1));
    let display = format!("{}", poly);

    println!("polymorphic : {}", poly);
    assert!(
        display.contains("α"),
        "Should contain type variable α, got: {}",
        display
    );
}
