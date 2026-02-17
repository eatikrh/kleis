//! MCP Server — JSON-RPC 2.0 over stdio
//!
//! Implements the Model Context Protocol transport layer.
//! Reads JSON-RPC messages from stdin, dispatches to tool handlers,
//! and writes responses to stdout.

use super::policy::{PolicyEngine, RuleKind};
use super::protocol::{
    self, JsonRpcRequest, JsonRpcResponse, McpCapabilities, McpInitializeResult, McpServerInfo,
    McpToolContent,
};
use serde_json::Value;
use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;

/// MCP Server state
pub struct McpServer {
    policy: PolicyEngine,
    verbose: bool,
}

impl McpServer {
    /// Create a new MCP server with a loaded policy
    pub fn new(policy_path: &PathBuf, verbose: bool) -> Result<Self, String> {
        let policy = PolicyEngine::load(policy_path)?;
        Ok(Self { policy, verbose })
    }

    /// Run the MCP server over stdio (blocking)
    pub fn run(&self) -> Result<(), String> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut reader = stdin.lock();
        let mut writer = stdout.lock();

        self.log("Kleis MCP Policy Server started");
        self.log(&format!("Policy: {}", self.policy.policy_file().display()));

        let stats = self.policy.stats();
        self.log(&format!(
            "Loaded: {} check functions, {} axioms, {} total functions",
            stats.get("check_functions").unwrap_or(&0),
            stats.get("axioms").unwrap_or(&0),
            stats.get("functions").unwrap_or(&0),
        ));

        // MCP message loop: supports both Content-Length framing (LSP-style)
        // and newline-delimited JSON (NDJSON, used by Cursor).
        // Auto-detects the framing from the first byte of each message.
        let mut use_ndjson = false; // detected on first message

        loop {
            let body: Vec<u8> = if use_ndjson {
                // NDJSON mode: read one line = one JSON message
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => return Ok(()), // EOF
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
                // Read the first line to detect framing
                let mut first_line = String::new();
                match reader.read_line(&mut first_line) {
                    Ok(0) => return Ok(()), // EOF
                    Ok(_) => {}
                    Err(e) => return Err(format!("Read error: {}", e)),
                }

                let trimmed = first_line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if trimmed.starts_with('{') {
                    // First message is raw JSON — switch to NDJSON mode permanently
                    use_ndjson = true;
                    self.log("Detected NDJSON transport (Cursor-style)");
                    trimmed.as_bytes().to_vec()
                } else if let Some(len_str) = trimmed.strip_prefix("Content-Length:") {
                    // Content-Length framing (LSP-style)
                    self.log("Detected Content-Length transport (LSP-style)");
                    let content_length: usize = len_str.trim().parse().unwrap_or(0);

                    // Read remaining headers until blank line
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

                    // Read content body
                    let mut buf = vec![0u8; content_length];
                    reader
                        .read_exact(&mut buf)
                        .map_err(|e| format!("Read body error: {}", e))?;
                    buf
                } else {
                    // Unknown line, skip
                    self.log(&format!("Skipping unknown line: {}", trimmed));
                    continue;
                }
            };

            // Parse JSON-RPC request
            let request: JsonRpcRequest = match serde_json::from_slice(&body) {
                Ok(req) => req,
                Err(e) => {
                    self.log(&format!("Invalid JSON-RPC: {}", e));
                    continue;
                }
            };

            self.log(&format!("← {}", request.method));

            // Handle the request
            let response = self.handle_request(&request);

            // Send response (only for requests with an id)
            if let Some(ref response) = response {
                let response_str = serde_json::to_string(response)
                    .map_err(|e| format!("Serialize error: {}", e))?;

                if use_ndjson {
                    // NDJSON: one JSON object per line
                    writer
                        .write_all(response_str.as_bytes())
                        .map_err(|e| format!("Write error: {}", e))?;
                    writer
                        .write_all(b"\n")
                        .map_err(|e| format!("Write error: {}", e))?;
                } else {
                    // Content-Length framing
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

    /// Dispatch a JSON-RPC request to the appropriate handler
    fn handle_request(&self, request: &JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = request.id.clone().unwrap_or(Value::Null);

        match request.method.as_str() {
            // ================================================================
            // MCP Lifecycle
            // ================================================================
            "initialize" => {
                let result = McpInitializeResult {
                    protocol_version: "2024-11-05".to_string(),
                    capabilities: McpCapabilities {
                        tools: Some(serde_json::json!({})),
                    },
                    server_info: McpServerInfo {
                        name: "kleis-policy".to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    },
                };

                Some(JsonRpcResponse::success(
                    id,
                    serde_json::to_value(result).unwrap(),
                ))
            }

            // Notification — no response required
            "notifications/initialized" | "initialized" => None,

            // ================================================================
            // Tool Discovery
            // ================================================================
            "tools/list" => {
                let tools = protocol::tool_definitions();
                Some(JsonRpcResponse::success(
                    id,
                    serde_json::json!({ "tools": tools }),
                ))
            }

            // ================================================================
            // Tool Execution
            // ================================================================
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
                    "check_action" => self.handle_check_action(&arguments),
                    "list_rules" => self.handle_list_rules(),
                    "explain_rule" => self.handle_explain_rule(&arguments),
                    "describe_schema" => self.handle_describe_schema(),
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

            // ================================================================
            // Unknown method
            // ================================================================
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

    /// Handle check_action tool call
    fn handle_check_action(&self, arguments: &Value) -> Value {
        let decision = self.policy.check_action(arguments);

        let action_type = arguments
            .get("action")
            .and_then(|a| a.as_str())
            .unwrap_or("unknown");

        let emoji = if decision.allowed { "✅" } else { "🚫" };
        let status = if decision.allowed {
            "ALLOWED"
        } else {
            "DENIED"
        };

        self.log(&format!(
            "{} {} {} — {}",
            emoji, status, action_type, decision.reason
        ));

        let mut text = format!(
            "{} {} {}\n\nAction: {}\nRule: {}\nReason: {}",
            emoji,
            status,
            action_type,
            serde_json::to_string_pretty(arguments).unwrap_or_default(),
            decision.rule_name.as_deref().unwrap_or("(none)"),
            decision.reason,
        );

        if !decision.preconditions.is_empty() {
            text.push_str("\n\n⚠️ Preconditions (run these first):");
            for (i, step) in decision.preconditions.iter().enumerate() {
                text.push_str(&format!("\n  {}. {}", i + 1, step));
            }
            self.log(&format!(
                "  preconditions: {}",
                decision.preconditions.join(", ")
            ));
        }

        let content = McpToolContent {
            content_type: "text".to_string(),
            text,
        };

        serde_json::json!({
            "content": [content],
            "isError": !decision.allowed
        })
    }

    /// Handle list_rules tool call
    fn handle_list_rules(&self) -> Value {
        let rules = self.policy.list_rules();
        let stats = self.policy.stats();

        let mut text = format!(
            "📋 Kleis Policy Rules ({})\n\nPolicy file: {}\n\n",
            rules.len(),
            self.policy.policy_file().display()
        );

        // Group by kind
        text.push_str("## Check Functions (enforce actions)\n\n");
        for rule in rules
            .iter()
            .filter(|r| matches!(r.kind, RuleKind::CheckFunction))
        {
            text.push_str(&format!("  • {} — {}\n", rule.name, rule.description));
        }

        let precondition_rules: Vec<_> = rules
            .iter()
            .filter(|r| matches!(r.kind, RuleKind::Precondition))
            .collect();
        if !precondition_rules.is_empty() {
            text.push_str("\n## Preconditions (run X before Y)\n\n");
            for rule in precondition_rules {
                text.push_str(&format!("  • {} — {}\n", rule.name, rule.description));
            }
        }

        let axiom_rules: Vec<_> = rules
            .iter()
            .filter(|r| matches!(r.kind, RuleKind::Axiom))
            .collect();
        if !axiom_rules.is_empty() {
            text.push_str("\n## Axioms (formal invariants)\n\n");
            for rule in axiom_rules {
                text.push_str(&format!("  • {} — {}\n", rule.name, rule.description));
            }
        }

        let helper_rules: Vec<_> = rules
            .iter()
            .filter(|r| matches!(r.kind, RuleKind::Define))
            .collect();
        if !helper_rules.is_empty() {
            text.push_str("\n## Helper Functions\n\n");
            for rule in helper_rules {
                text.push_str(&format!("  • {}\n", rule.name));
            }
        }

        text.push_str(&format!(
            "\n---\nTotal: {} rules, {} functions loaded",
            stats.get("total_rules").unwrap_or(&0),
            stats.get("functions").unwrap_or(&0),
        ));

        let content = McpToolContent {
            content_type: "text".to_string(),
            text,
        };

        serde_json::json!({
            "content": [content]
        })
    }

    /// Handle explain_rule tool call
    fn handle_explain_rule(&self, arguments: &Value) -> Value {
        let rule_name = arguments
            .get("rule_name")
            .and_then(|n| n.as_str())
            .unwrap_or("");

        let text = if let Some(rule) = self.policy.explain_rule(rule_name) {
            let kind_str = match rule.kind {
                RuleKind::CheckFunction => "Check Function (enforces agent actions)",
                RuleKind::Precondition => "Precondition (must run X before Y)",
                RuleKind::Axiom => "Axiom (formal invariant verified by Z3)",
                RuleKind::Define => "Helper Function (used by check functions)",
            };

            format!(
                "📖 Rule: {}\n\nKind: {}\nSpecification: {}\nPolicy file: {}",
                rule.name,
                kind_str,
                rule.description,
                self.policy.policy_file().display(),
            )
        } else {
            format!(
                "❓ Rule '{}' not found.\n\nAvailable rules:\n{}",
                rule_name,
                self.policy
                    .list_rules()
                    .iter()
                    .map(|r| format!("  • {}", r.name))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        let content = McpToolContent {
            content_type: "text".to_string(),
            text,
        };

        serde_json::json!({
            "content": [content]
        })
    }

    // ========================================================================
    // Schema Introspection
    // ========================================================================

    /// Handle describe_schema tool call
    fn handle_describe_schema(&self) -> Value {
        self.log("describe_schema requested");

        let schema = self.policy.describe_schema();

        // Build a rich, agent-readable summary that teaches the agent the
        // vocabulary it needs to synthesize propositions.
        let mut text = String::new();
        text.push_str("📐 Kleis Schema\n\n");

        // ---- Stats ----
        let stats = schema
            .get("stats")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        text.push_str(&format!(
            "Policy: {}\nStructures: {}, Data types: {}, Functions: {}, Axioms: {}\n",
            schema
                .get("policy_file")
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
        ));

        // ---- Structures with axioms ----
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

        // ---- Data types ----
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

        // ---- Check functions (full Kleis source) ----
        if let Some(fns) = schema.get("check_functions").and_then(|f| f.as_array()) {
            if !fns.is_empty() {
                text.push_str("## Policy Check Functions\n\n");
                text.push_str("These return \"allow\" or \"deny\". Call via `evaluate`.\n\n");
                for f in fns {
                    if let Some(kleis) = f.get("kleis").and_then(|k| k.as_str()) {
                        text.push_str(&format!("```\n{}\n```\n\n", kleis));
                    }
                }
            }
        }

        // ---- Precondition functions (full Kleis source) ----
        if let Some(fns) = schema
            .get("precondition_functions")
            .and_then(|f| f.as_array())
        {
            if !fns.is_empty() {
                text.push_str("## Precondition Functions\n\n");
                text.push_str(
                    "Return \"none\" or a command to run first. Evaluated via `check_action`.\n\n",
                );
                for f in fns {
                    if let Some(kleis) = f.get("kleis").and_then(|k| k.as_str()) {
                        text.push_str(&format!("```\n{}\n```\n\n", kleis));
                    }
                }
            }
        }

        // ---- Verifiable propositions ----
        if let Some(props) = schema
            .get("verifiable_propositions")
            .and_then(|p| p.as_array())
        {
            if !props.is_empty() {
                text.push_str("## Verifiable Propositions\n\n");
                text.push_str(
                    "You can send these to `evaluate` to verify properties of the policy.\n\
                     Propositions with ∀/∃ are routed to Z3 automatically.\n\n",
                );
                for p in props {
                    let kleis = p.get("kleis").and_then(|k| k.as_str()).unwrap_or("?");
                    let desc = p.get("description").and_then(|d| d.as_str()).unwrap_or("");
                    let hint = p.get("hint").and_then(|h| h.as_str()).unwrap_or("evaluate");
                    text.push_str(&format!("- `{}` — {} [{}]\n", kleis, desc, hint,));
                }
                text.push('\n');
                text.push_str(
                    "You can also synthesize your own propositions using the same syntax.\n\
                     Use ∀(x : Type). ... for universal claims, function calls for concrete checks.\n"
                );
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

    // ========================================================================
    // General Expression Evaluation (with Z3 for propositions)
    // ========================================================================

    /// Handle evaluate tool call.
    ///
    /// Delegates to `PolicyEngine::evaluate_expression` which:
    /// - For concrete expressions: uses `eval_concrete`
    /// - For propositions (∀, ∃, →, etc.): uses the evaluator's `assert`
    ///   pipeline which tries concrete evaluation first, then Z3
    fn handle_evaluate(&self, arguments: &Value) -> Value {
        let expr_str = arguments
            .get("expression")
            .and_then(|e| e.as_str())
            .unwrap_or("");

        self.log(&format!("evaluate: {}", expr_str));

        let result = self.policy.evaluate_expression(expr_str);

        // Error case
        if let Some(ref err) = result.error {
            // If we also have a value, it's a partial result (e.g. Unknown with context)
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

        // Proposition verification result
        if let Some(verified) = result.verified {
            let (emoji, status) = if verified {
                ("✅", "VERIFIED")
            } else {
                ("❌", "DISPROVED")
            };
            let value_str = result.value.as_deref().unwrap_or("?");

            self.log(&format!("  → {} {} — {}", emoji, status, value_str));

            let mut text = format!(
                "{} {}\n\nProposition: {}\nResult: {}",
                emoji, status, expr_str, value_str
            );

            // Render structured witness as Kleis expressions
            // For verified existentials: "Witness" (satisfying assignment)
            // For disproved universals: "Counterexample" (violating assignment)
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
                    // No structured bindings — fall back to raw
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

        // Concrete evaluation result
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

    /// Log to stderr (MCP uses stdio, so logs go to stderr)
    fn log(&self, msg: &str) {
        if self.verbose {
            eprintln!("[kleis-mcp] {}", msg);
        }
    }
}
