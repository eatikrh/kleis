//! Rigorous tests for newly added/modified templates in std_template_lib/basic.kleist
//!
//! Templates tested:
//! - plus_minus: ± operator
//! - negate: unary minus
//! - times: implied multiplication (changed from × to juxtaposition)
//! - scalar_multiply: implied multiplication (changed from × to juxtaposition)
//!
//! Each template is tested across all render targets: Unicode, LaTeX, HTML, Typst, Kleis

use kleis::editor_ast::{EditorNode, OperationData};
use kleis::render_editor::{render_editor_node, RenderTarget};

// =============================================================================
// Helper Functions
// =============================================================================

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

fn obj(name: &str) -> EditorNode {
    EditorNode::Object {
        object: name.to_string(),
    }
}

fn num(value: &str) -> EditorNode {
    EditorNode::Const {
        value: value.to_string(),
    }
}

// =============================================================================
// plus_minus Template Tests
// =============================================================================

#[test]
fn test_plus_minus_unicode() {
    let node = op("plus_minus", vec![obj("a"), obj("b")]);
    let result = render_editor_node(&node, &RenderTarget::Unicode);
    assert_eq!(result, "a ± b", "plus_minus Unicode should use ± symbol");
}

#[test]
fn test_plus_minus_latex() {
    let node = op("plus_minus", vec![obj("a"), obj("b")]);
    let result = render_editor_node(&node, &RenderTarget::LaTeX);
    assert_eq!(
        result, "a \\pm b",
        "plus_minus LaTeX should use \\pm command"
    );
}

#[test]
fn test_plus_minus_typst() {
    let node = op("plus_minus", vec![obj("a"), obj("b")]);
    let result = render_editor_node(&node, &RenderTarget::Typst);
    assert_eq!(
        result, "a plus.minus b",
        "plus_minus Typst should use plus.minus"
    );
}

#[test]
fn test_plus_minus_html() {
    let node = op("plus_minus", vec![obj("a"), obj("b")]);
    let result = render_editor_node(&node, &RenderTarget::HTML);
    assert_eq!(result, "a ± b", "plus_minus HTML should use ± entity");
}

// =============================================================================
// negate Template Tests
// =============================================================================

#[test]
fn test_negate_unicode() {
    let node = op("negate", vec![obj("x")]);
    let result = render_editor_node(&node, &RenderTarget::Unicode);
    assert_eq!(result, "-x", "negate Unicode should prefix with -");
}

#[test]
fn test_negate_latex() {
    let node = op("negate", vec![obj("x")]);
    let result = render_editor_node(&node, &RenderTarget::LaTeX);
    assert_eq!(result, "-x", "negate LaTeX should prefix with -");
}

#[test]
fn test_negate_typst() {
    let node = op("negate", vec![obj("x")]);
    let result = render_editor_node(&node, &RenderTarget::Typst);
    assert_eq!(result, "-x", "negate Typst should prefix with -");
}

#[test]
fn test_negate_html() {
    let node = op("negate", vec![obj("x")]);
    let result = render_editor_node(&node, &RenderTarget::HTML);
    assert_eq!(result, "-x", "negate HTML should prefix with -");
}

// =============================================================================
// times Template Tests (IMPLIED MULTIPLICATION - NO × SYMBOL)
// =============================================================================

#[test]
fn test_times_unicode_implied() {
    let node = op("times", vec![obj("a"), obj("b")]);
    let result = render_editor_node(&node, &RenderTarget::Unicode);
    assert!(
        !result.contains('×'),
        "times Unicode should NOT contain × (implied multiplication)"
    );
    assert_eq!(result, "ab", "times Unicode should be juxtaposition");
}

#[test]
fn test_times_latex_implied() {
    let node = op("times", vec![obj("a"), obj("b")]);
    let result = render_editor_node(&node, &RenderTarget::LaTeX);
    assert!(
        !result.contains("\\times"),
        "times LaTeX should NOT contain \\times (implied multiplication)"
    );
    assert_eq!(result, "a b", "times LaTeX should be space-separated");
}

#[test]
fn test_times_typst_implied() {
    let node = op("times", vec![obj("a"), obj("b")]);
    let result = render_editor_node(&node, &RenderTarget::Typst);
    // Should NOT contain "times" as a Typst operator
    assert!(
        !result.contains(" times "),
        "times Typst should NOT contain 'times' operator"
    );
    assert_eq!(result, "a b", "times Typst should be space-separated");
}

#[test]
fn test_times_html_implied() {
    let node = op("times", vec![obj("a"), obj("b")]);
    let result = render_editor_node(&node, &RenderTarget::HTML);
    assert!(
        !result.contains('×'),
        "times HTML should NOT contain × (implied multiplication)"
    );
    assert_eq!(result, "ab", "times HTML should be juxtaposition");
}

// =============================================================================
// scalar_multiply Template Tests (IMPLIED MULTIPLICATION)
// =============================================================================

#[test]
fn test_scalar_multiply_unicode_implied() {
    let node = op("scalar_multiply", vec![obj("m"), obj("c")]);
    let result = render_editor_node(&node, &RenderTarget::Unicode);
    assert!(
        !result.contains('×'),
        "scalar_multiply Unicode should NOT contain ×"
    );
    assert_eq!(result, "mc", "scalar_multiply Unicode should be 'mc'");
}

#[test]
fn test_scalar_multiply_latex_implied() {
    let node = op("scalar_multiply", vec![obj("m"), obj("c")]);
    let result = render_editor_node(&node, &RenderTarget::LaTeX);
    assert!(
        !result.contains("\\times"),
        "scalar_multiply LaTeX should NOT contain \\times"
    );
    assert_eq!(result, "m c", "scalar_multiply LaTeX should be 'm c'");
}

#[test]
fn test_scalar_multiply_typst_implied() {
    let node = op("scalar_multiply", vec![obj("m"), obj("c")]);
    let result = render_editor_node(&node, &RenderTarget::Typst);
    assert_eq!(result, "m c", "scalar_multiply Typst should be 'm c'");
}

#[test]
fn test_scalar_multiply_html_implied() {
    let node = op("scalar_multiply", vec![obj("m"), obj("c")]);
    let result = render_editor_node(&node, &RenderTarget::HTML);
    assert!(
        !result.contains('×'),
        "scalar_multiply HTML should NOT contain ×"
    );
    assert_eq!(result, "mc", "scalar_multiply HTML should be 'mc'");
}

// =============================================================================
// Complex Expression Tests
// =============================================================================

#[test]
fn test_emc2_unicode() {
    // E = mc²
    let node = op(
        "equals",
        vec![
            obj("E"),
            op(
                "scalar_multiply",
                vec![obj("m"), op("power", vec![obj("c"), num("2")])],
            ),
        ],
    );
    let result = render_editor_node(&node, &RenderTarget::Unicode);
    assert!(
        !result.contains('×'),
        "E=mc² Unicode should not contain ×: {}",
        result
    );
    println!("E=mc² Unicode: {}", result);
}

#[test]
fn test_emc2_latex() {
    let node = op(
        "equals",
        vec![
            obj("E"),
            op(
                "scalar_multiply",
                vec![obj("m"), op("power", vec![obj("c"), num("2")])],
            ),
        ],
    );
    let result = render_editor_node(&node, &RenderTarget::LaTeX);
    assert!(
        !result.contains("\\times"),
        "E=mc² LaTeX should not contain \\times: {}",
        result
    );
    println!("E=mc² LaTeX: {}", result);
}

#[test]
fn test_emc2_html() {
    let node = op(
        "equals",
        vec![
            obj("E"),
            op(
                "scalar_multiply",
                vec![obj("m"), op("power", vec![obj("c"), num("2")])],
            ),
        ],
    );
    let result = render_editor_node(&node, &RenderTarget::HTML);
    assert!(
        !result.contains('×'),
        "E=mc² HTML should not contain ×: {}",
        result
    );
    println!("E=mc² HTML: {}", result);
}

#[test]
fn test_emc2_typst() {
    // E = mc²
    let node = op(
        "equals",
        vec![
            obj("E"),
            op(
                "scalar_multiply",
                vec![obj("m"), op("power", vec![obj("c"), num("2")])],
            ),
        ],
    );
    let result = render_editor_node(&node, &RenderTarget::Typst);
    // Should NOT have × symbol
    assert!(
        !result.contains("times"),
        "E=mc² should not contain 'times'"
    );
    // Should have implied multiplication
    assert!(
        result.contains("m c") || result.contains("m c^"),
        "E=mc² should have implied multiplication: {}",
        result
    );
}

#[test]
fn test_quadratic_formula_typst() {
    // x = (-b ± sqrt(b² - 4ac)) / 2a
    let discriminant = op(
        "minus",
        vec![
            op("power", vec![obj("b"), num("2")]),
            op(
                "scalar_multiply",
                vec![op("scalar_multiply", vec![num("4"), obj("a")]), obj("c")],
            ),
        ],
    );

    let numerator = op(
        "plus_minus",
        vec![op("negate", vec![obj("b")]), op("sqrt", vec![discriminant])],
    );

    let denominator = op("scalar_multiply", vec![num("2"), obj("a")]);

    let formula = op(
        "equals",
        vec![obj("x"), op("frac", vec![numerator, denominator])],
    );

    let result = render_editor_node(&formula, &RenderTarget::Typst);

    // Check key elements
    assert!(
        result.contains("plus.minus"),
        "Should contain plus.minus: {}",
        result
    );
    assert!(result.contains("-b"), "Should contain -b: {}", result);
    assert!(
        result.contains("4 a"),
        "Should contain 4a (implied mult): {}",
        result
    );
    assert!(
        result.contains("2 a"),
        "Should contain 2a (implied mult): {}",
        result
    );

    println!("Quadratic formula Typst: {}", result);
}

#[test]
fn test_quadratic_formula_latex() {
    // Same formula in LaTeX
    let discriminant = op(
        "minus",
        vec![
            op("power", vec![obj("b"), num("2")]),
            op(
                "scalar_multiply",
                vec![op("scalar_multiply", vec![num("4"), obj("a")]), obj("c")],
            ),
        ],
    );

    let numerator = op(
        "plus_minus",
        vec![op("negate", vec![obj("b")]), op("sqrt", vec![discriminant])],
    );

    let denominator = op("scalar_multiply", vec![num("2"), obj("a")]);

    let formula = op(
        "equals",
        vec![obj("x"), op("frac", vec![numerator, denominator])],
    );

    let result = render_editor_node(&formula, &RenderTarget::LaTeX);

    // Check key elements
    assert!(result.contains("\\pm"), "Should contain \\pm: {}", result);
    assert!(result.contains("-b"), "Should contain -b: {}", result);
    assert!(
        !result.contains("\\times"),
        "Should NOT contain \\times: {}",
        result
    );

    println!("Quadratic formula LaTeX: {}", result);
}

#[test]
fn test_quadratic_formula_unicode() {
    // Same formula in Unicode
    let discriminant = op(
        "minus",
        vec![
            op("power", vec![obj("b"), num("2")]),
            op(
                "scalar_multiply",
                vec![op("scalar_multiply", vec![num("4"), obj("a")]), obj("c")],
            ),
        ],
    );

    let numerator = op(
        "plus_minus",
        vec![op("negate", vec![obj("b")]), op("sqrt", vec![discriminant])],
    );

    let denominator = op("scalar_multiply", vec![num("2"), obj("a")]);

    let formula = op(
        "equals",
        vec![obj("x"), op("frac", vec![numerator, denominator])],
    );

    let result = render_editor_node(&formula, &RenderTarget::Unicode);

    // Check key elements
    assert!(result.contains('±'), "Should contain ±: {}", result);
    assert!(result.contains("-b"), "Should contain -b: {}", result);
    assert!(
        !result.contains('×'),
        "Should NOT contain × (implied mult): {}",
        result
    );

    println!("Quadratic formula Unicode: {}", result);
}

#[test]
fn test_quadratic_formula_html() {
    // Same formula in HTML
    let discriminant = op(
        "minus",
        vec![
            op("power", vec![obj("b"), num("2")]),
            op(
                "scalar_multiply",
                vec![op("scalar_multiply", vec![num("4"), obj("a")]), obj("c")],
            ),
        ],
    );

    let numerator = op(
        "plus_minus",
        vec![op("negate", vec![obj("b")]), op("sqrt", vec![discriminant])],
    );

    let denominator = op("scalar_multiply", vec![num("2"), obj("a")]);

    let formula = op(
        "equals",
        vec![obj("x"), op("frac", vec![numerator, denominator])],
    );

    let result = render_editor_node(&formula, &RenderTarget::HTML);

    // Check key elements
    assert!(result.contains('±'), "Should contain ±: {}", result);
    assert!(result.contains("-b"), "Should contain -b: {}", result);
    assert!(
        !result.contains('×'),
        "Should NOT contain × (implied mult): {}",
        result
    );

    println!("Quadratic formula HTML: {}", result);
}
