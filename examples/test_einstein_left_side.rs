#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
// Test just the left side of Einstein equation
// plus(einstein(...), scalar_multiply(Lambda, placeholder))

use kleis::ast::Expression;
use kleis::type_checker::TypeChecker;

fn main() {
    println!("=== Testing LEFT SIDE: G_μν + Λg_μν ===\n");

    let ast = Expression::Operation {
        name: "plus".to_string(),
        args: vec![
            Expression::Operation {
                name: "einstein".to_string(),
                args: vec![
                    Expression::Placeholder {
                        id: 0,
                        hint: "R_mu_nu".to_string(),
                    },
                    Expression::Placeholder {
                        id: 1,
                        hint: "R_scalar".to_string(),
                    },
                    Expression::Placeholder {
                        id: 2,
                        hint: "metric".to_string(),
                    },
                ],
                span: None,
            },
            Expression::Operation {
                name: "scalar_multiply".to_string(),
                args: vec![
                    Expression::Object("Lambda".to_string()),
                    Expression::Placeholder {
                        id: 3,
                        hint: "metric_g_mu_nu".to_string(),
                    },
                ],
                span: None,
            },
        ],
        span: None,
    };

    println!("Testing: plus(einstein(...), scalar_multiply(Λ, ?))");
    println!("Expected: Tensor(0, 2, 4, ℝ)\n");

    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    let result = checker.check(&ast);

    match result {
        kleis::type_checker::TypeCheckResult::Success(ty) => {
            println!("Result type: {:?}\n", ty);

            match &ty {
                kleis::type_inference::Type::Data { constructor, .. }
                    if constructor == "Tensor" =>
                {
                    println!("✅ Left side returns Tensor!");
                }
                kleis::type_inference::Type::Var(_) => {
                    println!("❌ Left side returns Var!");
                }
                _ => {}
            }
        }
        _ => {}
    }
}
