//! Kleis Debug Adapter Protocol (DAP) Server
//!
//! This provides step-through debugging for Kleis via the Debug Adapter Protocol:
//! - Breakpoints on functions and lines
//! - Step in/out/over
//! - Variable inspection
//! - Call stack viewing
//!
//! ## Usage
//!
//! ```bash
//! cargo build --release --bin kleis-debug
//! ```
//!
//! Then configure your editor to use `target/release/kleis-debug` as the
//! debug adapter for `.kleis` files.

use kleis::debug::{
    Breakpoint as DebugBreakpoint, DebugAction, DebugHook, DebugState as HookDebugState,
    SourceLocation, StackFrame as DebugStackFrame,
};
use kleis::evaluator::Evaluator;
use kleis::kleis_parser::parse_kleis_program;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex};

/// DAP Message types
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
enum DapMessage {
    #[serde(rename = "request")]
    Request(DapRequest),
    #[serde(rename = "response")]
    Response(DapResponse),
    #[serde(rename = "event")]
    Event(DapEvent),
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DapRequest {
    seq: i64,
    command: String,
    #[serde(default)]
    arguments: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DapResponse {
    seq: i64,
    request_seq: i64,
    success: bool,
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
struct DapEvent {
    seq: i64,
    event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<Value>,
}

/// Shared state for communication between DAP and debug hook
struct SharedDebugState {
    /// The action to take (set by DAP, read by hook)
    pending_action: Mutex<Option<DebugAction>>,
    /// Condition variable for waiting
    condvar: Condvar,
    /// Whether we're paused
    paused: Mutex<bool>,
}

impl SharedDebugState {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            pending_action: Mutex::new(None),
            condvar: Condvar::new(),
            paused: Mutex::new(false),
        })
    }

    /// Signal that we should continue with the given action
    fn signal_continue(&self, action: DebugAction) {
        let mut pending = self.pending_action.lock().unwrap();
        *pending = Some(action);
        *self.paused.lock().unwrap() = false;
        self.condvar.notify_all();
    }

    /// Wait for a continue signal (called from the hook)
    fn wait_for_continue(&self) -> DebugAction {
        *self.paused.lock().unwrap() = true;
        let mut pending = self.pending_action.lock().unwrap();
        while pending.is_none() {
            pending = self.condvar.wait(pending).unwrap();
        }
        pending.take().unwrap_or(DebugAction::Continue)
    }
}

/// Debug adapter state
struct DebugAdapter {
    /// Sequence number for outgoing messages
    seq: i64,
    /// Breakpoints by file path
    breakpoints: HashMap<String, Vec<DapBreakpoint>>,
    /// Whether we're currently debugging
    running: bool,
    /// Current evaluation state
    state: DapDebugState,
    /// The Kleis evaluator
    evaluator: Evaluator,
    /// The program being debugged
    program_path: Option<PathBuf>,
    /// Shared state for hook communication
    shared_state: Arc<SharedDebugState>,
    /// Debug hook (stored separately for access to stack)
    hook_stack: Arc<Mutex<Vec<DebugStackFrame>>>,
    /// Current bindings from the hook
    hook_bindings: Arc<Mutex<HashMap<String, String>>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct DapBreakpoint {
    line: i64,
    verified: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum DapDebugState {
    NotStarted,
    Running,
    Paused { reason: String },
    Terminated,
}

/// A debug hook that bridges between the evaluator and the DAP server
struct DapDebugHook {
    /// Shared state for synchronization
    shared_state: Arc<SharedDebugState>,
    /// Call stack (shared with DAP server)
    stack: Arc<Mutex<Vec<DebugStackFrame>>>,
    /// Variable bindings (shared with DAP server)
    bindings: Arc<Mutex<HashMap<String, String>>>,
    /// Breakpoints
    breakpoints: Vec<DebugBreakpoint>,
    /// Current hook state
    state: HookDebugState,
    /// Current depth
    current_depth: usize,
}

impl DapDebugHook {
    fn new(
        shared_state: Arc<SharedDebugState>,
        stack: Arc<Mutex<Vec<DebugStackFrame>>>,
        bindings: Arc<Mutex<HashMap<String, String>>>,
    ) -> Self {
        Self {
            shared_state,
            stack,
            bindings,
            breakpoints: Vec::new(),
            state: HookDebugState::Paused, // Start paused
            current_depth: 0,
        }
    }

    fn matches_breakpoint(&self, location: &SourceLocation) -> bool {
        if let Some(ref file) = location.file {
            self.breakpoints
                .iter()
                .any(|bp| bp.enabled && &bp.file == file && bp.line == location.line)
        } else {
            false
        }
    }
}

impl DebugHook for DapDebugHook {
    fn on_eval_start(
        &mut self,
        _expr: &kleis::ast::Expression,
        location: &SourceLocation,
        depth: usize,
    ) -> DebugAction {
        self.current_depth = depth;

        // Check if we should stop
        let should_stop = match &self.state {
            HookDebugState::Paused => true,
            HookDebugState::Running => self.matches_breakpoint(location),
            HookDebugState::Stepping => true,
            HookDebugState::SteppingOver { target_depth } => depth <= *target_depth,
            HookDebugState::SteppingOut { target_depth } => depth <= *target_depth,
        };

        if should_stop {
            self.state = HookDebugState::Paused;
            // Wait for command from DAP server
            let action = self.shared_state.wait_for_continue();
            // Update state based on action
            match action {
                DebugAction::Continue => self.state = HookDebugState::Running,
                DebugAction::StepInto => self.state = HookDebugState::Stepping,
                DebugAction::StepOver => {
                    self.state = HookDebugState::SteppingOver {
                        target_depth: self.current_depth,
                    }
                }
                DebugAction::StepOut => {
                    self.state = HookDebugState::SteppingOut {
                        target_depth: self.current_depth.saturating_sub(1),
                    }
                }
            }
            return action;
        }

        DebugAction::Continue
    }

    fn on_eval_end(
        &mut self,
        _expr: &kleis::ast::Expression,
        _result: &Result<kleis::ast::Expression, String>,
        _depth: usize,
    ) {
    }

    fn on_function_enter(&mut self, name: &str, _args: &[kleis::ast::Expression], location: &SourceLocation, depth: usize) {
        let mut stack = self.stack.lock().unwrap();
        stack.push(DebugStackFrame::new(name, location.clone()));
        self.current_depth = depth;
    }

    fn on_function_exit(
        &mut self,
        _name: &str,
        _result: &Result<kleis::ast::Expression, String>,
        depth: usize,
    ) {
        let mut stack = self.stack.lock().unwrap();
        if stack.len() > 1 {
            stack.pop();
        }
        self.current_depth = depth;
    }

    fn on_bind(&mut self, name: &str, value: &kleis::ast::Expression, _depth: usize) {
        // Update shared bindings
        let mut bindings = self.bindings.lock().unwrap();
        bindings.insert(name.to_string(), format!("{:?}", value));

        // Also update the top stack frame
        let mut stack = self.stack.lock().unwrap();
        if let Some(frame) = stack.last_mut() {
            frame
                .bindings
                .insert(name.to_string(), format!("{:?}", value));
        }
    }

    fn state(&self) -> &HookDebugState {
        &self.state
    }

    fn should_stop(&self, location: &SourceLocation, depth: usize) -> bool {
        match &self.state {
            HookDebugState::Paused => true,
            HookDebugState::Running => self.matches_breakpoint(location),
            HookDebugState::Stepping => true,
            HookDebugState::SteppingOver { target_depth } => depth <= *target_depth,
            HookDebugState::SteppingOut { target_depth } => depth <= *target_depth,
        }
    }

    fn wait_for_command(&mut self) -> DebugAction {
        self.shared_state.wait_for_continue()
    }

    fn get_stack(&self) -> &[DebugStackFrame] {
        // Can't return reference to mutex-guarded data
        // This is a limitation - we'll handle this differently
        &[]
    }

    fn push_frame(&mut self, frame: DebugStackFrame) {
        let mut stack = self.stack.lock().unwrap();
        stack.push(frame);
    }

    fn pop_frame(&mut self) -> Option<DebugStackFrame> {
        let mut stack = self.stack.lock().unwrap();
        if stack.len() > 1 {
            stack.pop()
        } else {
            None
        }
    }
}

impl DebugAdapter {
    fn new() -> Self {
        Self {
            seq: 1,
            breakpoints: HashMap::new(),
            running: false,
            state: DapDebugState::NotStarted,
            evaluator: Evaluator::new(),
            program_path: None,
            shared_state: SharedDebugState::new(),
            hook_stack: Arc::new(Mutex::new(vec![DebugStackFrame::top_level()])),
            hook_bindings: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Send a DAP message to the editor
    fn send(&mut self, msg: &Value) -> io::Result<()> {
        let content = serde_json::to_string(msg)?;
        let header = format!("Content-Length: {}\r\n\r\n", content.len());

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(header.as_bytes())?;
        handle.write_all(content.as_bytes())?;
        handle.flush()?;

        Ok(())
    }

    /// Send a response to a request
    fn respond(
        &mut self,
        request: &DapRequest,
        success: bool,
        body: Option<Value>,
    ) -> io::Result<()> {
        let response = DapResponse {
            seq: self.next_seq(),
            request_seq: request.seq,
            success,
            command: request.command.clone(),
            message: if success {
                None
            } else {
                Some("Error".to_string())
            },
            body,
        };
        self.send(&serde_json::to_value(&response)?)
    }

    /// Send an event to the editor
    fn send_event(&mut self, event: &str, body: Option<Value>) -> io::Result<()> {
        let evt = json!({
            "seq": self.next_seq(),
            "type": "event",
            "event": event,
            "body": body
        });
        self.send(&evt)
    }

    fn next_seq(&mut self) -> i64 {
        let s = self.seq;
        self.seq += 1;
        s
    }

    /// Handle an incoming request
    fn handle_request(&mut self, request: DapRequest) -> io::Result<()> {
        eprintln!(
            "[kleis-debug] Received: {} (seq={})",
            request.command, request.seq
        );

        match request.command.as_str() {
            "initialize" => self.handle_initialize(&request),
            "launch" => self.handle_launch(&request),
            "attach" => self.handle_attach(&request),
            "disconnect" => self.handle_disconnect(&request),
            "terminate" => self.handle_terminate(&request),
            "setBreakpoints" => self.handle_set_breakpoints(&request),
            "setExceptionBreakpoints" => self.handle_set_exception_breakpoints(&request),
            "configurationDone" => self.handle_configuration_done(&request),
            "threads" => self.handle_threads(&request),
            "stackTrace" => self.handle_stack_trace(&request),
            "scopes" => self.handle_scopes(&request),
            "variables" => self.handle_variables(&request),
            "continue" => self.handle_continue(&request),
            "next" => self.handle_next(&request),
            "stepIn" => self.handle_step_in(&request),
            "stepOut" => self.handle_step_out(&request),
            "evaluate" => self.handle_evaluate(&request),
            _ => {
                eprintln!("[kleis-debug] Unknown command: {}", request.command);
                self.respond(&request, false, None)
            }
        }
    }

    // === Request Handlers ===

    fn handle_initialize(&mut self, request: &DapRequest) -> io::Result<()> {
        // Respond with our capabilities
        let capabilities = json!({
            "supportsConfigurationDoneRequest": true,
            "supportsFunctionBreakpoints": true,
            "supportsConditionalBreakpoints": false,
            "supportsEvaluateForHovers": true,
            "supportsStepBack": false,
            "supportsSetVariable": false,
            "supportsRestartFrame": false,
            "supportsStepInTargetsRequest": false,
            "supportsTerminateRequest": true,
            "supportsCompletionsRequest": false,
            "supportsModulesRequest": false,
            "supportsExceptionInfoRequest": false,
            "supportsValueFormattingOptions": false,
            "supportTerminateDebuggee": true,
            "supportsDelayedStackTraceLoading": false,
        });

        self.respond(request, true, Some(capabilities))?;

        // Send initialized event
        self.send_event("initialized", None)
    }

    fn handle_launch(&mut self, request: &DapRequest) -> io::Result<()> {
        self.running = true;
        self.state = DapDebugState::Running;

        // Extract program path from arguments
        let program_path = request
            .arguments
            .as_ref()
            .and_then(|a| a.get("program"))
            .and_then(|p| p.as_str())
            .map(PathBuf::from);

        if let Some(ref path) = program_path {
            eprintln!("[kleis-debug] Launching: {}", path.display());
            self.program_path = Some(path.clone());

            // Load the program
            match fs::read_to_string(path) {
                Ok(source) => match parse_kleis_program(&source) {
                    Ok(program) => {
                        if let Err(e) = self.evaluator.load_program(&program) {
                            eprintln!("[kleis-debug] Error loading program: {}", e);
                        } else {
                            eprintln!(
                                "[kleis-debug] Loaded {} definitions",
                                self.evaluator.list_functions().len()
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("[kleis-debug] Parse error: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("[kleis-debug] Error reading file: {}", e);
                }
            }

            // Set up the debug hook
            self.setup_debug_hook();
        }

        self.respond(request, true, None)?;

        // Stop at entry
        self.state = DapDebugState::Paused {
            reason: "entry".to_string(),
        };
        self.send_event(
            "stopped",
            Some(json!({
                "reason": "entry",
                "threadId": 1,
                "allThreadsStopped": true
            })),
        )
    }

    /// Set up the debug hook on the evaluator
    fn setup_debug_hook(&mut self) {
        let shared_state = self.shared_state.clone();
        let hook_stack = self.hook_stack.clone();
        let hook_bindings = self.hook_bindings.clone();

        // Create a custom hook that communicates with the DAP
        let hook = DapDebugHook::new(shared_state, hook_stack, hook_bindings);
        self.evaluator.set_debug_hook(Box::new(hook));
    }

    fn handle_attach(&mut self, request: &DapRequest) -> io::Result<()> {
        // Attach is similar to launch for us
        self.handle_launch(request)
    }

    fn handle_disconnect(&mut self, request: &DapRequest) -> io::Result<()> {
        self.running = false;
        self.state = DapDebugState::Terminated;
        // Signal hook to continue (so it unblocks)
        self.shared_state.signal_continue(DebugAction::Continue);
        self.respond(request, true, None)
    }

    fn handle_terminate(&mut self, request: &DapRequest) -> io::Result<()> {
        self.running = false;
        self.state = DapDebugState::Terminated;
        // Signal hook to continue (so it unblocks)
        self.shared_state.signal_continue(DebugAction::Continue);
        self.respond(request, true, None)?;
        self.send_event("terminated", None)
    }

    fn handle_set_breakpoints(&mut self, request: &DapRequest) -> io::Result<()> {
        let mut breakpoints = Vec::new();

        if let Some(args) = &request.arguments {
            if let Some(source) = args.get("source") {
                let path = source.get("path").and_then(|p| p.as_str()).unwrap_or("");

                if let Some(bps) = args.get("breakpoints").and_then(|b| b.as_array()) {
                    let mut file_breakpoints = Vec::new();

                    for bp in bps {
                        if let Some(line) = bp.get("line").and_then(|l| l.as_i64()) {
                            file_breakpoints.push(DapBreakpoint {
                                line,
                                verified: true,
                            });
                            breakpoints.push(json!({
                                "verified": true,
                                "line": line
                            }));
                        }
                    }

                    let num_breakpoints = file_breakpoints.len();
                    self.breakpoints.insert(path.to_string(), file_breakpoints);
                    eprintln!(
                        "[kleis-debug] Set {} breakpoints in {}",
                        num_breakpoints, path
                    );
                }
            }
        }

        self.respond(
            request,
            true,
            Some(json!({
                "breakpoints": breakpoints
            })),
        )
    }

    fn handle_set_exception_breakpoints(&mut self, request: &DapRequest) -> io::Result<()> {
        // We don't have exception breakpoints yet
        self.respond(request, true, None)
    }

    fn handle_configuration_done(&mut self, request: &DapRequest) -> io::Result<()> {
        self.respond(request, true, None)
    }

    fn handle_threads(&mut self, request: &DapRequest) -> io::Result<()> {
        // Kleis is single-threaded
        self.respond(
            request,
            true,
            Some(json!({
                "threads": [{
                    "id": 1,
                    "name": "Main Thread"
                }]
            })),
        )
    }

    fn handle_stack_trace(&mut self, request: &DapRequest) -> io::Result<()> {
        // Get the real stack from the hook (collect data before calling respond)
        let source_path = self
            .program_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "<repl>".to_string());
        let source_name = self
            .program_path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "repl".to_string());

        let (frames, total) = {
            let stack = self.hook_stack.lock().unwrap();
            let frames: Vec<Value> = stack
                .iter()
                .rev() // Most recent first
                .enumerate()
                .map(|(i, frame)| {
                    json!({
                        "id": i + 1,
                        "name": frame.name,
                        "source": {
                            "name": &source_name,
                            "path": &source_path
                        },
                        "line": frame.location.line,
                        "column": frame.location.column
                    })
                })
                .collect();
            let total = frames.len();
            (frames, total)
        }; // Lock released here

        self.respond(
            request,
            true,
            Some(json!({
                "stackFrames": frames,
                "totalFrames": total
            })),
        )
    }

    fn handle_scopes(&mut self, request: &DapRequest) -> io::Result<()> {
        // Scopes mirror the evaluator's actual scope model:
        // 1. Current Substitution - the subst map from current function/let evaluation
        // 2. REPL Bindings - from :let commands (evaluator.bindings)
        // 3. Functions - defined functions (evaluator.functions)
        self.respond(
            request,
            true,
            Some(json!({
                "scopes": [{
                    "name": "Current Substitution",
                    "presentationHint": "locals",
                    "variablesReference": 1,
                    "expensive": false
                }, {
                    "name": "REPL Bindings",
                    "presentationHint": "globals",
                    "variablesReference": 2,
                    "expensive": false
                }, {
                    "name": "Functions",
                    "variablesReference": 3,
                    "expensive": false
                }]
            })),
        )
    }

    fn handle_variables(&mut self, request: &DapRequest) -> io::Result<()> {
        let reference = request
            .arguments
            .as_ref()
            .and_then(|a| a.get("variablesReference"))
            .and_then(|r| r.as_i64())
            .unwrap_or(0);

        let variables: Vec<Value> = if reference == 1 {
            // Current Substitution - the subst map from current function/let
            // This mirrors what the evaluator passes to substitute()
            let stack = self.hook_stack.lock().unwrap();
            if let Some(frame) = stack.last() {
                frame
                    .bindings
                    .iter()
                    .map(|(name, value)| {
                        json!({
                            "name": name,
                            "value": value,
                            "type": "substitution",
                            "variablesReference": 0
                        })
                    })
                    .collect()
            } else {
                Vec::new()
            }
        } else if reference == 2 {
            // REPL Bindings - from evaluator.bindings (:let commands)
            // This is the evaluator's self.bindings HashMap
            self.evaluator
                .list_bindings()
                .iter()
                .map(|(name, expr)| {
                    json!({
                        "name": name,
                        "value": format!("{:?}", expr),
                        "type": "binding",
                        "variablesReference": 0
                    })
                })
                .collect()
        } else if reference == 3 {
            // Functions - from evaluator.functions (define commands)
            // This is the evaluator's self.functions HashMap
            self.evaluator
                .list_functions()
                .iter()
                .map(|name| {
                    // Get function info if available
                    let info = self
                        .evaluator
                        .get_function(name)
                        .map(|c| format!("({}) -> ...", c.params.join(", ")))
                        .unwrap_or_else(|| "<function>".to_string());
                    json!({
                        "name": name,
                        "value": info,
                        "type": "function",
                        "variablesReference": 0
                    })
                })
                .collect()
        } else {
            Vec::new()
        };

        self.respond(
            request,
            true,
            Some(json!({
                "variables": variables
            })),
        )
    }

    fn handle_continue(&mut self, request: &DapRequest) -> io::Result<()> {
        self.state = DapDebugState::Running;
        self.respond(
            request,
            true,
            Some(json!({
                "allThreadsContinued": true
            })),
        )?;

        // Signal the hook to continue
        self.shared_state.signal_continue(DebugAction::Continue);

        // The hook will pause again at the next breakpoint
        // For now, we don't wait - the hook will trigger a stopped event
        Ok(())
    }

    fn handle_next(&mut self, request: &DapRequest) -> io::Result<()> {
        // Step over - continue until we're back at the same depth
        self.respond(request, true, None)?;

        // Signal the hook to step over
        self.shared_state.signal_continue(DebugAction::StepOver);

        // The hook will pause and we'll send a stopped event
        self.state = DapDebugState::Paused {
            reason: "step".to_string(),
        };
        self.send_event(
            "stopped",
            Some(json!({
                "reason": "step",
                "threadId": 1,
                "allThreadsStopped": true
            })),
        )
    }

    fn handle_step_in(&mut self, request: &DapRequest) -> io::Result<()> {
        self.respond(request, true, None)?;

        // Signal the hook to step into
        self.shared_state.signal_continue(DebugAction::StepInto);

        self.state = DapDebugState::Paused {
            reason: "step".to_string(),
        };
        self.send_event(
            "stopped",
            Some(json!({
                "reason": "step",
                "threadId": 1,
                "allThreadsStopped": true
            })),
        )
    }

    fn handle_step_out(&mut self, request: &DapRequest) -> io::Result<()> {
        self.respond(request, true, None)?;

        // Signal the hook to step out
        self.shared_state.signal_continue(DebugAction::StepOut);

        self.state = DapDebugState::Paused {
            reason: "step".to_string(),
        };
        self.send_event(
            "stopped",
            Some(json!({
                "reason": "step",
                "threadId": 1,
                "allThreadsStopped": true
            })),
        )
    }

    fn handle_evaluate(&mut self, request: &DapRequest) -> io::Result<()> {
        let expression = request
            .arguments
            .as_ref()
            .and_then(|a| a.get("expression"))
            .and_then(|e| e.as_str())
            .unwrap_or("");

        // Try to parse and evaluate the expression
        let result = match kleis::kleis_parser::parse_kleis(expression) {
            Ok(expr) => match self.evaluator.eval(&expr) {
                Ok(result) => format!("{:?}", result),
                Err(e) => format!("Error: {}", e),
            },
            Err(e) => format!("Parse error: {}", e),
        };

        self.respond(
            request,
            true,
            Some(json!({
                "result": result,
                "variablesReference": 0
            })),
        )
    }

    /// Main message loop
    fn run(&mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin.lock());

        eprintln!("[kleis-debug] Debug adapter started");

        loop {
            // Read Content-Length header
            let mut header = String::new();
            reader.read_line(&mut header)?;

            if header.is_empty() {
                break; // EOF
            }

            // Parse Content-Length
            let content_length: usize = if header.starts_with("Content-Length:") {
                header
                    .trim_start_matches("Content-Length:")
                    .trim()
                    .parse()
                    .unwrap_or(0)
            } else {
                continue;
            };

            // Skip blank line
            let mut blank = String::new();
            reader.read_line(&mut blank)?;

            // Read content
            let mut content = vec![0u8; content_length];
            reader.read_exact(&mut content)?;

            // Parse JSON
            let message: Result<DapRequest, _> = serde_json::from_slice(&content);

            match message {
                Ok(request) => {
                    if let Err(e) = self.handle_request(request) {
                        eprintln!("[kleis-debug] Error handling request: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("[kleis-debug] Failed to parse message: {}", e);
                }
            }

            if self.state == DapDebugState::Terminated {
                break;
            }
        }

        eprintln!("[kleis-debug] Debug adapter terminated");
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut adapter = DebugAdapter::new();
    adapter.run()
}
