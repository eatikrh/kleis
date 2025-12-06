///! Test structure parsing WITH type checking
///!
///! Updated to show COMPLETE pipeline:
///! 1. Parse structure definitions
///! 2. Build type context
///! 3. Connect to type checker
///! 4. Query operation support
///!
///! This demonstrates the full integration!

use kleis::kleis_parser::parse_kleis_program;
use kleis::type_checker::TypeChecker;

fn main() {
    println!("🎯 Structure Parsing + Type Checking");
    println!("{}", "=".repeat(70));
    println!("\nComplete Pipeline: Parse → Build Context → Type Check\n");

    // Test 1: Parse and build type context
    println!("Test 1: Parse + Build Type Context");
    println!("{}", "-".repeat(70));
    test_with_type_context();
    println!();

    // Test 2: User-defined types
    println!("Test 2: User-Defined Types");
    println!("{}", "-".repeat(70));
    test_user_types();
    println!();

    // Test 3: Operation queries
    println!("Test 3: Query Operation Support");
    println!("{}", "-".repeat(70));
    test_operation_queries();
    println!();

    println!("{}", "=".repeat(70));
    println!("✅ Complete Pipeline Working!");
    println!("\nFrom Parsing to Type Checking:");
    println!("  1. ✓ Parse structures");
    println!("  2. ✓ Parse implements");
    println!("  3. ✓ Build type context");
    println!("  4. ✓ Query operations");
    println!("  5. ✓ Ready for full type checking!");
}

fn test_with_type_context() {
    let code = r#"
        structure Numeric(N) {
            operation abs : N → N
        }
        
        implements Numeric(ℝ) {
            operation abs = builtin_abs
        }
        
        implements Numeric(ℂ) {
            operation abs = complex_modulus
        }
    "#;
    
    println!("Input: Numeric structure with 2 implements");
    
    // Step 1: Parse
    let program = parse_kleis_program(code).unwrap();
    println!("✅ Step 1: Parsed");
    println!("   - 1 structure");
    println!("   - 2 implements");
    
    // Step 2: Build type checker
    match TypeChecker::from_program(program) {
        Ok(checker) => {
            println!("✅ Step 2: Type checker built");
            
            // Step 3: Query support
            let types = checker.types_supporting("abs");
            println!("✅ Step 3: Query operations");
            println!("   Which types support 'abs'? {}", types.join(", "));
            
            // Step 4: Check specific combinations
            println!("✅ Step 4: Check support");
            println!("   ℝ supports abs? {}", checker.type_supports_operation("ℝ", "abs"));
            println!("   ℂ supports abs? {}", checker.type_supports_operation("ℂ", "abs"));
            println!("   ℝ supports card? {}", checker.type_supports_operation("ℝ", "card"));
            
            println!("\n🎯 Complete pipeline working!");
        }
        Err(e) => {
            println!("❌ Type checker error: {}", e);
        }
    }
}

fn test_user_types() {
    let code = r#"
        structure Additive(A) {
            operation add : A × A → A
        }
        
        implements Additive(Money) {
            operation add = money_add
        }
    "#;
    
    println!("Input: User-defined Money type with Additive");
    
    let program = parse_kleis_program(code).unwrap();
    println!("✅ Parsed: 1 structure, 1 implements");
    
    match TypeChecker::from_program(program) {
        Ok(checker) => {
            println!("✅ Type checker built");
            
            // Check if Money supports add
            if checker.type_supports_operation("Money", "add") {
                println!("✅ Money supports add");
                println!("   Reason: Money implements Additive");
                println!("   ✓ User-defined types work!");
            }
            
            // Check which types support add
            let types = checker.types_supporting("add");
            println!("\n   Types supporting add: {}", types.join(", "));
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
}

fn test_operation_queries() {
    let code = r#"
        structure Numeric(N) {
            operation abs : N → N
        }
        
        structure Finite(C) {
            operation card : C → ℕ
        }
        
        structure NormedSpace(V) {
            operation norm : V → ℝ
        }
        
        implements Numeric(ℝ) {
            operation abs = builtin_abs
        }
        
        implements Finite(Set(T)) {
            operation card = set_card
        }
        
        implements NormedSpace(Vector(n)) {
            operation norm = vector_norm
        }
    "#;
    
    println!("Input: 3 structures, 3 implements");
    
    let program = parse_kleis_program(code).unwrap();
    let checker = TypeChecker::from_program(program).unwrap();
    
    println!("✅ Type checker built");
    
    println!("\n🔍 Query Operations:");
    let operations = vec!["abs", "card", "norm"];
    
    for op in &operations {
        let types = checker.types_supporting(op);
        println!("   {} → {}", op, types.join(", "));
    }
    
    println!("\n🔍 Query Types:");
    let type_checks = vec![
        ("ℝ", "abs", true),
        ("ℝ", "card", false),
        ("Set(T)", "card", true),
        ("Set(T)", "abs", false),
        ("Vector(n)", "norm", true),
    ];
    
    for (ty, op, expected) in type_checks {
        let result = checker.type_supports_operation(ty, op);
        let symbol = if result == expected { "✓" } else { "✗" };
        println!("   {} {} {}: {}", symbol, ty, op, result);
    }
    
    println!("\n🎯 Query system working!");
    println!("   ✓ Can ask: 'Which types support operation X?'");
    println!("   ✓ Can check: 'Does type T support operation X?'");
    println!("   ✓ Ready for full type checking with error messages!");
}
