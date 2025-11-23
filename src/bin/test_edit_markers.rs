use kleis::ast::Expression;
use kleis::math_layout::compile_math_to_svg_with_ids;
use kleis::math_layout::typst_compiler::ArgumentBoundingBox;
use kleis::render::{RenderTarget, build_default_context, render_expression};

fn main() {
    println!("=== Edit Marker Placement Test ===\n");

    let ctx = build_default_context();

    // Test cases: (name, AST constructor)
    let test_cases = vec![
        ("Fraction", frac(o("a"), o("b"))),
        ("Nested fraction", frac(frac(o("a"), o("b")), o("c"))),
        ("Square root", sqrt(o("x"))),
        ("Power", pow(o("x"), o("2"))),
        ("Subscript", sub(o("a"), o("n"))),
        ("Matrix 2x2", matrix2x2(o("a"), o("b"), o("c"), o("d"))),
        ("Inner product", inner(o("u"), o("v"))),
        ("Integral", int_bounds(o("f(x)"), o("0"), o("1"), o("x"))),
        ("Sum", sum_bounds(o("a_n"), o("n=1"), o("\\infty"))),
    ];

    for (name, ast) in test_cases {
        println!("--- {} ---", name);

        // Render to Typst
        let typst = render_expression(&ast, &ctx, &RenderTarget::Typst);
        println!("Typst markup: {}", typst);

        // Compile to SVG with layout info
        match compile_math_to_svg_with_ids(&typst, &[]) {
            Ok(output) => {
                println!("✓ Compiled successfully");
                println!("  Placeholders: {}", output.placeholder_positions.len());
                println!("  Arguments: {}", output.argument_bounding_boxes.len());

                // Show argument bounding boxes
                for (i, arg) in output.argument_bounding_boxes.iter().enumerate() {
                    println!(
                        "  Arg {}: x={:.1}, y={:.1}, w={:.1}, h={:.1}",
                        i, arg.x, arg.y, arg.width, arg.height
                    );
                }

                // Check for overlapping boxes (potential issue)
                for i in 0..output.argument_bounding_boxes.len() {
                    for j in i + 1..output.argument_bounding_boxes.len() {
                        let a = &output.argument_bounding_boxes[i];
                        let b = &output.argument_bounding_boxes[j];
                        if boxes_overlap(a, b) {
                            println!("  ⚠️  WARNING: Args {} and {} overlap!", i, j);
                        }
                    }
                }
            }
            Err(e) => {
                println!("✗ Compilation error: {}", e);
            }
        }

        println!();
    }
}

fn boxes_overlap(a: &ArgumentBoundingBox, b: &ArgumentBoundingBox) -> bool {
    !(a.x + a.width < b.x || b.x + b.width < a.x || a.y + a.height < b.y || b.y + b.height < a.y)
}

// Helper functions to construct ASTs
fn o(s: &str) -> Expression {
    Expression::Object(s.to_string())
}
fn op(name: &str, args: Vec<Expression>) -> Expression {
    Expression::Operation {
        name: name.to_string(),
        args,
    }
}

fn frac(num: Expression, den: Expression) -> Expression {
    op("frac", vec![num, den])
}

fn sqrt(arg: Expression) -> Expression {
    op("sqrt", vec![arg])
}

fn pow(base: Expression, exp: Expression) -> Expression {
    op("sup", vec![base, exp])
}

fn sub(base: Expression, idx: Expression) -> Expression {
    op("sub", vec![base, idx])
}

fn matrix2x2(a11: Expression, a12: Expression, a21: Expression, a22: Expression) -> Expression {
    op("matrix2x2", vec![a11, a12, a21, a22])
}

fn inner(u: Expression, v: Expression) -> Expression {
    op("inner", vec![u, v])
}

fn int_bounds(
    integrand: Expression,
    lower: Expression,
    upper: Expression,
    var: Expression,
) -> Expression {
    op("int_bounds", vec![integrand, lower, upper, var])
}

fn sum_bounds(body: Expression, from: Expression, to: Expression) -> Expression {
    op("sum_bounds", vec![body, from, to])
}
