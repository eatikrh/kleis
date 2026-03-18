//! Tests for intToStr and strToInt builtins
//!
//! These builtins enable conversion between integers and strings,
//! which is essential for document generation (thesis_compiler.kleis).

use kleis::evaluator::Evaluator;
use kleis::kleis_parser::KleisParser;
use kleis::pretty_print::PrettyPrinter;

/// Helper to evaluate an expression string and return the result
fn eval(evaluator: &Evaluator, input: &str) -> Result<String, String> {
    let mut parser = KleisParser::new(input);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    let result = evaluator.eval_concrete(&expr)?;
    let pp = PrettyPrinter::new();
    Ok(pp.format_expression(&result))
}

// ============================================================================
// intToStr tests
// ============================================================================

#[test]
fn test_int_to_str_zero() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "intToStr(0)").unwrap(), "\"0\"");
}

#[test]
fn test_int_to_str_positive() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "intToStr(42)").unwrap(), "\"42\"");
}

#[test]
fn test_int_to_str_large() {
    let evaluator = Evaluator::new();
    assert_eq!(
        eval(&evaluator, "intToStr(1000000)").unwrap(),
        "\"1000000\""
    );
}

// ============================================================================
// strToInt tests
// ============================================================================

#[test]
fn test_str_to_int_zero() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "strToInt(\"0\")").unwrap(), "0");
}

#[test]
fn test_str_to_int_positive() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "strToInt(\"42\")").unwrap(), "42");
}

#[test]
fn test_str_to_int_negative() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "strToInt(\"-123\")").unwrap(), "-123");
}

#[test]
fn test_str_to_int_invalid() {
    let evaluator = Evaluator::new();
    // Invalid string returns -1
    assert_eq!(eval(&evaluator, "strToInt(\"abc\")").unwrap(), "-1");
}

#[test]
fn test_str_to_int_empty() {
    let evaluator = Evaluator::new();
    // Empty string returns -1
    assert_eq!(eval(&evaluator, "strToInt(\"\")").unwrap(), "-1");
}

#[test]
fn test_str_to_int_whitespace() {
    let evaluator = Evaluator::new();
    // Whitespace should be trimmed
    assert_eq!(eval(&evaluator, "strToInt(\"  42  \")").unwrap(), "42");
}

// ============================================================================
// Round-trip tests
// ============================================================================

#[test]
fn test_roundtrip_int_to_str_to_int() {
    let evaluator = Evaluator::new();
    // strToInt(intToStr(42)) should be 42
    assert_eq!(eval(&evaluator, "strToInt(intToStr(42))").unwrap(), "42");
}

#[test]
fn test_roundtrip_larger_number() {
    let evaluator = Evaluator::new();
    assert_eq!(
        eval(&evaluator, "strToInt(intToStr(99999))").unwrap(),
        "99999"
    );
}

// ============================================================================
// Alternative name tests (aliases)
// ============================================================================

#[test]
fn test_from_int_alias() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "fromInt(99)").unwrap(), "\"99\"");
}

#[test]
fn test_to_int_alias() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "toInt(\"99\")").unwrap(), "99");
}

#[test]
fn test_int_to_string_alias() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "intToString(77)").unwrap(), "\"77\"");
}

#[test]
fn test_str_to_int_alias() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "str_to_int(\"88\")").unwrap(), "88");
}

#[test]
fn test_int_to_str_alias() {
    let evaluator = Evaluator::new();
    assert_eq!(eval(&evaluator, "int_to_str(66)").unwrap(), "\"66\"");
}
