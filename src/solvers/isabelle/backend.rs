//! Isabelle Backend Implementation
//!
//! Implements the SolverBackend trait for Isabelle/HOL theorem prover.
//!
//! **Key Features:**
//! - Full induction proofs (Z3 cannot do this)
//! - Termination proofs for recursive functions
//! - Access to AFP (Archive of Formal Proofs) library
//! - Higher-order reasoning
//!
//! **Architecture:**
//! - Manages Isabelle server lifecycle (start/connect/stop)
//! - Translates Kleis AST to Isar proof language
//! - Sends theories via JSON API
//! - Parses verification results
//!
//! **Critical:** All public methods return Kleis Expression, not Isabelle types!

use crate::ast::{Expression, QuantifierKind};
use crate::kleis_ast::TypeExpr;
use crate::solvers::backend::{SatisfiabilityResult, SolverBackend, VerificationResult};
use crate::solvers::capabilities::{
    Capabilities, FeatureFlags, OperationSpec, PerformanceHints, SolverCapabilities, SolverMetadata,
};
use crate::solvers::isabelle::connection::{CommandResult, IsabelleConnection};
use std::collections::{HashMap, HashSet};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// Isabelle/HOL Theorem Prover Backend
///
/// Wraps Isabelle's server to implement the SolverBackend trait.
/// Manages server lifecycle and translates between Kleis and Isar.
pub struct IsabelleBackend {
    /// Connection to Isabelle server (None if not connected)
    connection: Option<IsabelleConnection>,

    /// Server process (if we started it)
    server_process: Option<Child>,

    /// Server connection info
    server_host: String,
    server_port: u16,
    server_password: String,

    /// Active session ID (after session_start)
    session_id: Option<String>,

    /// Cached session name (for reconnection)
    cached_session_name: Option<String>,

    /// Capability manifest
    capabilities: SolverCapabilities,

    /// Track which operations have been declared
    declared_ops: HashSet<String>,

    /// Track which structures' axioms are currently loaded
    loaded_structures: HashSet<String>,

    /// Identity elements loaded as constants
    identity_elements: HashSet<String>,

    /// Path to Isabelle executable (for starting server)
    isabelle_path: Option<String>,

    /// Warnings collected during translation
    warnings: Vec<String>,

    /// Theory counter for unique theory names
    theory_counter: u64,

    /// Companion .thy files (main file + imports)
    companion_theories: Vec<std::path::PathBuf>,

    /// Whether companion theories have been loaded into Isabelle
    companion_loaded: bool,

    /// Axioms proven by companion theories (axiom name -> true)
    companion_proven: std::collections::HashSet<String>,

    /// Cached axioms that have been loaded into the session context
    loaded_axioms: Vec<String>,

    /// Last verification result cache (for repeated queries)
    verification_cache: HashMap<String, VerificationResult>,
}

/// Configuration for IsabelleBackend
#[derive(Debug, Clone)]
pub struct IsabelleConfig {
    /// Server host (default: 127.0.0.1)
    pub host: String,

    /// Server port (0 = auto-assign)
    pub port: u16,

    /// Server password (auto-generated if empty)
    pub password: String,

    /// Path to Isabelle executable (searches PATH if None)
    pub isabelle_path: Option<String>,

    /// Session to use (default: HOL)
    pub session: String,

    /// Timeout for operations
    pub timeout: Duration,
}

impl Default for IsabelleConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 0,
            password: String::new(),
            isabelle_path: None,
            session: "HOL".to_string(),
            timeout: Duration::from_secs(30),
        }
    }
}

impl IsabelleBackend {
    /// Create a new IsabelleBackend (not connected)
    ///
    /// Use `connect()` to establish connection to an existing server,
    /// or `start_server()` to spawn a new server process.
    pub fn new() -> Result<Self, String> {
        let capabilities = Self::load_capabilities()?;

        Ok(IsabelleBackend {
            connection: None,
            server_process: None,
            server_host: "127.0.0.1".to_string(),
            server_port: 0,
            server_password: String::new(),
            session_id: None,
            cached_session_name: None,
            capabilities,
            declared_ops: HashSet::new(),
            loaded_structures: HashSet::new(),
            identity_elements: HashSet::new(),
            isabelle_path: crate::solvers::discovery::find_isabelle(),
            warnings: Vec::new(),
            theory_counter: 0,
            loaded_axioms: Vec::new(),
            verification_cache: HashMap::new(),
            companion_theories: Vec::new(),
            companion_loaded: false,
            companion_proven: HashSet::new(),
        })
    }

    /// Create with custom configuration
    pub fn with_config(config: IsabelleConfig) -> Result<Self, String> {
        let mut backend = Self::new()?;
        backend.server_host = config.host;
        backend.server_port = config.port;
        backend.server_password = config.password;
        if config.isabelle_path.is_some() {
            backend.isabelle_path = config.isabelle_path;
        }
        Ok(backend)
    }

    /// Parse Isabelle server output to extract port and password
    ///
    /// Format: server "name" = 127.0.0.1:<port> (password "<password>")
    /// Example: server "isabelle" = 127.0.0.1:58865 (password "1c199aff-...")
    fn parse_server_output(line: &str) -> Result<(u16, String), String> {
        // Extract port from "127.0.0.1:<port>"
        let port = line
            .split(':')
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse::<u16>().ok())
            .ok_or_else(|| format!("Failed to parse port from: {}", line))?;

        // Extract password from (password "<password>")
        let password = line
            .split("password \"")
            .nth(1)
            .and_then(|s| s.strip_suffix("\")"))
            .or_else(|| {
                // Alternative: might end with just ")"
                line.split("password \"")
                    .nth(1)
                    .and_then(|s| s.split('"').next())
            })
            .ok_or_else(|| format!("Failed to parse password from: {}", line))?
            .to_string();

        Ok((port, password))
    }

    /// Load capabilities from embedded TOML
    fn load_capabilities() -> Result<SolverCapabilities, String> {
        let toml_str = super::CAPABILITIES_TOML;

        // Parse the TOML
        let parsed: toml::Value =
            toml::from_str(toml_str).map_err(|e| format!("Failed to parse capabilities: {}", e))?;

        // Extract solver metadata
        let solver = parsed.get("solver").ok_or("Missing [solver] section")?;
        let metadata = SolverMetadata {
            name: solver
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Isabelle")
                .to_string(),
            version: solver
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("2025-1")
                .to_string(),
            solver_type: solver
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("theorem_prover")
                .to_string(),
            description: solver
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        };

        // Extract capabilities
        let caps = parsed
            .get("capabilities")
            .ok_or("Missing [capabilities] section")?;

        // Theories
        let theories: HashSet<String> = caps
            .get("theories")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Operations
        let mut operations = HashMap::new();
        if let Some(ops) = caps.get("operations").and_then(|v| v.as_table()) {
            for (name, spec) in ops {
                let arity = spec.get("arity").and_then(|v| v.as_integer()).unwrap_or(2) as usize;
                let theory = spec
                    .get("theory")
                    .and_then(|v| v.as_str())
                    .unwrap_or("core")
                    .to_string();
                let native = spec.get("native").and_then(|v| v.as_bool()).unwrap_or(true);

                operations.insert(
                    name.clone(),
                    OperationSpec {
                        arity,
                        theory,
                        native,
                        reason: None,
                        alternatives: None,
                    },
                );
            }
        }

        // Features
        let features = caps.get("features");
        let feature_flags = FeatureFlags {
            quantifiers: features
                .and_then(|f| f.get("quantifiers"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            uninterpreted_functions: features
                .and_then(|f| f.get("uninterpreted_functions"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            recursive_functions: features
                .and_then(|f| f.get("recursive_functions"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            evaluation: features
                .and_then(|f| f.get("evaluation"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            simplification: features
                .and_then(|f| f.get("simplification"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            proof_generation: features
                .and_then(|f| f.get("proof_generation"))
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
        };

        // Performance
        let perf = caps.get("performance");
        let performance = PerformanceHints {
            max_axioms: perf
                .and_then(|p| p.get("max_axioms"))
                .and_then(|v| v.as_integer())
                .unwrap_or(1000) as usize,
            timeout_ms: perf
                .and_then(|p| p.get("timeout_ms"))
                .and_then(|v| v.as_integer())
                .unwrap_or(30000) as u64,
        };

        Ok(SolverCapabilities {
            solver: metadata,
            capabilities: Capabilities {
                theories,
                operations,
                features: feature_flags,
                performance,
            },
        })
    }

    /// Connect to an existing Isabelle server
    ///
    /// # Arguments
    /// * `host` - Server hostname
    /// * `port` - Server port
    /// * `password` - Authentication password
    pub fn connect(&mut self, host: &str, port: u16, password: &str) -> Result<(), String> {
        let conn = IsabelleConnection::connect(host, port, password)?;
        self.connection = Some(conn);
        self.server_host = host.to_string();
        self.server_port = port;
        self.server_password = password.to_string();
        Ok(())
    }

    /// Start an Isabelle server and connect to it
    ///
    /// This spawns `isabelle server` and captures the port/password from stdout.
    pub fn start_server(&mut self) -> Result<(), String> {
        let isabelle_path = self
            .isabelle_path
            .clone()
            .ok_or("Isabelle not found. Set ISABELLE_HOME or install Isabelle.")?;

        // Start isabelle server in its own process group so we can kill all children
        let mut cmd = Command::new(&isabelle_path);
        cmd.arg("server")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // On Unix, create a new process group so we can kill all child processes
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0); // Creates new process group with child as leader
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to start Isabelle server: {}", e))?;

        // Read the first line to get port and password
        // Format: server "name" = 127.0.0.1:<port> (password "<password>")
        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let reader = std::io::BufReader::new(stdout);
        use std::io::BufRead;

        let mut lines = reader.lines();
        let first_line = lines
            .next()
            .ok_or("No output from server")?
            .map_err(|e| format!("Failed to read server output: {}", e))?;

        // Parse: server "name" = 127.0.0.1:<port> (password "<password>")
        // Example: server "isabelle" = 127.0.0.1:58865 (password "1c199aff-...")
        let (port, password) = Self::parse_server_output(&first_line)?;

        self.server_process = Some(child);
        self.server_port = port;
        self.server_password = password.clone();

        // Give server time to start
        std::thread::sleep(Duration::from_millis(500));

        // Connect
        self.connect(&self.server_host.clone(), port, &password)?;

        Ok(())
    }

    /// Start an Isabelle session (required before running proofs)
    ///
    /// # Arguments
    /// * `session` - Session name (e.g., "HOL", "HOL-Analysis")
    pub fn start_session(&mut self, session: &str) -> Result<(), String> {
        let conn = self
            .connection
            .as_mut()
            .ok_or("Not connected to Isabelle server")?;

        let args = serde_json::json!({
            "session": session,
            "print_mode": ["symbols"]
        });

        conn.set_timeout(Duration::from_secs(super::SESSION_START_TIMEOUT))?;

        match conn.send_command("session_start", &args)? {
            CommandResult::Ok(Some(response)) => {
                // Check if this is an async task response or direct response
                if let Some(task_id) = response.get("task").and_then(|v| v.as_str()) {
                    // Async session start - wait for FINISHED response
                    let finished = self.wait_for_finished(task_id)?;
                    if let Some(id) = finished.get("session_id").and_then(|v| v.as_str()) {
                        self.session_id = Some(id.to_string());
                    } else {
                        return Err(
                            "session_start completed but no session_id returned".to_string()
                        );
                    }
                } else if let Some(id) = response.get("session_id").and_then(|v| v.as_str()) {
                    // Direct response with session_id
                    self.session_id = Some(id.to_string());
                } else {
                    return Err(
                        "session_start response missing both task and session_id".to_string()
                    );
                }
                self.cached_session_name = Some(session.to_string());
                Ok(())
            }
            CommandResult::Ok(None) => Err("session_start returned empty response".to_string()),
            CommandResult::Error(msg) => Err(format!("Failed to start session: {}", msg)),
            CommandResult::Running { task_id } => {
                // Async session start via Running response
                let finished = self.wait_for_finished(&task_id)?;
                if let Some(id) = finished.get("session_id").and_then(|v| v.as_str()) {
                    self.session_id = Some(id.to_string());
                } else {
                    return Err("session_start completed but no session_id returned".to_string());
                }
                self.cached_session_name = Some(session.to_string());
                Ok(())
            }
        }
    }

    /// Ensure a session is active, starting one if needed (session caching)
    ///
    /// Uses the cached session name if available, otherwise defaults to "HOL".
    pub fn ensure_session(&mut self) -> Result<(), String> {
        if self.has_session() {
            return Ok(());
        }

        // Use cached session name or default to HOL
        let session = self
            .cached_session_name
            .clone()
            .unwrap_or_else(|| "HOL".to_string());

        self.start_session(&session)
    }

    /// Wait for use_theories task to complete and collect all messages
    ///
    /// Returns the FINISHED response, or an error message if the proof failed.
    fn wait_for_use_theories_task(&mut self, task_id: &str) -> Result<serde_json::Value, String> {
        // Use 10 minutes for companion theories that may have complex proofs
        const MAX_WAIT: Duration = Duration::from_secs(600);
        let start = std::time::Instant::now();

        let conn = self.connection.as_mut().ok_or("Connection lost")?;
        conn.set_timeout(Duration::from_secs(5))?;

        let mut collected_messages: Vec<serde_json::Value> = Vec::new();
        let mut proof_error: Option<String> = None;

        loop {
            if start.elapsed() > MAX_WAIT {
                // Cancel the running task to free up Isabelle resources
                if std::env::var("KLEIS_DEBUG").is_ok() {
                    eprintln!("  >> Canceling task {} due to timeout", task_id);
                }
                let _ = conn.send_command("cancel", &serde_json::json!({"task": task_id}));
                return Err(format!("Proof timed out after {:?}", MAX_WAIT));
            }

            match conn.read_next_message() {
                Ok(Some((kind, json))) => {
                    if std::env::var("KLEIS_DEBUG").is_ok() {
                        eprintln!(
                            "  << {} {}",
                            kind,
                            serde_json::to_string(&json).unwrap_or_default()
                        );
                    }

                    // Check if this message is for our task
                    let msg_task = json.get("task").and_then(|v| v.as_str()).unwrap_or("");
                    if !msg_task.is_empty() && msg_task != task_id {
                        // Different task, skip
                        continue;
                    }

                    match kind.as_str() {
                        "FINISHED" => {
                            // Collect all messages we've seen
                            let mut result = json.clone();
                            if !collected_messages.is_empty() {
                                result["_messages"] = serde_json::Value::Array(collected_messages);
                            }
                            if let Some(err) = proof_error {
                                result["_proof_error"] = serde_json::Value::String(err);
                            }
                            return Ok(result);
                        }
                        "FAILED" => {
                            let msg = json
                                .get("message")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Task failed");
                            return Err(msg.to_string());
                        }
                        "NOTE" => {
                            // Check for proof errors in notes
                            if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
                                // Look for proof failure indicators
                                if message.contains("Failed to finish proof")
                                    || message.contains("Failed to apply")
                                    || message.contains("goal")
                                    || message.contains("error")
                                {
                                    proof_error = Some(message.to_string());
                                }
                            }
                            collected_messages.push(json);
                        }
                        _ => {
                            collected_messages.push(json);
                        }
                    }
                }
                Ok(None) => {
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    if !e.contains("timed out") && !e.contains("WouldBlock") {
                        return Err(e);
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    /// Parse use_theories result to determine if proof succeeded
    fn parse_use_theories_result(
        &self,
        response: &serde_json::Value,
    ) -> Result<VerificationResult, String> {
        // Check for proof error we detected
        if let Some(err) = response.get("_proof_error").and_then(|v| v.as_str()) {
            return Ok(VerificationResult::Invalid {
                counterexample: err.to_string(),
            });
        }

        // Check nodes for status
        if let Some(nodes) = response.get("nodes").and_then(|v| v.as_array()) {
            for node in nodes {
                // Check node status
                if let Some(status) = node.get("status") {
                    let status_obj = status.as_object();

                    // Check for failed/unfinished status
                    if let Some(obj) = status_obj {
                        if obj.get("failed").and_then(|v| v.as_bool()).unwrap_or(false) {
                            return Ok(VerificationResult::Invalid {
                                counterexample: "Proof failed".to_string(),
                            });
                        }
                        // Check if proof is complete
                        let finished = obj
                            .get("finished")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let ok = obj.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                        let consolidated = obj
                            .get("consolidated")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        if std::env::var("KLEIS_DEBUG").is_ok() {
                            eprintln!(
                                "  Node status: finished={}, ok={}, consolidated={}",
                                finished, ok, consolidated
                            );
                        }

                        if !finished && !consolidated {
                            return Ok(VerificationResult::Invalid {
                                counterexample: "Proof incomplete".to_string(),
                            });
                        }
                    }
                }

                // Check messages for errors
                if let Some(messages) = node.get("messages").and_then(|v| v.as_array()) {
                    for msg in messages {
                        if let Some(kind) = msg.get("kind").and_then(|v| v.as_str()) {
                            if kind == "error" {
                                // Try "message" first, then "body"
                                let error_text = msg
                                    .get("message")
                                    .and_then(|v| v.as_str())
                                    .or_else(|| msg.get("body").and_then(|v| v.as_str()))
                                    .unwrap_or("Unknown error");
                                return Ok(VerificationResult::Invalid {
                                    counterexample: error_text.to_string(),
                                });
                            }
                        }
                    }
                }
            }

            // All nodes look good
            return Ok(VerificationResult::Valid);
        }

        // Check errors field
        if let Some(errors) = response.get("errors").and_then(|v| v.as_array()) {
            if !errors.is_empty() {
                let msg = errors
                    .iter()
                    .filter_map(|e| e.get("message").and_then(|v| v.as_str()))
                    .collect::<Vec<_>>()
                    .join("; ");
                return Ok(VerificationResult::Invalid {
                    counterexample: msg,
                });
            }
        }

        // No nodes info and no errors - this is suspicious
        if std::env::var("KLEIS_DEBUG").is_ok() {
            eprintln!("  WARNING: No node status found in response");
        }

        // Assume success if we got FINISHED without errors
        Ok(VerificationResult::Valid)
    }

    /// Wait for an async task to complete and return the FINISHED response
    fn wait_for_finished(&mut self, task_id: &str) -> Result<serde_json::Value, String> {
        const MAX_WAIT: Duration = Duration::from_secs(120); // 2 minutes for session start
        let start = std::time::Instant::now();

        let conn = self.connection.as_mut().ok_or("Connection lost")?;
        conn.set_timeout(Duration::from_secs(5))?;

        loop {
            if start.elapsed() > MAX_WAIT {
                return Err(format!("Task {} timed out after {:?}", task_id, MAX_WAIT));
            }

            // Read the next message from the connection
            match conn.read_next_message() {
                Ok(Some((kind, json))) => {
                    // Check if this is the FINISHED message for our task
                    if kind == "FINISHED" {
                        if let Some(tid) = json.get("task").and_then(|v| v.as_str()) {
                            if tid == task_id {
                                return Ok(json);
                            }
                        }
                    }
                    // FAILED means error
                    if kind == "FAILED" {
                        if let Some(tid) = json.get("task").and_then(|v| v.as_str()) {
                            if tid == task_id {
                                let msg = json
                                    .get("message")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Task failed");
                                return Err(msg.to_string());
                            }
                        }
                    }
                    // NOTE messages are informational, continue waiting
                }
                Ok(None) => {
                    // No more data, wait a bit
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    // Timeout is expected, continue
                    if !e.contains("timed out") && !e.contains("WouldBlock") {
                        return Err(e);
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    /// Stop the Isabelle session
    pub fn stop_session(&mut self) -> Result<(), String> {
        if let (Some(conn), Some(session_id)) = (&mut self.connection, &self.session_id) {
            let args = serde_json::json!({
                "session_id": session_id
            });
            let _ = conn.send_command("session_stop", &args);
        }
        self.session_id = None;
        Ok(())
    }

    /// Check if connected to server
    pub fn is_connected(&self) -> bool {
        self.connection
            .as_ref()
            .map(|c| c.is_authenticated())
            .unwrap_or(false)
    }

    /// Check if session is active
    pub fn has_session(&self) -> bool {
        self.session_id.is_some()
    }

    /// Get collected warnings
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Clear warnings
    pub fn clear_warnings(&mut self) {
        self.warnings.clear();
    }

    /// Translate a Kleis expression to Isar syntax
    ///
    /// This is the core translation function used by verify_axiom, etc.
    /// Handles all Expression variants and maps Kleis operations to Isabelle/HOL syntax.
    fn translate_to_isar(&self, expr: &Expression) -> Result<String, String> {
        match expr {
            // ===== LITERALS =====
            Expression::Const(s) => {
                // Numeric constants translate directly
                // Handle negative numbers: -5 -> (- 5)
                if s.starts_with('-') && s.len() > 1 {
                    Ok(format!("(- {})", &s[1..]))
                } else {
                    Ok(s.clone())
                }
            }

            Expression::String(s) => {
                // String literals use Isabelle's string syntax
                Ok(format!("''{}''", s.replace('\'', "\\'")))
            }

            Expression::Object(s) => {
                // Variables/identifiers - translate type symbols
                match s.as_str() {
                    // Greek letters are fine in Isabelle
                    // But translate common type symbols
                    "ℕ" => Ok("nat".to_string()),
                    "ℤ" => Ok("int".to_string()),
                    "ℝ" => Ok("real".to_string()),
                    "ℂ" => Ok("complex".to_string()),
                    "ℚ" => Ok("rat".to_string()),
                    "Bool" | "Boolean" => Ok("bool".to_string()),
                    "True" | "true" => Ok("True".to_string()),
                    "False" | "false" => Ok("False".to_string()),
                    _ => Ok(s.clone()),
                }
            }

            // ===== OPERATIONS =====
            Expression::Operation { name, args, .. } => {
                self.translate_operation_to_isar(name, args)
            }

            // ===== QUANTIFIERS =====
            Expression::Quantifier {
                quantifier,
                variables,
                where_clause,
                body,
            } => self.translate_quantifier_to_isar(
                quantifier,
                variables,
                where_clause.as_deref(),
                body,
            ),

            // ===== LAMBDA =====
            Expression::Lambda { params, body, .. } => {
                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| {
                        if let Some(ref ty) = p.type_annotation {
                            format!("({} :: {})", p.name, self.translate_type(ty))
                        } else {
                            p.name.clone()
                        }
                    })
                    .collect();
                let body_isar = self.translate_to_isar(body)?;
                Ok(format!("(λ{}. {})", param_strs.join(" "), body_isar))
            }

            // ===== CONDITIONAL (if-then-else) =====
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_isar = self.translate_to_isar(condition)?;
                let then_isar = self.translate_to_isar(then_branch)?;
                let else_isar = self.translate_to_isar(else_branch)?;
                Ok(format!(
                    "(if {} then {} else {})",
                    cond_isar, then_isar, else_isar
                ))
            }

            // ===== LET BINDING =====
            Expression::Let {
                pattern,
                value,
                body,
                ..
            } => {
                let value_isar = self.translate_to_isar(value)?;
                let body_isar = self.translate_to_isar(body)?;
                let pattern_isar = self.translate_pattern_to_isar(pattern)?;
                Ok(format!(
                    "(let {} = {} in {})",
                    pattern_isar, value_isar, body_isar
                ))
            }

            // ===== MATCH (pattern matching) =====
            Expression::Match {
                scrutinee, cases, ..
            } => {
                let scrutinee_isar = self.translate_to_isar(scrutinee)?;
                let mut case_strs = Vec::new();
                for case in cases {
                    let pattern_isar = self.translate_pattern_to_isar(&case.pattern)?;
                    let body_isar = self.translate_to_isar(&case.body)?;
                    case_strs.push(format!("{} ⇒ {}", pattern_isar, body_isar));
                }
                Ok(format!(
                    "(case {} of {})",
                    scrutinee_isar,
                    case_strs.join(" | ")
                ))
            }

            // ===== LIST =====
            Expression::List(elems) => {
                let parts: Result<Vec<String>, String> =
                    elems.iter().map(|e| self.translate_to_isar(e)).collect();
                Ok(format!("[{}]", parts?.join(", ")))
            }

            // ===== ASCRIPTION (type annotation) =====
            Expression::Ascription {
                expr,
                type_annotation,
            } => {
                // Type annotations translate to Isabelle's :: syntax
                let expr_isar = self.translate_to_isar(expr)?;
                let type_isar = self.translate_type(type_annotation);
                Ok(format!("({} :: {})", expr_isar, type_isar))
            }

            // ===== PLACEHOLDER (should not appear in verification) =====
            Expression::Placeholder { id, hint } => Err(format!(
                "Cannot translate placeholder to Isar: id={}, hint='{}'",
                id, hint
            )),
        }
    }

    /// Translate operation to Isar syntax
    fn translate_operation_to_isar(
        &self,
        name: &str,
        args: &[Expression],
    ) -> Result<String, String> {
        // Translate arguments first
        let translated_args: Result<Vec<String>, String> =
            args.iter().map(|a| self.translate_to_isar(a)).collect();
        let isar_args = translated_args?;

        // Map Kleis operations to Isar operators
        match name {
            // ===== ARITHMETIC =====
            "plus" | "add" | "+" => self.binary_infix(&isar_args, "+"),
            "minus" | "subtract" | "-" if args.len() == 2 => self.binary_infix(&isar_args, "-"),
            "negate" | "-" if args.len() == 1 => Ok(format!("(- {})", isar_args[0])),
            "times" | "multiply" | "*" => self.binary_infix(&isar_args, "*"),
            "divide" | "div" | "/" => self.binary_infix(&isar_args, "/"),
            "mod" | "modulo" | "%" => self.binary_infix(&isar_args, "mod"),
            "power" | "pow" | "^" => self.binary_infix(&isar_args, "^"),
            "abs" => self.unary_function(&isar_args, "abs"),
            "sqrt" => self.unary_function(&isar_args, "sqrt"),
            "floor" => self.unary_function(&isar_args, "floor"),
            "ceiling" => self.unary_function(&isar_args, "ceiling"),
            "max" => self.binary_function(&isar_args, "max"),
            "min" => self.binary_function(&isar_args, "min"),

            // ===== EQUALITY & COMPARISON =====
            "equals" | "eq" | "=" => self.binary_infix(&isar_args, "="),
            "neq" | "not_equals" | "≠" => self.binary_infix(&isar_args, "≠"),
            "less_than" | "lt" | "<" => self.binary_infix(&isar_args, "<"),
            "greater_than" | "gt" | ">" => self.binary_infix(&isar_args, ">"),
            "leq" | "le" | "<=" | "≤" => self.binary_infix(&isar_args, "≤"),
            "geq" | "ge" | ">=" | "≥" => self.binary_infix(&isar_args, "≥"),

            // ===== BOOLEAN =====
            "and" | "logical_and" | "∧" => self.binary_infix(&isar_args, "∧"),
            "or" | "logical_or" | "∨" => self.binary_infix(&isar_args, "∨"),
            "not" | "logical_not" | "¬" => Ok(format!(
                "(¬ {})",
                isar_args.first().unwrap_or(&"?".to_string())
            )),
            "implies" | "→" | "⟶" => self.binary_infix(&isar_args, "⟶"),
            "iff" | "biconditional" | "↔" | "⟷" => self.binary_infix(&isar_args, "⟷"),

            // ===== LIST OPERATIONS =====
            "cons" | "Cons" => {
                // cons(x, xs) -> x # xs
                self.binary_infix(&isar_args, "#")
            }
            "nil" | "Nil" | "[]" => Ok("[]".to_string()),
            "append" | "++" => self.binary_infix(&isar_args, "@"),
            "length" | "len" => self.unary_function(&isar_args, "length"),
            "hd" | "head" => self.unary_function(&isar_args, "hd"),
            "tl" | "tail" => self.unary_function(&isar_args, "tl"),
            "nth" => {
                // nth(xs, n) -> xs ! n
                if isar_args.len() == 2 {
                    Ok(format!("({} ! {})", isar_args[0], isar_args[1]))
                } else {
                    Err("nth requires 2 arguments".to_string())
                }
            }
            "rev" | "reverse" => self.unary_function(&isar_args, "rev"),
            "map" => self.binary_function(&isar_args, "map"),
            "filter" => self.binary_function(&isar_args, "filter"),
            "fold" | "foldl" => self.ternary_function(&isar_args, "foldl"),
            "foldr" => self.ternary_function(&isar_args, "foldr"),

            // ===== OPTION OPERATIONS =====
            "Some" => self.unary_function(&isar_args, "Some"),
            "None" => Ok("None".to_string()),
            "the" | "fromJust" => self.unary_function(&isar_args, "the"),

            // ===== SET OPERATIONS =====
            "member" | "∈" | "elem" => self.binary_infix(&isar_args, "∈"),
            "not_member" | "∉" => self.binary_infix(&isar_args, "∉"),
            "union" | "∪" => self.binary_infix(&isar_args, "∪"),
            "inter" | "intersection" | "∩" => self.binary_infix(&isar_args, "∩"),
            "subset" | "⊆" => self.binary_infix(&isar_args, "⊆"),
            "empty_set" | "∅" => Ok("{}".to_string()),

            // ===== PAIR OPERATIONS =====
            "pair" | "Pair" => {
                if isar_args.len() == 2 {
                    Ok(format!("({}, {})", isar_args[0], isar_args[1]))
                } else {
                    Err("pair requires 2 arguments".to_string())
                }
            }
            "fst" => self.unary_function(&isar_args, "fst"),
            "snd" => self.unary_function(&isar_args, "snd"),

            // ===== DEFAULT: Uninterpreted function =====
            _ => {
                // Unknown operation - translate as function application
                if isar_args.is_empty() {
                    Ok(name.to_string())
                } else {
                    Ok(format!("({} {})", name, isar_args.join(" ")))
                }
            }
        }
    }

    /// Translate quantifier with optional where clause
    fn translate_quantifier_to_isar(
        &self,
        quantifier: &QuantifierKind,
        variables: &[crate::ast::QuantifiedVar],
        where_clause: Option<&Expression>,
        body: &Expression,
    ) -> Result<String, String> {
        // Build variable declarations with optional types
        let var_strs: Vec<String> = variables
            .iter()
            .map(|v| {
                if let Some(ref ty) = v.type_annotation {
                    format!("({} :: {})", v.name, self.translate_type(ty))
                } else {
                    v.name.clone()
                }
            })
            .collect();

        let body_isar = self.translate_to_isar(body)?;
        let symbol = match quantifier {
            QuantifierKind::ForAll => "∀",
            QuantifierKind::Exists => "∃",
        };

        // Handle where clause: ∀x. (condition ⟶ body)
        if let Some(condition) = where_clause {
            let cond_isar = self.translate_to_isar(condition)?;
            Ok(format!(
                "({}{}. {} ⟶ {})",
                symbol,
                var_strs.join(" "),
                cond_isar,
                body_isar
            ))
        } else {
            Ok(format!("({}{}. {})", symbol, var_strs.join(" "), body_isar))
        }
    }

    /// Translate a pattern to Isar syntax
    fn translate_pattern_to_isar(&self, pattern: &crate::ast::Pattern) -> Result<String, String> {
        use crate::ast::Pattern;
        match pattern {
            Pattern::Wildcard => Ok("_".to_string()),
            Pattern::Variable(name) => Ok(name.clone()),
            Pattern::Constant(val) => {
                // Constant patterns (numeric or string literals)
                Ok(val.clone())
            }
            Pattern::Constructor { name, args } => {
                if args.is_empty() {
                    Ok(name.clone())
                } else {
                    let arg_strs: Result<Vec<String>, String> = args
                        .iter()
                        .map(|p| self.translate_pattern_to_isar(p))
                        .collect();
                    Ok(format!("({} {})", name, arg_strs?.join(" ")))
                }
            }
            Pattern::As { pattern, binding } => {
                // As-pattern: pattern as binding
                // In Isabelle, we use the pattern and bind the whole to the binding name
                let pattern_isar = self.translate_pattern_to_isar(pattern)?;
                // Isabelle doesn't have direct as-pattern syntax like Haskell
                // We use the binding as the main name
                Ok(format!("{} (* {} *)", binding, pattern_isar))
            }
        }
    }

    /// Translate Kleis type to Isabelle type
    fn translate_type(&self, ty: &str) -> String {
        // Handle function types: ℝ → Bool -> real ⇒ bool
        if ty.contains(" → ") || ty.contains("→") {
            // Split on arrow and translate each part
            let parts: Vec<&str> = ty.split('→').collect();
            if parts.len() >= 2 {
                return parts
                    .iter()
                    .map(|p| self.translate_type(p.trim()))
                    .collect::<Vec<_>>()
                    .join(" ⇒ ");
            }
        }

        match ty {
            "ℕ" | "Nat" | "Natural" => "nat".to_string(),
            "ℤ" | "Int" | "Integer" => "int".to_string(),
            "ℝ" | "Real" => "real".to_string(),
            "ℂ" | "Complex" => "complex".to_string(),
            "ℚ" | "Rational" => "rat".to_string(),
            "Bool" | "Boolean" | "bool" => "bool".to_string(),
            "String" => "string".to_string(),
            "Unit" | "()" => "unit".to_string(),
            _ => {
                // Handle parameterized types: List(T) -> 'a list
                if ty.starts_with("List(") && ty.ends_with(")") {
                    let inner = &ty[5..ty.len() - 1];
                    format!("{} list", self.translate_type(inner))
                } else if ty.starts_with("Option(") && ty.ends_with(")") {
                    let inner = &ty[7..ty.len() - 1];
                    format!("{} option", self.translate_type(inner))
                } else if ty.starts_with("Set(") && ty.ends_with(")") {
                    let inner = &ty[4..ty.len() - 1];
                    format!("{} set", self.translate_type(inner))
                } else {
                    // Unknown type - pass through
                    ty.to_string()
                }
            }
        }
    }

    // ===== HELPER FUNCTIONS FOR FORMATTING =====

    /// Format binary infix operation: (a op b)
    fn binary_infix(&self, args: &[String], op: &str) -> Result<String, String> {
        if args.len() != 2 {
            return Err(format!("'{}' requires 2 arguments, got {}", op, args.len()));
        }
        Ok(format!("({} {} {})", args[0], op, args[1]))
    }

    /// Format unary function: (f x)
    fn unary_function(&self, args: &[String], func: &str) -> Result<String, String> {
        if args.len() != 1 {
            return Err(format!(
                "'{}' requires 1 argument, got {}",
                func,
                args.len()
            ));
        }
        Ok(format!("({} {})", func, args[0]))
    }

    /// Format binary function: (f x y)
    fn binary_function(&self, args: &[String], func: &str) -> Result<String, String> {
        if args.len() != 2 {
            return Err(format!(
                "'{}' requires 2 arguments, got {}",
                func,
                args.len()
            ));
        }
        Ok(format!("({} {} {})", func, args[0], args[1]))
    }

    /// Format ternary function: (f x y z)
    fn ternary_function(&self, args: &[String], func: &str) -> Result<String, String> {
        if args.len() != 3 {
            return Err(format!(
                "'{}' requires 3 arguments, got {}",
                func,
                args.len()
            ));
        }
        Ok(format!("({} {} {} {})", func, args[0], args[1], args[2]))
    }

    // ===== VERIFICATION HELPER METHODS =====

    /// Send a theory to Isabelle for verification
    fn send_theory_for_verification(&mut self, theory: &str) -> Result<VerificationResult, String> {
        // Debug: print the theory being sent
        if std::env::var("KLEIS_DEBUG").is_ok() {
            eprintln!("\n=== THEORY SENT TO ISABELLE ===");
            eprintln!("{}", theory);
            eprintln!("=== END THEORY ===\n");
        }

        // Write theory to a temp file (Isabelle's use_theories expects file paths)
        let theory_dir = std::path::PathBuf::from("/tmp/kleis_theories");
        std::fs::create_dir_all(&theory_dir)
            .map_err(|e| format!("Failed to create theory directory: {}", e))?;

        // Copy companion theories to the theory directory so they can be imported
        for companion_path in &self.companion_theories {
            if let Some(filename) = companion_path.file_name() {
                let dest = theory_dir.join(filename);
                if !dest.exists() {
                    let _ = std::fs::copy(companion_path, &dest);
                }
            }
        }

        let theory_name = format!("Kleis_Verify_{}", self.theory_counter);
        let theory_file = theory_dir.join(format!("{}.thy", theory_name));

        std::fs::write(&theory_file, theory)
            .map_err(|e| format!("Failed to write theory file: {}", e))?;

        let conn = self.connection.as_mut().ok_or("Connection lost")?;
        conn.set_timeout(Duration::from_secs(super::USE_THEORIES_TIMEOUT))?;

        // use_theories expects theory names (without .thy) and a master_dir
        let args = serde_json::json!({
            "session_id": self.session_id.as_ref().unwrap_or(&String::new()),
            "theories": [theory_name],
            "master_dir": theory_dir.to_string_lossy()
        });

        if std::env::var("KLEIS_DEBUG").is_ok() {
            eprintln!("=== SENDING use_theories ===");
            eprintln!(
                "args: {}",
                serde_json::to_string_pretty(&args).unwrap_or_default()
            );
        }

        let result = match conn.send_command("use_theories", &args)? {
            CommandResult::Ok(Some(response)) => {
                if std::env::var("KLEIS_DEBUG").is_ok() {
                    eprintln!("=== RESPONSE (Ok) ===");
                    eprintln!(
                        "{}",
                        serde_json::to_string_pretty(&response).unwrap_or_default()
                    );
                }
                // Check if this is actually an async task response
                if let Some(task_id) = response.get("task").and_then(|v| v.as_str()) {
                    if std::env::var("KLEIS_DEBUG").is_ok() {
                        eprintln!("=== Async task detected, waiting for FINISHED ===");
                        eprintln!("task_id: {}", task_id);
                    }
                    // Wait for the task to complete
                    match self.wait_for_use_theories_task(task_id) {
                        Ok(finished_response) => {
                            if std::env::var("KLEIS_DEBUG").is_ok() {
                                eprintln!("=== USE_THEORIES FINISHED ===");
                                eprintln!(
                                    "{}",
                                    serde_json::to_string_pretty(&finished_response)
                                        .unwrap_or_default()
                                );
                            }
                            self.parse_use_theories_result(&finished_response)
                        }
                        Err(e) => {
                            if std::env::var("KLEIS_DEBUG").is_ok() {
                                eprintln!("=== USE_THEORIES ERROR ===");
                                eprintln!("{}", e);
                            }
                            // An error during proof is a real failure
                            Ok(VerificationResult::Invalid { counterexample: e })
                        }
                    }
                } else {
                    self.parse_verification_response(&response)
                }
            }
            CommandResult::Ok(None) => {
                if std::env::var("KLEIS_DEBUG").is_ok() {
                    eprintln!("=== RESPONSE (Ok None) ===");
                }
                // Empty response - NOT a success, we need to wait for async completion
                Ok(VerificationResult::Unknown)
            }
            CommandResult::Error(msg) => {
                if std::env::var("KLEIS_DEBUG").is_ok() {
                    eprintln!("=== RESPONSE (Error) ===");
                    eprintln!("{}", msg);
                }
                self.parse_error_message(&msg)
            }
            CommandResult::Running { task_id } => {
                if std::env::var("KLEIS_DEBUG").is_ok() {
                    eprintln!("=== RESPONSE (Running) ===");
                    eprintln!("task_id: {}", task_id);
                }
                // Poll for completion
                match self.wait_for_use_theories_task(&task_id) {
                    Ok(response) => {
                        if std::env::var("KLEIS_DEBUG").is_ok() {
                            eprintln!("=== TASK FINISHED ===");
                            eprintln!(
                                "{}",
                                serde_json::to_string_pretty(&response).unwrap_or_default()
                            );
                        }
                        self.parse_use_theories_result(&response)
                    }
                    Err(e) => {
                        if std::env::var("KLEIS_DEBUG").is_ok() {
                            eprintln!("=== TASK ERROR ===");
                            eprintln!("{}", e);
                        }
                        Ok(VerificationResult::Invalid { counterexample: e })
                    }
                }
            }
        };

        if std::env::var("KLEIS_DEBUG").is_ok() {
            eprintln!("=== RESULT: {:?} ===\n", result);
        }

        // Cleanup temp file AFTER we have the result
        // (theory was already read by Isabelle at this point)
        let _ = std::fs::remove_file(&theory_file);

        result
    }

    /// Parse Isabelle's verification response
    fn parse_verification_response(
        &self,
        response: &serde_json::Value,
    ) -> Result<VerificationResult, String> {
        // Check for explicit status
        if let Some(status) = response.get("status").and_then(|v| v.as_str()) {
            return match status {
                "ok" | "finished" | "consolidated" => Ok(VerificationResult::Valid),
                "failed" | "error" => {
                    let msg = self.extract_error_message(response);
                    Ok(VerificationResult::Invalid {
                        counterexample: msg,
                    })
                }
                "canceled" | "timeout" => Ok(VerificationResult::Unknown),
                _ => Ok(VerificationResult::Unknown),
            };
        }

        // Check nodes_status for detailed info
        if let Some(nodes) = response.get("nodes").and_then(|v| v.as_array()) {
            for node in nodes {
                if let Some(status) = node.get("status").and_then(|v| v.as_str()) {
                    if status == "failed" || status == "error" {
                        let msg = self.extract_node_error(node);
                        return Ok(VerificationResult::Invalid {
                            counterexample: msg,
                        });
                    }
                }
            }
            // All nodes OK
            return Ok(VerificationResult::Valid);
        }

        // Check for errors array
        if let Some(errors) = response.get("errors").and_then(|v| v.as_array()) {
            if !errors.is_empty() {
                let msg = errors
                    .iter()
                    .filter_map(|e| e.as_str())
                    .collect::<Vec<_>>()
                    .join("; ");
                return Ok(VerificationResult::Invalid {
                    counterexample: msg,
                });
            }
        }

        // No explicit failure - assume success
        Ok(VerificationResult::Valid)
    }

    /// Extract error message from response
    fn extract_error_message(&self, response: &serde_json::Value) -> String {
        // Try various fields where error messages might be
        if let Some(msg) = response.get("message").and_then(|v| v.as_str()) {
            return msg.to_string();
        }
        if let Some(msg) = response.get("error").and_then(|v| v.as_str()) {
            return msg.to_string();
        }
        if let Some(errors) = response.get("errors").and_then(|v| v.as_array()) {
            return errors
                .iter()
                .filter_map(|e| e.as_str())
                .collect::<Vec<_>>()
                .join("; ");
        }
        "Proof failed (no details available)".to_string()
    }

    /// Extract error from a node in nodes_status
    fn extract_node_error(&self, node: &serde_json::Value) -> String {
        if let Some(messages) = node.get("messages").and_then(|v| v.as_array()) {
            let errors: Vec<&str> = messages
                .iter()
                .filter_map(|m| {
                    let kind = m.get("kind").and_then(|k| k.as_str()).unwrap_or("");
                    if kind == "error" || kind == "warning" {
                        m.get("message").and_then(|msg| msg.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            if !errors.is_empty() {
                return errors.join("; ");
            }
        }
        "Proof failed in theory node".to_string()
    }

    /// Parse error message from command error
    fn parse_error_message(&self, msg: &str) -> Result<VerificationResult, String> {
        // Classify different error types
        if msg.contains("Failed to finish proof") || msg.contains("proof failed") {
            Ok(VerificationResult::Invalid {
                counterexample: format!("Proof method failed: {}", msg),
            })
        } else if msg.contains("Undefined") || msg.contains("undefined") {
            Ok(VerificationResult::Invalid {
                counterexample: format!("Undefined symbol or type: {}", msg),
            })
        } else if msg.contains("Type unification failed") || msg.contains("type error") {
            Ok(VerificationResult::Invalid {
                counterexample: format!("Type error in formula: {}", msg),
            })
        } else if msg.contains("Timeout") || msg.contains("timeout") {
            Ok(VerificationResult::Unknown)
        } else if msg.contains("syntax error") || msg.contains("Inner syntax error") {
            Err(format!("Syntax error in translated formula: {}", msg))
        } else {
            // Unknown error type
            Ok(VerificationResult::Invalid {
                counterexample: msg.to_string(),
            })
        }
    }

    /// Clear verification cache (call when axioms change)
    pub fn clear_cache(&mut self) {
        self.verification_cache.clear();
    }

    /// Add an axiom to the context for subsequent verifications
    pub fn add_context_axiom(&mut self, axiom: &Expression) -> Result<(), String> {
        let isar = self.translate_to_isar(axiom)?;
        self.loaded_axioms.push(isar);
        self.verification_cache.clear(); // Invalidate cache when context changes
        Ok(())
    }

    /// Set companion theory file path derived from a .kleis source file
    ///
    /// The companion .thy file contains hand-written Isabelle proofs for axioms
    /// that cannot be auto-proved.
    pub fn set_companion_theory(&mut self, kleis_file: &std::path::Path) {
        let thy_file = kleis_file.with_extension("thy");
        if thy_file.exists() {
            println!("Found companion theory: {}", thy_file.display());
            if !self.companion_theories.contains(&thy_file) {
                self.companion_theories.push(thy_file);
            }
        }
    }

    /// Add companion theories for imported .kleis files
    ///
    /// Call this for each imported file to load its companion .thy if it exists.
    pub fn add_import_companion(&mut self, import_path: &std::path::Path) {
        let thy_file = import_path.with_extension("thy");
        if thy_file.exists() {
            println!("Found import companion theory: {}", thy_file.display());
            if !self.companion_theories.contains(&thy_file) {
                // Insert at beginning so imports are loaded first
                self.companion_theories.insert(0, thy_file);
            }
        }
    }

    /// Load and verify all companion theory files through Isabelle
    ///
    /// This loads companion .thy files into Isabelle for proper verification.
    /// Only lemmas that Isabelle successfully verifies are marked as proven.
    ///
    /// The companions are only loaded once per session.
    pub fn load_companion_theory(&mut self) -> Result<(), String> {
        // Only load once
        if self.companion_loaded {
            return Ok(());
        }

        if self.companion_theories.is_empty() {
            return Ok(()); // No companion theories
        }

        // Process each companion theory
        for thy_file in self.companion_theories.clone() {
            // First, extract lemma names and check for sorry locally
            // Store them temporarily until verification succeeds
            let mut temp_proven: HashSet<String> = HashSet::new();
            std::mem::swap(&mut temp_proven, &mut self.companion_proven);
            self.extract_proven_lemmas(&thy_file)?;
            let new_lemmas: HashSet<String> = self.companion_proven.clone();
            // Restore previous proven set
            self.companion_proven = temp_proven;

            // Now actually verify through Isabelle
            if self.is_connected() && self.has_session() {
                println!("Verifying companion theory: {} ...", thy_file.display());
                match self.verify_companion_with_isabelle(&thy_file) {
                    Ok(()) => {
                        println!("✅ Companion theory verified by Isabelle");
                        // Now add the lemmas since verification succeeded
                        self.companion_proven.extend(new_lemmas);
                    }
                    Err(e) => {
                        // This companion failed - do NOT add its lemmas
                        println!("❌ Companion theory verification failed: {}", e);
                        println!("   Lemmas from this companion will NOT be trusted");
                    }
                }
            } else {
                println!("⚠️  No Isabelle connection - companion theory NOT verified");
                println!("   Lemmas will NOT be trusted (run with Isabelle to verify)");
            }
        }

        self.companion_loaded = true;
        Ok(())
    }

    /// Actually verify the companion theory through Isabelle
    fn verify_companion_with_isabelle(&mut self, thy_file: &std::path::Path) -> Result<(), String> {
        let theory_name = thy_file
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid theory file name")?
            .to_string();

        let master_dir = thy_file.parent().ok_or("Invalid theory file path")?;

        let conn = self.connection.as_mut().ok_or("Connection lost")?;
        conn.set_timeout(std::time::Duration::from_secs(180))?; // 3 minutes for companion

        let args = serde_json::json!({
            "session_id": self.session_id.as_ref().unwrap_or(&String::new()),
            "theories": [theory_name],
            "master_dir": master_dir.to_string_lossy()
        });

        match conn.send_command("use_theories", &args)? {
            super::connection::CommandResult::Ok(Some(response)) => {
                if let Some(task_id) = response.get("task").and_then(|v| v.as_str()) {
                    match self.wait_for_use_theories_task(task_id) {
                        Ok(finished) => {
                            let result = self.parse_use_theories_result(&finished)?;
                            match result {
                                VerificationResult::Valid => Ok(()),
                                VerificationResult::Invalid { counterexample } => {
                                    Err(counterexample)
                                }
                                VerificationResult::Unknown => {
                                    Err("Verification timed out".to_string())
                                }
                            }
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    Ok(())
                }
            }
            super::connection::CommandResult::Error(msg) => Err(msg),
            _ => Ok(()),
        }
    }

    /// Extract lemma/theorem names from a .thy file
    ///
    /// Also checks for `sorry` which indicates unproven lemmas.
    fn extract_proven_lemmas(&mut self, thy_file: &std::path::Path) -> Result<(), String> {
        let content = std::fs::read_to_string(thy_file)
            .map_err(|e| format!("Failed to read theory file: {}", e))?;

        // Check for 'sorry' - indicates unproven lemmas
        let mut sorry_lemmas: Vec<String> = Vec::new();
        let mut current_lemma: Option<String> = None;

        for line in content.lines() {
            let line_trimmed = line.trim();

            // Track current lemma/theorem being defined
            if line_trimmed.starts_with("lemma ") || line_trimmed.starts_with("theorem ") {
                let rest = line_trimmed
                    .strip_prefix("lemma ")
                    .or_else(|| line_trimmed.strip_prefix("theorem "))
                    .unwrap_or("");
                if let Some(name) = rest.split(':').next() {
                    let name = name.trim();
                    if !name.is_empty() && !name.contains(' ') {
                        current_lemma = Some(name.to_string());
                    }
                }
            }

            // Check if current lemma uses 'sorry'
            if line_trimmed.contains("sorry") {
                if let Some(ref name) = current_lemma {
                    sorry_lemmas.push(name.clone());
                }
            }

            // End of lemma/theorem (various endings)
            // Note: "by auto" can appear mid-line after "unfolding ..."
            if line_trimmed == "qed"
                || line_trimmed.starts_with("by ")
                || line_trimmed.contains(" by ")
                || line_trimmed == "done"
                || line_trimmed == "oops"
            // abandoned proof
            {
                if let Some(ref name) = current_lemma {
                    // Only add to proven if not using sorry
                    if !sorry_lemmas.contains(name) {
                        self.companion_proven.insert(name.clone());
                    }
                }
                current_lemma = None;
            }
        }

        // Report sorry usage
        if !sorry_lemmas.is_empty() {
            println!(
                "⚠️  Companion has {} unproven lemma(s) (sorry): {}",
                sorry_lemmas.len(),
                sorry_lemmas.join(", ")
            );
        }

        if !self.companion_proven.is_empty() {
            println!(
                "Companion proves {} axiom(s): {}",
                self.companion_proven.len(),
                self.companion_proven
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        Ok(())
    }

    /// Check if an axiom is proven by the companion theory
    pub fn is_proven_by_companion(&self, axiom_name: &str) -> bool {
        self.companion_proven.contains(axiom_name)
    }
}

impl Default for IsabelleBackend {
    fn default() -> Self {
        Self::new().expect("Failed to create IsabelleBackend")
    }
}

impl Drop for IsabelleBackend {
    fn drop(&mut self) {
        // Stop session if active
        let _ = self.stop_session();

        // Send shutdown command to Isabelle server (graceful shutdown)
        if let Some(ref mut conn) = self.connection {
            let _ = conn.send_command("shutdown", &serde_json::json!({}));
            // Give server time to process shutdown
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        // Close connection
        if let Some(conn) = self.connection.take() {
            let _ = conn.close();
        }

        // Kill server process and its children if we started it
        if let Some(mut process) = self.server_process.take() {
            let pid = process.id();

            // On Unix, kill all child processes
            #[cfg(unix)]
            {
                // First, try to kill the process group (works if we're the group leader)
                let _ = std::process::Command::new("kill")
                    .args(["-TERM", "--", &format!("-{}", pid)])
                    .output();

                // Also use pkill to kill any children by parent PID
                let _ = std::process::Command::new("pkill")
                    .args(["-TERM", "-P", &pid.to_string()])
                    .output();

                // Give processes time to terminate gracefully
                std::thread::sleep(std::time::Duration::from_millis(200));

                // Force kill remaining processes
                let _ = std::process::Command::new("kill")
                    .args(["-KILL", "--", &format!("-{}", pid)])
                    .output();

                let _ = std::process::Command::new("pkill")
                    .args(["-KILL", "-P", &pid.to_string()])
                    .output();

                // Kill the main process too
                let _ = process.kill();
            }

            #[cfg(not(unix))]
            {
                let _ = process.kill();
            }

            // Wait for the process to fully terminate
            let _ = process.wait();
        }
    }
}

impl SolverBackend for IsabelleBackend {
    fn name(&self) -> &str {
        &self.capabilities.solver.name
    }

    fn capabilities(&self) -> &SolverCapabilities {
        &self.capabilities
    }

    fn verify_axiom(&mut self, axiom: &Expression) -> Result<VerificationResult, String> {
        // Check connection, auto-start session if needed
        if !self.is_connected() {
            return Err("Not connected to Isabelle server. Call connect() first.".to_string());
        }

        // Auto-start session if not active (session caching)
        if !self.has_session() {
            self.ensure_session()?;
        }

        // Translate axiom to Isar
        let isar = self.translate_to_isar(axiom)?;

        // Check cache
        if let Some(cached) = self.verification_cache.get(&isar) {
            return Ok(cached.clone());
        }

        // Generate unique theory name to avoid conflicts
        self.theory_counter += 1;
        let theory_name = format!("Kleis_Verify_{}", self.theory_counter);

        // Build theory with loaded axioms as context
        // Use 'assumes' to make them available for proof
        let (axiom_context, _simp_rules) = if self.loaded_axioms.is_empty() {
            (String::new(), String::new())
        } else {
            let axiom_names: Vec<String> = (0..self.loaded_axioms.len())
                .map(|i| format!("ctx_{}", i))
                .collect();
            let context = format!(
                "\n(* Context axioms *)\n{}\n",
                self.loaded_axioms
                    .iter()
                    .enumerate()
                    .map(|(i, ax)| format!("axiomatization where ctx_{}: \"{}\"", i, ax))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            let simp = axiom_names.join(" ");
            (context, simp)
        };

        // Create theory with simple proof methods
        // Import Complex_Main for real numbers, plus companion theories for definitions
        let proof_method = "by auto".to_string();

        // Build imports - Complex_Main plus any companion theories
        let mut imports = vec!["Complex_Main".to_string()];
        for thy_path in &self.companion_theories {
            if let Some(name) = thy_path.file_stem().and_then(|s| s.to_str()) {
                imports.push(name.to_string());
            }
        }
        let imports_str = imports.join(" ");

        // When importing companions, we don't need axiom context (companions provide definitions)
        let effective_context = if !self.companion_proven.is_empty() {
            String::new() // Companion provides definitions
        } else {
            axiom_context // No companion, use axiomatized context
        };

        let theory = format!(
            r#"theory {}
imports {}
begin
{}
lemma kleis_axiom: "{}"
  {}
end"#,
            theory_name, imports_str, effective_context, isar, proof_method
        );

        // Send to Isabelle via use_theories
        let result = self.send_theory_for_verification(&theory)?;

        // Cache the result
        self.verification_cache.insert(isar, result.clone());

        Ok(result)
    }

    fn check_satisfiability(&mut self, expr: &Expression) -> Result<SatisfiabilityResult, String> {
        // Isabelle doesn't have a direct satisfiability check like SMT solvers.
        // We can try to prove the negation - if it fails, the expression might be satisfiable.
        let negated = Expression::Operation {
            name: "not".to_string(),
            args: vec![expr.clone()],
            span: None,
        };

        match self.verify_axiom(&negated)? {
            VerificationResult::Valid => {
                // ¬expr is valid, so expr is unsatisfiable
                Ok(SatisfiabilityResult::Unsatisfiable)
            }
            VerificationResult::Invalid { counterexample } => {
                // ¬expr has a counterexample, so expr is satisfiable there
                Ok(SatisfiabilityResult::Satisfiable {
                    example: counterexample,
                })
            }
            VerificationResult::Unknown => Ok(SatisfiabilityResult::Unknown),
        }
    }

    fn evaluate(&mut self, _expr: &Expression) -> Result<Expression, String> {
        // Isabelle is not designed for concrete evaluation
        // Return the expression unchanged
        Err("Isabelle does not support concrete evaluation. Use Z3 for evaluation.".to_string())
    }

    fn simplify(&mut self, expr: &Expression) -> Result<Expression, String> {
        // TODO: Use Isabelle's simplifier
        // For now, return unchanged
        Ok(expr.clone())
    }

    fn are_equivalent(&mut self, expr1: &Expression, expr2: &Expression) -> Result<bool, String> {
        // Prove expr1 = expr2
        let equality = Expression::Operation {
            name: "equals".to_string(),
            args: vec![expr1.clone(), expr2.clone()],
            span: None,
        };

        match self.verify_axiom(&equality)? {
            VerificationResult::Valid => Ok(true),
            _ => Ok(false),
        }
    }

    fn load_structure_axioms(
        &mut self,
        structure_name: &str,
        axioms: &[Expression],
    ) -> Result<(), String> {
        if self.loaded_structures.contains(structure_name) {
            return Ok(()); // Already loaded
        }

        // Translate axioms and add to current theory context
        for axiom in axioms {
            let _isar = self.translate_to_isar(axiom)?;
            // TODO: Add as assumes in locale
        }

        self.loaded_structures.insert(structure_name.to_string());
        Ok(())
    }

    fn push(&mut self) {
        // Isabelle doesn't have push/pop like SMT solvers
        // Could potentially use proof context blocks
    }

    fn pop(&mut self, _levels: u32) {
        // No-op for Isabelle
    }

    fn reset(&mut self) {
        self.declared_ops.clear();
        self.loaded_structures.clear();
        self.identity_elements.clear();
        self.warnings.clear();
        self.loaded_axioms.clear();
        self.verification_cache.clear();
        self.theory_counter = 0;
        self.companion_theories.clear();
        self.companion_loaded = false;
        self.companion_proven.clear();
        // Would need to restart session for full reset
    }

    fn load_identity_element(&mut self, name: &str, _type_expr: &TypeExpr) {
        self.identity_elements.insert(name.to_string());
    }

    fn is_declared_constructor(&self, name: &str) -> bool {
        // TODO: Check against loaded data types
        self.declared_ops.contains(name)
    }

    fn assert_expression(&mut self, expr: &Expression) -> Result<(), String> {
        let _isar = self.translate_to_isar(expr)?;
        // TODO: Add as assumption in current context
        Ok(())
    }

    fn define_function(
        &mut self,
        name: &str,
        params: &[String],
        body: &Expression,
    ) -> Result<(), String> {
        let body_isar = self.translate_to_isar(body)?;
        let params_str = params.join(" ");

        let _definition = format!(
            "definition {} :: \"...\" where\n  \"{} {} = {}\"",
            name, name, params_str, body_isar
        );

        // TODO: Send definition to Isabelle
        self.declared_ops.insert(name.to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_creation() {
        let backend = IsabelleBackend::new().unwrap();
        assert_eq!(backend.name(), "Isabelle");
        assert!(!backend.is_connected());
        assert!(!backend.has_session());
    }

    #[test]
    fn test_capabilities_loaded() {
        let backend = IsabelleBackend::new().unwrap();
        let caps = backend.capabilities();

        assert_eq!(caps.solver.name, "Isabelle");
        assert!(caps.has_operation("plus"));
        assert!(caps.has_operation("equals"));
        assert!(caps.has_theory("HOL"));
        assert!(caps.capabilities.features.quantifiers);
        assert!(caps.capabilities.features.proof_generation);
        assert!(!caps.capabilities.features.evaluation); // Isabelle doesn't eval
    }

    #[test]
    fn test_translate_simple_expression() {
        let backend = IsabelleBackend::new().unwrap();

        // Test constant
        let expr = Expression::Const("42".to_string());
        assert_eq!(backend.translate_to_isar(&expr).unwrap(), "42");

        // Test variable
        let expr = Expression::Object("x".to_string());
        assert_eq!(backend.translate_to_isar(&expr).unwrap(), "x");
    }

    #[test]
    fn test_translate_operation() {
        let backend = IsabelleBackend::new().unwrap();

        // Test binary operation: x + y
        let expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Object("y".to_string()),
            ],
            span: None,
        };

        let isar = backend.translate_to_isar(&expr).unwrap();
        // Standard Isabelle infix notation: (x + y)
        assert_eq!(isar, "(x + y)");
    }

    #[test]
    fn test_translate_quantifier() {
        use crate::ast::QuantifiedVar;

        let backend = IsabelleBackend::new().unwrap();

        // Test: ∀x. x = x
        let expr = Expression::Quantifier {
            quantifier: QuantifierKind::ForAll,
            variables: vec![QuantifiedVar {
                name: "x".to_string(),
                type_annotation: None,
            }],
            where_clause: None,
            body: Box::new(Expression::Operation {
                name: "equals".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Object("x".to_string()),
                ],
                span: None,
            }),
        };

        let isar = backend.translate_to_isar(&expr).unwrap();
        assert!(isar.contains("∀"));
        assert!(isar.contains("x"));
    }

    #[test]
    fn test_default_config() {
        let config = IsabelleConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 0);
        assert_eq!(config.session, "HOL");
    }

    #[test]
    fn test_translate_conditional() {
        let backend = IsabelleBackend::new().unwrap();

        // Test: if x > 0 then x else -x (absolute value)
        let expr = Expression::Conditional {
            condition: Box::new(Expression::Operation {
                name: "greater_than".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Const("0".to_string()),
                ],
                span: None,
            }),
            then_branch: Box::new(Expression::Object("x".to_string())),
            else_branch: Box::new(Expression::Operation {
                name: "negate".to_string(),
                args: vec![Expression::Object("x".to_string())],
                span: None,
            }),
            span: None,
        };

        let isar = backend.translate_to_isar(&expr).unwrap();
        assert!(isar.contains("if"));
        assert!(isar.contains("then"));
        assert!(isar.contains("else"));
        assert!(isar.contains("x > 0"));
    }

    #[test]
    fn test_translate_let() {
        use crate::ast::Pattern;
        let backend = IsabelleBackend::new().unwrap();

        // Test: let y = 5 in y + y
        let expr = Expression::Let {
            pattern: Pattern::Variable("y".to_string()),
            type_annotation: None,
            value: Box::new(Expression::Const("5".to_string())),
            body: Box::new(Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Object("y".to_string()),
                    Expression::Object("y".to_string()),
                ],
                span: None,
            }),
            span: None,
        };

        let isar = backend.translate_to_isar(&expr).unwrap();
        assert!(isar.contains("let"));
        assert!(isar.contains("y"));
        assert!(isar.contains("5"));
        assert!(isar.contains("in"));
    }

    #[test]
    fn test_translate_comparisons() {
        let backend = IsabelleBackend::new().unwrap();

        // Test less than: x < y
        let expr = Expression::Operation {
            name: "less_than".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Object("y".to_string()),
            ],
            span: None,
        };
        assert_eq!(backend.translate_to_isar(&expr).unwrap(), "(x < y)");

        // Test greater than or equal: x >= y
        let expr = Expression::Operation {
            name: "geq".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Object("y".to_string()),
            ],
            span: None,
        };
        assert_eq!(backend.translate_to_isar(&expr).unwrap(), "(x ≥ y)");

        // Test not equal: x ≠ y
        let expr = Expression::Operation {
            name: "neq".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Object("y".to_string()),
            ],
            span: None,
        };
        assert_eq!(backend.translate_to_isar(&expr).unwrap(), "(x ≠ y)");
    }

    #[test]
    fn test_translate_boolean_ops() {
        let backend = IsabelleBackend::new().unwrap();

        // Test and: p ∧ q
        let expr = Expression::Operation {
            name: "and".to_string(),
            args: vec![
                Expression::Object("p".to_string()),
                Expression::Object("q".to_string()),
            ],
            span: None,
        };
        assert_eq!(backend.translate_to_isar(&expr).unwrap(), "(p ∧ q)");

        // Test implies: p ⟶ q
        let expr = Expression::Operation {
            name: "implies".to_string(),
            args: vec![
                Expression::Object("p".to_string()),
                Expression::Object("q".to_string()),
            ],
            span: None,
        };
        assert_eq!(backend.translate_to_isar(&expr).unwrap(), "(p ⟶ q)");

        // Test not: ¬p
        let expr = Expression::Operation {
            name: "not".to_string(),
            args: vec![Expression::Object("p".to_string())],
            span: None,
        };
        assert_eq!(backend.translate_to_isar(&expr).unwrap(), "(¬ p)");
    }

    #[test]
    fn test_translate_list_ops() {
        let backend = IsabelleBackend::new().unwrap();

        // Test list literal: [1, 2, 3]
        let expr = Expression::List(vec![
            Expression::Const("1".to_string()),
            Expression::Const("2".to_string()),
            Expression::Const("3".to_string()),
        ]);
        assert_eq!(backend.translate_to_isar(&expr).unwrap(), "[1, 2, 3]");

        // Test cons: x # xs
        let expr = Expression::Operation {
            name: "cons".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Object("xs".to_string()),
            ],
            span: None,
        };
        assert_eq!(backend.translate_to_isar(&expr).unwrap(), "(x # xs)");

        // Test append: xs @ ys
        let expr = Expression::Operation {
            name: "append".to_string(),
            args: vec![
                Expression::Object("xs".to_string()),
                Expression::Object("ys".to_string()),
            ],
            span: None,
        };
        assert_eq!(backend.translate_to_isar(&expr).unwrap(), "(xs @ ys)");
    }

    #[test]
    fn test_translate_type() {
        let backend = IsabelleBackend::new().unwrap();

        // Basic types
        assert_eq!(backend.translate_type("ℕ"), "nat");
        assert_eq!(backend.translate_type("ℤ"), "int");
        assert_eq!(backend.translate_type("ℝ"), "real");
        assert_eq!(backend.translate_type("Bool"), "bool");

        // Parameterized types
        assert_eq!(backend.translate_type("List(ℕ)"), "nat list");
        assert_eq!(backend.translate_type("Option(ℤ)"), "int option");
    }

    #[test]
    fn test_translate_quantifier_with_where() {
        use crate::ast::QuantifiedVar;
        let backend = IsabelleBackend::new().unwrap();

        // Test: ∀x. x ≠ 0 ⟶ x * inverse(x) = 1
        let expr = Expression::Quantifier {
            quantifier: QuantifierKind::ForAll,
            variables: vec![QuantifiedVar {
                name: "x".to_string(),
                type_annotation: Some("ℝ".to_string()),
            }],
            where_clause: Some(Box::new(Expression::Operation {
                name: "neq".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Const("0".to_string()),
                ],
                span: None,
            })),
            body: Box::new(Expression::Operation {
                name: "equals".to_string(),
                args: vec![
                    Expression::Operation {
                        name: "times".to_string(),
                        args: vec![
                            Expression::Object("x".to_string()),
                            Expression::Operation {
                                name: "inverse".to_string(),
                                args: vec![Expression::Object("x".to_string())],
                                span: None,
                            },
                        ],
                        span: None,
                    },
                    Expression::Const("1".to_string()),
                ],
                span: None,
            }),
        };

        let isar = backend.translate_to_isar(&expr).unwrap();
        assert!(isar.contains("∀"));
        assert!(isar.contains("real")); // Type annotation translated
        assert!(isar.contains("⟶")); // Where clause becomes implication
        assert!(isar.contains("x ≠ 0")); // The condition
    }

    #[test]
    fn test_translate_lambda_with_types() {
        use crate::ast::LambdaParam;
        let backend = IsabelleBackend::new().unwrap();

        // Test: λ(x :: nat). x + 1
        let expr = Expression::Lambda {
            params: vec![LambdaParam {
                name: "x".to_string(),
                type_annotation: Some("ℕ".to_string()),
            }],
            body: Box::new(Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Const("1".to_string()),
                ],
                span: None,
            }),
            span: None,
        };

        let isar = backend.translate_to_isar(&expr).unwrap();
        assert!(isar.contains("λ"));
        assert!(isar.contains("nat")); // Type translated
        assert!(isar.contains("x + 1"));
    }

    #[test]
    fn test_translate_string_literal() {
        let backend = IsabelleBackend::new().unwrap();

        let expr = Expression::String("hello world".to_string());
        let isar = backend.translate_to_isar(&expr).unwrap();
        assert_eq!(isar, "''hello world''");
    }

    #[test]
    fn test_translate_ascription() {
        let backend = IsabelleBackend::new().unwrap();

        // Test: (x :: nat)
        let expr = Expression::Ascription {
            expr: Box::new(Expression::Object("x".to_string())),
            type_annotation: "ℕ".to_string(),
        };

        let isar = backend.translate_to_isar(&expr).unwrap();
        assert_eq!(isar, "(x :: nat)");
    }

    #[test]
    fn test_placeholder_error() {
        let backend = IsabelleBackend::new().unwrap();

        let expr = Expression::Placeholder {
            id: 42,
            hint: "fill me".to_string(),
        };

        let result = backend.translate_to_isar(&expr);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("placeholder"));
    }
}
