// Test Einstein's Field Equations - CONTRACTED FORM
//
// This tests the TRACE (contracted) form of Einstein's equations:
// G^μ_μ + Λg^μ_μ = κT^μ_μ
//
// This is a SCALAR equation (taking the trace of both sides).
// See test_einstein_tensor.rs for the full TENSOR form.
//
// Result type: Scalar (ℝ)

use kleis::ast::Expression;
use kleis::type_checker::TypeChecker;
use std::fs;

fn main() {
    println!("=== Testing Einstein's Field Equations (CONTRACTED FORM) ===\n");
    println!("Equation: G^μ_μ + Λg^μ_μ = κT^μ_μ");
    println!("This is the TRACE (scalar) form.\n");

    // Load the semantic AST
    let ast_json = fs::read_to_string("examples/einstein_equations_contracted.json")
        .expect("Failed to read semantic AST file");
    
    let ast: Expression = serde_json::from_str(&ast_json)
        .expect("Failed to parse AST JSON");

    println!("AST Structure:");
    println!("{:#?}\n", ast);

    // Create type checker with full stdlib (includes tensors!)
    let mut checker = TypeChecker::with_stdlib()
        .expect("Failed to load stdlib");

    println!("Type checking contracted Einstein's equations...\n");

    // Type check the expression
    let result = checker.check(&ast);
    
    match result {
        kleis::type_checker::TypeCheckResult::Success(ty) => {
            println!("✅ Type checking SUCCESS!\n");
            println!("Inferred Type: {:?}\n", ty);

            // Expected type analysis:
            println!("=== Expected Types ===");
            println!("Left side: contract(einstein(...)) + scalar_multiply(Λ, contract(metric))");
            println!("  - einstein returns Tensor(0, 2, 4, ℝ)");
            println!("  - contract returns ℝ (scalar)");
            println!("  - Should be: ℝ + ℝ = ℝ");
            println!();
            println!("Right side: scalar_multiply(κ, contract(stress_energy))");
            println!("  - stress_energy is Tensor(0, 2, 4, ℝ)");
            println!("  - contract returns ℝ");
            println!("  - Should be: ℝ * ℝ = ℝ");
            println!();
            println!("Equation: ℝ = ℝ ✓");
        }
        kleis::type_checker::TypeCheckResult::Error { message, suggestion } => {
            println!("❌ Type checking FAILED:");
            println!("{}\n", message);
            
            if let Some(s) = suggestion {
                println!("💡 Suggestion: {}\n", s);
            }
            
            println!("This might mean:");
            println!("- Missing operation definitions in stdlib");
            println!("- contract operation needs proper signature");
            println!("- einstein operation needs proper signature");
        }
        kleis::type_checker::TypeCheckResult::Polymorphic { type_var, available_types } => {
            println!("⚠️  Type is polymorphic (needs more context):");
            println!("Type variable: {:?}", type_var);
            println!("Available types: {:?}\n", available_types);
        }
    }

    println!("\n=== Comparison ===");
    println!("LaTeX-parsed equation (from gallery):");
    println!("  Type: Var(α) - generic notation");
    println!("  No semantic knowledge");
    println!();
    println!("Semantically constructed (with tensor ops):");
    println!("  Type: Should be ℝ = ℝ (scalar equation)");
    println!("  Full tensor type knowledge");
    println!("  Validates dimensional consistency");
    println!();
    println!("The PALETTE provides the semantics!");
    println!("LaTeX is just for display/import.");
}

