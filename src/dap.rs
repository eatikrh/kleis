//! Debug Adapter Protocol (DAP) Implementation for Kleis
//!
//! This module implements the Debug Adapter Protocol, enabling IDE debugging
//! support for Kleis programs in VS Code and other DAP-compatible editors.
//!
//! ## Architecture
//!
//! The DAP server communicates with the IDE over either:
//! - **stdio**: Standard input/output (default for VS Code)
//! - **TCP**: Network socket (useful for development/testing)
//!
//! ## Important: No stdout/stderr in stdio mode!
//!
//! When running in stdio mode, the DAP protocol uses stdin/stdout for
//! communication. Any `println!` or `eprintln!` would corrupt the protocol.
//! Use the `dap_log!` macro which only outputs in TCP mode or to a log file.
//!
//! ## Supported Features
//!
//! - [ ] Launch/Attach
//! - [ ] Breakpoints (line, conditional)
//! - [ ] Step In/Out/Over
//! - [ ] Variable inspection
//! - [ ] Expression evaluation
//! - [ ] Stack traces
//!
//! ## References
//!
//! - [DAP Specification](https://microsoft.github.io/debug-adapter-protocol/)
//! - [debug-adapter-protocol crate](https://docs.rs/debug-adapter-protocol)

use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::context::SharedContext;

/// Global flag: are we in stdio mode? If so, suppress all console output.
static STDIO_MODE: AtomicBool = AtomicBool::new(false);

/// Log macro that only outputs when NOT in stdio mode.
/// In stdio mode, stdout/stderr would corrupt the DAP protocol.
macro_rules! dap_log {
    ($($arg:tt)*) => {
        if !STDIO_MODE.load(Ordering::Relaxed) {
            eprintln!($($arg)*);
        }
    };
}

/// Run the DAP server over stdio (default mode for VS Code)
///
/// **Important:** No console output in this mode! stdout is used for DAP messages.
pub fn run_stdio_server() -> Result<(), Box<dyn std::error::Error>> {
    // Set stdio mode flag - suppresses all dap_log! output
    STDIO_MODE.store(true, Ordering::Relaxed);

    let stdin = io::stdin();
    let stdout = io::stdout();

    run_server(stdin.lock(), stdout.lock())
}

/// Run the DAP server over stdio with shared context
pub fn run_stdio_server_with_context(ctx: SharedContext) -> Result<(), Box<dyn std::error::Error>> {
    // Set stdio mode flag - suppresses all dap_log! output
    STDIO_MODE.store(true, Ordering::Relaxed);

    let stdin = io::stdin();
    let stdout = io::stdout();

    run_server_with_context(stdin.lock(), stdout.lock(), ctx)
}

/// Run the DAP server over TCP (useful for development)
///
/// TCP mode allows console output for debugging.
pub fn run_tcp_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    // TCP mode: console output is allowed
    STDIO_MODE.store(false, Ordering::Relaxed);

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
    dap_log!("🐛 Kleis DAP server listening on port {}", port);

    for stream in listener.incoming() {
        let stream = stream?;
        dap_log!("Client connected: {:?}", stream.peer_addr());

        let reader = BufReader::new(stream.try_clone()?);
        let writer = stream;

        if let Err(e) = run_server(reader, writer) {
            dap_log!("Session error: {}", e);
        }
    }

    Ok(())
}

/// Run the DAP server over TCP with shared context
pub fn run_tcp_server_with_context(
    port: u16,
    ctx: SharedContext,
) -> Result<(), Box<dyn std::error::Error>> {
    // TCP mode: console output is allowed
    STDIO_MODE.store(false, Ordering::Relaxed);

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
    dap_log!("🐛 Kleis DAP server listening on port {}", port);

    for stream in listener.incoming() {
        let stream = stream?;
        dap_log!("Client connected: {:?}", stream.peer_addr());

        let reader = BufReader::new(stream.try_clone()?);
        let writer = stream;

        if let Err(e) = run_server_with_context(reader, writer, ctx.clone()) {
            dap_log!("Session error: {}", e);
        }
    }

    Ok(())
}

/// Main DAP message loop
fn run_server<R: BufRead, W: Write>(
    reader: R,
    writer: W,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut debugger = DapDebugger::new(None);
    let mut reader = reader;
    let mut writer = writer;
    run_server_loop(&mut reader, &mut writer, &mut debugger)
}

fn run_server_with_context<R: BufRead, W: Write>(
    reader: R,
    writer: W,
    ctx: SharedContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut debugger = DapDebugger::new(Some(ctx));
    let mut reader = reader;
    let mut writer = writer;
    run_server_loop(&mut reader, &mut writer, &mut debugger)
}

fn run_server_loop<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    debugger: &mut DapDebugger,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Read DAP message (Content-Length header + JSON body)
        let message = match read_dap_message(reader) {
            Ok(Some(msg)) => msg,
            Ok(None) => break, // EOF
            Err(e) => {
                dap_log!("Read error: {}", e);
                break;
            }
        };

        // Handle the message and get response
        let response = debugger.handle_message(&message);

        // Send response
        if let Some(resp) = response {
            write_dap_message(writer, &resp)?;
        }

        // Check if we should terminate
        if debugger.should_terminate {
            break;
        }
    }

    Ok(())
}

/// Read a DAP message (Content-Length header followed by JSON body)
fn read_dap_message<R: BufRead>(reader: &mut R) -> io::Result<Option<String>> {
    // Read Content-Length header
    let mut header = String::new();
    loop {
        header.clear();
        let bytes_read = reader.read_line(&mut header)?;
        if bytes_read == 0 {
            return Ok(None); // EOF
        }

        let header = header.trim();
        if header.is_empty() {
            continue;
        }

        if header.starts_with("Content-Length:") {
            break;
        }
    }

    // Parse content length
    let content_length: usize = header
        .strip_prefix("Content-Length:")
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid header"))?
        .trim()
        .parse()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Skip the blank line after headers
    let mut blank = String::new();
    reader.read_line(&mut blank)?;

    // Read the JSON body
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body)?;

    String::from_utf8(body)
        .map(Some)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Write a DAP message with Content-Length header
fn write_dap_message<W: Write>(writer: &mut W, message: &str) -> io::Result<()> {
    write!(
        writer,
        "Content-Length: {}\r\n\r\n{}",
        message.len(),
        message
    )?;
    writer.flush()
}

/// The Kleis DAP debugger state
struct DapDebugger {
    /// Sequence number for responses
    seq: i32,
    /// Whether the session should terminate
    should_terminate: bool,
    /// Current breakpoints by file
    breakpoints: std::collections::HashMap<String, Vec<Breakpoint>>,
    /// Whether we're currently stopped
    is_stopped: bool,
    /// Shared context (for accessing evaluator, type checker, etc.)
    context: Option<SharedContext>,
    /// Currently loaded file path
    current_file: Option<String>,
    /// Example blocks found in the program (v0.93)
    example_blocks: Vec<ExampleBlockInfo>,
    /// Current execution state
    execution_state: ExecutionState,
}

/// Info about an example block for debugging
#[derive(Debug, Clone)]
struct ExampleBlockInfo {
    name: String,
    start_line: u32,
    statement_count: usize,
}

/// Current execution state
#[derive(Debug, Clone, Default)]
struct ExecutionState {
    /// Index of current example being debugged (-1 = none)
    current_example: i32,
    /// Index of current statement within example
    current_statement: usize,
    /// Whether execution is paused
    paused: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Breakpoint {
    line: u32,
    condition: Option<String>,
    verified: bool,
}

impl DapDebugger {
    fn new(context: Option<SharedContext>) -> Self {
        Self {
            seq: 0,
            should_terminate: false,
            breakpoints: std::collections::HashMap::new(),
            is_stopped: false,
            context,
            current_file: None,
            example_blocks: Vec::new(),
            execution_state: ExecutionState::default(),
        }
    }

    fn next_seq(&mut self) -> i32 {
        self.seq += 1;
        self.seq
    }

    fn handle_message(&mut self, message: &str) -> Option<String> {
        // Parse the JSON message
        let request: serde_json::Value = match serde_json::from_str(message) {
            Ok(v) => v,
            Err(e) => {
                dap_log!("Failed to parse DAP message: {}", e);
                return None;
            }
        };

        let command = request.get("command")?.as_str()?;
        let request_seq = request.get("seq")?.as_i64()? as i32;

        dap_log!("DAP request: {}", command);

        match command {
            "initialize" => self.handle_initialize(request_seq),
            "launch" => self.handle_launch(request_seq, &request),
            "setBreakpoints" => self.handle_set_breakpoints(request_seq, &request),
            "configurationDone" => self.handle_configuration_done(request_seq),
            "threads" => self.handle_threads(request_seq),
            "stackTrace" => self.handle_stack_trace(request_seq),
            "scopes" => self.handle_scopes(request_seq, &request),
            "variables" => self.handle_variables(request_seq, &request),
            "continue" => self.handle_continue(request_seq),
            "next" => self.handle_next(request_seq),
            "stepIn" => self.handle_step_in(request_seq),
            "stepOut" => self.handle_step_out(request_seq),
            "disconnect" => self.handle_disconnect(request_seq),
            _ => {
                dap_log!("Unhandled DAP command: {}", command);
                Some(self.error_response(request_seq, command, "Not implemented"))
            }
        }
    }

    fn handle_initialize(&mut self, request_seq: i32) -> Option<String> {
        let response = serde_json::json!({
            "seq": self.next_seq(),
            "type": "response",
            "request_seq": request_seq,
            "success": true,
            "command": "initialize",
            "body": {
                "supportsConfigurationDoneRequest": true,
                "supportsFunctionBreakpoints": false,
                "supportsConditionalBreakpoints": true,
                "supportsEvaluateForHovers": true,
                "supportsStepBack": false,
                "supportsSetVariable": false,
                "supportsRestartFrame": false,
                "supportsGotoTargetsRequest": false,
                "supportsStepInTargetsRequest": false,
                "supportsCompletionsRequest": false,
                "supportsModulesRequest": false,
                "supportsExceptionOptions": false,
                "supportsValueFormattingOptions": false,
                "supportsExceptionInfoRequest": false,
                "supportTerminateDebuggee": true,
                "supportsDelayedStackTraceLoading": false,
                "supportsLoadedSourcesRequest": false,
            }
        });
        Some(response.to_string())
    }

    fn handle_launch(&mut self, request_seq: i32, request: &serde_json::Value) -> Option<String> {
        // Get the program to launch
        let program_path = request
            .get("arguments")
            .and_then(|a| a.get("program"))
            .and_then(|p| p.as_str())
            .unwrap_or("");

        dap_log!("Launching: {}", program_path);

        // Load the program into the shared evaluator
        if let Some(ref ctx) = self.context {
            if let Ok(mut ctx_guard) = ctx.write() {
                // Read and parse the file
                match std::fs::read_to_string(program_path) {
                    Ok(source) => {
                        use crate::kleis_parser::parse_kleis_program;
                        match parse_kleis_program(&source) {
                            Ok(program) => {
                                if let Err(e) = ctx_guard.evaluator.load_program(&program) {
                                    dap_log!("Failed to load program: {}", e);
                                    return Some(self.error_response(
                                        request_seq,
                                        "launch",
                                        &format!("Failed to load: {}", e),
                                    ));
                                }
                                dap_log!("Program loaded successfully");
                                
                                // Store the program path for breakpoint matching
                                self.current_file = Some(program_path.to_string());
                                
                                // Detect example blocks (v0.93)
                                use crate::kleis_ast::TopLevel;
                                self.example_blocks.clear();
                                for (idx, item) in program.items.iter().enumerate() {
                                    if let TopLevel::ExampleBlock(ex) = item {
                                        self.example_blocks.push(ExampleBlockInfo {
                                            name: ex.name.clone(),
                                            start_line: (idx + 1) as u32, // Approximate line
                                            statement_count: ex.statements.len(),
                                        });
                                        dap_log!("Found example: {} ({} statements)", ex.name, ex.statements.len());
                                    }
                                }
                                
                                if !self.example_blocks.is_empty() {
                                    dap_log!("Found {} example blocks", self.example_blocks.len());
                                }
                            }
                            Err(e) => {
                                dap_log!("Parse error: {}", e);
                                return Some(self.error_response(
                                    request_seq,
                                    "launch",
                                    &format!("Parse error: {}", e),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        dap_log!("Cannot read file: {}", e);
                        return Some(self.error_response(
                            request_seq,
                            "launch",
                            &format!("Cannot read file: {}", e),
                        ));
                    }
                }
            }
        }

        let response = serde_json::json!({
            "seq": self.next_seq(),
            "type": "response",
            "request_seq": request_seq,
            "success": true,
            "command": "launch"
        });
        Some(response.to_string())
    }

    fn handle_set_breakpoints(
        &mut self,
        request_seq: i32,
        request: &serde_json::Value,
    ) -> Option<String> {
        let args = request.get("arguments")?;
        let source = args.get("source")?;
        let path = source.get("path").and_then(|p| p.as_str()).unwrap_or("");

        let breakpoints: Vec<Breakpoint> = args
            .get("breakpoints")
            .and_then(|b| b.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|bp| {
                        Some(Breakpoint {
                            line: bp.get("line")?.as_u64()? as u32,
                            condition: bp
                                .get("condition")
                                .and_then(|c| c.as_str())
                                .map(String::from),
                            verified: true, // TODO: Validate breakpoint location
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let response_breakpoints: Vec<serde_json::Value> = breakpoints
            .iter()
            .map(|bp| {
                serde_json::json!({
                    "verified": bp.verified,
                    "line": bp.line
                })
            })
            .collect();

        self.breakpoints.insert(path.to_string(), breakpoints);

        let response = serde_json::json!({
            "seq": self.next_seq(),
            "type": "response",
            "request_seq": request_seq,
            "success": true,
            "command": "setBreakpoints",
            "body": {
                "breakpoints": response_breakpoints
            }
        });
        Some(response.to_string())
    }

    fn handle_configuration_done(&mut self, request_seq: i32) -> Option<String> {
        let response = serde_json::json!({
            "seq": self.next_seq(),
            "type": "response",
            "request_seq": request_seq,
            "success": true,
            "command": "configurationDone"
        });

        // Send initialized event
        // TODO: Start execution and stop at first breakpoint

        Some(response.to_string())
    }

    fn handle_threads(&mut self, request_seq: i32) -> Option<String> {
        // Kleis is single-threaded for now
        let response = serde_json::json!({
            "seq": self.next_seq(),
            "type": "response",
            "request_seq": request_seq,
            "success": true,
            "command": "threads",
            "body": {
                "threads": [{
                    "id": 1,
                    "name": "main"
                }]
            }
        });
        Some(response.to_string())
    }

    fn handle_stack_trace(&mut self, request_seq: i32) -> Option<String> {
        // TODO: Return actual stack trace
        let response = serde_json::json!({
            "seq": self.next_seq(),
            "type": "response",
            "request_seq": request_seq,
            "success": true,
            "command": "stackTrace",
            "body": {
                "stackFrames": [],
                "totalFrames": 0
            }
        });
        Some(response.to_string())
    }

    fn handle_scopes(&mut self, request_seq: i32, _request: &serde_json::Value) -> Option<String> {
        // TODO: Return scopes for the given frame
        let response = serde_json::json!({
            "seq": self.next_seq(),
            "type": "response",
            "request_seq": request_seq,
            "success": true,
            "command": "scopes",
            "body": {
                "scopes": [{
                    "name": "Locals",
                    "variablesReference": 1,
                    "expensive": false
                }]
            }
        });
        Some(response.to_string())
    }

    fn handle_variables(
        &mut self,
        request_seq: i32,
        _request: &serde_json::Value,
    ) -> Option<String> {
        // Get variables from the shared evaluator
        let mut variables = Vec::new();
        
        if let Some(ref ctx) = self.context {
            if let Ok(ctx_guard) = ctx.read() {
                // Get all bindings from evaluator
                for (name, value) in ctx_guard.evaluator.get_all_bindings() {
                    variables.push(serde_json::json!({
                        "name": name,
                        "value": format!("{:?}", value),
                        "variablesReference": 0  // No nested variables for now
                    }));
                }
                
                // Also show defined functions
                for func_name in ctx_guard.evaluator.list_functions() {
                    variables.push(serde_json::json!({
                        "name": func_name,
                        "value": "<function>",
                        "variablesReference": 0
                    }));
                }
            }
        }
        
        let response = serde_json::json!({
            "seq": self.next_seq(),
            "type": "response",
            "request_seq": request_seq,
            "success": true,
            "command": "variables",
            "body": {
                "variables": variables
            }
        });
        Some(response.to_string())
    }

    fn handle_continue(&mut self, request_seq: i32) -> Option<String> {
        self.is_stopped = false;
        let response = serde_json::json!({
            "seq": self.next_seq(),
            "type": "response",
            "request_seq": request_seq,
            "success": true,
            "command": "continue",
            "body": {
                "allThreadsContinued": true
            }
        });
        Some(response.to_string())
    }

    fn handle_next(&mut self, request_seq: i32) -> Option<String> {
        // TODO: Step over
        let response = serde_json::json!({
            "seq": self.next_seq(),
            "type": "response",
            "request_seq": request_seq,
            "success": true,
            "command": "next"
        });
        Some(response.to_string())
    }

    fn handle_step_in(&mut self, request_seq: i32) -> Option<String> {
        // TODO: Step into
        let response = serde_json::json!({
            "seq": self.next_seq(),
            "type": "response",
            "request_seq": request_seq,
            "success": true,
            "command": "stepIn"
        });
        Some(response.to_string())
    }

    fn handle_step_out(&mut self, request_seq: i32) -> Option<String> {
        // TODO: Step out
        let response = serde_json::json!({
            "seq": self.next_seq(),
            "type": "response",
            "request_seq": request_seq,
            "success": true,
            "command": "stepOut"
        });
        Some(response.to_string())
    }

    fn handle_disconnect(&mut self, request_seq: i32) -> Option<String> {
        self.should_terminate = true;
        let response = serde_json::json!({
            "seq": self.next_seq(),
            "type": "response",
            "request_seq": request_seq,
            "success": true,
            "command": "disconnect"
        });
        Some(response.to_string())
    }

    fn error_response(&mut self, request_seq: i32, command: &str, message: &str) -> String {
        serde_json::json!({
            "seq": self.next_seq(),
            "type": "response",
            "request_seq": request_seq,
            "success": false,
            "command": command,
            "message": message
        })
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_initialize() {
        let mut debugger = DapDebugger::new(None);
        let init_request = r#"{"seq":1,"type":"request","command":"initialize","arguments":{}}"#;
        let response = debugger.handle_message(init_request);
        assert!(response.is_some());
        let resp: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert_eq!(resp["success"], true);
        assert_eq!(resp["command"], "initialize");
    }
}
