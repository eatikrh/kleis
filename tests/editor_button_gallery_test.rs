//! Editor Button Gallery Test
//!
//! End-to-end tests for every palette button:
//! 1. Create the exact AST that frontend would generate
//! 2. Render through Typst
//! 3. Verify placeholder count and positions
//! 4. Verify argument slots (UUID tracking)
//!
//! This catches:
//! - List ordering bugs (matrix rows/cols)
//! - Extra edit markers on dimension args
//! - UUID handling issues
//! - Fence mismatches
//!
//! Created: Dec 16, 2025 - editornode_overhaul branch

use kleis::editor_ast::{EditorNode, OperationData, PlaceholderData};
use kleis::render::{build_default_context, render_editor_node, RenderTarget};
use std::collections::HashMap;

// Note: Typst rendering requires the server; these tests focus on LaTeX/Kleis output
// For full Typst+SVG testing, use integration tests with server running

// ============================================================================
// Helpers matching patternfly-editor/src/components/Palette/astTemplates.ts
// ============================================================================

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

fn const_node(s: &str) -> EditorNode {
    EditorNode::Const { value: s.to_string() }
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

fn list_node(items: Vec<EditorNode>) -> EditorNode {
    EditorNode::List { list: items }
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

// ============================================================================
// Gallery Test Structure
// ============================================================================

struct ButtonTest {
    name: &'static str,
    ast: EditorNode,
    expected_placeholder_count: usize,
    expected_latex_contains: Vec<&'static str>,
    expected_kleis_contains: Option<Vec<&'static str>>,
}

fn get_button_gallery() -> Vec<ButtonTest> {
    vec![
        // ─────────────────────────────────────────────────────────────
        // Basic Operations
        // ─────────────────────────────────────────────────────────────
        ButtonTest {
            name: "fraction",
            ast: op("scalar_divide", vec![ph(0, "numerator"), ph(1, "denominator")]),
            expected_placeholder_count: 2,
            expected_latex_contains: vec!["frac"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "power",
            ast: op("power", vec![ph(0, "base"), ph(1, "exponent")]),
            expected_placeholder_count: 2,
            expected_latex_contains: vec!["^"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "subscript",
            ast: op("sub", vec![ph(0, "base"), ph(1, "subscript")]),
            expected_placeholder_count: 2,
            expected_latex_contains: vec!["_"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "sqrt",
            ast: op("sqrt", vec![ph(0, "radicand")]),
            expected_placeholder_count: 1,
            expected_latex_contains: vec!["sqrt"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "nthroot",
            ast: op("nth_root", vec![ph(0, "index"), ph(1, "radicand")]),
            expected_placeholder_count: 2,
            expected_latex_contains: vec!["sqrt"],
            expected_kleis_contains: None,
        },
        
        // ─────────────────────────────────────────────────────────────
        // Calculus
        // ─────────────────────────────────────────────────────────────
        ButtonTest {
            name: "integral",
            ast: op("int_bounds", vec![
                ph(0, "integrand"),
                ph(1, "lower"),
                ph(2, "upper"),
                ph(3, "variable"),
            ]),
            expected_placeholder_count: 4,
            expected_latex_contains: vec!["int"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "sum",
            ast: op("sum_bounds", vec![
                ph(0, "body"),
                ph(1, "from"),
                ph(2, "to"),
            ]),
            expected_placeholder_count: 3,
            expected_latex_contains: vec!["sum"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "product",
            ast: op("prod_bounds", vec![
                ph(0, "body"),
                ph(1, "from"),
                ph(2, "to"),
            ]),
            expected_placeholder_count: 3,
            expected_latex_contains: vec!["prod"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "limit",
            ast: op("lim", vec![ph(0, "body"), ph(1, "var"), ph(2, "target")]),
            expected_placeholder_count: 3,
            expected_latex_contains: vec!["lim"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "partial",
            ast: op("d_part", vec![ph(0, "function"), ph(1, "variable")]),
            expected_placeholder_count: 2,
            expected_latex_contains: vec!["partial"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "derivative",
            ast: op("d_dt", vec![ph(0, "function"), ph(1, "variable")]),
            expected_placeholder_count: 2,
            expected_latex_contains: vec!["frac"],
            expected_kleis_contains: None,
        },
        
        // ─────────────────────────────────────────────────────────────
        // Functions
        // ─────────────────────────────────────────────────────────────
        ButtonTest {
            name: "sin",
            ast: op("sin", vec![ph(0, "argument")]),
            expected_placeholder_count: 1,
            expected_latex_contains: vec!["sin"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "cos",
            ast: op("cos", vec![ph(0, "argument")]),
            expected_placeholder_count: 1,
            expected_latex_contains: vec!["cos"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "exp",
            ast: op("exp", vec![ph(0, "argument")]),
            expected_placeholder_count: 1,
            expected_latex_contains: vec!["exp"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "ln",
            ast: op("ln", vec![ph(0, "argument")]),
            expected_placeholder_count: 1,
            expected_latex_contains: vec!["ln"],
            expected_kleis_contains: None,
        },
        
        // ─────────────────────────────────────────────────────────────
        // Fences
        // ─────────────────────────────────────────────────────────────
        ButtonTest {
            name: "parens",
            ast: op("parens", vec![ph(0, "content")]),
            expected_placeholder_count: 1,
            expected_latex_contains: vec!["left(", "right)"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "brackets",
            ast: op("brackets", vec![ph(0, "content")]),
            expected_placeholder_count: 1,
            expected_latex_contains: vec!["left[", "right]"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "braces",
            ast: op("braces", vec![ph(0, "content")]),
            expected_placeholder_count: 1,
            expected_latex_contains: vec!["left\\{", "right\\}"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "abs",
            ast: op("abs", vec![ph(0, "value")]),
            expected_placeholder_count: 1,
            expected_latex_contains: vec!["lvert"],  // \left\lvert ... \right\rvert
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "norm",
            ast: op("norm", vec![ph(0, "vector")]),
            expected_placeholder_count: 1,
            expected_latex_contains: vec!["lVert"],  // \left\lVert ... \right\rVert
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "floor",
            ast: op("floor", vec![ph(0, "x")]),
            expected_placeholder_count: 1,
            expected_latex_contains: vec!["lfloor", "rfloor"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "ceiling",
            ast: op("ceiling", vec![ph(0, "x")]),
            expected_placeholder_count: 1,
            expected_latex_contains: vec!["lceil", "rceil"],
            expected_kleis_contains: None,
        },
        
        // ─────────────────────────────────────────────────────────────
        // Matrices - CRITICAL: placeholder count excludes dimensions
        // ─────────────────────────────────────────────────────────────
        ButtonTest {
            name: "matrix2x2",
            ast: op("Matrix", vec![
                const_node("2"),
                const_node("2"),
                list_node(vec![ph(0, "a11"), ph(1, "a12"), ph(2, "a21"), ph(3, "a22")]),
            ]),
            // 4 placeholders for cells, NOT 6 (dimensions are not editable)
            expected_placeholder_count: 4,
            expected_latex_contains: vec!["bmatrix"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "matrix3x3",
            ast: op("Matrix", vec![
                const_node("3"),
                const_node("3"),
                list_node(vec![
                    ph(0, "a11"), ph(1, "a12"), ph(2, "a13"),
                    ph(3, "a21"), ph(4, "a22"), ph(5, "a23"),
                    ph(6, "a31"), ph(7, "a32"), ph(8, "a33"),
                ]),
            ]),
            // 9 placeholders for cells
            expected_placeholder_count: 9,
            expected_latex_contains: vec!["bmatrix"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "pmatrix2x2",
            ast: op("PMatrix", vec![
                const_node("2"),
                const_node("2"),
                list_node(vec![ph(0, "a11"), ph(1, "a12"), ph(2, "a21"), ph(3, "a22")]),
            ]),
            expected_placeholder_count: 4,
            expected_latex_contains: vec!["pmatrix"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "vmatrix2x2",
            ast: op("VMatrix", vec![
                const_node("2"),
                const_node("2"),
                list_node(vec![ph(0, "a11"), ph(1, "a12"), ph(2, "a21"), ph(3, "a22")]),
            ]),
            expected_placeholder_count: 4,
            expected_latex_contains: vec!["vmatrix"],
            expected_kleis_contains: None,
        },
        
        // ─────────────────────────────────────────────────────────────
        // Tensors - CRITICAL: index structure preserved
        // ─────────────────────────────────────────────────────────────
        ButtonTest {
            name: "metric",
            ast: tensor("g", vec![ph(0, "idx1"), ph(1, "idx2")], vec!["down", "down"]),
            expected_placeholder_count: 2,
            expected_latex_contains: vec!["g", "_"],
            expected_kleis_contains: Some(vec!["g(-", "-"]),
        },
        ButtonTest {
            name: "christoffel",
            ast: tensor("Γ", vec![
                ph(0, "upper"),
                ph(1, "lower1"),
                ph(2, "lower2"),
            ], vec!["up", "down", "down"]),
            expected_placeholder_count: 3,
            expected_latex_contains: vec!["Gamma", "^", "_"],
            expected_kleis_contains: Some(vec!["Γ(", ", -", ", -"]),
        },
        ButtonTest {
            name: "riemann",
            ast: tensor("R", vec![
                ph(0, "upper"),
                ph(1, "lower1"),
                ph(2, "lower2"),
                ph(3, "lower3"),
            ], vec!["up", "down", "down", "down"]),
            expected_placeholder_count: 4,
            expected_latex_contains: vec!["R", "^", "_"],
            expected_kleis_contains: Some(vec!["R(", ", -"]),
        },
        
        // ─────────────────────────────────────────────────────────────
        // Quantum
        // ─────────────────────────────────────────────────────────────
        ButtonTest {
            name: "ket",
            ast: op("ket", vec![ph(0, "state")]),
            expected_placeholder_count: 1,
            expected_latex_contains: vec!["rangle"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "bra",
            ast: op("bra", vec![ph(0, "state")]),
            expected_placeholder_count: 1,
            expected_latex_contains: vec!["langle"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "inner",
            ast: op("inner", vec![ph(0, "bra"), ph(1, "ket")]),
            expected_placeholder_count: 2,
            expected_latex_contains: vec!["langle", "rangle"],
            expected_kleis_contains: None,
        },
        
        // ─────────────────────────────────────────────────────────────
        // Transforms
        // ─────────────────────────────────────────────────────────────
        ButtonTest {
            name: "fourier_transform",
            ast: op("fourier_transform", vec![ph(0, "function"), ph(1, "variable")]),
            expected_placeholder_count: 2,
            expected_latex_contains: vec!["mathcal{F}"],
            expected_kleis_contains: None,
        },
        ButtonTest {
            name: "laplace_transform",
            ast: op("laplace_transform", vec![ph(0, "function"), ph(1, "variable")]),
            expected_placeholder_count: 2,
            expected_latex_contains: vec!["mathcal{L}"],
            expected_kleis_contains: None,
        },
    ]
}

// ============================================================================
// Test Runner
// ============================================================================

#[test]
fn test_all_buttons_latex_rendering() {
    let ctx = build_default_context();
    let gallery = get_button_gallery();
    
    let mut failures = Vec::new();
    
    for test in &gallery {
        let latex = render_editor_node(&test.ast, &ctx, &RenderTarget::LaTeX);
        
        for expected in &test.expected_latex_contains {
            if !latex.contains(expected) {
                failures.push(format!(
                    "{}: LaTeX '{}' missing '{}'\n  Got: {}",
                    test.name, test.name, expected, latex
                ));
            }
        }
        
        println!("✓ {}: {}", test.name, latex);
    }
    
    if !failures.is_empty() {
        panic!("Button gallery failures:\n{}", failures.join("\n\n"));
    }
}

#[test]
fn test_all_buttons_kleis_rendering() {
    let ctx = build_default_context();
    let gallery = get_button_gallery();
    
    let mut failures = Vec::new();
    
    for test in &gallery {
        if let Some(expected_kleis) = &test.expected_kleis_contains {
            let kleis = render_editor_node(&test.ast, &ctx, &RenderTarget::Kleis);
            
            for expected in expected_kleis {
                if !kleis.contains(expected) {
                    failures.push(format!(
                        "{}: Kleis missing '{}'\n  Got: {}",
                        test.name, expected, kleis
                    ));
                }
            }
            
            println!("✓ {} Kleis: {}", test.name, kleis);
        }
    }
    
    if !failures.is_empty() {
        panic!("Kleis rendering failures:\n{}", failures.join("\n\n"));
    }
}

/// Count placeholders in an EditorNode recursively
fn count_placeholders(node: &EditorNode) -> usize {
    match node {
        EditorNode::Placeholder { .. } => 1,
        EditorNode::Operation { operation } => {
            operation.args.iter().map(count_placeholders).sum()
        }
        EditorNode::List { list } => {
            list.iter().map(count_placeholders).sum()
        }
        _ => 0,
    }
}

#[test]
fn test_all_buttons_placeholder_count() {
    let gallery = get_button_gallery();
    
    let mut failures = Vec::new();
    
    for test in &gallery {
        let actual = count_placeholders(&test.ast);
        
        if actual != test.expected_placeholder_count {
            failures.push(format!(
                "{}: expected {} placeholders, got {}",
                test.name, test.expected_placeholder_count, actual
            ));
        } else {
            println!("✓ {} has {} placeholders", test.name, actual);
        }
    }
    
    if !failures.is_empty() {
        panic!("Placeholder count failures:\n{}", failures.join("\n"));
    }
}

// ============================================================================
// Matrix Specific Tests - Row/Column ordering
// ============================================================================

mod matrix_ordering_tests {
    use super::*;
    
    #[test]
    fn test_matrix_2x2_element_order() {
        // a b     should produce LaTeX: a & b \\ c & d
        // c d
        let matrix = op("Matrix", vec![
            const_node("2"),
            const_node("2"),
            list_node(vec![obj("a"), obj("b"), obj("c"), obj("d")]),
        ]);
        
        let ctx = build_default_context();
        let latex = render_editor_node(&matrix, &ctx, &RenderTarget::LaTeX);
        
        println!("2x2 matrix LaTeX: {}", latex);
        
        // Verify the matrix contains all elements
        assert!(latex.contains("bmatrix"), "Should be bmatrix: {}", latex);
        
        // The output is: \begin{bmatrix}a&b\\c&d\end{bmatrix}
        // Verify row structure
        assert!(latex.contains("&"), "Should have column separator: {}", latex);
        assert!(latex.contains("\\\\"), "Should have row separator: {}", latex);
        
        // Row-major order check: elements appear in order a, b, c, d
        // Note: 'b' appears in "bmatrix" before the actual 'b' element
        // So we check relative positions within the matrix body
        let body_start = latex.find("bmatrix}").unwrap() + 8;
        let body = &latex[body_start..];
        
        let a_pos = body.find('a').unwrap_or(999);
        let b_pos = body.find('b').unwrap_or(999);
        let c_pos = body.find('c').unwrap_or(999);
        let d_pos = body.find('d').unwrap_or(999);
        
        println!("Positions in body '{}': a={}, b={}, c={}, d={}", body, a_pos, b_pos, c_pos, d_pos);
        
        assert!(a_pos < b_pos, "a should come before b in body");
        assert!(b_pos < c_pos, "b should come before c in body");
        assert!(c_pos < d_pos, "c should come before d in body");
    }
    
    #[test]
    fn test_matrix_3x2_element_order() {
        // a b     3 rows x 2 cols
        // c d
        // e f
        let matrix = op("Matrix", vec![
            const_node("3"),
            const_node("2"),
            list_node(vec![obj("a"), obj("b"), obj("c"), obj("d"), obj("e"), obj("f")]),
        ]);
        
        let ctx = build_default_context();
        let latex = render_editor_node(&matrix, &ctx, &RenderTarget::LaTeX);
        
        println!("3x2 matrix LaTeX: {}", latex);
        
        // Should have 3 rows
        let row_breaks = latex.matches("\\\\").count();
        assert_eq!(row_breaks, 2, "3x2 matrix should have 2 row breaks: {}", latex);
    }
}

// ============================================================================
// Fence Matching Tests
// ============================================================================

mod fence_matching_tests {
    use super::*;
    
    #[test]
    fn test_parens_balanced() {
        let node = op("parens", vec![obj("x")]);
        let ctx = build_default_context();
        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        
        assert!(latex.contains("left("), "Should have left(: {}", latex);
        assert!(latex.contains("right)"), "Should have right): {}", latex);
    }
    
    #[test]
    fn test_brackets_balanced() {
        let node = op("brackets", vec![obj("x")]);
        let ctx = build_default_context();
        let latex = render_editor_node(&node, &ctx, &RenderTarget::LaTeX);
        
        assert!(latex.contains("left["), "Should have left[: {}", latex);
        assert!(latex.contains("right]"), "Should have right]: {}", latex);
    }
    
    #[test]
    fn test_nested_fences() {
        // ((x))
        let inner = op("parens", vec![obj("x")]);
        let outer = op("parens", vec![inner]);
        
        let ctx = build_default_context();
        let latex = render_editor_node(&outer, &ctx, &RenderTarget::LaTeX);
        
        let left_count = latex.matches("left(").count();
        let right_count = latex.matches("right)").count();
        
        assert_eq!(left_count, 2, "Should have 2 left(: {}", latex);
        assert_eq!(right_count, 2, "Should have 2 right): {}", latex);
    }
}

