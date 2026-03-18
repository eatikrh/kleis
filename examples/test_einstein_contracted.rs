#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
// Test Einstein's Field Equations - CONTRACTED FORM
//
// This tests the TRACE (contracted) form of Einstein's equations:
// G^μ_μ + Λg^μ_μ = κT^μ_μ
//
// This is a SCALAR equation (taking the trace of both sides).
// See test_einstein_tensor.rs for the full TENSOR form.
//
// EXPECTED RESULT: Scalar (ℝ)
//
// WHY THIS WORKS BETTER:
// The contract() operation explicitly takes traces, returning Scalar.
// Even though Λ and κ are undefined, the scalar operations (plus, scalar_multiply)
// have concrete implementations for ℝ, so the equation type-checks.
//
// This demonstrates:
// - Contracted (trace) form reduces to scalar algebra
// - Scalar arithmetic is well-defined in stdlib
// - Works even without declaring constants (but shouldn't!)
//
// Note: Even though this type-checks, we SHOULD still declare constants
// with proper units for dimensional consistency!

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

    let ast: Expression = serde_json::from_str(&ast_json).expect("Failed to parse AST JSON");

    println!("AST Structure:");
    println!("{:#?}\n", ast);

    // Create type checker with full stdlib (includes tensors!)
    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    println!("Type checking contracted Einstein's equations...\n");

    // Type check the expression
    let result = checker.check(&ast);

    match result {
        kleis::type_checker::TypeCheckResult::Success(ty) => {
            println!("✅ Type checking SUCCESS!\n");
            println!("Inferred Type: {:?}\n", ty);

            // Validate we got Scalar
            match &ty {
                kleis::type_inference::Type::Data { constructor, .. }
                    if constructor == "Scalar" =>
                {
                    println!("✅ CORRECT! Inferred as Scalar");
                    println!("    Contracted form reduces tensors to scalars via trace.");
                }
                _ => {
                    println!("⚠️  Got unexpected type (expected Scalar): {:?}", ty);
                }
            }
            println!();

            // Type analysis:
            println!("=== Type Flow ===");
            println!("Left side: contract(einstein(...)) + scalar_multiply(Λ, contract(metric))");
            println!("  1. einstein(...) → Tensor(0, 2, 4, ℝ)");
            println!("  2. contract(Tensor) → ℝ (takes trace: G^μ_μ)");
            println!("  3. contract(metric) → ℝ (takes trace: g^μ_μ)");
            println!("  4. scalar_multiply(Λ, ℝ) → ℝ");
            println!("  5. plus(ℝ, ℝ) → ℝ");
            println!();
            println!("Right side: scalar_multiply(κ, contract(stress_energy))");
            println!("  1. contract(T_μν) → ℝ (takes trace: T^μ_μ)");
            println!("  2. scalar_multiply(κ, ℝ) → ℝ");
            println!();
            println!("Final: Scalar = Scalar ✓");
            println!();
            println!("NOTE: This works even without declaring Λ and κ because");
            println!("      scalar arithmetic (ℝ operations) is well-defined.");
            println!("      But we SHOULD declare constants with units for physics!");
        }
        kleis::type_checker::TypeCheckResult::Error {
            message,
            suggestion,
        } => {
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
        kleis::type_checker::TypeCheckResult::Polymorphic {
            type_var,
            available_types,
        } => {
            println!("⚠️  Type is polymorphic (needs more context):");
            println!("Type variable: {:?}", type_var);
            println!("Available types: {:?}\n", available_types);
        }
    }

    println!("\n=== Key Insights ===");
    println!();
    println!("1. CONTRACTED FORM (this test):");
    println!("   - Takes trace: G^μ_μ, g^μ_μ, T^μ_μ");
    println!("   - Result: Scalar equation");
    println!("   - Type-checks: Scalar = Scalar ✓");
    println!("   - Physical meaning: Energy conservation");
    println!();
    println!("2. TENSOR FORM (see test_einstein_tensor.rs):");
    println!("   - Full equation: G_μν + Λg_μν = κT_μν");
    println!("   - Result: Tensor equation (10 independent equations)");
    println!("   - Type-checks: Var(α) - requires constant declarations");
    println!("   - Physical meaning: Fundamental field equations");
    println!();
    println!("3. PALETTE vs LATEX:");
    println!("   - Palette: Semantic operations (einstein, contract, etc.)");
    println!("   - LaTeX: Visual notation only (from gallery)");
    println!("   - Type checking needs SEMANTICS from palette!");
    println!();
    println!("4. CONSTANTS NEED UNITS:");
    println!("   - Λ is not just 1.089e-52");
    println!("   - Λ is 1.089e-52 m⁻² (with units!)");
    println!("   - Type system should enforce dimensional consistency");
    println!();
    println!("See UNIVERSAL_CONSTANTS_FINDING.md for full analysis.");
}
