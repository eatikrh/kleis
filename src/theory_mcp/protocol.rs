//! Theory MCP Protocol — JSON-RPC 2.0 types and tool definitions
//!
//! Reuses the core MCP types from `crate::mcp::protocol` and defines
//! the theory-specific tool schemas.

pub use crate::mcp::protocol::{
    JsonRpcError, JsonRpcRequest, JsonRpcResponse, McpCapabilities, McpInitializeResult,
    McpServerInfo, McpTool, McpToolContent,
};

/// Returns the list of tools the theory MCP server exposes.
pub fn theory_tool_definitions() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "evaluate".to_string(),
            description: "Evaluate a Kleis expression or verify a proposition using Z3.\n\n\
                         For concrete expressions, returns the computed value.\n\
                         For propositions (∀/∃/implies/and/or), routes to Z3 for verification.\n\n\
                         Use `describe_schema` first to learn available functions and structures.\n\n\
                         Syntax:\n\
                         - Universal: ∀(x : Type). expr\n\
                         - Existential: ∃(x : Type). expr\n\
                         - Equality: a = b\n\
                         - Logical: and(a, b), or(a, b), implies(a, b), not(a)\n\
                         - Strings: \"value\", hasPrefix(s, p), contains(s, sub)\n\
                         - Regex: matches(s, re_literal(\"..\")), isAscii(s), etc."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "expression": {
                        "type": "string",
                        "description": "A Kleis expression or proposition to evaluate/verify."
                    }
                },
                "required": ["expression"]
            }),
        },
        McpTool {
            name: "describe_schema".to_string(),
            description: "Describe the full Kleis schema currently loaded: structures \
                         (with fields, operations, axioms), data types, and functions. \
                         Includes everything from imports (stdlib, prelude) plus any \
                         structures the agent has submitted in this session.\n\n\
                         Use this to understand the domain vocabulary before formulating \
                         propositions or submitting new structures."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "submit_structure".to_string(),
            description: "Submit a Kleis structure definition to the theory.\n\n\
                         The structure is first validated (parsed, loaded, Z3-checked) \
                         in isolation. If consistent, it is committed to the session.\n\n\
                         Structures can include fields, operations, and axioms. Axioms \
                         are registered with Z3 and participate in all subsequent \
                         proposition verifications.\n\n\
                         Example:\n\
                         ```\n\
                         structure MyGroup(G) {\n\
                           operation e : G\n\
                           operation mul : G -> G -> G\n\
                           operation inv : G -> G\n\
                           axiom identity : ∀(x : G). mul(e, x) = x\n\
                           axiom inverse : ∀(x : G). mul(inv(x), x) = e\n\
                         }\n\
                         ```"
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "kleis": {
                        "type": "string",
                        "description": "Kleis structure definition source code"
                    }
                },
                "required": ["kleis"]
            }),
        },
        McpTool {
            name: "submit_define".to_string(),
            description: "Submit a Kleis function definition to the theory.\n\n\
                         Example:\n\
                         ```\n\
                         define square(x) = x * x\n\
                         ```"
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "kleis": {
                        "type": "string",
                        "description": "Kleis function definition source code"
                    }
                },
                "required": ["kleis"]
            }),
        },
        McpTool {
            name: "submit_data".to_string(),
            description: "Submit a Kleis data type definition to the theory.\n\n\
                         Example:\n\
                         ```\n\
                         data Color = Red | Green | Blue\n\
                         ```"
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "kleis": {
                        "type": "string",
                        "description": "Kleis data type definition source code"
                    }
                },
                "required": ["kleis"]
            }),
        },
        McpTool {
            name: "try_structure".to_string(),
            description: "Dry-run: check if a Kleis structure would be accepted \
                         without committing it to the session. Use this to test \
                         whether new axioms are consistent with the current theory \
                         before submitting.\n\n\
                         Returns the same result as submit_structure but never \
                         modifies the session."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "kleis": {
                        "type": "string",
                        "description": "Kleis source code to validate (structure, define, or data)"
                    }
                },
                "required": ["kleis"]
            }),
        },
        McpTool {
            name: "list_session".to_string(),
            description: "List what the agent has submitted in this session, in order.\n\n\
                         Returns the ordered history of all accepted submissions \
                         (structures, definitions, data types)."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        McpTool {
            name: "load_theory".to_string(),
            description: "Start a new theory session with the specified imports.\n\n\
                         Replaces the current session entirely. The evaluator is \
                         rebuilt from scratch with only the specified imports.\n\n\
                         Pass import paths like:\n\
                         - \"stdlib/prelude.kleis\" for the standard prelude\n\
                         - \"theories/my_saved.kleis\" for a previously saved theory\n\n\
                         Omit to start with nothing (empty universe)."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "imports": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of Kleis files to import (e.g. [\"stdlib/prelude.kleis\"])"
                    }
                },
                "required": ["imports"]
            }),
        },
        McpTool {
            name: "save_theory".to_string(),
            description: "Save the current session to a named .kleis file in the \
                         theories/ directory. The saved file includes all imports and \
                         agent-submitted definitions, and can be loaded in future \
                         sessions via load_theory."
                .to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name for the saved theory (e.g. \"group_theory\" → theories/group_theory.kleis)"
                    }
                },
                "required": ["name"]
            }),
        },
    ]
}
