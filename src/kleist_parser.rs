//! Kleist Template Parser - Parses .kleist files into template and palette definitions
//!
//! **Purpose:** Parse template definition files that externalize rendering templates
//! and palette layouts from Rust code.
//!
//! **File Format:**
//! ```kleist
//! @template plus {
//!     pattern: "plus(left, right)"
//!     unicode: "{left} + {right}"
//!     latex: "{left} + {right}"
//!     typst: "{left} + {right}"
//! }
//!
//! @palette {
//!     tab "Basics" {
//!         group "Arithmetic" {
//!             plus
//!             minus
//!         }
//!     }
//! }
//! ```
//!
//! **Grammar:** See docs/grammar/kleist_grammar.ebnf

use std::fmt;

// =============================================================================
// AST Types
// =============================================================================

/// A parsed .kleist file
#[derive(Debug, Clone, Default)]
pub struct KleistFile {
    pub templates: Vec<TemplateDefinition>,
    pub palette: Option<PaletteDefinition>,
}

/// A single @template block
#[derive(Debug, Clone)]
pub struct TemplateDefinition {
    pub name: String,
    pub pattern: Option<String>,
    pub unicode: Option<String>,
    pub latex: Option<String>,
    pub html: Option<String>,
    pub typst: Option<String>,
    pub kleis: Option<String>,
    pub category: Option<String>,
    pub shortcut: Option<String>,
    /// SVG for palette button (inline or path reference)
    pub svg: Option<String>,
    /// Glyph symbol for button display (e.g., "Γ", "∫", "Σ")
    pub glyph: Option<String>,
}

impl TemplateDefinition {
    pub fn new(name: String) -> Self {
        TemplateDefinition {
            name,
            pattern: None,
            unicode: None,
            latex: None,
            html: None,
            typst: None,
            kleis: None,
            category: None,
            shortcut: None,
            svg: None,
            glyph: None,
        }
    }
}

/// A @palette block
#[derive(Debug, Clone, Default)]
pub struct PaletteDefinition {
    pub tabs: Vec<TabDefinition>,
}

/// A tab within a palette
#[derive(Debug, Clone)]
pub struct TabDefinition {
    pub name: String,
    pub items: Vec<TabItem>,
}

/// An item within a tab
#[derive(Debug, Clone)]
pub enum TabItem {
    Group(GroupDefinition),
    Template(TemplateReference),
    Separator,
}

/// A group of templates within a tab
#[derive(Debug, Clone)]
pub struct GroupDefinition {
    pub name: String,
    pub templates: Vec<TemplateReference>,
}

/// A reference to a template with optional options
#[derive(Debug, Clone)]
pub struct TemplateReference {
    pub name: String,
    pub shortcut: Option<String>,
}

// =============================================================================
// Parser Error
// =============================================================================

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at {}:{}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}

// =============================================================================
// Tokenizer
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
enum Token {
    At,         // @
    OpenBrace,  // {
    CloseBrace, // }
    Colon,      // :
    Identifier(String),
    StringLiteral(String),
    Keyword(Keyword),
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
enum Keyword {
    Template,
    Palette,
    Tab,
    Group,
    Separator,
    Shortcut,
    Pattern,
    Unicode,
    Latex,
    Html,
    Typst,
    Kleis,
    Category,
    Svg,
    Glyph,
}

struct Tokenizer<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    line: usize,
    column: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Tokenizer {
            input,
            chars: input.char_indices().peekable(),
            line: 1,
            column: 1,
        }
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            line: self.line,
            column: self.column,
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while let Some(&(_, c)) = self.chars.peek() {
                if c == '\n' {
                    self.chars.next();
                    self.line += 1;
                    self.column = 1;
                } else if c.is_whitespace() {
                    self.chars.next();
                    self.column += 1;
                } else {
                    break;
                }
            }

            // Check for comments
            if let Some(&(pos, '/')) = self.chars.peek() {
                let next_char = self.input.get(pos + 1..pos + 2);
                if next_char == Some("/") {
                    // Line comment
                    self.chars.next();
                    self.chars.next();
                    while let Some(&(_, c)) = self.chars.peek() {
                        self.chars.next();
                        if c == '\n' {
                            self.line += 1;
                            self.column = 1;
                            break;
                        }
                    }
                    continue;
                } else if next_char == Some("*") {
                    // Block comment
                    self.chars.next();
                    self.chars.next();
                    while let Some((_pos, c)) = self.chars.next() {
                        if c == '\n' {
                            self.line += 1;
                            self.column = 1;
                        } else if c == '*' {
                            if let Some(&(_, '/')) = self.chars.peek() {
                                self.chars.next();
                                break;
                            }
                        }
                    }
                    continue;
                }
            }

            break;
        }
    }

    fn next_token(&mut self) -> Result<Token, ParseError> {
        self.skip_whitespace_and_comments();

        let Some(&(_, c)) = self.chars.peek() else {
            return Ok(Token::Eof);
        };

        match c {
            '@' => {
                self.chars.next();
                self.column += 1;
                Ok(Token::At)
            }
            '{' => {
                self.chars.next();
                self.column += 1;
                Ok(Token::OpenBrace)
            }
            '}' => {
                self.chars.next();
                self.column += 1;
                Ok(Token::CloseBrace)
            }
            ':' => {
                self.chars.next();
                self.column += 1;
                Ok(Token::Colon)
            }
            '"' => self.read_string(),
            _ if c.is_alphabetic() || c == '_' => self.read_identifier(),
            _ => Err(self.error(&format!("Unexpected character: '{}'", c))),
        }
    }

    fn read_string(&mut self) -> Result<Token, ParseError> {
        self.chars.next(); // consume opening quote
        self.column += 1;

        let mut s = String::new();
        loop {
            match self.chars.next() {
                Some((_, '"')) => {
                    self.column += 1;
                    return Ok(Token::StringLiteral(s));
                }
                Some((_, '\\')) => {
                    self.column += 1;
                    match self.chars.next() {
                        Some((_, 'n')) => s.push('\n'),
                        Some((_, 't')) => s.push('\t'),
                        Some((_, '"')) => s.push('"'),
                        Some((_, '\\')) => s.push('\\'),
                        Some((_, '{')) => s.push('{'),
                        Some((_, '}')) => s.push('}'),
                        Some((_, c)) => {
                            // Keep backslash for LaTeX escapes like \\ \frac etc.
                            s.push('\\');
                            s.push(c);
                        }
                        None => return Err(self.error("Unexpected end of string")),
                    }
                    self.column += 1;
                }
                Some((_, '\n')) => {
                    self.line += 1;
                    self.column = 1;
                    s.push('\n');
                }
                Some((_, c)) => {
                    s.push(c);
                    self.column += 1;
                }
                None => return Err(self.error("Unterminated string literal")),
            }
        }
    }

    fn read_identifier(&mut self) -> Result<Token, ParseError> {
        let mut ident = String::new();
        while let Some(&(_, c)) = self.chars.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.chars.next();
                self.column += 1;
            } else {
                break;
            }
        }

        // Check for keywords
        let token = match ident.as_str() {
            "template" => Token::Keyword(Keyword::Template),
            "palette" => Token::Keyword(Keyword::Palette),
            "tab" => Token::Keyword(Keyword::Tab),
            "group" => Token::Keyword(Keyword::Group),
            "separator" => Token::Keyword(Keyword::Separator),
            "shortcut" => Token::Keyword(Keyword::Shortcut),
            "pattern" => Token::Keyword(Keyword::Pattern),
            "unicode" => Token::Keyword(Keyword::Unicode),
            "latex" => Token::Keyword(Keyword::Latex),
            "html" => Token::Keyword(Keyword::Html),
            "typst" => Token::Keyword(Keyword::Typst),
            "kleis" => Token::Keyword(Keyword::Kleis),
            "category" => Token::Keyword(Keyword::Category),
            "svg" => Token::Keyword(Keyword::Svg),
            "glyph" => Token::Keyword(Keyword::Glyph),
            _ => Token::Identifier(ident),
        };
        Ok(token)
    }
}

// =============================================================================
// Parser
// =============================================================================

pub struct KleistParser<'a> {
    tokenizer: Tokenizer<'a>,
    current: Token,
}

impl<'a> KleistParser<'a> {
    pub fn new(input: &'a str) -> Result<Self, ParseError> {
        let mut tokenizer = Tokenizer::new(input);
        let current = tokenizer.next_token()?;
        Ok(KleistParser { tokenizer, current })
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        self.current = self.tokenizer.next_token()?;
        Ok(())
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if self.current == expected {
            self.advance()
        } else {
            Err(self
                .tokenizer
                .error(&format!("Expected {:?}, got {:?}", expected, self.current)))
        }
    }

    fn expect_string(&mut self) -> Result<String, ParseError> {
        match &self.current {
            Token::StringLiteral(s) => {
                let s = s.clone();
                self.advance()?;
                Ok(s)
            }
            _ => Err(self
                .tokenizer
                .error(&format!("Expected string literal, got {:?}", self.current))),
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        match &self.current {
            Token::Identifier(s) => {
                let s = s.clone();
                self.advance()?;
                Ok(s)
            }
            _ => Err(self
                .tokenizer
                .error(&format!("Expected identifier, got {:?}", self.current))),
        }
    }

    pub fn parse(&mut self) -> Result<KleistFile, ParseError> {
        let mut file = KleistFile::default();

        while self.current != Token::Eof {
            if self.current == Token::At {
                self.advance()?;
                match &self.current {
                    Token::Keyword(Keyword::Template) => {
                        let template = self.parse_template()?;
                        file.templates.push(template);
                    }
                    Token::Keyword(Keyword::Palette) => {
                        let palette = self.parse_palette()?;
                        file.palette = Some(palette);
                    }
                    _ => {
                        return Err(self
                            .tokenizer
                            .error("Expected 'template' or 'palette' after @"))
                    }
                }
            } else {
                return Err(self.tokenizer.error("Expected '@'"));
            }
        }

        Ok(file)
    }

    fn parse_template(&mut self) -> Result<TemplateDefinition, ParseError> {
        self.expect(Token::Keyword(Keyword::Template))?;
        let name = self.expect_identifier()?;
        self.expect(Token::OpenBrace)?;

        let mut template = TemplateDefinition::new(name);

        while self.current != Token::CloseBrace {
            match &self.current {
                Token::Keyword(Keyword::Pattern) => {
                    self.advance()?;
                    self.expect(Token::Colon)?;
                    template.pattern = Some(self.expect_string()?);
                }
                Token::Keyword(Keyword::Unicode) => {
                    self.advance()?;
                    self.expect(Token::Colon)?;
                    template.unicode = Some(self.expect_string()?);
                }
                Token::Keyword(Keyword::Latex) => {
                    self.advance()?;
                    self.expect(Token::Colon)?;
                    template.latex = Some(self.expect_string()?);
                }
                Token::Keyword(Keyword::Html) => {
                    self.advance()?;
                    self.expect(Token::Colon)?;
                    template.html = Some(self.expect_string()?);
                }
                Token::Keyword(Keyword::Typst) => {
                    self.advance()?;
                    self.expect(Token::Colon)?;
                    template.typst = Some(self.expect_string()?);
                }
                Token::Keyword(Keyword::Kleis) => {
                    self.advance()?;
                    self.expect(Token::Colon)?;
                    template.kleis = Some(self.expect_string()?);
                }
                Token::Keyword(Keyword::Category) => {
                    self.advance()?;
                    self.expect(Token::Colon)?;
                    template.category = Some(self.expect_string()?);
                }
                Token::Keyword(Keyword::Shortcut) => {
                    self.advance()?;
                    self.expect(Token::Colon)?;
                    template.shortcut = Some(self.expect_string()?);
                }
                Token::Keyword(Keyword::Svg) => {
                    self.advance()?;
                    self.expect(Token::Colon)?;
                    template.svg = Some(self.expect_string()?);
                }
                Token::Keyword(Keyword::Glyph) => {
                    self.advance()?;
                    self.expect(Token::Colon)?;
                    template.glyph = Some(self.expect_string()?);
                }
                _ => {
                    return Err(self
                        .tokenizer
                        .error(&format!("Unexpected token in template: {:?}", self.current)))
                }
            }
        }

        self.expect(Token::CloseBrace)?;
        Ok(template)
    }

    fn parse_palette(&mut self) -> Result<PaletteDefinition, ParseError> {
        self.expect(Token::Keyword(Keyword::Palette))?;
        self.expect(Token::OpenBrace)?;

        let mut palette = PaletteDefinition::default();

        while self.current != Token::CloseBrace {
            match &self.current {
                Token::Keyword(Keyword::Tab) => {
                    let tab = self.parse_tab()?;
                    palette.tabs.push(tab);
                }
                _ => {
                    return Err(self.tokenizer.error(&format!(
                        "Expected 'tab' in palette, got {:?}",
                        self.current
                    )))
                }
            }
        }

        self.expect(Token::CloseBrace)?;
        Ok(palette)
    }

    fn parse_tab(&mut self) -> Result<TabDefinition, ParseError> {
        self.expect(Token::Keyword(Keyword::Tab))?;
        let name = self.expect_string()?;
        self.expect(Token::OpenBrace)?;

        let mut tab = TabDefinition {
            name,
            items: Vec::new(),
        };

        while self.current != Token::CloseBrace {
            match &self.current {
                Token::Keyword(Keyword::Group) => {
                    let group = self.parse_group()?;
                    tab.items.push(TabItem::Group(group));
                }
                Token::Keyword(Keyword::Separator) => {
                    self.advance()?;
                    tab.items.push(TabItem::Separator);
                }
                Token::Identifier(_) => {
                    let template_ref = self.parse_template_reference()?;
                    tab.items.push(TabItem::Template(template_ref));
                }
                _ => {
                    return Err(self
                        .tokenizer
                        .error(&format!("Unexpected token in tab: {:?}", self.current)))
                }
            }
        }

        self.expect(Token::CloseBrace)?;
        Ok(tab)
    }

    fn parse_group(&mut self) -> Result<GroupDefinition, ParseError> {
        self.expect(Token::Keyword(Keyword::Group))?;
        let name = self.expect_string()?;
        self.expect(Token::OpenBrace)?;

        let mut group = GroupDefinition {
            name,
            templates: Vec::new(),
        };

        while self.current != Token::CloseBrace {
            if let Token::Identifier(_) = &self.current {
                let template_ref = self.parse_template_reference()?;
                group.templates.push(template_ref);
            } else {
                return Err(self.tokenizer.error(&format!(
                    "Expected template name in group, got {:?}",
                    self.current
                )));
            }
        }

        self.expect(Token::CloseBrace)?;
        Ok(group)
    }

    fn parse_template_reference(&mut self) -> Result<TemplateReference, ParseError> {
        let name = self.expect_identifier()?;
        let mut template_ref = TemplateReference {
            name,
            shortcut: None,
        };

        // Check for optional shortcut
        if self.current == Token::Keyword(Keyword::Shortcut) {
            self.advance()?;
            self.expect(Token::Colon)?;
            template_ref.shortcut = Some(self.expect_string()?);
        }

        Ok(template_ref)
    }
}

// =============================================================================
// Public API
// =============================================================================

/// Parse a .kleist file from a string
pub fn parse_kleist(input: &str) -> Result<KleistFile, ParseError> {
    let mut parser = KleistParser::new(input)?;
    parser.parse()
}

/// Parse a .kleist file from a file path
pub fn parse_kleist_file(path: &std::path::Path) -> Result<KleistFile, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let file = parse_kleist(&content)?;
    Ok(file)
}

/// Load all .kleist files from a directory
pub fn load_kleist_directory(
    dir: &std::path::Path,
) -> Result<KleistFile, Box<dyn std::error::Error>> {
    let mut combined = KleistFile::default();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "kleist").unwrap_or(false) {
            let file = parse_kleist_file(&path)?;
            combined.templates.extend(file.templates);
            if file.palette.is_some() {
                combined.palette = file.palette;
            }
        }
    }

    Ok(combined)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_template() {
        let input = r#"
@template plus {
    pattern: "plus(left, right)"
    unicode: "{left} + {right}"
    latex: "{left} + {right}"
    typst: "{left} + {right}"
}
"#;
        let file = parse_kleist(input).unwrap();
        assert_eq!(file.templates.len(), 1);
        assert_eq!(file.templates[0].name, "plus");
        assert_eq!(
            file.templates[0].unicode.as_deref(),
            Some("{left} + {right}")
        );
    }

    #[test]
    fn test_parse_template_with_latex_escapes() {
        let input = r#"
@template frac {
    pattern: "frac(num, den)"
    latex: "\\frac{{{num}}}{{{den}}}"
}
"#;
        let file = parse_kleist(input).unwrap();
        assert_eq!(file.templates.len(), 1);
        assert_eq!(
            file.templates[0].latex.as_deref(),
            Some("\\frac{{{num}}}{{{den}}}")
        );
    }

    #[test]
    fn test_parse_palette() {
        let input = r#"
@palette {
    tab "Basics" {
        group "Arithmetic" {
            plus
            minus
        }
    }
}
"#;
        let file = parse_kleist(input).unwrap();
        assert!(file.palette.is_some());
        let palette = file.palette.unwrap();
        assert_eq!(palette.tabs.len(), 1);
        assert_eq!(palette.tabs[0].name, "Basics");
    }

    #[test]
    fn test_parse_template_with_shortcut() {
        let input = r#"
@palette {
    tab "Calculus" {
        integral shortcut: "Ctrl+I"
        derivative
    }
}
"#;
        let file = parse_kleist(input).unwrap();
        let palette = file.palette.unwrap();
        let items = &palette.tabs[0].items;
        if let TabItem::Template(ref t) = items[0] {
            assert_eq!(t.shortcut.as_deref(), Some("Ctrl+I"));
        } else {
            panic!("Expected template");
        }
    }

    #[test]
    fn test_parse_comments() {
        let input = r#"
// This is a line comment
@template plus {
    /* This is a 
       block comment */
    unicode: "{left} + {right}"
}
"#;
        let file = parse_kleist(input).unwrap();
        assert_eq!(file.templates.len(), 1);
    }

    #[test]
    fn test_parse_separator() {
        let input = r#"
@palette {
    tab "Test" {
        plus
        separator
        minus
    }
}
"#;
        let file = parse_kleist(input).unwrap();
        let palette = file.palette.unwrap();
        let items = &palette.tabs[0].items;
        assert_eq!(items.len(), 3);
        assert!(matches!(items[1], TabItem::Separator));
    }

    #[test]
    fn test_load_std_template_lib() {
        let dir = std::path::Path::new("std_template_lib");
        if !dir.exists() {
            // Skip if directory doesn't exist (e.g., in CI before files are committed)
            return;
        }

        let result = load_kleist_directory(dir);
        assert!(
            result.is_ok(),
            "Failed to load std_template_lib: {:?}",
            result.err()
        );

        let file = result.unwrap();
        println!("Loaded {} templates", file.templates.len());
        assert!(
            file.templates.len() >= 50,
            "Expected at least 50 templates, got {}",
            file.templates.len()
        );

        // Check palette was loaded
        assert!(file.palette.is_some(), "Expected palette to be loaded");
        let palette = file.palette.unwrap();
        println!("Loaded {} tabs", palette.tabs.len());
        assert!(palette.tabs.len() >= 5, "Expected at least 5 tabs");
    }
}
