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

    // Note: transpose is now in matrices.kleis, not minimal_prelude.kleis
    // This is the correct organization (all Matrix operations together)
    let plus_types = checker.types_supporting("plus");
    println!("Types supporting 'plus': {:?}", plus_types);
    assert!(!plus_types.is_empty(), "Expected 'plus' operation");
}
