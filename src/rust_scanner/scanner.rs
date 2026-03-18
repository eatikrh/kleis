//! Rust structural scanner that emits Kleis AST.
//!
//! A hand-written, zero-dependency tokenizer + recursive descent parser for
//! Rust source code.  Produces the same Expression types as the Kleis-native
//! `scan()` in `rust_parser.kleis`.
//!
//! Grammar reference: IntelliJ Rust BNF (MIT license)
//! https://github.com/intellij-rust/intellij-rust/blob/master/src/main/grammars/RustParser.bnf

use crate::ast::Expression;

// ===========================================================================
// Public entry point
// ===========================================================================

/// Parse Rust source and return a Kleis `Crate(items, comments, line_count)`.
///
/// Handles both real newlines and the escaped two-char `\n` sequence that Kleis
/// string literals produce (same auto-detection as the `foldLines` builtin).
pub fn scan_rust(source: &str) -> Result<Expression, String> {
    let normalized;
    let src = if !source.contains('\n') && source.contains("\\n") {
        normalized = source.replace("\\n", "\n");
        &normalized
    } else {
        source
    };
    let tokens = tokenize(src);
    let line_count = src.lines().count();
    let mut parser = Parser::new(&tokens, src);
    parser.parse_crate();

    let items: Vec<Expression> = parser.items.into_iter().map(|i| i.to_expr()).collect();
    let comments: Vec<Expression> = parser.comments.into_iter().map(|c| c.to_expr()).collect();

    Ok(mk_op(
        "Crate",
        vec![mk_list(items), mk_list(comments), mk_int(line_count as i64)],
    ))
}

// ===========================================================================
// Tokens
// ===========================================================================

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    // Keywords
    Fn,
    Pub,
    Crate,
    Mod,
    Struct,
    Enum,
    Trait,
    Impl,
    Use,
    Const,
    Static,
    Type,
    Let,
    Unsafe,
    Async,
    Extern,
    For,
    Where,
    As,
    In,
    Self_,
    Super,
    Macro,
    MacroRules,
    Union,
    Auto,

    // Punctuation
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Semi,
    Comma,
    Colon,
    ColonColon,
    Hash,
    Bang,
    Arrow,    // ->
    FatArrow, // =>
    Eq,
    Star,
    Amp,
    Dot,
    DotDot,
    Lt,
    Gt,
    Question,

    // Literals & identifiers
    Ident(String),
    StringLit,
    CharLit,
    NumberLit,
    Lifetime,

    // Comments (kept as tokens so we can extract them)
    LineComment(String),
    OuterLineDoc(String),
    InnerLineDoc(String),
    BlockComment(String),
    OuterBlockDoc(String),
    InnerBlockDoc(String),

    // Attribute brackets (the content between `[` and `]` after `#` or `#!`)
    // We don't tokenize these specially; attributes are parsed from Hash + tokens.
    Eof,
    Unknown,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    line: usize, // 1-based
}

// ===========================================================================
// Tokenizer
// ===========================================================================

fn tokenize(source: &str) -> Vec<Token> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut tokens = Vec::new();
    let mut pos = 0;
    let mut line: usize = 1;

    while pos < len {
        let b = bytes[pos];

        // Newlines
        if b == b'\n' {
            line += 1;
            pos += 1;
            continue;
        }
        if b == b'\r' {
            pos += 1;
            if pos < len && bytes[pos] == b'\n' {
                pos += 1;
            }
            line += 1;
            continue;
        }

        // Whitespace
        if b == b' ' || b == b'\t' {
            pos += 1;
            continue;
        }

        // Comments and `/`
        if b == b'/' {
            if pos + 1 < len && bytes[pos + 1] == b'/' {
                let start = pos;
                // Line comment
                pos += 2;
                let comment_start = pos;
                while pos < len && bytes[pos] != b'\n' {
                    pos += 1;
                }
                let text: String = source[comment_start..pos].to_string();
                let full = &source[start..pos];

                if full.starts_with("///") && !full.starts_with("////") {
                    let doc_text = text
                        .strip_prefix('/')
                        .map(|s| s.to_string())
                        .unwrap_or(text);
                    tokens.push(Token {
                        kind: TokenKind::OuterLineDoc(doc_text),
                        line,
                    });
                } else if full.starts_with("//!") {
                    let doc_text = text
                        .strip_prefix('!')
                        .map(|s| s.to_string())
                        .unwrap_or(text);
                    tokens.push(Token {
                        kind: TokenKind::InnerLineDoc(doc_text),
                        line,
                    });
                } else {
                    tokens.push(Token {
                        kind: TokenKind::LineComment(text),
                        line,
                    });
                }
                continue;
            }
            if pos + 1 < len && bytes[pos + 1] == b'*' {
                let start_line = line;
                pos += 2;
                let content_start = pos;
                let mut depth = 1;
                while pos < len && depth > 0 {
                    if bytes[pos] == b'\n' {
                        line += 1;
                    }
                    if pos + 1 < len && bytes[pos] == b'/' && bytes[pos + 1] == b'*' {
                        depth += 1;
                        pos += 2;
                        continue;
                    }
                    if pos + 1 < len && bytes[pos] == b'*' && bytes[pos + 1] == b'/' {
                        depth -= 1;
                        if depth == 0 {
                            let text = source[content_start..pos].to_string();
                            // Check the opening: /** or /*! or /*
                            let open =
                                &source[content_start.saturating_sub(2)..content_start.min(len)];
                            if open.starts_with("**") && !open.starts_with("**/") {
                                let doc_text = text.strip_prefix('*').unwrap_or(&text).to_string();
                                tokens.push(Token {
                                    kind: TokenKind::OuterBlockDoc(doc_text),
                                    line: start_line,
                                });
                            } else if open.starts_with("*!") {
                                let doc_text = text.strip_prefix('!').unwrap_or(&text).to_string();
                                tokens.push(Token {
                                    kind: TokenKind::InnerBlockDoc(doc_text),
                                    line: start_line,
                                });
                            } else {
                                tokens.push(Token {
                                    kind: TokenKind::BlockComment(text),
                                    line: start_line,
                                });
                            }
                        }
                        pos += 2;
                        continue;
                    }
                    pos += 1;
                }
                continue;
            }
            // Bare `/` or `/=`
            pos += 1;
            continue;
        }

        // String literals
        if b == b'"' {
            pos = skip_string(bytes, pos, &mut line);
            tokens.push(Token {
                kind: TokenKind::StringLit,
                line,
            });
            continue;
        }

        // Raw string literals: r"..." or r#"..."#
        if b == b'r' && pos + 1 < len && (bytes[pos + 1] == b'"' || bytes[pos + 1] == b'#') {
            pos = skip_raw_string(bytes, pos, &mut line);
            tokens.push(Token {
                kind: TokenKind::StringLit,
                line,
            });
            continue;
        }

        // Byte string literals: b"..." or b'...'
        if b == b'b' && pos + 1 < len && (bytes[pos + 1] == b'"' || bytes[pos + 1] == b'\'') {
            if bytes[pos + 1] == b'"' {
                pos = skip_string(bytes, pos + 1, &mut line);
            } else {
                pos = skip_char_lit(bytes, pos + 1);
            }
            tokens.push(Token {
                kind: TokenKind::StringLit,
                line,
            });
            continue;
        }

        // br"..." raw byte strings
        if b == b'b'
            && pos + 2 < len
            && bytes[pos + 1] == b'r'
            && (bytes[pos + 2] == b'"' || bytes[pos + 2] == b'#')
        {
            pos = skip_raw_string(bytes, pos + 1, &mut line);
            tokens.push(Token {
                kind: TokenKind::StringLit,
                line,
            });
            continue;
        }

        // Char literals
        if b == b'\'' {
            // Distinguish lifetime 'a from char literal 'x'
            if pos + 2 < len && is_ident_start(bytes[pos + 1]) && bytes[pos + 2] != b'\'' {
                // Lifetime
                pos += 1;
                let start = pos;
                while pos < len && is_ident_continue(bytes[pos]) {
                    pos += 1;
                }
                let _name = &source[start..pos];
                tokens.push(Token {
                    kind: TokenKind::Lifetime,
                    line,
                });
                continue;
            }
            pos = skip_char_lit(bytes, pos);
            tokens.push(Token {
                kind: TokenKind::CharLit,
                line,
            });
            continue;
        }

        // Numbers
        if b.is_ascii_digit() {
            while pos < len
                && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_' || bytes[pos] == b'.')
            {
                if bytes[pos] == b'.' && pos + 1 < len && bytes[pos + 1] == b'.' {
                    break; // `..` range operator
                }
                pos += 1;
            }
            tokens.push(Token {
                kind: TokenKind::NumberLit,
                line,
            });
            continue;
        }

        // Identifiers and keywords
        if is_ident_start(b) {
            let start = pos;
            pos += 1;
            while pos < len && is_ident_continue(bytes[pos]) {
                pos += 1;
            }
            let word = &source[start..pos];
            let kind = match word {
                "fn" => TokenKind::Fn,
                "pub" => TokenKind::Pub,
                "crate" => TokenKind::Crate,
                "mod" => TokenKind::Mod,
                "struct" => TokenKind::Struct,
                "enum" => TokenKind::Enum,
                "trait" => TokenKind::Trait,
                "impl" => TokenKind::Impl,
                "use" => TokenKind::Use,
                "const" => TokenKind::Const,
                "static" => TokenKind::Static,
                "type" => TokenKind::Type,
                "let" => TokenKind::Let,
                "unsafe" => TokenKind::Unsafe,
                "async" => TokenKind::Async,
                "extern" => TokenKind::Extern,
                "for" => TokenKind::For,
                "where" => TokenKind::Where,
                "as" => TokenKind::As,
                "in" => TokenKind::In,
                "self" => TokenKind::Self_,
                "super" => TokenKind::Super,
                "Self" => TokenKind::Ident("Self".to_string()),
                "macro_rules" => TokenKind::MacroRules,
                "macro" => TokenKind::Macro,
                "union" => TokenKind::Union,
                "auto" => TokenKind::Auto,
                _ => TokenKind::Ident(word.to_string()),
            };
            tokens.push(Token { kind, line });
            continue;
        }

        // Multi-char punctuation
        if b == b'-' && pos + 1 < len && bytes[pos + 1] == b'>' {
            tokens.push(Token {
                kind: TokenKind::Arrow,
                line,
            });
            pos += 2;
            continue;
        }
        if b == b'=' && pos + 1 < len && bytes[pos + 1] == b'>' {
            tokens.push(Token {
                kind: TokenKind::FatArrow,
                line,
            });
            pos += 2;
            continue;
        }
        if b == b':' && pos + 1 < len && bytes[pos + 1] == b':' {
            tokens.push(Token {
                kind: TokenKind::ColonColon,
                line,
            });
            pos += 2;
            continue;
        }
        if b == b'.' && pos + 1 < len && bytes[pos + 1] == b'.' {
            tokens.push(Token {
                kind: TokenKind::DotDot,
                line,
            });
            pos += 2;
            if pos < len && bytes[pos] == b'.' {
                pos += 1; // ...
            }
            if pos < len && bytes[pos] == b'=' {
                pos += 1; // ..=
            }
            continue;
        }

        // Single-char punctuation
        let kind = match b {
            b'{' => TokenKind::LBrace,
            b'}' => TokenKind::RBrace,
            b'(' => TokenKind::LParen,
            b')' => TokenKind::RParen,
            b'[' => TokenKind::LBracket,
            b']' => TokenKind::RBracket,
            b';' => TokenKind::Semi,
            b',' => TokenKind::Comma,
            b':' => TokenKind::Colon,
            b'#' => TokenKind::Hash,
            b'!' => TokenKind::Bang,
            b'=' => TokenKind::Eq,
            b'*' => TokenKind::Star,
            b'&' => TokenKind::Amp,
            b'.' => TokenKind::Dot,
            b'<' => TokenKind::Lt,
            b'>' => TokenKind::Gt,
            b'?' => TokenKind::Question,
            _ => TokenKind::Unknown,
        };
        tokens.push(Token { kind, line });
        pos += 1;
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        line,
    });
    tokens
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn skip_string(bytes: &[u8], start: usize, line: &mut usize) -> usize {
    let mut pos = start + 1; // skip opening "
    while pos < bytes.len() {
        if bytes[pos] == b'\n' {
            *line += 1;
        }
        if bytes[pos] == b'\\' {
            pos += 2;
            continue;
        }
        if bytes[pos] == b'"' {
            pos += 1;
            break;
        }
        pos += 1;
    }
    pos
}

fn skip_char_lit(bytes: &[u8], start: usize) -> usize {
    let mut pos = start + 1; // skip opening '
    while pos < bytes.len() {
        if bytes[pos] == b'\\' {
            pos += 2;
            continue;
        }
        if bytes[pos] == b'\'' {
            pos += 1;
            break;
        }
        pos += 1;
    }
    pos
}

fn skip_raw_string(bytes: &[u8], start: usize, line: &mut usize) -> usize {
    let mut pos = start + 1; // skip 'r'
    let mut hashes = 0;
    while pos < bytes.len() && bytes[pos] == b'#' {
        hashes += 1;
        pos += 1;
    }
    if pos < bytes.len() && bytes[pos] == b'"' {
        pos += 1; // skip opening "
    }
    // Read until closing " followed by `hashes` #s
    while pos < bytes.len() {
        if bytes[pos] == b'\n' {
            *line += 1;
        }
        if bytes[pos] == b'"' {
            let mut matching = 0;
            let mut check = pos + 1;
            while check < bytes.len() && bytes[check] == b'#' && matching < hashes {
                matching += 1;
                check += 1;
            }
            if matching == hashes {
                pos = check;
                return pos;
            }
        }
        pos += 1;
    }
    pos
}

// ===========================================================================
// Internal AST
// ===========================================================================

#[derive(Debug, Clone)]
enum Visibility {
    Private,
    Pub,
    PubCrate,
    PubSuper,
    PubSelf,
    PubIn(String),
}

#[derive(Debug, Clone)]
enum CommentNode {
    LineComment(String, usize),
    OuterLineDoc(String, usize),
    InnerLineDoc(String, usize),
    BlockComment(String, usize),
    OuterBlockDoc(String, usize),
    InnerBlockDoc(String, usize),
}

#[derive(Debug, Clone)]
struct Attribute {
    is_inner: bool,
    content: String,
    line: usize,
}

#[derive(Debug, Clone)]
enum UseTree {
    Path(String),
    Wildcard(String),
    Group(String, Vec<String>),
    Alias(String, String),
}

#[derive(Debug, Clone)]
struct FnDecl {
    vis: Visibility,
    is_async: bool,
    is_const: bool,
    is_unsafe: bool,
    name: String,
    params_text: String,
    return_type: String,
    body_line_count: usize,
    max_nesting: usize,
    attrs: Vec<Attribute>,
    line: usize,
}

#[derive(Debug, Clone)]
struct ModDecl {
    vis: Visibility,
    is_unsafe: bool,
    name: String,
    is_inline: bool,
    attrs: Vec<Attribute>,
    line: usize,
}

#[derive(Debug, Clone)]
struct StructDecl {
    vis: Visibility,
    name: String,
    line: usize,
}

#[derive(Debug, Clone)]
struct EnumDecl {
    vis: Visibility,
    name: String,
    line: usize,
}

#[derive(Debug, Clone)]
struct TraitDecl {
    vis: Visibility,
    name: String,
    is_unsafe: bool,
    line: usize,
}

#[derive(Debug, Clone)]
enum ImplBlock {
    Inherent {
        target: String,
        line: usize,
    },
    Trait {
        trait_name: String,
        target: String,
        line: usize,
    },
}

#[derive(Debug, Clone)]
enum Item {
    Use(Visibility, UseTree, usize),
    Fn(FnDecl),
    Mod(ModDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Trait(TraitDecl),
    Impl(ImplBlock),
    Const(String, Visibility, usize),
    Static(String, Visibility, usize),
    TypeAlias(String, Visibility, usize),
    MacroCall(String, usize),
}

// ===========================================================================
// Conversion to Kleis Expression
// ===========================================================================

impl Visibility {
    fn to_expr(&self) -> Expression {
        match self {
            Visibility::Private => mk_op("Private", vec![]),
            Visibility::Pub => mk_op("Pub", vec![]),
            Visibility::PubCrate => mk_op("PubCrate", vec![]),
            Visibility::PubSuper => mk_op("PubSuper", vec![]),
            Visibility::PubSelf => mk_op("PubSelf", vec![]),
            Visibility::PubIn(path) => mk_op("PubIn", vec![Expression::String(path.clone())]),
        }
    }
}

impl CommentNode {
    fn to_expr(&self) -> Expression {
        match self {
            CommentNode::LineComment(t, l) => mk_op(
                "LineComment",
                vec![Expression::String(t.clone()), mk_int(*l as i64)],
            ),
            CommentNode::OuterLineDoc(t, l) => mk_op(
                "OuterLineDoc",
                vec![Expression::String(t.clone()), mk_int(*l as i64)],
            ),
            CommentNode::InnerLineDoc(t, l) => mk_op(
                "InnerLineDoc",
                vec![Expression::String(t.clone()), mk_int(*l as i64)],
            ),
            CommentNode::BlockComment(t, l) => mk_op(
                "BlockComment",
                vec![Expression::String(t.clone()), mk_int(*l as i64)],
            ),
            CommentNode::OuterBlockDoc(t, l) => mk_op(
                "OuterBlockDoc",
                vec![Expression::String(t.clone()), mk_int(*l as i64)],
            ),
            CommentNode::InnerBlockDoc(t, l) => mk_op(
                "InnerBlockDoc",
                vec![Expression::String(t.clone()), mk_int(*l as i64)],
            ),
        }
    }
}

impl Attribute {
    fn to_expr(&self) -> Expression {
        if self.is_inner {
            mk_op(
                "InnerAttr",
                vec![
                    Expression::String(self.content.clone()),
                    mk_int(self.line as i64),
                ],
            )
        } else {
            mk_op(
                "OuterAttr",
                vec![
                    Expression::String(self.content.clone()),
                    mk_int(self.line as i64),
                ],
            )
        }
    }
}

impl UseTree {
    fn to_expr(&self) -> Expression {
        match self {
            UseTree::Path(p) => mk_op("UsePath", vec![Expression::String(p.clone())]),
            UseTree::Wildcard(p) => mk_op("UseWildcard", vec![Expression::String(p.clone())]),
            UseTree::Group(prefix, items) => mk_op(
                "UseGroup",
                vec![
                    Expression::String(prefix.clone()),
                    mk_list(
                        items
                            .iter()
                            .map(|i| Expression::String(i.clone()))
                            .collect(),
                    ),
                ],
            ),
            UseTree::Alias(path, alias) => mk_op(
                "UseAlias",
                vec![
                    Expression::String(path.clone()),
                    Expression::String(alias.clone()),
                ],
            ),
        }
    }
}

impl FnDecl {
    fn to_expr(&self) -> Expression {
        mk_op(
            "FnDecl",
            vec![
                self.vis.to_expr(),
                mk_bool(self.is_async),
                mk_bool(self.is_const),
                mk_bool(self.is_unsafe),
                Expression::String(self.name.clone()),
                Expression::String(self.params_text.clone()),
                Expression::String(self.return_type.clone()),
                mk_int(self.body_line_count as i64),
                mk_int(self.max_nesting as i64),
                mk_list(self.attrs.iter().map(|a| a.to_expr()).collect()),
                mk_int(self.line as i64),
            ],
        )
    }
}

impl Item {
    fn to_expr(&self) -> Expression {
        match self {
            Item::Use(vis, tree, line) => {
                let decl = mk_op(
                    "UseDecl",
                    vec![vis.to_expr(), tree.to_expr(), mk_int(*line as i64)],
                );
                mk_op("ItemUse", vec![decl])
            }
            Item::Fn(f) => mk_op("ItemFn", vec![f.to_expr()]),
            Item::Mod(m) => {
                let decl = mk_op(
                    "ModDecl",
                    vec![
                        m.vis.to_expr(),
                        mk_bool(m.is_unsafe),
                        Expression::String(m.name.clone()),
                        mk_bool(m.is_inline),
                        mk_list(m.attrs.iter().map(|a| a.to_expr()).collect()),
                        mk_int(m.line as i64),
                    ],
                );
                mk_op("ItemMod", vec![decl])
            }
            Item::Struct(s) => {
                let decl = mk_op(
                    "Struct",
                    vec![
                        s.vis.to_expr(),
                        Expression::String(s.name.clone()),
                        mk_int(s.line as i64),
                    ],
                );
                mk_op("ItemStruct", vec![decl])
            }
            Item::Enum(e) => {
                let decl = mk_op(
                    "Enum",
                    vec![
                        e.vis.to_expr(),
                        Expression::String(e.name.clone()),
                        mk_int(e.line as i64),
                    ],
                );
                mk_op("ItemEnum", vec![decl])
            }
            Item::Trait(t) => {
                let decl = mk_op(
                    "Trait",
                    vec![
                        t.vis.to_expr(),
                        Expression::String(t.name.clone()),
                        mk_bool(t.is_unsafe),
                        mk_int(t.line as i64),
                    ],
                );
                mk_op("ItemTrait", vec![decl])
            }
            Item::Impl(ib) => {
                let decl = match ib {
                    ImplBlock::Inherent { target, line } => mk_op(
                        "InherentImpl",
                        vec![Expression::String(target.clone()), mk_int(*line as i64)],
                    ),
                    ImplBlock::Trait {
                        trait_name,
                        target,
                        line,
                    } => mk_op(
                        "TraitImpl",
                        vec![
                            Expression::String(trait_name.clone()),
                            Expression::String(target.clone()),
                            mk_int(*line as i64),
                        ],
                    ),
                };
                mk_op("ItemImpl", vec![decl])
            }
            Item::Const(name, vis, line) => mk_op(
                "ItemConst",
                vec![
                    Expression::String(name.clone()),
                    vis.to_expr(),
                    mk_int(*line as i64),
                ],
            ),
            Item::Static(name, vis, line) => mk_op(
                "ItemStatic",
                vec![
                    Expression::String(name.clone()),
                    vis.to_expr(),
                    mk_int(*line as i64),
                ],
            ),
            Item::TypeAlias(name, vis, line) => mk_op(
                "ItemType",
                vec![
                    Expression::String(name.clone()),
                    vis.to_expr(),
                    mk_int(*line as i64),
                ],
            ),
            Item::MacroCall(name, line) => mk_op(
                "ItemMacroCall",
                vec![Expression::String(name.clone()), mk_int(*line as i64)],
            ),
        }
    }
}

// ===========================================================================
// Expression helpers (same pattern as Python scanner)
// ===========================================================================

fn mk_op(name: &str, args: Vec<Expression>) -> Expression {
    Expression::Operation {
        name: name.to_string(),
        args,
        span: None,
    }
}

fn mk_int(n: i64) -> Expression {
    Expression::Const(n.to_string())
}

fn mk_bool(b: bool) -> Expression {
    Expression::Object(if b { "true" } else { "false" }.to_string())
}

fn mk_list(items: Vec<Expression>) -> Expression {
    items
        .into_iter()
        .rev()
        .fold(mk_op("Nil", vec![]), |acc, item| {
            mk_op("Cons", vec![item, acc])
        })
}

// ===========================================================================
// Parser
// ===========================================================================

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    #[allow(dead_code)]
    source: &'a str,
    items: Vec<Item>,
    comments: Vec<CommentNode>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token], source: &'a str) -> Self {
        Parser {
            tokens,
            pos: 0,
            source,
            items: Vec::new(),
            comments: Vec::new(),
        }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token {
            kind: TokenKind::Eof,
            line: 0,
        })
    }

    // We need a static fallback for EOF; use a thread-local or just inline it.
    // Actually, current() returns a reference. Let's fix this by keeping an EOF sentinel.

    fn peek_kind(&self) -> &TokenKind {
        &self.current().kind
    }

    fn peek_line(&self) -> usize {
        self.current().line
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }

    fn expect(&mut self, kind: &TokenKind) -> bool {
        if self.peek_kind() == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn eat_ident(&mut self) -> Option<String> {
        match self.peek_kind() {
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();
                Some(name)
            }
            // Contextual keywords that can appear as identifiers
            TokenKind::Union => {
                self.advance();
                Some("union".to_string())
            }
            TokenKind::Auto => {
                self.advance();
                Some("auto".to_string())
            }
            _ => None,
        }
    }

    // -----------------------------------------------------------------------
    // Top-level parsing
    // -----------------------------------------------------------------------

    fn parse_crate(&mut self) {
        while !self.at_eof() {
            self.skip_comments();
            if self.at_eof() {
                break;
            }
            if !self.parse_item() {
                // Recovery: skip one token
                self.advance();
            }
        }
    }

    fn skip_comments(&mut self) {
        loop {
            match self.peek_kind() {
                TokenKind::LineComment(text) => {
                    self.comments
                        .push(CommentNode::LineComment(text.clone(), self.peek_line()));
                    self.advance();
                }
                TokenKind::OuterLineDoc(text) => {
                    self.comments
                        .push(CommentNode::OuterLineDoc(text.clone(), self.peek_line()));
                    self.advance();
                }
                TokenKind::InnerLineDoc(text) => {
                    self.comments
                        .push(CommentNode::InnerLineDoc(text.clone(), self.peek_line()));
                    self.advance();
                }
                TokenKind::BlockComment(text) => {
                    self.comments
                        .push(CommentNode::BlockComment(text.clone(), self.peek_line()));
                    self.advance();
                }
                TokenKind::OuterBlockDoc(text) => {
                    self.comments
                        .push(CommentNode::OuterBlockDoc(text.clone(), self.peek_line()));
                    self.advance();
                }
                TokenKind::InnerBlockDoc(text) => {
                    self.comments
                        .push(CommentNode::InnerBlockDoc(text.clone(), self.peek_line()));
                    self.advance();
                }
                _ => break,
            }
        }
    }

    /// Parse one top-level item. Returns false if nothing was recognized.
    fn parse_item(&mut self) -> bool {
        self.skip_comments();
        if self.at_eof() {
            return false;
        }

        // Collect outer attributes
        let attrs = self.parse_outer_attrs();
        let item_line = self.peek_line();

        // Parse visibility
        let vis = self.parse_visibility();

        self.skip_comments();

        match self.peek_kind().clone() {
            TokenKind::Fn => {
                self.parse_fn(vis, false, false, false, &attrs, item_line);
                true
            }
            TokenKind::Async => {
                self.advance();
                self.skip_comments();
                if matches!(self.peek_kind(), TokenKind::Fn) {
                    self.parse_fn(vis, true, false, false, &attrs, item_line);
                } else {
                    self.skip_to_semi_or_brace();
                }
                true
            }
            TokenKind::Const => {
                self.advance();
                self.skip_comments();
                // `const fn` or `const IDENT`
                if matches!(self.peek_kind(), TokenKind::Fn) {
                    self.parse_fn(vis, false, true, false, &attrs, item_line);
                } else if matches!(self.peek_kind(), TokenKind::Unsafe) {
                    // const unsafe fn
                    self.advance();
                    self.skip_comments();
                    if matches!(self.peek_kind(), TokenKind::Fn) {
                        self.parse_fn(vis, false, true, true, &attrs, item_line);
                    } else {
                        self.skip_to_semi_or_brace();
                    }
                } else if let Some(name) = self.eat_ident() {
                    self.skip_to_semi_or_brace();
                    self.items.push(Item::Const(name, vis, item_line));
                } else if matches!(self.peek_kind(), TokenKind::Ident(_)) {
                    let name = if let TokenKind::Ident(n) = self.peek_kind() {
                        n.clone()
                    } else {
                        String::new()
                    };
                    self.advance();
                    self.skip_to_semi_or_brace();
                    self.items.push(Item::Const(name, vis, item_line));
                } else {
                    self.skip_to_semi_or_brace();
                }
                true
            }
            TokenKind::Unsafe => {
                self.advance();
                self.skip_comments();
                match self.peek_kind().clone() {
                    TokenKind::Fn => {
                        self.parse_fn(vis, false, false, true, &attrs, item_line);
                    }
                    TokenKind::Async => {
                        self.advance();
                        self.skip_comments();
                        if matches!(self.peek_kind(), TokenKind::Fn) {
                            self.parse_fn(vis, true, false, true, &attrs, item_line);
                        } else {
                            self.skip_to_semi_or_brace();
                        }
                    }
                    TokenKind::Trait => {
                        self.parse_trait(vis, true, &attrs, item_line);
                    }
                    TokenKind::Impl => {
                        self.parse_impl(true, item_line);
                    }
                    TokenKind::Mod => {
                        self.parse_mod(vis, true, &attrs, item_line);
                    }
                    TokenKind::Extern => {
                        self.advance();
                        self.skip_comments();
                        // unsafe extern "C" fn ...
                        if matches!(self.peek_kind(), TokenKind::StringLit) {
                            self.advance();
                        }
                        self.skip_comments();
                        if matches!(self.peek_kind(), TokenKind::Fn) {
                            self.parse_fn(vis, false, false, true, &attrs, item_line);
                        } else if matches!(self.peek_kind(), TokenKind::LBrace) {
                            self.skip_braced_block();
                        } else {
                            self.skip_to_semi_or_brace();
                        }
                    }
                    _ => {
                        self.skip_to_semi_or_brace();
                    }
                }
                true
            }
            TokenKind::Struct | TokenKind::Union => {
                let is_union = matches!(self.peek_kind(), TokenKind::Union);
                self.advance();
                self.skip_comments();
                if let Some(name) = self.eat_ident() {
                    self.skip_generics();
                    self.skip_where_clause();
                    // Could be followed by { fields }, ( tuple ), or ;
                    if matches!(self.peek_kind(), TokenKind::LBrace) {
                        self.skip_braced_block();
                    } else if matches!(self.peek_kind(), TokenKind::LParen) {
                        self.skip_parens();
                        self.skip_where_clause();
                        self.expect(&TokenKind::Semi);
                    } else {
                        self.expect(&TokenKind::Semi);
                    }
                    let label = if is_union {
                        format!("union {}", name)
                    } else {
                        name
                    };
                    self.items.push(Item::Struct(StructDecl {
                        vis,
                        name: label,
                        line: item_line,
                    }));
                } else {
                    self.skip_to_semi_or_brace();
                }
                true
            }
            TokenKind::Enum => {
                self.advance();
                self.skip_comments();
                if let Some(name) = self.eat_ident() {
                    self.skip_generics();
                    self.skip_where_clause();
                    if matches!(self.peek_kind(), TokenKind::LBrace) {
                        self.skip_braced_block();
                    }
                    self.items.push(Item::Enum(EnumDecl {
                        vis,
                        name,
                        line: item_line,
                    }));
                } else {
                    self.skip_to_semi_or_brace();
                }
                true
            }
            TokenKind::Trait => {
                self.parse_trait(vis, false, &attrs, item_line);
                true
            }
            TokenKind::Impl => {
                self.parse_impl(false, item_line);
                true
            }
            TokenKind::Mod => {
                self.parse_mod(vis, false, &attrs, item_line);
                true
            }
            TokenKind::Use => {
                self.parse_use(vis, item_line);
                true
            }
            TokenKind::Static => {
                self.advance();
                self.skip_comments();
                // static mut?
                if matches!(self.peek_kind(), TokenKind::Ident(n) if n == "mut") {
                    self.advance();
                    self.skip_comments();
                }
                if let Some(name) = self.eat_ident() {
                    self.skip_to_semi_or_brace();
                    self.items.push(Item::Static(name, vis, item_line));
                } else {
                    self.skip_to_semi_or_brace();
                }
                true
            }
            TokenKind::Type => {
                self.advance();
                self.skip_comments();
                if let Some(name) = self.eat_ident() {
                    self.skip_to_semi_or_brace();
                    self.items.push(Item::TypeAlias(name, vis, item_line));
                } else {
                    self.skip_to_semi_or_brace();
                }
                true
            }
            TokenKind::Extern => {
                self.advance();
                self.skip_comments();
                // extern "C" fn, extern "C" { }, or extern crate
                if matches!(self.peek_kind(), TokenKind::Crate) {
                    self.advance();
                    self.skip_to_semi_or_brace();
                } else {
                    if matches!(self.peek_kind(), TokenKind::StringLit) {
                        self.advance();
                        self.skip_comments();
                    }
                    if matches!(self.peek_kind(), TokenKind::Fn) {
                        self.parse_fn(vis, false, false, false, &attrs, item_line);
                    } else if matches!(self.peek_kind(), TokenKind::LBrace) {
                        self.skip_braced_block();
                    } else {
                        self.skip_to_semi_or_brace();
                    }
                }
                true
            }
            TokenKind::MacroRules => {
                self.advance();
                self.skip_comments();
                self.expect(&TokenKind::Bang);
                self.skip_comments();
                let name = self.eat_ident().unwrap_or_default();
                self.skip_to_semi_or_brace();
                self.items.push(Item::MacroCall(name, item_line));
                true
            }
            TokenKind::Macro => {
                self.advance();
                self.skip_comments();
                let name = self.eat_ident().unwrap_or_default();
                self.skip_to_semi_or_brace();
                self.items.push(Item::MacroCall(name, item_line));
                true
            }
            TokenKind::Ident(ref name) => {
                let name = name.clone();
                // Could be a macro invocation: `ident!(...)`
                self.advance();
                self.skip_comments();
                if matches!(self.peek_kind(), TokenKind::Bang) {
                    self.advance();
                    self.skip_comments();
                    // Skip macro body
                    if matches!(self.peek_kind(), TokenKind::LBrace) {
                        self.skip_braced_block();
                    } else if matches!(self.peek_kind(), TokenKind::LParen) {
                        self.skip_parens();
                        self.expect(&TokenKind::Semi);
                    } else if matches!(self.peek_kind(), TokenKind::LBracket) {
                        self.skip_brackets();
                        self.expect(&TokenKind::Semi);
                    } else {
                        self.skip_to_semi_or_brace();
                    }
                    self.items.push(Item::MacroCall(name, item_line));
                    true
                } else {
                    // Unknown top-level construct; skip
                    self.skip_to_semi_or_brace();
                    true
                }
            }
            TokenKind::Auto => {
                // `auto trait`
                self.advance();
                self.skip_comments();
                if matches!(self.peek_kind(), TokenKind::Trait) {
                    self.parse_trait(vis, false, &attrs, item_line);
                } else {
                    self.skip_to_semi_or_brace();
                }
                true
            }
            TokenKind::Hash => {
                // Stray attribute at top level without an item following
                // (already consumed in parse_outer_attrs, so this shouldn't happen)
                self.advance();
                true
            }
            _ => {
                // Can't recognize this as an item
                if !attrs.is_empty() {
                    // We consumed attributes but no item followed
                    self.skip_to_semi_or_brace();
                    return true;
                }
                false
            }
        }
    }

    // -----------------------------------------------------------------------
    // Attributes
    // -----------------------------------------------------------------------

    fn parse_outer_attrs(&mut self) -> Vec<Attribute> {
        let mut attrs = Vec::new();
        loop {
            self.skip_comments();
            if !matches!(self.peek_kind(), TokenKind::Hash) {
                break;
            }
            // Check if it's `#![...]` (inner) or `#[...]` (outer)
            let attr_line = self.peek_line();
            // Peek ahead: is next token `!` ?
            let is_inner = self.pos + 1 < self.tokens.len()
                && matches!(self.tokens[self.pos + 1].kind, TokenKind::Bang);

            if is_inner {
                // Inner attribute — don't consume here (belongs to the module/crate)
                // Actually, inner attrs like #![...] at file top are valid.
                // Consume them.
                self.advance(); // #
                self.advance(); // !
                let content = self.parse_attr_content();
                attrs.push(Attribute {
                    is_inner: true,
                    content,
                    line: attr_line,
                });
            } else {
                // Check if next is `[` — if not, this `#` is something else
                if self.pos + 1 < self.tokens.len()
                    && matches!(self.tokens[self.pos + 1].kind, TokenKind::LBracket)
                {
                    self.advance(); // #
                    let content = self.parse_attr_content();
                    attrs.push(Attribute {
                        is_inner: false,
                        content,
                        line: attr_line,
                    });
                } else {
                    break;
                }
            }
        }
        attrs
    }

    fn parse_attr_content(&mut self) -> String {
        // Expect `[`, consume until matching `]`
        if !self.expect(&TokenKind::LBracket) {
            return String::new();
        }
        let mut depth = 1;
        let mut parts = Vec::new();
        while !self.at_eof() && depth > 0 {
            match self.peek_kind() {
                TokenKind::LBracket => {
                    depth += 1;
                    parts.push("[".to_string());
                    self.advance();
                }
                TokenKind::RBracket => {
                    depth -= 1;
                    if depth > 0 {
                        parts.push("]".to_string());
                    }
                    self.advance();
                }
                _ => {
                    parts.push(self.token_text());
                    self.advance();
                }
            }
        }
        parts.join("")
    }

    fn token_text(&self) -> String {
        match self.peek_kind() {
            TokenKind::Ident(n) => n.clone(),
            TokenKind::Fn => "fn".to_string(),
            TokenKind::Pub => "pub".to_string(),
            TokenKind::Crate => "crate".to_string(),
            TokenKind::Mod => "mod".to_string(),
            TokenKind::Struct => "struct".to_string(),
            TokenKind::Enum => "enum".to_string(),
            TokenKind::Trait => "trait".to_string(),
            TokenKind::Impl => "impl".to_string(),
            TokenKind::Use => "use".to_string(),
            TokenKind::Const => "const".to_string(),
            TokenKind::Static => "static".to_string(),
            TokenKind::Type => "type".to_string(),
            TokenKind::Let => "let".to_string(),
            TokenKind::Unsafe => "unsafe".to_string(),
            TokenKind::Async => "async".to_string(),
            TokenKind::Extern => "extern".to_string(),
            TokenKind::For => "for".to_string(),
            TokenKind::Where => "where".to_string(),
            TokenKind::As => "as".to_string(),
            TokenKind::In => "in".to_string(),
            TokenKind::Self_ => "self".to_string(),
            TokenKind::Super => "super".to_string(),
            TokenKind::Macro => "macro".to_string(),
            TokenKind::MacroRules => "macro_rules".to_string(),
            TokenKind::Union => "union".to_string(),
            TokenKind::Auto => "auto".to_string(),
            TokenKind::LBrace => "{".to_string(),
            TokenKind::RBrace => "}".to_string(),
            TokenKind::LParen => "(".to_string(),
            TokenKind::RParen => ")".to_string(),
            TokenKind::LBracket => "[".to_string(),
            TokenKind::RBracket => "]".to_string(),
            TokenKind::Semi => ";".to_string(),
            TokenKind::Comma => ",".to_string(),
            TokenKind::Colon => ":".to_string(),
            TokenKind::ColonColon => "::".to_string(),
            TokenKind::Hash => "#".to_string(),
            TokenKind::Bang => "!".to_string(),
            TokenKind::Arrow => "->".to_string(),
            TokenKind::FatArrow => "=>".to_string(),
            TokenKind::Eq => "=".to_string(),
            TokenKind::Star => "*".to_string(),
            TokenKind::Amp => "&".to_string(),
            TokenKind::Dot => ".".to_string(),
            TokenKind::DotDot => "..".to_string(),
            TokenKind::Lt => "<".to_string(),
            TokenKind::Gt => ">".to_string(),
            TokenKind::Question => "?".to_string(),
            TokenKind::StringLit => "\"...\"".to_string(),
            TokenKind::CharLit => "'...'".to_string(),
            TokenKind::NumberLit => "0".to_string(),
            TokenKind::Lifetime => "'_".to_string(),
            _ => "".to_string(),
        }
    }

    // -----------------------------------------------------------------------
    // Visibility
    // -----------------------------------------------------------------------

    fn parse_visibility(&mut self) -> Visibility {
        self.skip_comments();
        if !matches!(self.peek_kind(), TokenKind::Pub) {
            if matches!(self.peek_kind(), TokenKind::Crate) {
                // bare `crate` visibility (edition 2015)
                // Don't consume — `crate` could be `crate::` path start
                return Visibility::Private;
            }
            return Visibility::Private;
        }
        self.advance(); // pub

        // Check for pub(...)
        if !matches!(self.peek_kind(), TokenKind::LParen) {
            return Visibility::Pub;
        }
        self.advance(); // (
        self.skip_comments();

        let vis = match self.peek_kind() {
            TokenKind::Crate => {
                self.advance();
                Visibility::PubCrate
            }
            TokenKind::Super => {
                self.advance();
                Visibility::PubSuper
            }
            TokenKind::Self_ => {
                self.advance();
                Visibility::PubSelf
            }
            TokenKind::In => {
                self.advance();
                self.skip_comments();
                let mut path = String::new();
                while !self.at_eof() && !matches!(self.peek_kind(), TokenKind::RParen) {
                    path.push_str(&self.token_text());
                    self.advance();
                }
                Visibility::PubIn(path)
            }
            _ => Visibility::Pub, // malformed, treat as pub
        };

        self.expect(&TokenKind::RParen);
        vis
    }

    // -----------------------------------------------------------------------
    // Function
    // -----------------------------------------------------------------------

    fn parse_fn(
        &mut self,
        vis: Visibility,
        is_async: bool,
        is_const: bool,
        is_unsafe: bool,
        attrs: &[Attribute],
        item_line: usize,
    ) {
        self.advance(); // fn
        self.skip_comments();

        let name = self.eat_ident().unwrap_or_default();

        // Generic params
        self.skip_generics();
        self.skip_comments();

        // Parameters
        let params_text = if matches!(self.peek_kind(), TokenKind::LParen) {
            self.collect_parens_text()
        } else {
            String::new()
        };

        self.skip_comments();

        // Return type
        let return_type = if matches!(self.peek_kind(), TokenKind::Arrow) {
            self.advance(); // ->
            self.skip_comments();
            self.collect_type_text()
        } else {
            String::new()
        };

        // Where clause
        self.skip_where_clause();
        self.skip_comments();

        // Body or semicolon
        let (body_line_count, max_nesting) = if matches!(self.peek_kind(), TokenKind::LBrace) {
            self.measure_braced_block()
        } else {
            self.expect(&TokenKind::Semi);
            (0, 0)
        };

        self.items.push(Item::Fn(FnDecl {
            vis,
            is_async,
            is_const,
            is_unsafe,
            name,
            params_text,
            return_type,
            body_line_count,
            max_nesting,
            attrs: attrs.to_vec(),
            line: item_line,
        }));
    }

    /// Collect the text inside parentheses (for function parameters).
    fn collect_parens_text(&mut self) -> String {
        if !self.expect(&TokenKind::LParen) {
            return String::new();
        }
        let mut depth = 1;
        let mut parts = Vec::new();
        let mut last_was_space_needing = false;
        while !self.at_eof() && depth > 0 {
            match self.peek_kind() {
                TokenKind::LParen => {
                    depth += 1;
                    parts.push("(".to_string());
                    last_was_space_needing = false;
                    self.advance();
                }
                TokenKind::RParen => {
                    depth -= 1;
                    if depth > 0 {
                        parts.push(")".to_string());
                    }
                    last_was_space_needing = false;
                    self.advance();
                }
                TokenKind::Comma => {
                    parts.push(", ".to_string());
                    last_was_space_needing = false;
                    self.advance();
                }
                TokenKind::Colon => {
                    parts.push(": ".to_string());
                    last_was_space_needing = false;
                    self.advance();
                }
                _ => {
                    let text = self.token_text();
                    if last_was_space_needing && !text.is_empty() {
                        parts.push(" ".to_string());
                    }
                    parts.push(text);
                    last_was_space_needing = matches!(
                        self.peek_kind(),
                        TokenKind::Ident(_)
                            | TokenKind::Amp
                            | TokenKind::Star
                            | TokenKind::Self_
                            | TokenKind::Lifetime
                            | TokenKind::Fn
                            | TokenKind::Unsafe
                            | TokenKind::Extern
                            | TokenKind::Const
                    );
                    self.advance();
                }
            }
        }
        parts.join("")
    }

    /// Collect a type expression until we hit `{`, `;`, or `where`.
    fn collect_type_text(&mut self) -> String {
        let mut parts = Vec::new();
        let mut angle_depth = 0;
        loop {
            self.skip_comments();
            match self.peek_kind() {
                TokenKind::LBrace | TokenKind::Semi | TokenKind::Eof => break,
                TokenKind::Where if angle_depth == 0 => break,
                TokenKind::Lt => {
                    angle_depth += 1;
                    parts.push("<".to_string());
                    self.advance();
                }
                TokenKind::Gt => {
                    if angle_depth > 0 {
                        angle_depth -= 1;
                    } else {
                        break;
                    }
                    parts.push(">".to_string());
                    self.advance();
                }
                _ => {
                    parts.push(self.token_text());
                    self.advance();
                }
            }
        }
        let raw = parts.join("");
        // Normalize whitespace
        raw.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    // -----------------------------------------------------------------------
    // Trait
    // -----------------------------------------------------------------------

    fn parse_trait(
        &mut self,
        vis: Visibility,
        is_unsafe: bool,
        _attrs: &[Attribute],
        item_line: usize,
    ) {
        self.advance(); // trait
        self.skip_comments();
        let name = self.eat_ident().unwrap_or_default();
        self.skip_generics();
        // Supertraits: : Bound + Bound
        if matches!(self.peek_kind(), TokenKind::Colon) {
            self.advance();
            self.skip_until_brace_or_where();
        }
        self.skip_where_clause();
        if matches!(self.peek_kind(), TokenKind::LBrace) {
            self.skip_braced_block();
        }
        self.items.push(Item::Trait(TraitDecl {
            vis,
            name,
            is_unsafe,
            line: item_line,
        }));
    }

    // -----------------------------------------------------------------------
    // Impl
    // -----------------------------------------------------------------------

    fn parse_impl(&mut self, _is_unsafe: bool, item_line: usize) {
        self.advance(); // impl
        self.skip_comments();

        // Skip optional generics after `impl`
        self.skip_generics();
        self.skip_comments();

        // Collect the type/trait path until `for` or `{` or `where`
        let first_path = self.collect_impl_path();
        self.skip_comments();

        let impl_block = if matches!(self.peek_kind(), TokenKind::For) {
            self.advance(); // for
            self.skip_comments();
            let target = self.collect_impl_path();
            self.skip_comments();
            ImplBlock::Trait {
                trait_name: first_path,
                target,
                line: item_line,
            }
        } else {
            ImplBlock::Inherent {
                target: first_path,
                line: item_line,
            }
        };

        self.skip_where_clause();
        if matches!(self.peek_kind(), TokenKind::LBrace) {
            self.skip_braced_block();
        }

        self.items.push(Item::Impl(impl_block));
    }

    fn collect_impl_path(&mut self) -> String {
        let mut parts = Vec::new();
        let mut angle_depth = 0;
        loop {
            self.skip_comments();
            match self.peek_kind() {
                TokenKind::LBrace | TokenKind::Semi | TokenKind::Eof => break,
                TokenKind::For if angle_depth == 0 => break,
                TokenKind::Where if angle_depth == 0 => break,
                TokenKind::Lt => {
                    angle_depth += 1;
                    parts.push("<".to_string());
                    self.advance();
                }
                TokenKind::Gt => {
                    if angle_depth > 0 {
                        angle_depth -= 1;
                    }
                    parts.push(">".to_string());
                    self.advance();
                    if angle_depth == 0 {
                        break;
                    }
                }
                _ => {
                    parts.push(self.token_text());
                    self.advance();
                }
            }
        }
        parts.join("")
    }

    // -----------------------------------------------------------------------
    // Mod
    // -----------------------------------------------------------------------

    fn parse_mod(
        &mut self,
        vis: Visibility,
        is_unsafe: bool,
        attrs: &[Attribute],
        item_line: usize,
    ) {
        self.advance(); // mod
        self.skip_comments();
        let name = self.eat_ident().unwrap_or_default();
        self.skip_comments();

        let is_inline = if matches!(self.peek_kind(), TokenKind::LBrace) {
            self.skip_braced_block();
            true
        } else {
            self.expect(&TokenKind::Semi);
            false
        };

        self.items.push(Item::Mod(ModDecl {
            vis,
            is_unsafe,
            name,
            is_inline,
            attrs: attrs.to_vec(),
            line: item_line,
        }));
    }

    // -----------------------------------------------------------------------
    // Use
    // -----------------------------------------------------------------------

    fn parse_use(&mut self, vis: Visibility, item_line: usize) {
        self.advance(); // use
        self.skip_comments();

        let tree = self.parse_use_tree();
        self.expect(&TokenKind::Semi);
        self.items.push(Item::Use(vis, tree, item_line));
    }

    fn parse_use_tree(&mut self) -> UseTree {
        self.skip_comments();

        // Leading ::
        let mut path = String::new();
        if matches!(self.peek_kind(), TokenKind::ColonColon) {
            path.push_str("::");
            self.advance();
        }

        // Collect path segments: ident (:: ident)*
        loop {
            self.skip_comments();
            match self.peek_kind() {
                TokenKind::Ident(name) => {
                    path.push_str(&name.clone());
                    self.advance();
                }
                TokenKind::Self_ => {
                    path.push_str("self");
                    self.advance();
                }
                TokenKind::Super => {
                    path.push_str("super");
                    self.advance();
                }
                TokenKind::Crate => {
                    path.push_str("crate");
                    self.advance();
                }
                TokenKind::Star => {
                    // use foo::*
                    self.advance();
                    return UseTree::Wildcard(path);
                }
                TokenKind::LBrace => {
                    // use foo::{a, b}
                    return self.parse_use_group(path);
                }
                _ => break,
            }

            self.skip_comments();
            if matches!(self.peek_kind(), TokenKind::ColonColon) {
                path.push_str("::");
                self.advance();
            } else {
                break;
            }
        }

        // Check for `as alias`
        self.skip_comments();
        if matches!(self.peek_kind(), TokenKind::As) {
            self.advance();
            self.skip_comments();
            let alias = self.eat_ident().unwrap_or_default();
            return UseTree::Alias(path, alias);
        }

        UseTree::Path(path)
    }

    fn parse_use_group(&mut self, prefix: String) -> UseTree {
        self.advance(); // {
        let mut items = Vec::new();
        loop {
            self.skip_comments();
            if matches!(self.peek_kind(), TokenKind::RBrace) || self.at_eof() {
                break;
            }
            // Collect each item in the group as a simple string
            let mut item = String::new();
            loop {
                self.skip_comments();
                match self.peek_kind() {
                    TokenKind::Comma | TokenKind::RBrace | TokenKind::Eof => break,
                    _ => {
                        item.push_str(&self.token_text());
                        self.advance();
                    }
                }
            }
            if !item.is_empty() {
                items.push(item);
            }
            if matches!(self.peek_kind(), TokenKind::Comma) {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBrace);
        UseTree::Group(prefix, items)
    }

    // -----------------------------------------------------------------------
    // Skip helpers
    // -----------------------------------------------------------------------

    fn skip_braced_block(&mut self) -> (usize, usize) {
        if !self.expect(&TokenKind::LBrace) {
            return (0, 0);
        }
        let start_line = self.peek_line();
        let mut depth = 1;
        let mut max_depth = 1;
        while !self.at_eof() && depth > 0 {
            self.skip_comments();
            match self.peek_kind() {
                TokenKind::LBrace => {
                    depth += 1;
                    if depth > max_depth {
                        max_depth = depth;
                    }
                    self.advance();
                }
                TokenKind::RBrace => {
                    depth -= 1;
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
        let end_line = self
            .tokens
            .get(self.pos.saturating_sub(1))
            .map_or(0, |t| t.line);
        let line_count = if end_line >= start_line {
            end_line - start_line + 1
        } else {
            0
        };
        (line_count, max_depth)
    }

    fn measure_braced_block(&mut self) -> (usize, usize) {
        self.skip_braced_block()
    }

    fn skip_parens(&mut self) {
        if !self.expect(&TokenKind::LParen) {
            return;
        }
        let mut depth = 1;
        while !self.at_eof() && depth > 0 {
            match self.peek_kind() {
                TokenKind::LParen => {
                    depth += 1;
                    self.advance();
                }
                TokenKind::RParen => {
                    depth -= 1;
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn skip_brackets(&mut self) {
        if !self.expect(&TokenKind::LBracket) {
            return;
        }
        let mut depth = 1;
        while !self.at_eof() && depth > 0 {
            match self.peek_kind() {
                TokenKind::LBracket => {
                    depth += 1;
                    self.advance();
                }
                TokenKind::RBracket => {
                    depth -= 1;
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn skip_generics(&mut self) {
        self.skip_comments();
        if !matches!(self.peek_kind(), TokenKind::Lt) {
            return;
        }
        self.advance(); // <
        let mut depth = 1;
        while !self.at_eof() && depth > 0 {
            self.skip_comments();
            match self.peek_kind() {
                TokenKind::Lt => {
                    depth += 1;
                    self.advance();
                }
                TokenKind::Gt => {
                    depth -= 1;
                    self.advance();
                }
                // Handle >> as two >
                TokenKind::LBrace | TokenKind::Semi | TokenKind::Eof => break,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn skip_where_clause(&mut self) {
        self.skip_comments();
        if !matches!(self.peek_kind(), TokenKind::Where) {
            return;
        }
        self.advance(); // where
                        // Skip until `{` or `;`
        while !self.at_eof() {
            self.skip_comments();
            match self.peek_kind() {
                TokenKind::LBrace | TokenKind::Semi | TokenKind::Eof => break,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn skip_until_brace_or_where(&mut self) {
        while !self.at_eof() {
            self.skip_comments();
            match self.peek_kind() {
                TokenKind::LBrace | TokenKind::Where | TokenKind::Eof => break,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn skip_to_semi_or_brace(&mut self) {
        while !self.at_eof() {
            match self.peek_kind() {
                TokenKind::Semi => {
                    self.advance();
                    return;
                }
                TokenKind::LBrace => {
                    self.skip_braced_block();
                    return;
                }
                TokenKind::Eof => return,
                _ => {
                    self.advance();
                }
            }
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn cons_len(expr: &Expression) -> usize {
        match expr {
            Expression::Operation { name, args, .. } if name == "Cons" => 1 + cons_len(&args[1]),
            Expression::Operation { name, .. } if name == "Nil" => 0,
            _ => 0,
        }
    }

    fn get_crate_parts(expr: &Expression) -> (&Expression, &Expression, &Expression) {
        match expr {
            Expression::Operation { name, args, .. } if name == "Crate" => {
                (&args[0], &args[1], &args[2])
            }
            _ => panic!("Expected Crate"),
        }
    }

    fn item_name(expr: &Expression) -> String {
        match expr {
            Expression::Operation { name, args, .. } => {
                match name.as_str() {
                    "ItemFn" => {
                        // ItemFn(FnDecl(..., name, ...))
                        if let Expression::Operation { args: fn_args, .. } = &args[0] {
                            if let Expression::String(n) = &fn_args[4] {
                                return n.clone();
                            }
                        }
                        String::new()
                    }
                    "ItemStruct" => {
                        if let Expression::Operation { args: s_args, .. } = &args[0] {
                            if let Expression::String(n) = &s_args[1] {
                                return n.clone();
                            }
                        }
                        String::new()
                    }
                    "ItemEnum" => {
                        if let Expression::Operation { args: e_args, .. } = &args[0] {
                            if let Expression::String(n) = &e_args[1] {
                                return n.clone();
                            }
                        }
                        String::new()
                    }
                    "ItemUse" => "use".to_string(),
                    "ItemMod" => {
                        if let Expression::Operation { args: m_args, .. } = &args[0] {
                            if let Expression::String(n) = &m_args[2] {
                                return n.clone();
                            }
                        }
                        String::new()
                    }
                    "ItemTrait" => {
                        if let Expression::Operation { args: t_args, .. } = &args[0] {
                            if let Expression::String(n) = &t_args[1] {
                                return n.clone();
                            }
                        }
                        String::new()
                    }
                    "ItemImpl" => "impl".to_string(),
                    "ItemConst" => {
                        if let Expression::String(n) = &args[0] {
                            return n.clone();
                        }
                        String::new()
                    }
                    "ItemStatic" => {
                        if let Expression::String(n) = &args[0] {
                            return n.clone();
                        }
                        String::new()
                    }
                    "ItemType" => {
                        if let Expression::String(n) = &args[0] {
                            return n.clone();
                        }
                        String::new()
                    }
                    "ItemMacroCall" => {
                        if let Expression::String(n) = &args[0] {
                            return n.clone();
                        }
                        String::new()
                    }
                    _ => format!("?{}", name),
                }
            }
            _ => String::new(),
        }
    }

    fn collect_items(expr: &Expression) -> Vec<String> {
        let mut result = Vec::new();
        let mut cur = expr;
        loop {
            match cur {
                Expression::Operation { name, args, .. } if name == "Cons" => {
                    result.push(item_name(&args[0]));
                    cur = &args[1];
                }
                _ => break,
            }
        }
        result
    }

    #[test]
    fn test_simple_fn() {
        let source = "fn main() { println!(\"hello\"); }";
        let result = scan_rust(source).unwrap();
        let (items, _comments, _lines) = get_crate_parts(&result);
        assert_eq!(cons_len(items), 1);
        assert_eq!(collect_items(items), vec!["main"]);
    }

    #[test]
    fn test_pub_fn_with_return() {
        let source = "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}";
        let result = scan_rust(source).unwrap();
        let (items, _comments, _lines) = get_crate_parts(&result);
        assert_eq!(collect_items(items), vec!["add"]);
    }

    #[test]
    fn test_struct_and_impl() {
        let source = "pub struct Point {\n    x: f64,\n    y: f64,\n}\n\nimpl Point {\n    pub fn new(x: f64, y: f64) -> Self {\n        Point { x, y }\n    }\n}";
        let result = scan_rust(source).unwrap();
        let (items, _, _) = get_crate_parts(&result);
        let names = collect_items(items);
        assert_eq!(names, vec!["Point", "impl"]);
    }

    #[test]
    fn test_enum() {
        let source = "pub enum Color { Red, Green, Blue }";
        let result = scan_rust(source).unwrap();
        let (items, _, _) = get_crate_parts(&result);
        assert_eq!(collect_items(items), vec!["Color"]);
    }

    #[test]
    fn test_trait_and_impl() {
        let source = "pub trait Display {\n    fn fmt(&self) -> String;\n}\n\nimpl Display for Point {\n    fn fmt(&self) -> String {\n        format!(\"{},{}\", self.x, self.y)\n    }\n}";
        let result = scan_rust(source).unwrap();
        let (items, _, _) = get_crate_parts(&result);
        let names = collect_items(items);
        assert_eq!(names, vec!["Display", "impl"]);
    }

    #[test]
    fn test_use_declarations() {
        let source =
            "use std::io;\nuse std::collections::HashMap;\nuse std::io::*;\nuse super::utils;\n";
        let result = scan_rust(source).unwrap();
        let (items, _, _) = get_crate_parts(&result);
        assert_eq!(cons_len(items), 4);
    }

    #[test]
    fn test_comments() {
        let source = "// Regular comment\n/// Doc comment\n//! Inner doc\nfn main() {}\n";
        let result = scan_rust(source).unwrap();
        let (items, comments, _) = get_crate_parts(&result);
        assert_eq!(cons_len(items), 1);
        assert_eq!(cons_len(comments), 3);
    }

    #[test]
    fn test_attributes() {
        let source = "#[derive(Debug)]\n#[cfg(test)]\npub struct Foo;\n";
        let result = scan_rust(source).unwrap();
        let (items, _, _) = get_crate_parts(&result);
        assert_eq!(cons_len(items), 1);
    }

    #[test]
    fn test_async_unsafe_const() {
        let source = "async fn fetch() {}\nconst fn square(x: i32) -> i32 { x * x }\nunsafe fn danger() {}\n";
        let result = scan_rust(source).unwrap();
        let (items, _, _) = get_crate_parts(&result);
        let names = collect_items(items);
        assert_eq!(names, vec!["fetch", "square", "danger"]);
    }

    #[test]
    fn test_mod_declarations() {
        let source = "mod tests;\npub mod utils {\n    fn helper() {}\n}\n";
        let result = scan_rust(source).unwrap();
        let (items, _, _) = get_crate_parts(&result);
        let names = collect_items(items);
        assert_eq!(names, vec!["tests", "utils"]);
    }

    #[test]
    fn test_macro_rules() {
        let source = "macro_rules! my_macro { () => {} }";
        let result = scan_rust(source).unwrap();
        let (items, _, _) = get_crate_parts(&result);
        assert_eq!(collect_items(items), vec!["my_macro"]);
    }

    #[test]
    fn test_const_and_static() {
        let source = "const MAX: usize = 100;\nstatic mut COUNTER: u32 = 0;\n";
        let result = scan_rust(source).unwrap();
        let (items, _, _) = get_crate_parts(&result);
        let names = collect_items(items);
        assert_eq!(names, vec!["MAX", "COUNTER"]);
    }

    #[test]
    fn test_type_alias() {
        let source = "type Result<T> = std::result::Result<T, MyError>;";
        let result = scan_rust(source).unwrap();
        let (items, _, _) = get_crate_parts(&result);
        assert_eq!(collect_items(items), vec!["Result"]);
    }

    #[test]
    fn test_use_group() {
        let source = "use std::collections::{HashMap, BTreeMap};";
        let result = scan_rust(source).unwrap();
        let (items, _, _) = get_crate_parts(&result);
        assert_eq!(cons_len(items), 1);
    }

    #[test]
    fn test_raw_strings() {
        let source = r##"fn main() { let s = r#"hello"#; }"##;
        let result = scan_rust(source).unwrap();
        let (items, _, _) = get_crate_parts(&result);
        assert_eq!(collect_items(items), vec!["main"]);
    }

    #[test]
    fn test_body_line_count() {
        let source = "fn multi_line() {
    let a = 1;
    let b = 2;
    a + b
}
";
        let result = scan_rust(source).unwrap();
        let (items, _, _) = get_crate_parts(&result);
        if let Expression::Operation { name, args, .. } = items {
            if name == "Cons" {
                if let Expression::Operation {
                    name: item_name,
                    args: item_args,
                    ..
                } = &args[0]
                {
                    assert_eq!(item_name, "ItemFn");
                    if let Expression::Operation { args: fn_args, .. } = &item_args[0] {
                        if let Expression::Const(n) = &fn_args[7] {
                            let lines: i64 = n.parse().unwrap();
                            assert!(lines >= 4, "body_line_count should be >= 4, got {}", lines);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_cfg_test_module() {
        let source = "fn production() {
    compute();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it() {
        assert_eq!(1, 1);
    }
}
";
        let result = scan_rust(source).unwrap();
        let (items, _, _) = get_crate_parts(&result);
        let names = collect_items(items);
        assert_eq!(names, vec!["production", "tests"]);
    }

    #[test]
    fn test_nested_block_comments() {
        let source = "/* outer /* inner */ end */ fn main() {}";
        let result = scan_rust(source).unwrap();
        let (items, comments, _) = get_crate_parts(&result);
        assert_eq!(cons_len(items), 1);
        assert_eq!(cons_len(comments), 1);
    }

    #[test]
    fn test_visibility_variants() {
        let source = "pub fn a() {}
pub(crate) fn b() {}
pub(super) fn c() {}
fn d() {}
";
        let result = scan_rust(source).unwrap();
        let (items, _, _) = get_crate_parts(&result);
        assert_eq!(cons_len(items), 4);
    }
}
