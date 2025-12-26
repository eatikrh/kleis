#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
// Test: scalar_multiply(ℝ, Tensor)
//
// Should return Tensor when multiplying scalar by tensor

use kleis::ast::Expression;
use kleis::type_checker::TypeChecker;

fn main() {
    println!("=== Testing Scalar × Tensor ===\n");

    // Test: scalar_multiply(Lambda, einstein(...))
    let ast = Expression::Operation {
        name: "scalar_multiply".to_string(),
        args: vec![
            Expression::Object("Lambda".to_string()),
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
                span: None,
            },
        ],
        span: None,
    };

    println!("Testing: scalar_multiply(Λ, einstein(...))");
    println!("  Λ: Scalar");
    println!("  einstein: Tensor(0, 2, 4, ℝ)");
    println!("Expected: Tensor(0, 2, 4, ℝ)\n");

    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    let result = checker.check(&ast);

    match result {
        kleis::type_checker::TypeCheckResult::Success(ty) => {
            println!("✅ Type checking SUCCESS!");
            println!("Got type: {:?}\n", ty);

            match &ty {
                kleis::type_inference::Type::Data { constructor, .. }
                    if constructor == "Tensor" =>
                {
                    println!("🎉 CORRECT! Scalar × Tensor = Tensor");
                }
                kleis::type_inference::Type::Var(_) => {
                    println!("❌ BUG! Got type variable");
                    println!("    scalar_multiply doesn't know how to handle Tensor!");
                }
                _ => {
                    println!("Unexpected type: {:?}", ty);
                }
            }
        }
        kleis::type_checker::TypeCheckResult::Error { message, .. } => {
            println!("❌ Type checking FAILED:");
            println!("{}", message);
        }
        _ => {}
    }
}
