#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
// Simple test: Just the einstein operation by itself
//
// einstein(?, ?, ?) should return Tensor(0, 2, 4, ℝ)
// regardless of placeholder inputs!

use kleis::ast::Expression;
use kleis::type_checker::TypeChecker;

fn main() {
    println!("=== Testing einstein operation alone ===\n");

    // Just: einstein(placeholder, placeholder, placeholder)
    let ast = Expression::Operation {
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
                hint: "g_mu_nu".to_string(),
            },
        ],
        span: None,
    };

    println!("Testing: einstein(?, ?, ?)");
    println!("Expected: Tensor(0, 2, 4, ℝ)\n");

    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    let result = checker.check(&ast);

    match result {
        kleis::type_checker::TypeCheckResult::Success(ty) => {
            println!("✅ Type checking SUCCESS!");
            println!("Got type: {:?}\n", ty);

            match &ty {
                kleis::type_inference::Type::Data {
                    constructor, args, ..
                } if constructor == "Tensor" => {
                    println!("🎉 CORRECT! Inferred as Tensor!");
                    if let (
                        kleis::type_inference::Type::NatValue(upper),
                        kleis::type_inference::Type::NatValue(lower),
                        kleis::type_inference::Type::NatValue(dim),
                    ) = (&args[0], &args[1], &args[2])
                    {
                        println!("    Rank: ({}, {}) contravariant, covariant", upper, lower);
                        println!("    Dimension: {}", dim);
                    }
                }
                kleis::type_inference::Type::Var(v) => {
                    println!("❌ WRONG! Got type variable: {:?}", v);
                    println!("    This is the bug - should return concrete Tensor type!");
                    println!("    The operation signature explicitly says:");
                    println!(
                        "    einstein : Tensor(0,2,4,ℝ) → ℝ → Tensor(0,2,4,ℝ) → Tensor(0,2,4,ℝ)"
                    );
                    println!(
                        "                                                        ^^^^^^^^^^^^^^^^^^^^"
                    );
                    println!(
                        "                                                        This is concrete!"
                    );
                }
                _ => {
                    println!("Got unexpected type: {:?}", ty);
                }
            }
        }
        kleis::type_checker::TypeCheckResult::Error { message, .. } => {
            println!("❌ Type checking FAILED:");
            println!("{}", message);
        }
        kleis::type_checker::TypeCheckResult::Polymorphic { type_var, .. } => {
            println!("⚠️  Polymorphic: {:?}", type_var);
        }
    }
}
