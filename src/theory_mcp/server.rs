//! Theory MCP Server — JSON-RPC 2.0 over stdio
//!
//! Implements the MCP transport layer for interactive theory building.
//! Reads JSON-RPC messages from stdin, dispatches to theory tool handlers,
//! and writes responses to stdout.

use super::engine::TheoryEngine;
use super::protocol::{
    self, JsonRpcRequest, JsonRpcResponse, McpCapabilities, McpInitializeResult, McpServerInfo,
    McpToolContent,
};
use crate::config::TheoryConfig;
use serde_json::Value;
use std::io::{self, BufRead, Read, Write};

/// Theory MCP Server state
pub struct TheoryMcpServer {
    engine: TheoryEngine,
    verbose: bool,
}

impl TheoryMcpServer {
    /// Create a new theory MCP server.
    pub fn new(config: &TheoryConfig, verbose: bool) -> Result<Self, String> {
        let engine = TheoryEngine::new(config)?;
        Ok(Self { engine, verbose })
    }

    /// Run the server over stdio (blocking).
    pub fn run(&mut self) -> Result<(), String> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = stdin.lock();
        let mut writer = stdout.lock();

        self.log("Kleis Theory MCP Server started");

        let stats = self.engine.stats();
        self.log(&format!(
            "Session: {} structures, {} functions loaded",
            stats.get("structures").unwrap_or(&0),
            stats.get("functions").unwrap_or(&0),
        ));

        let mut use_ndjson = false;

        loop {
            let body: Vec<u8> = if use_ndjson {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => return Ok(()),
                    Ok(_) => {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        trimmed.as_bytes().to_vec()
                    }
                    Err(e) => return Err(format!("Read error: {}", e)),
                }
            } else {
                let mut first_line = String::new();
                match reader.read_line(&mut first_line) {
                    Ok(0) => return Ok(()),
                    Ok(_) => {}
                    Err(e) => return Err(format!("Read error: {}", e)),
                }

                let trimmed = first_line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if trimmed.starts_with('{') {
                    use_ndjson = true;
                    self.log("Detected NDJSON transport (Cursor-style)");
                    trimmed.as_bytes().to_vec()
                } else if let Some(len_str) = trimmed.strip_prefix("Content-Length:") {
                    self.log("Detected Content-Length transport (LSP-style)");
                    let content_length: usize = len_str.trim().parse().unwrap_or(0);

                    loop {
                        let mut header_line = String::new();
                        match reader.read_line(&mut header_line) {
                            Ok(0) => return Ok(()),
                            Ok(_) => {
                                if header_line.trim().is_empty() {
                                    break;
                                }
                            }
                            Err(e) => return Err(format!("Read error: {}", e)),
                        }
                    }

                    let mut buf = vec![0u8; content_length];
                    reader
                        .read_exact(&mut buf)
                        .map_err(|e| format!("Read body error: {}", e))?;
                    buf
                } else {
                    self.log(&format!("Skipping unknown line: {}", trimmed));
                    continue;
                }
            };

            let request: JsonRpcRequest = match serde_json::from_slice(&body) {
                Ok(req) => req,
                Err(e) => {
                    self.log(&format!("Invalid JSON-RPC: {}", e));
                    continue;
                }
            };

            self.log(&format!("← {}", request.method));

            let response = self.handle_request(&request);

            if let Some(ref response) = response {
                let response_str = serde_json::to_string(response)
                    .map_err(|e| format!("Serialize error: {}", e))?;

                if use_ndjson {
                    writer
                        .write_all(response_str.as_bytes())
                        .map_err(|e| format!("Write error: {}", e))?;
                    writer
                        .write_all(b"\n")
                        .map_err(|e| format!("Write error: {}", e))?;
                } else {
                    let header = format!("Content-Length: {}\r\n\r\n", response_str.len());
                    writer
                        .write_all(header.as_bytes())
                        .map_err(|e| format!("Write error: {}", e))?;
                    writer
                        .write_all(response_str.as_bytes())
                        .map_err(|e| format!("Write error: {}", e))?;
                }
                writer.flush().map_err(|e| format!("Flush error: {}", e))?;

                self.log("→ response sent");
            }
        }
    }

    /// Dispatch a JSON-RPC request to the appropriate handler.
    fn handle_request(&mut self, request: &JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = request.id.clone().unwrap_or(Value::Null);

        match request.method.as_str() {
            "initialize" => {
                let result = McpInitializeResult {
                    protocol_version: "2024-11-05".to_string(),
                    capabilities: McpCapabilities {
                        tools: Some(serde_json::json!({})),
                    },
                    server_info: McpServerInfo {
                        name: "kleis-theory".to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    },
                };

                Some(JsonRpcResponse::success(
                    id,
                    serde_json::to_value(result).unwrap(),
                ))
            }

            "notifications/initialized" | "initialized" => None,

            "tools/list" => {
                let tools = protocol::theory_tool_definitions();
                Some(JsonRpcResponse::success(
                    id,
                    serde_json::json!({ "tools": tools }),
                ))
            }

            "tools/call" => {
                let params = request.params.as_ref();
                let tool_name = params
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");
                let arguments = params
                    .and_then(|p| p.get("arguments"))
                    .cloned()
                    .unwrap_or(serde_json::json!({}));

                let result = match tool_name {
                    "evaluate" => self.handle_evaluate(&arguments),
                    "describe_schema" => self.handle_describe_schema(),
                    "submit_structure" => self.handle_submit(&arguments, "structure"),
                    "submit_define" => self.handle_submit(&arguments, "define"),
                    "submit_data" => self.handle_submit(&arguments, "data"),
                    "try_structure" => self.handle_try(&arguments),
                    "list_session" => self.handle_list_session(),
                    "load_theory" => self.handle_load_theory(&arguments),
                    "save_theory" => self.handle_save_theory(&arguments),
                    _ => {
                        let content = McpToolContent {
                            content_type: "text".to_string(),
                            text: format!("Unknown tool: '{}'", tool_name),
                        };
                        serde_json::json!({
                            "content": [content],
                            "isError": true
                        })
                    }
                };

                Some(JsonRpcResponse::success(id, result))
            }

            _ => {
                self.log(&format!("Unknown method: {}", request.method));
                Some(JsonRpcResponse::error(
                    id,
                    -32601,
                    format!("Method not found: {}", request.method),
                ))
            }
        }
    }

    // ========================================================================
    // Tool Handlers
    // ========================================================================

    fn handle_evaluate(&self, arguments: &Value) -> Value {
        let expr_str = arguments
            .get("expression")
            .and_then(|e| e.as_str())
            .unwrap_or("");

        self.log(&format!("evaluate: {}", expr_str));

        let result = self.engine.evaluate_expression(expr_str);

        if let Some(ref err) = result.error {
            if result.value.is_none() && result.verified.is_none() {
                self.log(&format!("  → error: {}", err));
                let content = McpToolContent {
                    content_type: "text".to_string(),
                    text: format!("❌ {}\n\nExpression: {}", err, expr_str),
                };
                return serde_json::json!({
                    "content": [content],
                    "isError": true,
                });
            }
        }

        if let Some(verified) = result.verified {
            let is_inconsistency = result
                .error
                .as_ref()
                .map(|e| e.contains("INCONSISTENCY"))
                .unwrap_or(false);

            let (emoji, status) = if is_inconsistency {
                ("🚨", "AXIOM INCONSISTENCY")
            } else if verified {
                ("✅", "VERIFIED")
            } else {
                ("❌", "DISPROVED")
            };
            let value_str = result.value.as_deref().unwrap_or("?");

            self.log(&format!("  → {} {} — {}", emoji, status, value_str));

            let mut text = if is_inconsistency {
                format!(
                    "{} {}\n\nProposition: {}\n\n{}\n\nAll assertions would be vacuously true. Fix the axiom definitions before verifying.",
                    emoji,
                    status,
                    expr_str,
                    result.error.as_deref().unwrap_or("")
                )
            } else {
                format!(
                    "{} {}\n\nProposition: {}\nResult: {}",
                    emoji, status, expr_str, value_str
                )
            };

            let (witness_str, witness_bindings) = if let Some(ref w) = result.witness {
                let pp = crate::pretty_print::PrettyPrinter::new();
                let binding_strs: Vec<String> = w
                    .bindings
                    .iter()
                    .map(|b| format!("{} = {}", b.name, pp.format_expression(&b.value)))
                    .collect();
                let binding_json: Vec<serde_json::Value> = w
                    .bindings
                    .iter()
                    .map(|b| {
                        serde_json::json!({
                            "variable": b.name,
                            "value": pp.format_expression(&b.value),
                        })
                    })
                    .collect();

                let label = if verified {
                    "Witness"
                } else {
                    "Counterexample"
                };
                if binding_strs.is_empty() {
                    text.push_str(&format!("\n\n{} (raw):\n{}", label, w.raw));
                    (Some(w.raw.clone()), binding_json)
                } else {
                    text.push_str(&format!("\n\n{} (Kleis):", label));
                    for s in &binding_strs {
                        text.push_str(&format!("\n  {}", s));
                    }
                    (Some(binding_strs.join(", ")), binding_json)
                }
            } else {
                (None, Vec::new())
            };

            let content = McpToolContent {
                content_type: "text".to_string(),
                text,
            };

            return serde_json::json!({
                "content": [content],
                "verified": verified,
                "witness": witness_str,
                "witness_bindings": witness_bindings,
            });
        }

        let value_str = result.value.as_deref().unwrap_or("(no result)");
        self.log(&format!("  → {}", value_str));

        let content = McpToolContent {
            content_type: "text".to_string(),
            text: format!(
                "✅ {}\n\nExpression: {}\nResult: {}",
                value_str, expr_str, value_str
            ),
        };

        serde_json::json!({
            "content": [content],
            "result": value_str,
        })
    }

    fn handle_describe_schema(&self) -> Value {
        self.log("describe_schema requested");

        let schema = self.engine.describe_schema();

        let mut text = String::new();
        text.push_str("📐 Kleis Theory Schema\n\n");

        let stats = schema
            .get("stats")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        text.push_str(&format!(
            "Session: {}\nStructures: {}, Data types: {}, Functions: {}, Axioms: {}\n\
             Session items submitted: {}\n",
            schema
                .get("session_file")
                .and_then(|p| p.as_str())
                .unwrap_or("?"),
            stats
                .get("structures")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            stats
                .get("data_types")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            stats.get("functions").and_then(|v| v.as_u64()).unwrap_or(0),
            stats.get("axioms").and_then(|v| v.as_u64()).unwrap_or(0),
            schema
                .get("session_history_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        ));

        if let Some(structures) = schema.get("structures").and_then(|s| s.as_array()) {
            if !structures.is_empty() {
                text.push_str("\n## Structures\n\n");
                for s in structures {
                    let name = s.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                    let params = s
                        .get("type_params")
                        .and_then(|p| p.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    text.push_str(&format!("### structure {}({})\n", name, params));

                    if let Some(fields) = s.get("fields").and_then(|f| f.as_array()) {
                        for field in fields {
                            let fn_ = field.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                            let ft = field.get("type").and_then(|t| t.as_str()).unwrap_or("?");
                            text.push_str(&format!("  field {} : {}\n", fn_, ft));
                        }
                    }
                    if let Some(ops) = s.get("operations").and_then(|o| o.as_array()) {
                        for op in ops {
                            let on = op.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                            let ot = op.get("type").and_then(|t| t.as_str()).unwrap_or("?");
                            text.push_str(&format!("  operation {} : {}\n", on, ot));
                        }
                    }
                    if let Some(axioms) = s.get("axioms").and_then(|a| a.as_array()) {
                        for ax in axioms {
                            let an = ax.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                            let ak = ax.get("kleis").and_then(|k| k.as_str()).unwrap_or("?");
                            text.push_str(&format!("  axiom {} : {}\n", an, ak));
                        }
                    }
                    text.push('\n');
                }
            }
        }

        if let Some(data_types) = schema.get("data_types").and_then(|d| d.as_array()) {
            if !data_types.is_empty() {
                text.push_str("## Data Types\n\n");
                for d in data_types {
                    let name = d.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                    let variants = d
                        .get("variants")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.get("name").and_then(|n| n.as_str()))
                                .collect::<Vec<_>>()
                                .join(" | ")
                        })
                        .unwrap_or_default();
                    text.push_str(&format!("data {} = {}\n", name, variants));
                }
                text.push('\n');
            }
        }

        if let Some(fns) = schema.get("functions").and_then(|f| f.as_array()) {
            if !fns.is_empty() {
                text.push_str("## Functions\n\n");
                for f in fns {
                    if let Some(kleis) = f.get("kleis").and_then(|k| k.as_str()) {
                        text.push_str(&format!("```\n{}\n```\n\n", kleis));
                    }
                }
            }
        }

        text.push_str("\n---\n");
        text.push_str("Use `evaluate` to test expressions or verify propositions.\n");
        text.push_str(
            "Use `submit_structure`/`submit_define`/`submit_data` to add to the theory.\n",
        );
        text.push_str("Use `try_structure` to dry-run before committing.\n");

        let content = McpToolContent {
            content_type: "text".to_string(),
            text,
        };

        serde_json::json!({
            "content": [content],
            "schema": schema,
        })
    }

    fn handle_submit(&mut self, arguments: &Value, kind: &str) -> Value {
        let kleis_source = arguments
            .get("kleis")
            .and_then(|k| k.as_str())
            .unwrap_or("");

        let truncated: String = kleis_source.chars().take(80).collect();
        self.log(&format!("submit_{}: {}[...]", kind, truncated));

        let result = self.engine.submit_kleis(kleis_source);

        if result.accepted {
            let mut text = format!("✅ {} accepted\n", kind);
            if !result.structures_added.is_empty() {
                text.push_str(&format!(
                    "\nStructures added: {}\n",
                    result.structures_added.join(", ")
                ));
            }
            if !result.functions_added.is_empty() {
                text.push_str(&format!(
                    "Functions added: {}\n",
                    result.functions_added.join(", ")
                ));
            }
            if !result.data_types_added.is_empty() {
                text.push_str(&format!(
                    "Data types added: {}\n",
                    result.data_types_added.join(", ")
                ));
            }
            text.push_str("\nThe new definitions are now part of the theory. ");
            text.push_str("Use `evaluate` to test propositions involving them.");

            self.log("  → accepted");

            let content = McpToolContent {
                content_type: "text".to_string(),
                text,
            };

            serde_json::json!({
                "content": [content],
                "accepted": true,
                "structures_added": result.structures_added,
                "functions_added": result.functions_added,
                "data_types_added": result.data_types_added,
            })
        } else {
            let err = result.error.unwrap_or_else(|| "Unknown error".to_string());
            self.log(&format!("  → rejected: {}", err));

            let content = McpToolContent {
                content_type: "text".to_string(),
                text: format!(
                    "❌ {} rejected\n\nError: {}\n\nThe session was not modified. \
                     Fix the error and try again.",
                    kind, err
                ),
            };

            serde_json::json!({
                "content": [content],
                "isError": true,
                "accepted": false,
                "error": err,
            })
        }
    }

    fn handle_try(&self, arguments: &Value) -> Value {
        let kleis_source = arguments
            .get("kleis")
            .and_then(|k| k.as_str())
            .unwrap_or("");

        let truncated: String = kleis_source.chars().take(80).collect();
        self.log(&format!("try_structure: {}[...]", truncated));

        let result = self.engine.try_kleis(kleis_source);

        if result.accepted {
            let mut text = "✅ Would be accepted (dry run)\n".to_string();
            if !result.structures_added.is_empty() {
                text.push_str(&format!(
                    "\nStructures: {}\n",
                    result.structures_added.join(", ")
                ));
            }
            if !result.functions_added.is_empty() {
                text.push_str(&format!(
                    "Functions: {}\n",
                    result.functions_added.join(", ")
                ));
            }
            if !result.data_types_added.is_empty() {
                text.push_str(&format!(
                    "Data types: {}\n",
                    result.data_types_added.join(", ")
                ));
            }
            text.push_str("\nThe session was NOT modified. Use submit_structure to commit.");

            self.log("  → would be accepted");

            let content = McpToolContent {
                content_type: "text".to_string(),
                text,
            };

            serde_json::json!({
                "content": [content],
                "accepted": true,
                "structures_added": result.structures_added,
                "functions_added": result.functions_added,
                "data_types_added": result.data_types_added,
            })
        } else {
            let err = result.error.unwrap_or_else(|| "Unknown error".to_string());
            self.log(&format!("  → would be rejected: {}", err));

            let content = McpToolContent {
                content_type: "text".to_string(),
                text: format!("❌ Would be rejected\n\nError: {}", err),
            };

            serde_json::json!({
                "content": [content],
                "isError": true,
                "accepted": false,
                "error": err,
            })
        }
    }

    fn handle_list_session(&self) -> Value {
        let history = self.engine.list_session();

        let text = if history.is_empty() {
            "📋 Session is empty — no submissions yet.\n\n\
             Use submit_structure, submit_define, or submit_data to build your theory."
                .to_string()
        } else {
            let mut t = format!("📋 Session history ({} items)\n\n", history.len());
            for (i, item) in history.iter().enumerate() {
                let preview: String = if item.chars().count() > 100 {
                    format!("{}...", item.chars().take(100).collect::<String>())
                } else {
                    item.clone()
                };
                t.push_str(&format!("{}. ```\n{}\n```\n\n", i + 1, preview));
            }
            t
        };

        let content = McpToolContent {
            content_type: "text".to_string(),
            text,
        };

        serde_json::json!({
            "content": [content],
            "count": history.len(),
        })
    }

    fn handle_load_theory(&mut self, arguments: &Value) -> Value {
        let imports: Vec<String> = arguments
            .get("imports")
            .and_then(|i| i.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        self.log(&format!("load_theory: {:?}", imports));

        match self.engine.load_theory(imports.clone()) {
            Ok(()) => {
                let stats = self.engine.stats();
                let text = format!(
                    "✅ New session started\n\nImports: {}\nFunctions loaded: {}\n\
                     Structures loaded: {}\n\nPrevious session was discarded.",
                    if imports.is_empty() {
                        "(none)".to_string()
                    } else {
                        imports.join(", ")
                    },
                    stats.get("functions").unwrap_or(&0),
                    stats.get("structures").unwrap_or(&0),
                );

                self.log(&format!(
                    "  → new session: {} functions, {} structures",
                    stats.get("functions").unwrap_or(&0),
                    stats.get("structures").unwrap_or(&0),
                ));

                let content = McpToolContent {
                    content_type: "text".to_string(),
                    text,
                };

                serde_json::json!({
                    "content": [content],
                })
            }
            Err(e) => {
                self.log(&format!("  → error: {}", e));

                let content = McpToolContent {
                    content_type: "text".to_string(),
                    text: format!(
                        "❌ Failed to load theory\n\nError: {}\n\n\
                         The previous session is still active.",
                        e
                    ),
                };

                serde_json::json!({
                    "content": [content],
                    "isError": true,
                })
            }
        }
    }

    fn handle_save_theory(&self, arguments: &Value) -> Value {
        let name = arguments
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("unnamed");

        self.log(&format!("save_theory: {}", name));

        match self.engine.save_theory(name) {
            Ok(path) => {
                let text = format!(
                    "✅ Theory saved to {}\n\nYou can reload it later with:\n\
                     load_theory(imports: [\"{}\"])",
                    path.display(),
                    path.display(),
                );

                self.log(&format!("  → saved to {}", path.display()));

                let content = McpToolContent {
                    content_type: "text".to_string(),
                    text,
                };

                serde_json::json!({
                    "content": [content],
                    "path": path.to_string_lossy(),
                })
            }
            Err(e) => {
                self.log(&format!("  → error: {}", e));

                let content = McpToolContent {
                    content_type: "text".to_string(),
                    text: format!("❌ Failed to save theory\n\nError: {}", e),
                };

                serde_json::json!({
                    "content": [content],
                    "isError": true,
                })
            }
        }
    }

    fn log(&self, msg: &str) {
        if self.verbose {
            eprintln!("[kleis-theory] {}", msg);
        }
    }
}
