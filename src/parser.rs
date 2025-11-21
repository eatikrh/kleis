// LaTeX Parser - Converts LaTeX strings to Kleis Expression AST
// This validates that our 56 operations can represent real LaTeX notation

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Const(String),
    Object(String),
    Operation { name: String, args: Vec<Expression> },
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parse error at position {}: {}", self.position, self.message)
    }
}

impl std::error::Error for ParseError {}

// Convenience constructors
fn c(s: impl Into<String>) -> Expression { Expression::Const(s.into()) }
fn o(s: impl Into<String>) -> Expression { Expression::Object(s.into()) }
fn op(name: impl Into<String>, args: Vec<Expression>) -> Expression {
    Expression::Operation { name: name.into(), args }
}

pub struct Parser {
    input: Vec<char>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Parser {
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

    fn parse_command(&mut self) -> Result<String, ParseError> {
        // Parse \commandname or special chars like \{ \}
        if self.advance() != Some('\\') {
            return Err(ParseError {
                message: "Expected backslash".to_string(),
                position: self.pos,
            });
        }

        let start = self.pos;
        
        // Check for special single-character commands
        if let Some(ch) = self.peek() {
            if !ch.is_alphanumeric() && ch != ' ' {
                // Special chars like \{ \} \, \!
                self.advance();
                return Ok(ch.to_string());
            }
        }
        
        // Parse multi-character command
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() {
                self.advance();
            } else {
                break;
            }
        }

        if self.pos == start {
            return Err(ParseError {
                message: "Empty command name".to_string(),
                position: self.pos,
            });
        }

        Ok(self.input[start..self.pos].iter().collect())
    }

    fn parse_group(&mut self) -> Result<Expression, ParseError> {
        // Parse {content}
        if self.advance() != Some('{') {
            return Err(ParseError {
                message: "Expected '{'".to_string(),
                position: self.pos,
            });
        }

        let expr = self.parse_expression_until('}')?;

        if self.advance() != Some('}') {
            return Err(ParseError {
                message: "Expected '}'".to_string(),
                position: self.pos,
            });
        }

        Ok(expr)
    }

    fn parse_group_or_parens(&mut self) -> Result<Expression, ParseError> {
        // Parse {content} or (content) - for functions that accept both
        self.skip_whitespace();
        
        match self.peek() {
            Some('{') => {
                // Use standard group parsing
                self.parse_group()
            }
            Some('(') => {
                // Parse parentheses
                self.advance(); // consume (
                let expr = self.parse_expression_until(')')?;
                if self.advance() != Some(')') {
                    return Err(ParseError {
                        message: "Expected ')'".to_string(),
                        position: self.pos,
                    });
                }
                Ok(expr)
            }
            _ => {
                // Neither braces nor parentheses - try to parse next primary
                self.parse_primary()
            }
        }
    }

    fn parse_text_group(&mut self) -> Result<String, ParseError> {
        // Parse {content} as plain text without expression parsing
        // Used for environment names where we don't want implicit multiplication
        if self.advance() != Some('{') {
            return Err(ParseError {
                message: "Expected '{'".to_string(),
                position: self.pos,
            });
        }

        let mut text = String::new();
        while let Some(ch) = self.peek() {
            if ch == '}' {
                break;
            }
            text.push(ch);
            self.advance();
        }

        if self.advance() != Some('}') {
            return Err(ParseError {
                message: "Expected '}'".to_string(),
                position: self.pos,
            });
        }

        Ok(text)
    }

    fn parse_optional_group(&mut self) -> Result<Option<Expression>, ParseError> {
        // Parse [content]
        self.skip_whitespace();
        if self.peek() == Some('[') {
            self.advance(); // consume '['
            let expr = self.parse_expression_until(']')?;
            if self.advance() != Some(']') {
                return Err(ParseError {
                    message: "Expected ']'".to_string(),
                    position: self.pos,
                });
            }
            Ok(Some(expr))
        } else {
            Ok(None)
        }
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        // Parse primary expressions: numbers, identifiers, commands, groups
        self.skip_whitespace();

        // Addition 2: Handle unary minus
        if self.peek() == Some('-') {
            self.advance();
            let expr = self.parse_primary()?;
            return Ok(op("minus", vec![c("0"), expr])); // -x = 0 - x
        }
        
        // Handle unary plus (just skip it)
        if self.peek() == Some('+') {
            self.advance();
            return self.parse_primary();
        }

        match self.peek() {
            Some('\\') => self.parse_latex_command(),
            Some('{') => self.parse_group(),
            Some('(') => {
                // Addition 4: Check if this is a function call
                self.advance();
                let first_expr = self.parse_additive()?;
                self.skip_whitespace();
                
                // Check for comma-separated arguments (function call)
                if self.peek() == Some(',') {
                    let mut args = vec![first_expr];
                    while self.peek() == Some(',') {
                        self.advance();
                        self.skip_whitespace();
                        args.push(self.parse_additive()?);
                        self.skip_whitespace();
                    }
                    if self.peek() == Some(')') {
                        self.advance();
                        // Return as generic function with multiple args
                        return Ok(op("function_call", args));
                    }
                    // Failed to parse as function, return what we have
                    return Ok(args.into_iter().next().unwrap_or(o("")));
                }
                
                // Single expression in parentheses
                if self.peek() == Some(')') {
                    self.advance();
                }
                Ok(first_expr)
            }
            Some('|') => {
                // Ket vector: |ψ⟩
                self.advance();
                let content = self.parse_primary()?;
                // Look for \rangle or ⟩
                self.skip_whitespace();
                if self.peek() == Some('\\') {
                    let saved = self.pos;
                    if let Ok(cmd) = self.parse_command() {
                        if cmd == "rangle" {
                            return Ok(op("ket", vec![content]));
                        }
                    }
                    self.pos = saved;
                }
                // Not a ket, just return the content
                Ok(content)
            }
            Some('[') => {
                // Commutator: [A, B]
                self.advance();
                let first = self.parse_additive()?;
                self.skip_whitespace();
                if self.peek() == Some(',') {
                    self.advance();
                    self.skip_whitespace();
                    let second = self.parse_additive()?;
                    self.skip_whitespace();
                    if self.peek() == Some(']') {
                        self.advance();
                        return Ok(op("commutator", vec![first, second]));
                    }
                }
                // Not a commutator
                Ok(first)
            }
            Some(ch) if ch.is_numeric() => {
                // Parse number - stop at first non-numeric
                let mut text = String::new();
                while let Some(c) = self.peek() {
                    if c.is_numeric() || c == '.' {
                        text.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                Ok(c(text))
            }
            Some(ch) if ch.is_alphabetic() => {
                // Parse identifier
                let mut text = String::new();
                text.push(ch);
                self.advance();
                
                // For implicit multiplication: single lowercase letters = variables
                // Multi-letter: functions or constants (Sin, cos, Gamma, etc.)
                if ch.is_uppercase() {
                    // Capital letter - consume more for multi-char names
                    while let Some(c) = self.peek() {
                        if c.is_alphabetic() || c.is_numeric() {
                            text.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                // Lowercase: single letter (for implicit mult: ab → a*b)
                
                Ok(o(text))
            }
            _ => Err(ParseError {
                message: format!("Unexpected character: {:?}", self.peek()),
                position: self.pos,
            }),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expression, ParseError> {
        // Handle subscripts and superscripts
        let mut expr = self.parse_primary()?;

        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('_') => {
                    self.advance();
                    let sub = if self.peek() == Some('{') {
                        self.parse_group()?
                    } else {
                        // Single character subscript
                        let ch = self.advance().ok_or(ParseError {
                            message: "Expected subscript".to_string(),
                            position: self.pos,
                        })?;
                        o(ch.to_string())
                    };
                    expr = op("sub", vec![expr, sub]);
                }
                Some('^') => {
                    self.advance();
                    let sup = if self.peek() == Some('{') {
                        self.parse_group()?
                    } else {
                        // Single character superscript
                        let ch = self.advance().ok_or(ParseError {
                            message: "Expected superscript".to_string(),
                            position: self.pos,
                        })?;
                        o(ch.to_string())
                    };
                    expr = op("sup", vec![expr, sup]);
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, ParseError> {
        // Handle * / \cdot \times and implicit multiplication
        let mut left = self.parse_postfix()?;

        loop {
            self.skip_whitespace();
            
            // Check for explicit operators first
            let op_name = if self.peek() == Some('*') {
                self.advance();
                "scalar_multiply"
            } else if self.peek() == Some('/') {
                self.advance();
                "scalar_divide"
            } else if self.peek() == Some('\\') {
                let saved_pos = self.pos;
                if let Ok(cmd) = self.parse_command() {
                    match cmd.as_str() {
                        "cdot" => "dot",
                        "times" => "scalar_multiply",
                        "div" => "scalar_divide",
                        _ => {
                            // Not a multiplicative operator, backtrack
                            self.pos = saved_pos;
                            break;
                        }
                    }
                } else {
                    self.pos = saved_pos;
                    break;
                }
            } else {
                // Addition 3: Check for implicit multiplication
                // If next char could start a term (letter, digit, \, {, (, |, [)
                match self.peek() {
                    Some(ch) if ch.is_alphanumeric() || ch == '\\' || ch == '{' 
                               || ch == '(' || ch == '|' || ch == '[' => {
                        // Implicit multiplication!
                        "scalar_multiply"
                    }
                    _ => break,
                }
            };

            let right = self.parse_postfix()?;
            left = op(op_name, vec![left, right]);
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expression, ParseError> {
        // Handle + -
        let mut left = self.parse_multiplicative()?;

        loop {
            self.skip_whitespace();
            
            let op_name = match self.peek() {
                Some('+') => {
                    self.advance();
                    "plus"
                }
                Some('-') => {
                    self.advance();
                    "minus"
                }
                _ => break,
            };

            let right = self.parse_multiplicative()?;
            left = op(op_name, vec![left, right]);
        }

        Ok(left)
    }

    fn parse_expression_until(&mut self, end: char) -> Result<Expression, ParseError> {
        // Parse full expression with operator precedence
        let mut parts = Vec::new();

        while let Some(ch) = self.peek() {
            if ch == end {
                break;
            }

            // Try to parse an additive expression (highest level)
            match self.parse_additive() {
                Ok(expr) => parts.push(expr),
                Err(_) => {
                    // Skip unknown character and continue
                    if self.peek() == Some(end) {
                        break;
                    }
                    self.advance();
                }
            }

            self.skip_whitespace();
            
            // Check if we hit the end
            if self.peek() == Some(end) {
                break;
            }
        }

        if parts.is_empty() {
            return Ok(o(""));
        }

        if parts.len() == 1 {
            Ok(parts.into_iter().next().unwrap())
        } else {
            // Multiple parts - for now return first (will improve)
            Ok(parts.into_iter().next().unwrap())
        }
    }

    fn parse_matrix_environment(&mut self, env_name: &str) -> Result<Expression, ParseError> {
        // Parse matrix content between \begin{} and \end{}
        let mut rows: Vec<Vec<Expression>> = vec![vec![]];
        let mut current_row = 0;

        loop {
            self.skip_whitespace();
            
            // Check for end of input
            if self.peek().is_none() {
                return Err(ParseError {
                    message: format!("Unexpected end of input while parsing {} environment", env_name),
                    position: self.pos,
                });
            }
            
            // Check for \end or \\ (new row)
            if self.peek() == Some('\\') {
                self.advance(); // consume first \
                
                // Check if it's \\ (double backslash = new row)
                if self.peek() == Some('\\') {
                    self.advance(); // consume second \
                    current_row += 1;
                    rows.push(vec![]);
                    continue;
                }
                
                // Otherwise it's a command like \end
                let mut cmd_name = String::new();
                while let Some(ch) = self.peek() {
                    if ch.is_alphanumeric() {
                        cmd_name.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
                
                if cmd_name == "end" {
                    // Parse {bmatrix} or whatever
                    self.parse_text_group()?;
                    break;
                } else {
                    // Unknown command in matrix - just skip
                    continue;
                }
            }

            // Check for column separator
            if self.peek() == Some('&') {
                self.advance();
                // Don't push placeholder - just move to next cell
                continue;
            }

            // Parse cell expression with brace depth tracking
            let mut cell_content = String::new();
            let start_pos = self.pos;
            let mut brace_depth = 0;
            
            while let Some(ch) = self.peek() {
                // Track brace depth
                if ch == '{' {
                    brace_depth += 1;
                } else if ch == '}' {
                    brace_depth -= 1;
                }
                
                // Only break on & or \\ when NOT inside braces
                if brace_depth == 0 {
                    if ch == '&' {
                        break;
                    }
                    if ch == '\\' {
                        // Check if it's \\ (row sep) or \command
                        let next_pos = self.pos + 1;
                        if next_pos < self.input.len() && self.input[next_pos] == '\\' {
                            // It's \\, end cell
                            break;
                        }
                        // Check for \end command
                        let mut is_end = true;
                        for (i, c) in "end".chars().enumerate() {
                            if next_pos + i >= self.input.len() || self.input[next_pos + i] != c {
                                is_end = false;
                                break;
                            }
                        }
                        if is_end {
                            break;
                        }
                    }
                }
                
                cell_content.push(ch);
                self.advance();
            }

            // Always push something for the cell (even if empty)
            if !cell_content.trim().is_empty() {
                rows[current_row].push(o(cell_content.trim()));
            } else if self.pos > start_pos {
                // We collected nothing but moved - empty cell
                rows[current_row].push(o(""));
            }
        }

        // Convert rows to matrix operation
        // For 2x2 matrix
        if rows.len() == 2 && rows[0].len() == 2 && rows[1].len() == 2 {
            let op_name = match env_name {
                "pmatrix" => "pmatrix2x2",
                "vmatrix" => "vmatrix2x2",
                _ => "matrix2x2",
            };
            Ok(op(op_name, vec![
                rows[0][0].clone(), rows[0][1].clone(),
                rows[1][0].clone(), rows[1][1].clone(),
            ]))
        } else if rows.len() == 3 && rows[0].len() == 3 {
            let op_name = match env_name {
                "pmatrix" => "pmatrix3x3",
                "vmatrix" => "vmatrix3x3",
                _ => "matrix3x3",
            };
            Ok(op(op_name, vec![
                rows[0][0].clone(), rows[0][1].clone(), rows[0][2].clone(),
                rows[1][0].clone(), rows[1][1].clone(), rows[1][2].clone(),
                rows[2][0].clone(), rows[2][1].clone(), rows[2][2].clone(),
            ]))
        } else {
            // Generic matrix - store as operation with all elements
            let all_elements: Vec<Expression> = rows.into_iter()
                .flat_map(|row| row.into_iter())
                .collect();
            Ok(op("matrix", all_elements))
        }
    }

    fn parse_latex_command(&mut self) -> Result<Expression, ParseError> {
        let cmd = self.parse_command()?;

        self.skip_whitespace();

        match cmd.as_str() {
            // Environments
            "begin" => {
                let env_name = self.parse_text_group()?;
                
                match env_name.as_str() {
                    "bmatrix" | "pmatrix" | "vmatrix" | "matrix" => {
                        self.parse_matrix_environment(&env_name)
                    }
                    "cases" => {
                        // TODO: Parse cases environment
                        Err(ParseError {
                            message: "Cases environment not yet supported".to_string(),
                            position: self.pos,
                        })
                    }
                    _ => Err(ParseError {
                        message: format!("Unknown environment: {}", env_name),
                        position: self.pos,
                    }),
                }
            }

            // Delimiters (skip them, just parse content)
            "left" | "right" => {
                // Skip the delimiter character
                self.advance();
                Ok(o(""))
            }
            
            // Box operator (d'Alembertian) - Addition 5
            "Box" | "square" => {
                Ok(o("\\Box"))
            }
            
            // Escaped braces for anticommutator - Addition 1
            "{" => {
                // \{ - parse as anticommutator start
                // Look for pattern: \{A, B\}
                let first = self.parse_additive()?;
                self.skip_whitespace();
                if self.peek() == Some(',') {
                    self.advance();
                    self.skip_whitespace();
                    let second = self.parse_additive()?;
                    self.skip_whitespace();
                    // Look for \}
                    if self.peek() == Some('\\') {
                        let saved = self.pos;
                        if let Ok(cmd) = self.parse_command() {
                            if cmd == "}" {
                                return Ok(op("anticommutator", vec![first, second]));
                            }
                        }
                        self.pos = saved;
                    }
                }
                Ok(first)
            }
            "}" => {
                // Closing brace - shouldn't appear alone
                Ok(o(""))
            }
            
            // Arrow symbols
            "to" | "rightarrow" => Ok(o("\\to")),
            "mapsto" => Ok(o("\\mapsto")),

            // Fractions
            "frac" => {
                let num = self.parse_group()?;
                let den = self.parse_group()?;
                Ok(op("scalar_divide", vec![num, den]))
            }

            // Square roots
            "sqrt" => {
                let n = self.parse_optional_group()?;
                let arg = self.parse_group()?;
                if let Some(n_expr) = n {
                    Ok(op("nth_root", vec![arg, n_expr]))
                } else {
                    Ok(op("sqrt", vec![arg]))
                }
            }

            // Greek letters (lowercase)
            "alpha" => Ok(o("\\alpha")),
            "beta" => Ok(o("\\beta")),
            "gamma" => Ok(o("\\gamma")),
            "delta" => Ok(o("\\delta")),
            "epsilon" => Ok(o("\\epsilon")),
            "theta" => Ok(o("\\theta")),
            "lambda" => Ok(o("\\lambda")),
            "mu" => Ok(o("\\mu")),
            "nu" => Ok(o("\\nu")),
            "pi" => Ok(o("\\pi")),
            "rho" => Ok(o("\\rho")),
            "sigma" => Ok(o("\\sigma")),
            "tau" => Ok(o("\\tau")),
            "phi" => Ok(o("\\phi")),
            "psi" => Ok(o("\\psi")),
            "omega" => Ok(o("\\omega")),

            // Greek letters (uppercase)
            "Gamma" => Ok(o("\\Gamma")),
            "Delta" => Ok(o("\\Delta")),
            "Theta" => Ok(o("\\Theta")),
            "Lambda" => Ok(o("\\Lambda")),
            "Sigma" => Ok(o("\\Sigma")),
            "Phi" => Ok(o("\\Phi")),
            "Psi" => Ok(o("\\Psi")),
            "Omega" => Ok(o("\\Omega")),

            // Operators
            "cdot" => Ok(o("\\cdot")),
            "times" => Ok(o("\\times")),
            "div" => Ok(o("\\div")),
            "pm" => Ok(o("\\pm")),
            "nabla" => Ok(o("\\nabla")),
            "partial" => Ok(o("\\partial")),

            // Relations
            "neq" => Ok(o("\\neq")),
            "leq" => Ok(o("\\leq")),
            "geq" => Ok(o("\\geq")),
            "approx" => Ok(o("\\approx")),
            "equiv" => Ok(o("\\equiv")),
            "propto" => Ok(o("\\propto")),
            "in" => Ok(o("\\in")),

            // Set operations
            "cup" => Ok(o("\\cup")),
            "cap" => Ok(o("\\cap")),
            "subset" => Ok(o("\\subset")),
            "subseteq" => Ok(o("\\subseteq")),

            // Logic
            "forall" => Ok(o("\\forall")),
            "exists" => Ok(o("\\exists")),
            "Rightarrow" => Ok(o("\\Rightarrow")),
            "Leftrightarrow" => Ok(o("\\Leftrightarrow")),

            // Trig functions (accept both braces and parentheses)
            "sin" => {
                let arg = self.parse_group_or_parens()?;
                Ok(op("sin", vec![arg]))
            }
            "cos" => {
                let arg = self.parse_group_or_parens()?;
                Ok(op("cos", vec![arg]))
            }
            "tan" => {
                let arg = self.parse_group_or_parens()?;
                Ok(op("tan", vec![arg]))
            }

            // More functions (accept both braces and parentheses)
            "ln" => {
                let arg = self.parse_group_or_parens()?;
                Ok(op("ln", vec![arg]))
            }
            "log" => {
                let arg = self.parse_group_or_parens()?;
                Ok(op("log", vec![arg]))
            }
            "exp" => {
                let arg = self.parse_group_or_parens()?;
                Ok(op("exp", vec![arg]))
            }

            // Bra-ket
            "langle" => {
                // Check if this is a bra vector: \langle φ |
                self.skip_whitespace();
                let saved_pos = self.pos;
                
                // Try to parse content until |
                let mut depth = 0;
                let mut found_pipe = false;
                while let Some(ch) = self.peek() {
                    if ch == '|' && depth == 0 {
                        found_pipe = true;
                        break;
                    }
                    if ch == '{' {
                        depth += 1;
                    } else if ch == '}' {
                        depth -= 1;
                    }
                    self.advance();
                }
                
                if found_pipe {
                    // It's a bra! Parse the content
                    let content_str: String = self.input[saved_pos..self.pos].iter().collect();
                    self.advance(); // consume |
                    
                    // Parse the content as an expression
                    let mut content_parser = Parser::new(&content_str);
                    match content_parser.parse() {
                        Ok(content) => return Ok(op("bra", vec![content])),
                        Err(_) => {
                            // If content parsing fails, treat as object
                            return Ok(op("bra", vec![o(content_str.trim())]));
                        }
                    }
                }
                
                // Not a bra, just return langle
                self.pos = saved_pos;
                Ok(o("\\langle"))
            }
            "rangle" => Ok(o("\\rangle")),

            // Hat
            "hat" => {
                let arg = self.parse_group()?;
                Ok(op("hat", vec![arg]))
            }

            // Number sets
            "mathbb" => {
                let arg = self.parse_group()?;
                Ok(o(format!("\\mathbb{{{}}}", arg.as_string())))
            }

            // Math functions
            "min" | "max" => {
                // Could have subscript already parsed
                Ok(o(format!("\\{}", cmd)))
            }

            // Spacing commands (ignore)
            "," | ";" | "!" | "quad" | "qquad" => {
                Ok(o(" "))
            }

            // Text formatting (pass through)
            "mathbf" | "boldsymbol" | "mathrm" => {
                let arg = self.parse_group()?;
                Ok(o(format!("\\{}{{{}}}",cmd, arg.as_string())))
            }

            // Integrals
            "int" => {
                // Could have limits with _ and ^
                Ok(o("\\int"))
            }

            // Sum/Product
            "sum" | "prod" => {
                Ok(o(format!("\\{}", cmd)))
            }

            _ => {
                // Unknown command - return as object for now
                Ok(o(format!("\\{}", cmd)))
            }
        }
    }

    pub fn parse(&mut self) -> Result<Expression, ParseError> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return Err(ParseError {
                message: "Empty input".to_string(),
                position: 0,
            });
        }
        self.parse_relational()
    }

    fn parse_relational(&mut self) -> Result<Expression, ParseError> {
        // Parse relations: =, ≠, <, >, ≤, ≥, ≈, ≡, ∈, ⊂, ⊆, ∪, ∩
        // These have lowest precedence
        let mut left = self.parse_additive()?;

        loop {
            self.skip_whitespace();

            let op_name = if self.peek() == Some('=') {
                self.advance();
                "equals"
            } else if self.peek() == Some('<') {
                self.advance();
                "less_than"
            } else if self.peek() == Some('>') {
                self.advance();
                "greater_than"
            } else if self.peek() == Some('\\') {
                let saved_pos = self.pos;
                if let Ok(cmd) = self.parse_command() {
                    match cmd.as_str() {
                        "neq" => "not_equal",
                        "leq" => "leq",
                        "geq" => "geq",
                        "approx" => "approx",
                        "equiv" => "equiv",
                        "propto" => "proportional",
                        "in" => "in_set",
                        "subset" => "subset",
                        "subseteq" => "subseteq",
                        "cup" => "union",
                        "cap" => "intersection",
                        _ => {
                            // Not a relation operator, backtrack
                            self.pos = saved_pos;
                            break;
                        }
                    }
                } else {
                    self.pos = saved_pos;
                    break;
                }
            } else {
                break;
            };

            self.skip_whitespace();
            let right = self.parse_additive()?;
            left = op(op_name, vec![left, right]);
        }

        Ok(left)
    }
}

impl Expression {
    fn as_string(&self) -> String {
        match self {
            Expression::Const(s) => s.clone(),
            Expression::Object(s) => s.clone(),
            Expression::Operation { .. } => "".to_string(),
        }
    }
}

// Public API
pub fn parse_latex(input: &str) -> Result<Expression, ParseError> {
    let mut parser = Parser::new(input);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_fraction() {
        let result = parse_latex("\\frac{1}{2}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "scalar_divide");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected operation"),
        }
    }

    #[test]
    fn parses_square_root() {
        let result = parse_latex("\\sqrt{x}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "sqrt");
            }
            _ => panic!("Expected operation"),
        }
    }

    #[test]
    fn parses_greek_letters() {
        let result = parse_latex("\\alpha");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_trig_function() {
        let result = parse_latex("\\sin{x}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "sin");
            }
            _ => panic!("Expected operation"),
        }
    }

    #[test]
    fn parses_subscript() {
        let result = parse_latex("x_{0}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "sub");
            }
            _ => panic!("Expected subscript operation"),
        }
    }

    #[test]
    fn parses_superscript() {
        let result = parse_latex("x^{2}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "sup");
            }
            _ => panic!("Expected superscript operation"),
        }
    }

    #[test]
    fn parses_addition() {
        let result = parse_latex("a + b");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "plus");
            }
            _ => panic!("Expected plus operation"),
        }
    }

    #[test]
    fn parses_multiplication() {
        let result = parse_latex("a * b");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "scalar_multiply");
            }
            _ => panic!("Expected multiply operation"),
        }
    }

    #[test]
    fn parses_complex_expression() {
        // a + b * c should parse as a + (b * c) due to precedence
        let result = parse_latex("a + b * c");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_simple_matrix() {
        let result = parse_latex("\\begin{bmatrix}a&b\\\\c&d\\end{bmatrix}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "matrix2x2");
            }
            _ => panic!("Expected matrix operation"),
        }
    }

    // === Final Parser Push: Top 5 Additions ===

    // Addition 1: Anticommutator
    #[test]
    fn parses_anticommutator_escaped_braces() {
        let result = parse_latex("\\{A, B\\}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "anticommutator");
            }
            _ => panic!("Expected anticommutator"),
        }
    }

    // Addition 2: Unary minus
    #[test]
    fn parses_unary_minus() {
        let result = parse_latex("-x");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "minus");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected minus operation"),
        }
    }

    #[test]
    fn parses_negative_fraction() {
        let result = parse_latex("-\\frac{1}{2}");
        assert!(result.is_ok());
    }

    // Addition 3: Implicit multiplication
    #[test]
    fn parses_implicit_multiplication_number_variable() {
        let result = parse_latex("2x");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "scalar_multiply");
            }
            _ => panic!("Expected multiplication for 2x"),
        }
    }

    #[test]
    fn parses_implicit_multiplication_variables() {
        let result = parse_latex("ab");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "scalar_multiply");
            }
            _ => panic!("Expected multiplication for ab"),
        }
    }

    #[test]
    fn parses_implicit_in_fraction() {
        let result = parse_latex("\\frac{1}{2m}");
        assert!(result.is_ok());
        // Denominator should be 2*m
    }

    // Addition 4: Function calls
    #[test]
    fn parses_function_single_arg() {
        let result = parse_latex("f(x)");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_function_multi_args() {
        let result = parse_latex("F(x, y)");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                // Should detect as function call or implicit multiply with group
                assert!(args.len() >= 2);
            }
            _ => {}
        }
    }

    // Addition 5: Box operator
    #[test]
    fn parses_box_operator() {
        let result = parse_latex("\\Box");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_klein_gordon() {
        let result = parse_latex("\\Box \\phi = 0");
        assert!(result.is_ok());
    }

    // Integration: Complex real-world examples
    #[test]
    fn parses_schrodinger_with_unary() {
        let result = parse_latex("-\\frac{\\hbar^{2}}{2m}");
        assert!(result.is_ok());
    }

    // Relations (NEW)
    #[test]
    fn parses_equation() {
        let result = parse_latex("E = mc^{2}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "equals");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected equals operation"),
        }
    }

    #[test]
    fn parses_inequality() {
        let result = parse_latex("x \\leq y");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "leq");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected leq operation"),
        }
    }

    #[test]
    fn parses_set_membership() {
        let result = parse_latex("x \\in \\mathbb{R}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "in_set");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected in_set operation"),
        }
    }

    #[test]
    fn parses_subset() {
        let result = parse_latex("A \\subseteq B");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "subseteq");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected subseteq operation"),
        }
    }

    #[test]
    fn parses_union() {
        let result = parse_latex("A \\cup B");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "union");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected union operation"),
        }
    }

    #[test]
    fn parses_bra_vector() {
        let result = parse_latex("\\langle\\phi|");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "bra");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected bra operation"),
        }
    }

    #[test]
    fn parses_complex_equation() {
        let result = parse_latex("\\alpha + \\beta = \\gamma");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "equals");
            }
            _ => panic!("Expected equals operation"),
        }
    }
}

