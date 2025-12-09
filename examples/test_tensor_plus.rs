// Test: Can we add two tensors?
//
// plus(Tensor, Tensor) should work because plus : T → T → T is polymorphic
//
// But currently only Arithmetic(ℝ) is implemented, not Arithmetic(Tensor(...))

use kleis::ast::Expression;
use kleis::type_checker::TypeChecker;

fn main() {
    println!("=== Testing Tensor Addition ===\n");

    // Test: plus(einstein(...), einstein(...))
    let ast = Expression::Operation {
        name: "plus".to_string(),
        args: vec![
            Expression::Operation {
                name: "einstein".to_string(),
                args: vec![
                    Expression::Placeholder { id: 0, hint: "R1".to_string() },
                    Expression::Placeholder { id: 1, hint: "R_scalar1".to_string() },
                    Expression::Placeholder { id: 2, hint: "g1".to_string() },
                ],
            },
            Expression::Operation {
                name: "einstein".to_string(),
                args: vec![
                    Expression::Placeholder { id: 3, hint: "R2".to_string() },
                    Expression::Placeholder { id: 4, hint: "R_scalar2".to_string() },
                    Expression::Placeholder { id: 5, hint: "g2".to_string() },
                ],
            },
        ],
    };

    println!("Testing: plus(einstein(...), einstein(...))");
    println!("Each einstein returns: Tensor(0, 2, 4, ℝ)");
    println!("Expected result: Tensor(0, 2, 4, ℝ)\n");

    let mut checker = TypeChecker::with_stdlib()
        .expect("Failed to load stdlib");

    let result = checker.check(&ast);
    
    match result {
        kleis::type_checker::TypeCheckResult::Success(ty) => {
            println!("✅ Type checking SUCCESS!");
            println!("Got type: {:?}\n", ty);
            
            match &ty {
                kleis::type_inference::Type::Data { constructor, args, .. } 
                    if constructor == "Tensor" => {
                    println!("🎉 CORRECT! plus preserves Tensor type!");
                }
                kleis::type_inference::Type::Var(_) => {
                    println!("❌ BUG FOUND! Got type variable instead of Tensor");
                    println!("    This is the problem!");
                    println!();
                    println!("ROOT CAUSE:");
                    println!("  plus : T → T → T is polymorphic (correct!)");
                    println!("  But only Arithmetic(ℝ) is implemented");
                    println!("  Need: Arithmetic(Tensor(upper, lower, dim, ℝ))");
                    println!();
                    println!("PARSER LIMITATION:");
                    println!("  Can't parse: implements Arithmetic(Tensor(...))");
                    println!("  Parser doesn't support complex parametric types in implements");
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
        kleis::type_checker::TypeCheckResult::Polymorphic { .. } => {
            println!("⚠️  Polymorphic result");
        }
    }
}

