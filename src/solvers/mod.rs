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

// Re-exports for convenience
pub use backend::{SolverBackend, SolverStats, VerificationResult};
pub use capabilities::{
    Capabilities, FeatureFlags, OperationSpec, PerformanceHints, SolverCapabilities,
    SolverMetadata,
};
pub use result_converter::ResultConverter;

#[cfg(feature = "axiom-verification")]
pub use z3::Z3Backend;

/// Solver discovery and comparison utilities
///
/// Future: Will list available solvers, compare capabilities, etc.
pub mod discovery {
    use super::*;

    /// List all available solver backends
    ///
    /// Future: Will scan for installed solvers (Z3, CVC5, etc.)
    pub fn list_solvers() -> Vec<String> {
        let mut solvers = Vec::new();

        #[cfg(feature = "axiom-verification")]
        solvers.push("Z3".to_string());

        // Future: Add detection for CVC5, etc.

        solvers
    }

    /// Check if a specific solver is available
    pub fn is_available(solver_name: &str) -> bool {
        list_solvers().contains(&solver_name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::discovery;

    #[test]
    fn test_solver_discovery() {
        let solvers = discovery::list_solvers();
        #[cfg(feature = "axiom-verification")]
        assert!(solvers.contains(&"Z3".to_string()));
    }

    #[test]
    fn test_is_available() {
        #[cfg(feature = "axiom-verification")]
        assert!(discovery::is_available("Z3"));

        assert!(!discovery::is_available("NonexistentSolver"));
    }
}

