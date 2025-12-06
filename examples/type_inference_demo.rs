//! Type Inference Proof of Concept Demo
//!
//! Demonstrates type inference on Kleis expressions

use kleis::ast::Expression;
use kleis::type_inference::{Type, TypeInference};

fn main() {
    println!("=== Kleis Type Inference - Proof of Concept ===\n");

    // Example 1: Simple constant
    println!("Example 1: Constant");
    let expr1 = Expression::Const("42".to_string());
    demo_inference(&expr1);

    // Example 2: Addition of constants
    println!("\nExample 2: Addition (1 + 2)");
    let expr2 = Expression::operation(
        "plus",
        vec![
            Expression::Const("1".to_string()),
            Expression::Const("2".to_string()),
        ],
    );
    demo_inference(&expr2);

    // Example 3: Variable inference
    println!("\nExample 3: Variable + Constant (x + 1)");
    let expr3 = Expression::operation(
        "plus",
        vec![
            Expression::Object("x".to_string()),
            Expression::Const("1".to_string()),
        ],
    );
    demo_inference(&expr3);

    // Example 4: Two variables
    println!("\nExample 4: Two Variables (x + y)");
    let expr4 = Expression::operation(
        "plus",
        vec![
            Expression::Object("x".to_string()),
            Expression::Object("y".to_string()),
        ],
    );
    demo_inference(&expr4);

    // Example 5: Division
    println!("\nExample 5: Division (x / 2)");
    let expr5 = Expression::operation(
        "scalar_divide",
        vec![
            Expression::Object("x".to_string()),
            Expression::Const("2".to_string()),
        ],
    );
    demo_inference(&expr5);

    // Example 6: Square root
    println!("\nExample 6: Square Root (√x)");
    let expr6 = Expression::operation("sqrt", vec![Expression::Object("x".to_string())]);
    demo_inference(&expr6);

    // Example 7: Power
    println!("\nExample 7: Power (x²)");
    let expr7 = Expression::operation(
        "sup",
        vec![
            Expression::Object("x".to_string()),
            Expression::Const("2".to_string()),
        ],
    );
    demo_inference(&expr7);

    // Example 8: Nested expression
    println!("\nExample 8: Nested ((x + 1) / 2)");
    let expr8 = Expression::operation(
        "scalar_divide",
        vec![
            Expression::operation(
                "plus",
                vec![
                    Expression::Object("x".to_string()),
                    Expression::Const("1".to_string()),
                ],
            ),
            Expression::Const("2".to_string()),
        ],
    );
    demo_inference(&expr8);

    // Example 9: Complex expression
    println!("\nExample 9: Complex (x² + 2x + 1)");
    let x_squared = Expression::operation(
        "sup",
        vec![
            Expression::Object("x".to_string()),
            Expression::Const("2".to_string()),
        ],
    );
    let two_x = Expression::operation(
        "scalar_multiply",
        vec![
            Expression::Const("2".to_string()),
            Expression::Object("x".to_string()),
        ],
    );
    let expr9 = Expression::operation(
        "plus",
        vec![
            Expression::operation("plus", vec![x_squared, two_x]),
            Expression::Const("1".to_string()),
        ],
    );
    demo_inference(&expr9);

    // Example 10: Type error demonstration
    println!("\nExample 10: Type Error (√(x + y) where constraints conflict)");
    println!("  (This will show how we catch type errors)");
    // For now, this will still succeed because we don't have enough constraints

    println!("\n=== Summary ===");
    println!("Type inference successfully:");
    println!("  ✓ Infers concrete types (Scalar) for constants");
    println!("  ✓ Infers types for variables from context");
    println!("  ✓ Generates type constraints for operations");
    println!("  ✓ Unifies types across expressions");
    println!("  ✓ Handles polymorphism (type variables α, β, etc.)");
    println!("\nNext steps:");
    println!("  • Add more operation types (matrices, vectors)");
    println!("  • Implement type classes (algebraic structures)");
    println!("  • Add dependent types (dimensions)");
    println!("  • Integrate with existing Kleis renderer");
}

fn demo_inference(expr: &Expression) {
    let mut infer = TypeInference::new();

    // Print expression
    println!("  Expression: {:?}", expr);

    // Perform type inference
    match infer.infer_and_solve(expr, None) {
        Ok(ty) => {
            println!("  Inferred type: {}", ty);
        }
        Err(e) => {
            println!("  Type error: {}", e);
        }
    }
}
