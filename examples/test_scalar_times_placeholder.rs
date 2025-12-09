// Test: What type does scalar_multiply return with a placeholder?

use kleis::ast::Expression;
use kleis::type_checker::TypeChecker;

fn main() {
    println!("=== Testing scalar_multiply(Scalar, Placeholder) ===\n");

    let ast = Expression::Operation {
        name: "scalar_multiply".to_string(),
        args: vec![
            Expression::Object("Lambda".to_string()),
            Expression::Placeholder {
                id: 0,
                hint: "unknown".to_string(),
            },
        ],
    };

    println!("Testing: scalar_multiply(Λ, ?)");
    println!("  Λ: Object (unknown type)");
    println!("  ?: Placeholder (unknown type)\n");

    let mut checker = TypeChecker::with_stdlib().expect("Failed to load stdlib");

    let result = checker.check(&ast);

    match result {
        kleis::type_checker::TypeCheckResult::Success(ty) => {
            println!("Result type: {:?}\n", ty);

            println!("FINDING:");
            println!("When both args are unknown (Object + Placeholder),");
            println!("scalar_multiply returns a fresh type variable.");
            println!();
            println!("Then when we do: plus(Tensor, Var),");
            println!("The Var could be anything, so result becomes Var!");
        }
        _ => {}
    }
}
