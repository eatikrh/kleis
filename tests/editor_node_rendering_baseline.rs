//! EditorNode Rendering Baseline Tests
//!
//! These tests capture the CURRENT rendering behavior before any refactoring.
//! Run these BEFORE making changes to establish a baseline.
//! Run these AFTER making changes to ensure no regressions.
//!
//! Created: Dec 16, 2025 - editornode_overhaul branch

use kleis::editor_ast::{EditorNode, OperationData, PlaceholderData};
use kleis::render::{build_default_context, render_editor_node, RenderTarget};
use std::collections::HashMap;

/// Helper to create test nodes quickly
fn ph(id: usize, hint: &str) -> EditorNode {
    EditorNode::Placeholder {
        placeholder: PlaceholderData {
            id,
            hint: Some(hint.to_string()),
        },
    }
}

fn obj(s: &str) -> EditorNode {
    EditorNode::Object { object: s.to_string() }
}

fn op(name: &str, args: Vec<EditorNode>) -> EditorNode {
    EditorNode::Operation {
        operation: OperationData {
            name: name.to_string(),
            args,
            kind: None,
            metadata: None,
        },
    }
}

fn op_with_kind(name: &str, args: Vec<EditorNode>, kind: &str) -> EditorNode {
    EditorNode::Operation {
        operation: OperationData {
            name: name.to_string(),
            args,
            kind: Some(kind.to_string()),
            metadata: None,
        },
    }
}

fn tensor(symbol: &str, indices: Vec<EditorNode>, index_structure: Vec<&str>) -> EditorNode {
    let mut metadata = HashMap::new();
    let structure: Vec<serde_json::Value> = index_structure
        .into_iter()
        .map(|s| serde_json::Value::String(s.to_string()))
        .collect();
    metadata.insert(
        "indexStructure".to_string(),
        serde_json::Value::Array(structure),
    );

    let mut args = vec![obj(symbol)];
    args.extend(indices);

    EditorNode::Operation {
        operation: OperationData {
            name: "tensor".to_string(),
            args,
            kind: Some("tensor".to_string()),
            metadata: Some(metadata),
        },
    }
}

fn const_node(s: &str) -> EditorNode {
    EditorNode::Const { value: s.to_string() }
}

fn list_node(items: Vec<EditorNode>) -> EditorNode {
    EditorNode::List { list: items }
}

// ============================================================================
// BASELINE TESTS - Capture current behavior
// ============================================================================

mod baseline_basic_operations {
    use super::*;

    #[test]
    fn test_fraction() {
        let ctx = build_default_context();
        let node = op("scalar_divide", vec![ph(0, "num"), ph(1, "den")]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        let unicode = render_editor_node(&node, &ctx, &RenderTarget::Unicode);

        // Capture current output - these are the BASELINE
        assert!(latex.contains("frac"), "LaTeX should contain frac: {}", latex);
        println!("BASELINE fraction LaTeX: {}", latex);
        println!("BASELINE fraction Unicode: {}", unicode);
    }

    #[test]
    fn test_power() {
        let ctx = build_default_context();
        let node = op("power", vec![obj("x"), obj("2")]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        let unicode = render_editor_node(&node, &ctx, &RenderTarget::Unicode);

        assert!(latex.contains("^"), "LaTeX should contain ^: {}", latex);
        println!("BASELINE power LaTeX: {}", latex);
        println!("BASELINE power Unicode: {}", unicode);
    }

    #[test]
    fn test_sqrt() {
        let ctx = build_default_context();
        let node = op("sqrt", vec![obj("x")]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        let unicode = render_editor_node(&node, &ctx, &RenderTarget::Unicode);

        assert!(latex.contains("sqrt"), "LaTeX should contain sqrt: {}", latex);
        println!("BASELINE sqrt LaTeX: {}", latex);
        println!("BASELINE sqrt Unicode: {}", unicode);
    }

    #[test]
    fn test_sin() {
        let ctx = build_default_context();
        let node = op("sin", vec![obj("x")]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        let unicode = render_editor_node(&node, &ctx, &RenderTarget::Unicode);

        assert!(latex.contains("sin"), "LaTeX should contain sin: {}", latex);
        println!("BASELINE sin LaTeX: {}", latex);
        println!("BASELINE sin Unicode: {}", unicode);
    }

    #[test]
    fn test_parens() {
        let ctx = build_default_context();
        let node = op("parens", vec![obj("x")]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        let unicode = render_editor_node(&node, &ctx, &RenderTarget::Unicode);

        assert!(latex.contains("(") && latex.contains(")"), "LaTeX should have parens: {}", latex);
        println!("BASELINE parens LaTeX: {}", latex);
        println!("BASELINE parens Unicode: {}", unicode);
    }
}

mod baseline_calculus {
    use super::*;

    #[test]
    fn test_integral_with_bounds() {
        let ctx = build_default_context();
        let node = op("int_bounds", vec![
            obj("f"),
            obj("0"),
            obj("1"),
            obj("x"),
        ]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        let unicode = render_editor_node(&node, &ctx, &RenderTarget::Unicode);

        assert!(latex.contains("int"), "LaTeX should contain int: {}", latex);
        println!("BASELINE integral LaTeX: {}", latex);
        println!("BASELINE integral Unicode: {}", unicode);
    }

    #[test]
    fn test_sum_with_bounds() {
        let ctx = build_default_context();
        let node = op("sum_bounds", vec![
            obj("a_i"),
            obj("i=1"),
            obj("n"),
        ]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        let unicode = render_editor_node(&node, &ctx, &RenderTarget::Unicode);

        assert!(latex.contains("sum"), "LaTeX should contain sum: {}", latex);
        println!("BASELINE sum LaTeX: {}", latex);
        println!("BASELINE sum Unicode: {}", unicode);
    }

    #[test]
    fn test_partial_derivative() {
        let ctx = build_default_context();
        let node = op("d_part", vec![obj("f"), obj("x")]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        let unicode = render_editor_node(&node, &ctx, &RenderTarget::Unicode);

        println!("BASELINE partial LaTeX: {}", latex);
        println!("BASELINE partial Unicode: {}", unicode);
    }

    #[test]
    fn test_limit() {
        let ctx = build_default_context();
        let node = op("lim", vec![obj("f(x)"), obj("x"), obj("0")]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        let unicode = render_editor_node(&node, &ctx, &RenderTarget::Unicode);

        println!("BASELINE limit LaTeX: {}", latex);
        println!("BASELINE limit Unicode: {}", unicode);
    }
}

mod baseline_tensors {
    use super::*;

    #[test]
    fn test_christoffel_with_metadata() {
        let ctx = build_default_context();
        let node = tensor("Γ", vec![obj("λ"), obj("μ"), obj("ν")], vec!["up", "down", "down"]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        let kleis = render_editor_node(&node, &ctx, &RenderTarget::Kleis);
        let unicode = render_editor_node(&node, &ctx, &RenderTarget::Unicode);

        assert!(latex.contains("Gamma"), "LaTeX should contain Gamma: {}", latex);
        println!("BASELINE christoffel LaTeX: {}", latex);
        println!("BASELINE christoffel Kleis: {}", kleis);
        println!("BASELINE christoffel Unicode: {}", unicode);
    }

    #[test]
    fn test_metric_tensor() {
        let ctx = build_default_context();
        let node = tensor("g", vec![obj("μ"), obj("ν")], vec!["down", "down"]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        let kleis = render_editor_node(&node, &ctx, &RenderTarget::Kleis);

        println!("BASELINE metric LaTeX: {}", latex);
        println!("BASELINE metric Kleis: {}", kleis);
    }

    #[test]
    fn test_riemann_tensor() {
        let ctx = build_default_context();
        let node = tensor("R", vec![obj("ρ"), obj("σ"), obj("μ"), obj("ν")], vec!["up", "down", "down", "down"]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        let kleis = render_editor_node(&node, &ctx, &RenderTarget::Kleis);

        println!("BASELINE riemann LaTeX: {}", latex);
        println!("BASELINE riemann Kleis: {}", kleis);
    }
}

mod baseline_matrices {
    use super::*;

    #[test]
    fn test_matrix_2x2() {
        let ctx = build_default_context();
        let node = op("Matrix", vec![
            const_node("2"),
            const_node("2"),
            list_node(vec![obj("a"), obj("b"), obj("c"), obj("d")]),
        ]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        let unicode = render_editor_node(&node, &ctx, &RenderTarget::Unicode);

        println!("BASELINE matrix2x2 LaTeX: {}", latex);
        println!("BASELINE matrix2x2 Unicode: {}", unicode);
    }

    #[test]
    fn test_pmatrix_2x2() {
        let ctx = build_default_context();
        let node = op("PMatrix", vec![
            const_node("2"),
            const_node("2"),
            list_node(vec![obj("a"), obj("b"), obj("c"), obj("d")]),
        ]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);

        println!("BASELINE pmatrix2x2 LaTeX: {}", latex);
    }
}

mod baseline_quantum {
    use super::*;

    #[test]
    fn test_ket() {
        let ctx = build_default_context();
        let node = op("ket", vec![obj("ψ")]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);

        assert!(latex.contains("rangle"), "LaTeX should contain rangle: {}", latex);
        println!("BASELINE ket LaTeX: {}", latex);
    }

    #[test]
    fn test_bra() {
        let ctx = build_default_context();
        let node = op("bra", vec![obj("φ")]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);

        assert!(latex.contains("langle"), "LaTeX should contain langle: {}", latex);
        println!("BASELINE bra LaTeX: {}", latex);
    }

    #[test]
    fn test_inner_product() {
        let ctx = build_default_context();
        let node = op("inner", vec![obj("φ"), obj("ψ")]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);

        println!("BASELINE inner LaTeX: {}", latex);
    }
}

mod baseline_transforms {
    use super::*;

    #[test]
    fn test_fourier_transform() {
        let ctx = build_default_context();
        let node = op("fourier_transform", vec![obj("f"), obj("ω")]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        let unicode = render_editor_node(&node, &ctx, &RenderTarget::Unicode);

        println!("BASELINE fourier LaTeX: {}", latex);
        println!("BASELINE fourier Unicode: {}", unicode);
    }

    #[test]
    fn test_laplace_transform() {
        let ctx = build_default_context();
        let node = op("laplace_transform", vec![obj("f"), obj("s")]);

        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        let unicode = render_editor_node(&node, &ctx, &RenderTarget::Unicode);

        println!("BASELINE laplace LaTeX: {}", latex);
        println!("BASELINE laplace Unicode: {}", unicode);
    }
}

mod baseline_nested {
    use super::*;

    #[test]
    fn test_nested_function_calls() {
        let ctx = build_default_context();
        // f(g(h(x)))
        let inner = op("function_call", vec![obj("h"), obj("x")]);
        let middle = op("function_call", vec![obj("g"), inner]);
        let outer = op("function_call", vec![obj("f"), middle]);

        let latex = render_editor_node(&outer, &ctx, &RenderTarget::LaTeX);
        let unicode = render_editor_node(&outer, &ctx, &RenderTarget::Unicode);

        println!("BASELINE nested f(g(h(x))) LaTeX: {}", latex);
        println!("BASELINE nested f(g(h(x))) Unicode: {}", unicode);
    }

    #[test]
    fn test_integral_of_fraction() {
        let ctx = build_default_context();
        // ∫ (1/x) dx
        let fraction = op("scalar_divide", vec![const_node("1"), obj("x")]);
        let integral = op("int_bounds", vec![fraction, obj("1"), obj("∞"), obj("x")]);

        let latex = render_editor_node(&integral, &ctx, &RenderTarget::LaTeX);

        println!("BASELINE integral of fraction LaTeX: {}", latex);
    }

    #[test]
    fn test_sum_of_powers() {
        let ctx = build_default_context();
        // Σ x^n
        let power = op("power", vec![obj("x"), obj("n")]);
        let sum = op("sum_bounds", vec![power, obj("n=0"), obj("∞")]);

        let latex = render_editor_node(&sum, &ctx, &RenderTarget::LaTeX);

        println!("BASELINE sum of powers LaTeX: {}", latex);
    }
}

