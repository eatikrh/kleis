#![allow(deprecated)] // SymbolInformation::deprecated field is deprecated in lsp-types
//! Kleis Language Server Protocol (LSP) Implementation
//!
//! This provides IDE support for Kleis via the Language Server Protocol:
//! - Real-time diagnostics (parse errors, type errors)
//! - Hover information (type signatures)
//! - Go to definition
//! - Document symbols
//! - Semantic token highlighting
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
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use kleis::kleis_parser::{parse_kleis_program, KleisParseError};

/// Document state stored by the language server
struct Document {
    /// The document content as a rope (efficient for edits)
    content: Rope,
    /// The parsed AST (if parsing succeeded)
    #[allow(dead_code)]
    ast: Option<kleis::kleis_ast::Program>,
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

    /// Parse a document and return diagnostics
    fn parse_document(
        &self,
        _uri: &Url,
        text: &str,
    ) -> (Option<kleis::kleis_ast::Program>, Vec<Diagnostic>) {
        match parse_kleis_program(text) {
            Ok(program) => (Some(program), vec![]),
            Err(e) => {
                let diagnostic = self.error_to_diagnostic(&e, text);
                (None, vec![diagnostic])
            }
        }
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
        let (ast, diagnostics) = self.parse_document(&uri, text);

        // Store the document
        self.documents.insert(
            uri.clone(),
            Document {
                content: Rope::from_str(text),
                ast,
            },
        );

        // Send diagnostics to the client
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
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

        // TODO: Look up type information from the AST
        // For now, just show the word as a placeholder
        let hover_content = format!("**{}**\n\n_Type information coming soon_", word);

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

#[tokio::main]
async fn main() {
    // Set up stdin/stdout for LSP communication
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(KleisLanguageServer::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
