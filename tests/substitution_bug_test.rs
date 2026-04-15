//! Tests for the evaluator substitution bug:
//! Compound fst/snd arithmetic inside list_map lambdas gives wrong results.
//!
//! Bug: `fst(ga)*snd(gb) - snd(ga)*fst(gb)` evaluates incorrectly when
//! ga/gb come from list_nth inside a list_map lambda. Decomposing into
//! separate let bindings per product works correctly.

use kleis::evaluator::Evaluator;
use kleis::kleis_parser::KleisParser;
use kleis::pretty_print::PrettyPrinter;

fn eval(evaluator: &Evaluator, input: &str) -> Result<String, String> {
    let mut parser = KleisParser::new(input);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    let result = evaluator.eval_concrete(&expr)?;
    let pp = PrettyPrinter::new();
    Ok(pp.format_expression(&result))
}

// =============================================================================
// Layer 1: Reproduce the bug via string evaluation
// =============================================================================

#[test]
fn test_1a_cross_product_outside_list_map() {
    // fst/snd cross-product with let-bound Pairs, NO list_map.
    // This should work correctly (confirmed empirically).
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let p1 = Pair(2, 3) in
        let p2 = Pair(4, 5) in
        fst(p1)*snd(p2) - snd(p1)*fst(p2)
        "#,
    )
    .unwrap();
    assert_eq!(result, "-2", "Cross product outside list_map should be -2");
}

#[test]
fn test_1b_cross_product_inside_list_map() {
    // The EXACT failing pattern: list_map + list_nth + compound fst/snd arithmetic.
    // ga = Pair(1,0), gb = Pair(0,1) → cross = 1*1 - 0*0 = 1
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let gs = [Pair(1, 0), Pair(0, 1)] in
        list_map(λ j .
            let ga = list_nth(gs, j) in
            let gb = list_nth(gs, j + 1) in
            fst(ga)*snd(gb) - snd(ga)*fst(gb)
        , [0])
        "#,
    )
    .unwrap();
    // Expected: [1] (cross product of (1,0) and (0,1) = 1*1 - 0*0 = 1)
    assert_eq!(
        result, "[1]",
        "BUG REPRO: cross product inside list_map should be [1]"
    );
}

#[test]
fn test_1c_decomposed_cross_product_inside_list_map() {
    // The WORKAROUND: decompose into separate let bindings.
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let gs = [Pair(1, 0), Pair(0, 1)] in
        list_map(λ j .
            let ga = list_nth(gs, j) in
            let gb = list_nth(gs, j + 1) in
            let p1 = fst(ga)*snd(gb) in
            let p2 = snd(ga)*fst(gb) in
            p1 - p2
        , [0])
        "#,
    )
    .unwrap();
    assert_eq!(result, "[1]", "Decomposed cross product should be [1]");
}

#[test]
fn test_1d_multiple_elements_cross_product() {
    // Test with 3 elements to see pattern across iterations.
    // gs = [(2,3), (4,5), (6,7)]
    // j=0: cross(g0,g1) = 2*5 - 3*4 = -2
    // j=1: cross(g1,g2) = 4*7 - 5*6 = -2
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let gs = [Pair(2, 3), Pair(4, 5), Pair(6, 7)] in
        list_map(λ j .
            let ga = list_nth(gs, j) in
            let gb = list_nth(gs, j + 1) in
            fst(ga)*snd(gb) - snd(ga)*fst(gb)
        , [0, 1])
        "#,
    )
    .unwrap();
    assert_eq!(
        result, "[-2, -2]",
        "Multiple cross products should all be -2"
    );
}

// =============================================================================
// Layer 2: Narrow the trigger conditions
// =============================================================================

#[test]
fn test_2a_single_product_inside_list_map() {
    // Just ONE product: fst(ga)*snd(gb). No subtraction.
    // ga = Pair(1,0), gb = Pair(0,1) → fst(ga)*snd(gb) = 1*1 = 1
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let gs = [Pair(1, 0), Pair(0, 1)] in
        list_map(λ j .
            let ga = list_nth(gs, j) in
            let gb = list_nth(gs, j + 1) in
            fst(ga)*snd(gb)
        , [0])
        "#,
    )
    .unwrap();
    assert_eq!(result, "[1]", "Single product fst(ga)*snd(gb) should be 1");
}

#[test]
fn test_2b_two_products_subtracted() {
    // Two products with subtraction, using specific values where the bug is visible.
    // ga = Pair(2,3), gb = Pair(4,5)
    // fst(ga)*snd(gb) - snd(ga)*fst(gb) = 2*5 - 3*4 = 10 - 12 = -2
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let gs = [Pair(2, 3), Pair(4, 5)] in
        list_map(λ j .
            let ga = list_nth(gs, j) in
            let gb = list_nth(gs, j + 1) in
            fst(ga)*snd(gb) - snd(ga)*fst(gb)
        , [0])
        "#,
    )
    .unwrap();
    assert_eq!(result, "[-2]", "Cross product (2,3)x(4,5) should be -2");
}

#[test]
fn test_2c_inline_list_nth_no_let() {
    // No intermediate let bindings — inline list_nth directly in the expression.
    // fst(list_nth(gs, j)) * snd(list_nth(gs, j+1)) - snd(list_nth(gs, j)) * fst(list_nth(gs, j+1))
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let gs = [Pair(1, 0), Pair(0, 1)] in
        list_map(λ j .
            fst(list_nth(gs, j))*snd(list_nth(gs, j + 1)) - snd(list_nth(gs, j))*fst(list_nth(gs, j + 1))
        , [0])
        "#,
    )
    .unwrap();
    assert_eq!(result, "[1]", "Inline list_nth cross product should be 1");
}

#[test]
fn test_2d_dot_product_inside_list_map() {
    // Dot product: fst(ga)*fst(gb) + snd(ga)*snd(gb)
    // ga = Pair(1,0), gb = Pair(0,1) → 1*0 + 0*1 = 0
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let gs = [Pair(1, 0), Pair(0, 1)] in
        list_map(λ j .
            let ga = list_nth(gs, j) in
            let gb = list_nth(gs, j + 1) in
            fst(ga)*fst(gb) + snd(ga)*snd(gb)
        , [0])
        "#,
    )
    .unwrap();
    assert_eq!(result, "[0]", "Dot product (1,0)·(0,1) should be 0");
}

#[test]
fn test_2e_addition_instead_of_subtraction() {
    // fst(ga)*snd(gb) + snd(ga)*fst(gb) (addition, not subtraction)
    // ga = Pair(2,3), gb = Pair(4,5) → 2*5 + 3*4 = 10 + 12 = 22
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let gs = [Pair(2, 3), Pair(4, 5)] in
        list_map(λ j .
            let ga = list_nth(gs, j) in
            let gb = list_nth(gs, j + 1) in
            fst(ga)*snd(gb) + snd(ga)*fst(gb)
        , [0])
        "#,
    )
    .unwrap();
    assert_eq!(result, "[22]", "Sum of products should be 22");
}

// =============================================================================
// Layer 3: AST-level inspection
// =============================================================================

#[test]
fn test_3a_individual_fst_snd_in_list_map() {
    // Extract individual fst/snd values inside list_map to verify they're correct.
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let gs = [Pair(2, 3), Pair(4, 5)] in
        list_map(λ j .
            let ga = list_nth(gs, j) in
            let gb = list_nth(gs, j + 1) in
            [fst(ga), snd(ga), fst(gb), snd(gb)]
        , [0])
        "#,
    )
    .unwrap();
    // Should be [[2, 3, 4, 5]]
    assert!(
        result.contains("2")
            && result.contains("3")
            && result.contains("4")
            && result.contains("5"),
        "fst/snd should extract correct values: got {}",
        result
    );
}

#[test]
fn test_3b_individual_products_in_list_map() {
    // Extract individual products as a list (no combining with +/-).
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let gs = [Pair(2, 3), Pair(4, 5)] in
        list_map(λ j .
            let ga = list_nth(gs, j) in
            let gb = list_nth(gs, j + 1) in
            [fst(ga)*snd(gb), snd(ga)*fst(gb)]
        , [0])
        "#,
    )
    .unwrap();
    // fst(ga)*snd(gb) = 2*5 = 10, snd(ga)*fst(gb) = 3*4 = 12
    assert!(
        result.contains("10") && result.contains("12"),
        "Individual products should be [10, 12]: got {}",
        result
    );
}

#[test]
fn test_3c_cr_dt_as_pair_in_list_map() {
    // Compute cr and dt as a Pair to see their raw values.
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let gs = [Pair(2, 3), Pair(4, 5)] in
        list_map(λ j .
            let ga = list_nth(gs, j) in
            let gb = list_nth(gs, j + 1) in
            let cr = fst(ga)*snd(gb) - snd(ga)*fst(gb) in
            let dt = fst(ga)*fst(gb) + snd(ga)*snd(gb) in
            Pair(cr, dt)
        , [0])
        "#,
    )
    .unwrap();
    // cr = 2*5 - 3*4 = -2, dt = 2*4 + 3*5 = 23
    assert!(
        result.contains("-2") && result.contains("23"),
        "cr should be -2 and dt should be 23: got {}",
        result
    );
}

#[test]
fn test_3d_beta_reduce_ast_inspection() {
    // Directly inspect the AST after beta reduction of the lambda.
    use kleis::ast::Expression;

    let evaluator = Evaluator::new();

    // Parse the lambda
    let mut parser = KleisParser::new(
        r#"λ j .
            let ga = list_nth(gs, j) in
            let gb = list_nth(gs, j + 1) in
            fst(ga)*snd(gb) - snd(ga)*fst(gb)
        "#,
    );
    let lambda = parser.parse().unwrap();

    // Beta reduce with j = 0
    let arg = Expression::Const("0".to_string());
    let reduced = evaluator.beta_reduce(&lambda, &arg).unwrap();

    // The reduced body should be a Let chain.
    // Verify it's structurally a Let with ga binding.
    match &reduced {
        Expression::Let {
            pattern: _,
            value,
            body,
            ..
        } => {
            // value should be list_nth(gs, 0)
            match value.as_ref() {
                Expression::Operation { name, args, .. } => {
                    assert_eq!(name, "list_nth", "First let should bind via list_nth");
                    // Second arg should be Const("0") after substitution
                    assert!(
                        matches!(&args[1], Expression::Const(s) if s == "0"),
                        "list_nth index should be 0 after beta reduction, got {:?}",
                        args[1]
                    );
                }
                other => panic!("Expected Operation for list_nth, got {:?}", other),
            }

            // body should be another Let (gb)
            match body.as_ref() {
                Expression::Let {
                    value: gb_val,
                    body: inner,
                    ..
                } => {
                    // gb value should be list_nth(gs, 0 + 1) or list_nth(gs, plus(0, 1))
                    match gb_val.as_ref() {
                        Expression::Operation { name, .. } => {
                            assert_eq!(name, "list_nth", "Second let should also be list_nth");
                        }
                        other => panic!("Expected list_nth for gb, got {:?}", other),
                    }

                    // inner body should be the arithmetic expression (minus of two times)
                    // Print it for inspection
                    let pp = PrettyPrinter::new();
                    let inner_str = pp.format_expression(inner);
                    eprintln!("Inner expression after beta reduce: {}", inner_str);

                    // It should be a minus(times(...), times(...))
                    match inner.as_ref() {
                        Expression::Operation { name, args, .. } => {
                            assert_eq!(
                                name, "minus",
                                "Top-level op should be minus, got '{}'.\nFull expr: {}",
                                name, inner_str
                            );
                            assert_eq!(args.len(), 2, "minus should have 2 args");
                            // Both args should be times(...)
                            for (i, arg) in args.iter().enumerate() {
                                match arg {
                                    Expression::Operation { name, .. } => {
                                        assert_eq!(
                                            name, "times",
                                            "arg {} of minus should be times, got '{}'",
                                            i, name
                                        );
                                    }
                                    other => panic!(
                                        "arg {} of minus should be times op, got {:?}",
                                        i, other
                                    ),
                                }
                            }
                        }
                        other => panic!(
                            "Expected minus operation, got {:?}\nFormatted: {}",
                            other, inner_str
                        ),
                    }
                }
                other => panic!("Expected inner Let for gb, got {:?}", other),
            }
        }
        other => panic!("Expected Let after beta reduce, got {:?}", other),
    }
}

// =============================================================================
// Layer 4: Hypothesis tests
// =============================================================================

// =============================================================================
// Layer 3b: Narrow the "subsequent let" trigger
// =============================================================================

#[test]
fn test_3e_cr_returned_via_let_binding() {
    // Same compound expression, but returned via let cr = ... in cr.
    // Does having a let binding around it break things?
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let gs = [Pair(2, 3), Pair(4, 5)] in
        list_map(λ j .
            let ga = list_nth(gs, j) in
            let gb = list_nth(gs, j + 1) in
            let cr = fst(ga)*snd(gb) - snd(ga)*fst(gb) in
            cr
        , [0])
        "#,
    )
    .unwrap();
    assert_eq!(result, "[-2]", "cr via let should be -2");
}

#[test]
fn test_3f_cr_with_trivial_subsequent_let() {
    // Compound cr, followed by a TRIVIAL let (no fst/snd).
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let gs = [Pair(2, 3), Pair(4, 5)] in
        list_map(λ j .
            let ga = list_nth(gs, j) in
            let gb = list_nth(gs, j + 1) in
            let cr = fst(ga)*snd(gb) - snd(ga)*fst(gb) in
            let dt = 99 in
            Pair(cr, dt)
        , [0])
        "#,
    )
    .unwrap();
    assert_eq!(
        result, "[Pair(-2, 99)]",
        "cr with trivial subsequent let should be -2"
    );
}

#[test]
fn test_3g_cr_with_compound_subsequent_let() {
    // Compound cr, followed by ANOTHER compound let using same ga/gb.
    // This is the minimal reproduction of the bug from test_3c.
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let gs = [Pair(2, 3), Pair(4, 5)] in
        list_map(λ j .
            let ga = list_nth(gs, j) in
            let gb = list_nth(gs, j + 1) in
            let cr = fst(ga)*snd(gb) - snd(ga)*fst(gb) in
            let dt = fst(ga)*fst(gb) + snd(ga)*snd(gb) in
            Pair(cr, dt)
        , [0])
        "#,
    )
    .unwrap();
    assert_eq!(result, "[Pair(-2, 23)]", "cr with compound subsequent let");
}

#[test]
fn test_3h_two_lets_but_second_uses_different_vars() {
    // Compound cr using ga/gb, followed by a let that does NOT use ga/gb.
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let gs = [Pair(2, 3), Pair(4, 5)] in
        list_map(λ j .
            let ga = list_nth(gs, j) in
            let gb = list_nth(gs, j + 1) in
            let cr = fst(ga)*snd(gb) - snd(ga)*fst(gb) in
            let dt = 10 + 13 in
            Pair(cr, dt)
        , [0])
        "#,
    )
    .unwrap();
    assert_eq!(result, "[Pair(-2, 23)]", "cr with non-ga/gb subsequent let");
}

#[test]
fn test_3i_parse_let_value_ast() {
    // Parse the lambda and inspect the AST to check if the parser
    // is parsing `let cr = EXPR in let dt = ...` correctly.
    use kleis::ast::Expression;
    let mut parser = KleisParser::new(
        r#"
        let cr = fst(ga)*snd(gb) - snd(ga)*fst(gb) in
        let dt = fst(ga)*fst(gb) + snd(ga)*snd(gb) in
        Pair(cr, dt)
        "#,
    );
    let expr = parser.parse().unwrap();

    let pp = PrettyPrinter::new();
    eprintln!("Parsed double-let: {}", pp.format_expression(&expr));

    // The top-level should be Let(cr, VALUE, BODY)
    match &expr {
        Expression::Let { value, body, .. } => {
            let val_str = pp.format_expression(value);
            eprintln!("  cr value: {}", val_str);

            // The VALUE of cr should be minus(times(...), times(...))
            match value.as_ref() {
                Expression::Operation { name, args, .. } => {
                    assert_eq!(
                        name, "minus",
                        "cr value top-op should be 'minus', got '{}'. Full: {}",
                        name, val_str
                    );
                    assert_eq!(args.len(), 2, "minus should have 2 args");
                }
                other => panic!("cr value should be Operation, got {:?}", other),
            }

            // The BODY should be Let(dt, ...)
            match body.as_ref() {
                Expression::Let { value: dt_val, .. } => {
                    let dt_str = pp.format_expression(dt_val);
                    eprintln!("  dt value: {}", dt_str);
                    match dt_val.as_ref() {
                        Expression::Operation { name, .. } => {
                            assert_eq!(
                                name, "plus",
                                "dt value top-op should be 'plus', got '{}'. Full: {}",
                                name, dt_str
                            );
                        }
                        other => panic!("dt value should be Operation, got {:?}", other),
                    }
                }
                other => panic!("cr body should be Let(dt,...), got {:?}", other),
            }
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

// =============================================================================
// Layer 5: PROVE the parser hypothesis
// =============================================================================

#[test]
fn test_5a_numeric_precedence_standalone_vs_let_in() {
    // 2*3 - 4*5 should be -14 regardless of context.
    let evaluator = Evaluator::new();

    // Standalone expression (uses parse_expression → correct precedence)
    let standalone = eval(&evaluator, "2*3 - 4*5").unwrap();
    assert_eq!(standalone, "-14", "Standalone: 2*3 - 4*5 = -14");

    // Inside let ... in (uses parse_let_value → suspected flat parser)
    let let_in = eval(&evaluator, "let x = 2*3 - 4*5 in x").unwrap();
    assert_eq!(let_in, "-14", "let x = 2*3 - 4*5 in x should be -14");
}

#[test]
fn test_5b_numeric_precedence_three_terms() {
    // 2*3 + 4*5 - 6*7 should be 6 + 20 - 42 = -16
    let evaluator = Evaluator::new();

    let standalone = eval(&evaluator, "2*3 + 4*5 - 6*7").unwrap();
    assert_eq!(standalone, "-16", "Standalone: 2*3 + 4*5 - 6*7 = -16");

    let let_in = eval(&evaluator, "let x = 2*3 + 4*5 - 6*7 in x").unwrap();
    assert_eq!(let_in, "-16", "let-in: 2*3 + 4*5 - 6*7 = -16");
}

#[test]
fn test_5c_parse_ast_standalone_vs_let_in() {
    // Parse the SAME expression two ways and compare AST structure.
    use kleis::ast::Expression;

    // Standalone: should parse as minus(times(2,3), times(4,5))
    let mut p1 = KleisParser::new("2*3 - 4*5");
    let standalone_ast = p1.parse().unwrap();

    let pp = PrettyPrinter::new();
    let standalone_str = pp.format_expression(&standalone_ast);
    eprintln!("Standalone AST: {}", standalone_str);

    // Verify standalone has correct structure: top = minus
    match &standalone_ast {
        Expression::Operation { name, .. } => {
            assert_eq!(
                name, "minus",
                "Standalone top-op should be minus, got {}",
                name
            );
        }
        other => panic!("Expected Operation, got {:?}", other),
    }

    // let-in: parse the value part
    let mut p2 = KleisParser::new("let x = 2*3 - 4*5 in x");
    let let_ast = p2.parse().unwrap();

    // Extract the value from the Let node
    match &let_ast {
        Expression::Let { value, .. } => {
            let let_value_str = pp.format_expression(value);
            eprintln!("Let-in value AST: {}", let_value_str);

            // This is the KEY assertion: the let value should ALSO have minus at the top
            match value.as_ref() {
                Expression::Operation { name, .. } => {
                    assert_eq!(
                        name, "minus",
                        "BUG CONFIRMED: let-in value top-op is '{}' instead of 'minus'.\n  Standalone: {}\n  Let-in val: {}",
                        name, standalone_str, let_value_str
                    );
                }
                other => panic!("Expected Operation for let value, got {:?}", other),
            }
        }
        other => panic!("Expected Let, got {:?}", other),
    }
}

#[test]
fn test_5d_parse_ast_with_function_calls() {
    // Same test but with fst/snd function calls (the original bug trigger).
    use kleis::ast::Expression;

    let pp = PrettyPrinter::new();

    // Standalone
    let mut p1 = KleisParser::new("fst(a)*snd(b) - snd(a)*fst(b)");
    let standalone = p1.parse().unwrap();
    let s1 = pp.format_expression(&standalone);
    eprintln!("Standalone: {}", s1);

    match &standalone {
        Expression::Operation { name, .. } => {
            assert_eq!(name, "minus", "Standalone top-op should be minus");
        }
        _ => panic!("Expected Operation"),
    }

    // let-in
    let mut p2 = KleisParser::new("let cr = fst(a)*snd(b) - snd(a)*fst(b) in cr");
    let let_ast = p2.parse().unwrap();
    match &let_ast {
        Expression::Let { value, .. } => {
            let s2 = pp.format_expression(value);
            eprintln!("Let-in value: {}", s2);

            match value.as_ref() {
                Expression::Operation { name, .. } => {
                    assert_eq!(
                        name, "minus",
                        "BUG: let-in value top-op is '{}', not 'minus'.\n  Standalone: {}\n  Let-in:     {}",
                        name, s1, s2
                    );
                }
                _ => panic!("Expected Operation for let value"),
            }
        }
        _ => panic!("Expected Let"),
    }
}

#[test]
fn test_5e_correct_parse_with_parentheses() {
    // Parenthesized version should work even with the broken parser.
    let evaluator = Evaluator::new();
    let result = eval(&evaluator, "let x = (2*3) - (4*5) in x").unwrap();
    assert_eq!(result, "-14", "Parenthesized let-in should be -14");
}

// =============================================================================
// Layer 4: Hypothesis tests
// =============================================================================

#[test]
fn test_4a_beta_reduce_preserves_variable_names() {
    // Verify that beta_reduce with a constant arg does NOT rename inner
    // let-bound variables (no spurious alpha conversion).
    use kleis::ast::Expression;

    let evaluator = Evaluator::new();

    let mut parser = KleisParser::new(
        r#"λ j .
            let ga = list_nth(gs, j) in
            let gb = list_nth(gs, j + 1) in
            fst(ga)*snd(gb) - snd(ga)*fst(gb)
        "#,
    );
    let lambda = parser.parse().unwrap();

    let arg = Expression::Const("0".to_string());
    let reduced = evaluator.beta_reduce(&lambda, &arg).unwrap();

    let pp = PrettyPrinter::new();
    let reduced_str = pp.format_expression(&reduced);
    eprintln!("After beta_reduce(lambda, 0): {}", reduced_str);

    // The inner variable names ga, gb should NOT be renamed (no capture risk).
    // If alpha conversion renamed them to ga', gb', that's a bug.
    assert!(
        reduced_str.contains("ga") && !reduced_str.contains("ga'"),
        "ga should not be alpha-renamed: {}",
        reduced_str
    );
    assert!(
        reduced_str.contains("gb") && !reduced_str.contains("gb'"),
        "gb should not be alpha-renamed: {}",
        reduced_str
    );
}

#[test]
fn test_4b_let_chain_eval() {
    // Evaluate the exact let chain that list_map would produce after beta reduction.
    // This tests eval_concrete on the substituted AST without going through list_map.
    let evaluator = Evaluator::new();
    let result = eval(
        &evaluator,
        r#"
        let gs = [Pair(2, 3), Pair(4, 5)] in
        let ga = list_nth(gs, 0) in
        let gb = list_nth(gs, 1) in
        fst(ga)*snd(gb) - snd(ga)*fst(gb)
        "#,
    )
    .unwrap();
    // 2*5 - 3*4 = -2
    assert_eq!(result, "-2", "Let-chain without list_map should be -2");
}

#[test]
fn test_4c_eval_concrete_compound_with_pair_args() {
    // Build the exact AST that would exist after substitution:
    // minus(times(fst(Pair(2,3)), snd(Pair(4,5))), times(snd(Pair(2,3)), fst(Pair(4,5))))
    // and evaluate it directly.
    use kleis::ast::Expression;

    let evaluator = Evaluator::new();

    let pair1 = Expression::operation(
        "Pair",
        vec![
            Expression::Const("2".to_string()),
            Expression::Const("3".to_string()),
        ],
    );
    let pair2 = Expression::operation(
        "Pair",
        vec![
            Expression::Const("4".to_string()),
            Expression::Const("5".to_string()),
        ],
    );

    let expr = Expression::operation(
        "minus",
        vec![
            Expression::operation(
                "times",
                vec![
                    Expression::operation("fst", vec![pair1.clone()]),
                    Expression::operation("snd", vec![pair2.clone()]),
                ],
            ),
            Expression::operation(
                "times",
                vec![
                    Expression::operation("snd", vec![pair1.clone()]),
                    Expression::operation("fst", vec![pair2.clone()]),
                ],
            ),
        ],
    );

    let result = evaluator.eval_concrete(&expr).unwrap();
    let pp = PrettyPrinter::new();
    let result_str = pp.format_expression(&result);
    assert_eq!(result_str, "-2", "Hand-built AST should evaluate to -2");
}
