//! Kleis - Unified Binary
//!
//! This is the main entry point for Kleis, providing multiple modes:
//!
//! - `kleis server` - Unified server (LSP + DAP + REPL) for IDE integration
//! - `kleis eval` - Evaluate expressions from command line
//! - `kleis check` - Check a file for parse/type errors
//! - `kleis repl` - Interactive REPL in terminal
//!
//! ## Usage
//!
//! ```bash
//! # IDE integration (VS Code, etc.)
//! kleis server
//!
//! # Command line evaluation
//! kleis eval "1 + 2"
//! kleis eval -f script.kleis
//!
//! # Check files
//! kleis check myfile.kleis
//!
//! # Interactive REPL
//! kleis repl
//! kleis repl --load stdlib/prelude.kleis
//! ```

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Kleis - A symbolic mathematics language
#[derive(Parser)]
#[command(name = "kleis")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the unified server (LSP + DAP) for IDE integration
    Server {
        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },

    /// Evaluate an expression and print the result
    Eval {
        /// Expression to evaluate
        expression: Option<String>,

        /// File to evaluate
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Check a file for parse and type errors
    Check {
        /// File to check
        file: PathBuf,
    },

    /// Start an interactive REPL
    Repl {
        /// Files to load on startup
        #[arg(short, long)]
        load: Vec<PathBuf>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server { verbose } => {
            run_server(verbose).await;
        }
        Commands::Eval { expression, file } => {
            run_eval(expression, file);
        }
        Commands::Check { file } => {
            run_check(file);
        }
        Commands::Repl { load } => {
            run_repl(load);
        }
    }
}

/// Run the unified server (LSP + DAP)
async fn run_server(verbose: bool) {
    if verbose {
        eprintln!("[kleis] Starting unified server (LSP + DAP)");
    }

    // For now, just run the LSP server
    // We'll add DAP spawning via custom LSP requests
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    // Import the LSP types we need
    use tower_lsp::{LspService, Server};

    // Create and run the language server
    // Note: We'll refactor this to use SharedContext once we have that
    let (service, socket) = LspService::new(|client| KleisUnifiedServer::new(client, verbose));

    Server::new(stdin, stdout, socket).serve(service).await;
}

/// Run a one-shot evaluation
fn run_eval(expression: Option<String>, file: Option<PathBuf>) {
    use kleis::evaluator::Evaluator;
    use kleis::kleis_parser::{parse_kleis, parse_kleis_program};

    let mut evaluator = Evaluator::new();
    let has_file = file.is_some();

    // If a file is provided, load it first
    if let Some(file_path) = file {
        match std::fs::read_to_string(&file_path) {
            Ok(source) => match parse_kleis_program(&source) {
                Ok(program) => {
                    if let Err(e) = evaluator.load_program(&program) {
                        eprintln!("Error loading {}: {}", file_path.display(), e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Parse error in {}: {}", file_path.display(), e);
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Cannot read {}: {}", file_path.display(), e);
                std::process::exit(1);
            }
        }
    }

    // Evaluate expression if provided
    if let Some(expr_str) = expression {
        match parse_kleis(&expr_str) {
            Ok(expr) => match evaluator.eval(&expr) {
                Ok(result) => {
                    println!("{:?}", result);
                }
                Err(e) => {
                    eprintln!("Evaluation error: {}", e);
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Parse error: {}", e);
                std::process::exit(1);
            }
        }
    } else if !has_file {
        eprintln!("Error: Provide an expression or --file");
        std::process::exit(1);
    }
}

/// Check a file for errors
fn run_check(file: PathBuf) {
    use kleis::evaluator::Evaluator;
    use kleis::kleis_parser::parse_kleis_program;

    match std::fs::read_to_string(&file) {
        Ok(source) => match parse_kleis_program(&source) {
            Ok(program) => {
                // Try to load into evaluator (validates definitions)
                let mut evaluator = Evaluator::new();
                if let Err(e) = evaluator.load_program(&program) {
                    eprintln!("{}: error: {}", file.display(), e);
                    std::process::exit(1);
                }
                let (funcs, data, structs, _) = evaluator.definition_counts();
                println!(
                    "{}: OK ({} functions, {} data types, {} structures)",
                    file.display(),
                    funcs,
                    data,
                    structs
                );
            }
            Err(e) => {
                eprintln!("{}: parse error: {}", file.display(), e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("{}: {}", file.display(), e);
            std::process::exit(1);
        }
    }
}

/// Run the interactive REPL
fn run_repl(load: Vec<PathBuf>) {
    // For now, just print a message - the full REPL is in repl.rs
    // We'll integrate it properly later
    eprintln!("Starting Kleis REPL...");
    if !load.is_empty() {
        eprintln!("Loading: {:?}", load);
    }
    eprintln!("TODO: Integrate with existing REPL implementation");
    eprintln!("For now, use: cargo run --bin repl");
}

// ============================================================================
// Unified Language Server
// ============================================================================

use dashmap::DashMap;
use kleis::evaluator::Evaluator;
use kleis::kleis_ast::Program;
use kleis::kleis_parser::parse_kleis_program;
use kleis::structure_registry::StructureRegistry;
use kleis::type_checker::TypeChecker;
use ropey::Rope;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

/// Shared context between LSP, DAP, and REPL
struct SharedContext {
    /// The evaluator (holds functions, bindings)
    evaluator: Arc<Mutex<Evaluator>>,
    /// Type checker
    type_checker: Arc<Mutex<TypeChecker>>,
    /// Structure registry
    structure_registry: Arc<Mutex<StructureRegistry>>,
}

impl SharedContext {
    fn new() -> Self {
        Self {
            evaluator: Arc::new(Mutex::new(Evaluator::new())),
            type_checker: Arc::new(Mutex::new(TypeChecker::new())),
            structure_registry: Arc::new(Mutex::new(StructureRegistry::new())),
        }
    }
}

/// Document state
struct Document {
    content: Rope,
    ast: Option<Program>,
}

/// The unified Kleis server
struct KleisUnifiedServer {
    client: Client,
    documents: DashMap<Url, Document>,
    shared: SharedContext,
    verbose: bool,
    /// Port where DAP is listening (if started)
    dap_port: Arc<Mutex<Option<u16>>>,
}

impl KleisUnifiedServer {
    fn new(client: Client, verbose: bool) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            shared: SharedContext::new(),
            verbose,
            dap_port: Arc::new(Mutex::new(None)),
        }
    }

    fn log(&self, msg: &str) {
        if self.verbose {
            eprintln!("[kleis] {}", msg);
        }
    }

    /// Start the DAP server on a dynamic port
    fn start_dap_server(&self) -> std::result::Result<u16, String> {
        // Bind to port 0 - OS assigns available port
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| format!("Failed to bind DAP server: {}", e))?;

        let port = listener
            .local_addr()
            .map_err(|e| format!("Failed to get port: {}", e))?
            .port();

        self.log(&format!("DAP server starting on port {}", port));

        // Clone shared context for the DAP thread
        let evaluator = self.shared.evaluator.clone();
        let verbose = self.verbose;

        // Spawn DAP server thread
        std::thread::spawn(move || {
            if let Err(e) = run_dap_on_listener(listener, evaluator, verbose) {
                eprintln!("[kleis-dap] Error: {}", e);
            }
        });

        // Store the port
        *self.dap_port.lock().unwrap() = Some(port);

        Ok(port)
    }

    /// Parse and validate a document
    fn parse_document(&self, uri: &Url, content: &str) -> (Option<Program>, Vec<Diagnostic>) {
        let mut diagnostics = Vec::new();

        match parse_kleis_program(content) {
            Ok(program) => {
                // Load into shared evaluator
                if let Ok(mut eval) = self.shared.evaluator.lock() {
                    if let Err(e) = eval.load_program(&program) {
                        diagnostics.push(Diagnostic {
                            range: Range::default(),
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: format!("Load error: {}", e),
                            ..Default::default()
                        });
                    }
                }
                (Some(program), diagnostics)
            }
            Err(e) => {
                // Convert parse error to diagnostic
                let line = content[..e.position.min(content.len())]
                    .chars()
                    .filter(|c| *c == '\n')
                    .count() as u32;

                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line, character: 0 },
                        end: Position {
                            line,
                            character: 100,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: e.message,
                    ..Default::default()
                });

                (None, diagnostics)
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for KleisUnifiedServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        self.log("Initializing Kleis unified server");

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["kleis.startDebugSession".to_string()],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.log("Kleis unified server initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        self.log("Shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;

        let (ast, diagnostics) = self.parse_document(&uri, &content);

        self.documents.insert(
            uri.clone(),
            Document {
                content: Rope::from_str(&content),
                ast,
            },
        );

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(change) = params.content_changes.into_iter().next() {
            let content = change.text;
            let (ast, diagnostics) = self.parse_document(&uri, &content);

            self.documents.insert(
                uri.clone(),
                Document {
                    content: Rope::from_str(&content),
                    ast,
                },
            );

            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        // Basic hover - show "Kleis" for now
        // TODO: Integrate with type checker for real hover info
        let _uri = params.text_document_position_params.text_document.uri;
        let _position = params.text_document_position_params.position;

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "**Kleis** - Unified Server".to_string(),
            }),
            range: None,
        }))
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        // Basic completions
        let items = vec![
            CompletionItem {
                label: "define".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Define a function".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "let".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Local binding".to_string()),
                ..Default::default()
            },
        ];

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        match params.command.as_str() {
            "kleis.startDebugSession" => {
                self.log("Received kleis.startDebugSession command");

                // Extract program path from arguments
                let program_path = params
                    .arguments
                    .first()
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Start DAP server on dynamic port
                match self.start_dap_server() {
                    Ok(port) => {
                        self.log(&format!("DAP server started on port {}", port));
                        Ok(Some(serde_json::json!({
                            "port": port,
                            "program": program_path
                        })))
                    }
                    Err(e) => {
                        self.log(&format!("Failed to start DAP: {}", e));
                        Ok(Some(serde_json::json!({
                            "error": e
                        })))
                    }
                }
            }
            other => {
                self.log(&format!("Unknown command: {}", other));
                Ok(None)
            }
        }
    }
}

/// Run the DAP server on an existing listener
fn run_dap_on_listener(
    listener: TcpListener,
    evaluator: Arc<Mutex<Evaluator>>,
    verbose: bool,
) -> std::result::Result<(), String> {
    use std::io::{BufRead, BufReader, Read, Write};

    if verbose {
        eprintln!("[kleis-dap] Waiting for connection...");
    }

    // Accept one connection
    let (stream, addr) = listener
        .accept()
        .map_err(|e| format!("Accept failed: {}", e))?;

    if verbose {
        eprintln!("[kleis-dap] Connected from {}", addr);
    }

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = stream;

    // Simple DAP message loop
    loop {
        // Read Content-Length header
        let mut header = String::new();
        if reader.read_line(&mut header).map_err(|e| e.to_string())? == 0 {
            break; // EOF
        }

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
        reader.read_line(&mut blank).ok();

        // Read content
        let mut content = vec![0u8; content_length];
        reader.read_exact(&mut content).map_err(|e| e.to_string())?;

        // Parse and handle request
        if let Ok(request) = serde_json::from_slice::<serde_json::Value>(&content) {
            if verbose {
                if let Some(cmd) = request.get("command").and_then(|c| c.as_str()) {
                    eprintln!("[kleis-dap] Request: {}", cmd);
                }
            }

            // Handle the request (simplified for now)
            let response = handle_dap_request(&request, &evaluator);

            // Send response
            let response_str = serde_json::to_string(&response).unwrap();
            let header = format!("Content-Length: {}\r\n\r\n", response_str.len());
            writer.write_all(header.as_bytes()).ok();
            writer.write_all(response_str.as_bytes()).ok();
            writer.flush().ok();

            // Check for terminate
            if request.get("command").and_then(|c| c.as_str()) == Some("disconnect") {
                break;
            }
        }
    }

    if verbose {
        eprintln!("[kleis-dap] Session ended");
    }

    Ok(())
}

/// Handle a DAP request (simplified)
fn handle_dap_request(
    request: &serde_json::Value,
    _evaluator: &Arc<Mutex<Evaluator>>,
) -> serde_json::Value {
    let seq = request.get("seq").and_then(|s| s.as_i64()).unwrap_or(0);
    let command = request
        .get("command")
        .and_then(|c| c.as_str())
        .unwrap_or("");

    match command {
        "initialize" => {
            serde_json::json!({
                "seq": 1,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "initialize",
                "body": {
                    "supportsConfigurationDoneRequest": true,
                    "supportsEvaluateForHovers": true
                }
            })
        }
        "launch" | "attach" => {
            serde_json::json!({
                "seq": 1,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": command
            })
        }
        "disconnect" | "terminate" => {
            serde_json::json!({
                "seq": 1,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": command
            })
        }
        _ => {
            serde_json::json!({
                "seq": 1,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": command
            })
        }
    }
}
