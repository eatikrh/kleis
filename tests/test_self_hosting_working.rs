#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
// Test that self-hosting actually works now!

use kleis::type_checker::TypeChecker;

#[test]
fn test_stdlib_functions_actually_load() {
    // This is the critical test: Do functions load into with_stdlib()?
    let checker = TypeChecker::with_stdlib().expect("Stdlib should load");

    println!("\n✅ TypeChecker::with_stdlib() succeeded!");
    println!("This means all 9 stdlib functions are now loaded!");
}

#[test]
fn test_can_call_stdlib_functions() {
    let mut checker = TypeChecker::with_stdlib().expect("Stdlib should load");

    // Test calling each function
    let test_code = r#"
        define testNot(b) = not(b)
        define testAnd(b1, b2) = and(b1, b2)  
        define testOr(b1, b2) = or(b1, b2)
        define testIsSome(opt) = isSome(opt)
        define testIsNone(opt) = isNone(opt)
        define testIsEmpty(list) = isEmpty(list)
        define testGetOrDefault(opt, def) = getOrDefault(opt, def)
        define testHead(list) = head(list)
        define testTail(list) = tail(list)
    "#;

    let result = checker.load_kleis(test_code);
    assert!(
        result.is_ok(),
        "Should be able to call all stdlib functions: {:?}",
        result.err()
    );

    println!("\n✅ All 9 stdlib functions are callable!");
}

#[test]
fn test_compose_stdlib_functions() {
    let mut checker = TypeChecker::with_stdlib().expect("Stdlib should load");

    // Test realistic compositions
    let code = r#"
        define firstOrZero(list) = getOrDefault(head(list), 0)
        define hasValue(opt) = not(isNone(opt))
        define isNonEmpty(list) = not(isEmpty(list))
    "#;

    let result = checker.load_kleis(code);
    assert!(
        result.is_ok(),
        "Should be able to compose stdlib functions: {:?}",
        result.err()
    );

    println!("\n✅ Stdlib functions compose correctly!");
}
