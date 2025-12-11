use kleis::ast::Expression;
use kleis::math_layout::compile_with_semantic_boxes;
use kleis::render::{build_default_context, render_expression, RenderTarget};

fn main() {
    let matrix = complex_matrix();
    let ctx = build_default_context();
    let typst = render_expression(&matrix, &ctx, &RenderTarget::Typst);
    println!("=== Complex Matrix Placeholder Test ===");
    println!("Typst markup:\n{}\n", typst);

    let placeholder_ids = collect_placeholder_ids(&matrix);
    println!("Placeholder IDs: {:?}\n", placeholder_ids);

    match compile_with_semantic_boxes(&matrix, &placeholder_ids, &placeholder_ids) {
        Ok(output) => {
            println!(
                "✅ compile_with_semantic_boxes succeeded ({} placeholders, {} argument boxes)\n",
                output.placeholder_positions.len(),
                output.argument_bounding_boxes.len()
            );

            println!("Placeholder positions:");
            for ph in &output.placeholder_positions {
                println!(
                    "  id {:>2}: x={:>6.1}, y={:>6.1}, w={:>5.1}, h={:>5.1}",
                    ph.id, ph.x, ph.y, ph.width, ph.height
                );
            }

            println!("\nArgument bounding boxes:");
            for arg in &output.argument_bounding_boxes {
                println!(
                    "  node {:<6} (arg {:>2}): x={:>6.1}, y={:>6.1}, w={:>5.1}, h={:>5.1}",
                    arg.node_id, arg.arg_index, arg.x, arg.y, arg.width, arg.height
                );
            }
        }
        Err(err) => {
            eprintln!("❌ Failed to compile complex matrix: {}", err);
            std::process::exit(1);
        }
    }
}

fn complex_matrix() -> Expression {
    let mut id_gen = IdGen::default();
    Expression::Operation {
        name: "matrix3x3".to_string(),
        args: vec![
            trig_op("sin", placeholder(&mut id_gen, "θ₁")),
            trig_op("cos", placeholder(&mut id_gen, "θ₂")),
            trig_op("tan", placeholder(&mut id_gen, "θ₃")),
            riemann_cell(&mut id_gen),
            christoffel_cell(&mut id_gen),
            gradient_cell(&mut id_gen),
            placeholder(&mut id_gen, "a31"),
            trig_op("sin", riemann_cell(&mut id_gen)),
            placeholder(&mut id_gen, "a33"),
        ],
    }
}

fn trig_op(op: &str, arg: Expression) -> Expression {
    Expression::Operation {
        name: op.to_string(),
        args: vec![arg],
    }
}

fn riemann_cell(id_gen: &mut IdGen) -> Expression {
    Expression::Operation {
        name: "riemann".to_string(),
        args: vec![
            Expression::Object("R".to_string()),
            placeholder(id_gen, "upper"),
            placeholder(id_gen, "lower₁"),
            placeholder(id_gen, "lower₂"),
            placeholder(id_gen, "lower₃"),
        ],
    }
}

fn christoffel_cell(id_gen: &mut IdGen) -> Expression {
    Expression::Operation {
        name: "gamma".to_string(),
        args: vec![
            placeholder(id_gen, "upper"),
            placeholder(id_gen, "lower₁"),
            placeholder(id_gen, "lower₂"),
        ],
    }
}

fn gradient_cell(id_gen: &mut IdGen) -> Expression {
    Expression::Operation {
        name: "grad".to_string(),
        args: vec![placeholder(id_gen, "potential")],
    }
}

#[derive(Default)]
struct IdGen {
    next: usize,
}

impl IdGen {
    fn next(&mut self) -> usize {
        let id = self.next;
        self.next += 1;
        id
    }
}

fn placeholder(id_gen: &mut IdGen, hint: &str) -> Expression {
    Expression::Placeholder {
        id: id_gen.next(),
        hint: hint.to_string(),
    }
}

fn collect_placeholder_ids(expr: &Expression) -> Vec<usize> {
    fn walk(expr: &Expression, ids: &mut Vec<usize>) {
        match expr {
            Expression::Placeholder { id, .. } => ids.push(*id),
            Expression::Operation { args, .. } => {
                for arg in args {
                    walk(arg, ids);
                }
            }
            _ => {}
        }
    }

    let mut ids = Vec::new();
    walk(expr, &mut ids);
    ids
}
