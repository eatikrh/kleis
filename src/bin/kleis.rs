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

    /// Run example blocks as tests (v0.93)
    Test {
        /// File containing example blocks to test
        file: PathBuf,

        /// Run only examples matching this name
        #[arg(short, long)]
        example: Option<String>,

        /// Show detailed output for each example
        #[arg(short, long)]
        verbose: bool,
    },

    /// Start an interactive REPL
    Repl {
        /// Files to load on startup
        #[arg(short, long)]
        load: Vec<PathBuf>,
    },

    /// Start standalone DAP server over stdio (for IDE debugging without LSP)
    Dap {
        /// Enable verbose logging (to stderr)
        #[arg(short, long)]
        verbose: bool,
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
        Commands::Test {
            file,
            example,
            verbose,
        } => {
            run_test(file, example, verbose);
        }
        Commands::Repl { load } => {
            run_repl(load);
        }
        Commands::Dap { verbose } => {
            run_dap(verbose);
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

/// Run example blocks as tests (v0.93)
fn run_test(file: PathBuf, example_filter: Option<String>, verbose: bool) {
    use kleis::evaluator::Evaluator;
    use kleis::kleis_ast::TopLevel;
    use kleis::kleis_parser::parse_kleis_program;

    // Read the file
    let source = match std::fs::read_to_string(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}: {}", file.display(), e);
            std::process::exit(1);
        }
    };

    // Parse the program
    let program = match parse_kleis_program(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}: parse error: {}", file.display(), e);
            std::process::exit(1);
        }
    };

    // Load the program into evaluator
    let mut evaluator = Evaluator::new();
    if let Err(e) = evaluator.load_program(&program) {
        eprintln!("{}: error: {}", file.display(), e);
        std::process::exit(1);
    }

    // Count example blocks
    let examples: Vec<_> = program
        .items
        .iter()
        .filter_map(|item| {
            if let TopLevel::ExampleBlock(ex) = item {
                Some(ex)
            } else {
                None
            }
        })
        .collect();

    if examples.is_empty() {
        println!("{}: no example blocks found", file.display());
        return;
    }

    // Run examples
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for example in &examples {
        // Apply filter if specified
        if let Some(ref filter) = example_filter {
            if !example.name.contains(filter.as_str()) {
                skipped += 1;
                if verbose {
                    println!("⏭️  {} (skipped)", example.name);
                }
                continue;
            }
        }

        let result = evaluator.eval_example_block(example);

        if result.passed {
            passed += 1;
            if verbose {
                println!(
                    "✅ {}: passed ({}/{} assertions)",
                    result.name, result.assertions_passed, result.assertions_total
                );
            } else {
                println!("✅ {}", result.name);
            }
        } else {
            failed += 1;
            println!("❌ {}", result.name);
            if let Some(error) = &result.error {
                println!("   {}", error);
            }
        }
    }

    // Print summary
    println!();
    let total = passed + failed;
    if failed == 0 {
        println!(
            "✅ {} example{} passed",
            total,
            if total == 1 { "" } else { "s" }
        );
        if skipped > 0 {
            println!("   ({} skipped by filter)", skipped);
        }
    } else {
        println!(
            "❌ {}/{} examples passed ({} failed)",
            passed, total, failed
        );
        std::process::exit(1);
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

/// Run standalone DAP server over stdio
/// This is used when the LSP server is not available
fn run_dap(verbose: bool) {
    use kleis::dap::run_stdio_server;

    if verbose {
        eprintln!("[kleis-dap] Starting standalone DAP server over stdio");
    }

    if let Err(e) = run_stdio_server() {
        eprintln!("[kleis-dap] Error: {}", e);
        std::process::exit(1);
    }
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
    fn parse_document(&self, _uri: &Url, content: &str) -> (Option<Program>, Vec<Diagnostic>) {
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
/// DAP session state with smart stepping using AST spans
struct DapState {
    current_file: Option<String>,
    current_line: u32,
    /// Statement line numbers from parsed AST (real source locations)
    statement_lines: Vec<u32>,
    /// Index into statement_lines for current position
    stmt_index: usize,
    /// Fallback: lines that contain executable code (text-based analysis)
    executable_lines: Vec<u32>,
    /// Total lines in file
    total_lines: u32,
    /// Validated breakpoints (only on statement lines)
    breakpoints: Vec<u32>,
}

impl Default for DapState {
    fn default() -> Self {
        Self {
            current_file: None,
            current_line: 1,
            statement_lines: Vec::new(),
            stmt_index: 0,
            executable_lines: Vec::new(),
            total_lines: 0,
            breakpoints: Vec::new(),
        }
    }
}

impl DapState {
    /// Parse file and extract statement line numbers from AST
    fn load_file(&mut self, path: &str) -> std::result::Result<(), String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read file: {}", e))?;
        
        // Canonicalize path for VS Code (needs absolute paths in stack traces)
        let path_buf = std::path::PathBuf::from(path);
        let canonical = path_buf.canonicalize().unwrap_or(path_buf);
        let canonical_str = canonical.to_string_lossy().to_string();
        
        self.current_file = Some(canonical_str.clone());
        self.statement_lines.clear();
        self.executable_lines.clear();
        self.stmt_index = 0;
        
        let lines: Vec<&str> = content.lines().collect();
        self.total_lines = lines.len() as u32;
        
        // Parse with canonicalized file path for VS Code debugging support
        if let Ok(program) = kleis::kleis_parser::parse_kleis_program_with_file(&content, &canonical_str) {
            for item in &program.items {
                self.extract_item_lines(item);
            }
        }
        
        // Fallback: also do text-based analysis for non-example code
        if self.statement_lines.is_empty() {
            for (i, line) in lines.iter().enumerate() {
                let line_num = (i + 1) as u32;
                if Self::is_executable_line(line) {
                    self.executable_lines.push(line_num);
                }
            }
        }
        
        // Combine and deduplicate
        let mut all_lines: Vec<u32> = self.statement_lines.clone();
        all_lines.extend(&self.executable_lines);
        all_lines.sort();
        all_lines.dedup();
        self.statement_lines = all_lines;
        
        // Start at first statement line
        if let Some(&first) = self.statement_lines.first() {
            self.current_line = first;
            self.stmt_index = 0;
        } else {
            self.current_line = 1;
        }
        
        Ok(())
    }
    
    /// Extract line numbers from a top-level item
    fn extract_item_lines(&mut self, item: &kleis::kleis_ast::TopLevel) {
        use kleis::kleis_ast::{TopLevel, ExampleStatement};
        
        match item {
            TopLevel::ExampleBlock(example) => {
                // Extract line numbers from each statement
                for stmt in &example.statements {
                    match stmt {
                        ExampleStatement::Let { location, .. } => {
                            if let Some(loc) = location {
                                self.statement_lines.push(loc.line);
                            }
                        }
                        ExampleStatement::Assert { location, .. } => {
                            if let Some(loc) = location {
                                self.statement_lines.push(loc.line);
                            }
                        }
                        ExampleStatement::Expr { location, .. } => {
                            if let Some(loc) = location {
                                self.statement_lines.push(loc.line);
                            }
                        }
                    }
                }
            }
            TopLevel::FunctionDef(func) => {
                // Function definitions also have spans
                if let Some(sp) = &func.span {
                    self.statement_lines.push(sp.line);
                }
            }
            _ => {
                // Other top-level items don't have spans yet
            }
        }
    }
    
    /// Check if a line contains executable code (text-based fallback)
    fn is_executable_line(line: &str) -> bool {
        let trimmed = line.trim();
        
        if trimmed.is_empty() { return false; }
        if trimmed.starts_with("//") || trimmed.starts_with('#') { return false; }
        if trimmed == "{" || trimmed == "}" || trimmed == "(" || trimmed == ")" { return false; }
        if trimmed.starts_with("/*") || trimmed.ends_with("*/") || trimmed == "*/" { return false; }
        
        true
    }
    
    /// Step to next statement, returns true if stepped
    fn step_next(&mut self) -> bool {
        if self.stmt_index + 1 < self.statement_lines.len() {
            self.stmt_index += 1;
            self.current_line = self.statement_lines[self.stmt_index];
            true
        } else {
            false // No more statements
        }
    }
    
    /// Validate a breakpoint - returns true if it's on a statement line
    fn validate_breakpoint(&self, line: u32) -> bool {
        self.statement_lines.contains(&line)
    }
    
    /// Check if current line hits a breakpoint
    fn is_at_breakpoint(&self) -> bool {
        self.breakpoints.contains(&self.current_line)
    }
}

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
    let mut state = DapState::default();

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

            // Handle the request - may return multiple messages (response + events)
            let messages = handle_dap_request(&request, &evaluator, &mut state);

            // Send all messages
            for msg in messages {
                let msg_str = serde_json::to_string(&msg).unwrap();
                let header = format!("Content-Length: {}\r\n\r\n", msg_str.len());
                writer.write_all(header.as_bytes()).ok();
                writer.write_all(msg_str.as_bytes()).ok();
                writer.flush().ok();
            }

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

/// Handle a DAP request with actual evaluator integration
/// Returns a vector of messages to send (response + any events)
fn handle_dap_request(
    request: &serde_json::Value,
    evaluator: &Arc<Mutex<Evaluator>>,
    state: &mut DapState,
) -> Vec<serde_json::Value> {
    let seq = request.get("seq").and_then(|s| s.as_i64()).unwrap_or(0);
    let command = request
        .get("command")
        .and_then(|c| c.as_str())
        .unwrap_or("");

    match command {
        "initialize" => {
            // Return both the response AND the initialized event
            // The initialized event tells VS Code we're ready for configuration
            vec![
                serde_json::json!({
                    "seq": 1,
                    "type": "response",
                    "request_seq": seq,
                    "success": true,
                    "command": "initialize",
                    "body": {
                        "supportsConfigurationDoneRequest": true,
                        "supportsEvaluateForHovers": true,
                        "supportsConditionalBreakpoints": true,
                        "supportsStepInTargetsRequest": true
                    }
                }),
                // CRITICAL: The initialized EVENT - without this, VS Code won't proceed!
                serde_json::json!({
                    "seq": 2,
                    "type": "event",
                    "event": "initialized"
                })
            ]
        }
        "launch" => {
            // Load program from file
            let program_path = request
                .get("arguments")
                .and_then(|a| a.get("program"))
                .and_then(|p| p.as_str());

            eprintln!("[kleis-dap] Launch request, program: {:?}", program_path);

            if let Some(program_path) = program_path {
                // Load file into state - this parses executable lines
                if let Err(e) = state.load_file(program_path) {
                    return vec![serde_json::json!({
                        "seq": 1,
                        "type": "response",
                        "request_seq": seq,
                        "success": false,
                        "command": "launch",
                        "message": e
                    })];
                }
                
                eprintln!(
                    "[kleis-dap] Loaded file with {} executable lines, starting at line {}",
                    state.executable_lines.len(),
                    state.current_line
                );
                
                // Parse and load into evaluator
                match std::fs::read_to_string(program_path) {
                    Ok(source) => match parse_kleis_program(&source) {
                        Ok(program) => {
                            if let Ok(mut eval) = evaluator.lock() {
                                if let Err(e) = eval.load_program(&program) {
                                    return vec![serde_json::json!({
                                        "seq": 1,
                                        "type": "response",
                                        "request_seq": seq,
                                        "success": false,
                                        "command": "launch",
                                        "message": format!("Load error: {}", e)
                                    })];
                                }
                            }
                        }
                        Err(e) => {
                            return vec![serde_json::json!({
                                "seq": 1,
                                "type": "response",
                                "request_seq": seq,
                                "success": false,
                                "command": "launch",
                                "message": format!("Parse error: {}", e)
                            })];
                        }
                    },
                    Err(e) => {
                        return vec![serde_json::json!({
                            "seq": 1,
                            "type": "response",
                            "request_seq": seq,
                            "success": false,
                            "command": "launch",
                            "message": format!("Cannot read file: {}", e)
                        })];
                    }
                }
            } else {
                eprintln!("[kleis-dap] No program path provided!");
                return vec![serde_json::json!({
                    "seq": 1,
                    "type": "response",
                    "request_seq": seq,
                    "success": false,
                    "command": "launch",
                    "message": "No program path provided"
                })];
            }
            vec![serde_json::json!({
                "seq": 1,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "launch"
            })]
        }
        "attach" => {
            vec![serde_json::json!({
                "seq": 1,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "attach"
            })]
        }
        "threads" => {
            vec![serde_json::json!({
                "seq": 1,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "threads",
                "body": {
                    "threads": [{
                        "id": 1,
                        "name": "main"
                    }]
                }
            })]
        }
        "stackTrace" => {
            // Return a stack frame with proper source info
            let file_path = state.current_file.clone().unwrap_or_default();
            let file_name = file_path.split('/').last().unwrap_or("unknown").to_string();
            
            vec![serde_json::json!({
                "seq": 1,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "stackTrace",
                "body": {
                    "stackFrames": [{
                        "id": 1,
                        "name": "<top-level>",
                        "source": {
                            "name": file_name,
                            "path": file_path
                        },
                        "line": state.current_line,
                        "column": 1
                    }],
                    "totalFrames": 1
                }
            })]
        }
        "scopes" => {
            vec![serde_json::json!({
                "seq": 1,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "scopes",
                "body": {
                    "scopes": [{
                        "name": "Globals",
                        "variablesReference": 1,
                        "expensive": false
                    }]
                }
            })]
        }
        "variables" => {
            // Return actual variables from the evaluator
            let mut variables = Vec::new();
            if let Ok(eval) = evaluator.lock() {
                // Get all bindings
                for (name, value) in eval.get_all_bindings() {
                    variables.push(serde_json::json!({
                        "name": name,
                        "value": format!("{:?}", value),
                        "variablesReference": 0
                    }));
                }
                // Show function count
                let func_count = eval.list_functions().len();
                variables.push(serde_json::json!({
                    "name": "<functions>",
                    "value": format!("{} defined", func_count),
                    "variablesReference": 0
                }));
            }
            vec![serde_json::json!({
                "seq": 1,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "variables",
                "body": {
                    "variables": variables
                }
            })]
        }
        "evaluate" => {
            // Evaluate an expression
            let result = if let Some(expr_str) = request
                .get("arguments")
                .and_then(|a| a.get("expression"))
                .and_then(|e| e.as_str())
            {
                match kleis::kleis_parser::parse_kleis(expr_str) {
                    Ok(expr) => {
                        if let Ok(eval) = evaluator.lock() {
                            match eval.eval(&expr) {
                                Ok(result) => format!("{:?}", result),
                                Err(e) => format!("Error: {}", e),
                            }
                        } else {
                            "Evaluator locked".to_string()
                        }
                    }
                    Err(e) => format!("Parse error: {}", e),
                }
            } else {
                "No expression".to_string()
            };
            vec![serde_json::json!({
                "seq": 1,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "evaluate",
                "body": {
                    "result": result,
                    "variablesReference": 0
                }
            })]
        }
        "setBreakpoints" => {
            // Extract requested breakpoint lines and validate them
            let mut breakpoints_response = Vec::new();
            state.breakpoints.clear();
            
            if let Some(args) = request.get("arguments") {
                if let Some(bps) = args.get("breakpoints").and_then(|b| b.as_array()) {
                    for bp in bps {
                        if let Some(line) = bp.get("line").and_then(|l| l.as_u64()) {
                            let line = line as u32;
                            let is_valid = state.validate_breakpoint(line);
                            
                            if is_valid {
                                state.breakpoints.push(line);
                            }
                            
                            let mut bp = serde_json::json!({
                                "verified": is_valid,
                                "line": line
                            });
                            if !is_valid {
                                bp["message"] = serde_json::Value::String(
                                    "Cannot set breakpoint on this line (comment or empty)".to_string()
                                );
                            }
                            breakpoints_response.push(bp);
                        }
                    }
                }
            }
            
            vec![serde_json::json!({
                "seq": 1,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": "setBreakpoints",
                "body": { "breakpoints": breakpoints_response }
            })]
        }
        "configurationDone" => {
            // Response + stopped event + output events
            vec![
                // Response
                serde_json::json!({
                    "seq": 1,
                    "type": "response",
                    "request_seq": seq,
                    "success": true,
                    "command": "configurationDone"
                }),
                // Output to Debug Console
                serde_json::json!({
                    "seq": 2,
                    "type": "event",
                    "event": "output",
                    "body": {
                        "category": "console",
                        "output": "🐛 Kleis Debugger\nPaused at entry point. Use Step Over (F10) to step through code.\n"
                    }
                }),
                // Stopped event - THIS IS CRITICAL for VS Code to show Paused state
                serde_json::json!({
                    "seq": 3,
                    "type": "event",
                    "event": "stopped",
                    "body": {
                        "reason": "entry",
                        "description": "Paused on entry",
                        "threadId": 1,
                        "allThreadsStopped": true
                    }
                }),
            ]
        }
        "continue" => {
            // Run until breakpoint or end
            let mut hit_breakpoint = false;
            while state.step_next() {
                if state.is_at_breakpoint() {
                    hit_breakpoint = true;
                    break;
                }
            }
            
            if hit_breakpoint {
                // Stopped at breakpoint
                vec![
                    serde_json::json!({
                        "seq": 1,
                        "type": "response",
                        "request_seq": seq,
                        "success": true,
                        "command": "continue"
                    }),
                    serde_json::json!({
                        "seq": 2,
                        "type": "event",
                        "event": "stopped",
                        "body": {
                            "reason": "breakpoint",
                            "threadId": 1,
                            "allThreadsStopped": true
                        }
                    })
                ]
            } else {
                // End of file - terminate
                vec![
                    serde_json::json!({
                        "seq": 1,
                        "type": "response",
                        "request_seq": seq,
                        "success": true,
                        "command": "continue"
                    }),
                    serde_json::json!({
                        "seq": 2,
                        "type": "event",
                        "event": "output",
                        "body": {
                            "category": "console",
                            "output": "\n✅ Execution completed\n"
                        }
                    }),
                    serde_json::json!({
                        "seq": 3,
                        "type": "event",
                        "event": "terminated"
                    })
                ]
            }
        }
        "next" | "stepIn" | "stepOut" => {
            // Step to next executable line
            if state.step_next() {
                // Check if we hit a breakpoint
                let reason = if state.is_at_breakpoint() { "breakpoint" } else { "step" };
                
                // Return response + stopped event
                vec![
                    serde_json::json!({
                        "seq": 1,
                        "type": "response",
                        "request_seq": seq,
                        "success": true,
                        "command": command
                    }),
                    serde_json::json!({
                        "seq": 2,
                        "type": "event",
                        "event": "stopped",
                        "body": {
                            "reason": reason,
                            "threadId": 1,
                            "allThreadsStopped": true
                        }
                    })
                ]
            } else {
                // End of file - terminate
                vec![
                    serde_json::json!({
                        "seq": 1,
                        "type": "response",
                        "request_seq": seq,
                        "success": true,
                        "command": command
                    }),
                    serde_json::json!({
                        "seq": 2,
                        "type": "event",
                        "event": "output",
                        "body": {
                            "category": "console",
                            "output": "\n✅ End of file reached\n"
                        }
                    }),
                    serde_json::json!({
                        "seq": 3,
                        "type": "event",
                        "event": "terminated"
                    })
                ]
            }
        }
        "disconnect" | "terminate" => {
            vec![serde_json::json!({
                "seq": 1,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": command
            })]
        }
        _ => {
            vec![serde_json::json!({
                "seq": 1,
                "type": "response",
                "request_seq": seq,
                "success": true,
                "command": command
            })]
        }
    }
}
