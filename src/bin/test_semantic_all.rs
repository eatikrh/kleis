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
        ("Fraction", op("frac", vec![o("a"), o("b")])),
        (
            "Nested fraction",
            op("frac", vec![op("frac", vec![o("a"), o("b")]), o("c")]),
        ),
        ("Square root", op("sqrt", vec![o("x")])),
        ("Power", op("sup", vec![o("x"), o("2")])),
        ("Subscript", op("sub", vec![o("a"), o("n")])),
        (
            "Matrix 2x2",
            op("matrix2x2", vec![o("a"), o("b"), o("c"), o("d")]),
        ),
        ("Inner product", op("inner", vec![o("u"), o("v")])),
        (
            "Integral",
            op("int_bounds", vec![o("f(x)"), o("0"), o("1"), o("x")]),
        ),
        (
            "Sum",
            op("sum_bounds", vec![o("a_n"), o("n=1"), o("\\infty")]),
        ),
        ("Commutator", op("commutator", vec![o("A"), o("B")])),
    ];

    println!("=== Testing Semantic Bounding Boxes Across Templates ===\n");

    for (name, ast) in tests {
        print!("{:20} ", name);
        match compile_with_semantic_boxes(&ast, &[], &[]) {
            Ok(output) => {
                let expected = match &ast {
                    Expression::Operation { args, .. } => args.len(),
                    _ => 0,
                };
                let actual = output.argument_bounding_boxes.len();

                if actual == expected {
                    println!("✅ {} args (expected {})", actual, expected);
                } else {
                    println!("⚠️  {} args (expected {})", actual, expected);
                }
            }
            Err(e) => println!("❌ Error: {}", e),
        }
    }
}
