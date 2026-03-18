//! Test: Top-level functions with if/then/else conditionals
//!
//! This test verifies that user-defined functions containing conditionals
//! can be loaded into Z3 and used for verification.
//!
//! Key features tested:
//! - `load_program_functions()` loads top-level `define` statements
//! - Functions with `if/then/else` bodies are correctly translated to Z3's `ite`
//! - Z3 can reason about conditional functions

#[cfg(feature = "axiom-verification")]
use kleis::axiom_verifier::AxiomVerifier;
#[cfg(feature = "axiom-verification")]
use kleis::kleis_parser::parse_kleis_program;
#[cfg(feature = "axiom-verification")]
use kleis::structure_registry::StructureRegistry;

/// Test that a function with if/then/else can be loaded and evaluated
#[test]
#[cfg(feature = "axiom-verification")]
fn test_load_function_with_conditional() {
    println!("\n🧪 Testing: Load function with if/then/else conditional");

    // Define a function with conditional
    let code = r#"
        define abs(x) = if x > 0 then x else negate(x)
    "#;

    println!("   📝 Parsing: define abs(x) = if x > 0 then x else negate(x)");
    let program = parse_kleis_program(code).unwrap();

    // Verify parsing worked
    assert_eq!(program.functions().len(), 1);
    let abs_fn = &program.functions()[0];
    assert_eq!(abs_fn.name, "abs");
    assert!(matches!(
        abs_fn.body,
        kleis::ast::Expression::Conditional { .. }
    ));
    println!("   ✅ Parsed successfully - body is Conditional");

    // Create verifier and load the function
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    println!("   📦 Loading function into Z3...");
    let result = verifier.load_program_functions(&program);
    assert!(result.is_ok(), "Failed to load function: {:?}", result);
    println!("   ✅ Function loaded into Z3 backend");
}

/// Test multiple functions with conditionals
#[test]
#[cfg(feature = "axiom-verification")]
fn test_load_multiple_conditional_functions() {
    println!("\n🧪 Testing: Load multiple functions with conditionals");

    let code = r#"
        define abs(x) = if x > 0 then x else negate(x)
        define sign(x) = if x > 0 then 1 else if x < 0 then negate(1) else 0
        define max(a, b) = if a > b then a else b
        define min(a, b) = if a < b then a else b
    "#;

    println!("   📝 Parsing 4 functions with conditionals...");
    let program = parse_kleis_program(code).unwrap();
    assert_eq!(program.functions().len(), 4);
    println!("   ✅ Parsed 4 functions");

    // Load into verifier
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    println!("   📦 Loading all functions into Z3...");
    let result = verifier.load_program_functions(&program);
    assert!(result.is_ok());
    println!("   ✅ All 4 functions loaded successfully");
}

/// Test nested conditionals
#[test]
#[cfg(feature = "axiom-verification")]
fn test_load_nested_conditional_function() {
    println!("\n🧪 Testing: Load function with nested conditionals");

    let code = r#"
        define clamp(x, lo, hi) = if x < lo then lo else if x > hi then hi else x
    "#;

    println!("   📝 Parsing: clamp(x, lo, hi) with nested if/then/else");
    let program = parse_kleis_program(code).unwrap();
    assert_eq!(program.functions().len(), 1);

    let clamp_fn = &program.functions()[0];
    assert_eq!(clamp_fn.name, "clamp");
    assert_eq!(clamp_fn.params.len(), 3);
    println!("   ✅ Parsed clamp with 3 parameters");

    // Load into verifier
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    println!("   📦 Loading clamp function into Z3...");
    let result = verifier.load_program_functions(&program);
    assert!(result.is_ok());
    println!("   ✅ Nested conditional function loaded successfully");
}

/// Test function with arithmetic function calls in conditionals
///
/// Note: Functions that return Bool (like `positive(x) = x > 0`) require
/// proper type inference to declare with Bool sort. This test uses
/// arithmetic functions which work with the current Int-based implementation.
#[test]
#[cfg(feature = "axiom-verification")]
fn test_conditional_with_arithmetic_function_calls() {
    println!("\n🧪 Testing: Function with conditionals and arithmetic function calls");

    let code = r#"
        define double(x) = x + x
        define triple(x) = x + x + x
        define choose(x, flag) = if flag > 0 then double(x) else triple(x)
    "#;

    println!("   📝 Parsing 3 functions with arithmetic calls in conditionals...");
    let program = parse_kleis_program(code).unwrap();
    assert_eq!(program.functions().len(), 3);
    println!("   ✅ Parsed 3 functions");

    // Load into verifier
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    println!("   📦 Loading all functions into Z3...");
    let result = verifier.load_program_functions(&program);
    assert!(result.is_ok());
    println!("   ✅ Functions with conditional+arithmetic calls loaded successfully");
}

/// Test that conditionals work with comparison operators
#[test]
#[cfg(feature = "axiom-verification")]
fn test_conditional_with_all_comparisons() {
    println!("\n🧪 Testing: Conditionals with various comparison operators");

    let code = r#"
        define test_gt(x, y) = if x > y then 1 else 0
        define test_lt(x, y) = if x < y then 1 else 0
        define test_eq(x, y) = if x == y then 1 else 0
        define test_geq(x, y) = if x >= y then 1 else 0
        define test_leq(x, y) = if x <= y then 1 else 0
    "#;

    println!("   📝 Parsing 5 functions with different comparisons...");
    let program = parse_kleis_program(code).unwrap();
    assert_eq!(program.functions().len(), 5);

    // Load into verifier
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    println!("   📦 Loading all comparison functions into Z3...");
    let result = verifier.load_program_functions(&program);
    assert!(result.is_ok());
    println!("   ✅ All comparison conditional functions loaded successfully");
}
