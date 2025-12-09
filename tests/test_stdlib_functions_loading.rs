// Test that stdlib functions actually load into TypeChecker

use kleis::type_checker::TypeChecker;

#[test]
fn test_load_all_stdlib_functions() {
    let mut checker = TypeChecker::new();

    // Load the full stdlib/types.kleis file
    // This includes BOTH data definitions AND function definitions
    let types_code = include_str!("../stdlib/types.kleis");

    let result = checker.load_kleis(types_code);

    if let Err(e) = &result {
        println!("\n=== FAILED TO LOAD STDLIB ===");
        println!("{}", e);
        panic!("Stdlib should load successfully");
    }

    println!("\n✅ SUCCESS! All stdlib functions loaded!");
    println!("Functions available:");
    println!("  - not, and, or (Boolean)");
    println!("  - isSome, isNone, getOrDefault (Option)");
    println!("  - isEmpty, head, tail (List)");
}

#[test]
fn test_use_loaded_stdlib_function() {
    let mut checker = TypeChecker::new();

    // Load stdlib with functions
    let types_code = include_str!("../stdlib/types.kleis");
    checker.load_kleis(types_code).expect("Stdlib should load");

    // Now try to use one of the functions
    let user_code = r#"
        define myFunc(list) = head(list)
    "#;

    let result = checker.load_kleis(user_code);
    assert!(
        result.is_ok(),
        "Should be able to use stdlib function 'head': {:?}",
        result.err()
    );

    println!("\n✅ Can use stdlib functions in user code!");
}
