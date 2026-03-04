//! Review MCP Server — JSON-RPC 2.0 over stdio
//!
//! Implements the MCP transport layer for code review.
//! Reads JSON-RPC messages from stdin, dispatches to review tool handlers,
//! and writes responses to stdout.

use super::engine::{ReviewEngine, ReviewRuleKind};
use super::protocol::{
    self, JsonRpcRequest, JsonRpcResponse, McpCapabilities, McpInitializeResult, McpServerInfo,
    McpToolContent,
};
use serde_json::Value;
use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};

/// Review MCP Server state
pub struct ReviewMcpServer {
    engine: ReviewEngine,
    server_name: String,
    verbose: bool,
}

impl ReviewMcpServer {
    /// Create a new review MCP server with a loaded policy
    pub fn new(policy_path: &PathBuf, verbose: bool) -> Result<Self, String> {
        let engine = ReviewEngine::load(policy_path)?;
        let server_name = Self::derive_server_name(policy_path);
        Ok(Self {
            engine,
            server_name,
            verbose,
        })
    }

    /// Derive a server name from the policy filename.
    /// e.g. "python_review_policy.kleis" -> "kleis-review-python"
    ///      "rust_review_policy.kleis"   -> "kleis-review-rust"
    fn derive_server_name(policy_path: &Path) -> String {
        let stem = policy_path
            .file_stem()
            .and_then(|s: &std::ffi::OsStr| s.to_str())
            .unwrap_or("");
        if let Some(lang) = stem.strip_suffix("_review_policy") {
            format!("kleis-review-{}", lang)
        } else {
            "kleis-review".to_string()
        }
    }

    /// Run the server over stdio (blocking)
    pub fn run(&self) -> Result<(), String> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = stdin.lock();
        let mut writer = stdout.lock();

        self.log("Kleis Review MCP Server started");
        self.log(&format!("Policy: {}", self.engine.policy_file().display()));

        let stats = self.engine.stats();
        self.log(&format!(
            "Loaded: {} check functions, {} total rules",
            stats.get("check_functions").unwrap_or(&0),
            stats.get("total_rules").unwrap_or(&0),
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

    fn handle_request(&self, request: &JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = request.id.clone().unwrap_or(Value::Null);

        match request.method.as_str() {
            "initialize" => {
                let result = McpInitializeResult {
                    protocol_version: "2024-11-05".to_string(),
                    capabilities: McpCapabilities {
                        tools: Some(serde_json::json!({})),
                    },
                    server_info: McpServerInfo {
                        name: self.server_name.clone(),
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
                let tools = protocol::review_tool_definitions();
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
                    "check_code" => self.handle_check_code(&arguments),
                    "check_file" => self.handle_check_file(&arguments),
                    "list_rules" => self.handle_list_rules(),
                    "explain_rule" => self.handle_explain_rule(&arguments),
                    "describe_standards" => self.handle_describe_standards(),
                    "evaluate" => self.handle_evaluate(&arguments),
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

    fn handle_check_code(&self, arguments: &Value) -> Value {
        let source = arguments
            .get("source")
            .and_then(|s| s.as_str())
            .unwrap_or("");
        let language = arguments
            .get("language")
            .and_then(|l| l.as_str())
            .unwrap_or("rust");

        self.log(&format!(
            "check_code: {} chars, language={}",
            source.len(),
            language
        ));

        let result = self.engine.check_code(source, language);

        let emoji = if result.passed { "✅" } else { "❌" };

        let mut text = format!("{} Code Review: {}\n\n", emoji, result.summary);

        for verdict in &result.verdicts {
            let v_emoji = if verdict.passed { "✅" } else { "❌" };
            text.push_str(&format!(
                "{} {} — {}\n",
                v_emoji, verdict.rule_name, verdict.message
            ));
        }

        self.log(&format!(
            "  → {} {}",
            emoji,
            if result.passed { "PASS" } else { "FAIL" }
        ));

        let content = McpToolContent {
            content_type: "text".to_string(),
            text,
        };

        serde_json::json!({
            "content": [content],
            "passed": result.passed,
            "verdicts": result.verdicts.iter().map(|v| serde_json::json!({
                "rule": v.rule_name,
                "passed": v.passed,
                "message": v.message,
            })).collect::<Vec<_>>(),
        })
    }

    fn check_file_error(&self, message: &str) -> Value {
        let content = McpToolContent {
            content_type: "text".to_string(),
            text: message.to_string(),
        };
        serde_json::json!({
            "content": [content],
            "isError": true,
        })
    }

    fn handle_check_file(&self, arguments: &Value) -> Value {
        let path = arguments.get("path").and_then(|p| p.as_str()).unwrap_or("");
        let language = arguments
            .get("language")
            .and_then(|l| l.as_str())
            .unwrap_or("rust");

        self.log(&format!("check_file: {}, language={}", path, language));

        let result = match self.engine.check_file(path, language) {
            Ok(r) => r,
            Err(e) => return self.check_file_error(&e),
        };

        let emoji = if result.passed { "✅" } else { "❌" };

        let mut text = format!("{} Code Review: {} — {}\n\n", emoji, path, result.summary);

        for verdict in &result.verdicts {
            let v_emoji = if verdict.passed { "✅" } else { "❌" };
            text.push_str(&format!(
                "{} {} — {}\n",
                v_emoji, verdict.rule_name, verdict.message
            ));
        }

        let content = McpToolContent {
            content_type: "text".to_string(),
            text,
        };

        serde_json::json!({
            "content": [content],
            "passed": result.passed,
            "verdicts": result.verdicts.iter().map(|v| serde_json::json!({
                "rule": v.rule_name,
                "passed": v.passed,
                "message": v.message,
            })).collect::<Vec<_>>(),
        })
    }

    fn handle_list_rules(&self) -> Value {
        let rules = self.engine.list_rules();

        let mut text = format!(
            "Coding Standards ({})\n\nPolicy: {}\n\n",
            rules.len(),
            self.engine.policy_file().display()
        );

        text.push_str("## Check Functions\n\n");
        for rule in rules
            .iter()
            .filter(|r| matches!(r.kind, ReviewRuleKind::CheckFunction))
        {
            text.push_str(&format!("  - {} — {}\n", rule.name, rule.description));
        }

        let axiom_rules: Vec<_> = rules
            .iter()
            .filter(|r| matches!(r.kind, ReviewRuleKind::Axiom))
            .collect();
        if !axiom_rules.is_empty() {
            text.push_str("\n## Axioms\n\n");
            for rule in axiom_rules {
                text.push_str(&format!("  - {} — {}\n", rule.name, rule.description));
            }
        }

        let helper_rules: Vec<_> = rules
            .iter()
            .filter(|r| matches!(r.kind, ReviewRuleKind::Helper))
            .collect();
        if !helper_rules.is_empty() {
            text.push_str("\n## Helper Functions\n\n");
            for rule in helper_rules {
                text.push_str(&format!("  - {}\n", rule.name));
            }
        }

        let content = McpToolContent {
            content_type: "text".to_string(),
            text,
        };

        serde_json::json!({ "content": [content] })
    }

    fn handle_explain_rule(&self, arguments: &Value) -> Value {
        let rule_name = arguments
            .get("rule_name")
            .and_then(|n| n.as_str())
            .unwrap_or("");

        let text = if let Some(rule) = self.engine.explain_rule(rule_name) {
            let kind_str = match rule.kind {
                ReviewRuleKind::CheckFunction => "Check Function (code review rule)",
                ReviewRuleKind::Axiom => "Axiom (formal invariant)",
                ReviewRuleKind::Helper => "Helper Function",
            };

            format!(
                "Rule: {}\n\nKind: {}\nSpecification: {}\nPolicy: {}",
                rule.name,
                kind_str,
                rule.description,
                self.engine.policy_file().display(),
            )
        } else {
            format!(
                "Rule '{}' not found.\n\nAvailable rules:\n{}",
                rule_name,
                self.engine
                    .list_rules()
                    .iter()
                    .map(|r| format!("  - {}", r.name))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        let content = McpToolContent {
            content_type: "text".to_string(),
            text,
        };

        serde_json::json!({ "content": [content] })
    }

    fn handle_describe_standards(&self) -> Value {
        self.log("describe_standards requested");

        let schema = self.engine.describe_schema();

        let mut text = String::new();
        text.push_str("Kleis Code Review Standards\n\n");

        text.push_str(&format!(
            "Policy: {}\n",
            schema
                .get("policy_file")
                .and_then(|p| p.as_str())
                .unwrap_or("?")
        ));

        if let Some(fns) = schema.get("check_functions").and_then(|f| f.as_array()) {
            if !fns.is_empty() {
                text.push_str("\n## Check Functions\n\n");
                text.push_str("Each function receives source code and returns \"pass\" or \"fail: <reason>\".\n\n");
                for f in fns {
                    if let Some(kleis) = f.get("kleis").and_then(|k| k.as_str()) {
                        text.push_str(&format!("```\n{}\n```\n\n", kleis));
                    }
                }
            }
        }

        if let Some(structures) = schema.get("structures").and_then(|s| s.as_array()) {
            if !structures.is_empty() {
                text.push_str("## Structures\n\n");
                for s in structures {
                    let name = s.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                    text.push_str(&format!("### {}\n", name));
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

        let content = McpToolContent {
            content_type: "text".to_string(),
            text,
        };

        serde_json::json!({
            "content": [content],
            "schema": schema,
        })
    }

    fn handle_evaluate(&self, arguments: &Value) -> Value {
        let expr_str = arguments
            .get("expression")
            .and_then(|e| e.as_str())
            .unwrap_or("");

        self.log(&format!("evaluate: {}", expr_str));

        if expr_str.is_empty() {
            let content = McpToolContent {
                content_type: "text".to_string(),
                text: "Error: 'expression' parameter is required".to_string(),
            };
            return serde_json::json!({
                "content": [content],
                "isError": true
            });
        }

        let result = self.engine.evaluate_expression(expr_str);

        let mut text = String::new();
        if let Some(ref value) = result.value {
            text.push_str(value);
        }
        if let Some(verified) = result.verified {
            if verified {
                text = format!("VERIFIED: {}", text);
            } else {
                text = format!("DISPROVED: {}", text);
            }
        }
        if let Some(ref error) = result.error {
            text = format!("Error: {}", error);
        }

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

            let label = if result.verified == Some(true) {
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

        serde_json::json!({
            "content": [content],
            "value": result.value,
            "verified": result.verified,
            "witness": witness_str,
            "witness_bindings": witness_bindings,
            "error": result.error,
        })
    }

    fn log(&self, msg: &str) {
        if self.verbose {
            eprintln!("[kleis-review] {}", msg);
        }
    }
}
