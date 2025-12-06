use kleis::ast::Expression;
use kleis::math_layout::compile_with_semantic_boxes;
use kleis::render::{RenderTarget, build_default_context, render_expression};

#[derive(Debug)]
struct ArgumentSlot {
    id: usize,
    path: Vec<usize>,
    hint: String,
    is_placeholder: bool,
    role: Option<String>,
}

fn main() {
    run_case(
        "Matrix placeholders",
        matrix_with_placeholders(),
        vec![0, 1, 2, 3],
    );
    println!("\n========================================\n");
    run_case("Matrix with sin in (2,1)", matrix_with_sin(), Vec::new());
}

fn run_case(label: &str, ast: Expression, placeholder_ids: Vec<usize>) {
    println!("=== {label} ===");
    let ctx = build_default_context();
    let typst_markup = render_expression(&ast, &ctx, &RenderTarget::Typst);
    println!("Typst markup: {typst_markup}");

    let output =
        compile_with_semantic_boxes(&ast, &placeholder_ids, &placeholder_ids).expect("compile");
    if let Ok(dir) = std::env::var("DEBUG_SVG_DIR") {
        let path = std::path::Path::new(&dir).join(format!("{}.svg", label.replace(' ', "_")));
        if let Err(err) = std::fs::write(&path, &output.svg) {
            eprintln!("Failed to write SVG dump to {:?}: {}", path, err);
        } else {
            println!("Wrote SVG dump to {:?}", path);
        }
    }
    println!(
        "argument_bounding_boxes ({} total):",
        output.argument_bounding_boxes.len()
    );
    for box_info in &output.argument_bounding_boxes {
        println!(
            "  node {node_id} arg{} -> x={:.1} y={:.1} w={:.1} h={:.1}",
            box_info.arg_index,
            box_info.x,
            box_info.y,
            box_info.width,
            box_info.height,
            node_id = box_info.node_id
        );
    }

    println!("\nplaceholder_positions:");
    for ph in &output.placeholder_positions {
        println!(
            "  placeholder {} -> x={:.1} y={:.1} w={:.1} h={:.1}",
            ph.id, ph.x, ph.y, ph.width, ph.height
        );
    }

    println!("\nargument_slots:");
    let slots = collect_argument_slots(&ast);
    for slot in slots {
        println!(
            "  slot id={} path={:?} hint={} placeholder={} role={:?}",
            slot.id, slot.path, slot.hint, slot.is_placeholder, slot.role
        );
    }
}

fn o<S: Into<String>>(s: S) -> Expression {
    Expression::Object(s.into())
}

fn matrix2x2(a11: Expression, a12: Expression, a21: Expression, a22: Expression) -> Expression {
    Expression::Operation {
        name: "matrix2x2".to_string(),
        args: vec![a11, a12, a21, a22],
    }
}

fn matrix_with_placeholders() -> Expression {
    matrix2x2(
        placeholder(0, "a11"),
        placeholder(1, "a12"),
        placeholder(2, "a21"),
        placeholder(3, "a22"),
    )
}

fn matrix_with_sin() -> Expression {
    matrix2x2(o("a"), o("b"), sin(o("x")), o("d"))
}

fn placeholder(id: usize, hint: &str) -> Expression {
    Expression::Placeholder {
        id,
        hint: hint.to_string(),
    }
}

fn sin(arg: Expression) -> Expression {
    Expression::Operation {
        name: "sin".to_string(),
        args: vec![arg],
    }
}

fn collect_argument_slots(expr: &Expression) -> Vec<ArgumentSlot> {
    let mut slots = Vec::new();
    let mut next_auto_id = 1000;
    collect_slots_recursive(expr, &mut slots, &mut next_auto_id, vec![], None);
    slots
}

fn collect_slots_recursive(
    expr: &Expression,
    slots: &mut Vec<ArgumentSlot>,
    next_auto_id: &mut usize,
    path: Vec<usize>,
    role: Option<String>,
) {
    match expr {
        Expression::Placeholder { id, hint } => slots.push(ArgumentSlot {
            id: *id,
            path,
            hint: hint.clone(),
            is_placeholder: true,
            role,
        }),
        Expression::Const(value) | Expression::Object(value) => {
            slots.push(ArgumentSlot {
                id: *next_auto_id,
                path,
                hint: format!("value: {value}"),
                is_placeholder: false,
                role,
            });
            *next_auto_id += 1;
        }
        Expression::Operation { name, args } => {
            for (i, arg) in args.iter().enumerate() {
                let mut child_path = path.clone();
                child_path.push(i);
                let child_role = determine_arg_role(name, i);
                collect_slots_recursive(arg, slots, next_auto_id, child_path, child_role);
            }
        }
    }
}

fn determine_arg_role(op_name: &str, arg_index: usize) -> Option<String> {
    match op_name {
        "sup" | "power" => match arg_index {
            0 => Some("base".into()),
            1 => Some("superscript".into()),
            _ => None,
        },
        "sub" => match arg_index {
            0 => Some("base".into()),
            1 => Some("subscript".into()),
            _ => None,
        },
        "index" | "index_mixed" | "tensor_mixed" => match arg_index {
            0 => Some("base".into()),
            1 => Some("superscript".into()),
            2 => Some("subscript".into()),
            _ => None,
        },
        _ => None,
    }
}
