///! Kleis Text Parser - Parses Kleis text syntax into AST
///!
///! **IMPORTANT:** This is a SIMPLIFIED parser for ADR-015 POC validation.
///! It implements ~30% of the formal Kleis v0.3 grammar.
///!
///! **What's Supported:**
///! - Function calls: abs(x), card(S), norm(v), frac(a, b)
///! - Operators: +, -, *, /, ^, ×, ·
///! - Identifiers and numbers
///! - Parentheses for grouping
///! - Proper operator precedence
///!
///! **What's NOT Supported (yet):**
///! - Prefix operators: -x, ∇f, √x
///! - Postfix operators: n!, Aᵀ, A†
///! - Vector literals: [1, 2, 3]
///! - Lambda expressions: λ x . x^2
///! - Let bindings: let x = 5 in x^2
///! - Conditionals: if/then/else
///! - Type annotations: x : ℝ
///! - Relations/logic: =, <, ∧, ∨, etc.
///! - Calculus operators as infix: ∫, ∂
///! - Symbolic constants: π, e, i
///! - Placeholders: □
///!
///! See docs/PARSER_GRAMMAR_COMPATIBILITY.md for full comparison with formal grammar.
///!
///! **Grammar (simplified):**
///!   expression := term (('+' | '-') term)*
///!   term := factor (('*' | '/') factor)*
///!   factor := primary ('^' primary)?
///!   primary := identifier | number | function_call | '(' expression ')'
///!   function_call := identifier '(' arguments ')'
///!   arguments := expression (',' expression)*
///!
///! **Purpose:** Validate ADR-015 design decisions, not production-ready!
use crate::ast::{Expression, MatchCase, Pattern};
use crate::kleis_ast::{
    DataDef, DataField, DataVariant, ImplMember, Implementation, ImplementsDef, OperationDecl,
    Program, StructureDef, StructureMember, TopLevel, TypeExpr,
};
use std::fmt;

#[derive(Debug)]
pub struct KleisParseError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for KleisParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Kleis parse error at position {}: {}",
            self.position, self.message
        )
    }
}

impl std::error::Error for KleisParseError {}

pub struct KleisParser {
    input: Vec<char>,
    pos: usize,
}

impl KleisParser {
    pub fn new(input: &str) -> Self {
        KleisParser {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.input.len() {
            let ch = self.input[self.pos];
            self.pos += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            // Skip whitespace characters
            while let Some(ch) = self.peek() {
                if ch.is_whitespace() {
                    self.advance();
                } else {
                    break;
                }
            }

            // Skip comments
            if self.peek() == Some('/') {
                if self.peek_ahead(1) == Some('/') {
                    // Line comment: skip until newline
                    self.advance(); // /
                    self.advance(); // /
                    while let Some(ch) = self.peek() {
                        if ch == '\n' {
                            self.advance(); // consume newline
                            break;
                        }
                        self.advance();
                    }
                    continue; // Re-check for more whitespace/comments
                } else if self.peek_ahead(1) == Some('*') {
                    // Block comment: skip until */
                    self.advance(); // /
                    self.advance(); // *
                    while let Some(ch) = self.peek() {
                        if ch == '*' && self.peek_ahead(1) == Some('/') {
                            self.advance(); // *
                            self.advance(); // /
                            break;
                        }
                        self.advance();
                    }
                    continue; // Re-check for more whitespace/comments
                }
            }

            // No more whitespace or comments
            break;
        }
    }

    fn parse_identifier(&mut self) -> Result<String, KleisParseError> {
        let start = self.pos;

        // First character must be letter or underscore
        match self.peek() {
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                self.advance();
            }
            _ => {
                return Err(KleisParseError {
                    message: "Expected identifier".to_string(),
                    position: self.pos,
                });
            }
        }

        // Subsequent characters can be alphanumeric or underscore
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        Ok(self.input[start..self.pos].iter().collect())
    }

    fn parse_number(&mut self) -> Result<String, KleisParseError> {
        let start = self.pos;

        // Integer part
        if !self.peek().map_or(false, |ch| ch.is_numeric()) {
            return Err(KleisParseError {
                message: "Expected number".to_string(),
                position: self.pos,
            });
        }

        while let Some(ch) = self.peek() {
            if ch.is_numeric() {
                self.advance();
            } else {
                break;
            }
        }

        // Optional decimal part
        if self.peek() == Some('.') {
            self.advance();
            while let Some(ch) = self.peek() {
                if ch.is_numeric() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        Ok(self.input[start..self.pos].iter().collect())
    }

    fn parse_arguments(&mut self) -> Result<Vec<Expression>, KleisParseError> {
        let mut args = Vec::new();

        // Empty argument list
        self.skip_whitespace();
        if self.peek() == Some(')') {
            return Ok(args);
        }

        // Parse comma-separated expressions
        loop {
            self.skip_whitespace();
            args.push(self.parse_expression()?);
            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.advance();
                    continue;
                }
                Some(')') => break,
                _ => {
                    return Err(KleisParseError {
                        message: "Expected ',' or ')'".to_string(),
                        position: self.pos,
                    });
                }
            }
        }

        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expression, KleisParseError> {
        self.skip_whitespace();

        // Match expression
        if self.peek_word("match") {
            return self.parse_match_expr();
        }

        // Parenthesized expression
        if self.peek() == Some('(') {
            self.advance();
            let expr = self.parse_expression()?;
            self.skip_whitespace();
            if self.advance() != Some(')') {
                return Err(KleisParseError {
                    message: "Expected ')'".to_string(),
                    position: self.pos,
                });
            }
            return Ok(expr);
        }

        // Number
        if self.peek().map_or(false, |ch| ch.is_numeric()) {
            let num = self.parse_number()?;
            return Ok(Expression::Const(num));
        }

        // Identifier or function call
        if self
            .peek()
            .map_or(false, |ch| ch.is_alphabetic() || ch == '_')
        {
            let id = self.parse_identifier()?;
            self.skip_whitespace();

            // Function call
            if self.peek() == Some('(') {
                self.advance();
                let args = self.parse_arguments()?;
                self.skip_whitespace();
                if self.advance() != Some(')') {
                    return Err(KleisParseError {
                        message: "Expected ')'".to_string(),
                        position: self.pos,
                    });
                }
                return Ok(Expression::Operation { name: id, args });
            }

            // Just an identifier
            return Ok(Expression::Object(id));
        }

        Err(KleisParseError {
            message: "Expected expression".to_string(),
            position: self.pos,
        })
    }

    fn parse_factor(&mut self) -> Result<Expression, KleisParseError> {
        let mut left = self.parse_primary()?;

        self.skip_whitespace();
        if self.peek() == Some('^') {
            self.advance();
            let right = self.parse_primary()?;
            left = Expression::Operation {
                name: "power".to_string(),
                args: vec![left, right],
            };
        }

        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expression, KleisParseError> {
        let mut left = self.parse_factor()?;

        loop {
            self.skip_whitespace();
            let op = match self.peek() {
                Some('*') => "times",
                Some('/') => "divide",
                Some('×') => "times",
                Some('·') => "times",
                _ => break,
            };

            self.advance();
            let right = self.parse_factor()?;
            left = Expression::Operation {
                name: op.to_string(),
                args: vec![left, right],
            };
        }

        Ok(left)
    }

    fn parse_expression(&mut self) -> Result<Expression, KleisParseError> {
        let mut left = self.parse_term()?;

        loop {
            self.skip_whitespace();
            let op = match self.peek() {
                Some('+') => "plus",
                Some('-') => "minus",
                _ => break,
            };

            self.advance();
            let right = self.parse_term()?;
            left = Expression::Operation {
                name: op.to_string(),
                args: vec![left, right],
            };
        }

        Ok(left)
    }

    pub fn parse(&mut self) -> Result<Expression, KleisParseError> {
        self.skip_whitespace();
        let expr = self.parse_expression()?;
        self.skip_whitespace();

        // Ensure we consumed all input
        if self.pos < self.input.len() {
            return Err(KleisParseError {
                message: format!("Unexpected character: '{}'", self.input[self.pos]),
                position: self.pos,
            });
        }

        Ok(expr)
    }

    /// Parse a type expression
    /// Examples: ℝ, Vector(3), Set(ℤ), ℝ → ℝ
    pub fn parse_type(&mut self) -> Result<TypeExpr, KleisParseError> {
        self.skip_whitespace();

        // Parse base type - could be identifier or number (for dimension literals)
        let base_name = if self.peek().map_or(false, |ch| ch.is_numeric()) {
            self.parse_number()?
        } else {
            self.parse_identifier()?
        };

        self.skip_whitespace();

        let mut ty = if self.peek() == Some('(') {
            // Parametric type: Vector(3), Set(ℤ)
            self.advance(); // consume '('
            let mut params = Vec::new();

            loop {
                self.skip_whitespace();
                if self.peek() == Some(')') {
                    break;
                }

                params.push(self.parse_type()?);
                self.skip_whitespace();

                if self.peek() == Some(',') {
                    self.advance();
                } else if self.peek() == Some(')') {
                    break;
                } else {
                    return Err(KleisParseError {
                        message: "Expected ',' or ')' in type parameters".to_string(),
                        position: self.pos,
                    });
                }
            }

            if self.advance() != Some(')') {
                return Err(KleisParseError {
                    message: "Expected ')'".to_string(),
                    position: self.pos,
                });
            }

            TypeExpr::Parametric(base_name, params)
        } else {
            TypeExpr::Named(base_name)
        };

        // Check for function type: T1 → T2 or T1 -> T2
        self.skip_whitespace();
        if self.peek() == Some('→') || (self.peek() == Some('-') && self.peek_ahead(1) == Some('>'))
        {
            // Consume arrow
            if self.peek() == Some('→') {
                self.advance();
            } else {
                self.advance(); // -
                self.advance(); // >
            }

            let return_type = self.parse_type()?;
            ty = TypeExpr::Function(Box::new(ty), Box::new(return_type));
        }

        // Check for × (product type for multi-arg functions)
        self.skip_whitespace();
        if self.peek() == Some('×') {
            let mut product_types = vec![ty];
            while self.peek() == Some('×') {
                self.advance();
                product_types.push(self.parse_type()?);
                self.skip_whitespace();
            }
            ty = TypeExpr::Product(product_types);
        }

        Ok(ty)
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        let pos = self.pos + offset;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    /// Consume a specific string if present, return true if successful
    fn consume_str(&mut self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        for (i, ch) in chars.iter().enumerate() {
            if self.peek_ahead(i) != Some(*ch) {
                return false;
            }
        }
        // Consume the string
        for _ in 0..chars.len() {
            self.advance();
        }
        true
    }

    /// Expect a specific character, return error if not found
    fn expect_char(&mut self, expected: char) -> Result<(), KleisParseError> {
        match self.advance() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(KleisParseError {
                message: format!("Expected '{}', found '{}'", expected, ch),
                position: self.pos - 1,
            }),
            None => Err(KleisParseError {
                message: format!("Expected '{}', found end of input", expected),
                position: self.pos,
            }),
        }
    }

    /// Expect a specific word/keyword, return error if not found
    fn expect_word(&mut self, word: &str) -> Result<(), KleisParseError> {
        if self.peek_word(word) {
            // Consume the word
            for _ in 0..word.len() {
                self.advance();
            }
            Ok(())
        } else {
            Err(KleisParseError {
                message: format!("Expected keyword '{}'", word),
                position: self.pos,
            })
        }
    }

    /// Parse a match expression
    /// Grammar: match expr { case1 | case2 ... }
    fn parse_match_expr(&mut self) -> Result<Expression, KleisParseError> {
        // Consume 'match' keyword
        self.expect_word("match")?;
        self.skip_whitespace();

        // Parse scrutinee expression
        let scrutinee = self.parse_expression()?;
        self.skip_whitespace();

        // Expect opening brace
        self.expect_char('{')?;
        self.skip_whitespace();

        // Parse cases
        let cases = self.parse_match_cases()?;
        self.skip_whitespace();

        // Expect closing brace
        self.expect_char('}')?;

        Ok(Expression::match_expr(scrutinee, cases))
    }

    /// Parse match cases separated by '|' or newlines
    fn parse_match_cases(&mut self) -> Result<Vec<MatchCase>, KleisParseError> {
        let mut cases = Vec::new();

        loop {
            self.skip_whitespace();

            // Check for closing brace
            if self.peek() == Some('}') {
                break;
            }

            // Parse one case
            let case = self.parse_match_case()?;
            cases.push(case);

            self.skip_whitespace();

            // Optional separator
            if self.peek() == Some('|') {
                self.advance();
            }
        }

        if cases.is_empty() {
            return Err(KleisParseError {
                message: "Match expression must have at least one case".to_string(),
                position: self.pos,
            });
        }

        Ok(cases)
    }

    /// Parse a single match case
    /// Grammar: pattern => expression
    fn parse_match_case(&mut self) -> Result<MatchCase, KleisParseError> {
        self.skip_whitespace();

        // Parse pattern
        let pattern = self.parse_pattern()?;
        self.skip_whitespace();

        // Expect =>
        if !self.consume_str("=>") {
            return Err(KleisParseError {
                message: "Expected '=>' after pattern".to_string(),
                position: self.pos,
            });
        }
        self.skip_whitespace();

        // Parse body expression
        let body = self.parse_expression()?;

        Ok(MatchCase::new(pattern, body))
    }

    /// Parse a pattern
    fn parse_pattern(&mut self) -> Result<Pattern, KleisParseError> {
        self.skip_whitespace();

        // Wildcard: _
        if self.peek() == Some('_') {
            let start_pos = self.pos;
            self.advance();
            // Make sure it's just underscore (not part of identifier)
            if self
                .peek()
                .map_or(true, |ch| !ch.is_alphanumeric() && ch != '_')
            {
                return Ok(Pattern::wildcard());
            }
            // Otherwise, it's an identifier starting with underscore
            self.pos = start_pos;
        }

        // Number constant
        if self.peek().map_or(false, |ch| ch.is_numeric()) {
            let num = self.parse_number()?;
            return Ok(Pattern::constant(num));
        }

        // Constructor or variable
        if self
            .peek()
            .map_or(false, |ch| ch.is_alphabetic() || ch == '_')
        {
            let id = self.parse_identifier()?;
            self.skip_whitespace();

            // Constructor with arguments: Some(x)
            if self.peek() == Some('(') {
                self.advance();
                let args = self.parse_pattern_args()?;
                self.skip_whitespace();
                self.expect_char(')')?;
                return Ok(Pattern::constructor(id, args));
            }

            // Determine if it's a constructor or variable
            // Heuristic: Capitalized = constructor, lowercase = variable
            if id.chars().next().unwrap().is_uppercase() {
                // Constructor without arguments: None, True, False
                return Ok(Pattern::constructor(id, vec![]));
            } else {
                // Variable binding: x, value, result
                return Ok(Pattern::variable(id));
            }
        }

        Err(KleisParseError {
            message: "Expected pattern (wildcard, variable, constructor, or constant)".to_string(),
            position: self.pos,
        })
    }

    /// Parse pattern arguments separated by commas
    fn parse_pattern_args(&mut self) -> Result<Vec<Pattern>, KleisParseError> {
        let mut args = Vec::new();

        loop {
            self.skip_whitespace();

            // Check for closing paren
            if self.peek() == Some(')') {
                break;
            }

            // Parse one pattern
            let pattern = self.parse_pattern()?;
            args.push(pattern);

            self.skip_whitespace();

            // Check for comma
            if self.peek() == Some(',') {
                self.advance();
            } else if self.peek() == Some(')') {
                break;
            } else {
                return Err(KleisParseError {
                    message: "Expected ',' or ')' in pattern arguments".to_string(),
                    position: self.pos,
                });
            }
        }

        Ok(args)
    }

    /// Parse structure member
    fn parse_structure_member(&mut self) -> Result<StructureMember, KleisParseError> {
        self.skip_whitespace();
        let name = self.parse_identifier()?;
        self.skip_whitespace();

        if self.advance() != Some(':') {
            return Err(KleisParseError {
                message: "Expected ':' after member name".to_string(),
                position: self.pos,
            });
        }

        let type_expr = self.parse_type()?;

        Ok(StructureMember::Field { name, type_expr })
    }

    /// Parse structure definition
    /// Example: structure Money { amount : ℝ }
    /// Or: structure Numeric(N) { operation abs : N → N }
    pub fn parse_structure(&mut self) -> Result<StructureDef, KleisParseError> {
        self.skip_whitespace();

        // Expect 'structure' keyword
        let keyword = self.parse_identifier()?;
        if keyword != "structure" {
            return Err(KleisParseError {
                message: format!("Expected 'structure', got '{}'", keyword),
                position: self.pos,
            });
        }

        self.skip_whitespace();
        let name = self.parse_identifier()?;
        self.skip_whitespace();

        // Optional type parameters: (N) or (m: Nat, n: Nat, T)
        let type_params = if self.peek() == Some('(') {
            self.advance();
            self.skip_whitespace();

            let mut params = Vec::new();

            while self.peek() != Some(')') {
                let param_name = self.parse_identifier()?;
                self.skip_whitespace();

                // Optional kind annotation: m: Nat
                let kind = if self.peek() == Some(':') {
                    self.advance();
                    self.skip_whitespace();
                    Some(self.parse_identifier()?)
                } else {
                    None
                };

                params.push(crate::kleis_ast::TypeParam {
                    name: param_name,
                    kind,
                });

                self.skip_whitespace();
                if self.peek() == Some(',') {
                    self.advance();
                    self.skip_whitespace();
                }
            }

            if self.advance() != Some(')') {
                return Err(KleisParseError {
                    message: "Expected ')' after type parameters".to_string(),
                    position: self.pos,
                });
            }

            self.skip_whitespace();
            params
        } else {
            Vec::new()
        };

        // Expect '{'
        if self.advance() != Some('{') {
            return Err(KleisParseError {
                message: "Expected '{'".to_string(),
                position: self.pos,
            });
        }

        // Parse members
        let mut members = Vec::new();
        loop {
            self.skip_whitespace();

            if self.peek() == Some('}') {
                break;
            }

            // Check for operation keyword
            let start_pos = self.pos;
            if self.peek_word("operation") {
                // Skip "operation"
                for _ in 0..9 {
                    self.advance();
                }
                self.skip_whitespace();

                let op_name = self.parse_identifier()?;
                self.skip_whitespace();

                if self.advance() != Some(':') {
                    return Err(KleisParseError {
                        message: "Expected ':' after operation name".to_string(),
                        position: self.pos,
                    });
                }

                let type_sig = self.parse_type()?;

                members.push(StructureMember::Operation {
                    name: op_name,
                    type_signature: type_sig,
                });
            } else {
                // Regular field
                self.pos = start_pos;
                members.push(self.parse_structure_member()?);
            }
        }

        // Expect '}'
        if self.advance() != Some('}') {
            return Err(KleisParseError {
                message: "Expected '}'".to_string(),
                position: self.pos,
            });
        }

        Ok(StructureDef {
            name,
            type_params,
            members,
        })
    }

    fn peek_word(&self, word: &str) -> bool {
        let word_chars: Vec<char> = word.chars().collect();
        for (i, ch) in word_chars.iter().enumerate() {
            if self.peek_ahead(i) != Some(*ch) {
                return false;
            }
        }
        // Check that it's followed by whitespace or special char (not part of identifier)
        if let Some(next) = self.peek_ahead(word_chars.len()) {
            !next.is_alphanumeric() && next != '_'
        } else {
            true
        }
    }

    /// Parse operation declaration (top-level)
    /// Example: operation abs : ℝ → ℝ
    pub fn parse_operation_decl(&mut self) -> Result<OperationDecl, KleisParseError> {
        self.skip_whitespace();

        // Expect 'operation' keyword
        let keyword = self.parse_identifier()?;
        if keyword != "operation" {
            return Err(KleisParseError {
                message: format!("Expected 'operation', got '{}'", keyword),
                position: self.pos,
            });
        }

        self.skip_whitespace();

        // Parse operation name (could be symbol like (+) or identifier)
        let name = if self.peek() == Some('(') {
            // Operator in parens: (+)
            self.advance(); // (
            self.skip_whitespace();
            let op = match self.peek() {
                Some('+') => "+",
                Some('-') => "-",
                Some('*') => "*",
                Some('/') => "/",
                Some('^') => "^",
                Some('×') => "×",
                Some('·') => "·",
                _ => {
                    return Err(KleisParseError {
                        message: "Expected operator symbol".to_string(),
                        position: self.pos,
                    });
                }
            };
            self.advance();
            self.skip_whitespace();
            if self.advance() != Some(')') {
                return Err(KleisParseError {
                    message: "Expected ')' after operator".to_string(),
                    position: self.pos,
                });
            }
            op.to_string()
        } else {
            self.parse_identifier()?
        };

        self.skip_whitespace();

        // Expect ':'
        if self.advance() != Some(':') {
            return Err(KleisParseError {
                message: "Expected ':' after operation name".to_string(),
                position: self.pos,
            });
        }

        let type_signature = self.parse_type()?;

        Ok(OperationDecl {
            name,
            type_signature,
        })
    }

    /// Parse implements block
    /// Example: implements Numeric(ℝ) { operation abs = builtin_abs }
    pub fn parse_implements(&mut self) -> Result<ImplementsDef, KleisParseError> {
        self.skip_whitespace();

        // Expect 'implements' keyword
        let keyword = self.parse_identifier()?;
        if keyword != "implements" {
            return Err(KleisParseError {
                message: format!("Expected 'implements', got '{}'", keyword),
                position: self.pos,
            });
        }

        self.skip_whitespace();

        // Parse structure name
        let structure_name = self.parse_identifier()?;
        self.skip_whitespace();

        // Parse type arguments: (ℝ) or (m, n, ℝ) or (Vector(n))
        if self.advance() != Some('(') {
            return Err(KleisParseError {
                message: "Expected '(' after structure name".to_string(),
                position: self.pos,
            });
        }

        let mut type_args = Vec::new();
        self.skip_whitespace();

        while self.peek() != Some(')') {
            type_args.push(self.parse_type()?);
            self.skip_whitespace();

            if self.peek() == Some(',') {
                self.advance();
                self.skip_whitespace();
            }
        }

        if self.advance() != Some(')') {
            return Err(KleisParseError {
                message: "Expected ')' after type arguments".to_string(),
                position: self.pos,
            });
        }

        self.skip_whitespace();

        // Parse members in { }
        if self.advance() != Some('{') {
            return Err(KleisParseError {
                message: "Expected '{'".to_string(),
                position: self.pos,
            });
        }

        let mut members = Vec::new();
        loop {
            self.skip_whitespace();

            if self.peek() == Some('}') {
                break;
            }

            members.push(self.parse_impl_member()?);
        }

        if self.advance() != Some('}') {
            return Err(KleisParseError {
                message: "Expected '}'".to_string(),
                position: self.pos,
            });
        }

        Ok(ImplementsDef {
            structure_name,
            type_args,
            members,
        })
    }

    fn parse_impl_member(&mut self) -> Result<ImplMember, KleisParseError> {
        self.skip_whitespace();
        let keyword = self.parse_identifier()?;
        self.skip_whitespace();

        match keyword.as_str() {
            "element" => {
                // element zero = 0
                let name = self.parse_identifier()?;
                self.skip_whitespace();

                if self.advance() != Some('=') {
                    return Err(KleisParseError {
                        message: "Expected '=' after element name".to_string(),
                        position: self.pos,
                    });
                }

                let value = self.parse_expression()?;

                Ok(ImplMember::Element { name, value })
            }
            "operation" => {
                // operation abs = builtin_abs
                // or operation abs(x) = x^2
                let name = self.parse_identifier()?;
                self.skip_whitespace();

                if self.advance() != Some('=') {
                    return Err(KleisParseError {
                        message: "Expected '=' after operation name".to_string(),
                        position: self.pos,
                    });
                }

                self.skip_whitespace();

                // Check if next is identifier (builtin) or expression
                // For now, assume builtin (TODO: handle inline definitions)
                let builtin_name = self.parse_identifier()?;

                Ok(ImplMember::Operation {
                    name,
                    implementation: Implementation::Builtin(builtin_name),
                })
            }
            _ => Err(KleisParseError {
                message: format!("Expected 'element' or 'operation', got '{}'", keyword),
                position: self.pos,
            }),
        }
    }

    /// Parse a data type definition: data Bool = True | False
    /// Grammar:
    ///   dataDecl ::= "data" identifier [ "(" typeParams ")" ] "="
    ///                dataVariant { "|" dataVariant }
    ///   dataVariant ::= identifier [ "(" dataFields ")" ]
    ///   dataFields ::= dataField { "," dataField }
    ///   dataField ::= [ identifier ":" ] typeExpr
    pub fn parse_data_def(&mut self) -> Result<DataDef, KleisParseError> {
        self.skip_whitespace();

        // Expect 'data' keyword
        let keyword = self.parse_identifier()?;
        if keyword != "data" {
            return Err(KleisParseError {
                message: format!("Expected 'data', got '{}'", keyword),
                position: self.pos,
            });
        }

        self.skip_whitespace();
        let name = self.parse_identifier()?;
        self.skip_whitespace();

        // Optional type parameters: (T) or (T, E) or (m: Nat, n: Nat)
        let type_params = if self.peek() == Some('(') {
            self.advance();
            self.skip_whitespace();

            let mut params = Vec::new();

            while self.peek() != Some(')') {
                let param_name = self.parse_identifier()?;
                self.skip_whitespace();

                // Optional kind annotation: m: Nat
                let kind = if self.peek() == Some(':') {
                    self.advance();
                    self.skip_whitespace();
                    Some(self.parse_identifier()?)
                } else {
                    None
                };

                params.push(crate::kleis_ast::TypeParam {
                    name: param_name,
                    kind,
                });

                self.skip_whitespace();
                if self.peek() == Some(',') {
                    self.advance();
                    self.skip_whitespace();
                }
            }

            if self.advance() != Some(')') {
                return Err(KleisParseError {
                    message: "Expected ')' after type parameters".to_string(),
                    position: self.pos,
                });
            }

            self.skip_whitespace();
            params
        } else {
            Vec::new()
        };

        // Expect '='
        if self.advance() != Some('=') {
            return Err(KleisParseError {
                message: "Expected '=' after data type name".to_string(),
                position: self.pos,
            });
        }

        self.skip_whitespace();

        // Parse first variant (required)
        let mut variants = vec![self.parse_data_variant()?];

        // Parse additional variants with "|" separator
        loop {
            self.skip_whitespace();

            if self.peek() != Some('|') {
                break;
            }

            self.advance(); // consume '|'
            self.skip_whitespace();

            variants.push(self.parse_data_variant()?);
        }

        Ok(DataDef {
            name,
            type_params,
            variants,
        })
    }

    /// Parse a data variant: Variant or Variant(fields)
    fn parse_data_variant(&mut self) -> Result<DataVariant, KleisParseError> {
        self.skip_whitespace();

        let name = self.parse_identifier()?;
        self.skip_whitespace();

        // Optional fields in parentheses
        let fields = if self.peek() == Some('(') {
            self.advance();
            self.skip_whitespace();

            let mut fields = Vec::new();

            while self.peek() != Some(')') {
                fields.push(self.parse_data_field()?);
                self.skip_whitespace();

                if self.peek() == Some(',') {
                    self.advance();
                    self.skip_whitespace();
                }
            }

            if self.advance() != Some(')') {
                return Err(KleisParseError {
                    message: "Expected ')' after data variant fields".to_string(),
                    position: self.pos,
                });
            }

            fields
        } else {
            Vec::new()
        };

        Ok(DataVariant { name, fields })
    }

    /// Parse a data field: type or name: type
    fn parse_data_field(&mut self) -> Result<DataField, KleisParseError> {
        self.skip_whitespace();

        // Try to parse as "name: type" or just "type"
        // We need to look ahead to see if there's a colon
        let start_pos = self.pos;

        // Try to parse identifier
        if let Ok(potential_name) = self.parse_identifier() {
            self.skip_whitespace();

            // Check if followed by colon
            if self.peek() == Some(':') {
                self.advance(); // consume ':'
                self.skip_whitespace();

                let type_expr = self.parse_type()?;

                return Ok(DataField {
                    name: Some(potential_name),
                    type_expr,
                });
            } else {
                // Not a named field, backtrack and parse as type
                self.pos = start_pos;
            }
        } else {
            // Not an identifier, reset position
            self.pos = start_pos;
        }

        // Parse as positional field (just a type)
        let type_expr = self.parse_type()?;

        Ok(DataField {
            name: None,
            type_expr,
        })
    }

    /// Parse a complete program (multiple top-level items)
    pub fn parse_program(&mut self) -> Result<Program, KleisParseError> {
        let mut program = Program::new();

        loop {
            self.skip_whitespace();

            if self.pos >= self.input.len() {
                break;
            }

            // Peek at next keyword
            if self.peek_word("structure") {
                let structure = self.parse_structure()?;
                program.add_item(TopLevel::StructureDef(structure));
            } else if self.peek_word("implements") {
                let implements = self.parse_implements()?;
                program.add_item(TopLevel::ImplementsDef(implements));
            } else if self.peek_word("data") {
                let data_def = self.parse_data_def()?;
                program.add_item(TopLevel::DataDef(data_def));
            } else if self.peek_word("operation") {
                let operation = self.parse_operation_decl()?;
                program.add_item(TopLevel::OperationDecl(operation));
            } else {
                return Err(KleisParseError {
                    message: "Expected 'structure', 'implements', 'data', or 'operation'"
                        .to_string(),
                    position: self.pos,
                });
            }
        }

        Ok(program)
    }
}

/// Parse Kleis text into an Expression AST
pub fn parse_kleis(input: &str) -> Result<Expression, KleisParseError> {
    let mut parser = KleisParser::new(input);
    parser.parse()
}

/// Parse a complete Kleis program (with structures, operations, etc.)
pub fn parse_kleis_program(input: &str) -> Result<Program, KleisParseError> {
    let mut parser = KleisParser::new(input);
    parser.parse_program()
}

/// Parse a type expression
pub fn parse_type_expr(input: &str) -> Result<TypeExpr, KleisParseError> {
    let mut parser = KleisParser::new(input);
    parser.parse_type()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_identifier() {
        let result = parse_kleis("x").unwrap();
        assert!(matches!(result, Expression::Object(ref s) if s == "x"));
    }

    #[test]
    fn test_number() {
        let result = parse_kleis("42").unwrap();
        assert!(matches!(result, Expression::Const(ref s) if s == "42"));
    }

    #[test]
    fn test_function_call_single_arg() {
        let result = parse_kleis("abs(x)").unwrap();
        match result {
            Expression::Operation { name, args } => {
                assert_eq!(name, "abs");
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], Expression::Object(ref s) if s == "x"));
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_function_call_two_args() {
        let result = parse_kleis("frac(a, b)").unwrap();
        match result {
            Expression::Operation { name, args } => {
                assert_eq!(name, "frac");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_nested_call() {
        let result = parse_kleis("abs(frac(a, b))").unwrap();
        match result {
            Expression::Operation { name, args } => {
                assert_eq!(name, "abs");
                assert_eq!(args.len(), 1);
                match &args[0] {
                    Expression::Operation { name, args } => {
                        assert_eq!(name, "frac");
                        assert_eq!(args.len(), 2);
                    }
                    _ => panic!("Expected nested Operation"),
                }
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_arithmetic() {
        let result = parse_kleis("a + b").unwrap();
        match result {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "plus");
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_division() {
        let result = parse_kleis("a / b").unwrap();
        match result {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "divide");
            }
            _ => panic!("Expected Operation"),
        }
    }

    // Tests for new features (structures, types, programs)

    #[test]
    fn test_parse_simple_type() {
        let result = parse_type_expr("ℝ").unwrap();
        assert_eq!(result, TypeExpr::Named("ℝ".to_string()));
    }

    #[test]
    fn test_parse_parametric_type() {
        let result = parse_type_expr("Vector(3)").unwrap();
        match result {
            TypeExpr::Parametric(name, params) => {
                assert_eq!(name, "Vector");
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], TypeExpr::Named("3".to_string()));
            }
            _ => panic!("Expected Parametric"),
        }
    }

    #[test]
    fn test_parse_function_type() {
        let result = parse_type_expr("ℝ → ℝ").unwrap();
        match result {
            TypeExpr::Function(from, to) => {
                assert_eq!(*from, TypeExpr::Named("ℝ".to_string()));
                assert_eq!(*to, TypeExpr::Named("ℝ".to_string()));
            }
            _ => panic!("Expected Function"),
        }
    }

    #[test]
    fn test_parse_operation_decl() {
        let code = "operation abs : ℝ → ℝ";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_operation_decl().unwrap();

        assert_eq!(result.name, "abs");
        assert!(matches!(result.type_signature, TypeExpr::Function(_, _)));
    }

    #[test]
    fn test_parse_structure_simple() {
        let code = "structure Money { amount : ℝ }";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_structure().unwrap();

        assert_eq!(result.name, "Money");
        assert_eq!(result.members.len(), 1);

        match &result.members[0] {
            StructureMember::Field { name, type_expr } => {
                assert_eq!(name, "amount");
                assert_eq!(*type_expr, TypeExpr::Named("ℝ".to_string()));
            }
            _ => panic!("Expected Field"),
        }
    }

    #[test]
    fn test_parse_structure_multiple_fields() {
        let code = "structure Money { amount : ℝ currency : String }";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_structure().unwrap();

        assert_eq!(result.name, "Money");
        assert_eq!(result.members.len(), 2);
    }

    #[test]
    fn test_parse_program_with_operations() {
        let code = r#"
            operation abs : ℝ → ℝ
            operation card : Set(ℤ) → ℕ
        "#;

        let result = parse_kleis_program(code).unwrap();
        assert_eq!(result.items.len(), 2);

        // Check first operation
        match &result.items[0] {
            TopLevel::OperationDecl(op) => {
                assert_eq!(op.name, "abs");
            }
            _ => panic!("Expected OperationDecl"),
        }
    }

    #[test]
    fn test_parse_program_with_structure() {
        let code = r#"
            structure Money {
                amount : ℝ
                currency : String
            }
            
            operation (+) : Money → Money
        "#;

        let result = parse_kleis_program(code).unwrap();
        assert_eq!(result.items.len(), 2);

        // Check structure
        match &result.items[0] {
            TopLevel::StructureDef(s) => {
                assert_eq!(s.name, "Money");
                assert_eq!(s.members.len(), 2);
            }
            _ => panic!("Expected StructureDef"),
        }

        // Check operation
        match &result.items[1] {
            TopLevel::OperationDecl(op) => {
                assert_eq!(op.name, "+");
            }
            _ => panic!("Expected OperationDecl"),
        }
    }

    #[test]
    fn test_parse_implements_simple() {
        let code = "implements Numeric(ℝ) { operation abs = builtin_abs }";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_implements().unwrap();

        assert_eq!(result.structure_name, "Numeric");
        assert_eq!(result.type_args.len(), 1);
        assert_eq!(result.type_args[0], TypeExpr::Named("ℝ".to_string()));
        assert_eq!(result.members.len(), 1);

        match &result.members[0] {
            ImplMember::Operation {
                name,
                implementation,
            } => {
                assert_eq!(name, "abs");
                assert!(matches!(implementation, Implementation::Builtin(_)));
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_parse_implements_multiple_members() {
        let code = r#"
            implements Numeric(ℝ) {
                element zero = 0
                operation abs = builtin_abs
                operation floor = builtin_floor
            }
        "#;

        let mut parser = KleisParser::new(code);
        let result = parser.parse_implements().unwrap();

        assert_eq!(result.members.len(), 3);

        // Check element
        match &result.members[0] {
            ImplMember::Element { name, .. } => {
                assert_eq!(name, "zero");
            }
            _ => panic!("Expected Element"),
        }

        // Check operations
        match &result.members[1] {
            ImplMember::Operation { name, .. } => {
                assert_eq!(name, "abs");
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_parse_program_with_structure_and_implements() {
        let code = r#"
            structure Numeric(N) {
                operation abs : N → N
            }
            
            implements Numeric(ℝ) {
                operation abs = builtin_abs
            }
        "#;

        let result = parse_kleis_program(code).unwrap();
        assert_eq!(result.items.len(), 2);

        // Check structure
        match &result.items[0] {
            TopLevel::StructureDef(s) => {
                assert_eq!(s.name, "Numeric");
            }
            _ => panic!("Expected StructureDef"),
        }

        // Check implements
        match &result.items[1] {
            TopLevel::ImplementsDef(impl_def) => {
                assert_eq!(impl_def.structure_name, "Numeric");
                assert_eq!(impl_def.type_args.len(), 1);
                assert_eq!(impl_def.type_args[0], TypeExpr::Named("ℝ".to_string()));
            }
            _ => panic!("Expected ImplementsDef"),
        }

        // Use helper methods
        let structures = result.structures();
        let implements = result.implements();

        assert_eq!(structures.len(), 1);
        assert_eq!(implements.len(), 1);
    }

    // ===== Data Type Parser Tests =====

    #[test]
    fn test_parse_data_simple() {
        // data Bool = True | False
        let code = "data Bool = True | False";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_data_def().unwrap();

        assert_eq!(result.name, "Bool");
        assert_eq!(result.type_params.len(), 0);
        assert_eq!(result.variants.len(), 2);

        assert_eq!(result.variants[0].name, "True");
        assert!(result.variants[0].fields.is_empty());

        assert_eq!(result.variants[1].name, "False");
        assert!(result.variants[1].fields.is_empty());
    }

    #[test]
    fn test_parse_data_parametric() {
        // data Option(T) = None | Some(T)
        let code = "data Option(T) = None | Some(T)";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_data_def().unwrap();

        assert_eq!(result.name, "Option");
        assert_eq!(result.type_params.len(), 1);
        assert_eq!(result.type_params[0].name, "T");
        assert_eq!(result.variants.len(), 2);

        // None variant
        assert_eq!(result.variants[0].name, "None");
        assert!(result.variants[0].fields.is_empty());

        // Some variant with one positional field
        assert_eq!(result.variants[1].name, "Some");
        assert_eq!(result.variants[1].fields.len(), 1);
        assert!(result.variants[1].fields[0].name.is_none()); // Positional
        // Note: Type variables are parsed as Named types at this stage
        assert!(matches!(
            result.variants[1].fields[0].type_expr,
            TypeExpr::Named(ref s) if s == "T"
        ));
    }

    #[test]
    fn test_parse_data_with_named_fields() {
        // data Type = Scalar | Matrix(m: Nat, n: Nat)
        let code = "data Type = Scalar | Matrix(m: Nat, n: Nat)";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_data_def().unwrap();

        assert_eq!(result.name, "Type");
        assert!(result.type_params.is_empty());
        assert_eq!(result.variants.len(), 2);

        // Scalar variant
        assert_eq!(result.variants[0].name, "Scalar");
        assert!(result.variants[0].fields.is_empty());

        // Matrix variant with named fields
        let matrix_variant = &result.variants[1];
        assert_eq!(matrix_variant.name, "Matrix");
        assert_eq!(matrix_variant.fields.len(), 2);

        // First field: m: Nat
        assert_eq!(matrix_variant.fields[0].name, Some("m".to_string()));
        assert!(matches!(
            matrix_variant.fields[0].type_expr,
            TypeExpr::Named(ref s) if s == "Nat"
        ));

        // Second field: n: Nat
        assert_eq!(matrix_variant.fields[1].name, Some("n".to_string()));
        assert!(matches!(
            matrix_variant.fields[1].type_expr,
            TypeExpr::Named(ref s) if s == "Nat"
        ));
    }

    #[test]
    fn test_parse_data_multi_param() {
        // data Result(T, E) = Ok(value: T) | Err(error: E)
        let code = "data Result(T, E) = Ok(value: T) | Err(error: E)";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_data_def().unwrap();

        assert_eq!(result.name, "Result");
        assert_eq!(result.type_params.len(), 2);
        assert_eq!(result.type_params[0].name, "T");
        assert_eq!(result.type_params[1].name, "E");
        assert_eq!(result.variants.len(), 2);

        // Ok variant
        assert_eq!(result.variants[0].name, "Ok");
        assert_eq!(result.variants[0].fields.len(), 1);
        assert_eq!(result.variants[0].fields[0].name, Some("value".to_string()));

        // Err variant
        assert_eq!(result.variants[1].name, "Err");
        assert_eq!(result.variants[1].fields.len(), 1);
        assert_eq!(result.variants[1].fields[0].name, Some("error".to_string()));
    }

    #[test]
    fn test_parse_data_multiple_variants() {
        // data Color = Red | Green | Blue | Yellow
        let code = "data Color = Red | Green | Blue | Yellow";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_data_def().unwrap();

        assert_eq!(result.name, "Color");
        assert_eq!(result.variants.len(), 4);
        assert_eq!(result.variants[0].name, "Red");
        assert_eq!(result.variants[1].name, "Green");
        assert_eq!(result.variants[2].name, "Blue");
        assert_eq!(result.variants[3].name, "Yellow");
    }

    #[test]
    fn test_parse_data_with_type_params_in_fields() {
        // data Vector(n: Nat) = Vector(elements: List(ℝ))
        let code = "data Vector(n: Nat) = Vector(elements: List(ℝ))";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_data_def().unwrap();

        assert_eq!(result.name, "Vector");
        assert_eq!(result.type_params.len(), 1);
        assert_eq!(result.type_params[0].name, "n");
        assert_eq!(result.type_params[0].kind, Some("Nat".to_string()));

        assert_eq!(result.variants.len(), 1);
        let variant = &result.variants[0];
        assert_eq!(variant.name, "Vector");
        assert_eq!(variant.fields.len(), 1);
        assert_eq!(variant.fields[0].name, Some("elements".to_string()));
    }

    #[test]
    fn test_parse_program_with_data() {
        let code = r#"
            data Bool = True | False
            
            structure Money {
                amount : ℝ
            }
            
            data Option(T) = None | Some(T)
        "#;

        let result = parse_kleis_program(code).unwrap();

        let data_types = result.data_types();
        let structures = result.structures();

        assert_eq!(data_types.len(), 2);
        assert_eq!(structures.len(), 1);
        assert_eq!(data_types[0].name, "Bool");
        assert_eq!(data_types[1].name, "Option");
    }

    #[test]
    fn test_parse_data_whitespace_tolerance() {
        // Test with various whitespace
        let code = "data   Bool   =   True   |   False";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_data_def().unwrap();

        assert_eq!(result.name, "Bool");
        assert_eq!(result.variants.len(), 2);
    }

    #[test]
    fn test_parse_data_error_missing_equals() {
        let code = "data Bool True | False";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_data_def();

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Expected '='"));
    }

    #[test]
    fn test_parse_data_error_no_variants() {
        let code = "data Bool =";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_data_def();

        assert!(result.is_err());
    }

    // ===== Pattern Matching Parser Tests =====

    #[test]
    fn test_parse_pattern_wildcard() {
        let code = "_";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_pattern().unwrap();

        assert_eq!(result, Pattern::Wildcard);
    }

    #[test]
    fn test_parse_pattern_variable() {
        let code = "x";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_pattern().unwrap();

        assert_eq!(result, Pattern::Variable("x".to_string()));
    }

    #[test]
    fn test_parse_pattern_constant_number() {
        let code = "42";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_pattern().unwrap();

        assert_eq!(result, Pattern::Constant("42".to_string()));
    }

    #[test]
    fn test_parse_pattern_constructor_no_args() {
        let code = "None";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_pattern().unwrap();

        assert_eq!(
            result,
            Pattern::Constructor {
                name: "None".to_string(),
                args: vec![]
            }
        );
    }

    #[test]
    fn test_parse_pattern_constructor_one_arg() {
        let code = "Some(x)";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_pattern().unwrap();

        assert_eq!(
            result,
            Pattern::Constructor {
                name: "Some".to_string(),
                args: vec![Pattern::Variable("x".to_string())]
            }
        );
    }

    #[test]
    fn test_parse_pattern_constructor_multiple_args() {
        let code = "Pair(a, b)";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_pattern().unwrap();

        assert_eq!(
            result,
            Pattern::Constructor {
                name: "Pair".to_string(),
                args: vec![
                    Pattern::Variable("a".to_string()),
                    Pattern::Variable("b".to_string())
                ]
            }
        );
    }

    #[test]
    fn test_parse_pattern_nested() {
        let code = "Some(Pair(x, y))";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_pattern().unwrap();

        assert_eq!(
            result,
            Pattern::Constructor {
                name: "Some".to_string(),
                args: vec![Pattern::Constructor {
                    name: "Pair".to_string(),
                    args: vec![
                        Pattern::Variable("x".to_string()),
                        Pattern::Variable("y".to_string())
                    ]
                }]
            }
        );
    }

    #[test]
    fn test_parse_match_simple() {
        let code = "match x { True => 1 | False => 0 }";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Match { scrutinee, cases } => {
                assert_eq!(*scrutinee, Expression::Object("x".to_string()));
                assert_eq!(cases.len(), 2);

                // First case: True => 1
                assert_eq!(
                    cases[0].pattern,
                    Pattern::Constructor {
                        name: "True".to_string(),
                        args: vec![]
                    }
                );
                assert_eq!(cases[0].body, Expression::Const("1".to_string()));

                // Second case: False => 0
                assert_eq!(
                    cases[1].pattern,
                    Pattern::Constructor {
                        name: "False".to_string(),
                        args: vec![]
                    }
                );
                assert_eq!(cases[1].body, Expression::Const("0".to_string()));
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_parse_match_with_variable_binding() {
        let code = "match opt { None => 0 | Some(x) => x }";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Match { scrutinee, cases } => {
                assert_eq!(*scrutinee, Expression::Object("opt".to_string()));
                assert_eq!(cases.len(), 2);

                // First case: None => 0
                assert_eq!(
                    cases[0].pattern,
                    Pattern::Constructor {
                        name: "None".to_string(),
                        args: vec![]
                    }
                );

                // Second case: Some(x) => x
                assert_eq!(
                    cases[1].pattern,
                    Pattern::Constructor {
                        name: "Some".to_string(),
                        args: vec![Pattern::Variable("x".to_string())]
                    }
                );
                assert_eq!(cases[1].body, Expression::Object("x".to_string()));
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_parse_match_with_wildcard() {
        let code = "match status { Running => 1 | _ => 0 }";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Match { cases, .. } => {
                assert_eq!(cases.len(), 2);
                assert_eq!(
                    cases[0].pattern,
                    Pattern::Constructor {
                        name: "Running".to_string(),
                        args: vec![]
                    }
                );
                assert_eq!(cases[1].pattern, Pattern::Wildcard);
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_parse_match_with_nested_pattern() {
        let code = "match result { Ok(Some(x)) => x | Ok(None) => 0 | Err(_) => minus(1) }";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Match { cases, .. } => {
                assert_eq!(cases.len(), 3);

                // First case: Ok(Some(x)) => x
                assert_eq!(
                    cases[0].pattern,
                    Pattern::Constructor {
                        name: "Ok".to_string(),
                        args: vec![Pattern::Constructor {
                            name: "Some".to_string(),
                            args: vec![Pattern::Variable("x".to_string())]
                        }]
                    }
                );

                // Second case: Ok(None) => 0
                assert_eq!(
                    cases[1].pattern,
                    Pattern::Constructor {
                        name: "Ok".to_string(),
                        args: vec![Pattern::Constructor {
                            name: "None".to_string(),
                            args: vec![]
                        }]
                    }
                );

                // Third case: Err(_) => -1
                assert_eq!(
                    cases[2].pattern,
                    Pattern::Constructor {
                        name: "Err".to_string(),
                        args: vec![Pattern::Wildcard]
                    }
                );
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_parse_match_with_expressions_in_body() {
        let code = "match pair { Pair(a, b) => plus(a, b) }";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Match { cases, .. } => {
                assert_eq!(cases.len(), 1);
                match &cases[0].body {
                    Expression::Operation { name, args } => {
                        assert_eq!(name, "plus");
                        assert_eq!(args.len(), 2);
                    }
                    _ => panic!("Expected Operation in body"),
                }
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_parse_match_multiline() {
        let code = r#"match x {
            None => 0
            Some(value) => value
        }"#;
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Match { cases, .. } => {
                assert_eq!(cases.len(), 2);
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_parse_match_with_numbers() {
        let code = "match n { 0 => zero | 1 => one | _ => other }";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Match { cases, .. } => {
                assert_eq!(cases.len(), 3);
                assert_eq!(cases[0].pattern, Pattern::Constant("0".to_string()));
                assert_eq!(cases[1].pattern, Pattern::Constant("1".to_string()));
                assert_eq!(cases[2].pattern, Pattern::Wildcard);
            }
            _ => panic!("Expected Match expression"),
        }
    }

    #[test]
    fn test_parse_match_error_no_cases() {
        let code = "match x { }";
        let mut parser = KleisParser::new(code);
        let result = parser.parse();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("must have at least one case")
        );
    }

    #[test]
    fn test_parse_match_error_missing_arrow() {
        let code = "match x { Some(x) 5 }";
        let mut parser = KleisParser::new(code);
        let result = parser.parse();

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Expected '=>'"));
    }

    #[test]
    fn test_parse_match_error_missing_closing_brace() {
        let code = "match x { None => 0";
        let mut parser = KleisParser::new(code);
        let result = parser.parse();

        assert!(result.is_err());
    }
}
