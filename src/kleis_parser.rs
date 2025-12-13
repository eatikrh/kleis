//! Kleis Text Parser - Parses Kleis text syntax into AST
//!
//! **IMPORTANT:** This parser is evolving toward production readiness.
//! It implements ~60% of the formal Kleis v0.6 grammar.
//!
//! **What's Supported:**
//! - Function calls: abs(x), card(S), norm(v), frac(a, b)
//! - Operators: +, -, *, /, ^, ×, ·
//! - Comparison operators: <, >, <=, >=, ==, !=, =
//! - Logical operators: and, or, not
//! - Identifiers and numbers
//! - Parentheses for grouping
//! - Proper operator precedence
//! - Function definitions: define f(x) = x + x
//! - Data types: data Bool = True | False
//! - Pattern matching: match x { True => 1 | False => 0 }
//! - Structures and implementations
//! - ✅ Let bindings: let x = 5 in x^2 (NEW Dec 2024)
//! - ✅ Conditionals: if x > 0 then x else 0 (NEW Dec 2024)
//! - Vector/list literals: [1, 2, 3]
//! - Quantifiers: ∀(x : T). P(x)
//!
//! **What's NOT Supported (yet):**
//! - Prefix operators: -x, ∇f, √x
//! - Postfix operators: n!, Aᵀ, A†
//! - Lambda expressions: λ x . x^2
//! - Type annotations in expressions: x : ℝ
//! - Calculus operators as infix: ∫, ∂
//! - Symbolic constants: π, e, i
//! - Placeholders: □
//! - Summation/product notation: Σ, Π
//!
//! See docs/PARSER_GRAMMAR_COMPATIBILITY.md for full comparison with formal grammar.
//!
//! **Grammar (simplified):**
//!   expression := term (('+' | '-') term)*
//!   term := factor (('*' | '/') factor)*
//!   factor := primary ('^' primary)?
//!   primary := identifier | number | function_call | '(' expression ')'
//!   function_call := identifier '(' arguments ')'
//!   arguments := expression (',' expression)*
//!
//! **Purpose:** Validate ADR-015 design decisions, not production-ready!
use crate::ast::{Expression, MatchCase, Pattern};
use crate::kleis_ast::{
    DataDef, DataField, DataVariant, FunctionDef, ImplMember, Implementation, ImplementsDef,
    OperationDecl, Program, StructureDef, StructureMember, TopLevel, TypeExpr,
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
        if !self.peek().is_some_and(|ch| ch.is_numeric()) {
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
        // Only consume '.' if there's a digit after it
        if self.peek() == Some('.') && self.peek_ahead(1).is_some_and(|ch| ch.is_numeric()) {
            self.advance(); // consume '.'
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

    fn parse_list_literal(&mut self) -> Result<Expression, KleisParseError> {
        // Consume '['
        assert_eq!(self.advance(), Some('['));

        let mut elements = Vec::new();

        // Empty list
        self.skip_whitespace();
        if self.peek() == Some(']') {
            self.advance();
            return Ok(Expression::List(elements));
        }

        // Parse comma-separated expressions
        loop {
            self.skip_whitespace();
            elements.push(self.parse_expression()?);
            self.skip_whitespace();

            match self.peek() {
                Some(',') => {
                    self.advance();
                    continue;
                }
                Some(']') => {
                    self.advance();
                    break;
                }
                _ => {
                    return Err(KleisParseError {
                        message: "Expected ',' or ']' in list literal".to_string(),
                        position: self.pos,
                    });
                }
            }
        }

        Ok(Expression::List(elements))
    }

    fn parse_primary(&mut self) -> Result<Expression, KleisParseError> {
        self.skip_whitespace();

        // Unary minus: -x (prefix operator)
        if self.peek() == Some('-') {
            self.advance(); // consume -
            let arg = self.parse_primary()?;
            return Ok(Expression::Operation {
                name: "negate".to_string(),
                args: vec![arg],
            });
        }

        // Negation: ¬A or not A (prefix operator)
        if self.peek() == Some('¬') {
            self.advance(); // consume ¬
            let arg = self.parse_primary()?;
            return Ok(Expression::Operation {
                name: "logical_not".to_string(),
                args: vec![arg],
            });
        }

        // Gradient: ∇f (nabla prefix operator)
        if self.peek() == Some('∇') {
            self.advance(); // consume ∇
            let arg = self.parse_primary()?;
            return Ok(Expression::Operation {
                name: "gradient".to_string(),
                args: vec![arg],
            });
        }

        // Partial derivative: ∂f (partial prefix operator)
        // For ∂/∂x notation, we'll need more complex parsing
        if self.peek() == Some('∂') {
            self.advance(); // consume ∂
            let arg = self.parse_primary()?;
            return Ok(Expression::Operation {
                name: "partial".to_string(),
                args: vec![arg],
            });
        }

        // Match expression
        if self.peek_word("match") {
            return self.parse_match_expr();
        }

        // Conditional: if cond then a else b
        if self.peek_word("if") {
            return self.parse_conditional();
        }

        // Let binding: let x = expr in body
        if self.peek_word("let") {
            return self.parse_let_binding();
        }

        // List literal: [a, b, c]
        if self.peek() == Some('[') {
            return self.parse_list_literal();
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
        if self.peek().is_some_and(|ch| ch.is_numeric()) {
            let num = self.parse_number()?;
            return Ok(Expression::Const(num));
        }

        // Identifier or function call
        if self
            .peek()
            .is_some_and(|ch| ch.is_alphabetic() || ch == '_')
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
        // Parse logical expression (lowest precedence)
        self.parse_implication()
    }

    /// Parse implication: A ⟹ B
    fn parse_implication(&mut self) -> Result<Expression, KleisParseError> {
        let mut left = self.parse_disjunction()?;

        loop {
            self.skip_whitespace();
            let is_implies = self.peek() == Some('⟹');

            if !is_implies {
                break;
            }

            self.advance(); // consume ⟹
            let right = self.parse_disjunction()?;
            left = Expression::Operation {
                name: "implies".to_string(),
                args: vec![left, right],
            };
        }

        Ok(left)
    }

    /// Parse disjunction: A ∨ B (logical or)
    fn parse_disjunction(&mut self) -> Result<Expression, KleisParseError> {
        let mut left = self.parse_conjunction()?;

        loop {
            self.skip_whitespace();
            let is_or = self.peek() == Some('∨');

            if !is_or {
                break;
            }

            self.advance(); // consume ∨
            let right = self.parse_conjunction()?;
            left = Expression::Operation {
                name: "logical_or".to_string(),
                args: vec![left, right],
            };
        }

        Ok(left)
    }

    /// Parse conjunction: A ∧ B (logical and)
    fn parse_conjunction(&mut self) -> Result<Expression, KleisParseError> {
        let mut left = self.parse_comparison()?;

        loop {
            self.skip_whitespace();
            let is_and = self.peek() == Some('∧');

            if !is_and {
                break;
            }

            self.advance(); // consume ∧
            let right = self.parse_comparison()?;
            left = Expression::Operation {
                name: "logical_and".to_string(),
                args: vec![left, right],
            };
        }

        Ok(left)
    }

    /// Parse comparison: A = B, A < B, A <= B, A >= B, etc.
    fn parse_comparison(&mut self) -> Result<Expression, KleisParseError> {
        let left = self.parse_arithmetic()?;

        self.skip_whitespace();

        // Check for two-character comparison operators first
        let op = if let Some(two_char) = self.peek_n(2) {
            match two_char.as_str() {
                "<=" => {
                    self.pos += 2;
                    Some("leq")
                }
                ">=" => {
                    self.pos += 2;
                    Some("geq")
                }
                "!=" => {
                    self.pos += 2;
                    Some("neq")
                }
                "==" => {
                    self.pos += 2;
                    Some("equals")
                }
                _ => None,
            }
        } else {
            None
        };

        // If no two-char operator, check single-char operators
        let op = op.or_else(|| {
            match self.peek() {
                Some('=') => {
                    // Check if it's not ⟹ (which is handled at higher level)
                    if self.peek_ahead(1) != Some('⟹') && self.peek_ahead(1) != Some('=') {
                        self.advance();
                        Some("equals")
                    } else {
                        None
                    }
                }
                Some('<') => {
                    // Make sure it's not <= (already handled above)
                    if self.peek_ahead(1) != Some('=') {
                        self.advance();
                        Some("less_than")
                    } else {
                        None
                    }
                }
                Some('>') => {
                    // Make sure it's not >= (already handled above)
                    if self.peek_ahead(1) != Some('=') {
                        self.advance();
                        Some("greater_than")
                    } else {
                        None
                    }
                }
                Some('≤') => {
                    self.advance();
                    Some("leq")
                }
                Some('≥') => {
                    self.advance();
                    Some("geq")
                }
                Some('≠') => {
                    self.advance();
                    Some("neq")
                }
                _ => None,
            }
        });

        if let Some(op) = op {
            let right = self.parse_arithmetic()?;
            Ok(Expression::Operation {
                name: op.to_string(),
                args: vec![left, right],
            })
        } else {
            Ok(left)
        }
    }

    /// Check if a character is a custom mathematical operator
    /// Includes Unicode math symbols like •, ⊗, ⊕, ∘, etc.
    fn is_custom_operator_char(&self, ch: char) -> bool {
        match ch {
            // Common mathematical operators (Unicode Symbol, Math category)
            '•' | '∘' | '∗' | '⋆' | '⊗' | '⊕' | '⊙' | '⊛' | '⊘' | '⊚' | '⊝' | '⊞' | '⊟' | '⊠'
            | '⊡' | '⨀' | '⨁' | '⨂' | '⨃' | '⨄' | '⊓' | '⊔' | '⊎' | '⊍' | '∪' | '∩' | '⋃' | '⋂'
            | '△' | '▽' => true,

            // Exclude operators already handled explicitly
            '+' | '-' | '*' | '/' | '^' | '×' | '·' => false,

            // Exclude comparison operators (handled separately)
            '=' | '<' | '>' | '≤' | '≥' | '≠' => false,

            // Exclude logical operators (handled separately)
            '∧' | '∨' | '¬' | '⟹' => false,

            // Exclude delimiters
            '(' | ')' | '[' | ']' | '{' | '}' | ',' | '.' | ':' | ';' => false,

            _ => false,
        }
    }

    /// Try to parse a custom operator (single Unicode math symbol)
    fn try_parse_custom_operator(&mut self) -> Option<String> {
        match self.peek() {
            Some(ch) if self.is_custom_operator_char(ch) => {
                self.advance();
                Some(ch.to_string())
            }
            _ => None,
        }
    }

    /// Parse arithmetic expressions: +, -, and custom operators
    fn parse_arithmetic(&mut self) -> Result<Expression, KleisParseError> {
        let mut left = self.parse_term()?;

        loop {
            self.skip_whitespace();

            // Try built-in operators first
            let op = match self.peek() {
                Some('+') => {
                    self.advance();
                    Some("plus".to_string())
                }
                Some('-') => {
                    self.advance();
                    Some("minus".to_string())
                }
                _ => {
                    // Try custom operators (like •, ⊗, ⊕, etc.)
                    self.try_parse_custom_operator()
                }
            };

            if let Some(op) = op {
                let right = self.parse_term()?;
                left = Expression::Operation {
                    name: op,
                    args: vec![left, right],
                };
            } else {
                break;
            }
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

    /// Parse operation name (identifier or operator symbol)
    /// Examples: abs, transpose, (+), (×), (•)
    fn parse_operation_name(&mut self) -> Result<String, KleisParseError> {
        self.skip_whitespace();

        // Check if it's an operator in parentheses
        if self.peek() == Some('(') {
            self.advance(); // consume '('
            self.skip_whitespace();

            // Parse operator symbol
            let op_symbol = match self.peek() {
                // Arithmetic
                Some('+') => "+",
                Some('-') => "-",
                Some('*') => "*",
                Some('/') => "/",
                Some('^') => "^",
                Some('×') => "×",
                Some('·') => "·",
                Some('•') => "•",
                // Comparisons
                Some('=') => "=",
                Some('<') => "<",
                Some('>') => ">",
                Some('≤') => "≤",
                Some('≥') => "≥",
                Some('≠') => "≠",
                // Logical
                Some('∧') => "∧",
                Some('∨') => "∨",
                Some('¬') => "¬",
                Some('⟹') => "⟹",
                // Algebra
                Some('∘') => "∘", // composition
                Some('⊗') => "⊗", // tensor product
                Some('⊕') => "⊕", // direct sum
                _ => {
                    return Err(KleisParseError {
                        message: format!("Expected operator symbol, got {:?}", self.peek()),
                        position: self.pos,
                    });
                }
            };

            self.advance(); // consume operator
            self.skip_whitespace();

            if self.advance() != Some(')') {
                return Err(KleisParseError {
                    message: "Expected ')' after operator symbol".to_string(),
                    position: self.pos,
                });
            }

            Ok(op_symbol.to_string())
        } else {
            // Regular identifier
            self.parse_identifier()
        }
    }

    /// Parse a proposition (for axioms)
    /// Supports quantifiers: ∀(x : T). body
    pub fn parse_proposition(&mut self) -> Result<Expression, KleisParseError> {
        self.skip_whitespace();

        // Check for quantifier
        if let Some('∀') | Some('∃') = self.peek() {
            self.parse_quantifier()
        } else if self.peek_word("forall") || self.peek_word("exists") {
            self.parse_quantifier()
        } else {
            // Just an expression
            self.parse_expression()
        }
    }

    /// Parse a quantifier expression
    /// ∀(x : M). x • e = x
    /// ∀(x y z : R). (x + y) + z = x + (y + z)
    fn parse_quantifier(&mut self) -> Result<Expression, KleisParseError> {
        self.skip_whitespace();

        // Parse quantifier symbol
        let quantifier = if self.peek() == Some('∀') {
            self.advance();
            crate::ast::QuantifierKind::ForAll
        } else if self.peek() == Some('∃') {
            self.advance();
            crate::ast::QuantifierKind::Exists
        } else if self.consume_word("forall") {
            crate::ast::QuantifierKind::ForAll
        } else if self.consume_word("exists") {
            crate::ast::QuantifierKind::Exists
        } else {
            return Err(KleisParseError {
                message: "Expected quantifier (∀, ∃, forall, or exists)".to_string(),
                position: self.pos,
            });
        };

        self.skip_whitespace();

        // Expect '('
        if self.advance() != Some('(') {
            return Err(KleisParseError {
                message: "Expected '(' after quantifier".to_string(),
                position: self.pos,
            });
        }

        // Parse variable list: x : T or x y z : T
        let variables = self.parse_quantified_vars()?;

        self.skip_whitespace();

        // Expect ')'
        if self.advance() != Some(')') {
            return Err(KleisParseError {
                message: "Expected ')' after quantified variables".to_string(),
                position: self.pos,
            });
        }

        self.skip_whitespace();

        // Optional where clause: where x ≠ zero
        let where_clause = if self.peek_word("where") {
            // Skip "where"
            for _ in 0..5 {
                self.advance();
            }
            self.skip_whitespace();

            // Parse condition expression (until we hit '.')
            // We need to parse a comparison but stop at '.'
            let condition = self.parse_where_condition()?;
            Some(Box::new(condition))
        } else {
            None
        };

        self.skip_whitespace();

        // Expect '.'
        if self.advance() != Some('.') {
            return Err(KleisParseError {
                message: "Expected '.' after quantified variables".to_string(),
                position: self.pos,
            });
        }

        // Parse body (recursively, to allow nested quantifiers)
        let body = self.parse_proposition()?;

        Ok(Expression::Quantifier {
            quantifier,
            variables,
            where_clause,
            body: Box::new(body),
        })
    }

    /// Parse where condition in quantifier (stops at '.')
    /// Example: x ≠ zero, x > 0, x • y = e
    fn parse_where_condition(&mut self) -> Result<Expression, KleisParseError> {
        // Parse left side
        let left = self.parse_where_term()?;

        self.skip_whitespace();

        // Check for comparison operator
        let op = match self.peek() {
            Some('=') => {
                self.advance();
                Some("equals")
            }
            Some('<') => {
                self.advance();
                Some("less_than")
            }
            Some('>') => {
                self.advance();
                Some("greater_than")
            }
            Some('≤') => {
                self.advance();
                Some("leq")
            }
            Some('≥') => {
                self.advance();
                Some("geq")
            }
            Some('≠') => {
                self.advance();
                Some("neq")
            }
            _ => None,
        };

        if let Some(op) = op {
            self.skip_whitespace();
            let right = self.parse_where_term()?;
            Ok(Expression::Operation {
                name: op.to_string(),
                args: vec![left, right],
            })
        } else {
            Ok(left)
        }
    }

    /// Parse a term in where condition (stops at '.', '=', '<', '>', etc.)
    fn parse_where_term(&mut self) -> Result<Expression, KleisParseError> {
        self.skip_whitespace();

        // Parse primary expressions and custom operators
        // But stop at comparison operators and '.'
        let mut left = self.parse_primary()?;

        loop {
            self.skip_whitespace();

            // Stop at comparison operators or '.'
            match self.peek() {
                Some('.') | Some('=') | Some('<') | Some('>') | Some('≤') | Some('≥')
                | Some('≠') => {
                    break;
                }
                _ => {}
            }

            // Check for custom operator
            if let Some(op) = self.try_parse_custom_operator() {
                let right = self.parse_primary()?;
                left = Expression::Operation {
                    name: op,
                    args: vec![left, right],
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Parse quantified variables: x : T or x y z : T or (c : F, u v : V)
    /// Supports comma-separated type groups: ∀(c : F, u v : V). body
    fn parse_quantified_vars(&mut self) -> Result<Vec<crate::ast::QuantifiedVar>, KleisParseError> {
        let mut all_vars = Vec::new();

        // Parse comma-separated groups of variables
        loop {
            self.skip_whitespace();

            // Check if we're done (hit closing paren or 'where')
            if self.peek() == Some(')') || self.peek_word("where") {
                break;
            }

            // Collect variable names until we hit ':'
            let mut names = Vec::new();
            loop {
                self.skip_whitespace();

                // Check if we hit the colon
                if self.peek() == Some(':') {
                    break;
                }

                // Parse identifier
                let name = self.parse_identifier()?;
                names.push(name);

                self.skip_whitespace();

                // Could be another variable or the colon
                if self.peek() == Some(':') {
                    break;
                }
            }

            if names.is_empty() {
                return Err(KleisParseError {
                    message: "Expected at least one variable name".to_string(),
                    position: self.pos,
                });
            }

            self.skip_whitespace();

            // Expect ':'
            if self.advance() != Some(':') {
                return Err(KleisParseError {
                    message: "Expected ':' after variable names".to_string(),
                    position: self.pos,
                });
            }

            self.skip_whitespace();

            // Parse type annotation
            let type_name = self.parse_identifier()?;

            // Create QuantifiedVar for each name with the same type
            for name in names {
                all_vars.push(crate::ast::QuantifiedVar::new(
                    name,
                    Some(type_name.clone()),
                ));
            }

            self.skip_whitespace();

            // Check for comma (more variable groups) or end
            if self.peek() == Some(',') {
                self.advance(); // consume comma
                                // Continue to parse next group
            } else {
                // No more groups
                break;
            }
        }

        if all_vars.is_empty() {
            return Err(KleisParseError {
                message: "Expected at least one quantified variable".to_string(),
                position: self.pos,
            });
        }

        Ok(all_vars)
    }

    /// Check if the next word matches (without consuming)
    fn peek_word(&self, word: &str) -> bool {
        let mut temp_pos = self.pos;

        // Skip whitespace
        while temp_pos < self.input.len() && self.input[temp_pos].is_whitespace() {
            temp_pos += 1;
        }

        // Check if word matches
        for ch in word.chars() {
            if temp_pos >= self.input.len() || self.input[temp_pos] != ch {
                return false;
            }
            temp_pos += 1;
        }

        // Check that it's followed by non-identifier character
        if temp_pos < self.input.len() {
            let next = self.input[temp_pos];
            if next.is_alphanumeric() || next == '_' {
                return false;
            }
        }

        true
    }

    /// Consume a word if it matches
    fn consume_word(&mut self, word: &str) -> bool {
        if self.peek_word(word) {
            // Skip whitespace
            self.skip_whitespace();
            // Consume the word
            for _ in word.chars() {
                self.advance();
            }
            true
        } else {
            false
        }
    }

    /// Parse a type expression
    /// Examples: ℝ, Vector(3), Set(ℤ), ℝ → ℝ, ∀(n : ℕ). Vector(n) → ℝ
    pub fn parse_type(&mut self) -> Result<TypeExpr, KleisParseError> {
        self.skip_whitespace();

        // Check for quantified type: ∀(vars). body
        if self.peek() == Some('∀') || self.peek_word("forall") {
            return self.parse_forall_type();
        }

        // Parse base type - could be identifier or number (for dimension literals)
        let base_name = if self.peek().is_some_and(|ch| ch.is_numeric()) {
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

    /// Parse a quantified (forall) type
    /// Examples:
    ///   ∀(n : ℕ). Vector(n) → ℝ
    ///   ∀(m n p : ℕ, T). Matrix(m,n,T) × Matrix(n,p,T) → Matrix(m,p,T)
    fn parse_forall_type(&mut self) -> Result<TypeExpr, KleisParseError> {
        self.skip_whitespace();

        // Parse quantifier symbol
        if self.peek() == Some('∀') {
            self.advance();
        } else if self.consume_word("forall") {
            // Already consumed
        } else {
            return Err(KleisParseError {
                message: "Expected '∀' or 'forall'".to_string(),
                position: self.pos,
            });
        }

        self.skip_whitespace();

        // Expect '('
        if self.advance() != Some('(') {
            return Err(KleisParseError {
                message: "Expected '(' after forall quantifier".to_string(),
                position: self.pos,
            });
        }

        // Parse variable declarations
        // Can be: n : ℕ  or  m n p : ℕ, T : Type
        let mut vars = Vec::new();

        loop {
            self.skip_whitespace();

            if self.peek() == Some(')') {
                break;
            }

            // Parse variable names (can be multiple with same type)
            let mut var_names = Vec::new();
            var_names.push(self.parse_identifier()?);

            self.skip_whitespace();

            // Check for more variable names before the colon or delimiter
            while self.peek() != Some(':') && self.peek() != Some(',') && self.peek() != Some(')') {
                var_names.push(self.parse_identifier()?);
                self.skip_whitespace();
            }

            // Type annotation is optional - if no ':', use implicit Type kind
            let var_type = if self.peek() == Some(':') {
                self.advance(); // consume ':'
                self.skip_whitespace();
                self.parse_type()?
            } else {
                // No type annotation - implicit Type kind for type variables
                TypeExpr::Named("Type".to_string())
            };

            // Add all variables with this type
            for var_name in var_names {
                vars.push((var_name, var_type.clone()));
            }

            self.skip_whitespace();

            // Check for comma (more variables) or closing paren
            if self.peek() == Some(',') {
                self.advance();
            } else if self.peek() == Some(')') {
                break;
            } else {
                return Err(KleisParseError {
                    message: "Expected ',' or ')' in forall variable list".to_string(),
                    position: self.pos,
                });
            }
        }

        // Consume ')'
        if self.advance() != Some(')') {
            return Err(KleisParseError {
                message: "Expected ')' after forall variables".to_string(),
                position: self.pos,
            });
        }

        self.skip_whitespace();

        // Expect '.'
        if self.advance() != Some('.') {
            return Err(KleisParseError {
                message: "Expected '.' after forall variables".to_string(),
                position: self.pos,
            });
        }

        self.skip_whitespace();

        // Parse the body type
        let body = self.parse_type()?;

        Ok(TypeExpr::ForAll {
            vars,
            body: Box::new(body),
        })
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        let pos = self.pos + offset;
        if pos < self.input.len() {
            Some(self.input[pos])
        } else {
            None
        }
    }

    /// Peek at the next n characters as a string
    fn peek_n(&self, n: usize) -> Option<String> {
        if self.pos + n <= self.input.len() {
            Some(self.input[self.pos..self.pos + n].iter().collect())
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

    /// Parse a conditional (if-then-else) expression
    /// Grammar: if expression then expression else expression
    ///
    /// Examples:
    ///   if x > 0 then x else -x
    ///   if condition then result1 else result2
    ///
    /// Note: The condition expression is parsed at comparison precedence level
    /// to avoid ambiguity with infix operators.
    fn parse_conditional(&mut self) -> Result<Expression, KleisParseError> {
        // Consume 'if' keyword
        self.expect_word("if")?;
        self.skip_whitespace();

        // Parse condition expression
        // We need to parse until we hit 'then', so use a limited expression parse
        let condition = self.parse_conditional_part("then")?;
        self.skip_whitespace();

        // Expect 'then' keyword
        self.expect_word("then")?;
        self.skip_whitespace();

        // Parse then branch
        let then_branch = self.parse_conditional_part("else")?;
        self.skip_whitespace();

        // Expect 'else' keyword
        self.expect_word("else")?;
        self.skip_whitespace();

        // Parse else branch (can be full expression)
        let else_branch = self.parse_expression()?;

        Ok(Expression::Conditional {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    /// Parse a let binding expression
    /// Grammar: let identifier = expression in expression
    ///
    /// Examples:
    ///   let x = 5 in x + x
    ///   let squared = x * x in squared + 1
    ///   let a = 1 in let b = 2 in a + b  (nested)
    ///
    /// Used within function definitions to introduce local bindings.
    /// Pure semantics: the bound value is substituted into the body.
    fn parse_let_binding(&mut self) -> Result<Expression, KleisParseError> {
        // Consume 'let' keyword
        self.expect_word("let")?;
        self.skip_whitespace();

        // Parse variable name
        let name = self.parse_identifier()?;
        self.skip_whitespace();

        // Expect '='
        self.expect_char('=')?;
        self.skip_whitespace();

        // Parse the value expression (stops at 'in')
        let value = self.parse_let_value()?;
        self.skip_whitespace();

        // Expect 'in' keyword
        self.expect_word("in")?;
        self.skip_whitespace();

        // Parse the body expression
        let body = self.parse_expression()?;

        Ok(Expression::Let {
            name,
            value: Box::new(value),
            body: Box::new(body),
        })
    }

    /// Parse the value part of a let binding (stops at 'in')
    fn parse_let_value(&mut self) -> Result<Expression, KleisParseError> {
        // Parse terms and operators until we hit 'in'
        let mut left = self.parse_primary()?;

        loop {
            self.skip_whitespace();

            // Check if we've hit 'in'
            if self.peek_word("in") {
                break;
            }

            // Try to parse an infix operator
            if let Some(op) = self.try_parse_infix_operator() {
                self.skip_whitespace();
                let right = self.parse_primary()?;
                left = Expression::Operation {
                    name: op,
                    args: vec![left, right],
                };
            } else {
                // No more operators, stop
                break;
            }
        }

        Ok(left)
    }

    /// Parse an expression that's part of a conditional (stops at 'then' or 'else')
    ///
    /// This is needed because `if a + b then` would otherwise try to parse
    /// `then` as part of the expression.
    fn parse_conditional_part(&mut self, stop_word: &str) -> Result<Expression, KleisParseError> {
        // Use the proper precedence-respecting expression parser
        // We need to parse a complete expression, but stop before the stop_word
        // The key insight: comparison expressions are complete units
        // So we parse at the comparison level, which respects precedence

        // First, check if the expression starts with a negation or other prefix
        self.skip_whitespace();

        // Try to peek and see if we're about to hit the stop word immediately
        if self.peek_word(stop_word) {
            return Err(KleisParseError {
                position: self.pos,
                message: format!("Expected expression before '{}'", stop_word),
            });
        }

        // Parse using the proper precedence chain, but stop at logical operators
        // since those might interfere with the stop word
        self.parse_conditional_part_with_precedence(stop_word)
    }

    /// Parse a conditional part respecting operator precedence
    fn parse_conditional_part_with_precedence(
        &mut self,
        stop_word: &str,
    ) -> Result<Expression, KleisParseError> {
        // Parse comparison (which includes arithmetic with proper precedence)
        let mut expr = self.parse_comparison()?;

        // Continue parsing logical operators if present and we haven't hit stop_word
        loop {
            self.skip_whitespace();

            // Check if we've hit the stop word
            if self.peek_word(stop_word) {
                break;
            }

            // Check for logical operators (lower precedence than comparison)
            if self.peek() == Some('∧') {
                self.advance();
                self.skip_whitespace();
                let right = self.parse_comparison()?;
                expr = Expression::Operation {
                    name: "logical_and".to_string(),
                    args: vec![expr, right],
                };
            } else if self.peek() == Some('∨') {
                self.advance();
                self.skip_whitespace();
                let right = self.parse_comparison()?;
                expr = Expression::Operation {
                    name: "logical_or".to_string(),
                    args: vec![expr, right],
                };
            } else if self.peek_n(2).as_deref() == Some("&&") {
                self.pos += 2;
                self.skip_whitespace();
                let right = self.parse_comparison()?;
                expr = Expression::Operation {
                    name: "logical_and".to_string(),
                    args: vec![expr, right],
                };
            } else if self.peek_n(2).as_deref() == Some("||") {
                self.pos += 2;
                self.skip_whitespace();
                let right = self.parse_comparison()?;
                expr = Expression::Operation {
                    name: "logical_or".to_string(),
                    args: vec![expr, right],
                };
            } else {
                // No more operators, stop
                break;
            }
        }

        Ok(expr)
    }

    /// Try to parse an infix operator, returning None if not found
    fn try_parse_infix_operator(&mut self) -> Option<String> {
        let start_pos = self.pos;

        // Check for comparison operators first (two chars)
        if self.pos + 1 < self.input.len() {
            let two_chars: String = self.input[self.pos..self.pos + 2].iter().collect();
            match two_chars.as_str() {
                "==" => {
                    self.pos += 2;
                    return Some("equals".to_string());
                }
                "!=" | "≠" => {
                    self.pos += 2;
                    return Some("not_equals".to_string());
                }
                "<=" | "≤" => {
                    self.pos += 2;
                    return Some("leq".to_string());
                }
                ">=" | "≥" => {
                    self.pos += 2;
                    return Some("geq".to_string());
                }
                "&&" => {
                    self.pos += 2;
                    return Some("logical_and".to_string());
                }
                "||" => {
                    self.pos += 2;
                    return Some("logical_or".to_string());
                }
                _ => {}
            }
        }

        // Single character operators
        if let Some(ch) = self.peek() {
            let op = match ch {
                '+' => Some("plus".to_string()),
                '-' => Some("minus".to_string()),
                '*' | '×' => Some("times".to_string()),
                '/' | '÷' => Some("divide".to_string()),
                '<' => Some("less_than".to_string()),
                '>' => Some("greater_than".to_string()),
                '=' => Some("equals".to_string()),
                '∧' => Some("logical_and".to_string()),
                '∨' => Some("logical_or".to_string()),
                '→' | '⟹' => Some("implies".to_string()),
                _ => None,
            };

            if op.is_some() {
                self.advance();
                return op;
            }
        }

        // Check for word operators (and, or)
        if self.peek_word("and") {
            self.expect_word("and").ok()?;
            return Some("logical_and".to_string());
        }
        if self.peek_word("or") {
            self.expect_word("or").ok()?;
            return Some("logical_or".to_string());
        }

        // Restore position if nothing matched
        self.pos = start_pos;
        None
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
                .is_none_or(|ch| !ch.is_alphanumeric() && ch != '_')
            {
                return Ok(Pattern::wildcard());
            }
            // Otherwise, it's an identifier starting with underscore
            self.pos = start_pos;
        }

        // Number constant
        if self.peek().is_some_and(|ch| ch.is_numeric()) {
            let num = self.parse_number()?;
            return Ok(Pattern::constant(num));
        }

        // Constructor or variable
        if self
            .peek()
            .is_some_and(|ch| ch.is_alphabetic() || ch == '_')
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

    /// Parse nested structure definition
    /// Example: structure additive : AbelianGroup(R) { ... }
    fn parse_nested_structure(&mut self) -> Result<StructureMember, KleisParseError> {
        // Skip "structure" keyword
        for _ in 0..9 {
            self.advance();
        }
        self.skip_whitespace();

        // Parse nested structure name
        let name = self.parse_identifier()?;
        self.skip_whitespace();

        // Expect ':'
        if self.advance() != Some(':') {
            return Err(KleisParseError {
                message: format!("Expected ':' after nested structure name '{}'", name),
                position: self.pos,
            });
        }

        self.skip_whitespace();

        // Parse structure type (e.g., AbelianGroup(R))
        let structure_type = self.parse_type()?;
        self.skip_whitespace();

        // Parse optional body { ... }
        let members = if self.peek() == Some('{') {
            self.advance(); // consume '{'
            let mut nested_members = Vec::new();

            loop {
                self.skip_whitespace();

                if self.peek() == Some('}') {
                    break;
                }

                // Recursively parse structure members
                // (nested structures can contain nested structures!)
                let start_pos = self.pos;
                if self.peek_word("structure") {
                    nested_members.push(self.parse_nested_structure()?);
                } else if self.peek_word("operation") {
                    // Parse operation
                    for _ in 0..9 {
                        self.advance();
                    }
                    self.skip_whitespace();

                    let op_name = self.parse_operation_name()?;
                    self.skip_whitespace();

                    if self.advance() != Some(':') {
                        return Err(KleisParseError {
                            message: "Expected ':' after operation name".to_string(),
                            position: self.pos,
                        });
                    }

                    let type_sig = self.parse_type()?;

                    nested_members.push(StructureMember::Operation {
                        name: op_name,
                        type_signature: type_sig,
                    });
                } else if self.peek_word("element") {
                    // element e : M (same as nullary operation)
                    for _ in 0..7 {
                        self.advance();
                    }
                    self.skip_whitespace();

                    let elem_name = self.parse_identifier()?;
                    self.skip_whitespace();

                    if self.advance() != Some(':') {
                        return Err(KleisParseError {
                            message: "Expected ':' after element name".to_string(),
                            position: self.pos,
                        });
                    }

                    let type_sig = self.parse_type()?;

                    nested_members.push(StructureMember::Operation {
                        name: elem_name,
                        type_signature: type_sig,
                    });
                } else if self.peek_word("axiom") {
                    // Parse axiom
                    for _ in 0..5 {
                        self.advance();
                    }
                    self.skip_whitespace();

                    let axiom_name = self.parse_identifier()?;
                    self.skip_whitespace();

                    if self.advance() != Some(':') {
                        return Err(KleisParseError {
                            message: "Expected ':' after axiom name".to_string(),
                            position: self.pos,
                        });
                    }

                    self.skip_whitespace();
                    let proposition = self.parse_proposition()?;

                    nested_members.push(StructureMember::Axiom {
                        name: axiom_name,
                        proposition,
                    });
                } else {
                    // Regular field
                    self.pos = start_pos;
                    nested_members.push(self.parse_structure_member()?);
                }
            }

            if self.advance() != Some('}') {
                return Err(KleisParseError {
                    message: "Expected '}' after nested structure body".to_string(),
                    position: self.pos,
                });
            }

            nested_members
        } else {
            // No body - just a reference to existing structure
            Vec::new()
        };

        Ok(StructureMember::NestedStructure {
            name,
            structure_type,
            members,
        })
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

        self.skip_whitespace();

        // Optional extends clause: extends ParentStructure(Args)
        let extends_clause = if self.peek_word("extends") {
            // Skip "extends"
            for _ in 0..7 {
                self.advance();
            }
            self.skip_whitespace();

            // Parse parent structure type (e.g., Semigroup(M))
            Some(self.parse_type()?)
        } else {
            None
        };

        self.skip_whitespace();

        // Optional over clause: over Field(F)
        let over_clause = if self.peek_word("over") {
            // Skip "over"
            for _ in 0..4 {
                self.advance();
            }
            self.skip_whitespace();

            // Parse field type (e.g., Field(F))
            Some(self.parse_type()?)
        } else {
            None
        };

        self.skip_whitespace();

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

            // Check for nested structure, operation, or axiom keyword
            let start_pos = self.pos;
            if self.peek_word("structure") {
                // Nested structure definition
                members.push(self.parse_nested_structure()?);
            } else if self.peek_word("operation") {
                // Skip "operation"
                for _ in 0..9 {
                    self.advance();
                }
                self.skip_whitespace();

                // Parse operation name (could be identifier or operator symbol)
                let op_name = self.parse_operation_name()?;
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
            } else if self.peek_word("element") {
                // element e : M
                // This is semantically equivalent to a nullary operation: operation e : M
                // Skip "element"
                for _ in 0..7 {
                    self.advance();
                }
                self.skip_whitespace();

                // Parse element name
                let elem_name = self.parse_identifier()?;
                self.skip_whitespace();

                if self.advance() != Some(':') {
                    return Err(KleisParseError {
                        message: "Expected ':' after element name".to_string(),
                        position: self.pos,
                    });
                }

                let type_sig = self.parse_type()?;

                // Store as Operation (nullary operation = identity element)
                members.push(StructureMember::Operation {
                    name: elem_name,
                    type_signature: type_sig,
                });
            } else if self.peek_word("define") {
                // define f(x) = expr (inline function definition in structure)
                // Grammar v0.6: functionDef is now allowed in structureMember
                let func_def = self.parse_function_def()?;
                members.push(StructureMember::FunctionDef(func_def));
            } else if self.peek_word("axiom") {
                // Skip "axiom"
                for _ in 0..5 {
                    self.advance();
                }
                self.skip_whitespace();

                // Parse axiom name
                let axiom_name = self.parse_identifier()?;
                self.skip_whitespace();

                if self.advance() != Some(':') {
                    return Err(KleisParseError {
                        message: "Expected ':' after axiom name".to_string(),
                        position: self.pos,
                    });
                }

                self.skip_whitespace();

                // Parse proposition (which may contain quantifiers)
                let proposition = self.parse_proposition()?;

                members.push(StructureMember::Axiom {
                    name: axiom_name,
                    proposition,
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
            extends_clause,
            over_clause,
        })
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

        // Parse operation name (identifier or operator symbol in parens)
        let name = self.parse_operation_name()?;

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

    /// Parse function definition (top-level)
    /// Examples:
    ///   define pi = 3.14159
    ///   define double(x) = x + x
    ///   define abs(x: ℝ) : ℝ = if x < 0 then minus(x) else x
    pub fn parse_function_def(&mut self) -> Result<FunctionDef, KleisParseError> {
        self.skip_whitespace();

        // Expect 'define' keyword
        let keyword = self.parse_identifier()?;
        if keyword != "define" {
            return Err(KleisParseError {
                message: format!("Expected 'define', got '{}'", keyword),
                position: self.pos,
            });
        }

        self.skip_whitespace();

        // Parse function name (can be identifier or operator symbol)
        // Examples: define add(x, y) = ... OR define (-)(x, y) = ...
        let name = self.parse_operation_name()?;
        self.skip_whitespace();

        // Check if this is a function with parameters or a simple definition
        let (params, type_annotation) = if self.peek() == Some('(') {
            // Function form: define f(x, y) = expr
            self.advance(); // consume '('
            let params = self.parse_params()?;
            self.skip_whitespace();

            if self.advance() != Some(')') {
                return Err(KleisParseError {
                    message: "Expected ')' after parameters".to_string(),
                    position: self.pos,
                });
            }

            self.skip_whitespace();

            // Optional return type annotation: : Type
            let type_annotation = if self.peek() == Some(':') {
                self.advance(); // consume ':'
                self.skip_whitespace();
                Some(self.parse_type()?)
            } else {
                None
            };

            (params, type_annotation)
        } else {
            // Simple form: define x = expr
            // Optional type annotation: : Type
            let type_annotation = if self.peek() == Some(':') {
                self.advance(); // consume ':'
                self.skip_whitespace();
                Some(self.parse_type()?)
            } else {
                None
            };

            (Vec::new(), type_annotation)
        };

        self.skip_whitespace();

        // Expect '='
        if self.advance() != Some('=') {
            return Err(KleisParseError {
                message: "Expected '=' after function signature".to_string(),
                position: self.pos,
            });
        }

        self.skip_whitespace();

        // Parse function body
        let body = self.parse_expression()?;

        Ok(FunctionDef {
            name,
            params,
            type_annotation,
            body,
        })
    }

    /// Parse function parameters
    /// Examples:
    ///   x, y, z
    ///   x: ℝ, y: ℝ
    ///   (x y : ℝ)  -- multiple params with same type (future enhancement)
    fn parse_params(&mut self) -> Result<Vec<String>, KleisParseError> {
        let mut params = Vec::new();

        // Empty parameter list
        self.skip_whitespace();
        if self.peek() == Some(')') {
            return Ok(params);
        }

        // Parse comma-separated parameters
        loop {
            self.skip_whitespace();

            // Parse parameter name
            let param_name = self.parse_identifier()?;

            self.skip_whitespace();

            // Optional type annotation (we parse but don't store it in the simple Vec<String> for now)
            if self.peek() == Some(':') {
                self.advance(); // consume ':'
                self.skip_whitespace();
                // Parse and discard type for now (stored in type_annotation on FunctionDef)
                // TODO: Store parameter types individually when we extend FunctionDef
                self.parse_type()?;
                self.skip_whitespace();
            }

            params.push(param_name);

            // Check for comma or end of parameter list
            if self.peek() == Some(',') {
                self.advance();
                continue;
            } else if self.peek() == Some(')') {
                break;
            } else {
                return Err(KleisParseError {
                    message: "Expected ',' or ')' in parameter list".to_string(),
                    position: self.pos,
                });
            }
        }

        Ok(params)
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

        // Parse optional over clause: over Field(F)
        let over_clause = if self.peek_word("over") {
            // Skip "over"
            for _ in 0..4 {
                self.advance();
            }
            self.skip_whitespace();

            // Parse field type (e.g., Field(ℝ))
            Some(self.parse_type()?)
        } else {
            None
        };

        self.skip_whitespace();

        // Parse optional where clause
        let where_clause = self.parse_where_clause()?;

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
            over_clause,
            where_clause,
        })
    }

    /// Parse optional where clause: where Constraint1, Constraint2, ...
    ///
    /// Example: where Semiring(T), Ord(T)
    fn parse_where_clause(
        &mut self,
    ) -> Result<Option<Vec<crate::kleis_ast::WhereConstraint>>, KleisParseError> {
        self.skip_whitespace();

        // Check if there's a 'where' keyword
        if !self.peek_word("where") {
            return Ok(None);
        }

        // Consume 'where' keyword
        let keyword = self.parse_identifier()?;
        if keyword != "where" {
            return Err(KleisParseError {
                message: format!("Expected 'where', got '{}'", keyword),
                position: self.pos,
            });
        }
        self.skip_whitespace();

        let mut constraints = Vec::new();

        loop {
            // Parse structure name
            let structure_name = self.parse_identifier()?;
            self.skip_whitespace();

            // Parse type arguments
            if self.advance() != Some('(') {
                return Err(KleisParseError {
                    message: format!(
                        "Expected '(' after structure name in where clause: {}",
                        structure_name
                    ),
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
                    message: "Expected ')' after type arguments in where clause".to_string(),
                    position: self.pos,
                });
            }

            constraints.push(crate::kleis_ast::WhereConstraint {
                structure_name,
                type_args,
            });

            self.skip_whitespace();

            // Check if there's another constraint (comma-separated)
            if self.peek() == Some(',') {
                self.advance();
                self.skip_whitespace();
            } else {
                // No more constraints
                break;
            }
        }

        Ok(Some(constraints))
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
                // or operation negate(x) = -x
                // or operation (+) = builtin_add
                let name = self.parse_operation_name()?;
                self.skip_whitespace();

                // Check for parameters: operation name(params) = expr
                let params = if self.peek() == Some('(') {
                    self.advance(); // consume '('
                    let params = self.parse_params()?;
                    self.skip_whitespace();

                    if self.advance() != Some(')') {
                        return Err(KleisParseError {
                            message: "Expected ')' after parameters".to_string(),
                            position: self.pos,
                        });
                    }
                    self.skip_whitespace();
                    params
                } else {
                    Vec::new()
                };

                if self.advance() != Some('=') {
                    return Err(KleisParseError {
                        message: "Expected '=' after operation name".to_string(),
                        position: self.pos,
                    });
                }

                self.skip_whitespace();

                // Parse implementation (could be builtin or inline expression)
                // Try to parse as identifier first (for builtin_xxx)
                let start_pos = self.pos;
                if let Ok(builtin_name) = self.parse_identifier() {
                    // Check if this looks like a builtin (starts with builtin_ or simple name without params)
                    if builtin_name.starts_with("builtin_")
                        || (params.is_empty() && !builtin_name.is_empty())
                    {
                        return Ok(ImplMember::Operation {
                            name,
                            implementation: Implementation::Builtin(builtin_name),
                        });
                    }
                }

                // Otherwise, parse as inline expression
                self.pos = start_pos;
                let expr = self.parse_expression()?;

                Ok(ImplMember::Operation {
                    name,
                    implementation: Implementation::Inline { params, body: expr },
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
            } else if self.peek_word("define") {
                let function_def = self.parse_function_def()?;
                program.add_item(TopLevel::FunctionDef(function_def));
            } else {
                return Err(KleisParseError {
                    message: "Expected 'structure', 'implements', 'data', 'operation', or 'define'"
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
        assert!(result
            .unwrap_err()
            .message
            .contains("must have at least one case"));
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

    // ===== Function Definition Parser Tests (define) =====

    #[test]
    fn test_parse_define_simple_constant() {
        let code = "define pi = 3.14159";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def().unwrap();

        assert_eq!(result.name, "pi");
        assert!(result.params.is_empty());
        assert!(result.type_annotation.is_none());
        assert!(matches!(result.body, Expression::Const(ref s) if s == "3.14159"));
    }

    #[test]
    fn test_parse_define_simple_with_type() {
        let code = "define pi : ℝ = 3.14159";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def().unwrap();

        assert_eq!(result.name, "pi");
        assert!(result.params.is_empty());
        assert_eq!(
            result.type_annotation,
            Some(TypeExpr::Named("ℝ".to_string()))
        );
        assert!(matches!(result.body, Expression::Const(ref s) if s == "3.14159"));
    }

    #[test]
    fn test_parse_define_function_one_param() {
        let code = "define double(x) = x + x";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def().unwrap();

        assert_eq!(result.name, "double");
        assert_eq!(result.params.len(), 1);
        assert_eq!(result.params[0], "x");
        assert!(result.type_annotation.is_none());
        assert!(matches!(result.body, Expression::Operation { .. }));
    }

    #[test]
    fn test_parse_define_function_two_params() {
        let code = "define add(x, y) = x + y";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def().unwrap();

        assert_eq!(result.name, "add");
        assert_eq!(result.params.len(), 2);
        assert_eq!(result.params[0], "x");
        assert_eq!(result.params[1], "y");
        assert!(result.type_annotation.is_none());
    }

    #[test]
    fn test_parse_define_function_with_param_types() {
        let code = "define add(x: ℝ, y: ℝ) = x + y";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def().unwrap();

        assert_eq!(result.name, "add");
        assert_eq!(result.params.len(), 2);
        assert_eq!(result.params[0], "x");
        assert_eq!(result.params[1], "y");
    }

    #[test]
    fn test_parse_define_function_with_return_type() {
        let code = "define square(x) : ℝ = x * x";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def().unwrap();

        assert_eq!(result.name, "square");
        assert_eq!(result.params.len(), 1);
        assert_eq!(result.params[0], "x");
        assert_eq!(
            result.type_annotation,
            Some(TypeExpr::Named("ℝ".to_string()))
        );
    }

    #[test]
    fn test_parse_define_function_with_all_types() {
        let code = "define abs(x: ℝ) : ℝ = x * x";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def().unwrap();

        assert_eq!(result.name, "abs");
        assert_eq!(result.params.len(), 1);
        assert_eq!(result.params[0], "x");
        assert_eq!(
            result.type_annotation,
            Some(TypeExpr::Named("ℝ".to_string()))
        );
    }

    #[test]
    fn test_parse_define_with_expression_body() {
        let code = "define factorial(n) = match n { 0 => 1 | _ => n }";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def().unwrap();

        assert_eq!(result.name, "factorial");
        assert_eq!(result.params.len(), 1);
        assert!(matches!(result.body, Expression::Match { .. }));
    }

    #[test]
    fn test_parse_define_with_function_call_body() {
        let code = "define not(b) = match b { True => False | False => True }";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def().unwrap();

        assert_eq!(result.name, "not");
        assert_eq!(result.params.len(), 1);
        assert_eq!(result.params[0], "b");
    }

    #[test]
    fn test_parse_program_with_define() {
        let code = r#"
            define pi = 3.14159
            define double(x) = x + x
        "#;

        let result = parse_kleis_program(code).unwrap();
        let functions = result.functions();

        assert_eq!(functions.len(), 2);
        assert_eq!(functions[0].name, "pi");
        assert_eq!(functions[1].name, "double");
    }

    #[test]
    fn test_parse_program_mixed_declarations() {
        let code = r#"
            data Bool = True | False
            
            define not(b) = match b { True => False | False => True }
            
            structure Numeric(N) {
                operation abs : N → N
            }
            
            define double(x) = x + x
        "#;

        let result = parse_kleis_program(code).unwrap();

        assert_eq!(result.data_types().len(), 1);
        assert_eq!(result.functions().len(), 2);
        assert_eq!(result.structures().len(), 1);
    }

    #[test]
    fn test_parse_define_error_missing_equals() {
        let code = "define x 5";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def();

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Expected '='"));
    }

    #[test]
    fn test_parse_define_error_missing_body() {
        let code = "define x =";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_define_error_unclosed_params() {
        let code = "define f(x, y = x + y";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def();

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_define_empty_params() {
        let code = "define f() = 42";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def().unwrap();

        assert_eq!(result.name, "f");
        assert!(result.params.is_empty());
    }

    #[test]
    fn test_parse_define_complex_body() {
        let code = "define compute(a, b, c) = a + b * c";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def().unwrap();

        assert_eq!(result.name, "compute");
        assert_eq!(result.params.len(), 3);
        assert!(matches!(result.body, Expression::Operation { .. }));
    }

    #[test]
    fn test_parse_define_with_function_type() {
        // Simple function type annotation
        let code = "define identity : ℝ → ℝ = x";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def().unwrap();

        assert_eq!(result.name, "identity");
        assert!(result.type_annotation.is_some());
        match result.type_annotation.unwrap() {
            TypeExpr::Function(_, _) => {} // Success
            _ => panic!("Expected function type"),
        }
    }

    // ===== Conditional (if-then-else) Parser Tests =====

    #[test]
    fn test_parse_conditional_simple() {
        let code = "if x then 1 else 0";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                assert_eq!(*condition, Expression::Object("x".to_string()));
                assert_eq!(*then_branch, Expression::Const("1".to_string()));
                assert_eq!(*else_branch, Expression::Const("0".to_string()));
            }
            _ => panic!("Expected Conditional expression, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_conditional_with_comparison() {
        let code = "if x > 0 then x else y";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                // Condition should be a comparison operation
                match condition.as_ref() {
                    Expression::Operation { name, args } => {
                        assert_eq!(name, "greater_than");
                        assert_eq!(args.len(), 2);
                    }
                    _ => panic!("Expected comparison operation in condition"),
                }
                assert_eq!(*then_branch, Expression::Object("x".to_string()));
                assert_eq!(*else_branch, Expression::Object("y".to_string()));
            }
            _ => panic!("Expected Conditional expression"),
        }
    }

    #[test]
    fn test_parse_conditional_with_arithmetic() {
        let code = "if a > b then a + 1 else b + 1";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                // Condition
                match condition.as_ref() {
                    Expression::Operation { name, .. } => {
                        assert_eq!(name, "greater_than");
                    }
                    _ => panic!("Expected comparison"),
                }
                // Then branch should be a plus operation
                match then_branch.as_ref() {
                    Expression::Operation { name, .. } => {
                        assert_eq!(name, "plus");
                    }
                    _ => panic!("Expected plus in then branch"),
                }
                // Else branch should be a plus operation
                match else_branch.as_ref() {
                    Expression::Operation { name, .. } => {
                        assert_eq!(name, "plus");
                    }
                    _ => panic!("Expected plus in else branch"),
                }
            }
            _ => panic!("Expected Conditional expression"),
        }
    }

    #[test]
    fn test_parse_conditional_nested_else() {
        // Nested conditional in else branch
        let code = "if a then 1 else if b then 2 else 3";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                assert_eq!(*condition, Expression::Object("a".to_string()));
                assert_eq!(*then_branch, Expression::Const("1".to_string()));

                // Else branch should be another conditional
                match else_branch.as_ref() {
                    Expression::Conditional {
                        condition: inner_cond,
                        then_branch: inner_then,
                        else_branch: inner_else,
                    } => {
                        assert_eq!(**inner_cond, Expression::Object("b".to_string()));
                        assert_eq!(**inner_then, Expression::Const("2".to_string()));
                        assert_eq!(**inner_else, Expression::Const("3".to_string()));
                    }
                    _ => panic!("Expected nested conditional in else branch"),
                }
            }
            _ => panic!("Expected Conditional expression"),
        }
    }

    #[test]
    fn test_parse_conditional_with_function_call() {
        let code = "if valid(x) then process(x) else default()";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => {
                match condition.as_ref() {
                    Expression::Operation { name, args } => {
                        assert_eq!(name, "valid");
                        assert_eq!(args.len(), 1);
                    }
                    _ => panic!("Expected function call in condition"),
                }
                match then_branch.as_ref() {
                    Expression::Operation { name, .. } => {
                        assert_eq!(name, "process");
                    }
                    _ => panic!("Expected function call in then branch"),
                }
                match else_branch.as_ref() {
                    Expression::Operation { name, args } => {
                        assert_eq!(name, "default");
                        assert!(args.is_empty());
                    }
                    _ => panic!("Expected function call in else branch"),
                }
            }
            _ => panic!("Expected Conditional expression"),
        }
    }

    #[test]
    fn test_parse_conditional_with_equality() {
        let code = "if x == 0 then zero else nonzero";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Conditional { condition, .. } => match condition.as_ref() {
                Expression::Operation { name, .. } => {
                    assert_eq!(name, "equals");
                }
                _ => panic!("Expected equals operation"),
            },
            _ => panic!("Expected Conditional expression"),
        }
    }

    #[test]
    fn test_parse_conditional_with_logical_and() {
        let code = "if a && b then yes else no";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Conditional { condition, .. } => match condition.as_ref() {
                Expression::Operation { name, args } => {
                    assert_eq!(name, "logical_and");
                    assert_eq!(args.len(), 2);
                }
                _ => panic!("Expected logical_and operation"),
            },
            _ => panic!("Expected Conditional expression"),
        }
    }

    #[test]
    fn test_parse_conditional_error_missing_then() {
        let code = "if x 1 else 0";
        let mut parser = KleisParser::new(code);
        let result = parser.parse();

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("then"));
    }

    #[test]
    fn test_parse_conditional_error_missing_else() {
        let code = "if x then 1";
        let mut parser = KleisParser::new(code);
        let result = parser.parse();

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("else"));
    }

    #[test]
    fn test_parse_define_with_conditional() {
        let code = "define abs(x) = if x > 0 then x else negate(x)";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def().unwrap();

        assert_eq!(result.name, "abs");
        assert_eq!(result.params.len(), 1);
        assert!(matches!(result.body, Expression::Conditional { .. }));
    }

    // ===== Let Binding Parser Tests =====

    #[test]
    fn test_parse_let_simple() {
        let code = "let x = 5 in x";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Let { name, value, body } => {
                assert_eq!(name, "x");
                assert_eq!(*value, Expression::Const("5".to_string()));
                assert_eq!(*body, Expression::Object("x".to_string()));
            }
            _ => panic!("Expected Let expression, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_let_with_arithmetic() {
        let code = "let x = 5 in x + x";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Let { name, value, body } => {
                assert_eq!(name, "x");
                assert_eq!(*value, Expression::Const("5".to_string()));
                match body.as_ref() {
                    Expression::Operation {
                        name: op_name,
                        args,
                    } => {
                        assert_eq!(op_name, "plus");
                        assert_eq!(args.len(), 2);
                    }
                    _ => panic!("Expected plus operation in body"),
                }
            }
            _ => panic!("Expected Let expression"),
        }
    }

    #[test]
    fn test_parse_let_with_expression_value() {
        let code = "let squared = x * x in squared + 1";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Let { name, value, body } => {
                assert_eq!(name, "squared");
                match value.as_ref() {
                    Expression::Operation { name: op_name, .. } => {
                        assert_eq!(op_name, "times");
                    }
                    _ => panic!("Expected times operation in value"),
                }
                match body.as_ref() {
                    Expression::Operation { name: op_name, .. } => {
                        assert_eq!(op_name, "plus");
                    }
                    _ => panic!("Expected plus operation in body"),
                }
            }
            _ => panic!("Expected Let expression"),
        }
    }

    #[test]
    fn test_parse_let_nested() {
        let code = "let a = 1 in let b = 2 in a + b";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Let {
                name: outer_name,
                value: outer_value,
                body: outer_body,
            } => {
                assert_eq!(outer_name, "a");
                assert_eq!(*outer_value, Expression::Const("1".to_string()));

                // Outer body should be another let
                match outer_body.as_ref() {
                    Expression::Let {
                        name: inner_name,
                        value: inner_value,
                        body: inner_body,
                    } => {
                        assert_eq!(inner_name, "b");
                        assert_eq!(**inner_value, Expression::Const("2".to_string()));
                        match inner_body.as_ref() {
                            Expression::Operation { name: op_name, .. } => {
                                assert_eq!(op_name, "plus");
                            }
                            _ => panic!("Expected plus in innermost body"),
                        }
                    }
                    _ => panic!("Expected nested Let in outer body"),
                }
            }
            _ => panic!("Expected Let expression"),
        }
    }

    #[test]
    fn test_parse_let_with_conditional() {
        let code = "let x = 5 in if x > 0 then x else 0";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Let { name, body, .. } => {
                assert_eq!(name, "x");
                assert!(matches!(body.as_ref(), Expression::Conditional { .. }));
            }
            _ => panic!("Expected Let expression"),
        }
    }

    #[test]
    fn test_parse_define_with_let() {
        let code = "define quadratic(a, b, c, x) = let x2 = x * x in a * x2 + b * x + c";
        let mut parser = KleisParser::new(code);
        let result = parser.parse_function_def().unwrap();

        assert_eq!(result.name, "quadratic");
        assert_eq!(result.params.len(), 4);
        assert!(matches!(result.body, Expression::Let { .. }));
    }

    #[test]
    fn test_parse_let_error_missing_equals() {
        let code = "let x 5 in x";
        let mut parser = KleisParser::new(code);
        let result = parser.parse();

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Expected '='"));
    }

    #[test]
    fn test_parse_let_error_missing_in() {
        let code = "let x = 5 x";
        let mut parser = KleisParser::new(code);
        let result = parser.parse();

        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("in"));
    }

    #[test]
    fn test_parse_let_with_function_call() {
        let code = "let result = compute(a, b) in result + 1";
        let mut parser = KleisParser::new(code);
        let result = parser.parse().unwrap();

        match result {
            Expression::Let { name, value, .. } => {
                assert_eq!(name, "result");
                match value.as_ref() {
                    Expression::Operation {
                        name: op_name,
                        args,
                    } => {
                        assert_eq!(op_name, "compute");
                        assert_eq!(args.len(), 2);
                    }
                    _ => panic!("Expected function call in value"),
                }
            }
            _ => panic!("Expected Let expression"),
        }
    }
}
