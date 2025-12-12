//! Solver Capabilities Declaration (MCP-Style)
//!
//! Solvers declare their capabilities upfront in a structured format.
//! This enables:
//! - Coverage analysis (what operations are natively supported)
//! - User extensibility (add translators for missing operations)
//! - Multi-solver comparison (which solver best fits a program)
//! - Documentation generation (what each solver can do)
//!
//! Inspired by Model Context Protocol (MCP) resource declarations.
//!
//! See: docs/session-2024-12-12/SOLVER_MCP_STYLE_CAPABILITIES.md

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Complete capability declaration for a solver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverCapabilities {
    /// Solver metadata
    pub solver: SolverMetadata,

    /// Capabilities: operations, theories, features
    pub capabilities: Capabilities,
}

impl SolverCapabilities {
    /// Check if solver natively supports an operation
    pub fn has_operation(&self, op: &str) -> bool {
        self.capabilities.operations.contains_key(op)
    }

    /// Get operation specification
    pub fn get_operation(&self, op: &str) -> Option<&OperationSpec> {
        self.capabilities.operations.get(op)
    }

    /// List all natively supported operations
    pub fn native_operations(&self) -> Vec<&str> {
        self.capabilities
            .operations
            .iter()
            .filter(|(_, spec)| spec.native)
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// List all operations (including non-native)
    pub fn all_operations(&self) -> Vec<&str> {
        self.capabilities
            .operations
            .keys()
            .map(|s| s.as_str())
            .collect()
    }

    /// Check if solver supports a theory
    pub fn has_theory(&self, theory: &str) -> bool {
        self.capabilities.theories.contains(theory)
    }
}

/// Solver metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverMetadata {
    /// Solver name (e.g., "Z3", "CVC5")
    pub name: String,

    /// Solver version (e.g., "4.12.0")
    pub version: String,

    /// Solver type (e.g., "smt", "sat", "theorem_prover")
    #[serde(rename = "type")]
    pub solver_type: String,

    /// Human-readable description
    pub description: String,
}

/// Capability declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    /// Supported theories (e.g., "arithmetic", "boolean", "quantifiers")
    pub theories: HashSet<String>,

    /// Operation specifications
    pub operations: HashMap<String, OperationSpec>,

    /// Feature flags
    #[serde(default)]
    pub features: FeatureFlags,

    /// Performance characteristics
    #[serde(default)]
    pub performance: PerformanceHints,
}

/// Operation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationSpec {
    /// Number of arguments
    pub arity: usize,

    /// Theory this operation belongs to (e.g., "Int/Real", "Bool", "Any")
    pub theory: String,

    /// Whether this operation has a native translator
    pub native: bool,

    /// Optional: Reason if not native (e.g., "No symbolic math support")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Optional: Alternative operations that could be used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternatives: Option<Vec<String>>,
}

/// Feature flags
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Supports quantifiers (∀, ∃)
    #[serde(default)]
    pub quantifiers: bool,

    /// Supports uninterpreted functions
    #[serde(default)]
    pub uninterpreted_functions: bool,

    /// Supports recursive function definitions
    #[serde(default)]
    pub recursive_functions: bool,

    /// Supports model evaluation (concrete values)
    #[serde(default)]
    pub evaluation: bool,

    /// Supports expression simplification
    #[serde(default)]
    pub simplification: bool,

    /// Supports proof generation
    #[serde(default)]
    pub proof_generation: bool,
}

/// Performance hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHints {
    /// Maximum recommended number of axioms
    #[serde(default = "default_max_axioms")]
    pub max_axioms: usize,

    /// Default timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

impl Default for PerformanceHints {
    fn default() -> Self {
        Self {
            max_axioms: default_max_axioms(),
            timeout_ms: default_timeout(),
        }
    }
}

fn default_max_axioms() -> usize {
    10000
}

fn default_timeout() -> u64 {
    5000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_query() {
        let mut ops = HashMap::new();
        ops.insert(
            "plus".to_string(),
            OperationSpec {
                arity: 2,
                theory: "Int/Real".to_string(),
                native: true,
                reason: None,
                alternatives: None,
            },
        );

        let caps = SolverCapabilities {
            solver: SolverMetadata {
                name: "TestSolver".to_string(),
                version: "1.0.0".to_string(),
                solver_type: "smt".to_string(),
                description: "Test solver".to_string(),
            },
            capabilities: Capabilities {
                theories: vec!["arithmetic".to_string()].into_iter().collect(),
                operations: ops,
                features: FeatureFlags::default(),
                performance: PerformanceHints::default(),
            },
        };

        assert!(caps.has_operation("plus"));
        assert!(!caps.has_operation("sin"));
        assert!(caps.has_theory("arithmetic"));
        assert!(!caps.has_theory("calculus"));
    }

    #[test]
    fn test_native_operations_filter() {
        let mut ops = HashMap::new();
        ops.insert(
            "plus".to_string(),
            OperationSpec {
                arity: 2,
                theory: "Int".to_string(),
                native: true,
                reason: None,
                alternatives: None,
            },
        );
        ops.insert(
            "sin".to_string(),
            OperationSpec {
                arity: 1,
                theory: "Real".to_string(),
                native: false,
                reason: Some("No symbolic math".to_string()),
                alternatives: None,
            },
        );

        let caps = SolverCapabilities {
            solver: SolverMetadata {
                name: "TestSolver".to_string(),
                version: "1.0.0".to_string(),
                solver_type: "smt".to_string(),
                description: "Test".to_string(),
            },
            capabilities: Capabilities {
                theories: HashSet::new(),
                operations: ops,
                features: FeatureFlags::default(),
                performance: PerformanceHints::default(),
            },
        };

        let native = caps.native_operations();
        assert_eq!(native.len(), 1);
        assert!(native.contains(&"plus"));

        let all = caps.all_operations();
        assert_eq!(all.len(), 2);
    }
}
