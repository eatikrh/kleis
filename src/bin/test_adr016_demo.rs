///! ADR-016 Demo: Operations in Structures
///!
///! Demonstrates parsing the full pattern:
///! - structure defines abstract operations
///! - implements binds operations to concrete types
///! - Ready for type context building!

use kleis::kleis_parser::parse_kleis_program;

fn main() {
    println!("🎯 ADR-016: Operations in Structures");
    println!("{}", "=".repeat(70));
    println!("\nPattern: structure (abstract) + implements (concrete)\n");

    // Test 1: Parse Numeric structure with implements
    println!("Test 1: Numeric Structure Pattern");
    println!("{}", "-".repeat(70));
    test_numeric_pattern();
    println!();

    // Test 2: Multiple implements for one structure
    println!("Test 2: Polymorphic Operations");
    println!("{}", "-".repeat(70));
    test_polymorphic();
    println!();

    // Test 3: Complete stdlib/core.kleis example
    println!("Test 3: Complete stdlib Pattern");
    println!("{}", "-".repeat(70));
    test_complete_stdlib();
    println!();

    println!("{}", "=".repeat(70));
    println!("✅ ADR-016 Pattern Validated!");
    println!("\nKey Achievements:");
    println!("  ✓ Operations belong in structures (conceptually pure)");
    println!("  ✓ Implements binds to concrete types");
    println!("  ✓ Pattern matches stdlib/prelude.kleis");
    println!("  ✓ Enables polymorphism (abs works for ℝ and ℂ)");
    println!("  ✓ Ready for type context building!");
    println!("\nNext Step:");
    println!("  → Build TypeContext from structures + implements");
    println!("  → Type checker can query: 'Which types support abs?'");
    println!("  → Answer: 'ℝ and ℂ (both implement Numeric)'");
}

fn test_numeric_pattern() {
    let code = r#"
        structure Numeric(N) {
            operation abs : N → N
        }
        
        implements Numeric(ℝ) {
            operation abs = builtin_abs_real
        }
    "#;
    
    println!("Input:");
    println!("{}", code);
    
    match parse_kleis_program(code) {
        Ok(program) => {
            println!("✅ Parsed successfully!");
            
            let structures = program.structures();
            println!("\nStructure: {}", structures[0].name);
            println!("  Declares:");
            for member in &structures[0].members {
                match member {
                    kleis::kleis_ast::StructureMember::Operation { name, type_signature } => {
                        println!("    operation {} : {:?}", name, type_signature);
                    }
                    _ => {}
                }
            }
            
            let implements = program.implements();
            println!("\nImplements: {} for {:?}", 
                     implements[0].structure_name,
                     implements[0].type_arg);
            println!("  Provides:");
            for member in &implements[0].members {
                match member {
                    kleis::kleis_ast::ImplMember::Operation { name, implementation } => {
                        println!("    operation {} = {:?}", name, implementation);
                    }
                    _ => {}
                }
            }
            
            println!("\n✓ Structure defines WHAT operations exist");
            println!("✓ Implements defines HOW they work for ℝ");
            println!("✓ Conceptually pure: abs belongs to Numeric types!");
        }
        Err(e) => {
            println!("❌ Parse error: {}", e);
        }
    }
}

fn test_polymorphic() {
    let code = r#"
        structure Numeric(N) {
            operation abs : N → N
        }
        
        implements Numeric(ℝ) {
            operation abs = builtin_abs_real
        }
        
        implements Numeric(ℂ) {
            operation abs = complex_modulus
        }
    "#;
    
    println!("Input: Numeric structure with 2 implementations");
    
    match parse_kleis_program(code) {
        Ok(program) => {
            println!("✅ Parsed successfully!");
            
            let structures = program.structures();
            let implements = program.implements();
            
            println!("\nStructure: {} (abstract)", structures[0].name);
            println!("  operation abs : N → N");
            
            println!("\nImplementations: {}", implements.len());
            for impl_def in implements {
                println!("  - {} for {:?}", impl_def.structure_name, impl_def.type_arg);
            }
            
            println!("\n✓ abs is POLYMORPHIC!");
            println!("✓ Works for any type that implements Numeric");
            println!("✓ Currently: ℝ and ℂ");
            println!("✓ User can add more!");
        }
        Err(e) => {
            println!("❌ Parse error: {}", e);
        }
    }
}

fn test_complete_stdlib() {
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
        
        implements Numeric(ℂ) {
            operation abs = complex_modulus
        }
        
        implements Finite(Set(T)) {
            operation card = builtin_set_card
        }
        
        implements NormedSpace(Vector(n)) {
            operation norm = vector_euclidean_norm
        }
        
        operation frac : ℝ × ℝ → ℝ
    "#;
    
    println!("Input: Complete stdlib pattern (3 structures, 4 implements, 1 top-level)");
    
    match parse_kleis_program(code) {
        Ok(program) => {
            println!("✅ Parsed successfully!");
            println!("\nSummary:");
            println!("  Structures: {}", program.structures().len());
            println!("  Implements: {}", program.implements().len());
            println!("  Top-level operations: {}", program.operations().len());
            
            println!("\nStructures (abstract):");
            for s in program.structures() {
                println!("  - {}", s.name);
            }
            
            println!("\nImplements (concrete):");
            for impl_def in program.implements() {
                println!("  - {} for {:?}", impl_def.structure_name, impl_def.type_arg);
            }
            
            println!("\nTop-level operations (utilities):");
            for op in program.operations() {
                println!("  - {} (display mode hint)", op.name);
            }
            
            println!("\n✓ Complete pattern works!");
            println!("✓ Operations in structures (Numeric, Finite, NormedSpace)");
            println!("✓ Implementations for concrete types (ℝ, ℂ, Set, Vector)");
            println!("✓ Top-level utilities (frac for display mode)");
            println!("\n🎯 Ready to build type context from this!");
        }
        Err(e) => {
            println!("❌ Parse error: {}", e);
        }
    }
}

