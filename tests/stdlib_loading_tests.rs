#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
///! Integration tests for standard library loading
///!
///! Tests that stdlib/minimal_prelude.kleis and stdlib/matrices.kleis
///! load correctly and populate the type context.
///!
///! **Note:** Currently testing with minimal_prelude.kleis because the full
///! prelude.kleis uses advanced syntax the parser doesn't support yet.
///! Once parser is extended (Phase 2), these tests will use full prelude.
use kleis::type_checker::TypeChecker;

#[test]
fn test_stdlib_loads_successfully() {
    let result = TypeChecker::with_stdlib();
    assert!(
        result.is_ok(),
        "Failed to load stdlib: {}",
        result.err().unwrap_or_default()
    );
}

#[test]
fn test_stdlib_has_operations() {
    let checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // Check that operations from minimal_prelude are available
    let abs_types = checker.types_supporting("abs");
    assert!(
        !abs_types.is_empty(),
        "Expected 'abs' operation to be defined in stdlib"
    );

    let floor_types = checker.types_supporting("floor");
    assert!(
        !floor_types.is_empty(),
        "Expected 'floor' operation to be defined in stdlib"
    );
}

#[test]
fn test_stdlib_has_matrix_operations() {
    let checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // Check matrix operations from stdlib/matrices.kleis
    let transpose_types = checker.types_supporting("transpose");
    assert!(
        !transpose_types.is_empty(),
        "Expected 'transpose' operation to be defined in stdlib"
    );

    // matrix_add is the dimension-checking version (ADR-016 compliant)
    let add_types = checker.types_supporting("matrix_add");
    assert!(
        !add_types.is_empty(),
        "Expected 'matrix_add' operation to be defined in stdlib"
    );

    let multiply_types = checker.types_supporting("multiply");
    assert!(
        !multiply_types.is_empty(),
        "Expected 'multiply' operation to be defined in stdlib"
    );
}

#[test]
fn test_stdlib_has_numeric_implementation() {
    let checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    // Check that ℝ implements Numeric operations
    let types = checker.types_supporting("abs");
    println!("Types supporting 'abs': {:?}", types);
    // We verify that SOME types support it (should include ℝ)
    assert!(!types.is_empty());
}

#[test]
fn test_empty_checker_has_no_operations() {
    let checker = TypeChecker::new();

    // Empty checker should have no operations
    let add_types = checker.types_supporting("+");
    assert!(
        add_types.is_empty(),
        "Empty checker should have no operations, but found: {:?}",
        add_types
    );
}

#[test]
fn test_incremental_loading() {
    let mut checker = TypeChecker::new();

    // Load minimal prelude
    let prelude = include_str!("../stdlib/minimal_prelude.kleis");
    let result = checker.load_kleis(prelude);
    assert!(
        result.is_ok(),
        "Failed to load minimal prelude: {:?}",
        result.err()
    );

    // Check operations are available
    let types = checker.types_supporting("abs");
    assert!(
        !types.is_empty(),
        "Expected operations after loading minimal prelude"
    );

    // Load matrices
    let matrices = include_str!("../stdlib/matrices.kleis");
    let result = checker.load_kleis(matrices);
    assert!(
        result.is_ok(),
        "Failed to load matrices: {:?}",
        result.err()
    );

    // Check matrix operations are available
    let transpose_types = checker.types_supporting("transpose");
    assert!(
        !transpose_types.is_empty(),
        "Expected matrix operations after loading matrices"
    );
}

#[test]
fn test_stdlib_parse_errors_handled() {
    let mut checker = TypeChecker::new();

    // Try to load invalid Kleis code
    let invalid_code = "this is not valid kleis code!!!";
    let result = checker.load_kleis(invalid_code);

    assert!(result.is_err(), "Expected parse error for invalid code");
    let error_msg = result.err().unwrap();
    assert!(
        error_msg.contains("Parse error"),
        "Expected 'Parse error' in message, got: {}",
        error_msg
    );
}
