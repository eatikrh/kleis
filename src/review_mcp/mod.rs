//! Review MCP — Code Review via Formal Coding Standards
//!
//! This module implements an MCP server that checks source code against
//! formal coding standards defined in `.kleis` policy files. The same
//! axiom/solver pattern used for physics and agent policies applies here:
//! standards are axioms, code is structure, Z3 checks compliance.
//!
//! ## Tools
//!
//! - `check_code` — check a source code snippet against loaded standards
//! - `check_file` — check a file on disk against loaded standards
//! - `list_rules` — list all loaded coding standard rules
//! - `explain_rule` — explain a specific coding standard rule
//! - `describe_standards` — show the full schema of loaded standards
//!
//! ## Usage
//!
//! ```bash
//! kleis review-mcp --policy examples/policies/rust_review_policy.kleis --verbose
//! ```

pub mod advisory;
pub mod engine;
pub mod protocol;
pub mod server;
