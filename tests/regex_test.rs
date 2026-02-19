//! Tests for Regular Expression operations
//!
//! These tests verify that Kleis's regex operations work correctly
//! with Z3's native regex theory:
//!   - Composable regex constructors (re_literal, re_range, re_star, etc.)
//!   - String-to-regex matching (matches)
//!   - Convenience predicates (isDigits, isAlpha, isAlphaNum, isAscii)
//!   - Formal verification of regex properties via Z3 quantifiers

#![allow(unused_imports)]

use kleis::ast::{Expression, QuantifiedVar, QuantifierKind};
use kleis::solvers::backend::{SatisfiabilityResult, SolverBackend, VerificationResult};
use kleis::solvers::z3::Z3Backend;
use kleis::structure_registry::StructureRegistry;

/// Helper to create a string literal expression
fn str_lit(s: &str) -> Expression {
    Expression::String(s.to_string())
}

/// Helper to create an operation expression
fn op(name: &str, args: Vec<Expression>) -> Expression {
    Expression::Operation {
        name: name.to_string(),
        args,
        span: None,
    }
}

/// Helper to create a universal quantifier over strings
fn forall_string(var: &str, body: Expression) -> Expression {
    Expression::Quantifier {
        quantifier: QuantifierKind::ForAll,
        variables: vec![QuantifiedVar {
            name: var.to_string(),
            type_annotation: Some("String".to_string()),
        }],
        where_clause: None,
        body: Box::new(body),
    }
}

// ============================================
// SECTION 1: Convenience Predicates
// ============================================

/// Test isDigits with digit-only strings
#[test]
#[cfg(feature = "axiom-verification")]
fn test_is_digits_true() {
    println!("\n🧪 Testing: isDigits(\"12345\") should be true");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // isDigits("12345") — should be valid (always true)
    let expr = op("isDigits", vec![str_lit("12345")]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "isDigits(\"12345\") should be true"
    );
    println!("   ✅ isDigits(\"12345\") = true verified");
}

/// Test isDigits rejects non-digit strings
#[test]
#[cfg(feature = "axiom-verification")]
fn test_is_digits_false() {
    println!("\n🧪 Testing: isDigits(\"hello\") should be false");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // not(isDigits("hello")) — should be valid
    let expr = op("not", vec![op("isDigits", vec![str_lit("hello")])]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "isDigits(\"hello\") should be false"
    );
    println!("   ✅ isDigits(\"hello\") = false verified");
}

/// Test isAlpha with letter-only strings
#[test]
#[cfg(feature = "axiom-verification")]
fn test_is_alpha_true() {
    println!("\n🧪 Testing: isAlpha(\"Hello\") should be true");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let expr = op("isAlpha", vec![str_lit("Hello")]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "isAlpha(\"Hello\") should be true"
    );
    println!("   ✅ isAlpha(\"Hello\") = true verified");
}

/// Test isAlphaNum with mixed strings
#[test]
#[cfg(feature = "axiom-verification")]
fn test_is_alphanum_true() {
    println!("\n🧪 Testing: isAlphaNum(\"Test123\") should be true");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let expr = op("isAlphaNum", vec![str_lit("Test123")]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "isAlphaNum(\"Test123\") should be true"
    );
    println!("   ✅ isAlphaNum(\"Test123\") = true verified");
}

/// Test isAscii with printable ASCII
#[test]
#[cfg(feature = "axiom-verification")]
fn test_is_ascii_printable() {
    println!("\n🧪 Testing: isAscii with printable ASCII strings");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // isAscii("Hello, World! 42") — should be valid
    let expr = op("isAscii", vec![str_lit("Hello, World! 42")]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "isAscii(\"Hello, World! 42\") should be true"
    );
    println!("   ✅ isAscii(\"Hello, World! 42\") = true verified");
}

/// Test isAscii rejects emoji
#[test]
#[cfg(feature = "axiom-verification")]
fn test_is_ascii_rejects_emoji() {
    println!("\n🧪 Testing: isAscii rejects emoji");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // not(isAscii("Hello 🌍")) — should be valid
    let expr = op("not", vec![op("isAscii", vec![str_lit("Hello \u{1F30D}")])]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "isAscii with emoji should be false"
    );
    println!("   ✅ isAscii rejects emoji verified");
}

/// Test isAscii accepts empty string
#[test]
#[cfg(feature = "axiom-verification")]
fn test_is_ascii_empty_string() {
    println!("\n🧪 Testing: isAscii(\"\") should be true (star matches empty)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let expr = op("isAscii", vec![str_lit("")]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "isAscii(\"\") should be true"
    );
    println!("   ✅ isAscii(\"\") = true verified");
}

// ============================================
// SECTION 2: Composable Regex Constructors
// ============================================

/// Test re_literal matches exact string
#[test]
#[cfg(feature = "axiom-verification")]
fn test_re_literal_exact_match() {
    println!("\n🧪 Testing: matches(\"foo\", re_literal(\"foo\")) = true");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let expr = op(
        "matches",
        vec![str_lit("foo"), op("re_literal", vec![str_lit("foo")])],
    );
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "re_literal(\"foo\") should match \"foo\""
    );
    println!("   ✅ re_literal exact match verified");
}

/// Test re_literal rejects non-matching string
#[test]
#[cfg(feature = "axiom-verification")]
fn test_re_literal_no_match() {
    println!("\n🧪 Testing: matches(\"bar\", re_literal(\"foo\")) = false");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let expr = op(
        "not",
        vec![op(
            "matches",
            vec![str_lit("bar"), op("re_literal", vec![str_lit("foo")])],
        )],
    );
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "re_literal(\"foo\") should not match \"bar\""
    );
    println!("   ✅ re_literal non-match verified");
}

/// Test re_range matches characters in range
#[test]
#[cfg(feature = "axiom-verification")]
fn test_re_range_single_char() {
    println!("\n🧪 Testing: matches(\"m\", re_range(\"a\", \"z\")) = true");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let expr = op(
        "matches",
        vec![
            str_lit("m"),
            op("re_range", vec![str_lit("a"), str_lit("z")]),
        ],
    );
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "re_range(\"a\", \"z\") should match \"m\""
    );
    println!("   ✅ re_range single char match verified");
}

/// Test re_plus matches one or more
#[test]
#[cfg(feature = "axiom-verification")]
fn test_re_plus_one_or_more() {
    println!("\n🧪 Testing: re_plus(re_range(\"a\", \"z\")) matches \"hello\"");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let re = op(
        "re_plus",
        vec![op("re_range", vec![str_lit("a"), str_lit("z")])],
    );
    let expr = op("matches", vec![str_lit("hello"), re]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "re_plus should match one or more lowercase letters"
    );
    println!("   ✅ re_plus match verified");
}

/// Test re_plus rejects empty string
#[test]
#[cfg(feature = "axiom-verification")]
fn test_re_plus_rejects_empty() {
    println!("\n🧪 Testing: re_plus(re_range(\"a\", \"z\")) rejects \"\"");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let re = op(
        "re_plus",
        vec![op("re_range", vec![str_lit("a"), str_lit("z")])],
    );
    let expr = op("not", vec![op("matches", vec![str_lit(""), re])]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "re_plus should reject empty string"
    );
    println!("   ✅ re_plus rejects empty string verified");
}

/// Test re_star accepts empty string
#[test]
#[cfg(feature = "axiom-verification")]
fn test_re_star_accepts_empty() {
    println!("\n🧪 Testing: re_star(re_range(\"a\", \"z\")) accepts \"\"");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let re = op(
        "re_star",
        vec![op("re_range", vec![str_lit("a"), str_lit("z")])],
    );
    let expr = op("matches", vec![str_lit(""), re]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "re_star should accept empty string"
    );
    println!("   ✅ re_star accepts empty string verified");
}

/// Test re_union alternation
#[test]
#[cfg(feature = "axiom-verification")]
fn test_re_union_alternation() {
    println!("\n🧪 Testing: re_union(re_literal(\"yes\"), re_literal(\"no\"))");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let re = op(
        "re_union",
        vec![
            op("re_literal", vec![str_lit("yes")]),
            op("re_literal", vec![str_lit("no")]),
        ],
    );

    // "yes" should match
    let expr = op("matches", vec![str_lit("yes"), re.clone()]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "re_union should match \"yes\""
    );
    println!("   ✅ \"yes\" matches");

    // "no" should match
    let expr = op("matches", vec![str_lit("no"), re.clone()]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "re_union should match \"no\""
    );
    println!("   ✅ \"no\" matches");

    // "maybe" should not match
    let expr = op("not", vec![op("matches", vec![str_lit("maybe"), re])]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "re_union should not match \"maybe\""
    );
    println!("   ✅ \"maybe\" rejected");
}

/// Test re_concat sequence
#[test]
#[cfg(feature = "axiom-verification")]
fn test_re_concat_sequence() {
    println!("\n🧪 Testing: re_concat(re_literal(\"foo\"), re_plus(re_range(\"0\", \"9\")))");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // Pattern: "foo" followed by one or more digits
    let re = op(
        "re_concat",
        vec![
            op("re_literal", vec![str_lit("foo")]),
            op(
                "re_plus",
                vec![op("re_range", vec![str_lit("0"), str_lit("9")])],
            ),
        ],
    );

    // "foo42" should match
    let expr = op("matches", vec![str_lit("foo42"), re.clone()]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "re_concat should match \"foo42\""
    );
    println!("   ✅ \"foo42\" matches foo[0-9]+");

    // "foo" alone should not match (needs at least one digit)
    let expr = op("not", vec![op("matches", vec![str_lit("foo"), re])]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "re_concat should not match \"foo\" alone"
    );
    println!("   ✅ \"foo\" rejected (needs digits)");
}

/// Test re_option (zero or one)
#[test]
#[cfg(feature = "axiom-verification")]
fn test_re_option_optional() {
    println!("\n🧪 Testing: re_option(re_literal(\"s\")) for optional plural");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // Pattern: "cat" optionally followed by "s"
    let re = op(
        "re_concat",
        vec![
            op("re_literal", vec![str_lit("cat")]),
            op("re_option", vec![op("re_literal", vec![str_lit("s")])]),
        ],
    );

    // "cat" should match
    let expr = op("matches", vec![str_lit("cat"), re.clone()]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(matches!(
        result,
        VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
    ),);
    println!("   ✅ \"cat\" matches cat(s)?");

    // "cats" should match
    let expr = op("matches", vec![str_lit("cats"), re]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(matches!(
        result,
        VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
    ),);
    println!("   ✅ \"cats\" matches cat(s)?");
}

/// Test re_complement
#[test]
#[cfg(feature = "axiom-verification")]
fn test_re_complement_negation() {
    println!("\n🧪 Testing: re_complement — match anything that ISN'T all digits");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // Complement of digits+: matches anything that isn't purely digits
    let re = op(
        "re_complement",
        vec![op(
            "re_plus",
            vec![op("re_range", vec![str_lit("0"), str_lit("9")])],
        )],
    );

    // "abc" matches complement (not all digits)
    let expr = op("matches", vec![str_lit("abc"), re.clone()]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(matches!(
        result,
        VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
    ),);
    println!("   ✅ \"abc\" matches complement of [0-9]+");

    // "123" should NOT match complement
    let expr = op("not", vec![op("matches", vec![str_lit("123"), re])]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(matches!(
        result,
        VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
    ),);
    println!("   ✅ \"123\" rejected by complement of [0-9]+");
}

// ============================================
// SECTION 3: Backward Compatibility
// ============================================

/// Test that matches() still works with a string literal (backward compatible)
#[test]
#[cfg(feature = "axiom-verification")]
fn test_matches_backward_compat_string_literal() {
    println!("\n🧪 Testing: matches(\"hello\", \"hello\") backward compat");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let expr = op("matches", vec![str_lit("hello"), str_lit("hello")]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "Backward-compatible literal match should work"
    );
    println!("   ✅ Backward-compatible matches(s, literal) works");
}

// ============================================
// SECTION 4: Z3 Verification of Regex Properties
// ============================================

/// Test Z3 can verify: isDigits(s) → isAlphaNum(s)
/// (If a string is all digits, it's also alphanumeric)
#[test]
#[cfg(feature = "axiom-verification")]
fn test_z3_verify_digits_implies_alphanum() {
    println!("\n🧪 Testing: ∀(s : String). isDigits(s) → isAlphaNum(s)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let axiom = forall_string(
        "s",
        op(
            "implies",
            vec![
                op("isDigits", vec![Expression::Object("s".to_string())]),
                op("isAlphaNum", vec![Expression::Object("s".to_string())]),
            ],
        ),
    );

    let result = backend.verify_axiom(&axiom).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "isDigits(s) → isAlphaNum(s) should be verified"
    );
    println!("   ✅ Verified: digits ⊂ alphanumeric");
}

/// Test Z3 can verify: isAlpha(s) → isAlphaNum(s)
/// (If a string is all letters, it's also alphanumeric)
#[test]
#[cfg(feature = "axiom-verification")]
fn test_z3_verify_alpha_implies_alphanum() {
    println!("\n🧪 Testing: ∀(s : String). isAlpha(s) → isAlphaNum(s)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let axiom = forall_string(
        "s",
        op(
            "implies",
            vec![
                op("isAlpha", vec![Expression::Object("s".to_string())]),
                op("isAlphaNum", vec![Expression::Object("s".to_string())]),
            ],
        ),
    );

    let result = backend.verify_axiom(&axiom).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "isAlpha(s) → isAlphaNum(s) should be verified"
    );
    println!("   ✅ Verified: alpha ⊂ alphanumeric");
}

/// Test Z3 can verify: isAlphaNum(s) → isAscii(s)
/// (All alphanumeric characters are ASCII printable)
#[test]
#[cfg(feature = "axiom-verification")]
fn test_z3_verify_alphanum_implies_ascii() {
    println!("\n🧪 Testing: ∀(s : String). isAlphaNum(s) → isAscii(s)");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let axiom = forall_string(
        "s",
        op(
            "implies",
            vec![
                op("isAlphaNum", vec![Expression::Object("s".to_string())]),
                op("isAscii", vec![Expression::Object("s".to_string())]),
            ],
        ),
    );

    let result = backend.verify_axiom(&axiom).unwrap();
    assert!(
        matches!(
            result,
            VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
        ),
        "isAlphaNum(s) → isAscii(s) should be verified"
    );
    println!("   ✅ Verified: alphanumeric ⊂ ASCII");
}

// ============================================
// SECTION 5: Nullary Regex Constructors
// ============================================

/// Test re_full matches any string
#[test]
#[cfg(feature = "axiom-verification")]
fn test_re_full_matches_anything() {
    println!("\n🧪 Testing: matches(\"anything\", re_full()) = true");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let expr = op(
        "matches",
        vec![str_lit("anything at all!"), op("re_full", vec![])],
    );
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(matches!(
        result,
        VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
    ),);
    println!("   ✅ re_full() matches any string");
}

/// Test re_empty matches nothing
#[test]
#[cfg(feature = "axiom-verification")]
fn test_re_empty_matches_nothing() {
    println!("\n🧪 Testing: matches(\"x\", re_empty()) = false");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    let expr = op(
        "not",
        vec![op("matches", vec![str_lit("x"), op("re_empty", vec![])])],
    );
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(matches!(
        result,
        VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
    ),);
    println!("   ✅ re_empty() matches nothing");
}

/// Test re_allchar matches exactly one character
#[test]
#[cfg(feature = "axiom-verification")]
fn test_re_allchar_single_char() {
    println!("\n🧪 Testing: re_allchar() matches single char, rejects multi");

    let registry = StructureRegistry::new();
    let mut backend = Z3Backend::new(&registry).unwrap();

    // "x" (single char) matches
    let expr = op("matches", vec![str_lit("x"), op("re_allchar", vec![])]);
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(matches!(
        result,
        VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
    ),);
    println!("   ✅ re_allchar() matches \"x\"");

    // "xy" (two chars) should NOT match
    let expr = op(
        "not",
        vec![op("matches", vec![str_lit("xy"), op("re_allchar", vec![])])],
    );
    let result = backend.verify_axiom(&expr).unwrap();
    assert!(matches!(
        result,
        VerificationResult::Valid | VerificationResult::ValidWithWitness { .. }
    ),);
    println!("   ✅ re_allchar() rejects \"xy\"");
}
