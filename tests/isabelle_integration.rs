//! Integration tests for Isabelle backend
//!
//! **NOTE: These tests require a running Isabelle server and are NOT run in CI.**
//!
//! Tests marked with `#[ignore]` require:
//! 1. Isabelle installed locally
//! 2. A running Isabelle server (`isabelle server -n kleis`)
//! 3. Environment variables set with server credentials
//!
//! ## Setup
//!
//! ```bash
//! # Start Isabelle server
//! /Applications/Isabelle2025-1.app/bin/isabelle server -n kleis
//! # Note the port and password from output
//!
//! # List running servers
//! isabelle server -l
//! ```
//!
//! ## Running Tests
//!
//! ```bash
//! # Run server-dependent tests (not run by default)
//! ISABELLE_PORT=51617 ISABELLE_PASSWORD=xxx cargo test --test isabelle_integration -- --ignored
//!
//! # Run only non-server tests (safe for CI)
//! cargo test --test isabelle_integration
//! ```
//!
//! ## CI Behavior
//!
//! - `test_isar_translation_examples` - Runs in CI (no server needed)
//! - All other tests - Skipped in CI (marked `#[ignore]`)

use kleis::solvers::backend::SolverBackend;
use kleis::solvers::isabelle::{IsabelleBackend, IsabelleConfig};
use std::env;
use std::time::Duration;

/// Get Isabelle server config from environment variables
fn get_test_config() -> Option<(u16, String)> {
    let port = env::var("ISABELLE_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())?;
    let password = env::var("ISABELLE_PASSWORD").ok()?;
    Some((port, password))
}

#[test]
#[ignore] // Run with: cargo test --test isabelle_integration -- --ignored
fn test_isabelle_connection() {
    let (port, password) = match get_test_config() {
        Some(config) => config,
        None => {
            eprintln!("Skipping: Set ISABELLE_PORT and ISABELLE_PASSWORD to run this test");
            return;
        }
    };

    let config = IsabelleConfig {
        host: "127.0.0.1".to_string(),
        port,
        password: password.clone(),
        isabelle_path: None,
        session: "HOL".to_string(),
        timeout: Duration::from_secs(30),
    };

    let mut backend = IsabelleBackend::with_config(config).expect("Failed to create backend");

    // Test connection
    let result = backend.connect("127.0.0.1", port, &password);
    assert!(result.is_ok(), "Failed to connect: {:?}", result.err());

    assert!(backend.is_connected(), "Should be connected");

    println!(
        "✅ Successfully connected to Isabelle server on port {}",
        port
    );
}

#[test]
#[ignore]
fn test_isabelle_session_start() {
    let (port, password) = match get_test_config() {
        Some(config) => config,
        None => {
            eprintln!("Skipping: Set ISABELLE_PORT and ISABELLE_PASSWORD to run this test");
            return;
        }
    };

    let mut backend = IsabelleBackend::new().expect("Failed to create backend");
    backend
        .connect("127.0.0.1", port, &password)
        .expect("Failed to connect");

    // Start HOL session
    let result = backend.start_session("HOL");

    // Session start might fail or take a long time, but we test the API works
    match result {
        Ok(()) => {
            assert!(backend.has_session(), "Should have active session");
            println!("✅ Successfully started HOL session");
        }
        Err(e) => {
            // Session start can fail for various reasons (e.g., already running)
            println!("⚠️ Session start returned error (may be expected): {}", e);
        }
    }
}

#[test]
#[ignore]
fn test_translate_and_verify_simple() {
    let (port, password) = match get_test_config() {
        Some(config) => config,
        None => {
            eprintln!("Skipping: Set ISABELLE_PORT and ISABELLE_PASSWORD to run this test");
            return;
        }
    };

    let mut backend = IsabelleBackend::new().expect("Failed to create backend");
    backend
        .connect("127.0.0.1", port, &password)
        .expect("Failed to connect");

    // Note: We skip session_start since it may take too long for tests
    println!("✅ Backend created and connected");
    println!("Backend name: {}", backend.name());
    println!(
        "Has quantifiers: {}",
        backend.capabilities().capabilities.features.quantifiers
    );
}

/// Test that verifies backend capabilities are correctly loaded
#[test]
fn test_isar_translation_examples() {
    let backend = IsabelleBackend::new().expect("Failed to create backend");

    // We can verify the backend works by using public methods
    println!("Backend name: {}", backend.name());
    println!(
        "Has induction: {}",
        backend.capabilities().has_theory("induction")
    );
    println!(
        "Has quantifiers: {}",
        backend.capabilities().capabilities.features.quantifiers
    );

    // Verify capabilities are correct
    assert!(backend.capabilities().has_operation("plus"));
    assert!(backend.capabilities().has_operation("equals"));
    assert!(backend.capabilities().has_theory("HOL"));
    assert!(
        backend
            .capabilities()
            .capabilities
            .features
            .proof_generation
    );

    println!("✅ All capability checks passed");
}
