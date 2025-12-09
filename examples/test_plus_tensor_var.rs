// Test: plus(Tensor, Var) - does it preserve Tensor type?

use kleis::ast::Expression;
use kleis::type_checker::TypeChecker;

fn main() {
    println!("=== Testing plus(Tensor, Var) ===\n");

    // plus(einstein(...), placeholder)
    let ast = Expression::Operation {
        name: "plus".to_string(),
        args: vec![
            Expression::Operation {
                name: "einstein".to_string(),
                args: vec![
                    Expression::Placeholder {
                        id: 0,
                        hint: "R".to_string(),
                    },
                    Expression::Placeholder {
                        id: 1,
                        hint: "R_scalar".to_string(),
                    },
                    Expression::Placeholder {
                        id: 2,
                        hint: "g".to_string(),
                    },
                ],
            },
            Expression::Placeholder {
                id: 3,
                hint: "something".to_string(),
            },
        ],
    };

    println!("Testing: plus(einstein(...), ?)");
    println!("  Left: Tensor(0, 2, 4, ℝ) (from einstein)");
    println!("  Right: Placeholder → Var(α)");
    println!();
    println!("Constraint: Both args must be same type");
    println!("  → Var(α) unifies with Tensor(0, 2, 4, ℝ)");
    println!("  → α := Tensor(0, 2, 4, ℝ)");
    println!();
    println!("Expected: plus should return Tensor(0, 2, 4, ℝ)\n");

    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    let result = checker.check(&ast);

    match result {
        kleis::type_checker::TypeCheckResult::Success(ty) => {
            println!("Result type: {:?}\n", ty);

            match &ty {
                kleis::type_inference::Type::Data { constructor, .. }
                    if constructor == "Tensor" =>
                {
                    println!("✅ CORRECT! Substitution applied, got Tensor");
                }
                kleis::type_inference::Type::Var(v) => {
                    println!("❌ BUG FOUND! Got Var({:?}) instead of Tensor", v);
                    println!();
                    println!("ROOT CAUSE:");
                    println!("  Substitution α := Tensor(0,2,4,ℝ) happens");
                    println!("  But the result type isn't applying the substitution!");
                    println!();
                    println!("LOCATION:");
                    println!("  Likely in infer_and_solve() or signature interpreter");
                    println!("  The substitution exists but isn't applied to return value");
                }
                _ => {}
            }
        }
        _ => {}
    }
}
