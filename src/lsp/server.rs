#![allow(deprecated)] // SymbolInformation::deprecated field is deprecated in lsp-types
//! LSP Server Implementation

use dashmap::DashMap;
use ropey::Rope;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::context::SharedContext;
use crate::kleis_ast::{Program, TopLevel};
use crate::kleis_parser::{parse_kleis_program, parse_kleis_program_with_file, KleisParseError};
use crate::type_checker::TypeChecker;
use crate::type_context::TypeContextBuilder;

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
pub struct KleisLanguageServer {
    /// LSP client for sending notifications
    client: Client,
    /// Open documents indexed by URI
    documents: DashMap<Url, Document>,
    /// Shared context for integration with REPL/Debugger (optional)
    #[allow(dead_code)]
    shared_ctx: Option<SharedContext>,
}

impl KleisLanguageServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            shared_ctx: None,
        }
    }

    fn with_context(client: Client, ctx: SharedContext) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            shared_ctx: Some(ctx),
        }
    }

    /// Resolve an import path relative to the document's directory
    ///
    /// For stdlib imports, checks KLEIS_ROOT environment variable first.
    fn resolve_import_path(import_path: &str, base_dir: &Path) -> PathBuf {
        let import = Path::new(import_path);

        if import.is_absolute() {
            // Absolute path: use as-is
            import.to_path_buf()
        } else if import_path.starts_with("stdlib/") {
            // Standard library: check KLEIS_ROOT first
            if let Ok(kleis_root) = std::env::var("KLEIS_ROOT") {
                let candidate = PathBuf::from(&kleis_root).join(import_path);
                if candidate.exists() {
                    return candidate;
                }
            }

            // Try current working directory
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

        // Parse with canonicalized file path for VS Code debugging support
        // VS Code needs absolute paths in stack traces
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let file_path_str = canonical.to_string_lossy().to_string();
        let program = match parse_kleis_program_with_file(&content, &file_path_str) {
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

        // =======================================================================
        // TYPE CHECKING - Run type inference on function definitions
        // =======================================================================
        let type_diagnostics = self.run_type_checking(&program);
        all_diagnostics.extend(type_diagnostics);

        (Some(program), Some(builder), imports, all_diagnostics)
    }

    /// Run type checking on function definitions and collect diagnostics
    fn run_type_checking(&self, program: &Program) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Create a type checker with stdlib loaded
        let mut checker = match TypeChecker::with_stdlib() {
            Ok(c) => c,
            Err(e) => {
                // If we can't load stdlib, just log and continue
                // This shouldn't block the user from editing
                eprintln!("Warning: Could not load stdlib for type checking: {}", e);
                return diagnostics;
            }
        };

        // Type check each function definition
        for item in &program.items {
            if let TopLevel::FunctionDef(func_def) = item {
                match checker.check_function_def(func_def) {
                    Ok(_inferred_type) => {
                        // Success! The function type-checks.
                        // We could add an info diagnostic or store the type for inlay hints
                    }
                    Err(e) => {
                        // Type error - create diagnostic using span from parsed AST
                        let (line, col, end_line, end_col) = func_def
                            .span
                            .clone()
                            .map(|s| (s.line.saturating_sub(1), s.column.saturating_sub(1), s.end_line.saturating_sub(1), s.end_column))
                            .unwrap_or((0, 0, 0, 80));
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line,
                                    character: col,
                                },
                                end: Position {
                                    line: end_line,
                                    character: end_col,
                                },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("kleis-types".to_string()),
                            message: e,
                            ..Default::default()
                        });
                    }
                }
            }
        }

        diagnostics
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
                // Signature Help - show function parameters as you type
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: Some(vec![",".to_string()]),
                    ..Default::default()
                }),
                // Inlay Hints - show inferred types inline
                inlay_hint_provider: Some(OneOf::Left(true)),
                // Document Formatting
                document_formatting_provider: Some(OneOf::Left(true)),
                // Find References
                references_provider: Some(OneOf::Left(true)),
                // Rename
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: Default::default(),
                })),
                // Workspace Symbols
                workspace_symbol_provider: Some(OneOf::Left(true)),
                // Code Actions - quick fixes
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                // Folding Ranges - code folding
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                // Document Links - clickable imports
                document_link_provider: Some(DocumentLinkOptions {
                    resolve_provider: Some(false),
                    work_done_progress_options: Default::default(),
                }),
                // Document Highlights - highlight matching symbols
                document_highlight_provider: Some(OneOf::Left(true)),
                // Selection Ranges - smart expand selection
                selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),
                // Code Lens - inline actions
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(false),
                }),
                // Implementation - find all implements
                implementation_provider: Some(ImplementationProviderCapability::Simple(true)),
                // On-type formatting - auto-indent
                document_on_type_formatting_provider: Some(DocumentOnTypeFormattingOptions {
                    first_trigger_character: "{".to_string(),
                    more_trigger_character: Some(vec!["}".to_string(), "\n".to_string()]),
                }),
                // Semantic Tokens - rich syntax highlighting
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: vec![
                                    SemanticTokenType::KEYWORD,
                                    SemanticTokenType::TYPE,
                                    SemanticTokenType::FUNCTION,
                                    SemanticTokenType::VARIABLE,
                                    SemanticTokenType::PARAMETER,
                                    SemanticTokenType::PROPERTY,
                                    SemanticTokenType::OPERATOR,
                                    SemanticTokenType::COMMENT,
                                    SemanticTokenType::STRING,
                                    SemanticTokenType::NUMBER,
                                    SemanticTokenType::NAMESPACE,
                                    SemanticTokenType::CLASS,
                                    SemanticTokenType::STRUCT,
                                    SemanticTokenType::ENUM,
                                    SemanticTokenType::ENUM_MEMBER,
                                    SemanticTokenType::MACRO,
                                ],
                                token_modifiers: vec![
                                    SemanticTokenModifier::DECLARATION,
                                    SemanticTokenModifier::DEFINITION,
                                    SemanticTokenModifier::READONLY,
                                    SemanticTokenModifier::STATIC,
                                    SemanticTokenModifier::ABSTRACT,
                                ],
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: None,
                            ..Default::default()
                        },
                    ),
                ),
                // Execute Command - for debugger integration
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["kleis.startDebugSession".to_string()],
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

        // Search all open documents for the definition
        for entry in self.documents.iter() {
            let doc_uri = entry.key().clone();
            let search_doc = entry.value();
            let text = search_doc.content.to_string();

            // Search for definition patterns in the text
            if let Some(location) = find_definition_in_text(&text, &word, &doc_uri) {
                return Ok(Some(GotoDefinitionResponse::Scalar(location)));
            }
        }

        // Also check imported files that might not be open
        if let Some(imports) = Some(&doc.imports) {
            if let Ok(doc_path) = uri.to_file_path() {
                if let Some(parent) = doc_path.parent() {
                    for import_path in imports.iter() {
                        let resolved = parent.join(import_path);
                        if resolved.exists() {
                            if let Ok(content) = std::fs::read_to_string(&resolved) {
                                if let Ok(import_uri) = Url::from_file_path(&resolved) {
                                    if let Some(location) =
                                        find_definition_in_text(&content, &word, &import_uri)
                                    {
                                        return Ok(Some(GotoDefinitionResponse::Scalar(location)));
                                    }
                                }
                            }
                        }
                    }
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
            use crate::kleis_ast::TopLevel;
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

    // =========================================================================
    // NEW LSP Features
    // =========================================================================

    /// Signature Help - shows function parameters as you type
    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let line_idx = position.line as usize;
        let col_idx = position.character as usize;

        let line = match doc.content.get_line(line_idx) {
            Some(line) => line.to_string(),
            None => return Ok(None),
        };

        // Find the function name before the opening parenthesis
        let before_cursor: String = line.chars().take(col_idx).collect();

        // Find the last '(' and extract function name before it
        if let Some(paren_pos) = before_cursor.rfind('(') {
            let before_paren: String = before_cursor.chars().take(paren_pos).collect();
            let func_name = extract_word_at(&before_paren, before_paren.len().saturating_sub(1));

            if !func_name.is_empty() {
                // Count commas to determine active parameter
                let after_paren: String = before_cursor.chars().skip(paren_pos + 1).collect();
                let active_param = after_paren.chars().filter(|c| *c == ',').count() as u32;

                // Look up function signature
                if let Some(ref ctx) = doc.type_context {
                    if let Some(sig) = ctx.operation_signature(&func_name) {
                        // Parse the signature to get parameters
                        let params = parse_signature_params(&sig);

                        let signature = SignatureInformation {
                            label: sig.clone(),
                            documentation: Some(Documentation::MarkupContent(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: format!(
                                    "**{}**\n\nOperation from imported structure.",
                                    func_name
                                ),
                            })),
                            parameters: Some(
                                params
                                    .iter()
                                    .map(|p| ParameterInformation {
                                        label: ParameterLabel::Simple(p.clone()),
                                        documentation: None,
                                    })
                                    .collect(),
                            ),
                            active_parameter: Some(active_param),
                        };

                        return Ok(Some(SignatureHelp {
                            signatures: vec![signature],
                            active_signature: Some(0),
                            active_parameter: Some(active_param),
                        }));
                    }
                }

                // Try builtin functions
                if let Some((sig, params)) = get_builtin_signature(&func_name) {
                    let signature = SignatureInformation {
                        label: sig.to_string(),
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!("**{}** (builtin)", func_name),
                        })),
                        parameters: Some(
                            params
                                .iter()
                                .map(|p| ParameterInformation {
                                    label: ParameterLabel::Simple(p.to_string()),
                                    documentation: None,
                                })
                                .collect(),
                        ),
                        active_parameter: Some(active_param),
                    };

                    return Ok(Some(SignatureHelp {
                        signatures: vec![signature],
                        active_signature: Some(0),
                        active_parameter: Some(active_param),
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Inlay Hints - show inferred types inline
    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let uri = &params.text_document.uri;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let hints = Vec::new();

        // Find let bindings and function definitions without type annotations
        if let Some(ref ast) = doc.ast {
            for item in &ast.items {
                if let TopLevel::FunctionDef(def) = item {
                    // Add hints for function parameters that could have types
                    // For now, show a simple hint for the function
                    // TODO: Use actual type inference

                    // We'll add more sophisticated hints when we have position info
                    let _ = def; // Placeholder - need AST positions
                }
            }
        }

        // For now, return empty hints (positions not available in AST)
        // Full implementation requires AST source spans
        Ok(Some(hints))
    }

    /// Document Formatting - format Kleis code
    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let content = doc.content.to_string();
        let formatted = format_kleis_code(&content);

        if formatted == content {
            return Ok(None);
        }

        // Replace entire document
        let lines = doc.content.len_lines();
        let last_line_len = doc.content.line(lines.saturating_sub(1)).len_chars();

        Ok(Some(vec![TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: lines.saturating_sub(1) as u32,
                    character: last_line_len as u32,
                },
            },
            new_text: formatted,
        }]))
    }

    /// Find References - find all usages of a symbol
    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Get the word at cursor
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

        // Find all occurrences in all open documents
        let mut locations = Vec::new();

        for entry in self.documents.iter() {
            let doc_uri = entry.key().clone();
            let doc = entry.value();

            let text = doc.content.to_string();
            for (line_num, line_content) in text.lines().enumerate() {
                // Find all occurrences of the word in this line
                let mut start = 0;
                while let Some(pos) = line_content[start..].find(&word) {
                    let actual_pos = start + pos;

                    // Check it's a whole word match
                    let before_ok = actual_pos == 0
                        || !line_content
                            .chars()
                            .nth(actual_pos - 1)
                            .is_some_and(|c| c.is_alphanumeric() || c == '_');
                    let after_ok = actual_pos + word.len() >= line_content.len()
                        || !line_content
                            .chars()
                            .nth(actual_pos + word.len())
                            .is_some_and(|c| c.is_alphanumeric() || c == '_');

                    if before_ok && after_ok {
                        locations.push(Location {
                            uri: doc_uri.clone(),
                            range: Range {
                                start: Position {
                                    line: line_num as u32,
                                    character: actual_pos as u32,
                                },
                                end: Position {
                                    line: line_num as u32,
                                    character: (actual_pos + word.len()) as u32,
                                },
                            },
                        });
                    }

                    start = actual_pos + word.len();
                }
            }
        }

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
    }

    /// Prepare Rename - check if rename is valid
    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let uri = &params.text_document.uri;
        let position = params.position;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

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

        // Find the range of the word
        let chars: Vec<char> = line.chars().collect();
        let is_word_char = |c: char| c.is_alphanumeric() || c == '_' || c == '\'';

        let mut start = col_idx;
        while start > 0 && is_word_char(chars[start - 1]) {
            start -= 1;
        }

        let mut end = col_idx;
        while end < chars.len() && is_word_char(chars[end]) {
            end += 1;
        }

        Ok(Some(PrepareRenameResponse::Range(Range {
            start: Position {
                line: position.line,
                character: start as u32,
            },
            end: Position {
                line: position.line,
                character: end as u32,
            },
        })))
    }

    /// Rename - rename symbol across all files
    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = &params.new_name;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let line_idx = position.line as usize;
        let col_idx = position.character as usize;

        let line = match doc.content.get_line(line_idx) {
            Some(line) => line.to_string(),
            None => return Ok(None),
        };

        let old_name = extract_word_at(&line, col_idx);
        if old_name.is_empty() {
            return Ok(None);
        }

        // Find all references and create edits
        let mut changes: std::collections::HashMap<Url, Vec<TextEdit>> =
            std::collections::HashMap::new();

        for entry in self.documents.iter() {
            let doc_uri = entry.key().clone();
            let doc = entry.value();

            let text = doc.content.to_string();
            let mut edits = Vec::new();

            for (line_num, line_content) in text.lines().enumerate() {
                let mut start = 0;
                while let Some(pos) = line_content[start..].find(&old_name) {
                    let actual_pos = start + pos;

                    // Check whole word match
                    let before_ok = actual_pos == 0
                        || !line_content
                            .chars()
                            .nth(actual_pos - 1)
                            .is_some_and(|c| c.is_alphanumeric() || c == '_');
                    let after_ok = actual_pos + old_name.len() >= line_content.len()
                        || !line_content
                            .chars()
                            .nth(actual_pos + old_name.len())
                            .is_some_and(|c| c.is_alphanumeric() || c == '_');

                    if before_ok && after_ok {
                        edits.push(TextEdit {
                            range: Range {
                                start: Position {
                                    line: line_num as u32,
                                    character: actual_pos as u32,
                                },
                                end: Position {
                                    line: line_num as u32,
                                    character: (actual_pos + old_name.len()) as u32,
                                },
                            },
                            new_text: new_name.clone(),
                        });
                    }

                    start = actual_pos + old_name.len();
                }
            }

            if !edits.is_empty() {
                changes.insert(doc_uri, edits);
            }
        }

        if changes.is_empty() {
            Ok(None)
        } else {
            Ok(Some(WorkspaceEdit {
                changes: Some(changes),
                ..Default::default()
            }))
        }
    }

    /// Workspace Symbols - search symbols across all files
    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let query = params.query.to_lowercase();
        let mut symbols = Vec::new();

        for entry in self.documents.iter() {
            let uri = entry.key().clone();
            let doc = entry.value();

            if let Some(ref ast) = doc.ast {
                for item in &ast.items {
                    let (name, kind) = match item {
                        TopLevel::FunctionDef(def) => (def.name.clone(), SymbolKind::FUNCTION),
                        TopLevel::StructureDef(s) => (s.name.clone(), SymbolKind::STRUCT),
                        TopLevel::ImplementsDef(i) => {
                            (format!("impl {}", i.structure_name), SymbolKind::CLASS)
                        }
                        TopLevel::DataDef(d) => (d.name.clone(), SymbolKind::ENUM),
                        TopLevel::TypeAlias(t) => (t.name.clone(), SymbolKind::TYPE_PARAMETER),
                        _ => continue,
                    };

                    // Filter by query
                    if query.is_empty() || name.to_lowercase().contains(&query) {
                        symbols.push(SymbolInformation {
                            name,
                            kind,
                            tags: None,
                            deprecated: None,
                            location: Location {
                                uri: uri.clone(),
                                range: Range::default(),
                            },
                            container_name: None,
                        });
                    }
                }
            }
        }

        if symbols.is_empty() {
            Ok(None)
        } else {
            #[allow(deprecated)]
            Ok(Some(symbols))
        }
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let uri = &params.text_document.uri;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let mut lenses = Vec::new();
        let text = doc.content.to_string();

        for (line_num, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            let line_num = line_num as u32;

            // Add lens for axioms
            if trimmed.starts_with("axiom ") {
                // Extract axiom name
                if let Some(name_end) = trimmed.find(':') {
                    let name = trimmed[6..name_end].trim();
                    lenses.push(CodeLens {
                        range: Range {
                            start: Position {
                                line: line_num,
                                character: 0,
                            },
                            end: Position {
                                line: line_num,
                                character: line.len() as u32,
                            },
                        },
                        command: Some(Command {
                            title: format!("▶ Verify axiom '{}'", name),
                            command: "kleis.verifyAxiom".to_string(),
                            arguments: Some(vec![
                                serde_json::Value::String(uri.to_string()),
                                serde_json::Value::String(name.to_string()),
                            ]),
                        }),
                        data: None,
                    });
                }
            }

            // Add lens for structure definitions
            if trimmed.starts_with("structure ") {
                if let Some(paren_pos) = trimmed.find('(') {
                    let name = trimmed[10..paren_pos].trim();
                    lenses.push(CodeLens {
                        range: Range {
                            start: Position {
                                line: line_num,
                                character: 0,
                            },
                            end: Position {
                                line: line_num,
                                character: line.len() as u32,
                            },
                        },
                        command: Some(Command {
                            title: "⚡ Find implementations".to_string(),
                            command: "kleis.findImplementations".to_string(),
                            arguments: Some(vec![serde_json::Value::String(name.to_string())]),
                        }),
                        data: None,
                    });
                }
            }

            // Add lens for define statements
            if let Some(rest) = trimmed.strip_prefix("define ") {
                // Extract function name
                let name_end = rest
                    .find('(')
                    .or_else(|| rest.find(' '))
                    .unwrap_or(rest.len());
                let name = rest[..name_end].trim();

                if !name.is_empty() {
                    lenses.push(CodeLens {
                        range: Range {
                            start: Position {
                                line: line_num,
                                character: 0,
                            },
                            end: Position {
                                line: line_num,
                                character: line.len() as u32,
                            },
                        },
                        command: Some(Command {
                            title: "📍 Find references".to_string(),
                            command: "editor.action.findReferences".to_string(),
                            arguments: Some(vec![
                                serde_json::Value::String(uri.to_string()),
                                serde_json::json!({
                                    "line": line_num,
                                    "character": 7  // Position after "define "
                                }),
                            ]),
                        }),
                        data: None,
                    });
                }
            }
        }

        if lenses.is_empty() {
            Ok(None)
        } else {
            Ok(Some(lenses))
        }
    }

    async fn goto_implementation(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Get the word at cursor
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

        // Find all "implements Word(" patterns
        let mut locations = Vec::new();
        let pattern = format!("implements {}(", word);

        for entry in self.documents.iter() {
            let doc_uri = entry.key().clone();
            let search_doc = entry.value();
            let text = search_doc.content.to_string();

            for (line_num, line_text) in text.lines().enumerate() {
                if let Some(col) = line_text.find(&pattern) {
                    locations.push(Location {
                        uri: doc_uri.clone(),
                        range: Range {
                            start: Position {
                                line: line_num as u32,
                                character: col as u32,
                            },
                            end: Position {
                                line: line_num as u32,
                                character: (col + pattern.len()) as u32,
                            },
                        },
                    });
                }
            }
        }

        if locations.is_empty() {
            Ok(None)
        } else if locations.len() == 1 {
            Ok(Some(GotoDefinitionResponse::Scalar(
                locations.into_iter().next().unwrap(),
            )))
        } else {
            Ok(Some(GotoDefinitionResponse::Array(locations)))
        }
    }

    async fn on_type_formatting(
        &self,
        params: DocumentOnTypeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let position = params.text_document_position.position;
        let ch = &params.ch;

        // Auto-indent after opening brace
        if ch == "{" {
            // Add newline with increased indent
            return Ok(Some(vec![TextEdit {
                range: Range {
                    start: Position {
                        line: position.line,
                        character: position.character + 1,
                    },
                    end: Position {
                        line: position.line,
                        character: position.character + 1,
                    },
                },
                new_text: "\n    ".to_string(),
            }]));
        }

        // Auto-dedent after closing brace
        if ch == "}" {
            // This is tricky - we'd need to look at current indentation
            // For now, just let the formatter handle it
            return Ok(None);
        }

        Ok(None)
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let uri = &params.text_document.uri;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let mut ranges = Vec::new();
        let text = doc.content.to_string();

        // Track brace/bracket nesting for folding
        let mut brace_stack: Vec<u32> = Vec::new();

        for (line_num, line) in text.lines().enumerate() {
            let line_num = line_num as u32;
            let trimmed = line.trim();

            // Structure, implements, data blocks
            if (trimmed.starts_with("structure ")
                || trimmed.starts_with("implements ")
                || trimmed.starts_with("data "))
                && trimmed.contains('{')
                && !trimmed.contains('}')
            {
                brace_stack.push(line_num);
            }

            // Opening braces
            for ch in trimmed.chars() {
                if ch == '{' {
                    brace_stack.push(line_num);
                } else if ch == '}' {
                    if let Some(start_line) = brace_stack.pop() {
                        if line_num > start_line {
                            ranges.push(FoldingRange {
                                start_line,
                                start_character: None,
                                end_line: line_num,
                                end_character: None,
                                kind: Some(FoldingRangeKind::Region),
                                collapsed_text: None,
                            });
                        }
                    }
                }
            }

            // Multi-line comments (if we add them)
            // For now, fold consecutive comment lines
            if trimmed.starts_with("//") {
                // Check if this starts a block of comments
                let mut end_line = line_num;
                for (i, next_line) in text.lines().enumerate().skip(line_num as usize + 1) {
                    if next_line.trim().starts_with("//") {
                        end_line = i as u32;
                    } else {
                        break;
                    }
                }
                if end_line > line_num {
                    ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: None,
                        end_line,
                        end_character: None,
                        kind: Some(FoldingRangeKind::Comment),
                        collapsed_text: Some("// ...".to_string()),
                    });
                }
            }

            // Imports section
            if trimmed.starts_with("import ") {
                let mut end_line = line_num;
                for (i, next_line) in text.lines().enumerate().skip(line_num as usize + 1) {
                    if next_line.trim().starts_with("import ") {
                        end_line = i as u32;
                    } else {
                        break;
                    }
                }
                if end_line > line_num {
                    ranges.push(FoldingRange {
                        start_line: line_num,
                        start_character: None,
                        end_line,
                        end_character: None,
                        kind: Some(FoldingRangeKind::Imports),
                        collapsed_text: Some("imports ...".to_string()),
                    });
                }
            }
        }

        if ranges.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ranges))
        }
    }

    async fn document_link(&self, params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
        let uri = &params.text_document.uri;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let mut links = Vec::new();
        let text = doc.content.to_string();

        for (line_num, line) in text.lines().enumerate() {
            let line_num = line_num as u32;

            // Find import statements: import "path/to/file.kleis"
            if let Some(import_start) = line.find("import ") {
                if let Some(quote_start) = line[import_start..].find('"') {
                    let path_start = import_start + quote_start + 1;
                    if let Some(quote_end) = line[path_start..].find('"') {
                        let import_path = &line[path_start..path_start + quote_end];

                        // Resolve relative to current document
                        if let Ok(doc_path) = uri.to_file_path() {
                            if let Some(parent) = doc_path.parent() {
                                let resolved = parent.join(import_path);
                                if resolved.exists() {
                                    if let Ok(target_uri) = Url::from_file_path(&resolved) {
                                        links.push(DocumentLink {
                                            range: Range {
                                                start: Position {
                                                    line: line_num,
                                                    character: path_start as u32 - 1,
                                                },
                                                end: Position {
                                                    line: line_num,
                                                    character: (path_start + quote_end + 1) as u32,
                                                },
                                            },
                                            target: Some(target_uri),
                                            tooltip: Some(format!("Open {}", resolved.display())),
                                            data: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if links.is_empty() {
            Ok(None)
        } else {
            Ok(Some(links))
        }
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Get word at cursor
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

        let mut highlights = Vec::new();
        let text = doc.content.to_string();

        // Find all occurrences of this word
        for (line_num, line_text) in text.lines().enumerate() {
            let line_num = line_num as u32;
            let mut search_start = 0;

            while let Some(col) = line_text[search_start..].find(&word) {
                let actual_col = search_start + col;

                // Verify it's a whole word match
                let before_ok = actual_col == 0
                    || !line_text
                        .chars()
                        .nth(actual_col - 1)
                        .map(|c| c.is_alphanumeric() || c == '_')
                        .unwrap_or(false);
                let after_ok = actual_col + word.len() >= line_text.len()
                    || !line_text
                        .chars()
                        .nth(actual_col + word.len())
                        .map(|c| c.is_alphanumeric() || c == '_')
                        .unwrap_or(false);

                if before_ok && after_ok {
                    highlights.push(DocumentHighlight {
                        range: Range {
                            start: Position {
                                line: line_num,
                                character: actual_col as u32,
                            },
                            end: Position {
                                line: line_num,
                                character: (actual_col + word.len()) as u32,
                            },
                        },
                        kind: Some(DocumentHighlightKind::TEXT),
                    });
                }

                search_start = actual_col + word.len();
            }
        }

        if highlights.is_empty() {
            Ok(None)
        } else {
            Ok(Some(highlights))
        }
    }

    async fn selection_range(
        &self,
        params: SelectionRangeParams,
    ) -> Result<Option<Vec<SelectionRange>>> {
        let uri = &params.text_document.uri;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let text = doc.content.to_string();
        let lines: Vec<&str> = text.lines().collect();

        let mut selection_ranges = Vec::new();

        for position in &params.positions {
            let line_idx = position.line as usize;
            let col_idx = position.character as usize;

            if line_idx >= lines.len() {
                continue;
            }

            let line = lines[line_idx];

            // Level 1: Word at cursor
            let word_range = if col_idx < line.len() {
                let mut start = col_idx;
                let mut end = col_idx;

                // Expand to word boundaries
                while start > 0
                    && line
                        .chars()
                        .nth(start - 1)
                        .map(|c| c.is_alphanumeric() || c == '_')
                        .unwrap_or(false)
                {
                    start -= 1;
                }
                while end < line.len()
                    && line
                        .chars()
                        .nth(end)
                        .map(|c| c.is_alphanumeric() || c == '_')
                        .unwrap_or(false)
                {
                    end += 1;
                }

                Range {
                    start: Position {
                        line: position.line,
                        character: start as u32,
                    },
                    end: Position {
                        line: position.line,
                        character: end as u32,
                    },
                }
            } else {
                Range {
                    start: *position,
                    end: *position,
                }
            };

            // Level 2: Current line
            let line_range = Range {
                start: Position {
                    line: position.line,
                    character: 0,
                },
                end: Position {
                    line: position.line,
                    character: line.len() as u32,
                },
            };

            // Level 3: Find enclosing block (braces)
            let block_range = find_enclosing_block(&text, line_idx, col_idx);

            // Level 4: Entire document
            let doc_range = Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: lines.len().saturating_sub(1) as u32,
                    character: lines.last().map(|l| l.len()).unwrap_or(0) as u32,
                },
            };

            // Build nested selection ranges (innermost to outermost)
            let doc_selection = SelectionRange {
                range: doc_range,
                parent: None,
            };

            let block_selection = if let Some(br) = block_range {
                SelectionRange {
                    range: br,
                    parent: Some(Box::new(doc_selection)),
                }
            } else {
                doc_selection
            };

            let line_selection = SelectionRange {
                range: line_range,
                parent: Some(Box::new(block_selection)),
            };

            let word_selection = SelectionRange {
                range: word_range,
                parent: Some(Box::new(line_selection)),
            };

            selection_ranges.push(word_selection);
        }

        if selection_ranges.is_empty() {
            Ok(None)
        } else {
            Ok(Some(selection_ranges))
        }
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let mut actions: Vec<CodeActionOrCommand> = Vec::new();

        // Get selected text range
        let range = params.range;
        let start_line = range.start.line as usize;
        let end_line = range.end.line as usize;

        // Extract text in range
        let mut text_in_range = String::new();
        for line_idx in start_line..=end_line {
            if let Some(line) = doc.content.get_line(line_idx) {
                let line_str = line.to_string();
                if line_idx == start_line && line_idx == end_line {
                    let start_col = range.start.character as usize;
                    let end_col = range.end.character as usize;
                    if end_col <= line_str.len() {
                        text_in_range.push_str(&line_str[start_col..end_col]);
                    }
                } else {
                    text_in_range.push_str(&line_str);
                }
            }
        }

        // ASCII to Unicode conversions
        let ascii_to_unicode_replacements = [
            // Quantifiers & Logic
            ("forall", "∀", "Convert 'forall' to ∀"),
            ("exists", "∃", "Convert 'exists' to ∃"),
            ("lambda", "λ", "Convert 'lambda' to λ"),
            ("and", "∧", "Convert 'and' to ∧"),
            ("or", "∨", "Convert 'or' to ∨"),
            ("not", "¬", "Convert 'not' to ¬"),
            // Arrows
            ("->", "→", "Convert '->' to →"),
            ("<-", "←", "Convert '<-' to ←"),
            ("=>", "⇒", "Convert '=>' to ⇒"),
            // Comparison
            ("<=", "≤", "Convert '<=' to ≤"),
            (">=", "≥", "Convert '>=' to ≥"),
            ("!=", "≠", "Convert '!=' to ≠"),
            ("/\\", "∧", "Convert '/\\' to ∧"),
            ("\\/", "∨", "Convert '\\/' to ∨"),
            // Calculus
            ("nabla", "∇", "Convert 'nabla' to ∇"),
            ("partial", "∂", "Convert 'partial' to ∂"),
            ("infinity", "∞", "Convert 'infinity' to ∞"),
            // Products
            ("times", "×", "Convert 'times' to ×"),
            ("tensor", "⊗", "Convert 'tensor' to ⊗"),
            // Number Sets
            ("Real", "ℝ", "Use Unicode ℝ for Real"),
            ("Complex", "ℂ", "Use Unicode ℂ for Complex"),
            ("Integer", "ℤ", "Use Unicode ℤ for Integer"),
            ("Nat", "ℕ", "Use Unicode ℕ for Nat"),
            ("Rational", "ℚ", "Use Unicode ℚ for Rational"),
            ("Bool", "𝔹", "Use Unicode 𝔹 for Bool"),
        ];

        for (ascii, unicode, title) in ascii_to_unicode_replacements {
            if text_in_range.contains(ascii) {
                let new_text = text_in_range.replace(ascii, unicode);
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: title.to_string(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    edit: Some(WorkspaceEdit {
                        changes: Some(
                            [(
                                uri.clone(),
                                vec![TextEdit {
                                    range,
                                    new_text: new_text.clone(),
                                }],
                            )]
                            .into_iter()
                            .collect(),
                        ),
                        ..Default::default()
                    }),
                    ..Default::default()
                }));
            }
        }

        // Structure snippet actions
        if text_in_range.trim() == "structure" || text_in_range.trim().starts_with("structure ") {
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Complete structure template".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                edit: Some(WorkspaceEdit {
                    changes: Some(
                        [(
                            uri.clone(),
                            vec![TextEdit {
                                range,
                                new_text:
                                    "structure Name(T) {\n    element identity : T\n    operation op : T → T → T\n    axiom law: ∀(a : T). op(identity, a) = a\n}"
                                        .to_string(),
                            }],
                        )]
                        .into_iter()
                        .collect(),
                    ),
                    ..Default::default()
                }),
                ..Default::default()
            }));
        }

        // Convert all ASCII to Unicode in selection
        if !text_in_range.is_empty() && range.start != range.end {
            let mut converted = text_in_range.clone();
            // Quantifiers & Logic
            converted = converted.replace("forall", "∀");
            converted = converted.replace("exists", "∃");
            converted = converted.replace("lambda", "λ");
            converted = converted.replace(" and ", " ∧ ");
            converted = converted.replace(" or ", " ∨ ");
            converted = converted.replace("not(", "¬(");
            // Arrows
            converted = converted.replace("->", "→");
            converted = converted.replace("=>", "⇒");
            // Comparison
            converted = converted.replace("<=", "≤");
            converted = converted.replace(">=", "≥");
            converted = converted.replace("!=", "≠");
            // Calculus
            converted = converted.replace("nabla", "∇");
            converted = converted.replace("partial", "∂");
            converted = converted.replace("infinity", "∞");

            if converted != text_in_range {
                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: "Convert all ASCII operators to Unicode".to_string(),
                    kind: Some(CodeActionKind::REFACTOR),
                    edit: Some(WorkspaceEdit {
                        changes: Some(
                            [(
                                uri.clone(),
                                vec![TextEdit {
                                    range,
                                    new_text: converted,
                                }],
                            )]
                            .into_iter()
                            .collect(),
                        ),
                        ..Default::default()
                    }),
                    ..Default::default()
                }));
            }
        }

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = &params.text_document.uri;

        let doc = match self.documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let text = doc.content.to_string();
        let tokens = tokenize_for_semantics(&text);

        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: tokens,
        })))
    }

    /// Handle workspace/executeCommand requests (for debugger integration)
    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<serde_json::Value>> {
        match params.command.as_str() {
            "kleis.startDebugSession" => {
                // Start DAP server on a dynamic port
                let program_path = params.arguments
                    .first()
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                self.client
                    .log_message(MessageType::INFO, format!("Starting debug session for: {}", program_path))
                    .await;
                
                // Find an available port
                let port = find_available_port().unwrap_or(0);
                
                if port == 0 {
                    return Ok(Some(serde_json::json!({
                        "error": "Could not find available port for DAP server"
                    })));
                }
                
                // Spawn DAP server in background
                let ctx = self.shared_ctx.clone();
                std::thread::spawn(move || {
                    if let Err(e) = crate::dap::run_tcp_server_with_context_on_port(port, ctx) {
                        eprintln!("DAP server error: {}", e);
                    }
                });
                
                // Give the server a moment to start
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                
                self.client
                    .log_message(MessageType::INFO, format!("DAP server started on port {}", port))
                    .await;
                
                Ok(Some(serde_json::json!({
                    "port": port
                })))
            }
            _ => {
                self.client
                    .log_message(MessageType::WARNING, format!("Unknown command: {}", params.command))
                    .await;
                Ok(None)
            }
        }
    }
}

/// Find an available TCP port for the DAP server
fn find_available_port() -> Option<u16> {
    // Try to bind to port 0 to get an available port from the OS
    std::net::TcpListener::bind("127.0.0.1:0")
        .ok()
        .and_then(|listener| listener.local_addr().ok())
        .map(|addr| addr.port())
}

/// Tokenize Kleis source for semantic highlighting
fn tokenize_for_semantics(source: &str) -> Vec<SemanticToken> {
    let mut tokens = Vec::new();

    // Token type indices (must match the order in SemanticTokensLegend)
    const KEYWORD: u32 = 0;
    const TYPE: u32 = 1;
    const FUNCTION: u32 = 2;
    const VARIABLE: u32 = 3;
    const PARAMETER: u32 = 4;
    const PROPERTY: u32 = 5;
    const OPERATOR: u32 = 6;
    const COMMENT: u32 = 7;
    const STRING: u32 = 8;
    const NUMBER: u32 = 9;
    const NAMESPACE: u32 = 10;
    const CLASS: u32 = 11;
    const STRUCT: u32 = 12;
    const ENUM: u32 = 13;
    const ENUM_MEMBER: u32 = 14;
    const MACRO: u32 = 15;

    // Modifier indices
    const MOD_DECLARATION: u32 = 1 << 0;
    const MOD_DEFINITION: u32 = 1 << 1;
    const MOD_READONLY: u32 = 1 << 2;

    // Kleis keywords
    let keywords: HashSet<&str> = [
        "structure",
        "implements",
        "extends",
        "over",
        "operation",
        "axiom",
        "element",
        "data",
        "define",
        "type",
        "import",
        "where",
        "let",
        "in",
        "if",
        "then",
        "else",
        "match",
        "with",
        "forall",
        "exists",
        "as",
        "True",
        "False",
    ]
    .into_iter()
    .collect();

    // Type names (both Unicode and ASCII)
    let type_names: HashSet<&str> = [
        "ℝ", "ℂ", "ℤ", "ℕ", "ℚ", "𝔹", "Real", "Complex", "Integer", "Nat", "Rational", "Bool",
        "Matrix", "Vector", "List", "Option", "Set", "Map", "String", "Tensor",
    ]
    .into_iter()
    .collect();

    // Mathematical operators
    let operators: HashSet<&str> = [
        "∀", "∃", "λ", "→", "←", "↔", "⇒", "⇔", "∧", "∨", "¬", "×", "⊗", "∘", "∇", "∂", "∫", "∑",
        "∏", "√", "≠", "≤", "≥", "≈", "≡", "∈", "∉", "⊂", "⊆", "†", "ᵀ", "⊤", "⊥", "∪", "∩", "∅",
        "∞",
    ]
    .into_iter()
    .collect();

    let mut prev_line = 0u32;
    let mut prev_col = 0u32;

    for (line_num, line) in source.lines().enumerate() {
        let line_num = line_num as u32;

        // Reset column for new line
        if line_num != prev_line {
            prev_col = 0;
        }

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Check for comments
        if let Some(comment_start) = line.find("//") {
            let col = comment_start as u32;
            let length = (line.len() - comment_start) as u32;

            let delta_line = line_num - prev_line;
            let delta_col = if delta_line == 0 { col - prev_col } else { col };

            tokens.push(SemanticToken {
                delta_line,
                delta_start: delta_col,
                length,
                token_type: COMMENT,
                token_modifiers_bitset: 0,
            });

            prev_line = line_num;
            prev_col = col;
        }

        // Check for strings
        let mut in_string = false;
        let mut string_start = 0;
        for (i, ch) in line.char_indices() {
            if ch == '"' {
                if in_string {
                    // End of string
                    let col = string_start as u32;
                    let length = (i - string_start + 1) as u32;

                    let delta_line = line_num - prev_line;
                    let delta_col = if delta_line == 0 { col - prev_col } else { col };

                    tokens.push(SemanticToken {
                        delta_line,
                        delta_start: delta_col,
                        length,
                        token_type: STRING,
                        token_modifiers_bitset: 0,
                    });

                    prev_line = line_num;
                    prev_col = col;
                    in_string = false;
                } else {
                    // Start of string
                    string_start = i;
                    in_string = true;
                }
            }
        }

        // Tokenize words
        let mut word_start = 0;
        let mut in_word = false;

        for (i, ch) in line.char_indices() {
            let is_word_char = ch.is_alphanumeric() || ch == '_' || ch == '\'';

            if is_word_char && !in_word {
                word_start = i;
                in_word = true;
            } else if !is_word_char && in_word {
                // End of word
                let word = &line[word_start..i];
                let col = word_start as u32;
                let length = word.chars().count() as u32;

                let token_type = if keywords.contains(word) {
                    Some((KEYWORD, 0))
                } else if type_names.contains(word) {
                    Some((TYPE, MOD_READONLY))
                } else if word
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    // Capitalized names are likely types or constructors
                    Some((CLASS, 0))
                } else if word.parse::<f64>().is_ok() || word.parse::<i64>().is_ok() {
                    Some((NUMBER, 0))
                } else if word.starts_with("builtin_") {
                    Some((MACRO, 0))
                } else {
                    None // Regular identifier - don't highlight
                };

                if let Some((tt, mods)) = token_type {
                    let delta_line = line_num - prev_line;
                    let delta_col = if delta_line == 0 { col - prev_col } else { col };

                    tokens.push(SemanticToken {
                        delta_line,
                        delta_start: delta_col,
                        length,
                        token_type: tt,
                        token_modifiers_bitset: mods,
                    });

                    prev_line = line_num;
                    prev_col = col;
                }

                in_word = false;
            }
        }

        // Handle word at end of line
        if in_word {
            let word = &line[word_start..];
            let col = word_start as u32;
            let length = word.chars().count() as u32;

            let token_type = if keywords.contains(word) {
                Some((KEYWORD, 0))
            } else if type_names.contains(word) {
                Some((TYPE, MOD_READONLY))
            } else if word
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
            {
                Some((CLASS, 0))
            } else if word.parse::<f64>().is_ok() || word.parse::<i64>().is_ok() {
                Some((NUMBER, 0))
            } else if word.starts_with("builtin_") {
                Some((MACRO, 0))
            } else {
                None
            };

            if let Some((tt, mods)) = token_type {
                let delta_line = line_num - prev_line;
                let delta_col = if delta_line == 0 { col - prev_col } else { col };

                tokens.push(SemanticToken {
                    delta_line,
                    delta_start: delta_col,
                    length,
                    token_type: tt,
                    token_modifiers_bitset: mods,
                });

                prev_line = line_num;
                prev_col = col;
            }
        }

        // Check for Unicode operators
        for op in &operators {
            for (i, _) in line.match_indices(op) {
                let col = i as u32;
                let length = op.chars().count() as u32;

                let delta_line = line_num - prev_line;
                let delta_col = if delta_line == 0 {
                    col.saturating_sub(prev_col)
                } else {
                    col
                };

                tokens.push(SemanticToken {
                    delta_line,
                    delta_start: delta_col,
                    length,
                    token_type: OPERATOR,
                    token_modifiers_bitset: 0,
                });

                prev_line = line_num;
                prev_col = col;
            }
        }
    }

    // Suppress unused warnings - these are for future use
    let _ = (
        FUNCTION,
        VARIABLE,
        PARAMETER,
        PROPERTY,
        NAMESPACE,
        STRUCT,
        ENUM,
        ENUM_MEMBER,
        MOD_DECLARATION,
        MOD_DEFINITION,
    );

    tokens
}

/// Parse signature to extract parameter names
fn parse_signature_params(sig: &str) -> Vec<String> {
    // Simple parser for signatures like "operation add : M → M → M"
    let mut params = Vec::new();

    if let Some(colon_pos) = sig.find(':') {
        let type_part = &sig[colon_pos + 1..];
        // Split by → and take all but the last (which is return type)
        let parts: Vec<&str> = type_part.split('→').collect();
        for (i, part) in parts.iter().take(parts.len().saturating_sub(1)).enumerate() {
            params.push(format!("arg{}: {}", i + 1, part.trim()));
        }
    }

    params
}

/// Get builtin function signatures
fn get_builtin_signature(name: &str) -> Option<(&'static str, Vec<&'static str>)> {
    match name {
        "sin" | "cos" | "tan" | "exp" | "ln" | "sqrt" | "abs" => {
            Some((format!("{}(x: ℝ) → ℝ", name).leak(), vec!["x: ℝ"]))
        }
        "atan2" => Some(("atan2(y: ℝ, x: ℝ) → ℝ", vec!["y: ℝ", "x: ℝ"])),
        "complex" => Some(("complex(re: ℝ, im: ℝ) → ℂ", vec!["re: ℝ", "im: ℝ"])),
        "re" | "im" => Some((format!("{}(z: ℂ) → ℝ", name).leak(), vec!["z: ℂ"])),
        "conj" => Some(("conj(z: ℂ) → ℂ", vec!["z: ℂ"])),
        "Matrix" => Some((
            "Matrix(m: ℕ, n: ℕ, T) → Type",
            vec!["m: ℕ", "n: ℕ", "T: Type"],
        )),
        "transpose" => Some((
            "transpose(A: Matrix(m,n,T)) → Matrix(n,m,T)",
            vec!["A: Matrix"],
        )),
        "det" => Some(("det(A: Matrix(n,n,T)) → T", vec!["A: Matrix(n,n)"])),
        "trace" => Some(("trace(A: Matrix(n,n,T)) → T", vec!["A: Matrix(n,n)"])),
        "head" => Some(("head(xs: List(T)) → Option(T)", vec!["xs: List(T)"])),
        "tail" => Some(("tail(xs: List(T)) → List(T)", vec!["xs: List(T)"])),
        "length" => Some(("length(xs: List(T)) → ℕ", vec!["xs: List(T)"])),
        "map" => Some((
            "map(f: A → B, xs: List(A)) → List(B)",
            vec!["f: A → B", "xs: List(A)"],
        )),
        "filter" => Some((
            "filter(p: A → Bool, xs: List(A)) → List(A)",
            vec!["p: A → Bool", "xs: List(A)"],
        )),
        "fold" => Some((
            "fold(f: (B,A) → B, init: B, xs: List(A)) → B",
            vec!["f: (B,A) → B", "init: B", "xs: List(A)"],
        )),
        "concat" => Some((
            "concat(s1: String, s2: String) → String",
            vec!["s1: String", "s2: String"],
        )),
        "strlen" => Some(("strlen(s: String) → ℕ", vec!["s: String"])),
        "substr" => Some((
            "substr(s: String, start: ℕ, len: ℕ) → String",
            vec!["s: String", "start: ℕ", "len: ℕ"],
        )),
        _ => None,
    }
}

/// Format Kleis code (basic implementation)
fn format_kleis_code(code: &str) -> String {
    let mut result = String::new();
    let mut indent: usize = 0;
    let indent_str = "    ";

    for line in code.lines() {
        let trimmed = line.trim();

        // Decrease indent for closing braces
        if trimmed.starts_with('}') || trimmed.starts_with(')') || trimmed.starts_with(']') {
            indent = indent.saturating_sub(1);
        }

        // Skip empty lines at the start
        if result.is_empty() && trimmed.is_empty() {
            continue;
        }

        // Add indented line
        if !trimmed.is_empty() {
            for _ in 0..indent {
                result.push_str(indent_str);
            }
            result.push_str(trimmed);
        }
        result.push('\n');

        // Increase indent after opening braces
        if trimmed.ends_with('{') || trimmed.ends_with('(') && !trimmed.ends_with("()") {
            indent += 1;
        }
    }

    // Remove trailing whitespace
    result.trim_end().to_string() + "\n"
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

/// Find where a symbol is defined in the text
fn find_definition_in_text(text: &str, word: &str, uri: &Url) -> Option<Location> {
    // Definition patterns to search for
    let patterns = [
        format!("define {}(", word),     // define name(
        format!("define {} =", word),    // define name =
        format!("define {} :", word),    // define name :
        format!("structure {}(", word),  // structure Name(
        format!("structure {} ", word),  // structure Name
        format!("data {} =", word),      // data Name =
        format!("data {}(", word),       // data Name(
        format!("type {} =", word),      // type Name =
        format!("operation {} :", word), // operation name :
        format!("element {} :", word),   // element name :
        format!("axiom {}:", word),      // axiom name:
        format!("axiom {} :", word),     // axiom name :
        format!("implements {}(", word), // implements Name(
        format!("| {}(", word),          // | Constructor(  (data variant)
        format!("| {}", word),           // | Constructor   (data variant)
    ];

    for (line_num, line) in text.lines().enumerate() {
        for pattern in &patterns {
            if let Some(col) = line.find(pattern.as_str()) {
                // Find where the actual name starts in the pattern
                let name_offset = pattern.find(word).unwrap_or(0);
                let name_col = col + name_offset;

                return Some(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position {
                            line: line_num as u32,
                            character: name_col as u32,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: (name_col + word.len()) as u32,
                        },
                    },
                });
            }
        }
    }

    None
}

/// Find the enclosing brace block for a given position
fn find_enclosing_block(text: &str, line_idx: usize, _col_idx: usize) -> Option<Range> {
    let lines: Vec<&str> = text.lines().collect();

    // Search backwards for opening brace
    let mut brace_count = 0;
    let mut block_start_line = None;

    for i in (0..=line_idx).rev() {
        let line = lines.get(i)?;
        for ch in line.chars().rev() {
            if ch == '}' {
                brace_count += 1;
            } else if ch == '{' {
                if brace_count == 0 {
                    block_start_line = Some(i);
                    break;
                } else {
                    brace_count -= 1;
                }
            }
        }
        if block_start_line.is_some() {
            break;
        }
    }

    let start_line = block_start_line?;

    // Search forwards for closing brace
    brace_count = 1; // We're inside the block
    let mut block_end_line = None;

    for i in (start_line + 1)..lines.len() {
        let line = lines.get(i)?;
        for ch in line.chars() {
            if ch == '{' {
                brace_count += 1;
            } else if ch == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    block_end_line = Some(i);
                    break;
                }
            }
        }
        if block_end_line.is_some() {
            break;
        }
    }

    let end_line = block_end_line?;

    Some(Range {
        start: Position {
            line: start_line as u32,
            character: 0,
        },
        end: Position {
            line: end_line as u32,
            character: lines[end_line].len() as u32,
        },
    })
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
    // ASCII TO UNICODE INPUT HELPERS
    // Type the ASCII, insert the Unicode symbol!
    // ═══════════════════════════════════════════════════════════════════════

    let ascii_to_unicode = [
        // Quantifiers & Logic
        ("forall", "∀", "Universal quantifier (for all)"),
        ("exists", "∃", "Existential quantifier (there exists)"),
        ("lambda", "λ", "Lambda abstraction"),
        ("->", "→", "Function arrow / implies"),
        ("<-", "←", "Left arrow"),
        ("<->", "↔", "Biconditional (iff)"),
        ("=>", "⇒", "Logical implication"),
        ("<=>", "⇔", "Logical equivalence"),
        ("and", "∧", "Logical AND"),
        ("or", "∨", "Logical OR"),
        ("not", "¬", "Logical NOT"),
        ("true", "⊤", "Top / True"),
        ("false", "⊥", "Bottom / False"),
        // Comparison
        ("!=", "≠", "Not equal"),
        ("/=", "≠", "Not equal (alt)"),
        ("<=", "≤", "Less than or equal"),
        (">=", "≥", "Greater than or equal"),
        ("~=", "≈", "Approximately equal"),
        ("===", "≡", "Identical / Equivalent"),
        // Set Theory
        ("elem", "∈", "Element of"),
        ("notelem", "∉", "Not element of"),
        ("subset", "⊂", "Proper subset"),
        ("subseteq", "⊆", "Subset or equal"),
        ("supset", "⊃", "Proper superset"),
        ("supseteq", "⊇", "Superset or equal"),
        ("union", "∪", "Set union"),
        ("intersect", "∩", "Set intersection"),
        ("emptyset", "∅", "Empty set"),
        // Calculus & Analysis
        ("nabla", "∇", "Gradient / Del operator"),
        ("partial", "∂", "Partial derivative"),
        ("integral", "∫", "Integral"),
        ("infinity", "∞", "Infinity"),
        ("inf", "∞", "Infinity (short)"),
        ("sqrt", "√", "Square root"),
        // Products & Sums
        ("times", "×", "Cross product / multiplication"),
        ("cdot", "·", "Dot product"),
        ("tensor", "⊗", "Tensor product"),
        ("oplus", "⊕", "Direct sum"),
        ("compose", "∘", "Function composition"),
        ("sum", "∑", "Summation"),
        ("prod", "∏", "Product"),
        // Number Sets
        ("Nat", "ℕ", "Natural numbers"),
        ("Int", "ℤ", "Integers"),
        ("Rat", "ℚ", "Rational numbers"),
        ("Real", "ℝ", "Real numbers"),
        ("Complex", "ℂ", "Complex numbers"),
        ("Bool", "𝔹", "Boolean"),
        // Matrix/Linear Algebra
        ("transpose", "ᵀ", "Matrix transpose"),
        ("dagger", "†", "Hermitian adjoint"),
        ("det", "det", "Determinant"),
        // Greek (also available via their names)
        ("Alpha", "Α", "Greek capital Alpha"),
        ("Beta", "Β", "Greek capital Beta"),
        ("Gamma", "Γ", "Greek capital Gamma"),
        ("Delta", "Δ", "Greek capital Delta"),
        ("Epsilon", "Ε", "Greek capital Epsilon"),
        ("Zeta", "Ζ", "Greek capital Zeta"),
        ("Eta", "Η", "Greek capital Eta"),
        ("Theta", "Θ", "Greek capital Theta"),
        ("Iota", "Ι", "Greek capital Iota"),
        ("Kappa", "Κ", "Greek capital Kappa"),
        ("Lambda", "Λ", "Greek capital Lambda"),
        ("Mu", "Μ", "Greek capital Mu"),
        ("Nu", "Ν", "Greek capital Nu"),
        ("Xi", "Ξ", "Greek capital Xi"),
        ("Omicron", "Ο", "Greek capital Omicron"),
        ("Pi", "Π", "Greek capital Pi"),
        ("Rho", "Ρ", "Greek capital Rho"),
        ("Sigma", "Σ", "Greek capital Sigma"),
        ("Tau", "Τ", "Greek capital Tau"),
        ("Upsilon", "Υ", "Greek capital Upsilon"),
        ("Phi", "Φ", "Greek capital Phi"),
        ("Chi", "Χ", "Greek capital Chi"),
        ("Psi", "Ψ", "Greek capital Psi"),
        ("Omega", "Ω", "Greek capital Omega"),
    ];

    for (ascii, unicode, description) in ascii_to_unicode {
        items.push(CompletionItem {
            label: ascii.to_string(),
            label_details: Some(CompletionItemLabelDetails {
                detail: Some(format!(" → {}", unicode)),
                description: None,
            }),
            kind: Some(CompletionItemKind::TEXT),
            detail: Some(format!("{} (inserts {})", description, unicode)),
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "**Unicode Input Helper**\n\n\
                     Type `{}` to insert `{}`\n\n\
                     {}",
                    ascii, unicode, description
                ),
            })),
            insert_text: Some(unicode.to_string()),
            filter_text: Some(ascii.to_string()),
            sort_text: Some(format!("1_{}", ascii)), // Sort before other items
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

/// Run LSP server over stdio
pub fn run_stdio() -> std::result::Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run_stdio_async(None));
    Ok(())
}

/// Run LSP server over stdio with shared context
pub fn run_stdio_with_context(
    ctx: SharedContext,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run_stdio_async(Some(ctx)));
    Ok(())
}

async fn run_stdio_async(ctx: Option<SharedContext>) {
    // Set up stdin/stdout for LSP communication
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = if let Some(ctx) = ctx {
        LspService::new(move |client| KleisLanguageServer::with_context(client, ctx.clone()))
    } else {
        LspService::new(KleisLanguageServer::new)
    };
    Server::new(stdin, stdout, socket).serve(service).await;
}
