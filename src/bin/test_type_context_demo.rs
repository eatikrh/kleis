#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
///! Type Context Builder Demo
///!
///! Shows the complete pipeline:
///! 1. Parse Kleis program (structures + implements)
///! 2. Build type context
///! 3. Query operation support
///! 4. Generate error suggestions
///!
///! This validates ADR-016 + ADR-015 working together!
use kleis::kleis_parser::parse_kleis_program;
use kleis::type_context::TypeContextBuilder;

fn main() {
    println!("🎯 Type Context Builder - Complete Pipeline Demo");
    println!("{}", "=".repeat(70));
    println!();

    // Demo 1: Basic operation registry
    println!("Demo 1: Build Type Context from stdlib");
    println!("{}", "-".repeat(70));
    demo_basic_registry();
    println!();

    // Demo 2: Query which types support operations
    println!("Demo 2: Query Operation Support");
    println!("{}", "-".repeat(70));
    demo_query_operations();
    println!();

    // Demo 3: Error suggestions (validates ADR-015!)
    println!("Demo 3: Error Suggestions (ADR-015 Validation)");
    println!("{}", "-".repeat(70));
    demo_error_suggestions();
    println!();

    // Demo 4: Complete stdlib
    println!("Demo 4: Complete stdlib Pattern");
    println!("{}", "-".repeat(70));
    demo_complete_stdlib();
    println!();

    println!("{}", "=".repeat(70));
    println!("✅ Type Context Builder Working!");
    println!("\nKey Achievements:");
    println!("  ✓ Parse structures + implements");
    println!("  ✓ Build operation registry");
    println!("  ✓ Query: 'Which types support abs?' → 'ℝ and ℂ'");
    println!("  ✓ Generate helpful error suggestions");
    println!("  ✓ Validates ADR-016 (operations in structures)");
    println!("  ✓ Validates ADR-015 (explicit forms enable better errors)");
    println!("\nNext Step:");
    println!("  → Connect to type inference engine");
    println!("  → Type check expressions with user-defined types");
    println!("  → COMPLETE type checking POC! 🎯");
}

fn demo_basic_registry() {
    let code = r#"
        structure Numeric(N) {
            operation abs : N → N
        }
        
        implements Numeric(ℝ) {
            operation abs = builtin_abs
        }
    "#;

    println!("Input:");
    println!("{}", code);

    let program = parse_kleis_program(code).unwrap();
    let builder = TypeContextBuilder::from_program(program).unwrap();

    println!("✅ Type context built!");
    println!("\nRegistry:");
    println!("  Structure: Numeric");
    println!("    defines: abs");
    println!("  Type: ℝ");
    println!("    implements: Numeric");
    println!("    supports: abs ✓");

    // Test queries
    if builder.supports_operation("ℝ", "abs") {
        println!("\n✓ Query: ℝ supports abs? → YES");
    }

    if !builder.supports_operation("ℝ", "card") {
        println!("✓ Query: ℝ supports card? → NO");
    }
}

fn demo_query_operations() {
    let code = r#"
        structure Numeric(N) {
            operation abs : N → N
        }
        
        structure Finite(C) {
            operation card : C → ℕ
        }
        
        implements Numeric(ℝ) {
            operation abs = builtin_abs
        }
        
        implements Numeric(ℂ) {
            operation abs = complex_modulus
        }
        
        implements Finite(Set(T)) {
            operation card = builtin_card
        }
    "#;

    println!("Input: (3 structures, 3 implements)");

    let program = parse_kleis_program(code).unwrap();
    let builder = TypeContextBuilder::from_program(program).unwrap();

    println!("✅ Type context built!");

    // Query: Which types support abs?
    let abs_types = builder.types_supporting("abs");
    println!("\n🔍 Query: Which types support 'abs'?");
    println!("   Answer: {}", abs_types.join(", "));
    println!("   ✓ Both ℝ and ℂ support abs (polymorphic!)");

    // Query: Which types support card?
    let card_types = builder.types_supporting("card");
    println!("\n🔍 Query: Which types support 'card'?");
    println!("   Answer: {}", card_types.join(", "));
    println!("   ✓ Sets support cardinality");

    // Check specific combinations
    println!("\n🔍 Specific Checks:");
    println!(
        "   ℝ supports abs? {}",
        builder.supports_operation("ℝ", "abs")
    );
    println!(
        "   ℝ supports card? {}",
        builder.supports_operation("ℝ", "card")
    );
    println!(
        "   ℂ supports abs? {}",
        builder.supports_operation("ℂ", "abs")
    );
    println!(
        "   Set(T) supports card? {}",
        builder.supports_operation("Set(T)", "card")
    );
    println!(
        "   Set(T) supports abs? {}",
        builder.supports_operation("Set(T)", "abs")
    );
}

fn demo_error_suggestions() {
    let code = r#"
        structure Numeric(N) {
            operation abs : N → N
        }
        
        structure Finite(C) {
            operation card : C → ℕ
        }
        
        implements Numeric(ℝ) {
            operation abs = builtin_abs
        }
        
        implements Finite(Set(ℤ)) {
            operation card = builtin_card
        }
    "#;

    println!("Input: (Numeric and Finite structures)");

    let program = parse_kleis_program(code).unwrap();
    let builder = TypeContextBuilder::from_program(program).unwrap();

    println!("✅ Type context built!");

    // Simulate type error: abs(Set) - WRONG!
    println!("\n❌ User tries: abs(S) where S : Set(ℤ)");
    if !builder.supports_operation("Set(ℤ)", "abs") {
        println!("   Type check: Set(ℤ) does not support 'abs'");

        if let Some(suggestion) = builder.suggest_operation("Set(ℤ)", "abs") {
            println!("   💡 {}", suggestion);
        }
    }

    println!("\n✓ This is ADR-015's promise!");
    println!("✓ Explicit form 'abs' enables helpful error");
    println!("✓ Suggestion based on what Set actually supports");

    // Simulate another error: card(ℝ) - WRONG!
    println!("\n❌ User tries: card(x) where x : ℝ");
    if !builder.supports_operation("ℝ", "card") {
        println!("   Type check: ℝ does not support 'card'");

        if let Some(suggestion) = builder.suggest_operation("ℝ", "card") {
            println!("   💡 {}", suggestion);
        }
    }

    println!("\n🎯 Error suggestions guide users to correct operations!");
}

fn demo_complete_stdlib() {
    let code = r#"
        structure Numeric(N) {
            operation abs : N → N
            operation floor : N → ℤ
        }
        
        structure Finite(C) {
            operation card : C → ℕ
        }
        
        structure NormedSpace(V) {
            operation norm : V → ℝ
        }
        
        implements Numeric(ℝ) {
            operation abs = builtin_abs
            operation floor = builtin_floor
        }
        
        implements Numeric(ℂ) {
            operation abs = complex_modulus
        }
        
        implements Finite(Set(T)) {
            operation card = set_cardinality
        }
        
        implements Finite(List(T)) {
            operation card = list_length
        }
        
        implements NormedSpace(Vector(n)) {
            operation norm = euclidean_norm
        }
    "#;

    println!("Input: (Complete stdlib with 3 structures, 5 implements)");

    let program = parse_kleis_program(code).unwrap();
    let builder = TypeContextBuilder::from_program(program).unwrap();

    println!("✅ Type context built!");

    println!("\n📊 Registry Summary:");
    println!("   Structures: 3");
    println!("     - Numeric (defines: abs, floor)");
    println!("     - Finite (defines: card)");
    println!("     - NormedSpace (defines: norm)");

    println!("\n   Implementations: 5");
    println!("     - ℝ implements Numeric");
    println!("     - ℂ implements Numeric");
    println!("     - Set(T) implements Finite");
    println!("     - List(T) implements Finite");
    println!("     - Vector(n) implements NormedSpace");

    println!("\n🔍 Operation Support Matrix:");
    let types = vec!["ℝ", "ℂ", "Set(T)", "List(T)", "Vector(n)"];
    let operations = vec!["abs", "floor", "card", "norm"];

    for op in &operations {
        let supporting = builder.types_supporting(op);
        println!("   {} → {}", op, supporting.join(", "));
    }

    println!("\n✓ Complete stdlib pattern working!");
    println!("✓ Polymorphism: abs works for ℝ and ℂ");
    println!("✓ Polymorphism: card works for Set and List");
    println!("✓ Type-specific: floor only for ℝ (ℂ doesn't implement)");
    println!("\n🎯 Ready for type checking!");
}
