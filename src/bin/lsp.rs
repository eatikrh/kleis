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

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let completions = get_kleis_completions();
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
