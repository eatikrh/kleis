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
use crate::ast::Expression;
use crate::kleis_ast::{
    ImplMember, Implementation, ImplementsDef, OperationDecl, Program, StructureDef,
    StructureMember, TopLevel, TypeExpr,
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
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
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

        // Optional type parameters: (N) or (M, N)
        if self.peek() == Some('(') {
            self.advance();
            // Skip type parameters for now (TODO: parse and store them)
            let mut depth = 1;
            while depth > 0 {
                match self.advance() {
                    Some('(') => depth += 1,
                    Some(')') => depth -= 1,
                    Some(_) => {}
                    None => {
                        return Err(KleisParseError {
                            message: "Unclosed type parameters".to_string(),
                            position: self.pos,
                        });
                    }
                }
            }
            self.skip_whitespace();
        }

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

        Ok(StructureDef { name, members })
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

        // Parse type argument: (ℝ) or (Vector(n))
        if self.advance() != Some('(') {
            return Err(KleisParseError {
                message: "Expected '(' after structure name".to_string(),
                position: self.pos,
            });
        }

        let type_arg = self.parse_type()?;
        self.skip_whitespace();

        if self.advance() != Some(')') {
            return Err(KleisParseError {
                message: "Expected ')' after type argument".to_string(),
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
            type_arg,
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
            } else if self.peek_word("operation") {
                let operation = self.parse_operation_decl()?;
                program.add_item(TopLevel::OperationDecl(operation));
            } else {
                return Err(KleisParseError {
                    message: "Expected 'structure', 'implements', or 'operation'".to_string(),
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
        assert_eq!(result.type_arg, TypeExpr::Named("ℝ".to_string()));
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
                assert_eq!(impl_def.type_arg, TypeExpr::Named("ℝ".to_string()));
            }
            _ => panic!("Expected ImplementsDef"),
        }

        // Use helper methods
        let structures = result.structures();
        let implements = result.implements();

        assert_eq!(structures.len(), 1);
        assert_eq!(implements.len(), 1);
    }
}
