//! MCP (Model Context Protocol) Server for Kleis
//!
//! This module implements an MCP server that exposes the full Kleis reasoning
//! engine to LLM agents over JSON-RPC 2.0 (stdio transport).
//!
//! ## Tools (5 total)
//!
//! ### Policy Layer
//! - `check_action` — verify whether a specific agent action is allowed
//! - `list_rules` — list all loaded policy rules
//! - `explain_rule` — explain a specific rule in detail
//!
//! ### Reasoning Layer
//! - `describe_schema` — introspect structures, data types, axioms, functions (no Z3)
//! - `evaluate` — evaluate any Kleis expression or verify propositions (uses
//!   the same `assert()` pipeline as Kleis example blocks, routing to Z3
//!   automatically for quantified/symbolic expressions)
//!
//! ## Architecture
//!
//! The MCP server:
//! 1. Loads a Kleis policy file (`.kleis`) containing formal rules
//! 2. Exposes tools over JSON-RPC 2.0 (stdio transport)
//! 3. The agent can introspect the schema, evaluate expressions, and prove theorems
//! 4. Policy checks return ALLOWED/DENIED with preconditions
//!
//! ## Usage
//!
//! ```bash
//! kleis mcp --policy examples/policies/agent_policy.kleis --verbose
//! ```

pub mod policy;
pub mod protocol;
pub mod server;
