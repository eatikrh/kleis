//! Z3 Solver Backend Implementation
//!
//! Concrete implementation of SolverBackend trait for Z3 SMT solver.
//!
//! **Architecture:**
//! - `Z3Backend` - Main solver interface
//! - `Z3ResultConverter` - Converts Z3::Dynamic to Kleis Expression
//! - `translators/` - Operation-specific translators (arithmetic, comparison, boolean)
//! - `capabilities.toml` - MCP-style capability declaration
//!
//! **Key Design Points:**
//!
//! 1. **Abstraction Boundary**
//!    - All Z3 types (Dynamic, Int, Real, Bool) stay internal
//!    - Public methods return Kleis Expression only
//!    - ResultConverter enforces this boundary
//!
//! 2. **Incremental Solving**
//!    - Long-lived Solver instance
//!    - push/pop for temporary assumptions
//!    - Efficient for thousands of queries
//!
//! 3. **Smart Axiom Loading**
//!    - On-demand structure loading
//!    - Dependency analysis (extends, over, where clauses)
//!    - Caches loaded structures
//!
//! 4. **Mixed Type Handling**
//!    - Automatically converts Int ↔ Real when needed
//!    - Falls back to uninterpreted functions if types mismatch
//!    - Preserves Z3's type safety
//!
//! **Migration Status:**
//! - ⏳ Extracting from src/axiom_verifier.rs (~300 lines of Z3 code)
//! - ⏳ Organizing into modular translator system
//! - ⏳ Adding ResultConverter for abstraction
//!
//! See: ADR-022 (current Z3 integration)
//!      ADR-023 (new solver abstraction layer)

pub mod backend;
pub mod converter;
pub mod translators;
pub mod type_mapping;

pub use backend::Z3Backend;
pub use converter::Z3ResultConverter;

use crate::solvers::capabilities::SolverCapabilities;

/// Load Z3 capabilities from embedded TOML
///
/// The capabilities are defined in `capabilities.toml` and embedded at compile time.
pub fn load_capabilities() -> Result<SolverCapabilities, String> {
    let toml_str = include_str!("capabilities.toml");
    toml::from_str(toml_str).map_err(|e| format!("Failed to parse Z3 capabilities: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_capabilities() {
        let caps = load_capabilities().expect("Failed to load Z3 capabilities");
        assert_eq!(caps.solver.name, "Z3");
        assert!(caps.has_operation("plus"));
        assert!(caps.has_operation("equals"));
        assert!(caps.has_theory("arithmetic"));
        assert!(caps.has_theory("boolean"));
    }

    #[test]
    fn test_native_operations() {
        let caps = load_capabilities().unwrap();
        let native_ops = caps.native_operations();

        // Check that all 15+ operations are present
        assert!(native_ops.contains(&"plus"));
        assert!(native_ops.contains(&"minus"));
        assert!(native_ops.contains(&"times"));
        assert!(native_ops.contains(&"equals"));
        assert!(native_ops.contains(&"lt"));
        assert!(native_ops.contains(&"and"));
        assert!(native_ops.contains(&"implies"));

        println!("Z3 native operations: {:?}", native_ops);
        assert!(
            native_ops.len() >= 15,
            "Expected at least 15 native operations"
        );
    }

    #[test]
    fn test_feature_flags() {
        let caps = load_capabilities().unwrap();
        assert!(caps.capabilities.features.quantifiers);
        assert!(caps.capabilities.features.uninterpreted_functions);
        assert!(caps.capabilities.features.recursive_functions);
        assert!(caps.capabilities.features.evaluation);
    }
}
