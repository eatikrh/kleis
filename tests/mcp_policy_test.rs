//! Integration tests for the MCP Policy Engine
//!
//! Tests the full pipeline:
//! - Policy loading and parsing
//! - check_action (allow/deny decisions)
//! - evaluate_expression (concrete evaluation + proposition verification)
//! - describe_schema (AST introspection)
//! - Preconditions (before_* functions)

use kleis::mcp::policy::PolicyEngine;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique counter so parallel tests never share the same temp file
static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Helper: write a policy string to a temp file and load a PolicyEngine
fn load_policy(source: &str) -> PolicyEngine {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "kleis_test_policy_{}_{}.kleis",
        std::process::id(),
        id
    ));
    std::fs::write(&path, source).expect("write temp policy file");
    let engine = PolicyEngine::load(&path).expect("load policy");
    let _ = std::fs::remove_file(&path); // cleanup
    engine
}

// =============================================================================
// Policy Loading
// =============================================================================

#[test]
fn test_load_empty_policy() {
    let engine = load_policy("// empty policy\n");
    let stats = engine.stats();
    assert_eq!(*stats.get("total_rules").unwrap(), 0);
    assert_eq!(*stats.get("functions").unwrap(), 0);
}

#[test]
fn test_load_policy_with_check_functions() {
    let engine = load_policy(
        r#"
        define check_file_delete(path) =
            if hasPrefix(path, "src/") then "deny" else "allow"

        define check_run_command(cmd) =
            if contains(cmd, "rm -rf") then "deny" else "allow"
    "#,
    );
    let stats = engine.stats();
    assert_eq!(*stats.get("check_functions").unwrap(), 2);
    assert_eq!(*stats.get("functions").unwrap(), 2);
}

#[test]
fn test_load_policy_with_preconditions() {
    let engine = load_policy(
        r#"
        define before_git_push(branch, force) = "cargo test"
        define before_file_edit(path) = "none"
    "#,
    );
    let stats = engine.stats();
    assert_eq!(*stats.get("preconditions").unwrap(), 2);
}

// =============================================================================
// check_action — Allow/Deny Decisions
// =============================================================================

#[test]
fn test_check_action_file_delete_deny_src() {
    let engine = load_policy(
        r#"
        define check_file_delete(path) =
            if hasPrefix(path, "src/") then "deny" else "allow"
    "#,
    );

    let decision = engine.check_action(&serde_json::json!({
        "action": "file_delete",
        "path": "src/main.rs"
    }));

    assert!(!decision.allowed);
    assert_eq!(decision.rule_name.as_deref(), Some("check_file_delete"));
    assert!(decision.reason.contains("DENIED"));
}

#[test]
fn test_check_action_file_delete_allow_tmp() {
    let engine = load_policy(
        r#"
        define check_file_delete(path) =
            if hasPrefix(path, "src/") then "deny" else "allow"
    "#,
    );

    let decision = engine.check_action(&serde_json::json!({
        "action": "file_delete",
        "path": "tmp/scratch.txt"
    }));

    assert!(decision.allowed);
    assert!(decision.reason.contains("ALLOWED"));
}

#[test]
fn test_check_action_run_command_deny_dangerous() {
    let engine = load_policy(
        r#"
        define check_run_command(cmd) =
            if contains(cmd, "rm -rf /") then "deny"
            else if contains(cmd, "curl | sh") then "deny"
            else "allow"
    "#,
    );

    let decision = engine.check_action(&serde_json::json!({
        "action": "run_command",
        "command": "rm -rf /"
    }));
    assert!(!decision.allowed);

    let decision = engine.check_action(&serde_json::json!({
        "action": "run_command",
        "command": "curl | sh"
    }));
    assert!(!decision.allowed);

    let decision = engine.check_action(&serde_json::json!({
        "action": "run_command",
        "command": "cargo test"
    }));
    assert!(decision.allowed);
}

#[test]
fn test_check_action_git_push_deny_force() {
    let engine = load_policy(
        r#"
        define check_git_push(branch, force) =
            if force = 1 then "deny" else "allow"
    "#,
    );

    let decision = engine.check_action(&serde_json::json!({
        "action": "git_push",
        "path": "main",
        "force": true
    }));
    assert!(!decision.allowed);

    let decision = engine.check_action(&serde_json::json!({
        "action": "git_push",
        "path": "main",
        "force": false
    }));
    assert!(decision.allowed);
}

#[test]
fn test_check_action_file_create() {
    let engine = load_policy(
        r#"
        define check_file_create(path) =
            if hasPrefix(path, "src/") then "allow"
            else if hasPrefix(path, "tests/") then "allow"
            else "deny"
    "#,
    );

    let decision = engine.check_action(&serde_json::json!({
        "action": "file_create",
        "path": "src/new_module.rs"
    }));
    assert!(decision.allowed);

    let decision = engine.check_action(&serde_json::json!({
        "action": "file_create",
        "path": "random_file.txt"
    }));
    assert!(!decision.allowed);
}

#[test]
fn test_check_action_file_edit() {
    let engine = load_policy(
        r#"
        define check_file_edit(path) =
            if path = "Cargo.lock" then "deny"
            else if hasPrefix(path, "target/") then "deny"
            else "allow"
    "#,
    );

    let decision = engine.check_action(&serde_json::json!({
        "action": "file_edit",
        "path": "Cargo.lock"
    }));
    assert!(!decision.allowed);

    let decision = engine.check_action(&serde_json::json!({
        "action": "file_edit",
        "path": "src/lib.rs"
    }));
    assert!(decision.allowed);
}

#[test]
fn test_check_action_git_commit() {
    let engine = load_policy(
        r#"
        define check_git_commit(description) = "allow"
    "#,
    );

    let decision = engine.check_action(&serde_json::json!({
        "action": "git_commit",
        "description": "fix bug"
    }));
    assert!(decision.allowed);
}

#[test]
fn test_check_action_unknown_type_defaults_deny() {
    let engine = load_policy("// no rules\n");

    let decision = engine.check_action(&serde_json::json!({
        "action": "unknown_action"
    }));
    assert!(!decision.allowed);
    assert!(decision.reason.contains("Unknown action type"));
}

#[test]
fn test_check_action_no_rule_defaults_allow() {
    let engine = load_policy("// no check_file_edit defined\n");

    let decision = engine.check_action(&serde_json::json!({
        "action": "file_edit",
        "path": "README.md"
    }));
    // No rule defined → open policy → allow
    assert!(decision.allowed);
    assert!(decision.reason.contains("allowed by default"));
}

// =============================================================================
// Preconditions (before_* functions)
// =============================================================================

#[test]
fn test_preconditions_before_git_push() {
    let engine = load_policy(
        r#"
        define check_git_push(branch, force) = "allow"
        define before_git_push(branch, force) = "cargo test"
    "#,
    );

    let decision = engine.check_action(&serde_json::json!({
        "action": "git_push",
        "path": "main",
        "force": false
    }));

    assert!(decision.allowed);
    assert_eq!(decision.preconditions, vec!["cargo test"]);
}

#[test]
fn test_preconditions_multiple_steps() {
    let engine = load_policy(
        r#"
        define check_git_push(branch, force) = "allow"
        define before_git_push(branch, force) = "cargo fmt && cargo test"
    "#,
    );

    let decision = engine.check_action(&serde_json::json!({
        "action": "git_push",
        "path": "main",
        "force": false
    }));

    assert_eq!(decision.preconditions, vec!["cargo fmt", "cargo test"]);
}

#[test]
fn test_preconditions_none_is_empty() {
    let engine = load_policy(
        r#"
        define check_file_edit(path) = "allow"
        define before_file_edit(path) = "none"
    "#,
    );

    let decision = engine.check_action(&serde_json::json!({
        "action": "file_edit",
        "path": "README.md"
    }));

    assert!(decision.preconditions.is_empty());
}

#[test]
fn test_preconditions_conditional() {
    let engine = load_policy(
        r#"
        define check_file_edit(path) = "allow"
        define before_file_edit(path) =
            if hasPrefix(path, "src/parser") then "cargo test parser"
            else if hasPrefix(path, "src/evaluator") then "cargo test evaluator"
            else "none"
    "#,
    );

    let decision = engine.check_action(&serde_json::json!({
        "action": "file_edit",
        "path": "src/parser/mod.rs"
    }));
    assert_eq!(decision.preconditions, vec!["cargo test parser"]);

    let decision = engine.check_action(&serde_json::json!({
        "action": "file_edit",
        "path": "src/evaluator.rs"
    }));
    assert_eq!(decision.preconditions, vec!["cargo test evaluator"]);

    let decision = engine.check_action(&serde_json::json!({
        "action": "file_edit",
        "path": "README.md"
    }));
    assert!(decision.preconditions.is_empty());
}

#[test]
fn test_preconditions_with_denied_action() {
    // Preconditions should still be returned even when action is denied
    let engine = load_policy(
        r#"
        define check_git_push(branch, force) =
            if force = 1 then "deny" else "allow"
        define before_git_push(branch, force) = "cargo test"
    "#,
    );

    let decision = engine.check_action(&serde_json::json!({
        "action": "git_push",
        "path": "main",
        "force": true
    }));

    assert!(!decision.allowed);
    // Preconditions are evaluated regardless of allow/deny
    assert_eq!(decision.preconditions, vec!["cargo test"]);
}

// =============================================================================
// evaluate_expression — Concrete Evaluation
// =============================================================================

#[test]
fn test_evaluate_concrete_function_call() {
    let engine = load_policy(
        r#"
        define check_file_delete(path) =
            if hasPrefix(path, "src/") then "deny" else "allow"
    "#,
    );

    let result = engine.evaluate_expression(r#"check_file_delete("src/main.rs")"#);
    assert!(
        result.error.is_none(),
        "unexpected error: {:?}",
        result.error
    );
    assert_eq!(result.value.as_deref(), Some("deny"));
    assert!(result.verified.is_none()); // not a proposition
}

#[test]
fn test_evaluate_concrete_allow() {
    let engine = load_policy(
        r#"
        define check_file_delete(path) =
            if hasPrefix(path, "src/") then "deny" else "allow"
    "#,
    );

    let result = engine.evaluate_expression(r#"check_file_delete("tmp/junk.txt")"#);
    assert_eq!(result.value.as_deref(), Some("allow"));
}

#[test]
fn test_evaluate_precondition_function() {
    let engine = load_policy(
        r#"
        define before_git_push(branch, force) =
            "cargo fmt && cargo test"
    "#,
    );

    let result = engine.evaluate_expression(r#"before_git_push("main", 0)"#);
    assert_eq!(result.value.as_deref(), Some("cargo fmt && cargo test"));
}

#[test]
fn test_evaluate_arithmetic() {
    let engine = load_policy("// no functions needed\n");
    let result = engine.evaluate_expression("2 + 3");
    assert_eq!(result.value.as_deref(), Some("5"));
}

#[test]
fn test_evaluate_parse_error() {
    let engine = load_policy("// no functions\n");
    let result = engine.evaluate_expression("define (((");
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("Parse error"));
}

#[test]
fn test_evaluate_undefined_function() {
    let engine = load_policy("// no functions\n");
    let result = engine.evaluate_expression(r#"nonexistent("arg")"#);
    // Should either return a symbolic result or an error, not panic
    // The exact behavior depends on eval_concrete for unknown functions
    assert!(result.value.is_some() || result.error.is_some());
}

// =============================================================================
// evaluate_expression — Proposition Verification (routes to Kleis assert → Z3)
// =============================================================================

#[test]
#[cfg(feature = "axiom-verification")]
fn test_evaluate_quantified_proposition() {
    // ∀(x : ℝ). x + 0 = x — basic arithmetic tautology
    let engine = load_policy("// no functions needed\n");
    let result = engine.evaluate_expression("∀(x : ℝ). x + 0 = x");

    // Should be routed through verify_proposition → Z3
    // The evaluator detects the ∀ quantifier and sends it to Z3
    assert!(
        result.error.is_none(),
        "Unexpected error: {:?}",
        result.error
    );
    assert!(
        result.verified == Some(true),
        "Expected verified=true, got: {:?}",
        result
    );
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_evaluate_quantified_proposition_with_policy_function() {
    // Verify a property about the policy using the same assert pipeline
    let engine = load_policy(
        r#"
        define check_git_push(branch, force) =
            if force = 1 then "deny" else "allow"
    "#,
    );

    // Ask: is force-push always denied?
    let result = engine.evaluate_expression(r#"∀(b : String). check_git_push(b, 1) = "deny""#);

    // The pipeline should not error — it goes through eval_assert → Z3
    assert!(
        result.error.is_none(),
        "Unexpected error: {:?}",
        result.error
    );
}

// =============================================================================
// describe_schema — AST Introspection
// =============================================================================

#[test]
fn test_describe_schema_functions() {
    let engine = load_policy(
        r#"
        define check_file_delete(path) = "allow"
        define check_run_command(cmd) = "allow"
        define before_git_push(branch, force) = "cargo test"
        define helper_fn(x) = x
    "#,
    );

    let schema = engine.describe_schema();

    // Check functions are categorized
    let check_fns = schema.get("check_functions").unwrap().as_array().unwrap();
    assert_eq!(check_fns.len(), 2);

    let before_fns = schema
        .get("precondition_functions")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(before_fns.len(), 1);

    let helper_fns = schema.get("helper_functions").unwrap().as_array().unwrap();
    assert_eq!(helper_fns.len(), 1);

    // Stats
    let stats = schema.get("stats").unwrap();
    assert_eq!(stats.get("functions").unwrap().as_u64().unwrap(), 4);
}

#[test]
fn test_describe_schema_with_structures() {
    let engine = load_policy(
        r#"
        structure Monoid(M) {
            operation unit : M
            operation combine : M × M → M
            axiom left_identity : ∀(x : M). combine(unit, x) = x
        }

        define check_file_edit(path) = "allow"
    "#,
    );

    let schema = engine.describe_schema();

    let structures = schema.get("structures").unwrap().as_array().unwrap();
    assert_eq!(structures.len(), 1);

    let monoid = &structures[0];
    assert_eq!(monoid.get("name").unwrap().as_str().unwrap(), "Monoid");

    let axioms = monoid.get("axioms").unwrap().as_array().unwrap();
    assert_eq!(axioms.len(), 1);
    assert_eq!(
        axioms[0].get("name").unwrap().as_str().unwrap(),
        "left_identity"
    );

    let operations = monoid.get("operations").unwrap().as_array().unwrap();
    assert_eq!(operations.len(), 2);

    // Stats should reflect the structure and axiom
    let stats = schema.get("stats").unwrap();
    assert_eq!(stats.get("structures").unwrap().as_u64().unwrap(), 1);
    assert_eq!(stats.get("axioms").unwrap().as_u64().unwrap(), 1);
}

#[test]
fn test_describe_schema_with_data_types() {
    let engine = load_policy(
        r#"
        data Action = FileEdit | FileDelete | RunCommand | GitPush

        define check_file_edit(path) = "allow"
    "#,
    );

    let schema = engine.describe_schema();

    let data_types = schema.get("data_types").unwrap().as_array().unwrap();
    assert_eq!(data_types.len(), 1);

    let action = &data_types[0];
    assert_eq!(action.get("name").unwrap().as_str().unwrap(), "Action");

    let variants = action.get("variants").unwrap().as_array().unwrap();
    assert_eq!(variants.len(), 4);
}

#[test]
fn test_describe_schema_function_params() {
    let engine = load_policy(
        r#"
        define check_git_push(branch, force) = "allow"
    "#,
    );

    let schema = engine.describe_schema();
    let check_fns = schema.get("check_functions").unwrap().as_array().unwrap();
    let push_fn = &check_fns[0];

    let params = push_fn.get("params").unwrap().as_array().unwrap();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].as_str().unwrap(), "branch");
    assert_eq!(params[1].as_str().unwrap(), "force");
}

#[test]
fn test_describe_schema_empty_policy() {
    let engine = load_policy("// empty\n");
    let schema = engine.describe_schema();

    assert_eq!(
        schema.get("structures").unwrap().as_array().unwrap().len(),
        0
    );
    assert_eq!(
        schema.get("data_types").unwrap().as_array().unwrap().len(),
        0
    );
    assert_eq!(
        schema
            .get("check_functions")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

// =============================================================================
// describe_schema — Kleis syntax rendering + verifiable propositions
// =============================================================================

#[test]
fn test_describe_schema_function_bodies_in_kleis_syntax() {
    let engine = load_policy(
        r#"
        define check_file_delete(path) =
            if hasPrefix(path, "src/") then "deny" else "allow"
    "#,
    );

    let schema = engine.describe_schema();
    let check_fns = schema.get("check_functions").unwrap().as_array().unwrap();
    let f = &check_fns[0];

    // Should have a "kleis" field with the full function definition
    let kleis = f
        .get("kleis")
        .and_then(|k| k.as_str())
        .expect("missing kleis field");
    assert!(
        kleis.contains("define check_file_delete(path)"),
        "kleis={}",
        kleis
    );
    assert!(kleis.contains("hasPrefix"), "kleis={}", kleis);

    // Should have a "body" field with just the body expression
    let body = f
        .get("body")
        .and_then(|b| b.as_str())
        .expect("missing body field");
    assert!(body.contains("hasPrefix"), "body={}", body);
}

#[test]
fn test_describe_schema_axioms_in_kleis_syntax() {
    let engine = load_policy(
        r#"
        structure Monoid(M) {
            operation unit : M
            operation combine : M × M → M
            axiom left_identity : ∀(x : M). combine(unit, x) = x
        }
    "#,
    );

    let schema = engine.describe_schema();
    let structures = schema.get("structures").unwrap().as_array().unwrap();
    let axioms = structures[0].get("axioms").unwrap().as_array().unwrap();

    // The "kleis" field should contain readable Kleis syntax, not Rust Debug format
    let kleis = axioms[0]
        .get("kleis")
        .and_then(|k| k.as_str())
        .expect("missing kleis field");
    assert!(
        !kleis.contains("Expression::"),
        "Should not contain Rust Debug format: {}",
        kleis
    );
    assert!(
        kleis.contains("combine"),
        "Should contain 'combine': {}",
        kleis
    );
}

#[test]
fn test_describe_schema_verifiable_propositions() {
    let engine = load_policy(
        r#"
        define check_file_delete(path) =
            if hasPrefix(path, "src/") then "deny" else "allow"
        define check_git_push(branch, force) = "allow"
    "#,
    );

    let schema = engine.describe_schema();
    let props = schema
        .get("verifiable_propositions")
        .unwrap()
        .as_array()
        .unwrap();

    // Should have generated propositions from check_* functions
    assert!(!props.is_empty(), "Should have verifiable propositions");

    // Each proposition should have kleis, description, and hint
    for p in props {
        assert!(p.get("kleis").is_some(), "missing kleis: {:?}", p);
        assert!(
            p.get("description").is_some(),
            "missing description: {:?}",
            p
        );
        assert!(p.get("hint").is_some(), "missing hint: {:?}", p);
    }

    // Should include ∀-quantified propositions for check functions
    let kleis_texts: Vec<&str> = props
        .iter()
        .filter_map(|p| p.get("kleis").and_then(|k| k.as_str()))
        .collect();
    assert!(
        kleis_texts
            .iter()
            .any(|t| t.contains("∀") && t.contains("check_file_delete")),
        "Should have ∀ proposition for check_file_delete: {:?}",
        kleis_texts
    );
    assert!(
        kleis_texts
            .iter()
            .any(|t| t.contains("∀") && t.contains("check_git_push")),
        "Should have ∀ proposition for check_git_push: {:?}",
        kleis_texts
    );

    // Should include concrete spot-check examples
    assert!(
        kleis_texts
            .iter()
            .any(|t| !t.contains("∀") && t.contains("check_file_delete")),
        "Should have concrete example for check_file_delete: {:?}",
        kleis_texts
    );
}

#[test]
fn test_describe_schema_verifiable_propositions_from_axioms() {
    let engine = load_policy(
        r#"
        structure Group(G) {
            operation identity : G
            operation compose : G × G → G
            axiom left_identity : ∀(x : G). compose(identity, x) = x
        }
        define check_file_edit(path) = "allow"
    "#,
    );

    let schema = engine.describe_schema();
    let props = schema
        .get("verifiable_propositions")
        .unwrap()
        .as_array()
        .unwrap();

    // Should include propositions inferred from structure axioms
    let kleis_texts: Vec<&str> = props
        .iter()
        .filter_map(|p| p.get("kleis").and_then(|k| k.as_str()))
        .collect();
    // PrettyPrinter may render compose(a, b) as "a ∘ b" or as "compose(a, b)"
    assert!(
        kleis_texts
            .iter()
            .any(|t| t.contains("compose") || t.contains("∘") || t.contains("identity")),
        "Should have proposition from Group axiom: {:?}",
        kleis_texts
    );
}

#[test]
fn test_synthesized_proposition_is_evaluable() {
    // The agent workflow: describe_schema → pick a proposition → evaluate it
    let engine = load_policy(
        r#"
        define check_git_push(branch, force) =
            if force = 1 then "deny" else "allow"
    "#,
    );

    let schema = engine.describe_schema();
    let props = schema
        .get("verifiable_propositions")
        .unwrap()
        .as_array()
        .unwrap();

    // Find the concrete spot-check
    let concrete = props
        .iter()
        .find(|p| {
            p.get("hint").and_then(|h| h.as_str()) == Some("evaluate")
                && p.get("kleis")
                    .and_then(|k| k.as_str())
                    .is_some_and(|k| k.contains("check_git_push"))
        })
        .expect("should have concrete evaluate example");

    let kleis_str = concrete.get("kleis").unwrap().as_str().unwrap();

    // Actually evaluate it — the synthesized expression should be parseable and evaluable
    let result = engine.evaluate_expression(kleis_str);
    assert!(
        result.error.is_none(),
        "Synthesized proposition '{}' failed: {:?}",
        kleis_str,
        result.error
    );
    assert!(
        result.value.is_some(),
        "Synthesized proposition '{}' returned no value",
        kleis_str
    );
}

// =============================================================================
// list_rules and explain_rule
// =============================================================================

#[test]
fn test_list_rules() {
    let engine = load_policy(
        r#"
        define check_file_delete(path) = "allow"
        define before_git_push(branch, force) = "cargo test"
    "#,
    );

    let rules = engine.list_rules();
    assert_eq!(rules.len(), 2);
}

#[test]
fn test_explain_rule_found() {
    let engine = load_policy(
        r#"
        define check_file_delete(path) = "allow"
    "#,
    );

    let rule = engine.explain_rule("check_file_delete");
    assert!(rule.is_some());
    assert_eq!(rule.unwrap().name, "check_file_delete");
}

#[test]
fn test_explain_rule_not_found() {
    let engine = load_policy("// empty\n");
    assert!(engine.explain_rule("nonexistent").is_none());
}

// =============================================================================
// Real policy file — agent_policy.kleis
// =============================================================================

#[test]
fn test_load_real_agent_policy() {
    let path = PathBuf::from("examples/policies/agent_policy.kleis");
    if !path.exists() {
        eprintln!("Skipping: agent_policy.kleis not found");
        return;
    }

    let engine = PolicyEngine::load(&path).expect("load agent_policy.kleis");
    let stats = engine.stats();

    // Should have multiple check and before functions
    assert!(
        *stats.get("check_functions").unwrap() >= 5,
        "Expected >= 5 check functions, got {}",
        stats.get("check_functions").unwrap()
    );
    assert!(
        *stats.get("preconditions").unwrap() >= 4,
        "Expected >= 4 preconditions, got {}",
        stats.get("preconditions").unwrap()
    );

    // Test specific rules from the real policy
    let decision = engine.check_action(&serde_json::json!({
        "action": "file_delete",
        "path": "src/main.rs"
    }));
    assert!(!decision.allowed, "Should deny deleting src/ files");

    let decision = engine.check_action(&serde_json::json!({
        "action": "git_push",
        "path": "main",
        "force": true
    }));
    assert!(!decision.allowed, "Should deny force-push");

    let decision = engine.check_action(&serde_json::json!({
        "action": "run_command",
        "command": "cargo test"
    }));
    assert!(decision.allowed, "Should allow cargo test");

    let decision = engine.check_action(&serde_json::json!({
        "action": "run_command",
        "command": "rm -rf /"
    }));
    assert!(!decision.allowed, "Should deny rm -rf /");

    // Test preconditions
    let decision = engine.check_action(&serde_json::json!({
        "action": "git_push",
        "path": "main",
        "force": false
    }));
    assert!(decision.allowed);
    assert!(
        !decision.preconditions.is_empty(),
        "Should have preconditions for git push"
    );

    // Test evaluate
    let result = engine.evaluate_expression(r#"check_file_delete("Cargo.toml")"#);
    assert_eq!(result.value.as_deref(), Some("deny"));

    let result = engine.evaluate_expression(r#"check_file_create("src/new.rs")"#);
    assert_eq!(result.value.as_deref(), Some("allow"));

    // Test describe_schema
    let schema = engine.describe_schema();
    assert!(
        schema
            .get("check_functions")
            .unwrap()
            .as_array()
            .unwrap()
            .len()
            >= 5
    );
    assert!(
        schema
            .get("precondition_functions")
            .unwrap()
            .as_array()
            .unwrap()
            .len()
            >= 4
    );
}
