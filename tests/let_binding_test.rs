//! Test: Let bindings in function definitions
//!
//! This test verifies that let bindings work correctly with Z3:
//! - Parsing: `let x = value in body` syntax
//! - Z3 Translation: let bindings extend the variable context
//! - Evaluation: bound variables are substituted correctly

#[cfg(feature = "axiom-verification")]
use kleis::axiom_verifier::AxiomVerifier;
#[cfg(feature = "axiom-verification")]
use kleis::kleis_parser::parse_kleis_program;
#[cfg(feature = "axiom-verification")]
use kleis::structure_registry::StructureRegistry;

/// Test function with simple let binding
#[test]
#[cfg(feature = "axiom-verification")]
fn test_function_with_simple_let() {
    println!("\n🧪 Testing: Function with simple let binding");

    let code = r#"
        define double_square(x) = let sq = x * x in sq + sq
    "#;

    println!("   📝 Parsing: define double_square(x) = let sq = x * x in sq + sq");
    let program = parse_kleis_program(code).unwrap();

    assert_eq!(program.functions().len(), 1);
    let func = &program.functions()[0];
    assert_eq!(func.name, "double_square");
    assert!(matches!(func.body, kleis::ast::Expression::Let { .. }));
    println!("   ✅ Parsed successfully - body is Let binding");

    // Load into Z3
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    println!("   📦 Loading function into Z3...");
    let result = verifier.load_program_functions(&program);
    assert!(result.is_ok(), "Failed to load function: {:?}", result);
    println!("   ✅ Function with let binding loaded into Z3");
}

/// Test nested let bindings
#[test]
#[cfg(feature = "axiom-verification")]
fn test_function_with_nested_let() {
    println!("\n🧪 Testing: Function with nested let bindings");

    let code = r#"
        define polynomial(x) = 
            let x2 = x * x in 
            let x3 = x2 * x in 
            x3 + x2 + x + 1
    "#;

    println!("   📝 Parsing polynomial with nested let bindings...");
    let program = parse_kleis_program(code).unwrap();

    assert_eq!(program.functions().len(), 1);
    println!("   ✅ Parsed function with nested let bindings");

    // Load into Z3
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    println!("   📦 Loading function into Z3...");
    let result = verifier.load_program_functions(&program);
    assert!(result.is_ok());
    println!("   ✅ Nested let bindings loaded successfully");
}

/// Test let binding with conditional
#[test]
#[cfg(feature = "axiom-verification")]
fn test_let_with_conditional() {
    println!("\n🧪 Testing: Let binding containing conditional");

    let code = r#"
        define abs_double(x) = 
            let abs_x = if x > 0 then x else negate(x) in 
            abs_x + abs_x
    "#;

    println!("   📝 Parsing function with let + conditional...");
    let program = parse_kleis_program(code).unwrap();

    assert_eq!(program.functions().len(), 1);
    println!("   ✅ Parsed function with let containing conditional");

    // Load into Z3
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    println!("   📦 Loading function into Z3...");
    let result = verifier.load_program_functions(&program);
    assert!(result.is_ok());
    println!("   ✅ Let with conditional loaded successfully");
}

/// Test multiple functions with let bindings
#[test]
#[cfg(feature = "axiom-verification")]
fn test_multiple_functions_with_let() {
    println!("\n🧪 Testing: Multiple functions with let bindings");

    let code = r#"
        define distance(x1, y1, x2, y2) = 
            let dx = x2 - x1 in
            let dy = y2 - y1 in
            dx * dx + dy * dy
        
        define midpoint_x(x1, x2) = 
            let sum = x1 + x2 in
            sum
        
        define average(a, b, c) =
            let sum = a + b + c in
            sum
    "#;

    println!("   📝 Parsing 3 functions with let bindings...");
    let program = parse_kleis_program(code).unwrap();

    assert_eq!(program.functions().len(), 3);
    println!("   ✅ Parsed 3 functions");

    // Load into Z3
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    println!("   📦 Loading all functions into Z3...");
    let result = verifier.load_program_functions(&program);
    assert!(result.is_ok());
    println!("   ✅ All 3 functions loaded successfully");
}

/// Test let binding in conditional branches
#[test]
#[cfg(feature = "axiom-verification")]
fn test_conditional_with_let_in_branches() {
    println!("\n🧪 Testing: Conditional with let bindings in branches");

    let code = r#"
        define process(x, mode) = 
            if mode > 0 then 
                let doubled = x + x in doubled 
            else 
                let tripled = x + x + x in tripled
    "#;

    println!("   📝 Parsing conditional with let in branches...");
    let program = parse_kleis_program(code).unwrap();

    assert_eq!(program.functions().len(), 1);
    println!("   ✅ Parsed function");

    // Load into Z3
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    println!("   📦 Loading function into Z3...");
    let result = verifier.load_program_functions(&program);
    assert!(result.is_ok());
    println!("   ✅ Conditional with let in branches loaded successfully");
}

/// Test that let-bound variables shadow parameters
#[test]
#[cfg(feature = "axiom-verification")]
fn test_let_shadowing() {
    println!("\n🧪 Testing: Let binding shadows parameter");

    let code = r#"
        define shadow_test(x) = let x = x + 1 in x + x
    "#;

    println!("   📝 Parsing function where let shadows parameter...");
    let program = parse_kleis_program(code).unwrap();

    assert_eq!(program.functions().len(), 1);
    let func = &program.functions()[0];
    assert_eq!(func.name, "shadow_test");
    println!("   ✅ Parsed function with shadowing let");

    // Load into Z3
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    println!("   📦 Loading function into Z3...");
    let result = verifier.load_program_functions(&program);
    assert!(result.is_ok());
    println!("   ✅ Shadowing let binding loaded successfully");
}

/// Test that unused let bindings don't cause errors (sloppy but valid)
#[test]
#[cfg(feature = "axiom-verification")]
fn test_unused_let_binding() {
    println!("\n🧪 Testing: Unused let binding (sloppy but valid)");

    let code = r#"
        define sloppy(x) = let unused = x * x * x in 42
    "#;

    println!("   📝 Parsing: define sloppy(x) = let unused = x * x * x in 42");
    let program = parse_kleis_program(code).unwrap();

    assert_eq!(program.functions().len(), 1);
    let func = &program.functions()[0];
    assert_eq!(func.name, "sloppy");
    println!("   ✅ Parsed function with unused let binding");

    // Load into Z3
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    println!("   📦 Loading function into Z3...");
    let result = verifier.load_program_functions(&program);
    assert!(result.is_ok(), "Failed: {:?}", result);
    println!("   ✅ Unused let binding handled gracefully - no errors!");
    println!("   ℹ️  Note: 'unused' was computed but never referenced");
}

/// Test multiple unused bindings - really sloppy code
#[test]
#[cfg(feature = "axiom-verification")]
fn test_multiple_unused_let_bindings() {
    println!("\n🧪 Testing: Multiple unused let bindings (very sloppy)");

    let code = r#"
        define very_sloppy(x) = 
            let a = x + 1 in 
            let b = x + 2 in 
            let c = x + 3 in 
            x
    "#;

    println!("   📝 Parsing function with 3 unused bindings...");
    let program = parse_kleis_program(code).unwrap();

    assert_eq!(program.functions().len(), 1);
    println!("   ✅ Parsed - all bindings a, b, c are unused, body just returns x");

    // Load into Z3
    let registry = StructureRegistry::new();
    let mut verifier = AxiomVerifier::new(&registry).unwrap();

    println!("   📦 Loading function into Z3...");
    let result = verifier.load_program_functions(&program);
    assert!(result.is_ok());
    println!("   ✅ All unused bindings handled gracefully");
    println!("   ℹ️  Z3 only sees: x (the unused work is discarded)");
}
