//! Level 3 Tests: Render → Parse → Z3 Verify
//!
//! These tests complete the testing ladder:
//! - Level 1: Render → Assert symbols ✅
//! - Level 2: Render → Parse → Compare ✅
//! - Level 3: Render → Parse → Z3 Verify ← THIS FILE
//!
//! Purpose: Discover what Z3 can actually verify from rendered Kleis syntax.
//!
//! Note: Full Z3 verification requires loading stdlib/calculus.kleis axioms.
//! These tests focus on the pipeline working correctly.

use kleis::ast::Expression;
use kleis::kleis_parser::parse_kleis;
use kleis::render::{build_default_context, render_expression, RenderTarget};
use kleis::solvers::backend::SolverBackend;
use kleis::solvers::z3::backend::Z3Backend;
use kleis::structure_registry::StructureRegistry;

// ============================================================
// HELPER FUNCTIONS
// ============================================================

fn o(name: &str) -> Expression {
    Expression::Object(name.to_string())
}

fn c(val: &str) -> Expression {
    Expression::Const(val.to_string())
}

fn op(name: &str, args: Vec<Expression>) -> Expression {
    Expression::Operation {
        name: name.to_string(),
        args,
        span: None,
    }
}

fn render_to_kleis(expr: &Expression) -> String {
    let ctx = build_default_context();
    render_expression(expr, &ctx, &RenderTarget::Kleis)
}

fn parse_kleis_expr(s: &str) -> Result<Expression, String> {
    parse_kleis(s).map_err(|e| format!("{:?}", e))
}

/// Full pipeline: Render → Parse → Z3 Translation check
/// Returns: (rendered, parsed, z3_translates_ok)
fn verify_pipeline(expr: &Expression) -> (String, Result<Expression, String>, bool) {
    let rendered = render_to_kleis(expr);
    let parsed = parse_kleis_expr(&rendered);

    let z3_ok = if let Ok(ref parsed_expr) = parsed {
        // Check if Z3 can translate the expression
        let registry = StructureRegistry::default();
        if let Ok(mut backend) = Z3Backend::new(&registry) {
            // Try to verify - if it doesn't crash, Z3 can handle it
            backend.verify_axiom(parsed_expr).is_ok()
        } else {
            false
        }
    } else {
        false
    };

    (rendered, parsed, z3_ok)
}

// ============================================================
// LEVEL 3: ARITHMETIC (Z3 should handle these)
// ============================================================

#[test]
fn z3_verify_simple_addition() {
    // 2 + 3 = 5
    let two_plus_three = op("plus", vec![c("2"), c("3")]);
    let five = c("5");
    let eq = op("equals", vec![two_plus_three, five]);

    let (rendered, parsed, z3_ok) = verify_pipeline(&eq);
    println!("Rendered: {}", rendered);
    println!("Parsed: {:?}", parsed);
    println!("Z3 translates: {}", z3_ok);

    assert!(parsed.is_ok(), "Should parse arithmetic");
}

#[test]
fn z3_verify_simple_multiplication() {
    // 4 * 5 = 20
    let four_times_five = op("scalar_multiply", vec![c("4"), c("5")]);
    let twenty = c("20");
    let eq = op("equals", vec![four_times_five, twenty]);

    let (rendered, parsed, z3_ok) = verify_pipeline(&eq);
    println!("Rendered: {}", rendered);
    println!("Parsed: {:?}", parsed);
    println!("Z3 translates: {}", z3_ok);

    assert!(parsed.is_ok(), "Should parse multiplication");
}

#[test]
fn z3_verify_commutativity() {
    // a + b = b + a
    let a_plus_b = op("plus", vec![o("a"), o("b")]);
    let b_plus_a = op("plus", vec![o("b"), o("a")]);
    let eq = op("equals", vec![a_plus_b, b_plus_a]);

    let (rendered, parsed, z3_ok) = verify_pipeline(&eq);
    println!("Rendered: {}", rendered);
    println!("Parsed: {:?}", parsed);
    println!("Z3 translates: {}", z3_ok);

    assert!(parsed.is_ok(), "Should parse equality");
}

// ============================================================
// LEVEL 3: COMPARISON (Z3 should handle these)
// ============================================================

#[test]
fn z3_verify_less_than() {
    // 3 < 5 should be satisfiable
    let three_lt_five = op("less_than", vec![c("3"), c("5")]);

    let (rendered, parsed, z3_ok) = verify_pipeline(&three_lt_five);
    println!("Rendered: {}", rendered);
    println!("Parsed: {:?}", parsed);
    println!("Z3 translates: {}", z3_ok);

    assert!(parsed.is_ok(), "Should parse less_than");
}

#[test]
fn z3_verify_implication() {
    // P ⟹ P should be valid (tautology)
    let p = o("P");
    let p_implies_p = op("implies", vec![p.clone(), p]);

    let (rendered, parsed, z3_ok) = verify_pipeline(&p_implies_p);
    println!("Rendered: {}", rendered);
    println!("Parsed: {:?}", parsed);
    println!("Z3 translates: {}", z3_ok);

    assert!(parsed.is_ok(), "Should parse implication");
}

// ============================================================
// LEVEL 3: CALCULUS (Z3 needs axioms from stdlib/calculus.kleis)
// ============================================================

#[test]
fn z3_verify_derivative_function_call() {
    // D(f, x) - should parse as function call
    let d_f_x = op("d_part", vec![o("f"), o("x")]);

    let (rendered, parsed, z3_ok) = verify_pipeline(&d_f_x);
    println!("Rendered: {}", rendered);
    println!("Parsed: {:?}", parsed);
    println!("Z3 translates: {}", z3_ok);

    assert_eq!(rendered, "D(f, x)", "Should render as Mathematica-style");
    // Z3 will treat D as uninterpreted function without calculus.kleis axioms
}

#[test]
fn z3_verify_total_derivative() {
    // Dt(y, t) - total derivative
    let dt_y_t = op("d_dt", vec![o("y"), o("t")]);

    let (rendered, parsed, z3_ok) = verify_pipeline(&dt_y_t);
    println!("Rendered: {}", rendered);
    println!("Parsed: {:?}", parsed);
    println!("Z3 translates: {}", z3_ok);

    assert_eq!(rendered, "Dt(y, t)", "Should render as Mathematica-style");
}

#[test]
fn z3_verify_limit() {
    // Limit(f, x, 0)
    let limit = op("lim", vec![o("f"), o("x"), c("0")]);

    let (rendered, parsed, z3_ok) = verify_pipeline(&limit);
    println!("Rendered: {}", rendered);
    println!("Parsed: {:?}", parsed);
    println!("Z3 translates: {}", z3_ok);

    assert!(
        rendered.contains("Limit("),
        "Should render as Limit function call"
    );
}

// ============================================================
// LEVEL 3: GRADIENT (Z3 should handle ∇ as prefix)
// ============================================================

#[test]
fn z3_verify_gradient() {
    // ∇f
    let grad_f = op("gradient", vec![o("f")]);

    let (rendered, parsed, z3_ok) = verify_pipeline(&grad_f);
    println!("Rendered: {}", rendered);
    println!("Parsed: {:?}", parsed);
    println!("Z3 translates: {}", z3_ok);

    assert!(
        parsed.is_ok(),
        "Gradient should parse (∇ is prefix operator)"
    );
}

// ============================================================
// LEVEL 3: CONDITIONAL (Z3 should handle if-then-else)
// ============================================================

#[test]
fn z3_verify_conditional() {
    // if x > 0 then x else -x (absolute value)
    let x = o("x");
    let zero = c("0");
    let x_gt_0 = op("greater_than", vec![x.clone(), zero]);
    let neg_x = op("negate", vec![x.clone()]);

    let cond = Expression::Conditional {
        condition: Box::new(x_gt_0),
        then_branch: Box::new(x.clone()),
        else_branch: Box::new(neg_x),
    };

    let (rendered, parsed, z3_ok) = verify_pipeline(&cond);
    println!("Rendered: {}", rendered);
    println!("Parsed: {:?}", parsed);
    println!("Z3 translates: {}", z3_ok);

    assert!(parsed.is_ok(), "Should parse conditional");
}

// ============================================================
// LEVEL 3: LET BINDING (Z3 should handle let-in)
// ============================================================

#[test]
fn z3_verify_let_binding() {
    // let x = 5 in x + x
    let five = c("5");
    let x = o("x");
    let x_plus_x = op("plus", vec![x.clone(), x.clone()]);

    let let_expr = Expression::Let {
        pattern: kleis::ast::Pattern::Variable("x".to_string()),
        type_annotation: None,
        value: Box::new(five),
        body: Box::new(x_plus_x),
        span: None,
    };

    let (rendered, parsed, z3_ok) = verify_pipeline(&let_expr);
    println!("Rendered: {}", rendered);
    println!("Parsed: {:?}", parsed);
    println!("Z3 translates: {}", z3_ok);

    assert!(parsed.is_ok(), "Should parse let binding");
}

// ============================================================
// SUMMARY TEST: Print all gaps discovered
// ============================================================

#[test]
fn z3_summary_report() {
    println!("\n=== Level 3 Test Summary ===\n");

    let test_cases = vec![
        (
            "Addition 2+3=5",
            op("equals", vec![op("plus", vec![c("2"), c("3")]), c("5")]),
        ),
        (
            "Multiplication",
            op(
                "equals",
                vec![op("scalar_multiply", vec![c("4"), c("5")]), c("20")],
            ),
        ),
        ("Less than", op("less_than", vec![c("3"), c("5")])),
        ("Implication", op("implies", vec![o("P"), o("P")])),
        ("Partial derivative", op("d_part", vec![o("f"), o("x")])),
        ("Total derivative", op("d_dt", vec![o("y"), o("t")])),
        ("Limit", op("lim", vec![o("f"), o("x"), c("0")])),
        ("Gradient", op("gradient", vec![o("f")])),
    ];

    let mut parse_pass = 0;
    let mut z3_pass = 0;
    let total = test_cases.len();

    for (name, expr) in test_cases {
        let (rendered, parsed, z3_ok) = verify_pipeline(&expr);
        let parse_ok = parsed.is_ok();

        if parse_ok {
            parse_pass += 1;
        }
        if z3_ok {
            z3_pass += 1;
        }

        let parse_status = if parse_ok { "✅" } else { "❌" };
        let z3_status = if z3_ok { "✅" } else { "⚠️" };

        println!(
            "{}: {} → Parse {} | Z3 {}",
            name, rendered, parse_status, z3_status
        );
    }

    println!("\n--- Results ---");
    println!("Parse: {}/{} passed", parse_pass, total);
    println!(
        "Z3: {}/{} translated (others need axioms or more work)",
        z3_pass, total
    );
    println!("\nTo enable Z3 calculus verification:");
    println!("  Load stdlib/calculus.kleis axioms into Z3 context");
}
