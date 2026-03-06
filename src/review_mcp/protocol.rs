//! Review MCP Tool Definitions
//!
//! Defines the tools exposed by the review MCP server.
//! Reuses JSON-RPC types from `crate::mcp::protocol`.

pub use crate::mcp::protocol::*;

/// Returns the list of tools the Kleis Review MCP server exposes
pub fn review_tool_definitions() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "check_code".to_string(),
            description: "Check a source code snippet against the loaded coding standards. \
                         Each check_* rule in the policy is evaluated with the source code \
                         as input. Returns pass/fail for each rule with reasons."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "source": {
                        "type": "string",
                        "description": "The source code to review"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language (default: rust)",
                        "default": "rust"
                    }
                },
                "required": ["source"]
            }),
        },
        McpTool {
            name: "check_file".to_string(),
            description: "Check a file on disk against the loaded coding standards. \
                         Reads the file and runs all check_* rules against its contents."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to review"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language (default: inferred from extension)",
                        "default": "rust"
                    }
                },
                "required": ["path"]
            }),
        },
        McpTool {
            name: "list_rules".to_string(),
            description: "List all coding standard rules loaded from the review policy file."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "explain_rule".to_string(),
            description: "Explain a specific coding standard rule in detail.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "rule_name": {
                        "type": "string",
                        "description": "The name of the rule to explain"
                    }
                },
                "required": ["rule_name"]
            }),
        },
        McpTool {
            name: "describe_standards".to_string(),
            description: "Describe the full schema of loaded coding standards, \
                         including all check functions, helper functions, and structures."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "diff_check_file".to_string(),
            description:
                "Check a file against diff-based rules by comparing the current \
                         version on disk with the version from a base branch (via git). \
                         Runs all diff_check_* and diff_advise_* rules (e.g., version bump checks)."
                    .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to review (absolute or relative to repo root)"
                    },
                    "base": {
                        "type": "string",
                        "description": "Base branch or commit to compare against (default: main)",
                        "default": "main"
                    }
                },
                "required": ["path"]
            }),
        },
        McpTool {
            name: "evaluate".to_string(),
            description: "Evaluate a Kleis expression or verify a proposition via Z3 \
                         in the context of the loaded review policy. Propositions \
                         (quantifiers, logical connectives) are verified by Z3; \
                         other expressions are evaluated concretely. Use this to \
                         verify formal properties of the coding standards."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "A Kleis expression to evaluate or a proposition to verify"
                    }
                },
                "required": ["expression"]
            }),
        },
    ]
}
