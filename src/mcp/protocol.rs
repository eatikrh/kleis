//! MCP JSON-RPC 2.0 Protocol Types
//!
//! Implements the Model Context Protocol message format.
//! Reference: https://modelcontextprotocol.io/specification

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 Request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Value, code: i64, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
        }
    }
}

// ============================================================================
// MCP-Specific Types
// ============================================================================

/// MCP Tool Definition (returned by tools/list)
#[derive(Debug, Serialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// MCP Tool Result Content (returned by tools/call)
#[derive(Debug, Serialize)]
pub struct McpToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// MCP Server Info
#[derive(Debug, Serialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
}

/// MCP Server Capabilities
#[derive(Debug, Serialize)]
pub struct McpCapabilities {
    pub tools: Option<Value>,
}

/// MCP Initialize Result
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpInitializeResult {
    pub protocol_version: String,
    pub capabilities: McpCapabilities,
    pub server_info: McpServerInfo,
}

// ============================================================================
// Tool Definitions
// ============================================================================

/// Returns the list of tools the Kleis MCP server exposes
pub fn tool_definitions() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "check_action".to_string(),
            description: "Check whether an agent action is allowed by the Kleis policy. \
                         MUST be called before performing any file edit, file creation, \
                         file deletion, terminal command, or git operation."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "description": "The action type: file_edit, file_create, file_delete, run_command, git_push, git_commit",
                        "enum": ["file_edit", "file_create", "file_delete", "run_command", "git_push", "git_commit"]
                    },
                    "path": {
                        "type": "string",
                        "description": "File path (for file operations) or branch name (for git operations)"
                    },
                    "command": {
                        "type": "string",
                        "description": "Shell command (for run_command actions)"
                    },
                    "force": {
                        "type": "boolean",
                        "description": "Whether the operation is forced (for git_push)"
                    },
                    "description": {
                        "type": "string",
                        "description": "Human-readable description of what the action does"
                    }
                },
                "required": ["action"]
            }),
        },
        McpTool {
            name: "list_rules".to_string(),
            description: "List all policy rules currently loaded from the Kleis policy file. \
                         Shows axiom names and their formal specifications."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "explain_rule".to_string(),
            description: "Explain a specific policy rule in detail, including its formal \
                         Kleis specification and what actions it constrains."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "rule_name": {
                        "type": "string",
                        "description": "The name of the rule/axiom to explain"
                    }
                },
                "required": ["rule_name"]
            }),
        },
        // ==================================================================
        // Schema Introspection
        // ==================================================================
        McpTool {
            name: "describe_schema".to_string(),
            description: "Describe the full Kleis schema: structures (with axioms), data types, \
                         functions (check_*, before_*, helpers), and type information. \
                         Use this to understand the policy domain before formulating queries. \
                         No Z3 needed — pure AST introspection."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        // ==================================================================
        // General Expression Evaluation (with Z3 for propositions)
        // ==================================================================
        McpTool {
            name: "evaluate".to_string(),
            description: "Evaluate any Kleis expression **or** verify a proposition. \
                         \n\n\
                         **Concrete evaluation**: call any function from the loaded policy. \
                         Example: `check_file_delete(\"src/main.rs\")` → `\"deny\"`. \
                         \n\n\
                         **Proposition verification**: write a claim using ∀/∃ quantifiers. \
                         Kleis detects propositions and routes them through Z3 automatically. \
                         Example: `∀(b : String). check_git_push(b, 1) = \"deny\"` → VERIFIED. \
                         \n\n\
                         **Workflow**: \
                         1. Call `describe_schema` to learn the functions and structures. \
                         2. The schema includes suggested `verifiable_propositions`. \
                         3. Send a proposition to `evaluate` to check if a property holds. \
                         4. You can also **synthesize your own** propositions from the schema. \
                         \n\n\
                         **Syntax guide**: \
                         - Universal: `∀(x : Type). expr` \
                         - Existential: `∃(x : Type). expr` \
                         - Equality: `a = b` \
                         - Logical: `and(a, b)`, `or(a, b)`, `implies(a, b)`, `not(a)` \
                         - Strings: `\"value\"`, `hasPrefix(s, p)`, `contains(s, sub)` \
                         - Numbers: `0`, `1`, `+`, `-`, `*`, `<`, `>` \
                         \n\n\
                         **Regex operations** (Z3 native regex theory): \
                         - Match: `matches(s, re)` — does string s match regex re? Returns Bool \
                         - Literal: `re_literal(\"foo\")` — matches exactly \"foo\" \
                         - Char range: `re_range(\"a\", \"z\")` — matches one char in [a-z] \
                         - Repetition: `re_star(re)` (0+), `re_plus(re)` (1+), `re_option(re)` (0 or 1) \
                         - Compose: `re_concat(re1, re2)` (sequence), `re_union(re1, re2)` (alternation) \
                         - Negate: `re_complement(re)` — matches anything re doesn't \
                         - Predicates: `isDigits(s)`, `isAlpha(s)`, `isAlphaNum(s)`, `isAscii(s)` \
                         \n\n\
                         **Regex examples**: \
                         - `isAscii(\"hello\")` → true (concrete check) \
                         - `∀(s : String). implies(isDigits(s), isAlphaNum(s))` → VERIFIED (digits ⊂ alphanum) \
                         - `∀(s : String). implies(matches(s, re_plus(re_range(\"a\", \"z\"))), isAlpha(s))` → VERIFIED \
                         - `∃(s : String). and(isAlpha(s), not(isAscii(s)))` → find non-ASCII letters"
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "A Kleis expression or proposition. Use describe_schema to discover available functions and structures."
                    }
                },
                "required": ["expression"]
            }),
        },
    ]
}
