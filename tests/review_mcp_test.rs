//! Integration tests for the Review MCP Engine
//!
//! Tests the code review pipeline:
//! - Policy loading
//! - check_code (pass/fail for each rule)
//! - list_rules / explain_rule
//! - describe_standards
//! - Real policy file (rust_review_policy.kleis)

use kleis::review_mcp::engine::ReviewEngine;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn load_review_policy(source: &str) -> ReviewEngine {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "kleis_test_review_{}_{}.kleis",
        std::process::id(),
        id
    ));
    std::fs::write(&path, source).expect("write temp policy file");
    let engine = ReviewEngine::load(&path).expect("load review policy");
    let _ = std::fs::remove_file(&path);
    engine
}

// =============================================================================
// Policy Loading
// =============================================================================

#[test]
fn test_load_empty_review_policy() {
    let engine = load_review_policy("// empty\n");
    let stats = engine.stats();
    assert_eq!(*stats.get("total_rules").unwrap(), 0);
}

#[test]
fn test_load_review_policy_with_checks() {
    let engine = load_review_policy(
        r#"
        define check_no_unwrap(source) =
            if contains(source, ".unwrap()") then "fail: unwrap" else "pass"
        define check_no_panic(source) =
            if contains(source, "panic!(") then "fail: panic" else "pass"
    "#,
    );
    let stats = engine.stats();
    assert_eq!(*stats.get("check_functions").unwrap(), 2);
}

// =============================================================================
// check_code — Pass/Fail
// =============================================================================

#[test]
fn test_check_code_clean_passes() {
    let engine = load_review_policy(
        r#"
        define check_no_unwrap(source) =
            if contains(source, ".unwrap()") then "fail: unwrap" else "pass"
    "#,
    );

    let result = engine.check_code("fn main() { let x = foo()?; }", "rust");
    assert!(result.passed);
    assert_eq!(result.verdicts.len(), 1);
    assert!(result.verdicts[0].passed);
}

#[test]
fn test_check_code_unwrap_fails() {
    let engine = load_review_policy(
        r#"
        define check_no_unwrap(source) =
            if contains(source, ".unwrap()") then "fail: contains .unwrap()" else "pass"
    "#,
    );

    let result = engine.check_code("fn main() { let x = foo().unwrap(); }", "rust");
    assert!(!result.passed);
    assert_eq!(result.verdicts.len(), 1);
    assert!(!result.verdicts[0].passed);
    assert!(result.verdicts[0].message.contains(".unwrap()"));
}

#[test]
fn test_check_code_multiple_rules() {
    let engine = load_review_policy(
        r#"
        define check_no_unwrap(source) =
            if contains(source, ".unwrap()") then "fail: unwrap" else "pass"
        define check_no_unsafe(source) =
            if contains(source, "unsafe {") then "fail: unsafe" else "pass"
        define check_no_println(source) =
            if contains(source, "println!(") then "fail: println" else "pass"
    "#,
    );

    // Clean code — all pass
    let result = engine.check_code("fn add(a: i32, b: i32) -> i32 { a + b }", "rust");
    assert!(result.passed);
    assert_eq!(result.verdicts.len(), 3);
    assert!(result.verdicts.iter().all(|v| v.passed));

    // Code with unwrap — one fail
    let result = engine.check_code("fn main() { foo().unwrap(); }", "rust");
    assert!(!result.passed);
    let failed: Vec<_> = result.verdicts.iter().filter(|v| !v.passed).collect();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].rule_name, "check_no_unwrap");

    // Code with all violations
    let result = engine.check_code(
        r#"fn main() { unsafe { println!("hi"); foo().unwrap(); } }"#,
        "rust",
    );
    assert!(!result.passed);
    let failed: Vec<_> = result.verdicts.iter().filter(|v| !v.passed).collect();
    assert_eq!(failed.len(), 3);
}

#[test]
fn test_check_code_unsafe_detection() {
    let engine = load_review_policy(
        r#"
        define check_no_unsafe(source) =
            if contains(source, "unsafe {") then "fail: unsafe block"
            else if contains(source, "unsafe{") then "fail: unsafe block"
            else "pass"
    "#,
    );

    let result = engine.check_code("unsafe { ptr::write(p, val); }", "rust");
    assert!(!result.passed);

    let result = engine.check_code("fn safe_fn() { let x = 1; }", "rust");
    assert!(result.passed);
}

#[test]
fn test_check_code_security_rules() {
    let engine = load_review_policy(
        r#"
        define check_no_hardcoded_password(source) =
            if contains(source, "password =") then "fail: hardcoded password"
            else "pass"
        define check_no_hardcoded_secret(source) =
            if contains(source, "secret =") then "fail: hardcoded secret"
            else "pass"
    "#,
    );

    let result = engine.check_code("let password = get_secret();", "rust");
    assert!(!result.passed);

    let result = engine.check_code("let token = get_env();", "rust");
    assert!(result.passed);
}

#[test]
fn test_check_code_summary() {
    let engine = load_review_policy(
        r#"
        define check_a(source) = "pass"
        define check_b(source) = "pass"
        define check_c(source) = if contains(source, "bad") then "fail: bad" else "pass"
    "#,
    );

    let result = engine.check_code("good code", "rust");
    assert!(result.summary.contains("3 checks passed"));

    let result = engine.check_code("bad code", "rust");
    assert!(result.summary.contains("1 failed"));
    assert!(result.summary.contains("2 passed"));
}

// =============================================================================
// list_rules / explain_rule
// =============================================================================

#[test]
fn test_list_review_rules() {
    let engine = load_review_policy(
        r#"
        define check_no_unwrap(source) = "pass"
        define check_no_unsafe(source) = "pass"
        define helper_fn(x) = x
    "#,
    );

    let rules = engine.list_rules();
    assert_eq!(rules.len(), 3);
}

#[test]
fn test_explain_review_rule() {
    let engine = load_review_policy(
        r#"
        define check_no_unwrap(source) =
            if contains(source, ".unwrap()") then "fail: unwrap" else "pass"
    "#,
    );

    let rule = engine.explain_rule("check_no_unwrap");
    assert!(rule.is_some());
    assert_eq!(rule.unwrap().name, "check_no_unwrap");

    assert!(engine.explain_rule("nonexistent").is_none());
}

// =============================================================================
// describe_standards
// =============================================================================

#[test]
fn test_describe_standards() {
    let engine = load_review_policy(
        r#"
        define check_no_unwrap(source) =
            if contains(source, ".unwrap()") then "fail" else "pass"
        define helper(x) = x
    "#,
    );

    let schema = engine.describe_schema();

    let check_fns = schema.get("check_functions").unwrap().as_array().unwrap();
    assert_eq!(check_fns.len(), 1);

    let helper_fns = schema.get("helper_functions").unwrap().as_array().unwrap();
    assert_eq!(helper_fns.len(), 1);
}

// =============================================================================
// Real policy file — rust_review_policy.kleis
// =============================================================================

#[test]
fn test_load_real_rust_review_policy() {
    let path = PathBuf::from("examples/policies/rust_review_policy.kleis");
    if !path.exists() {
        eprintln!("Skipping: rust_review_policy.kleis not found");
        return;
    }

    let engine = ReviewEngine::load(&path).expect("load rust_review_policy.kleis");
    let stats = engine.stats();

    assert!(
        *stats.get("check_functions").unwrap() >= 23,
        "Expected >= 23 check functions, got {}",
        stats.get("check_functions").unwrap()
    );

    // Clean code should pass all checks
    let result = engine.check_code("fn add(a: i32, b: i32) -> i32 { a + b }", "rust");
    assert!(
        result.passed,
        "Clean code should pass all checks. Failures: {:?}",
        result
            .verdicts
            .iter()
            .filter(|v| !v.passed)
            .collect::<Vec<_>>()
    );

    // Code with .unwrap() should fail
    let result = engine.check_code("fn main() { foo().unwrap(); }", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_no_unwrap" && !v.passed));

    // Code with unsafe should fail
    let result = engine.check_code("unsafe { *ptr = 1; }", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_no_unsafe" && !v.passed));

    // Code with println! should fail
    let result = engine.check_code("fn main() { println!(\"hello\"); }", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_no_println" && !v.passed));

    // Code with panic! should fail
    let result = engine.check_code("fn main() { panic!(\"oops\"); }", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_no_panic" && !v.passed));

    // Code with todo! should fail
    let result = engine.check_code("fn main() { todo!(\"later\"); }", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_no_todo" && !v.passed));

    // Code with dbg! should fail
    let result = engine.check_code("fn main() { dbg!(x); }", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_no_dbg" && !v.passed));

    // Code with hardcoded password should fail
    let result = engine.check_code("let password = get_secret();", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_no_hardcoded_password" && !v.passed));

    // Code with wildcard import should fail
    let result = engine.check_code("use std::collections::*;", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_no_wildcard_import" && !v.passed));

    // Code with #[allow(unused)] should fail
    let result = engine.check_code("#[allow(dead_code)]\nfn old() {}", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_no_allow_unused" && !v.passed));

    // Code with inline use statement should fail
    let result = engine.check_code("fn main() { use std::io; }", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_no_inline_use" && !v.passed));

    // Narrating comments should fail
    let result = engine.check_code("// Import the module\nuse std::io;", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_no_narrating_comments" && !v.passed));

    // Result<_, String> should fail
    let result = engine.check_code("fn load() -> Result<(), String> { Ok(()) }", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_no_result_string" && !v.passed));

    // Clippy suppression should fail
    let result = engine.check_code("#[allow(clippy::needless_return)]\nfn f() {}", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_no_clippy_suppression" && !v.passed));

    // Too many arguments suppression should fail
    let result = engine.check_code("#[allow(clippy::too_many_arguments)]\nfn f() {}", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_no_too_many_arguments" && !v.passed));

    // --- Clippy -D warnings patterns ---

    // &String should fail (clippy::ptr_arg)
    let result = engine.check_code("fn greet(name: &String) {}", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_clippy_ptr_arg" && !v.passed));

    // &Vec<T> should fail (clippy::ptr_arg)
    let result = engine.check_code("fn sum(items: &Vec<i32>) -> i32 { 0 }", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_clippy_ptr_arg" && !v.passed));

    // .len() == 0 should fail (clippy::len_zero)
    let result = engine.check_code("if v.len() == 0 { return; }", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_clippy_len_zero" && !v.passed));

    // .len() > 0 should fail (clippy::len_zero)
    let result = engine.check_code("if v.len() > 0 { process(); }", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_clippy_len_zero" && !v.passed));

    // == true should fail (clippy::bool_comparison)
    let result = engine.check_code("if flag == true { run(); }", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_clippy_bool_comparison" && !v.passed));

    // == false should fail (clippy::bool_comparison)
    let result = engine.check_code("if done == false { continue; }", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_clippy_bool_comparison" && !v.passed));

    // .to_string().as_str() should fail (redundant clone)
    let result = engine.check_code("let s = name.to_string().as_str();", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_clippy_redundant_clone" && !v.passed));

    // .expect(&format!()) should fail (clippy::expect_fun_call)
    let result = engine.check_code(
        "let x = m.get(k).expect(&format!(\"missing {}\", k));",
        "rust",
    );
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_clippy_expect_fun_call" && !v.passed));

    // --- No emoji ---

    // Emoji in string literal should fail
    let result = engine.check_code("let msg = \"Done ✅\";", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_no_emoji" && !v.passed));

    // Emoji in comment should fail
    let result = engine.check_code("// TODO: fix this 🐛\nfn f() {}", "rust");
    assert!(!result.passed);
    assert!(result
        .verdicts
        .iter()
        .any(|v| v.rule_name == "check_no_emoji" && !v.passed));

    // Plain text should pass
    let result_clean = engine.check_code("let msg = \"Done\";", "rust");
    assert!(
        !result_clean
            .verdicts
            .iter()
            .any(|v| v.rule_name == "check_no_emoji" && !v.passed),
        "Plain text should pass check_no_emoji"
    );
}

// ==========================================================================
// Z3 Verification Tests
// ==========================================================================

#[test]
fn test_evaluate_concrete_expression() {
    let path = PathBuf::from("examples/policies/rust_review_policy.kleis");
    if !path.exists() {
        return;
    }
    let engine = ReviewEngine::load(&path).expect("load policy");

    let result = engine.evaluate_expression("1 + 2");
    assert!(result.error.is_none(), "Should evaluate without error");
    assert!(result.value.is_some(), "Should produce a value");
    assert_eq!(result.value.as_deref(), Some("3"));
}

#[test]
fn test_evaluate_string_function() {
    let path = PathBuf::from("examples/policies/rust_review_policy.kleis");
    if !path.exists() {
        return;
    }
    let engine = ReviewEngine::load(&path).expect("load policy");

    let result = engine.evaluate_expression("contains(\"hello world\", \"world\")");
    assert!(result.error.is_none(), "Should evaluate without error");
    assert!(
        result.value.is_some(),
        "Should return a value for contains()"
    );
}

#[test]
fn test_evaluate_check_function() {
    let path = PathBuf::from("examples/policies/rust_review_policy.kleis");
    if !path.exists() {
        return;
    }
    let engine = ReviewEngine::load(&path).expect("load policy");

    let result = engine.evaluate_expression("check_no_unwrap(\"fn f() { a + b }\")");
    assert!(result.error.is_none(), "Should evaluate without error");
    assert_eq!(result.value.as_deref(), Some("pass"));

    let result = engine.evaluate_expression("check_no_unwrap(\"x.unwrap()\")");
    assert!(result.error.is_none());
    let val = result.value.unwrap();
    assert!(val.starts_with("fail"), "Should fail: {}", val);
}

#[cfg(feature = "axiom-verification")]
#[test]
fn test_evaluate_z3_proposition() {
    let path = PathBuf::from("examples/policies/rust_review_policy.kleis");
    if !path.exists() {
        return;
    }
    let engine = ReviewEngine::load(&path).expect("load policy");

    let result = engine
        .evaluate_expression("∀(s : String). implies(is_safe(s), not(contains(s, \".unwrap()\")))");
    assert!(
        result.error.is_none(),
        "Z3 proposition should not error: {:?}",
        result.error
    );
    assert_eq!(
        result.verified,
        Some(true),
        "safe_no_unwrap axiom should be verified by Z3"
    );
}

#[cfg(feature = "axiom-verification")]
#[test]
fn test_evaluate_z3_safe_no_panic() {
    let path = PathBuf::from("examples/policies/rust_review_policy.kleis");
    if !path.exists() {
        return;
    }
    let engine = ReviewEngine::load(&path).expect("load policy");

    let result = engine
        .evaluate_expression("∀(s : String). implies(is_safe(s), not(contains(s, \"panic!(\")))");
    assert!(
        result.error.is_none(),
        "Z3 should not error: {:?}",
        result.error
    );
    assert_eq!(
        result.verified,
        Some(true),
        "safe_no_panic axiom should be verified"
    );
}

#[cfg(feature = "axiom-verification")]
#[test]
fn test_evaluate_z3_clean_no_println() {
    let path = PathBuf::from("examples/policies/rust_review_policy.kleis");
    if !path.exists() {
        return;
    }
    let engine = ReviewEngine::load(&path).expect("load policy");

    let result = engine.evaluate_expression(
        "∀(s : String). implies(is_clean(s), not(contains(s, \"println!(\" )))",
    );
    assert!(
        result.error.is_none(),
        "Z3 should not error: {:?}",
        result.error
    );
    assert_eq!(
        result.verified,
        Some(true),
        "clean_no_println axiom should be verified"
    );
}

#[test]
fn test_existing_check_code_still_works() {
    let path = PathBuf::from("examples/policies/rust_review_policy.kleis");
    if !path.exists() {
        return;
    }
    let engine = ReviewEngine::load(&path).expect("load policy");

    let result = engine.check_code("fn add(a: i32, b: i32) -> i32 { a + b }", "rust");
    assert!(
        result.passed,
        "Clean code should still pass all checks after adding structures"
    );
}

#[test]
fn test_structures_loaded_in_policy() {
    let path = PathBuf::from("examples/policies/rust_review_policy.kleis");
    if !path.exists() {
        return;
    }
    let engine = ReviewEngine::load(&path).expect("load policy");
    let schema = engine.describe_schema();

    let structures = schema["structures"].as_array().expect("structures array");
    assert!(
        structures.len() >= 4,
        "Should have at least 4 structures (SafeCode, CleanCode, SecureCode, SqlSafe), got {}",
        structures.len()
    );

    let names: Vec<&str> = structures
        .iter()
        .filter_map(|s| s["name"].as_str())
        .collect();
    assert!(names.contains(&"SafeCode"), "Missing SafeCode structure");
    assert!(names.contains(&"CleanCode"), "Missing CleanCode structure");
    assert!(
        names.contains(&"SecureCode"),
        "Missing SecureCode structure"
    );
    assert!(names.contains(&"SqlSafe"), "Missing SqlSafe structure");
}

#[test]
fn test_check_no_sql_injection() {
    let path = PathBuf::from("examples/policies/rust_review_policy.kleis");
    if !path.exists() {
        return;
    }
    let engine = ReviewEngine::load(&path).expect("load policy");

    let bad_format = engine.check_code(
        "let q = format!(\"SELECT * FROM users WHERE id = {}\", id);",
        "rust",
    );
    assert!(
        bad_format
            .verdicts
            .iter()
            .any(|v| v.rule_name == "check_no_sql_injection" && !v.passed),
        "format! with SELECT should fail"
    );

    let bad_insert = engine.check_code(
        "let q = format!(\"INSERT INTO logs VALUES({})\", val);",
        "rust",
    );
    assert!(
        bad_insert
            .verdicts
            .iter()
            .any(|v| v.rule_name == "check_no_sql_injection" && !v.passed),
        "format! with INSERT should fail"
    );

    let safe_parameterized =
        engine.check_code("let row = sqlx::query!(\"SELECT id FROM users WHERE name = $1\", name).fetch_one(&pool).await?;", "rust");
    assert!(
        !safe_parameterized
            .verdicts
            .iter()
            .any(|v| v.rule_name == "check_no_sql_injection" && !v.passed),
        "Parameterized query should pass"
    );

    let clean_code = engine.check_code("fn add(a: i32, b: i32) -> i32 { a + b }", "rust");
    assert!(
        !clean_code
            .verdicts
            .iter()
            .any(|v| v.rule_name == "check_no_sql_injection" && !v.passed),
        "Code without SQL should pass"
    );
}

#[cfg(feature = "axiom-verification")]
#[test]
fn test_evaluate_z3_sql_safe_taint_property() {
    let path = PathBuf::from("examples/policies/rust_review_policy.kleis");
    if !path.exists() {
        return;
    }
    let engine = ReviewEngine::load(&path).expect("load policy");

    let result = engine.evaluate_expression(
        "∀(s : String). implies(and(is_tainted(s), reaches_query(s)), is_sanitized(s))",
    );
    assert!(
        result.error.is_none(),
        "Z3 should not error: {:?}",
        result.error
    );
    assert_eq!(
        result.verified,
        Some(true),
        "no_tainted_query axiom should be verified"
    );
}
