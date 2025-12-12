#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
// Test loading multiple functions in sequence

use kleis::type_checker::TypeChecker;

#[test]
fn test_load_all_boolean_functions() {
    let mut checker = TypeChecker::new();

    checker.load_data_types("data Bool = True | False").unwrap();

    let code = r#"
        define not(b) = match b {
          True => False
          | False => True
        }
        
        define and(b1, b2) = match b1 {
          False => False
          | True => b2
        }
        
        define or(b1, b2) = match b1 {
          True => True
          | False => b2
        }
    "#;

    let result = checker.load_kleis(code);
    assert!(
        result.is_ok(),
        "Boolean functions should load: {:?}",
        result.err()
    );
    println!("✅ All 3 boolean functions loaded!");
}

#[test]
fn test_load_bool_then_option_functions() {
    let mut checker = TypeChecker::new();

    let types = r#"
        data Bool = True | False
        data Option(T) = None | Some(value: T)
    "#;
    checker.load_data_types(types).unwrap();

    // First load boolean functions
    let bool_funcs = r#"
        define not(b) = match b {
          True => False
          | False => True
        }
    "#;
    checker.load_kleis(bool_funcs).unwrap();
    println!("✅ Boolean function loaded");

    // Then load option functions
    let option_funcs = r#"
        define isSome(opt) = match opt {
          None => False
          | Some(_) => True
        }
    "#;

    let result = checker.load_kleis(option_funcs);
    if let Err(e) = &result {
        println!("\n❌ ERROR loading option function after boolean:");
        println!("{}", e);
    }
    assert!(
        result.is_ok(),
        "Option function should load after boolean: {:?}",
        result.err()
    );
    println!("✅ Option function loaded after boolean!");
}
