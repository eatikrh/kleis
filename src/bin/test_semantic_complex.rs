#![allow(warnings)]
#![allow(clippy::all, unreachable_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use kleis::ast::Expression;
use kleis::math_layout::compile_with_semantic_boxes;

fn o(s: &str) -> Expression {
    Expression::Object(s.to_string())
}
fn op(name: &str, args: Vec<Expression>) -> Expression {
    Expression::Operation {
        name: name.to_string(),
        args,
    }
}

fn main() {
    let tests = vec![
        ("Inner product", op("inner", vec![o("u"), o("v")])),
        (
            "Integral",
            op("int_bounds", vec![o("f(x)"), o("0"), o("1"), o("x")]),
        ),
    ];

    for (name, ast) in tests {
        println!("\n=== {} ===", name);
        match compile_with_semantic_boxes(&ast, &[], &[]) {
            Ok(output) => {
                println!("✓ {} argument boxes", output.argument_bounding_boxes.len());
                for bbox in &output.argument_bounding_boxes {
                    println!("  Arg {}: node_id={}", bbox.arg_index, bbox.node_id);
                }
            }
            Err(e) => println!("✗ Error: {}", e),
        }
    }
}
