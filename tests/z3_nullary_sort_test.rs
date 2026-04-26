//! Tests for Z3 nullary operation sort correctness
//!
//! Verifies the fix for the bug where file-scope nullary operations like
//! `operation foo : ℝ` were incorrectly assigned Z3 Int sort instead of Real,
//! causing AXIOM INCONSISTENCY when equated with non-integer real literals.

use kleis::evaluator::Evaluator;
use kleis::kleis_ast::TopLevel;
use kleis::kleis_parser::parse_kleis_program_with_file;
use std::path::PathBuf;

fn run_examples(source: &str, filename: &str) -> Vec<(String, bool, Option<String>)> {
    let program = parse_kleis_program_with_file(source, filename).expect("Should parse");

    let mut evaluator = Evaluator::new();
    evaluator
        .load_program_with_file(&program, Some(PathBuf::from(filename)))
        .expect("Should load");

    let mut results = Vec::new();
    for item in &program.items {
        if let TopLevel::ExampleBlock(example) = item {
            let result = evaluator.eval_example_block(example);
            results.push((example.name.clone(), result.passed, result.error));
        }
    }
    results
}

/// The original bug: `operation foo : ℝ` with `foo = 0.6602` gave AXIOM INCONSISTENCY
#[test]
fn test_real_noninteger_equality() {
    let source = r#"
operation c : ℝ

structure S {
    axiom val : c = 0.6602
}

example "real = 0.6602" {
    assert(c = 0.6602)
}
"#;

    let results = run_examples(source, "test_real_eq.kleis");
    assert_eq!(results.len(), 1);
    assert!(
        results[0].1,
        "ℝ operation with non-integer equality should pass: {:?}",
        results[0].2
    );
}

/// ℝ operation with negative non-integer literal
#[test]
fn test_real_negative_noninteger_equality() {
    let source = r#"
operation pi_neg : ℝ

structure S {
    axiom val : pi_neg = -3.14159
}

example "real = -3.14159" {
    assert(pi_neg = -3.14159)
}
"#;

    let results = run_examples(source, "test_real_neg.kleis");
    assert_eq!(results.len(), 1);
    assert!(
        results[0].1,
        "ℝ operation with negative non-integer should pass: {:?}",
        results[0].2
    );
}

/// ℝ operation with integer-valued literal (worked before fix too)
#[test]
fn test_real_integer_equality() {
    let source = r#"
operation r : ℝ

structure S {
    axiom val : r = 2.0
}

example "real = 2.0" {
    assert(r = 2.0)
}
"#;

    let results = run_examples(source, "test_real_int.kleis");
    assert_eq!(results.len(), 1);
    assert!(
        results[0].1,
        "ℝ operation with integer-valued literal should pass: {:?}",
        results[0].2
    );
}

/// ℤ operation with integer equality (must still work)
#[test]
fn test_int_equality() {
    let source = r#"
operation n : ℤ

structure S {
    axiom val : n = 42
}

example "int = 42" {
    assert(n = 42)
}
"#;

    let results = run_examples(source, "test_int_eq.kleis");
    assert_eq!(results.len(), 1);
    assert!(
        results[0].1,
        "ℤ operation with integer equality should pass: {:?}",
        results[0].2
    );
}

/// Bool operation
#[test]
fn test_bool_operation() {
    let source = r#"
operation flag : Bool

structure S {
    axiom flag_holds : flag
}

example "bool flag" {
    assert(flag)
}
"#;

    let results = run_examples(source, "test_bool.kleis");
    assert_eq!(results.len(), 1);
    assert!(
        results[0].1,
        "Bool operation axiom should pass: {:?}",
        results[0].2
    );
}

/// ℝ arithmetic: a + b = c where all are nullary ℝ operations
#[test]
fn test_real_arithmetic() {
    let source = r#"
operation a : ℝ
operation b : ℝ
operation c : ℝ

structure S {
    axiom a_val : a = 1.5
    axiom b_val : b = 2.3
    axiom sum   : c = a + b
}

example "sum" {
    assert(c = 3.8)
}
"#;

    let results = run_examples(source, "test_real_arith.kleis");
    assert_eq!(results.len(), 1);
    assert!(results[0].1, "ℝ arithmetic should pass: {:?}", results[0].2);
}

/// Mixed ℝ and ℤ operations in the same context
#[test]
fn test_mixed_real_int() {
    let source = r#"
operation r : ℝ
operation n : ℤ

structure S {
    axiom r_val : r = 0.6602
    axiom n_val : n = 42
}

example "mixed" {
    assert(r < 1.0)
    assert(n > 40)
}
"#;

    let results = run_examples(source, "test_mixed.kleis");
    assert_eq!(results.len(), 1);
    assert!(
        results[0].1,
        "Mixed ℝ/ℤ sorts should pass: {:?}",
        results[0].2
    );
}

/// Structure-scope element declarations (always worked via identity_elements path)
#[test]
fn test_structure_scope_element() {
    let source = r#"
structure S {
    element x : ℝ
    element y : ℝ

    axiom x_val : x = 0.123
    axiom y_val : y = 0.456
}

example "element sum" {
    assert(x + y = 0.579)
}
"#;

    let results = run_examples(source, "test_elem.kleis");
    assert_eq!(results.len(), 1);
    assert!(
        results[0].1,
        "Structure-scope element ℝ should pass: {:?}",
        results[0].2
    );
}

/// Transitive chain: a = 1.5, c = a  →  c = 1.5
#[test]
fn test_real_transitive() {
    let source = r#"
operation a : ℝ
operation c : ℝ

structure S {
    axiom a_val  : a = 1.5
    axiom c_is_a : c = a
}

example "transitive" {
    assert(c = 1.5)
}
"#;

    let results = run_examples(source, "test_trans.kleis");
    assert_eq!(results.len(), 1);
    assert!(
        results[0].1,
        "Transitive ℝ equality should pass: {:?}",
        results[0].2
    );
}

// =============================================================================
// Scientific Notation Z3 Tests
// =============================================================================

/// Scientific notation constant can be assigned to ℝ operation and verified
#[test]
fn test_z3_scientific_notation() {
    let source = r#"
operation G : ℝ

structure S {
    axiom val : G = 6.674e-11
}

example "scientific notation constant" {
    assert(G = 6.674e-11)
}
"#;

    let results = run_examples(source, "test_sci.kleis");
    assert_eq!(results.len(), 1);
    assert!(
        results[0].1,
        "Scientific notation constant should verify: {:?}",
        results[0].2
    );
}

/// Z3 can verify that a scientific notation value is positive
#[test]
fn test_z3_scientific_comparison() {
    let source = r#"
operation G : ℝ

structure S {
    axiom val : G = 6.674e-11
}

example "scientific > 0" {
    assert(G > 0)
}
"#;

    let results = run_examples(source, "test_sci_cmp.kleis");
    assert_eq!(results.len(), 1);
    assert!(
        results[0].1,
        "Scientific notation comparison should verify: {:?}",
        results[0].2
    );
}

/// Z3 can verify arithmetic equivalence: 1e3 = 1000
#[test]
fn test_z3_scientific_arithmetic() {
    let source = r#"
operation c : ℝ

structure S {
    axiom val : c = 1e3
}

example "1e3 equals 1000" {
    assert(c = 1000)
}
"#;

    let results = run_examples(source, "test_sci_arith.kleis");
    assert_eq!(results.len(), 1);
    assert!(results[0].1, "1e3 should equal 1000: {:?}", results[0].2);
}

/// Scientific notation with positive exponent sign: 2.5e+2 = 250
#[test]
fn test_z3_scientific_positive_exp() {
    let source = r#"
operation v : ℝ

structure S {
    axiom val : v = 2.5e+2
}

example "2.5e+2 equals 250" {
    assert(v = 250)
}
"#;

    let results = run_examples(source, "test_sci_pos.kleis");
    assert_eq!(results.len(), 1);
    assert!(results[0].1, "2.5e+2 should equal 250: {:?}", results[0].2);
}
