//! Integration tests for the Theory MCP Engine (ADR-031)
//!
//! Tests the full pipeline:
//! - Engine creation with default prelude
//! - submit_structure (valid + syntax error)
//! - try_structure (dry run)
//! - evaluate expression and proposition
//! - describe_schema (prelude + agent structures)
//! - load_theory (new session)
//! - save_theory (persist to file)
//! - list_session (history tracking)

use kleis::config::TheoryConfig;
use kleis::theory_mcp::engine::TheoryEngine;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Create a TheoryEngine with unique temp directories for isolation.
fn create_engine() -> TheoryEngine {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let base =
        std::env::temp_dir().join(format!("kleis_theory_test_{}_{}", std::process::id(), id));
    let workspace = base.join("sessions");
    let save = base.join("theories");

    let config = TheoryConfig {
        workspace_dir: workspace.to_string_lossy().to_string(),
        save_dir: save.to_string_lossy().to_string(),
    };

    TheoryEngine::new(&config).expect("create theory engine")
}

/// Cleanup helper — remove temp dirs after test.
fn cleanup_engine_dirs(engine: &TheoryEngine) {
    // The engine's dirs are in the temp folder; they'll be cleaned up by the OS.
    // But we can be explicit for tidiness.
    let _ = engine; // just consume the reference
}

// =============================================================================
// Engine Creation
// =============================================================================

#[test]
fn test_engine_creation_with_prelude() {
    let engine = create_engine();
    let stats = engine.stats();

    // Prelude should load some functions
    assert!(
        *stats.get("functions").unwrap() > 0,
        "Prelude should load at least some functions, got {}",
        stats.get("functions").unwrap()
    );

    // Session should be empty (no agent submissions yet)
    assert_eq!(*stats.get("session_items").unwrap(), 0);

    cleanup_engine_dirs(&engine);
}

// =============================================================================
// Submit Structure
// =============================================================================

#[test]
fn test_submit_valid_structure() {
    let mut engine = create_engine();

    let result = engine.submit_kleis(
        r#"
        structure TestGroup(G) {
            operation identity : G
            operation compose : G -> G -> G
            axiom left_id : ∀(x : G). compose(identity, x) = x
        }
        "#,
    );

    assert!(
        result.accepted,
        "Should accept valid structure: {:?}",
        result.error
    );
    assert!(result.structures_added.contains(&"TestGroup".to_string()));
    assert!(result.error.is_none());

    // Session history should track it
    assert_eq!(engine.list_session().len(), 1);

    cleanup_engine_dirs(&engine);
}

#[test]
fn test_submit_structure_with_syntax_error() {
    let mut engine = create_engine();

    let result = engine.submit_kleis(
        r#"
        structure Broken {{{
            this is not valid kleis
        }
        "#,
    );

    assert!(!result.accepted, "Should reject syntax error");
    assert!(result.error.is_some());

    // Session should be unchanged
    assert_eq!(engine.list_session().len(), 0);

    cleanup_engine_dirs(&engine);
}

#[test]
fn test_submit_valid_define() {
    let mut engine = create_engine();

    let result = engine.submit_kleis("define square(x) = x * x");

    assert!(
        result.accepted,
        "Should accept valid define: {:?}",
        result.error
    );
    assert!(result.functions_added.contains(&"square".to_string()));

    cleanup_engine_dirs(&engine);
}

#[test]
fn test_submit_valid_data() {
    let mut engine = create_engine();

    let result = engine.submit_kleis("data Color = Red | Green | Blue");

    assert!(
        result.accepted,
        "Should accept valid data: {:?}",
        result.error
    );
    assert!(result.data_types_added.contains(&"Color".to_string()));

    cleanup_engine_dirs(&engine);
}

#[test]
fn test_submit_multiple_items_sequentially() {
    let mut engine = create_engine();

    let r1 = engine.submit_kleis("define double(x) = x + x");
    assert!(r1.accepted);

    let r2 = engine.submit_kleis("define triple(x) = x + x + x");
    assert!(r2.accepted);

    let r3 = engine.submit_kleis(
        r#"
        structure Ring(R) {
            operation zero : R
            operation add : R -> R -> R
        }
        "#,
    );
    assert!(r3.accepted);

    assert_eq!(engine.list_session().len(), 3);

    cleanup_engine_dirs(&engine);
}

// =============================================================================
// Try Structure (Dry Run)
// =============================================================================

#[test]
fn test_try_valid_structure_does_not_modify_session() {
    let engine = create_engine();

    let initial_fn_count = engine.stats()["functions"];

    let result = engine.try_kleis(
        r#"
        define helper_fn(x) = x + 1
        "#,
    );

    assert!(
        result.accepted,
        "Should accept in dry run: {:?}",
        result.error
    );
    assert!(result.functions_added.contains(&"helper_fn".to_string()));

    // Session must NOT be modified
    assert_eq!(engine.list_session().len(), 0);
    assert_eq!(engine.stats()["functions"], initial_fn_count);

    cleanup_engine_dirs(&engine);
}

#[test]
fn test_try_invalid_structure_returns_error() {
    let engine = create_engine();

    let result = engine.try_kleis("structure Bad {{{ invalid }}");

    assert!(!result.accepted);
    assert!(result.error.is_some());

    cleanup_engine_dirs(&engine);
}

// =============================================================================
// Evaluate Expression
// =============================================================================

#[test]
fn test_evaluate_arithmetic() {
    let engine = create_engine();

    let result = engine.evaluate_expression("2 + 3");
    assert!(result.error.is_none(), "error: {:?}", result.error);
    assert_eq!(result.value.as_deref(), Some("5"));
    assert!(result.verified.is_none());

    cleanup_engine_dirs(&engine);
}

#[test]
fn test_evaluate_submitted_function() {
    let mut engine = create_engine();

    let submit = engine.submit_kleis("define add_one(x) = x + 1");
    assert!(submit.accepted);

    let result = engine.evaluate_expression("add_one(41)");
    assert!(result.error.is_none(), "error: {:?}", result.error);
    assert_eq!(result.value.as_deref(), Some("42"));

    cleanup_engine_dirs(&engine);
}

#[test]
fn test_evaluate_parse_error() {
    let engine = create_engine();

    let result = engine.evaluate_expression("define (((");
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("Parse error"));

    cleanup_engine_dirs(&engine);
}

#[test]
#[cfg(feature = "axiom-verification")]
fn test_evaluate_proposition_verified() {
    let engine = create_engine();

    let result = engine.evaluate_expression("∀(x : ℝ). x + 0 = x");
    assert!(result.error.is_none(), "error: {:?}", result.error);
    assert_eq!(result.verified, Some(true));

    cleanup_engine_dirs(&engine);
}

// =============================================================================
// Describe Schema
// =============================================================================

#[test]
fn test_describe_schema_includes_prelude() {
    let engine = create_engine();

    let schema = engine.describe_schema();

    let stats = schema.get("stats").unwrap();
    let fn_count = stats.get("functions").and_then(|v| v.as_u64()).unwrap_or(0);

    assert!(fn_count > 0, "Schema should include prelude functions");

    cleanup_engine_dirs(&engine);
}

#[test]
fn test_describe_schema_includes_agent_submissions() {
    let mut engine = create_engine();

    engine.submit_kleis(
        r#"
        structure TestMonoid(M) {
            operation unit : M
            operation mul : M -> M -> M
            axiom left_id : ∀(x : M). mul(unit, x) = x
        }
        "#,
    );

    let schema = engine.describe_schema();

    let structures = schema.get("structures").unwrap().as_array().unwrap();
    let names: Vec<&str> = structures
        .iter()
        .filter_map(|s| s.get("name").and_then(|n| n.as_str()))
        .collect();
    assert!(
        names.contains(&"TestMonoid"),
        "Schema should include submitted structure, got: {:?}",
        names
    );

    let history_count = schema
        .get("session_history_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert_eq!(history_count, 1);

    cleanup_engine_dirs(&engine);
}

// =============================================================================
// Load Theory (New Session)
// =============================================================================

#[test]
fn test_load_theory_resets_session() {
    let mut engine = create_engine();

    // Add something to the session
    let r = engine.submit_kleis("define my_fn(x) = x");
    assert!(r.accepted);
    assert_eq!(engine.list_session().len(), 1);

    // Load a new theory (just prelude)
    engine
        .load_theory(vec!["stdlib/prelude.kleis".to_string()])
        .expect("load_theory");

    // Session should be reset
    assert_eq!(engine.list_session().len(), 0);

    // But prelude should still be loaded
    assert!(*engine.stats().get("functions").unwrap() > 0);

    cleanup_engine_dirs(&engine);
}

#[test]
fn test_load_theory_empty_imports() {
    let mut engine = create_engine();

    // Load with no imports (minimal universe)
    engine.load_theory(vec![]).expect("load empty theory");

    assert_eq!(engine.list_session().len(), 0);
    // With no imports, there should be very few (or zero) functions
    assert_eq!(*engine.stats().get("functions").unwrap(), 0);

    cleanup_engine_dirs(&engine);
}

// =============================================================================
// Save Theory
// =============================================================================

#[test]
fn test_save_theory_creates_file() {
    let mut engine = create_engine();

    engine.submit_kleis("define saved_fn(x) = x + 1");

    let path = engine.save_theory("test_save").expect("save_theory");

    assert!(
        path.exists(),
        "Saved file should exist at {}",
        path.display()
    );
    assert!(path.to_string_lossy().contains("test_save.kleis"));

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("saved_fn"),
        "Saved file should contain the submitted function"
    );
    assert!(
        content.contains("import"),
        "Saved file should contain import statements"
    );

    // Cleanup
    let _ = std::fs::remove_file(&path);

    cleanup_engine_dirs(&engine);
}

// =============================================================================
// List Session
// =============================================================================

#[test]
fn test_list_session_empty() {
    let engine = create_engine();
    assert!(engine.list_session().is_empty());
    cleanup_engine_dirs(&engine);
}

#[test]
fn test_list_session_tracks_submissions() {
    let mut engine = create_engine();

    engine.submit_kleis("define f1(x) = x");
    engine.submit_kleis("define f2(x) = x + 1");

    let history = engine.list_session();
    assert_eq!(history.len(), 2);
    assert!(history[0].contains("f1"));
    assert!(history[1].contains("f2"));

    cleanup_engine_dirs(&engine);
}

#[test]
fn test_list_session_does_not_track_failures() {
    let mut engine = create_engine();

    engine.submit_kleis("define good(x) = x");
    engine.submit_kleis("structure Bad {{{ invalid }}");

    let history = engine.list_session();
    assert_eq!(history.len(), 1);
    assert!(history[0].contains("good"));

    cleanup_engine_dirs(&engine);
}
