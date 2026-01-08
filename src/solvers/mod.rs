//! Solver Abstraction Layer
//!
//! Pluggable solver backend system inspired by Model Context Protocol (MCP).
//!
//! **Architecture Overview:**
//! ```text
//! User Code (Kleis Expression)
//!          |
//!    SolverBackend Trait
//!          |
//!    +-----+------+----------+
//!    |            |          |
//! Z3Backend  CVC5Backend  CustomBackend
//!    |            |          |
//!    +-----+------+----------+
//!          |
//!   OperationTranslators
//!          |
//!    ResultConverter
//!          |
//! User Code (Kleis Expression)
//! ```
//!
//! **Key Design Principles:**
//!
//! 1. **Solver Independence**
//!    - All public APIs work with Kleis `Expression` only
//!    - Solver-specific types (Z3::Dynamic, etc.) never escape module
//!    - Users can swap solvers without code changes
//!
//! 2. **MCP-Style Capabilities**
//!    - Solvers declare capabilities upfront (capabilities.toml)
//!    - Users can query what operations are natively supported
//!    - Coverage tracking shows what's uninterpreted
//!
//! 3. **User Extensibility**
//!    - Custom translators can be registered at runtime
//!    - Plugin system for adding operation support
//!    - No need to modify Rust code to add operations
//!
//! 4. **Multi-Solver Support**
//!    - Easy to add new solver backends
//!    - Compare coverage across solvers
//!    - Choose best solver for specific programs
//!
//! **Current Status (ADR-023):**
//! - ✅ Core traits defined
//! - ✅ Capability system designed
//! - ✅ Z3 backend being migrated
//! - ⏳ CVC5 backend (future)
//! - ⏳ Plugin system (future)
//!
//! See: docs/session-2024-12-12/SOLVER_ABSTRACTION_LAYER_DESIGN.md

// Core trait definitions
pub mod backend;
pub mod capabilities;
pub mod result_converter;

// Solver implementations
#[cfg(feature = "axiom-verification")]
pub mod z3;

// Isabelle/HOL backend (induction, termination, AFP library)
// Note: Always compiled, but requires Isabelle2025+ installed to use
pub mod isabelle;

// Re-exports for convenience
pub use backend::{SolverBackend, SolverStats, VerificationResult};
pub use capabilities::{
    Capabilities, FeatureFlags, OperationSpec, PerformanceHints, SolverCapabilities, SolverMetadata,
};
pub use isabelle::{IsabelleBackend, IsabelleConfig};
pub use result_converter::ResultConverter;

#[cfg(feature = "axiom-verification")]
pub use z3::Z3Backend;

/// Solver discovery and comparison utilities
///
/// Lists available solvers and checks their availability.
pub mod discovery {
    /// List all available solver backends
    ///
    /// Returns compiled-in solvers. Note that Isabelle requires
    /// external installation (Isabelle2025+) to actually function.
    #[allow(clippy::vec_init_then_push)]
    pub fn list_solvers() -> Vec<String> {
        let mut solvers = vec![];

        #[cfg(feature = "axiom-verification")]
        solvers.push("Z3".to_string());

        // Isabelle is always listed but requires external installation
        solvers.push("Isabelle".to_string());

        solvers
    }

    /// Check if a specific solver is available
    pub fn is_available(solver_name: &str) -> bool {
        list_solvers()
            .iter()
            .any(|s| s.eq_ignore_ascii_case(solver_name))
    }

    /// Check if Isabelle is installed on this system
    ///
    /// Searches common installation paths for the `isabelle` executable.
    pub fn is_isabelle_installed() -> bool {
        use std::path::Path;

        let search_paths = [
            "/Applications/Isabelle2025-1.app/bin/isabelle",
            "/Applications/Isabelle2025.app/bin/isabelle",
            "/usr/local/bin/isabelle",
        ];

        // Check ISABELLE_HOME env var first
        if let Ok(home) = std::env::var("ISABELLE_HOME") {
            let path = Path::new(&home).join("bin/isabelle");
            if path.exists() {
                return true;
            }
        }

        // Check common paths
        for path in search_paths {
            if Path::new(path).exists() {
                return true;
            }
        }

        false
    }

    /// Get the path to the Isabelle executable, if found
    pub fn find_isabelle() -> Option<String> {
        use std::path::Path;

        let search_paths = [
            "/Applications/Isabelle2025-1.app/bin/isabelle",
            "/Applications/Isabelle2025.app/bin/isabelle",
            "/usr/local/bin/isabelle",
        ];

        // Check ISABELLE_HOME env var first
        if let Ok(home) = std::env::var("ISABELLE_HOME") {
            let path = Path::new(&home).join("bin/isabelle");
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }

        // Check common paths
        for path in search_paths {
            if Path::new(path).exists() {
                return Some(path.to_string());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_solver_discovery() {
        use super::discovery;
        let solvers = discovery::list_solvers();

        #[cfg(feature = "axiom-verification")]
        assert!(solvers.contains(&"Z3".to_string()));

        // Isabelle is always in the list (even if not installed)
        assert!(solvers.contains(&"Isabelle".to_string()));
    }

    #[test]
    fn test_is_available() {
        use super::discovery;

        #[cfg(feature = "axiom-verification")]
        assert!(discovery::is_available("Z3"));

        // Case-insensitive check
        assert!(discovery::is_available("isabelle"));
        assert!(discovery::is_available("Isabelle"));
        assert!(discovery::is_available("ISABELLE"));

        assert!(!discovery::is_available("NonexistentSolver"));
    }

    #[test]
    fn test_isabelle_discovery() {
        use super::discovery;

        // This test documents the discovery API, not whether Isabelle is installed
        let _installed = discovery::is_isabelle_installed();
        let _path = discovery::find_isabelle();

        // If path is found, it should exist
        if let Some(path) = discovery::find_isabelle() {
            assert!(std::path::Path::new(&path).exists());
        }
    }
}
