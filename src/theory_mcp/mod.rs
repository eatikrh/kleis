//! Theory MCP — Interactive Theory Building with Agent Co-authorship
//!
//! This module implements an MCP server that lets agents co-author Kleis
//! theories interactively, with Z3 checking consistency at every step.
//!
//! Unlike the policy MCP (`src/mcp/`) which enforces fixed rules, the theory
//! MCP allows agents to submit new structures, definitions, and data types
//! into an evolving session. The file system is the source of truth; the
//! evaluator is rebuilt from files on each commit.
//!
//! ## Tools
//!
//! - `evaluate` — evaluate expression or verify proposition via Z3
//! - `describe_schema` — show everything loaded (imports + agent's additions)
//! - `submit_structure` — add a structure (with fields, operations, axioms)
//! - `submit_define` — add a top-level function definition
//! - `submit_data` — add a data type definition
//! - `try_structure` — dry-run: check consistency without committing
//! - `list_session` — show session history
//! - `load_theory` — restart with specified imports (new universe)
//! - `save_theory` — persist session to a named `.kleis` file
//!
//! ## Usage
//!
//! ```bash
//! kleis theory-mcp --verbose
//! ```

pub mod engine;
pub mod protocol;
pub mod server;
