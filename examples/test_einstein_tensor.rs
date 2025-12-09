// Test Einstein's Field Equations - FULL TENSOR FORM
//
// This tests the complete tensor form of Einstein's equations:
// G_μν + Λg_μν = (8πG/c⁴) T_μν
//
// This is a RANK-2 TENSOR equation (both sides are tensors).
// See test_einstein_contracted.rs for the scalar (trace) form.
//
// Expected result type: Tensor(0, 2, 4, ℝ) = Tensor(0, 2, 4, ℝ)
//
// This demonstrates:
// - Palette provides semantic operations (einstein, etc.)
// - Type system validates tensor ranks
// - GR equations type-check correctly!

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

            // Expected type analysis:
            println!("=== Expected Types ===");
            println!("Left side: einstein(R_μν, R, g_μν) + scalar_multiply(Λ, g_μν)");
            println!("  - einstein returns: Tensor(0, 2, 4, ℝ) → G_μν");
            println!("  - g_μν is: Tensor(0, 2, 4, ℝ)");
            println!("  - scalar_multiply(Λ, g_μν): ℝ × Tensor(0, 2, 4, ℝ) → Tensor(0, 2, 4, ℝ)");
            println!("  - plus: Tensor + Tensor → Tensor(0, 2, 4, ℝ)");
            println!();
            println!("Right side: scalar_multiply(κ, T_μν)");
            println!("  - T_μν is: Tensor(0, 2, 4, ℝ)");
            println!("  - scalar_multiply(κ, T_μν): ℝ × Tensor → Tensor(0, 2, 4, ℝ)");
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
            
            println!("This might mean:");
            println!("- Operations need tensor-aware arithmetic");
            println!("- scalar_multiply needs to handle Scalar × Tensor");
            println!("- plus needs to handle Tensor + Tensor");
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

