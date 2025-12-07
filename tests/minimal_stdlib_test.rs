///! Test loading minimal stdlib that matches current parser capabilities
use kleis::type_checker::TypeChecker;

#[test]
fn test_minimal_stdlib_parses() {
    let mut checker = TypeChecker::new();

    let minimal = include_str!("../stdlib/minimal_prelude.kleis");
    let result = checker.load_kleis(minimal);

    if let Err(e) = &result {
        println!("Parse error: {}", e);
    }

    assert!(
        result.is_ok(),
        "Failed to parse minimal stdlib: {:?}",
        result.err()
    );
}

#[test]
fn test_minimal_stdlib_has_operations() {
    let mut checker = TypeChecker::new();

    let minimal = include_str!("../stdlib/minimal_prelude.kleis");
    checker
        .load_kleis(minimal)
        .expect("Failed to load minimal stdlib");

    // Check that operations are available
    let abs_types = checker.types_supporting("abs");
    println!("Types supporting 'abs': {:?}", abs_types);
    assert!(!abs_types.is_empty(), "Expected 'abs' operation");

    let transpose_types = checker.types_supporting("transpose");
    println!("Types supporting 'transpose': {:?}", transpose_types);
    assert!(
        !transpose_types.is_empty(),
        "Expected 'transpose' operation"
    );
}
