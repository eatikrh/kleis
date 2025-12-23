#![allow(deprecated)] // SymbolInformation::deprecated field is deprecated in lsp-types
//! Kleis Language Server Protocol (LSP) Implementation
//!
//! This provides IDE support for Kleis via the Language Server Protocol:
//! - Real-time diagnostics (parse errors, type errors)
//! - Hover information (type signatures from imports)
//! - Go to definition
//! - Document symbols
//! - Context-aware completions (based on imports)
//!
//! ## Usage
//!
//! ```bash
//! cargo build --release --bin kleis-lsp
//! ```
//!
//! Then configure your editor to use `target/release/kleis-lsp` as the
//! language server for `.kleis` files.

use dashmap::DashMap;
use ropey::Rope;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use kleis::kleis_ast::{Program, TopLevel};
use kleis::kleis_parser::{parse_kleis_program, KleisParseError};
use kleis::type_context::TypeContextBuilder;

/// Document state stored by the language server
struct Document {
    /// The document content as a rope (efficient for edits)
    content: Rope,
    /// The parsed AST (if parsing succeeded)
    ast: Option<Program>,
    /// Type context built from this document and its imports
    type_context: Option<TypeContextBuilder>,
    /// List of import paths found in this document
    imports: Vec<String>,
}

/// The Kleis Language Server
struct KleisLanguageServer {
    /// LSP client for sending notifications
    client: Client,
    /// Open documents indexed by URI
    documents: DashMap<Url, Document>,
}

impl KleisLanguageServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
        }
    }

    /// Resolve an import path relative to the document's directory
    fn resolve_import_path(import_path: &str, base_dir: &Path) -> PathBuf {
        let import = Path::new(import_path);

        if import.is_absolute() {
            // Absolute path: use as-is
            import.to_path_buf()
        } else if import_path.starts_with("stdlib/") {
            // Standard library: look in common locations
            // 1. Current working directory
            // 2. Relative to workspace root
            let cwd_path = PathBuf::from(import_path);
            if cwd_path.exists() {
                return cwd_path;
            }

            // Try parent directories
            let mut search_dir = base_dir.to_path_buf();
            for _ in 0..5 {
                let candidate = search_dir.join(import_path);
                if candidate.exists() {
                    return candidate;
                }
                if !search_dir.pop() {
                    break;
                }
            }

            // Fallback: return as relative to cwd
            PathBuf::from(import_path)
        } else {
            // Relative import: resolve from the importing file's directory
            base_dir.join(import)
        }
    }

    /// Load and parse an import file, building type context
    fn load_import(
        path: &Path,
        loaded: &mut HashSet<PathBuf>,
        builder: &mut TypeContextBuilder,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Avoid cycles
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if loaded.contains(&canonical) {
            return diagnostics;
        }
        loaded.insert(canonical.clone());

        // Read and parse the file
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                diagnostics.push(Diagnostic {
                    range: Range::default(),
                    severity: Some(DiagnosticSeverity::WARNING),
                    source: Some("kleis".to_string()),
                    message: format!("Could not load import '{}': {}", path.display(), e),
                    ..Default::default()
                });
                return diagnostics;
            }
        };

        let program = match parse_kleis_program(&content) {
            Ok(p) => p,
            Err(e) => {
                diagnostics.push(Diagnostic {
                    range: Range::default(),
                    severity: Some(DiagnosticSeverity::WARNING),
                    source: Some("kleis".to_string()),
                    message: format!("Parse error in '{}': {}", path.display(), e.message),
                    ..Default::default()
                });
                return diagnostics;
            }
        };

        // Process nested imports first
        let base_dir = path.parent().unwrap_or(Path::new("."));
        for item in &program.items {
            if let TopLevel::Import(import_path) = item {
                let resolved = Self::resolve_import_path(import_path, base_dir);
                let import_diags = Self::load_import(&resolved, loaded, builder);
                diagnostics.extend(import_diags);
            }
        }

        // Build type context from this file
        if let Ok(file_builder) = TypeContextBuilder::from_program(program) {
            if let Err(e) = builder.merge(file_builder) {
                diagnostics.push(Diagnostic {
                    range: Range::default(),
                    severity: Some(DiagnosticSeverity::WARNING),
                    source: Some("kleis".to_string()),
                    message: format!("Error merging types from '{}': {}", path.display(), e),
                    ..Default::default()
                });
            }
        }

        diagnostics
    }

    /// Extract import paths from a program
    fn extract_imports(program: &Program) -> Vec<String> {
        program
            .items
            .iter()
            .filter_map(|item| {
                if let TopLevel::Import(path) = item {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Parse a document, resolve imports, and build type context
    fn parse_document(
        &self,
        uri: &Url,
        text: &str,
    ) -> (
        Option<Program>,
        Option<TypeContextBuilder>,
        Vec<String>,
        Vec<Diagnostic>,
    ) {
        let mut all_diagnostics = Vec::new();

        // Parse the main document
        let program = match parse_kleis_program(text) {
            Ok(p) => p,
            Err(e) => {
                let diagnostic = self.error_to_diagnostic(&e, text);
                return (None, None, vec![], vec![diagnostic]);
            }
        };

        // Extract imports
        let imports = Self::extract_imports(&program);

        // Determine base directory
        let base_dir = uri
            .to_file_path()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        // Build type context from imports
        let mut loaded = HashSet::new();
        let mut builder = TypeContextBuilder::new();

        for import_path in &imports {
            let resolved = Self::resolve_import_path(import_path, &base_dir);
            let import_diags = Self::load_import(&resolved, &mut loaded, &mut builder);
            all_diagnostics.extend(import_diags);
        }

        // Build type context from the main document
        if let Ok(main_builder) = TypeContextBuilder::from_program(program.clone()) {
            if let Err(e) = builder.merge(main_builder) {
                all_diagnostics.push(Diagnostic {
                    range: Range::default(),
                    severity: Some(DiagnosticSeverity::WARNING),
                    source: Some("kleis".to_string()),
                    message: format!("Error building type context: {}", e),
                    ..Default::default()
                });
            }
        }

        (Some(program), Some(builder), imports, all_diagnostics)
    }

    /// Convert a parse error to an LSP diagnostic
    fn error_to_diagnostic(&self, error: &KleisParseError, text: &str) -> Diagnostic {
        // Convert byte position to line/column
        let (line, col) = byte_offset_to_position(text, error.position);

        Diagnostic {
            range: Range {
                start: Position {
                    line: line as u32,
                    character: col as u32,
                },
                end: Position {
                    line: line as u32,
                    character: (col + 1) as u32,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("kleis".to_string()),
            message: error.message.clone(),
            related_information: None,
            tags: None,
            data: None,
        }
    }

    /// Publish diagnostics for a document
    async fn publish_diagnostics(&self, uri: Url, text: &str) {
        let (ast, type_context, imports, diagnostics) = self.parse_document(&uri, text);

        // Store the document with type context
        self.documents.insert(
            uri.clone(),
            Document {
                content: Rope::from_str(text),
                ast,
                type_context,
                imports,
            },
        );

        // Send diagnostics to the client
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    /// Build hover content for a word using type context
    fn build_hover_content(
        &self,
        word: &str,
        doc: &dashmap::mapref::one::Ref<Url, Document>,
    ) -> String {
        let mut content = format!("**{}**\n\n", word);

        // Check if we have type context
        if let Some(ref ctx) = doc.type_context {
            // Check if it's a structure
            if let Some(structure) = ctx.get_structure(word) {
                content.push_str(&format!("```kleis\nstructure {}", word));
                if !structure.type_params.is_empty() {
                    let params: Vec<String> = structure
                        .type_params
                        .iter()
                        .map(|p| p.name.clone())
                        .collect();
                    content.push_str(&format!("({})", params.join(", ")));
                }
                content.push_str("\n```\n\n");

                // List operations
                let ops = ctx.operations_for_structure(word);
                if !ops.is_empty() {
                    content.push_str("**Operations:**\n");
                    for op in ops.iter().take(10) {
                        content.push_str(&format!("- `{}`\n", op));
                    }
                    if ops.len() > 10 {
                        content.push_str(&format!("- ... and {} more\n", ops.len() - 10));
                    }
                }
                return content;
            }

            // Check if it's an operation
            if let Some(sig) = ctx.operation_signature(word) {
                content.push_str("**Operation**\n\n");
                content.push_str(&format!("```kleis\n{}\n```\n\n", sig));

                // Show which types support this operation
                let types = ctx.types_supporting(word);
                if !types.is_empty() {
                    content.push_str("**Implemented by:**\n");
                    for ty in types.iter().take(5) {
                        content.push_str(&format!("- `{}`\n", ty));
                    }
                    if types.len() > 5 {
                        content.push_str(&format!("- ... and {} more\n", types.len() - 5));
                    }
                }
                return content;
            }

            // Check if it's a type that supports operations
            if ctx.supports_any_operation(word) {
                content.push_str(&format!("**Type:** `{}`\n\n", word));
                let ops = ctx.operations_for_type(word);
                if !ops.is_empty() {
                    content.push_str("**Supports:**\n");
                    for op in ops.iter().take(10) {
                        content.push_str(&format!("- `{}`\n", op));
                    }
                    if ops.len() > 10 {
                        content.push_str(&format!("- ... and {} more\n", ops.len() - 10));
                    }
                }
                return content;
            }
        }

        // Check if it's a known keyword or builtin
        if let Some(desc) = get_builtin_description(word) {
            content.push_str(desc);
            return content;
        }

        // Fallback: show imports if available
        if !doc.imports.is_empty() {
            content.push_str("\n_Imported:_\n");
            for import in &doc.imports {
                content.push_str(&format!("- `{}`\n", import));
            }
        }

        content
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for KleisLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // Full document sync - we get the entire document on each change
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                // Hover support
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                // Go to definition
                definition_provider: Some(OneOf::Left(true)),
                // Document symbols (outline)
                document_symbol_provider: Some(OneOf::Left(true)),
                // Completion
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "kleis-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Kleis language server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.publish_diagnostics(uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        // With FULL sync, we get the entire document content
        if let Some(change) = params.content_changes.into_iter().next() {
            self.publish_diagnostics(uri, &change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        // Remove document from cache and clear diagnostics
        self.documents.remove(&params.text_document.uri);
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get the document
        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Get the word at the cursor position
        let line_idx = position.line as usize;
        let col_idx = position.character as usize;

        let line = match doc.content.get_line(line_idx) {
            Some(line) => line.to_string(),
            None => return Ok(None),
        };

        // Extract word at position
        let word = extract_word_at(&line, col_idx);
        if word.is_empty() {
            return Ok(None);
        }

        // Build hover content from type context
        let hover_content = self.build_hover_content(&word, &doc);

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: hover_content,
            }),
            range: None,
        }))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get the document
        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Get the word at the cursor position
        let line_idx = position.line as usize;
        let col_idx = position.character as usize;

        let line = match doc.content.get_line(line_idx) {
            Some(line) => line.to_string(),
            None => return Ok(None),
        };

        let word = extract_word_at(&line, col_idx);
        if word.is_empty() {
            return Ok(None);
        }

        // Search for definition in the AST
        if let Some(ref ast) = doc.ast {
            for item in &ast.items {
                use kleis::kleis_ast::TopLevel;
                match item {
                    TopLevel::FunctionDef(def) => {
                        if def.name == word {
                            // Found the definition - return its location
                            // TODO: Store source positions in AST for accurate locations
                            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                                uri: uri.clone(),
                                range: Range {
                                    start: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                    end: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                },
                            })));
                        }
                    }
                    TopLevel::StructureDef(s) => {
                        if s.name == word {
                            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                                uri: uri.clone(),
                                range: Range {
                                    start: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                    end: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                },
                            })));
                        }
                    }
                    TopLevel::DataDef(d) => {
                        if d.name == word {
                            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                                uri: uri.clone(),
                                range: Range {
                                    start: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                    end: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                },
                            })));
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let mut completions = get_kleis_completions();

        // Add context-aware completions from imports
        let uri = &params.text_document_position.text_document.uri;
        if let Some(doc) = self.documents.get(uri) {
            if let Some(ref ctx) = doc.type_context {
                // Add structures as type completions
                for structure_name in ctx.all_structure_names() {
                    completions.push(CompletionItem {
                        label: structure_name.clone(),
                        kind: Some(CompletionItemKind::CLASS),
                        detail: Some("(structure from import)".to_string()),
                        documentation: Some(Documentation::String(format!(
                            "Structure `{}` from imported files",
                            structure_name
                        ))),
                        ..Default::default()
                    });
                }

                // Add operations
                for op_name in ctx.all_operation_names() {
                    // Skip if already in static completions
                    if completions.iter().any(|c| c.label == op_name) {
                        continue;
                    }

                    let detail = if let Some(sig) = ctx.operation_signature(&op_name) {
                        sig
                    } else {
                        "(operation from import)".to_string()
                    };

                    completions.push(CompletionItem {
                        label: op_name.clone(),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some(detail),
                        ..Default::default()
                    });
                }
            }
        }

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        // Get the document
        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let mut symbols = Vec::new();

        if let Some(ref ast) = doc.ast {
            use kleis::kleis_ast::TopLevel;
            for item in &ast.items {
                match item {
                    TopLevel::FunctionDef(def) => {
                        symbols.push(SymbolInformation {
                            name: def.name.clone(),
                            kind: SymbolKind::FUNCTION,
                            tags: None,
                            deprecated: None,
                            location: Location {
                                uri: uri.clone(),
                                range: Range::default(),
                            },
                            container_name: None,
                        });
                    }
                    TopLevel::StructureDef(s) => {
                        symbols.push(SymbolInformation {
                            name: s.name.clone(),
                            kind: SymbolKind::STRUCT,
                            tags: None,
                            deprecated: None,
                            location: Location {
                                uri: uri.clone(),
                                range: Range::default(),
                            },
                            container_name: None,
                        });
                    }
                    TopLevel::DataDef(d) => {
                        symbols.push(SymbolInformation {
                            name: d.name.clone(),
                            kind: SymbolKind::ENUM,
                            tags: None,
                            deprecated: None,
                            location: Location {
                                uri: uri.clone(),
                                range: Range::default(),
                            },
                            container_name: None,
                        });
                    }
                    TopLevel::ImplementsDef(i) => {
                        symbols.push(SymbolInformation {
                            name: format!("implements {}", i.structure_name),
                            kind: SymbolKind::CLASS,
                            tags: None,
                            deprecated: None,
                            location: Location {
                                uri: uri.clone(),
                                range: Range::default(),
                            },
                            container_name: None,
                        });
                    }
                    TopLevel::TypeAlias(t) => {
                        symbols.push(SymbolInformation {
                            name: t.name.clone(),
                            kind: SymbolKind::TYPE_PARAMETER,
                            tags: None,
                            deprecated: None,
                            location: Location {
                                uri: uri.clone(),
                                range: Range::default(),
                            },
                            container_name: None,
                        });
                    }
                    _ => {}
                }
            }
        }

        #[allow(deprecated)]
        Ok(Some(DocumentSymbolResponse::Flat(symbols)))
    }
}

/// Convert a byte offset to (line, column)
fn byte_offset_to_position(text: &str, offset: usize) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;
    let mut current_offset = 0;

    for ch in text.chars() {
        if current_offset >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        current_offset += ch.len_utf8();
    }

    (line, col)
}

/// Extract the word at a given column position in a line
fn extract_word_at(line: &str, col: usize) -> String {
    let chars: Vec<char> = line.chars().collect();
    if col >= chars.len() {
        return String::new();
    }

    // Find word boundaries
    let is_word_char = |c: char| c.is_alphanumeric() || c == '_' || c == '\'';

    let mut start = col;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }

    let mut end = col;
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }

    chars[start..end].iter().collect()
}

/// Get description for builtin keywords and types
fn get_builtin_description(word: &str) -> Option<&'static str> {
    match word {
        // Types
        "ℝ" | "Real" => Some("**ℝ** (Real numbers)\n\nThe field of real numbers. Supports:\n- Arithmetic: `+`, `-`, `*`, `/`\n- Comparisons: `<`, `>`, `≤`, `≥`\n- Functions: `abs`, `sqrt`, `exp`, `ln`, `sin`, `cos`"),
        "ℂ" | "Complex" => Some("**ℂ** (Complex numbers)\n\nThe field of complex numbers. Supports:\n- Arithmetic: `+`, `-`, `*`, `/`\n- Operations: `re`, `im`, `conj`, `abs`\n- Constructor: `complex(re, im)`"),
        "ℤ" | "Int" | "Integer" => Some("**ℤ** (Integers)\n\nThe ring of integers. Supports:\n- Arithmetic: `+`, `-`, `*`\n- Division: `div`, `mod`\n- Comparisons: `<`, `>`, `≤`, `≥`"),
        "ℕ" | "Nat" | "Natural" => Some("**ℕ** (Natural numbers)\n\nNon-negative integers 0, 1, 2, ...\n- Closed under `+` and `*`\n- Not closed under `-`"),
        "ℚ" | "Rational" => Some("**ℚ** (Rational numbers)\n\nFractions p/q where p, q ∈ ℤ, q ≠ 0"),
        "𝔹" | "Bool" | "Boolean" => Some("**𝔹** (Boolean)\n\nLogical values: `true`, `false`\n- Operations: `∧`, `∨`, `¬`"),
        "Matrix" => Some("**Matrix(m, n, T)**\n\nParametric matrix type with dimensions m×n and element type T.\n\nOperations:\n- `transpose : Matrix(n, m, T)`\n- `det : T` (square matrices)\n- `trace : T` (square matrices)\n- `eigenvalues` (numerical)"),
        "Vector" => Some("**Vector(n, T)**\n\nColumn vector of dimension n with element type T.\n\nEquivalent to `Matrix(n, 1, T)`"),
        "Tensor" => Some("**Tensor(dims, T)**\n\nMulti-dimensional array with given dimensions and element type."),

        // Keywords
        "structure" => Some("**structure**\n\nDefines an algebraic structure with parameters, operations, and axioms.\n\n```kleis\nstructure Group(G) {\n    operation mul : G × G → G\n    element identity : G\n    axiom assoc: ∀(a b c : G). mul(mul(a,b),c) = mul(a,mul(b,c))\n}\n```"),
        "implements" => Some("**implements**\n\nProvides concrete implementations for a structure.\n\n```kleis\nimplements Group(ℤ) {\n    operation mul = builtin_add\n    element identity = 0\n}\n```"),
        "data" => Some("**data**\n\nDefines an algebraic data type (ADT).\n\n```kleis\ndata Option(T) {\n    constructor None\n    constructor Some(T)\n}\n```"),
        "define" => Some("**define**\n\nDefines a named expression or function.\n\n```kleis\ndefine square(x : ℝ) : ℝ = x * x\n```"),
        "axiom" => Some("**axiom**\n\nDeclares a mathematical axiom (assumed true).\n\n```kleis\naxiom commutativity: ∀(x y : G). mul(x, y) = mul(y, x)\n```"),
        "operation" => Some("**operation**\n\nDeclares an operation within a structure.\n\n```kleis\noperation add : T × T → T\n```"),
        "import" => Some("**import**\n\nImports definitions from another Kleis file.\n\n```kleis\nimport \"stdlib/matrices.kleis\"\n```"),

        // Quantifiers
        "∀" | "forall" => Some("**∀** (Universal quantifier)\n\nFor all values of a variable.\n\n```kleis\n∀(x : ℝ). x + 0 = x\n```"),
        "∃" | "exists" => Some("**∃** (Existential quantifier)\n\nThere exists a value.\n\n```kleis\n∃(x : ℝ). x * x = 2\n```"),
        "λ" | "lambda" => Some("**λ** (Lambda)\n\nAnonymous function.\n\n```kleis\nλ(x : ℝ). x * x\n```"),

        // Common operations
        "transpose" => Some("**transpose**\n\nMatrix transpose operation.\n\n`transpose : Matrix(m, n, T) → Matrix(n, m, T)`"),
        "det" => Some("**det**\n\nMatrix determinant (for square matrices).\n\n`det : Matrix(n, n, T) → T`"),
        "trace" => Some("**trace**\n\nMatrix trace (sum of diagonal elements).\n\n`trace : Matrix(n, n, T) → T`"),
        "inv" => Some("**inv**\n\nMatrix inverse (for invertible square matrices).\n\n`inv : Matrix(n, n, T) → Matrix(n, n, T)`"),
        "eigenvalues" => Some("**eigenvalues**\n\nCompute eigenvalues of a square matrix.\n\nReturns a list of eigenvalues (may be complex)."),
        "svd" => Some("**svd**\n\nSingular Value Decomposition.\n\nDecomposes A = U Σ Vᵀ"),

        _ => None,
    }
}

/// Generate all Kleis completions - keywords, types, operators, snippets
#[allow(clippy::vec_init_then_push)]
fn get_kleis_completions() -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // ═══════════════════════════════════════════════════════════════════════
    // KEYWORDS - Core language constructs
    // ═══════════════════════════════════════════════════════════════════════

    items.push(keyword_completion(
        "structure",
        "Define an algebraic structure",
        "structure ${1:Name}(${2:T}) {\n    $0\n}",
        "Structures define mathematical objects with operations and axioms.\n\n\
         Example:\n```kleis\nstructure Group(G) {\n    element identity : G\n    operation mul : G × G → G\n    axiom associativity: ∀(a b c : G). mul(mul(a,b),c) = mul(a,mul(b,c))\n}\n```",
    ));

    items.push(keyword_completion(
        "implements",
        "Implement a structure for a concrete type",
        "implements ${1:Structure}(${2:Type}) {\n    $0\n}",
        "Provides concrete implementations for structure operations.\n\n\
         Example:\n```kleis\nimplements Group(ℤ) {\n    element identity = 0\n    operation mul = builtin_add\n}\n```",
    ));

    items.push(keyword_completion(
        "operation",
        "Declare an operation signature",
        "operation ${1:name} : ${2:Type} → ${3:Type}",
        "Operations are the functions within a structure.\n\n\
         Example:\n```kleis\noperation inverse : G → G\noperation det : Matrix(n, n, ℝ) → ℝ\n```",
    ));

    items.push(keyword_completion(
        "axiom",
        "Declare an axiom (mathematical law)",
        "axiom ${1:name}: ${2:∀(x : T). condition}",
        "Axioms are the mathematical laws that must hold.\n\n\
         Example:\n```kleis\naxiom commutativity: ∀(a b : G). mul(a, b) = mul(b, a)\naxiom identity_left: ∀(a : G). mul(identity, a) = a\n```",
    ));

    items.push(keyword_completion(
        "element",
        "Declare a distinguished element",
        "element ${1:name} : ${2:Type}",
        "Elements are constants within a structure.\n\n\
         Example:\n```kleis\nelement zero : R\nelement one : R\nelement identity : G\n```",
    ));

    items.push(keyword_completion(
        "data",
        "Define an algebraic data type",
        "data ${1:Name} = ${2:Variant1} | ${3:Variant2}",
        "Sum types with multiple constructors.\n\n\
         Example:\n```kleis\ndata Option(T) = Some(T) | None\ndata List(T) = Nil | Cons(T, List(T))\n```",
    ));

    items.push(keyword_completion(
        "define",
        "Define a function or value",
        "define ${1:name}(${2:args}) = ${3:expr}",
        "Top-level function definitions.\n\n\
         Example:\n```kleis\ndefine square(x) = x * x\ndefine factorial(n) = if n = 0 then 1 else n * factorial(n - 1)\n```",
    ));

    items.push(keyword_completion(
        "type",
        "Define a type alias",
        "type ${1:Name} = ${2:Type}",
        "Creates an alias for a type expression.\n\n\
         Example:\n```kleis\ntype ComplexMatrix(n, m) = (Matrix(n, m, ℝ), Matrix(n, m, ℝ))\ntype Point = (ℝ, ℝ, ℝ)\n```",
    ));

    items.push(keyword_completion(
        "import",
        "Import definitions from another file",
        "import \"${1:path/to/file.kleis}\"",
        "Imports all definitions from the specified file.\n\n\
         Example:\n```kleis\nimport \"stdlib/matrices.kleis\"\nimport \"physics/relativity.kleis\"\n```",
    ));

    items.push(keyword_completion(
        "extends",
        "Inherit from another structure",
        "extends ${1:ParentStructure}(${2:T})",
        "Structure inheritance - includes all parent operations and axioms.\n\n\
         Example:\n```kleis\nstructure Ring(R) extends Group(R) {\n    operation mul : R × R → R\n}\n```",
    ));

    items.push(keyword_completion(
        "over",
        "Parameterize structure over a field",
        "over ${1:Field}(${2:F})",
        "Used for structures like vector spaces that are parameterized over fields.\n\n\
         Example:\n```kleis\nstructure VectorSpace(V) over Field(F) {\n    operation scale : F × V → V\n}\n```",
    ));

    // ═══════════════════════════════════════════════════════════════════════
    // QUANTIFIERS - Logical operators
    // ═══════════════════════════════════════════════════════════════════════

    items.push(CompletionItem {
        label: "∀".to_string(),
        label_details: Some(CompletionItemLabelDetails {
            detail: Some(" forall".to_string()),
            description: Some("Universal quantifier".to_string()),
        }),
        kind: Some(CompletionItemKind::OPERATOR),
        detail: Some("Universal quantifier (for all)".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value:
                "**Universal quantification**: asserts a property holds for all values.\n\n\
                    Type `forall` for ASCII alternative.\n\n\
                    Example:\n```kleis\n∀(x : ℝ). x + 0 = x\n∀(a b : G). mul(a, b) = mul(b, a)\n```"
                    .to_string(),
        })),
        insert_text: Some("∀(${1:x} : ${2:T}). ${0}".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "∃".to_string(),
        label_details: Some(CompletionItemLabelDetails {
            detail: Some(" exists".to_string()),
            description: Some("Existential quantifier".to_string()),
        }),
        kind: Some(CompletionItemKind::OPERATOR),
        detail: Some("Existential quantifier (there exists)".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Existential quantification**: asserts at least one value satisfies a property.\n\n\
                    Type `exists` for ASCII alternative.\n\n\
                    Example:\n```kleis\n∃(x : ℝ). x * x = 2\n∃(inv : G). mul(a, inv) = identity\n```".to_string(),
        })),
        insert_text: Some("∃(${1:x} : ${2:T}). ${0}".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "λ".to_string(),
        label_details: Some(CompletionItemLabelDetails {
            detail: Some(" lambda".to_string()),
            description: Some("Lambda abstraction".to_string()),
        }),
        kind: Some(CompletionItemKind::OPERATOR),
        detail: Some("Lambda (anonymous function)".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Lambda abstraction**: creates an anonymous function.\n\n\
                    Type `lambda` for ASCII alternative.\n\n\
                    Example:\n```kleis\nλ(x : ℝ). x * x\nλ(f : ℝ → ℝ). f(0)\n```"
                .to_string(),
        })),
        insert_text: Some("λ(${1:x} : ${2:T}). ${0}".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });

    // ═══════════════════════════════════════════════════════════════════════
    // TYPES - Primitive and parametric types
    // ═══════════════════════════════════════════════════════════════════════

    items.push(type_completion(
        "ℝ",
        "Real",
        "Real numbers (ℝ)",
        "The field of real numbers. Supports all arithmetic operations.",
    ));
    items.push(type_completion(
        "ℂ",
        "Complex",
        "Complex numbers (ℂ)",
        "The field of complex numbers. Use `complex(a, b)` for `a + bi`.",
    ));
    items.push(type_completion(
        "ℤ",
        "Integer",
        "Integers (ℤ)",
        "The ring of integers: ..., -2, -1, 0, 1, 2, ...",
    ));
    items.push(type_completion(
        "ℕ",
        "Nat",
        "Natural numbers (ℕ)",
        "Non-negative integers: 0, 1, 2, 3, ...",
    ));
    items.push(type_completion(
        "ℚ",
        "Rational",
        "Rational numbers (ℚ)",
        "Fractions p/q where p, q ∈ ℤ and q ≠ 0.",
    ));
    items.push(type_completion(
        "𝔹",
        "Bool",
        "Boolean (𝔹)",
        "Truth values: True or False.",
    ));

    items.push(CompletionItem {
        label: "Matrix".to_string(),
        kind: Some(CompletionItemKind::CLASS),
        detail: Some("Matrix(m, n, T) - m×n matrix over type T".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Matrices** with dimensions and element type.\n\n\
                    Example:\n```kleis\nMatrix(3, 3, ℝ)    // 3×3 real matrix\nMatrix(n, n, ℂ)    // n×n complex matrix\nMatrix(2*n, 2*n, ℝ) // dimension expressions\n```".to_string(),
        })),
        insert_text: Some("Matrix(${1:m}, ${2:n}, ${3:ℝ})".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "Vector".to_string(),
        kind: Some(CompletionItemKind::CLASS),
        detail: Some("Vector(n, T) - n-dimensional vector".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Vectors** with dimension and element type.\n\n\
                    Example:\n```kleis\nVector(3, ℝ)  // 3D real vector\nVector(n, ℂ)  // n-dimensional complex vector\n```".to_string(),
        })),
        insert_text: Some("Vector(${1:n}, ${2:ℝ})".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });

    // ═══════════════════════════════════════════════════════════════════════
    // GREEK LETTERS - Common mathematical variables
    // ═══════════════════════════════════════════════════════════════════════

    let greek_letters = [
        (
            "α",
            "alpha",
            "Commonly used for angles, coefficients, significance level",
        ),
        (
            "β",
            "beta",
            "Commonly used for angles, coefficients, beta functions",
        ),
        (
            "γ",
            "gamma",
            "Lorentz factor, Euler-Mascheroni constant, photon",
        ),
        ("δ", "delta", "Small change, Kronecker delta, Dirac delta"),
        (
            "ε",
            "epsilon",
            "Small positive quantity, permittivity, Levi-Civita",
        ),
        ("ζ", "zeta", "Riemann zeta function, damping ratio"),
        ("η", "eta", "Efficiency, metric tensor, learning rate"),
        ("θ", "theta", "Angle, phase, polar coordinate"),
        ("κ", "kappa", "Curvature, condition number, connectivity"),
        ("λ", "lambda", "Eigenvalue, wavelength, decay constant"),
        ("μ", "mu", "Mean, permeability, chemical potential, index"),
        ("ν", "nu", "Frequency, kinematic viscosity, index"),
        ("ξ", "xi", "Random variable, coordinate"),
        ("π", "pi", "Circle constant ≈ 3.14159..."),
        ("ρ", "rho", "Density, correlation, radius"),
        ("σ", "sigma", "Standard deviation, stress, sum"),
        ("τ", "tau", "Proper time, torque, time constant"),
        ("φ", "phi", "Angle, golden ratio, potential"),
        ("ψ", "psi", "Wave function, angle, digamma"),
        ("ω", "omega", "Angular frequency, solid angle"),
        ("Γ", "Gamma", "Gamma function, Christoffel symbol"),
        ("Δ", "Delta", "Difference, Laplacian, discriminant"),
        ("Θ", "Theta", "Heaviside function, big-O notation"),
        ("Λ", "Lambda", "Cosmological constant, diagonal matrix"),
        ("Σ", "Sigma", "Summation, covariance matrix"),
        ("Φ", "Phi", "Cumulative distribution, golden ratio"),
        ("Ψ", "Psi", "Wave function, digamma"),
        ("Ω", "Omega", "Ohm, sample space, solid angle"),
    ];

    for (symbol, name, description) in greek_letters {
        items.push(CompletionItem {
            label: symbol.to_string(),
            label_details: Some(CompletionItemLabelDetails {
                detail: Some(format!(" {}", name)),
                description: None,
            }),
            kind: Some(CompletionItemKind::CONSTANT),
            detail: Some(description.to_string()),
            insert_text: Some(symbol.to_string()),
            filter_text: Some(format!("{} {}", symbol, name)),
            ..Default::default()
        });
    }

    // ═══════════════════════════════════════════════════════════════════════
    // OPERATORS - Mathematical operators
    // ═══════════════════════════════════════════════════════════════════════

    let operators = [
        ("→", "arrow", "Function type: A → B"),
        ("⇒", "implies", "Logical implication: P ⇒ Q"),
        ("×", "times", "Product type or multiplication"),
        ("⊗", "tensor", "Tensor product"),
        ("∘", "compose", "Function composition"),
        ("∇", "nabla", "Gradient/del operator"),
        ("∂", "partial", "Partial derivative"),
        ("∫", "integral", "Integration"),
        ("∑", "sum", "Summation"),
        ("∏", "product", "Product"),
        ("√", "sqrt", "Square root"),
        ("∞", "infinity", "Infinity"),
        ("≠", "neq", "Not equal"),
        ("≤", "leq", "Less than or equal"),
        ("≥", "geq", "Greater than or equal"),
        ("≈", "approx", "Approximately equal"),
        ("≡", "equiv", "Equivalent/congruent"),
        ("∈", "in", "Element of (set membership)"),
        ("∉", "notin", "Not element of"),
        ("⊂", "subset", "Proper subset"),
        ("⊆", "subseteq", "Subset or equal"),
        ("∧", "and", "Logical AND"),
        ("∨", "or", "Logical OR"),
        ("¬", "not", "Logical NOT"),
        ("†", "dagger", "Hermitian conjugate (adjoint)"),
        ("ᵀ", "transpose", "Matrix transpose"),
    ];

    for (symbol, name, description) in operators {
        items.push(CompletionItem {
            label: symbol.to_string(),
            label_details: Some(CompletionItemLabelDetails {
                detail: Some(format!(" {}", name)),
                description: None,
            }),
            kind: Some(CompletionItemKind::OPERATOR),
            detail: Some(description.to_string()),
            insert_text: Some(symbol.to_string()),
            filter_text: Some(format!("{} {}", symbol, name)),
            ..Default::default()
        });
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DIMENSION FUNCTIONS - For type-level arithmetic
    // ═══════════════════════════════════════════════════════════════════════

    items.push(CompletionItem {
        label: "min".to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        detail: Some("min(a, b) - Minimum of dimensions".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "Dimension function: minimum of two dimensions.\n\n\
                    Example:\n```kleis\nMatrix(min(m,n), min(m,n), ℝ)  // Square submatrix\n```"
                .to_string(),
        })),
        insert_text: Some("min(${1:a}, ${2:b})".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "max".to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        detail: Some("max(a, b) - Maximum of dimensions".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "Dimension function: maximum of two dimensions.\n\n\
                    Example:\n```kleis\nVector(max(m,n), ℝ)  // Larger dimension\n```"
                .to_string(),
        })),
        insert_text: Some("max(${1:a}, ${2:b})".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "gcd".to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        detail: Some("gcd(a, b) - Greatest common divisor".to_string()),
        insert_text: Some("gcd(${1:a}, ${2:b})".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "lcm".to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        detail: Some("lcm(a, b) - Least common multiple".to_string()),
        insert_text: Some("lcm(${1:a}, ${2:b})".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });

    // ═══════════════════════════════════════════════════════════════════════
    // BUILTIN FUNCTIONS - Numerical operations
    // ═══════════════════════════════════════════════════════════════════════

    let builtins = [
        ("builtin_add", "Addition"),
        ("builtin_sub", "Subtraction"),
        ("builtin_mul", "Multiplication"),
        ("builtin_div", "Division"),
        ("builtin_negate", "Negation"),
        ("builtin_abs", "Absolute value"),
        ("builtin_sqrt", "Square root"),
        ("builtin_exp", "Exponential (e^x)"),
        ("builtin_log", "Natural logarithm"),
        ("builtin_sin", "Sine"),
        ("builtin_cos", "Cosine"),
        ("builtin_tan", "Tangent"),
    ];

    for (name, description) in builtins {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(format!("Builtin: {}", description)),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**{}**\n\nBuilt-in numerical operation for `{}`.\n\nUsed in `implements` blocks to provide concrete implementations.", description, description.to_lowercase()),
            })),
            ..Default::default()
        });
    }

    // Matrix builtins
    let matrix_builtins = [
        ("matrix_add", "Matrix addition"),
        ("matrix_mul", "Matrix multiplication"),
        ("matrix_transpose", "Matrix transpose"),
        ("matrix_det", "Matrix determinant"),
        ("matrix_trace", "Matrix trace"),
        ("eigenvalues", "Compute eigenvalues (LAPACK)"),
        ("svd", "Singular value decomposition (LAPACK)"),
        ("solve", "Solve linear system Ax = b (LAPACK)"),
        ("inv", "Matrix inverse (LAPACK)"),
        ("qr", "QR decomposition (LAPACK)"),
        ("cholesky", "Cholesky decomposition (LAPACK)"),
        ("schur", "Schur decomposition (LAPACK)"),
        ("expm", "Matrix exponential"),
        ("eye", "Identity matrix"),
        ("zeros", "Zero matrix"),
        ("ones", "Matrix of ones"),
    ];

    for (name, description) in matrix_builtins {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(description.to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "**{}**\n\nMatrix operation available in `:eval` context.",
                    description
                ),
            })),
            ..Default::default()
        });
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SNIPPETS - Common patterns
    // ═══════════════════════════════════════════════════════════════════════

    items.push(CompletionItem {
        label: "structure (full)".to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some("Complete structure template".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "Creates a complete structure with element, operation, and axiom.".to_string(),
        })),
        insert_text: Some(
            "structure ${1:Name}(${2:T}) {\n    \
             element ${3:identity} : ${2:T}\n    \
             operation ${4:op} : ${2:T} × ${2:T} → ${2:T}\n    \
             axiom ${5:law}: ∀(a b : ${2:T}). ${0}\n\
             }"
            .to_string(),
        ),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "implements (full)".to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some("Complete implements template".to_string()),
        insert_text: Some(
            "implements ${1:Structure}(${2:ℝ}) {\n    \
             element ${3:identity} = ${4:0}\n    \
             operation ${5:op} = ${6:builtin_add}\n\
             }"
            .to_string(),
        ),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "Monoid".to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some("Monoid structure template".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Monoid**: A set with an associative binary operation and identity element."
                .to_string(),
        })),
        insert_text: Some(
            "structure Monoid(M) {\n    \
             element identity : M\n    \
             operation op : M × M → M\n    \
             axiom associativity: ∀(a b c : M). op(op(a, b), c) = op(a, op(b, c))\n    \
             axiom left_identity: ∀(a : M). op(identity, a) = a\n    \
             axiom right_identity: ∀(a : M). op(a, identity) = a\n\
             }"
            .to_string(),
        ),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "Group".to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some("Group structure template".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Group**: A monoid where every element has an inverse.".to_string(),
        })),
        insert_text: Some(
            "structure Group(G) extends Monoid(G) {\n    \
             operation inverse : G → G\n    \
             axiom left_inverse: ∀(a : G). op(inverse(a), a) = identity\n    \
             axiom right_inverse: ∀(a : G). op(a, inverse(a)) = identity\n\
             }"
            .to_string(),
        ),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });

    // ═══════════════════════════════════════════════════════════════════════
    // STDLIB: ALGEBRAIC STRUCTURES (from prelude.kleis)
    // ═══════════════════════════════════════════════════════════════════════

    items.push(stdlib_completion(
        "Semigroup",
        "Semigroup(S) - Associative binary operation",
        "A set S with an associative binary operation (•).",
    ));

    items.push(stdlib_completion(
        "AbelianGroup",
        "AbelianGroup(A) - Commutative group",
        "A group where the operation is commutative: ∀(x y). x • y = y • x",
    ));

    items.push(stdlib_completion(
        "Ring",
        "Ring(R) - Two operations with distributivity",
        "Addition (abelian group) + multiplication (monoid) with distributivity.\nExamples: ℤ, polynomials, matrices",
    ));

    items.push(stdlib_completion(
        "Field",
        "Field(F) - Ring with multiplicative inverses",
        "Every non-zero element has a multiplicative inverse.\nExamples: ℝ, ℂ, ℚ",
    ));

    items.push(stdlib_completion(
        "VectorSpace",
        "VectorSpace(V) over Field(F)",
        "Module over a field with scalar multiplication.",
    ));

    // ═══════════════════════════════════════════════════════════════════════
    // STDLIB: TYPE PROMOTION (from prelude.kleis)
    // ═══════════════════════════════════════════════════════════════════════

    items.push(stdlib_completion(
        "Promotes",
        "Promotes(From, To) - Type promotion/lifting",
        "Lifting values from smaller to larger types.\n\nHierarchy: ℕ → ℤ → ℚ → ℝ → ℂ\n\nUse `lift` operation to promote values.",
    ));

    items.push(CompletionItem {
        label: "lift".to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        detail: Some("Promote value to larger type".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**lift** - From `Promotes(From, To)` structure.\n\nPromotes a value to a larger type in the hierarchy ℕ → ℤ → ℚ → ℝ → ℂ".to_string(),
        })),
        ..Default::default()
    });

    // ═══════════════════════════════════════════════════════════════════════
    // STDLIB: COMPLEX MATRICES (from matrices.kleis)
    // ═══════════════════════════════════════════════════════════════════════

    items.push(stdlib_completion(
        "ComplexMatrix",
        "ComplexMatrix(m, n) - Complex matrix as (Re, Im)",
        "type ComplexMatrix(m, n) = (Matrix(m, n, ℝ), Matrix(m, n, ℝ))\n\nA complex matrix M = A + B·i stored as (A, B).\nEnables real LAPACK routines for complex computations.",
    ));

    let cmat_ops = [
        ("cmat_add", "Add complex matrices"),
        ("cmat_sub", "Subtract complex matrices"),
        ("cmat_mul", "Multiply complex matrices"),
        ("cmat_conj", "Element-wise conjugate"),
        ("cmat_transpose", "Transpose complex matrix"),
        ("cmat_dagger", "Conjugate transpose (A†)"),
        ("cmat_trace", "Trace of square complex matrix"),
        ("cmat_eye", "Complex identity matrix"),
        ("cmat_zero", "Complex zero matrix"),
        ("cmat_eigenvalues", "Eigenvalues of complex matrix"),
        ("cmat_schur", "Schur decomposition"),
        ("cmat_expm", "Complex matrix exponential"),
    ];

    for (name, desc) in cmat_ops {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(desc.to_string()),
            ..Default::default()
        });
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STDLIB: REALIFICATION / COMPLEXIFICATION
    // ═══════════════════════════════════════════════════════════════════════

    items.push(CompletionItem {
        label: "realify".to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        detail: Some("Embed complex n×n into real 2n×2n".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Realification**: `realify((A,B)) = [[A,-B],[B,A]]`\n\nEmbed complex matrix into real block matrix for LAPACK.".to_string(),
        })),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "complexify".to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        detail: Some("Extract complex n×n from real 2n×2n".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Complexification**: `complexify([[A,-B],[B,A]]) = (A,B)`\n\nExtract complex matrix from structured real block matrix.\n\n**Precondition**: Must have [[A,-B],[B,A]] structure.".to_string(),
        })),
        ..Default::default()
    });

    // ═══════════════════════════════════════════════════════════════════════
    // STDLIB: COMPLEX NUMBERS
    // ═══════════════════════════════════════════════════════════════════════

    items.push(CompletionItem {
        label: "complex".to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        detail: Some("Create complex: complex(re, im)".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Complex Constructor**: `complex(3, 4)` creates 3 + 4i".to_string(),
        })),
        insert_text: Some("complex(${1:re}, ${2:im})".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });

    let complex_ops = [
        ("re", "Extract real part"),
        ("im", "Extract imaginary part"),
        ("conj", "Complex conjugate"),
        ("complex_add", "Add complex numbers"),
        ("complex_mul", "Multiply complex numbers"),
        ("abs_squared", "Magnitude squared |z|²"),
    ];

    for (name, desc) in complex_ops {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(desc.to_string()),
            ..Default::default()
        });
    }

    items.push(CompletionItem {
        label: "i".to_string(),
        kind: Some(CompletionItemKind::CONSTANT),
        detail: Some("Imaginary unit: i² = -1".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Imaginary unit** where i² = -1.\n\n⚠️ Avoid `i` as loop variable - use `k`, `j`, `n`, `m` instead.".to_string(),
        })),
        ..Default::default()
    });

    // ═══════════════════════════════════════════════════════════════════════
    // STDLIB: CALCULUS OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════

    items.push(CompletionItem {
        label: "gradient".to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        detail: Some("∇f - Gradient of scalar field".to_string()),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "divergence".to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        detail: Some("∇·F - Divergence of vector field".to_string()),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "curl".to_string(),
        kind: Some(CompletionItemKind::FUNCTION),
        detail: Some("∇×F - Curl (3D only)".to_string()),
        ..Default::default()
    });

    // ═══════════════════════════════════════════════════════════════════════
    // STDLIB: TRIGONOMETRIC & TRANSCENDENTAL
    // ═══════════════════════════════════════════════════════════════════════

    let math_funcs = [
        ("sin", "Sine"),
        ("cos", "Cosine"),
        ("tan", "Tangent"),
        ("exp", "Exponential e^x"),
        ("ln", "Natural log"),
        ("sqrt", "Square root"),
        ("abs", "Absolute value"),
        ("floor", "Floor function"),
    ];

    for (name, desc) in math_funcs {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(desc.to_string()),
            ..Default::default()
        });
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STDLIB: CONSTANTS
    // ═══════════════════════════════════════════════════════════════════════

    items.push(CompletionItem {
        label: "pi".to_string(),
        kind: Some(CompletionItemKind::CONSTANT),
        detail: Some("π ≈ 3.14159...".to_string()),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "e".to_string(),
        kind: Some(CompletionItemKind::CONSTANT),
        detail: Some("Euler's number ≈ 2.71828...".to_string()),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "phi".to_string(),
        kind: Some(CompletionItemKind::CONSTANT),
        detail: Some("Golden ratio φ ≈ 1.61803...".to_string()),
        ..Default::default()
    });

    // ═══════════════════════════════════════════════════════════════════════
    // MATRIX MANIPULATION (from manual chapter 19)
    // ═══════════════════════════════════════════════════════════════════════

    let matrix_manip = [
        ("matrix_get", "Get element at (i, j)"),
        ("matrix_row", "Extract row as list"),
        ("matrix_col", "Extract column as list"),
        ("matrix_diag", "Extract diagonal"),
        ("vstack", "Stack matrices vertically"),
        ("hstack", "Stack matrices horizontally"),
        ("append_row", "Append row to matrix"),
        ("prepend_row", "Prepend row to matrix"),
        ("append_col", "Append column"),
        ("prepend_col", "Prepend column"),
        ("set_element", "Set element at (i,j)"),
        ("set_row", "Replace row"),
        ("set_col", "Replace column"),
        ("set_diag", "Replace diagonal"),
        ("size", "Get (rows, cols) tuple"),
        ("nrows", "Get number of rows"),
        ("ncols", "Get number of columns"),
        ("diag_matrix", "Create diagonal matrix"),
        ("scalar_matrix_mul", "Scalar × matrix"),
    ];

    for (name, desc) in matrix_manip {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(desc.to_string()),
            ..Default::default()
        });
    }

    // ═══════════════════════════════════════════════════════════════════════
    // BIT-VECTOR OPERATIONS (from manual chapter 16)
    // ═══════════════════════════════════════════════════════════════════════

    let bitvec_ops = [
        // Bitwise
        ("bvand", "Bitwise AND"),
        ("bvor", "Bitwise OR"),
        ("bvxor", "Bitwise XOR"),
        ("bvnot", "Bitwise complement"),
        // Arithmetic
        ("bvadd", "Addition mod 2ⁿ"),
        ("bvsub", "Subtraction mod 2ⁿ"),
        ("bvmul", "Multiplication mod 2ⁿ"),
        ("bvneg", "Two's complement negation"),
        ("bvudiv", "Unsigned division"),
        ("bvsdiv", "Signed division"),
        ("bvurem", "Unsigned remainder"),
        // Shift
        ("bvshl", "Shift left"),
        ("bvlshr", "Logical shift right"),
        ("bvashr", "Arithmetic shift right"),
        // Comparison
        ("bvult", "Unsigned less than"),
        ("bvule", "Unsigned less or equal"),
        ("bvslt", "Signed less than"),
        ("bvsle", "Signed less or equal"),
        // Construction
        ("bvzero", "All zeros"),
        ("bvones", "All ones"),
        ("bvone", "Single 1 in LSB"),
        ("extract", "Extract bits [high:low]"),
        ("zext", "Zero extend"),
        ("sext", "Sign extend"),
    ];

    for (name, desc) in bitvec_ops {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(format!("BitVec: {}", desc)),
            ..Default::default()
        });
    }

    items.push(CompletionItem {
        label: "BitVec".to_string(),
        kind: Some(CompletionItemKind::CLASS),
        detail: Some("BitVec(n) - Fixed-width bit-vector".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Bit-Vector** of width n.\n\nUsed for hardware verification, cryptography.\n\n```kleis\ndefine byte : BitVec(8) = bvzero(8)\ndefine word : BitVec(32) = bvzero(32)\n```".to_string(),
        })),
        insert_text: Some("BitVec(${1:n})".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });

    // ═══════════════════════════════════════════════════════════════════════
    // SET OPERATIONS (from manual chapter 18)
    // ═══════════════════════════════════════════════════════════════════════

    let set_ops = [
        ("empty_set", "Empty set ∅"),
        ("singleton", "Set with one element {x}"),
        ("insert", "Add element to set"),
        ("remove", "Remove element from set"),
        ("union", "Set union A ∪ B"),
        ("intersect", "Set intersection A ∩ B"),
        ("difference", "Set difference A \\ B"),
        ("complement", "Set complement"),
        ("in_set", "Membership test x ∈ S"),
        ("subset", "Subset test A ⊆ B"),
        ("proper_subset", "Proper subset A ⊂ B"),
    ];

    for (name, desc) in set_ops {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(format!("Set: {}", desc)),
            ..Default::default()
        });
    }

    items.push(CompletionItem {
        label: "Set".to_string(),
        kind: Some(CompletionItemKind::CLASS),
        detail: Some("Set(T) - Mathematical set".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Set type** backed by Z3 set theory.\n\n```kleis\nimport \"stdlib/sets.kleis\"\n\n// Build {1, 2, 3}\ninsert(3, insert(2, insert(1, empty_set())))\n```".to_string(),
        })),
        insert_text: Some("Set(${1:T})".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    });

    // ═══════════════════════════════════════════════════════════════════════
    // COMMON ADT PATTERNS (from manual chapter 4)
    // ═══════════════════════════════════════════════════════════════════════

    items.push(CompletionItem {
        label: "Option".to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some("data Option(T) = Some(T) | None".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Optional value** - represents presence or absence.\n\n```kleis\ndata Option(T) = Some(value : T) | None\n```".to_string(),
        })),
        insert_text: Some("data Option(T) = Some(value : T) | None".to_string()),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "Result".to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some("data Result(T, E) = Ok(T) | Err(E)".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Result type** - success or error.\n\n```kleis\ndata Result(T, E) = Ok(value : T) | Err(error : E)\n```".to_string(),
        })),
        insert_text: Some("data Result(T, E) = Ok(value : T) | Err(error : E)".to_string()),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "List".to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some("data List(T) = Nil | Cons(T, List(T))".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Linked list** - recursive ADT.\n\n```kleis\ndata List(T) = Nil | Cons(head : T, tail : List(T))\n```".to_string(),
        })),
        insert_text: Some("data List(T) = Nil | Cons(head : T, tail : List(T))".to_string()),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "Tree".to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some("data Tree(T) = Leaf(T) | Node(Tree, T, Tree)".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Binary tree** - recursive ADT.\n\n```kleis\ndata Tree(T) = Leaf(value : T) | Node(left : Tree(T), value : T, right : Tree(T))\n```".to_string(),
        })),
        insert_text: Some("data Tree(T) = Leaf(value : T) | Node(left : Tree(T), value : T, right : Tree(T))".to_string()),
        ..Default::default()
    });

    // ═══════════════════════════════════════════════════════════════════════
    // REPL COMMANDS (from manual chapter 12)
    // ═══════════════════════════════════════════════════════════════════════

    let repl_commands = [
        (
            ":eval",
            "Concrete evaluation (computes values)",
            ":eval 2 + 3",
        ),
        (
            ":verify",
            "Verify theorem (always true?)",
            ":verify x + y = y + x",
        ),
        (
            ":sat",
            "Find solution (exists?)",
            ":sat ∃(x : ℝ). x * x = 4",
        ),
        (":let", "Bind value to variable", ":let x = 5"),
        (":load", "Load a .kleis file", ":load examples/foo.kleis"),
        (":env", "Show defined functions/bindings", ":env"),
        (":type", "Show inferred type", ":type 42"),
        (":ast", "Show parsed AST", ":ast sin(x)"),
        (":symbols", "Unicode math symbols", ":symbols"),
        (":syntax", "Syntax reference", ":syntax"),
        (":help", "Show help", ":help"),
        (":quit", "Exit REPL", ":quit"),
    ];

    for (cmd, desc, example) in repl_commands {
        items.push(CompletionItem {
            label: cmd.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(desc.to_string()),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**REPL Command**\n\n{}\n\nExample: `{}`", desc, example),
            })),
            ..Default::default()
        });
    }

    items.push(CompletionItem {
        label: "it".to_string(),
        kind: Some(CompletionItemKind::VARIABLE),
        detail: Some("Last :eval result".to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**Magic variable** - holds the result of the last `:eval` command.\n\n```\nλ> :eval 2 + 3\n✅ 5\nλ> :eval it * 2\n✅ 10\n```".to_string(),
        })),
        ..Default::default()
    });

    items
}

/// Helper for stdlib completions
fn stdlib_completion(name: &str, detail: &str, doc: &str) -> CompletionItem {
    CompletionItem {
        label: name.to_string(),
        kind: Some(CompletionItemKind::STRUCT),
        detail: Some(detail.to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("**stdlib**\n\n{}", doc),
        })),
        ..Default::default()
    }
}

/// Helper to create a keyword completion item
fn keyword_completion(label: &str, detail: &str, snippet: &str, doc: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        detail: Some(detail.to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: doc.to_string(),
        })),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    }
}

/// Helper to create a type completion item
fn type_completion(symbol: &str, ascii: &str, detail: &str, doc: &str) -> CompletionItem {
    CompletionItem {
        label: symbol.to_string(),
        label_details: Some(CompletionItemLabelDetails {
            detail: Some(format!(" {}", ascii)),
            description: None,
        }),
        kind: Some(CompletionItemKind::CLASS),
        detail: Some(detail.to_string()),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: doc.to_string(),
        })),
        filter_text: Some(format!("{} {}", symbol, ascii)),
        ..Default::default()
    }
}

#[tokio::main]
async fn main() {
    // Set up stdin/stdout for LSP communication
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(KleisLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
