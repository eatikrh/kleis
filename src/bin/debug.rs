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

use std::io::{self, BufRead, BufReader, Read, Write};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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

/// Debug adapter state
struct DebugAdapter {
    /// Sequence number for outgoing messages
    seq: i64,
    /// Breakpoints by file path
    breakpoints: HashMap<String, Vec<Breakpoint>>,
    /// Whether we're currently debugging
    running: bool,
    /// Current evaluation state
    state: DebugState,
}

#[derive(Debug, Clone)]
struct Breakpoint {
    line: i64,
    #[allow(dead_code)]
    verified: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum DebugState {
    NotStarted,
    Running,
    Paused { reason: String },
    Terminated,
}

impl DebugAdapter {
    fn new() -> Self {
        Self {
            seq: 1,
            breakpoints: HashMap::new(),
            running: false,
            state: DebugState::NotStarted,
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
    fn respond(&mut self, request: &DapRequest, success: bool, body: Option<Value>) -> io::Result<()> {
        let response = DapResponse {
            seq: self.next_seq(),
            request_seq: request.seq,
            success,
            command: request.command.clone(),
            message: if success { None } else { Some("Error".to_string()) },
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
        eprintln!("[kleis-debug] Received: {} (seq={})", request.command, request.seq);
        
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
        self.state = DebugState::Running;
        
        // Extract program path from arguments
        if let Some(args) = &request.arguments {
            if let Some(program) = args.get("program").and_then(|p| p.as_str()) {
                eprintln!("[kleis-debug] Launching: {}", program);
            }
        }
        
        self.respond(request, true, None)?;
        
        // For now, immediately stop at entry
        self.state = DebugState::Paused { reason: "entry".to_string() };
        self.send_event("stopped", Some(json!({
            "reason": "entry",
            "threadId": 1,
            "allThreadsStopped": true
        })))
    }

    fn handle_attach(&mut self, request: &DapRequest) -> io::Result<()> {
        // Attach is similar to launch for us
        self.handle_launch(request)
    }

    fn handle_disconnect(&mut self, request: &DapRequest) -> io::Result<()> {
        self.running = false;
        self.state = DebugState::Terminated;
        self.respond(request, true, None)
    }

    fn handle_terminate(&mut self, request: &DapRequest) -> io::Result<()> {
        self.running = false;
        self.state = DebugState::Terminated;
        self.respond(request, true, None)?;
        self.send_event("terminated", None)
    }

    fn handle_set_breakpoints(&mut self, request: &DapRequest) -> io::Result<()> {
        let mut breakpoints = Vec::new();
        
        if let Some(args) = &request.arguments {
            if let Some(source) = args.get("source") {
                let path = source.get("path")
                    .and_then(|p| p.as_str())
                    .unwrap_or("");
                
                if let Some(bps) = args.get("breakpoints").and_then(|b| b.as_array()) {
                    let mut file_breakpoints = Vec::new();
                    
                    for bp in bps {
                        if let Some(line) = bp.get("line").and_then(|l| l.as_i64()) {
                            file_breakpoints.push(Breakpoint {
                                line,
                                verified: true,
                            });
                            breakpoints.push(json!({
                                "verified": true,
                                "line": line
                            }));
                        }
                    }
                    
                    self.breakpoints.insert(path.to_string(), file_breakpoints);
                }
            }
        }
        
        self.respond(request, true, Some(json!({
            "breakpoints": breakpoints
        })))
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
        self.respond(request, true, Some(json!({
            "threads": [{
                "id": 1,
                "name": "Main Thread"
            }]
        })))
    }

    fn handle_stack_trace(&mut self, request: &DapRequest) -> io::Result<()> {
        // TODO: Implement real stack trace from evaluator
        self.respond(request, true, Some(json!({
            "stackFrames": [{
                "id": 1,
                "name": "<top-level>",
                "source": {
                    "name": "repl",
                    "path": "<repl>"
                },
                "line": 1,
                "column": 1
            }],
            "totalFrames": 1
        })))
    }

    fn handle_scopes(&mut self, request: &DapRequest) -> io::Result<()> {
        // TODO: Implement real scopes from evaluator
        self.respond(request, true, Some(json!({
            "scopes": [{
                "name": "Local",
                "variablesReference": 1,
                "expensive": false
            }, {
                "name": "Global",
                "variablesReference": 2,
                "expensive": false
            }]
        })))
    }

    fn handle_variables(&mut self, request: &DapRequest) -> io::Result<()> {
        // TODO: Implement real variable inspection
        let _reference = request.arguments
            .as_ref()
            .and_then(|a| a.get("variablesReference"))
            .and_then(|r| r.as_i64())
            .unwrap_or(0);
        
        self.respond(request, true, Some(json!({
            "variables": [{
                "name": "it",
                "value": "<no value>",
                "variablesReference": 0
            }]
        })))
    }

    fn handle_continue(&mut self, request: &DapRequest) -> io::Result<()> {
        self.state = DebugState::Running;
        self.respond(request, true, Some(json!({
            "allThreadsContinued": true
        })))?;
        
        // TODO: Actually continue evaluation
        // For now, immediately terminate
        self.state = DebugState::Terminated;
        self.send_event("terminated", None)
    }

    fn handle_next(&mut self, request: &DapRequest) -> io::Result<()> {
        // Step over
        self.respond(request, true, None)?;
        
        // TODO: Implement actual stepping
        self.send_event("stopped", Some(json!({
            "reason": "step",
            "threadId": 1,
            "allThreadsStopped": true
        })))
    }

    fn handle_step_in(&mut self, request: &DapRequest) -> io::Result<()> {
        self.respond(request, true, None)?;
        
        // TODO: Implement actual stepping
        self.send_event("stopped", Some(json!({
            "reason": "step",
            "threadId": 1,
            "allThreadsStopped": true
        })))
    }

    fn handle_step_out(&mut self, request: &DapRequest) -> io::Result<()> {
        self.respond(request, true, None)?;
        
        // TODO: Implement actual stepping
        self.send_event("stopped", Some(json!({
            "reason": "step",
            "threadId": 1,
            "allThreadsStopped": true
        })))
    }

    fn handle_evaluate(&mut self, request: &DapRequest) -> io::Result<()> {
        let expression = request.arguments
            .as_ref()
            .and_then(|a| a.get("expression"))
            .and_then(|e| e.as_str())
            .unwrap_or("");
        
        // TODO: Actually evaluate using Kleis evaluator
        self.respond(request, true, Some(json!({
            "result": format!("<cannot evaluate '{}' yet>", expression),
            "variablesReference": 0
        })))
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
                header.trim_start_matches("Content-Length:")
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
            
            if self.state == DebugState::Terminated {
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

