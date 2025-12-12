#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
// Test to reproduce polymorphic function loading error

use kleis::type_checker::TypeChecker;

#[test]
fn test_load_simple_polymorphic_function() {
    let mut checker = TypeChecker::new();

    // First load the data types
    let types_code = r#"
        data Option(T) = None | Some(value: T)
        data List(T) = Nil | Cons(head: T, tail: List(T))
    "#;

    let result = checker.load_data_types(types_code);
    assert!(
        result.is_ok(),
        "Failed to load data types: {:?}",
        result.err()
    );

    // Now try to load a polymorphic function
    let func_code = r#"
        define head(list) = match list {
          Nil => None
          | Cons(h, _) => Some(h)
        }
    "#;

    let result = checker.load_kleis(func_code);

    if let Err(e) = &result {
        println!("ERROR: {}", e);
        println!("\nThis is the polymorphism limitation we need to fix!");
    }

    // This will fail with current implementation
    // assert!(result.is_ok(), "Failed to load function: {:?}", result.err());
}

#[test]
fn test_load_stdlib_with_functions() {
    // Try to load full stdlib types.kleis
    let types_code = include_str!("../stdlib/types.kleis");

    let mut checker = TypeChecker::new();
    let result = checker.load_kleis(types_code);

    if let Err(e) = &result {
        println!("\n=== STDLIB LOADING ERROR ===");
        println!("{}", e);
        println!("\nThis shows exactly where polymorphic functions fail");
    }
}
