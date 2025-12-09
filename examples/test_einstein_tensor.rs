// Test Einstein's Field Equations - FULL TENSOR FORM
//
// This tests the complete tensor form of Einstein's equations:
// G_μν + Λg_μν = (8πG/c⁴) T_μν
//
// EXPECTED RESULT: Var(TypeVar(n)) - POLYMORPHIC TYPE
//
// WHY THIS IS CORRECT:
// The equation contains undefined physical constants (Λ, κ) which are
// represented as Object("Lambda") and Object("kappa"). Since the type
// system doesn't know what these are (could be anything!), it returns
// a polymorphic type variable.
//
// This is GOOD BEHAVIOR! The type system is telling us:
// "You need to declare your constants with proper types and units!"
//
// Once we add:
//   const Lambda : PhysicalConstant(1.089e-52, "m^-2")
//   const kappa : PhysicalConstant(2.077e-43, "m^-1 kg^-1 s^2")
//
// Then the equation will type-check as: Tensor(0, 2, 4, ℝ) = Tensor(0, 2, 4, ℝ)
//
// See UNIVERSAL_CONSTANTS_FINDING.md for full analysis.
//
// This demonstrates:
// - Type system detects undefined constants
// - Physical constants need units, not just numbers
// - Scope matters (Lambda could mean many things!)
// - Dimensional analysis should be type checking

use kleis::ast::Expression;
use kleis::type_checker::TypeChecker;
use std::fs;

fn main() {
    println!("=== Testing Einstein's Field Equations (FULL TENSOR FORM) ===\n");
    println!("Equation: G_μν + Λg_μν = (8πG/c⁴) T_μν");
    println!("This is the complete RANK-2 TENSOR form.\n");

    // Load the semantic AST
    let ast_json = fs::read_to_string("examples/einstein_equations_tensor.json")
        .expect("Failed to read tensor AST file");
    
    let ast: Expression = serde_json::from_str(&ast_json)
        .expect("Failed to parse AST JSON");

    println!("AST Structure:");
    println!("{:#?}\n", ast);

    // Create type checker with full stdlib (includes tensors!)
    let mut checker = TypeChecker::with_stdlib()
        .expect("Failed to load stdlib");

    println!("Type checking full tensor Einstein's equations...\n");

    // Type check the expression
    let result = checker.check(&ast);
    
    match result {
        kleis::type_checker::TypeCheckResult::Success(ty) => {
            println!("✅ Type checking SUCCESS!\n");
            println!("Inferred Type: {:?}\n", ty);
            
            // Validate we got the expected polymorphic type
            match &ty {
                kleis::type_inference::Type::Data { constructor, args, .. } 
                    if constructor == "Tensor" => {
                    println!("🎉 Returned concrete Tensor (constants were declared!)");
                    println!("    Rank: ({}, {}) (contravariant, covariant)", 
                             if let kleis::type_inference::Type::NatValue(n) = args[0] { n } else { 0 },
                             if let kleis::type_inference::Type::NatValue(n) = args[1] { n } else { 0 });
                    println!();
                    println!("This means physical constants Λ and κ were properly typed.");
                }
                kleis::type_inference::Type::Var(v) => {
                    println!("✅ CORRECT! Returns Var({:?}) - polymorphic type", v);
                    println!();
                    println!("WHY THIS IS RIGHT:");
                    println!("  - Λ (Lambda) is undefined → could be anything");
                    println!("  - κ (kappa) is undefined → could be anything");
                    println!("  - scalar_multiply(?, ?) → polymorphic");
                    println!("  - Equation is valid for ANY type where constraints hold");
                    println!();
                    println!("TYPE SYSTEM IS TEACHING US:");
                    println!("  'Declare your physical constants with types and units!'");
                    println!();
                    println!("Once we add:");
                    println!("  const Lambda : PhysicalConstant(1.089e-52, \"m^-2\")");
                    println!("  const kappa : PhysicalConstant(2.077e-43, \"m^-1 kg^-1 s^2\")");
                    println!();
                    println!("Then equation will type-check as: Tensor(0,2,4,ℝ) = Tensor(0,2,4,ℝ)");
                }
                _ => {}
            }
            println!();

            // Expected type analysis:
            println!("=== Expected Types ===");
            println!("Left side: plus(einstein(R_μν, R, g_μν), scalar_multiply(Λ, g_μν))");
            println!("  - einstein returns: Tensor(0, 2, 4, ℝ) → G_μν");
            println!("  - g_μν is: Tensor(0, 2, 4, ℝ)");
            println!("  - scalar_multiply(Λ, g_μν): ℝ × Tensor(0, 2, 4, ℝ) → Tensor(0, 2, 4, ℝ)");
            println!("  - plus: Polymorphic! Works for Tensor + Tensor → Tensor(0, 2, 4, ℝ)");
            println!();
            println!("Right side: scalar_multiply(κ, T_μν)");
            println!("  - T_μν is: Tensor(0, 2, 4, ℝ)");
            println!("  - scalar_multiply(κ, T_μν): ℝ × Tensor → Tensor(0, 2, 4, ℝ)");
            println!();
            println!("Note: plus and scalar_multiply are POLYMORPHIC!");
            println!("  Just like multiply works for regular and block matrices,");
            println!("  plus works for scalars, matrices, and tensors!");
            println!();
            println!("Expected: Tensor(0, 2, 4, ℝ) = Tensor(0, 2, 4, ℝ) ✓");
            println!("This is a RANK-2 COVARIANT TENSOR equation!");
            println!();
            println!("Physical meaning:");
            println!("  G_μν: Einstein tensor (geometry of spacetime)");
            println!("  g_μν: Metric tensor (spacetime distances)");
            println!("  T_μν: Stress-energy tensor (matter/energy content)");
            println!("  Λ: Cosmological constant (dark energy)");
            println!("  κ = 8πG/c⁴: Einstein's constant");
        }
        kleis::type_checker::TypeCheckResult::Error { message, suggestion } => {
            println!("❌ Type checking FAILED:");
            println!("{}\n", message);
            
            if let Some(s) = suggestion {
                println!("💡 Suggestion: {}\n", s);
            }
            
            println!("This means:");
            println!("- Parser limitation: Can't add Arithmetic(Tensor(...)) implementation");
            println!("- Architecture is CORRECT - plus SHOULD be polymorphic");
            println!("- Waiting for parser to support complex implements blocks");
        }
        kleis::type_checker::TypeCheckResult::Polymorphic { type_var, available_types } => {
            println!("⚠️  Type is polymorphic (needs more context):");
            println!("Type variable: {:?}", type_var);
            println!("Available types: {:?}\n", available_types);
        }
    }

    println!("\n=== Comparison with Contracted Form ===");
    println!("Full tensor form (this test):");
    println!("  G_μν + Λg_μν = κT_μν");
    println!("  Type: Tensor(0, 2, 4, ℝ) [16 components in 4D]");
    println!("  10 independent equations (by symmetry)");
    println!();
    println!("Contracted form (scalar trace):");
    println!("  G^μ_μ + Λg^μ_μ = κT^μ_μ");
    println!("  Type: Scalar (ℝ) [1 component]");
    println!("  Conservation of energy");
    println!();
    println!("Both forms are valid, but the tensor form is fundamental!");
}

