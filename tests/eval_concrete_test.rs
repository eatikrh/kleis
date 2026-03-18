//! Integration tests for :eval concrete evaluation
//!
//! Tests the eval_concrete functionality that enables actual computation
//! in Kleis (as opposed to symbolic evaluation or Z3 verification).

use kleis::evaluator::Evaluator;
use kleis::kleis_parser::parse_kleis_program;

/// Helper to evaluate an expression string and return the result as a string
fn eval(evaluator: &Evaluator, input: &str) -> Result<String, String> {
    use kleis::kleis_parser::KleisParser;
    use kleis::pretty_print::PrettyPrinter;

    let mut parser = KleisParser::new(input);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    let result = evaluator.eval_concrete(&expr)?;
    let pp = PrettyPrinter::new();
    Ok(pp.format_expression(&result))
}

// =============================================================================
// Arithmetic Tests
// =============================================================================

#[test]
fn test_arithmetic_addition() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "2 + 3").unwrap(), "5");
    assert_eq!(eval(&evaluator, "100 + 200").unwrap(), "300");
    assert_eq!(eval(&evaluator, "0 + 0").unwrap(), "0");
}

#[test]
fn test_arithmetic_subtraction() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "10 - 3").unwrap(), "7");
    assert_eq!(eval(&evaluator, "5 - 10").unwrap(), "-5");
}

#[test]
fn test_arithmetic_multiplication() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "6 * 7").unwrap(), "42");
    assert_eq!(eval(&evaluator, "0 * 100").unwrap(), "0");
}

#[test]
fn test_arithmetic_division() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "div(10, 2)").unwrap(), "5");
    assert_eq!(eval(&evaluator, "div(7, 2)").unwrap(), "3.5");
}

#[test]
fn test_arithmetic_nested() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "(2 + 3) * 4").unwrap(), "20");
    assert_eq!(eval(&evaluator, "2 + 3 * 4").unwrap(), "14"); // Tests precedence
}

// =============================================================================
// String Operation Tests
// =============================================================================

#[test]
fn test_string_concat() {
    let evaluator = Evaluator::new();
    assert_eq!(
        eval(&evaluator, r#"concat("hello", " world")"#).unwrap(),
        "\"hello world\""
    );
    assert_eq!(
        eval(&evaluator, r#"concat("", "test")"#).unwrap(),
        "\"test\""
    );
}

#[test]
fn test_string_strlen() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, r#"strlen("kleis")"#).unwrap(), "5");
    assert_eq!(eval(&evaluator, r#"strlen("")"#).unwrap(), "0");
    assert_eq!(eval(&evaluator, r#"strlen("hello world")"#).unwrap(), "11");
}

#[test]
fn test_string_has_prefix() {
    let evaluator = Evaluator::new();
    assert_eq!(
        eval(&evaluator, r#"hasPrefix("(define fib)", "(define")"#).unwrap(),
        "true"
    );
    assert_eq!(
        eval(&evaluator, r#"hasPrefix("hello", "world")"#).unwrap(),
        "false"
    );
}

#[test]
fn test_string_has_suffix() {
    let evaluator = Evaluator::new();
    assert_eq!(
        eval(&evaluator, r#"hasSuffix("hello.kleis", ".kleis")"#).unwrap(),
        "true"
    );
    assert_eq!(
        eval(&evaluator, r#"hasSuffix("hello", "world")"#).unwrap(),
        "false"
    );
}

#[test]
fn test_string_contains() {
    let evaluator = Evaluator::new();
    assert_eq!(
        eval(&evaluator, r#"contains("hello world", "wor")"#).unwrap(),
        "true"
    );
    assert_eq!(
        eval(&evaluator, r#"contains("hello", "xyz")"#).unwrap(),
        "false"
    );
}

#[test]
fn test_string_index_of() {
    let evaluator = Evaluator::new();
    assert_eq!(
        eval(&evaluator, r#"indexOf("hello world", "world")"#).unwrap(),
        "6"
    );
    assert_eq!(
        eval(&evaluator, r#"indexOf("hello", "xyz")"#).unwrap(),
        "-1"
    );
}

#[test]
fn test_string_substr() {
    let evaluator = Evaluator::new();
    assert_eq!(
        eval(&evaluator, r#"substr("hello world", 0, 5)"#).unwrap(),
        "\"hello\""
    );
    assert_eq!(
        eval(&evaluator, r#"substr("hello world", 6, 5)"#).unwrap(),
        "\"world\""
    );
}

#[test]
fn test_string_char_at() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, r#"charAt("hello", 0)"#).unwrap(), "\"h\"");
    assert_eq!(eval(&evaluator, r#"charAt("hello", 4)"#).unwrap(), "\"o\"");
}

#[test]
fn test_string_replace() {
    let evaluator = Evaluator::new();
    assert_eq!(
        eval(&evaluator, r#"replace("hello world", "world", "kleis")"#).unwrap(),
        "\"hello kleis\""
    );
}

// =============================================================================
// Comparison Tests
// =============================================================================

#[test]
fn test_comparison_gt() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "gt(5, 3)").unwrap(), "true");
    assert_eq!(eval(&evaluator, "gt(3, 5)").unwrap(), "false");
    assert_eq!(eval(&evaluator, "gt(5, 5)").unwrap(), "false");
}

#[test]
fn test_comparison_lt() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "lt(3, 5)").unwrap(), "true");
    assert_eq!(eval(&evaluator, "lt(5, 3)").unwrap(), "false");
}

#[test]
fn test_comparison_ge() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "ge(5, 3)").unwrap(), "true");
    assert_eq!(eval(&evaluator, "ge(5, 5)").unwrap(), "true");
    assert_eq!(eval(&evaluator, "ge(3, 5)").unwrap(), "false");
}

#[test]
fn test_comparison_le() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "le(3, 5)").unwrap(), "true");
    assert_eq!(eval(&evaluator, "le(5, 5)").unwrap(), "true");
    assert_eq!(eval(&evaluator, "le(5, 3)").unwrap(), "false");
}

#[test]
fn test_comparison_eq() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "eq(5, 5)").unwrap(), "true");
    assert_eq!(eval(&evaluator, "eq(5, 3)").unwrap(), "false");
}

// =============================================================================
// Boolean Tests
// =============================================================================

#[test]
fn test_boolean_and() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "and(true, true)").unwrap(), "true");
    assert_eq!(eval(&evaluator, "and(true, false)").unwrap(), "false");
    assert_eq!(eval(&evaluator, "and(false, false)").unwrap(), "false");
}

#[test]
fn test_boolean_or() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "or(true, false)").unwrap(), "true");
    assert_eq!(eval(&evaluator, "or(false, false)").unwrap(), "false");
}

#[test]
fn test_boolean_not() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "not(true)").unwrap(), "false");
    assert_eq!(eval(&evaluator, "not(false)").unwrap(), "true");
}

// =============================================================================
// Conditional Tests
// =============================================================================

#[test]
fn test_conditional_true_branch() {
    let evaluator = Evaluator::new();
    assert_eq!(
        eval(&evaluator, r#"if gt(5, 3) then "yes" else "no""#).unwrap(),
        "\"yes\""
    );
}

#[test]
fn test_conditional_false_branch() {
    let evaluator = Evaluator::new();
    assert_eq!(
        eval(&evaluator, r#"if lt(5, 3) then "yes" else "no""#).unwrap(),
        "\"no\""
    );
}

#[test]
fn test_conditional_nested() {
    let evaluator = Evaluator::new();
    // if gt(10, 5) then (if lt(3, 2) then "a" else "b") else "c"
    // gt(10, 5) = true, so we evaluate inner: lt(3, 2) = false, so "b"
    assert_eq!(
        eval(
            &evaluator,
            r#"if gt(10, 5) then (if lt(3, 2) then "a" else "b") else "c""#
        )
        .unwrap(),
        "\"b\""
    );
}

// =============================================================================
// User-Defined Function Tests
// =============================================================================

#[test]
fn test_user_function_simple() {
    let mut evaluator = Evaluator::new();

    let code = "define double(x) = x + x";
    let program = parse_kleis_program(code).unwrap();
    evaluator.load_program(&program).unwrap();

    assert_eq!(eval(&evaluator, "double(5)").unwrap(), "10");
    assert_eq!(eval(&evaluator, "double(0)").unwrap(), "0");
}

#[test]
fn test_user_function_multiple_params() {
    let mut evaluator = Evaluator::new();

    let code = "define add3(x, y, z) = x + y + z";
    let program = parse_kleis_program(code).unwrap();
    evaluator.load_program(&program).unwrap();

    assert_eq!(eval(&evaluator, "add3(1, 2, 3)").unwrap(), "6");
}

#[test]
fn test_user_function_composition() {
    let mut evaluator = Evaluator::new();

    let code = r#"
        define double(x) = x + x
        define quadruple(x) = double(double(x))
    "#;
    let program = parse_kleis_program(code).unwrap();
    evaluator.load_program(&program).unwrap();

    assert_eq!(eval(&evaluator, "quadruple(3)").unwrap(), "12");
}

#[test]
fn test_user_function_with_conditional() {
    let mut evaluator = Evaluator::new();

    let code = "define abs(x) = if ge(x, 0) then x else 0 - x";
    let program = parse_kleis_program(code).unwrap();
    evaluator.load_program(&program).unwrap();

    assert_eq!(eval(&evaluator, "abs(5)").unwrap(), "5");
    assert_eq!(eval(&evaluator, "abs(0)").unwrap(), "0");
    // Note: -5 would need to be parsed differently
}

// =============================================================================
// Recursion Tests
// =============================================================================

#[test]
fn test_recursion_fibonacci() {
    let mut evaluator = Evaluator::new();

    let code = "define fib(n) = if le(n, 1) then n else fib(n - 1) + fib(n - 2)";
    let program = parse_kleis_program(code).unwrap();
    evaluator.load_program(&program).unwrap();

    assert_eq!(eval(&evaluator, "fib(0)").unwrap(), "0");
    assert_eq!(eval(&evaluator, "fib(1)").unwrap(), "1");
    assert_eq!(eval(&evaluator, "fib(2)").unwrap(), "1");
    assert_eq!(eval(&evaluator, "fib(5)").unwrap(), "5");
    assert_eq!(eval(&evaluator, "fib(10)").unwrap(), "55");
}

#[test]
fn test_recursion_factorial() {
    let mut evaluator = Evaluator::new();

    let code = "define fact(n) = if le(n, 1) then 1 else n * fact(n - 1)";
    let program = parse_kleis_program(code).unwrap();
    evaluator.load_program(&program).unwrap();

    assert_eq!(eval(&evaluator, "fact(0)").unwrap(), "1");
    assert_eq!(eval(&evaluator, "fact(1)").unwrap(), "1");
    assert_eq!(eval(&evaluator, "fact(5)").unwrap(), "120");
}

#[test]
fn test_recursion_sum() {
    let mut evaluator = Evaluator::new();

    let code = "define sum_to(n) = if le(n, 0) then 0 else n + sum_to(n - 1)";
    let program = parse_kleis_program(code).unwrap();
    evaluator.load_program(&program).unwrap();

    assert_eq!(eval(&evaluator, "sum_to(0)").unwrap(), "0");
    assert_eq!(eval(&evaluator, "sum_to(5)").unwrap(), "15"); // 5+4+3+2+1
    assert_eq!(eval(&evaluator, "sum_to(10)").unwrap(), "55"); // 10+9+...+1
}

// =============================================================================
// LISP Parsing Tests
// =============================================================================

#[test]
fn test_lisp_is_list_expr() {
    let mut evaluator = Evaluator::new();

    let code = r#"define is_list_expr(s) = hasPrefix(s, "(")"#;
    let program = parse_kleis_program(code).unwrap();
    evaluator.load_program(&program).unwrap();

    assert_eq!(
        eval(&evaluator, r#"is_list_expr("(+ 2 3)")"#).unwrap(),
        "true"
    );
    assert_eq!(
        eval(&evaluator, r#"is_list_expr("hello")"#).unwrap(),
        "false"
    );
}

#[test]
fn test_lisp_strip_parens() {
    let mut evaluator = Evaluator::new();

    let code = "define strip_parens(s) = substr(s, 1, strlen(s) - 2)";
    let program = parse_kleis_program(code).unwrap();
    evaluator.load_program(&program).unwrap();

    assert_eq!(
        eval(&evaluator, r#"strip_parens("(+ 2 3)")"#).unwrap(),
        "\"+ 2 3\""
    );
    assert_eq!(
        eval(&evaluator, r#"strip_parens("(hello)")"#).unwrap(),
        "\"hello\""
    );
}

#[test]
fn test_lisp_get_operator() {
    let mut evaluator = Evaluator::new();

    let code = r#"
        define strip_parens(s) = substr(s, 1, strlen(s) - 2)
        define get_op(s) = charAt(strip_parens(s), 0)
    "#;
    let program = parse_kleis_program(code).unwrap();
    evaluator.load_program(&program).unwrap();

    assert_eq!(eval(&evaluator, r#"get_op("(+ 2 3)")"#).unwrap(), "\"+\"");
    assert_eq!(eval(&evaluator, r#"get_op("(* 10 20)")"#).unwrap(), "\"*\"");
}

#[test]
fn test_lisp_balanced_parens() {
    let mut evaluator = Evaluator::new();

    let code = r#"
        define count_open(s) = strlen(s) - strlen(replace(s, "(", ""))
        define count_close(s) = strlen(s) - strlen(replace(s, ")", ""))
        define is_balanced(s) = eq(count_open(s), count_close(s))
    "#;
    let program = parse_kleis_program(code).unwrap();
    evaluator.load_program(&program).unwrap();

    assert_eq!(
        eval(&evaluator, r#"is_balanced("(+ 2 3)")"#).unwrap(),
        "true"
    );
    assert_eq!(
        eval(&evaluator, r#"is_balanced("((a) (b))")"#).unwrap(),
        "true"
    );
    assert_eq!(
        eval(&evaluator, r#"is_balanced("(+ 2 3")"#).unwrap(),
        "false"
    );
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_empty_string() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, r#"strlen("")"#).unwrap(), "0");
    assert_eq!(eval(&evaluator, r#"concat("", "")"#).unwrap(), "\"\"");
}

#[test]
fn test_zero_arithmetic() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "0 + 0").unwrap(), "0");
    assert_eq!(eval(&evaluator, "0 * 100").unwrap(), "0");
    assert_eq!(eval(&evaluator, "100 - 100").unwrap(), "0");
}

#[test]
fn test_negative_numbers() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "5 - 10").unwrap(), "-5");
    assert_eq!(eval(&evaluator, "0 - 42").unwrap(), "-42");
}
