//! Tests for Grammar v0.9 features:
//! 1. Quantifiers as expression operands (∀/∃ inside ∧/∨/→)
//! 2. Function types in type annotations (ℝ → ℝ)

use kleis::kleis_parser::parse_kleis_program;

/// Helper to check if a program parses successfully
fn parses_ok(source: &str) -> bool {
    parse_kleis_program(source).is_ok()
}

/// Helper to get parse error message (for debugging)
#[allow(dead_code)]
fn parse_error(source: &str) -> String {
    match parse_kleis_program(source) {
        Ok(_) => "No error".to_string(),
        Err(e) => e.message,
    }
}

// =============================================================================
// Test 1: Quantifiers as expression operands
// =============================================================================

#[test]
fn test_quantifier_in_conjunction() {
    let source = r#"
structure Test {
    axiom nested: (x > 0) ∧ (∀(y : ℝ). y > 0)
}
"#;
    assert!(parses_ok(source), "Should parse quantifier inside ∧");
}

#[test]
fn test_quantifier_in_disjunction() {
    let source = r#"
structure Test {
    axiom or_quant: (x = 0) ∨ (∃(y : ℝ). y > x)
}
"#;
    assert!(parses_ok(source), "Should parse quantifier inside ∨");
}

#[test]
fn test_quantifier_in_implication_rhs() {
    let source = r#"
structure Test {
    axiom impl_quant: (x > 0) → (∀(y : ℝ). x + y > y)
}
"#;
    assert!(parses_ok(source), "Should parse quantifier as RHS of →");
}

#[test]
fn test_nested_quantifiers_in_expression() {
    let source = r#"
structure Test {
    axiom deeply_nested: (a > 0) ∧ (∀(x : ℝ). x > 0 → (∃(y : ℝ). y > x))
}
"#;
    assert!(parses_ok(source), "Should parse deeply nested quantifiers");
}

#[test]
fn test_epsilon_delta_limit() {
    let source = r#"
structure Limits {
    axiom epsilon_delta: ∀(L a : ℝ, ε : ℝ). ε > 0 → 
        (∃(δ : ℝ). δ > 0 ∧ (∀(x : ℝ). abs(x - a) < δ → abs(f(x) - L) < ε))
}
"#;
    assert!(
        parses_ok(source),
        "Should parse full epsilon-delta definition"
    );
}

#[test]
fn test_forall_keyword_in_expression() {
    let source = r#"
structure Test {
    axiom keyword: (x > 0) ∧ (forall(y : ℝ). y > 0)
}
"#;
    assert!(parses_ok(source), "Should parse 'forall' keyword inside ∧");
}

#[test]
fn test_exists_keyword_in_expression() {
    let source = r#"
structure Test {
    axiom keyword: (x > 0) ∧ (exists(y : ℝ). y > 0)
}
"#;
    assert!(parses_ok(source), "Should parse 'exists' keyword inside ∧");
}

// =============================================================================
// Test 2: Function types in type annotations
// =============================================================================

#[test]
fn test_simple_function_type() {
    let source = r#"
structure Test {
    axiom func: ∀(f : ℝ → ℝ). f(0) = f(0)
}
"#;
    assert!(parses_ok(source), "Should parse simple function type ℝ → ℝ");
}

#[test]
fn test_function_type_ascii_arrow() {
    let source = r#"
structure Test {
    axiom func: ∀(f : ℝ -> ℝ). f(0) = f(0)
}
"#;
    assert!(parses_ok(source), "Should parse ASCII arrow ->");
}

#[test]
fn test_curried_function_type() {
    // Note: The type ℝ → ℝ → ℝ parses, but curried application f(1)(2) is not yet supported.
    // This tests that the function type itself parses correctly.
    let source = r#"
structure Test {
    axiom curried: ∀(f : ℝ → ℝ → ℝ). f = f
}
"#;
    assert!(
        parses_ok(source),
        "Should parse curried function type ℝ → ℝ → ℝ"
    );
}

#[test]
fn test_parametric_function_type() {
    let source = r#"
structure Test {
    axiom param: ∀(f : Set(ℝ) → Set(ℝ)). f(S) = f(S)
}
"#;
    assert!(parses_ok(source), "Should parse parametric function type");
}

#[test]
fn test_topology_continuity() {
    let source = r#"
structure Topology {
    axiom continuity: ∀(f : X → Y, V : Set(Y)). 
        is_open(V) → is_open(preimage(f, V))
}
"#;
    assert!(parses_ok(source), "Should parse topology continuity axiom");
}

#[test]
fn test_multiple_function_type_vars() {
    let source = r#"
structure Test {
    axiom compose: ∀(f : ℝ → ℝ, g : ℝ → ℝ). compose(f, g) = compose(f, g)
}
"#;
    assert!(
        parses_ok(source),
        "Should parse multiple function type variables"
    );
}

// =============================================================================
// Test 3: Combined features
// =============================================================================

#[test]
fn test_function_type_with_nested_quantifier() {
    let source = r#"
structure Analysis {
    axiom bounded: ∀(f : ℝ → ℝ). (∃(M : ℝ). ∀(x : ℝ). abs(f(x)) < M)
}
"#;
    assert!(
        parses_ok(source),
        "Should parse function type with nested quantifiers"
    );
}

#[test]
fn test_metric_space_continuity() {
    let source = r#"
structure MetricContinuity {
    axiom continuous_at: ∀(f : X → Y, a : X, ε : ℝ). ε > 0 →
        (∃(δ : ℝ). δ > 0 ∧ (∀(x : X). d(x, a) < δ → d(f(x), f(a)) < ε))
}
"#;
    assert!(
        parses_ok(source),
        "Should parse metric space continuity definition"
    );
}

#[test]
fn test_homeomorphism() {
    let source = r#"
structure Homeomorphism {
    axiom def: ∀(f : X → Y, g : Y → X). 
        (∀(x : X). g(f(x)) = x) ∧ (∀(y : Y). f(g(y)) = y) → bijective(f)
}
"#;
    assert!(parses_ok(source), "Should parse homeomorphism definition");
}

// =============================================================================
// Regression tests - ensure existing syntax still works
// =============================================================================

#[test]
fn test_simple_quantifier_still_works() {
    let source = r#"
structure Test {
    axiom simple: ∀(x : ℝ). x = x
}
"#;
    assert!(parses_ok(source), "Simple quantifier should still work");
}

#[test]
fn test_where_clause_still_works() {
    let source = r#"
structure Test {
    axiom with_where: ∀(x : ℝ) where x > 0. x = x
}
"#;
    assert!(parses_ok(source), "Where clause should still work");
}

#[test]
fn test_parametric_type_still_works() {
    let source = r#"
structure Test {
    axiom vec: ∀(v : Vector(3, ℝ)). v = v
}
"#;
    assert!(parses_ok(source), "Parametric types should still work");
}

#[test]
fn test_multiple_var_groups_still_works() {
    let source = r#"
structure Test {
    axiom multi: ∀(x y : ℝ, n : ℕ). x + y = y + x
}
"#;
    assert!(
        parses_ok(source),
        "Multiple variable groups should still work"
    );
}
