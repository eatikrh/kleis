//! Isabelle/HOL Backend for Kleis
//!
//! This module provides integration with Isabelle/HOL theorem prover,
//! enabling deep proofs that Z3 cannot handle (induction, termination,
//! higher-order reasoning).
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                        Kleis                                 │
//! └─────────────────────────┬───────────────────────────────────┘
//!                           │ verify ... with isabelle
//!                           ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                  IsabelleBackend                             │
//! │  - Manages server lifecycle                                  │
//! │  - Translates Kleis AST → Isar                              │
//! │  - Sends theories via JSON API                              │
//! │  - Parses verification results                              │
//! └─────────────────────────┬───────────────────────────────────┘
//!                           │ TCP + JSON
//!                           ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                  Isabelle Server                             │
//! │  - isabelle server (port + password)                        │
//! │  - session_start, use_theories, etc.                        │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Capabilities
//!
//! See `capabilities.toml` for full details. Key capabilities:
//!
//! - **Induction**: Prove properties over natural numbers, lists, trees
//! - **Termination**: Prove recursive functions terminate
//! - **AFP Access**: Use 800+ formalized theories from Archive of Formal Proofs
//! - **Higher-Order**: Full higher-order logic reasoning
//!
//! # Example
//!
//! ```kleis
//! // Prove by induction (Z3 cannot do this)
//! axiom sum_formula: ∀(n : ℕ). sum_to(n) = n * (n + 1) / 2
//!
//! verify sum_formula with isabelle
//! ```
//!
//! # Server API Reference
//!
//! Based on Isabelle System Manual, Chapter 4:
//! <https://isabelle.in.tum.de/dist/Isabelle2025-1/doc/system.pdf>

// TODO: Implement these modules
// mod backend;
// mod connection;
// mod translator;
// mod parser;

/// Capabilities configuration loaded from capabilities.toml
pub const CAPABILITIES_TOML: &str = include_str!("capabilities.toml");

/// Default timeout for Isabelle server operations (seconds)
pub const DEFAULT_TIMEOUT: u64 = 30;

/// Timeout for session_start (building HOL takes time)
pub const SESSION_START_TIMEOUT: u64 = 120;

/// Timeout for use_theories (complex proofs take time)
pub const USE_THEORIES_TIMEOUT: u64 = 300;

// TODO: IsabelleBackend struct
// pub struct IsabelleBackend {
//     host: String,
//     port: u16,
//     password: String,
//     session_id: Option<String>,
// }

// TODO: Implement SolverBackend trait
// impl SolverBackend for IsabelleBackend {
//     fn verify_axiom(&self, axiom: &Axiom) -> Result<VerificationResult>;
//     fn name(&self) -> &str { "isabelle" }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capabilities_toml_loads() {
        // Verify the TOML is valid
        let parsed: toml::Value = toml::from_str(CAPABILITIES_TOML)
            .expect("capabilities.toml should be valid TOML");
        
        // Check key sections exist
        assert!(parsed.get("backend").is_some());
        assert!(parsed.get("capabilities").is_some());
        assert!(parsed.get("server").is_some());
        assert!(parsed.get("sessions").is_some());
        assert!(parsed.get("afp").is_some());
        assert!(parsed.get("translation").is_some());
    }

    #[test]
    fn test_backend_name() {
        let parsed: toml::Value = toml::from_str(CAPABILITIES_TOML).unwrap();
        let name = parsed["backend"]["name"].as_str().unwrap();
        assert_eq!(name, "isabelle");
    }

    #[test]
    fn test_default_session_is_hol() {
        let parsed: toml::Value = toml::from_str(CAPABILITIES_TOML).unwrap();
        let default = parsed["sessions"]["default"].as_str().unwrap();
        assert_eq!(default, "HOL");
    }

    #[test]
    fn test_neural_networks_afp_entry() {
        let parsed: toml::Value = toml::from_str(CAPABILITIES_TOML).unwrap();
        let nn = &parsed["afp"]["entries"]["Neural_Networks"];
        assert!(nn.get("description").is_some());
        assert!(nn.get("url").is_some());
    }
}

