// LaTeX Parser - Converts LaTeX strings to Kleis Expression AST
// This validates that our 56 operations can represent real LaTeX notation

use crate::ast::Expression;
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Parse error at position {}: {}",
            self.position, self.message
        )
    }
}

impl std::error::Error for ParseError {}

// Convenience constructors
fn c(s: impl Into<String>) -> Expression {
    Expression::Const(s.into())
}
fn o(s: impl Into<String>) -> Expression {
    Expression::Object(s.into())
}
fn op(name: impl Into<String>, args: Vec<Expression>) -> Expression {
    Expression::Operation {
        name: name.into(),
        args,
    }
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

    fn parse_script_group(&mut self) -> Result<Expression, ParseError> {
        // Scripts use standard group parsing but collapse simple literal chains
        let expr = self.parse_group()?;
        Ok(collapse_script_literals(expr))
    }

    fn parse_group_or_parens(&mut self) -> Result<Expression, ParseError> {
        // Parse {content} or (content) - for functions that accept both

        // Skip whitespace and spacing commands
        loop {
            self.skip_whitespace();

            if self.peek() == Some('\\') {
                let saved_pos = self.pos;
                if let Ok(cmd) = self.parse_command() {
                    if matches!(cmd.as_str(), "," | ";" | "!" | "quad" | "qquad" | "colon") {
                        continue;
                    }
                }
                // Not a spacing command, restore position
                self.pos = saved_pos;
            }
            break;
        }

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

                // Check if followed by parentheses (function call)
                self.skip_whitespace();
                if self.peek() == Some('(') {
                    self.advance(); // consume (
                    let mut args = vec![o(text)]; // function name as first arg

                    // Parse comma-separated arguments
                    loop {
                        self.skip_whitespace();
                        if self.peek() == Some(')') {
                            self.advance();
                            break;
                        }

                        args.push(self.parse_additive()?);
                        self.skip_whitespace();

                        if self.peek() == Some(',') {
                            self.advance();
                        } else if self.peek() == Some(')') {
                            self.advance();
                            break;
                        } else {
                            return Err(ParseError {
                                message: "Expected ',' or ')' in function call".to_string(),
                                position: self.pos,
                            });
                        }
                    }

                    Ok(op("function_call", args))
                } else {
                    Ok(o(text))
                }
            }
            _ => Err(ParseError {
                message: format!("Unexpected character: {:?}", self.peek()),
                position: self.pos,
            }),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expression, ParseError> {
        // Handle subscripts, superscripts, and prime notation
        let mut expr = self.parse_primary()?;

        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('_') => {
                    self.advance();
                    let sub = if self.peek() == Some('{') {
                        self.parse_script_group()?
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
                Some('\'') => {
                    // Prime notation (derivative): y', y'', etc.
                    let mut prime_count = 0;
                    while self.peek() == Some('\'') {
                        self.advance();
                        prime_count += 1;
                    }
                    // Represent as superscript with prime symbols
                    let prime_str = "'".repeat(prime_count);
                    expr = op("sup", vec![expr, o(prime_str)]);
                }
                Some('!') => {
                    // Factorial postfix: allow repeated !! notation
                    while self.peek() == Some('!') {
                        self.advance();
                        expr = op("factorial", vec![expr]);
                    }
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
                            // Check if it's a command that starts a new term (for implicit multiplication)
                            // Parse command again to check
                            if let Ok(cmd2) = self.parse_command() {
                                // Check if it's a spacing command - if so, skip it and continue
                                let is_spacing =
                                    matches!(cmd2.as_str(), "," | ";" | "!" | "quad" | "qquad");
                                if is_spacing {
                                    // Skip the spacing command and continue the loop
                                    continue;
                                }

                                // Commands that start new terms (functions, roots, fractions, etc.)
                                let is_term_starter = matches!(
                                    cmd2.as_str(),
                                    "sqrt"
                                        | "frac"
                                        | "sin"
                                        | "cos"
                                        | "tan"
                                        | "sec"
                                        | "csc"
                                        | "cot"
                                        | "arcsin"
                                        | "arccos"
                                        | "arctan"
                                        | "sinh"
                                        | "cosh"
                                        | "tanh"
                                        | "ln"
                                        | "log"
                                        | "exp"
                                        | "int"
                                        | "sum"
                                        | "prod"
                                        | "lim"
                                        | "alpha"
                                        | "beta"
                                        | "gamma"
                                        | "delta"
                                        | "epsilon"
                                        | "zeta"
                                        | "eta"
                                        | "theta"
                                        | "iota"
                                        | "kappa"
                                        | "lambda"
                                        | "mu"
                                        | "nu"
                                        | "xi"
                                        | "pi"
                                        | "rho"
                                        | "sigma"
                                        | "tau"
                                        | "upsilon"
                                        | "phi"
                                        | "chi"
                                        | "psi"
                                        | "omega"
                                        | "Gamma"
                                        | "Delta"
                                        | "Theta"
                                        | "Lambda"
                                        | "Xi"
                                        | "Pi"
                                        | "Sigma"
                                        | "Upsilon"
                                        | "Phi"
                                        | "Psi"
                                        | "Omega"
                                        | "aleph"
                                        | "beth"
                                        | "gimel"
                                        | "daleth"
                                        | "mathbb"
                                        | "boldsymbol"
                                        | "vec"
                                        | "hat"
                                        | "bar"
                                        | "tilde"
                                        | "overline"
                                        | "dot"
                                        | "ddot"
                                        | "partial"
                                        | "nabla"
                                        | "hbar"
                                        | "infty"
                                        | "emptyset"
                                        | "mathrm"
                                        | "Rightarrow"
                                        | "Leftarrow"
                                        | "Leftrightarrow"
                                        | "colon"
                                        | "forall"
                                        | "exists"
                                        | "pmod"
                                        | "begin"
                                        | "left"
                                );
                                self.pos = saved_pos; // Backtrack again
                                if is_term_starter {
                                    "scalar_multiply"
                                } else {
                                    break;
                                }
                            } else {
                                self.pos = saved_pos;
                                break;
                            }
                        }
                    }
                } else {
                    self.pos = saved_pos;
                    break;
                }
            } else {
                // Addition 3: Check for implicit multiplication
                // If next char could start a term (letter, digit, {, (, |, [)
                match self.peek() {
                    Some(ch)
                        if ch.is_alphanumeric()
                            || ch == '{'
                            || ch == '('
                            || ch == '|'
                            || ch == '[' =>
                    {
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

            // Try to parse a relational expression (highest level, includes =, <, >, etc.)
            match self.parse_relational() {
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
                    message: format!(
                        "Unexpected end of input while parsing {} environment",
                        env_name
                    ),
                    position: self.pos,
                });
            }

            // Check for \end or \\ (new row) - but DON'T consume other commands
            if self.peek() == Some('\\') {
                let saved_pos = self.pos;
                self.advance(); // consume first \

                // Check if it's \\ (double backslash = new row)
                if self.peek() == Some('\\') {
                    self.advance(); // consume second \
                    current_row += 1;
                    rows.push(vec![]);
                    continue;
                }

                // Otherwise it's a command - check if it's \end
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
                    // It's some other command (\frac, \sqrt, etc.)
                    // Restore position and let it be part of cell content
                    self.pos = saved_pos;
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

            // Parse the cell content as an expression (same as cases environment)
            if !cell_content.trim().is_empty() {
                // Try to parse the cell content as a proper expression
                match parse_latex(cell_content.trim()) {
                    Ok(expr) => rows[current_row].push(expr),
                    Err(_) => {
                        // Fallback: store as object if parsing fails
                        rows[current_row].push(o(cell_content.trim()))
                    }
                }
            } else if self.pos > start_pos {
                // Empty cell
                rows[current_row].push(o(""));
            }
        }

        // Convert rows to matrix operation with generic Matrix constructor
        // Matrix operations now have format: Matrix(rows, cols, ...elements)
        // Or PMatrix(rows, cols, ...elements) for parenthesis matrices
        // Or VMatrix(rows, cols, ...elements) for determinant bars

        let num_rows = rows.len();
        let num_cols = if !rows.is_empty() { rows[0].len() } else { 0 };

        // Flatten all elements
        let mut args = vec![
            Expression::Const(num_rows.to_string()), // First arg: rows
            Expression::Const(num_cols.to_string()), // Second arg: cols
        ];
        for row in rows {
            args.extend(row);
        }

        // Choose operation name based on environment
        let op_name = match env_name {
            "pmatrix" => "PMatrix",
            "vmatrix" => "VMatrix",
            "bmatrix" | "matrix" => "Matrix", // Both use square brackets
            _ => "Matrix",
        };

        Ok(op(op_name, args))
    }

    fn parse_cases_environment(&mut self) -> Result<Expression, ParseError> {
        // Parse cases environment: \begin{cases} expr1 & cond1 \\ expr2 & cond2 \\ ... \end{cases}
        // Cases always have 2 columns: expression & condition
        let mut rows: Vec<Vec<Expression>> = vec![vec![]];
        let mut current_row = 0;

        loop {
            self.skip_whitespace();

            // Check for end of input
            if self.peek().is_none() {
                return Err(ParseError {
                    message: "Unexpected end of input while parsing cases environment".to_string(),
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
                    // Parse {cases}
                    self.parse_text_group()?;
                    break;
                } else {
                    // Unknown command - just skip
                    continue;
                }
            }

            // Check for column separator
            if self.peek() == Some('&') {
                self.advance();
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

            // Parse the cell content as an expression
            if !cell_content.trim().is_empty() {
                // Try to parse the cell content as a proper expression
                match parse_latex(cell_content.trim()) {
                    Ok(expr) => rows[current_row].push(expr),
                    Err(_) => {
                        // Fallback: store as object if parsing fails
                        rows[current_row].push(o(cell_content.trim()))
                    }
                }
            } else if self.pos > start_pos {
                // Empty cell
                rows[current_row].push(o(""));
            }
        }

        // Filter out empty rows
        let rows: Vec<Vec<Expression>> = rows.into_iter().filter(|row| !row.is_empty()).collect();

        // Validate: cases should have exactly 2 columns per row
        for (i, row) in rows.iter().enumerate() {
            if row.len() != 2 {
                return Err(ParseError {
                    message: format!(
                        "Cases environment row {} has {} columns, expected 2 (expression & condition)",
                        i,
                        row.len()
                    ),
                    position: self.pos,
                });
            }
        }

        // Convert to appropriate cases operation based on number of rows
        match rows.len() {
            2 => {
                // cases2: expr1, cond1, expr2, cond2
                Ok(op(
                    "cases2",
                    vec![
                        rows[0][0].clone(),
                        rows[0][1].clone(),
                        rows[1][0].clone(),
                        rows[1][1].clone(),
                    ],
                ))
            }
            3 => {
                // cases3: expr1, cond1, expr2, cond2, expr3, cond3
                Ok(op(
                    "cases3",
                    vec![
                        rows[0][0].clone(),
                        rows[0][1].clone(),
                        rows[1][0].clone(),
                        rows[1][1].clone(),
                        rows[2][0].clone(),
                        rows[2][1].clone(),
                    ],
                ))
            }
            n if n > 3 => {
                // Generic cases with more rows - flatten all
                let all_elements: Vec<Expression> =
                    rows.into_iter().flat_map(|row| row.into_iter()).collect();
                Ok(op("cases", all_elements))
            }
            _ => Err(ParseError {
                message: "Cases environment must have at least 2 rows".to_string(),
                position: self.pos,
            }),
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
                    "cases" => self.parse_cases_environment(),
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
            "Box" | "square" => Ok(o("\\Box")),

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

            // Binomial coefficient
            "binom" => {
                let n = self.parse_group()?;
                let k = self.parse_group()?;
                Ok(op("binomial", vec![n, k]))
            }

            // Floor function
            "lfloor" => {
                // Parse content until \rfloor
                self.skip_whitespace();
                let saved_pos = self.pos;
                let mut depth = 0;

                while let Some(ch) = self.peek() {
                    if ch == '\\' {
                        let cmd_pos = self.pos;
                        self.advance(); // consume \
                        if let Some(next) = self.peek() {
                            if next.is_alphabetic() {
                                // Parse command name
                                let mut cmd_name = String::new();
                                while let Some(c) = self.peek() {
                                    if c.is_alphabetic() {
                                        cmd_name.push(c);
                                        self.advance();
                                    } else {
                                        break;
                                    }
                                }
                                if cmd_name == "rfloor" && depth == 0 {
                                    let content_str: String =
                                        self.input[saved_pos..cmd_pos].iter().collect();
                                    let mut content_parser = Parser::new(&content_str);
                                    match content_parser.parse() {
                                        Ok(content) => return Ok(op("floor", vec![content])),
                                        Err(_) => {
                                            return Ok(op("floor", vec![o(content_str.trim())]));
                                        }
                                    }
                                }
                            }
                        }
                    } else if ch == '{' {
                        depth += 1;
                        self.advance();
                    } else if ch == '}' {
                        depth -= 1;
                        self.advance();
                    } else {
                        self.advance();
                    }
                }

                Err(ParseError {
                    message: "Expected \\rfloor".to_string(),
                    position: self.pos,
                })
            }

            "rfloor" => Ok(o("")), // Should be consumed by lfloor

            // Ceiling function
            "lceil" => {
                // Parse content until \rceil
                self.skip_whitespace();
                let saved_pos = self.pos;
                let mut depth = 0;

                while let Some(ch) = self.peek() {
                    if ch == '\\' {
                        let cmd_pos = self.pos;
                        self.advance(); // consume \
                        if let Some(next) = self.peek() {
                            if next.is_alphabetic() {
                                // Parse command name
                                let mut cmd_name = String::new();
                                while let Some(c) = self.peek() {
                                    if c.is_alphabetic() {
                                        cmd_name.push(c);
                                        self.advance();
                                    } else {
                                        break;
                                    }
                                }
                                if cmd_name == "rceil" && depth == 0 {
                                    let content_str: String =
                                        self.input[saved_pos..cmd_pos].iter().collect();
                                    let mut content_parser = Parser::new(&content_str);
                                    match content_parser.parse() {
                                        Ok(content) => return Ok(op("ceiling", vec![content])),
                                        Err(_) => {
                                            return Ok(op("ceiling", vec![o(content_str.trim())]));
                                        }
                                    }
                                }
                            }
                        }
                    } else if ch == '{' {
                        depth += 1;
                        self.advance();
                    } else if ch == '}' {
                        depth -= 1;
                        self.advance();
                    } else {
                        self.advance();
                    }
                }

                Err(ParseError {
                    message: "Expected \\rceil".to_string(),
                    position: self.pos,
                })
            }

            "rceil" => Ok(o("")), // Should be consumed by lceil

            // Greek letters (lowercase) - complete alphabet
            "alpha" => Ok(o("\\alpha")),
            "beta" => Ok(o("\\beta")),
            "gamma" => Ok(o("\\gamma")),
            "delta" => Ok(o("\\delta")),
            "epsilon" => Ok(o("\\epsilon")),
            "zeta" => {
                // Check if followed by parentheses (function call)
                self.skip_whitespace();
                if self.peek() == Some('(') {
                    self.advance(); // consume (
                    let mut args = vec![o("\\zeta")];

                    loop {
                        self.skip_whitespace();
                        if self.peek() == Some(')') {
                            self.advance();
                            break;
                        }

                        args.push(self.parse_relational()?);
                        self.skip_whitespace();

                        if self.peek() == Some(',') {
                            self.advance();
                        } else if self.peek() == Some(')') {
                            self.advance();
                            break;
                        } else {
                            return Err(ParseError {
                                message: "Expected ',' or ')' in function call".to_string(),
                                position: self.pos,
                            });
                        }
                    }

                    Ok(op("function_call", args))
                } else {
                    Ok(o("\\zeta"))
                }
            }
            "eta" => Ok(o("\\eta")),
            "theta" => Ok(o("\\theta")),
            "iota" => Ok(o("\\iota")),
            "kappa" => Ok(o("\\kappa")),
            "lambda" => Ok(o("\\lambda")),
            "mu" => Ok(o("\\mu")),
            "nu" => Ok(o("\\nu")),
            "xi" => Ok(o("\\xi")),
            "omicron" => Ok(o("\\omicron")),
            "pi" => Ok(o("\\pi")),
            "rho" => Ok(o("\\rho")),
            "sigma" => Ok(o("\\sigma")),
            "tau" => Ok(o("\\tau")),
            "upsilon" => Ok(o("\\upsilon")),
            "phi" => Ok(o("\\phi")),
            "chi" => Ok(o("\\chi")),
            "psi" => Ok(o("\\psi")),
            "omega" => Ok(o("\\omega")),

            // Greek letter variants
            "varepsilon" => Ok(o("\\varepsilon")),
            "vartheta" => Ok(o("\\vartheta")),
            "varkappa" => Ok(o("\\varkappa")),
            "varpi" => Ok(o("\\varpi")),
            "varrho" => Ok(o("\\varrho")),
            "varsigma" => Ok(o("\\varsigma")),
            "varphi" => Ok(o("\\varphi")),

            // Greek letters (uppercase)
            "Gamma" => {
                // Check if followed by parentheses (function call)
                self.skip_whitespace();
                if self.peek() == Some('(') {
                    self.advance(); // consume (
                    let mut args = vec![o("\\Gamma")];

                    loop {
                        self.skip_whitespace();
                        if self.peek() == Some(')') {
                            self.advance();
                            break;
                        }

                        args.push(self.parse_relational()?);
                        self.skip_whitespace();

                        if self.peek() == Some(',') {
                            self.advance();
                        } else if self.peek() == Some(')') {
                            self.advance();
                            break;
                        } else {
                            return Err(ParseError {
                                message: "Expected ',' or ')' in function call".to_string(),
                                position: self.pos,
                            });
                        }
                    }

                    Ok(op("function_call", args))
                } else {
                    Ok(o("\\Gamma"))
                }
            }
            "Delta" => Ok(o("\\Delta")),
            "Theta" => Ok(o("\\Theta")),
            "Lambda" => Ok(o("\\Lambda")),
            "Xi" => Ok(o("\\Xi")),
            "Pi" => Ok(o("\\Pi")),
            "Sigma" => Ok(o("\\Sigma")),
            "Upsilon" => Ok(o("\\Upsilon")),
            "Phi" => Ok(o("\\Phi")),
            "Psi" => Ok(o("\\Psi")),
            "Omega" => Ok(o("\\Omega")),

            // Hebrew letters
            "aleph" => Ok(o("\\aleph")),
            "beth" => Ok(o("\\beth")),
            "gimel" => Ok(o("\\gimel")),
            "daleth" => Ok(o("\\daleth")),

            // Operators
            "cdot" => Ok(o("\\cdot")),
            "times" => Ok(o("\\times")),
            "div" => Ok(o("\\div")),
            "pm" => Ok(o("\\pm")),
            "nabla" => Ok(o("\\nabla")),
            "partial" => Ok(o("\\partial")),

            // Ellipsis (dots)
            "cdots" => Ok(o("\\cdots")),
            "ldots" => Ok(o("\\ldots")),
            "vdots" => Ok(o("\\vdots")),
            "ddots" => Ok(o("\\ddots")),
            "iddots" => Ok(o("\\iddots")),

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
            "sec" => {
                let arg = self.parse_group_or_parens()?;
                Ok(op("sec", vec![arg]))
            }
            "csc" => {
                let arg = self.parse_group_or_parens()?;
                Ok(op("csc", vec![arg]))
            }
            "cot" => {
                let arg = self.parse_group_or_parens()?;
                Ok(op("cot", vec![arg]))
            }
            "arcsin" => {
                let arg = self.parse_group_or_parens()?;
                Ok(op("arcsin", vec![arg]))
            }
            "arccos" => {
                let arg = self.parse_group_or_parens()?;
                Ok(op("arccos", vec![arg]))
            }
            "arctan" => {
                let arg = self.parse_group_or_parens()?;
                Ok(op("arctan", vec![arg]))
            }
            "sinh" => {
                let arg = self.parse_group_or_parens()?;
                Ok(op("sinh", vec![arg]))
            }
            "cosh" => {
                let arg = self.parse_group_or_parens()?;
                Ok(op("cosh", vec![arg]))
            }
            "tanh" => {
                let arg = self.parse_group_or_parens()?;
                Ok(op("tanh", vec![arg]))
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
                // Check for \langle ... | ... \rangle (inner)
                // or \langle ... \rangle (expectation)
                // or \langle ... | (bra)
                self.skip_whitespace();
                let start = self.pos;

                let mut p_pos = None;
                let mut r_pos = None;
                let mut depth = 0;

                let mut i = start;
                while i < self.input.len() {
                    let ch = self.input[i];
                    if ch == '\\' {
                        // Check for rangle
                        if i + 6 < self.input.len() {
                            let cmd: String = self.input[i + 1..i + 7].iter().collect();
                            if cmd == "rangle" && depth == 0 {
                                r_pos = Some(i);
                                break;
                            }
                        }
                    } else if ch == '|' && depth == 0 {
                        if p_pos.is_none() {
                            p_pos = Some(i);
                        }
                    } else if ch == '{' {
                        depth += 1;
                    } else if ch == '}' {
                        depth -= 1;
                    }
                    i += 1;
                }

                if let Some(r_idx) = r_pos {
                    if let Some(p_idx) = p_pos {
                        // Inner product: < u | v >
                        let bra_str: String = self.input[start..p_idx].iter().collect();
                        let ket_str: String = self.input[p_idx + 1..r_idx].iter().collect();

                        self.pos = r_idx + 7; // skip \rangle

                        let bra_expr = parse_latex(bra_str.trim()).unwrap_or(o(bra_str.trim()));
                        let ket_expr = parse_latex(ket_str.trim()).unwrap_or(o(ket_str.trim()));

                        return Ok(op("inner", vec![bra_expr, ket_expr]));
                    } else {
                        // Expectation: < A >
                        let content_str: String = self.input[start..r_idx].iter().collect();
                        self.pos = r_idx + 7; // skip \rangle

                        let content_expr =
                            parse_latex(content_str.trim()).unwrap_or(o(content_str.trim()));
                        return Ok(op("expectation", vec![content_expr]));
                    }
                } else if let Some(p_idx) = p_pos {
                    // Bra: < u |
                    let content_str: String = self.input[start..p_idx].iter().collect();
                    self.pos = p_idx + 1; // skip |

                    let content_expr =
                        parse_latex(content_str.trim()).unwrap_or(o(content_str.trim()));
                    return Ok(op("bra", vec![content_expr]));
                }

                // Just \langle
                Ok(o("\\langle"))
            }
            "rangle" => Ok(o("\\rangle")),

            // Accent commands
            "hat" => {
                let arg = self.parse_group()?;
                Ok(op("hat", vec![arg]))
            }
            "bar" => {
                let arg = self.parse_group()?;
                Ok(op("bar", vec![arg]))
            }
            "tilde" => {
                let arg = self.parse_group()?;
                Ok(op("tilde", vec![arg]))
            }
            "overline" => {
                let arg = self.parse_group()?;
                Ok(op("overline", vec![arg]))
            }
            "vec" => {
                let arg = self.parse_group()?;
                Ok(op("vector_arrow", vec![arg]))
            }
            "boldsymbol" | "mathbf" => {
                let arg = self.parse_group()?;
                Ok(op("vector_bold", vec![arg]))
            }
            "mathrm" => {
                let arg = self.parse_group()?;
                Ok(op("mathrm", vec![arg]))
            }
            "dot" => {
                let arg = self.parse_group()?;
                Ok(op("dot_accent", vec![arg]))
            }
            "ddot" => {
                let arg = self.parse_group()?;
                Ok(op("ddot_accent", vec![arg]))
            }

            // Number sets
            "mathbb" => {
                let arg = self.parse_group()?;
                Ok(o(format!("\\mathbb{{{}}}", arg.as_string())))
            }

            // Math operators with optional subscripts (min_u, max_x, etc.)
            "min" | "max" | "sup" | "inf" => {
                // Return as object - subscript will be handled by parse_postfix
                Ok(o(format!("\\{}", cmd)))
            }

            // Spacing commands (return empty marker to be skipped)
            "," | ";" | "!" | "quad" | "qquad" | "colon" => Ok(o("__SPACE__")),

            // Text mode - plain text within math
            "text" => {
                let text_content = self.parse_text_group()?;
                Ok(op("text", vec![o(text_content)]))
            }

            // Text formatting (pass through)
            // "mathbf" | "boldsymbol" | "mathrm" handled above as operations

            // Integrals
            "int" => {
                // Could have limits with _ and ^
                Ok(o("\\int"))
            }

            // Sum/Product
            "sum" | "prod" => Ok(o(format!("\\{}", cmd))),

            // Limit operators
            "lim" | "limsup" | "liminf" => {
                // Return as object - subscript will be handled by parse_postfix
                // The subscript should contain "var \to target"
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

fn collapse_script_literals(expr: Expression) -> Expression {
    match expr {
        Expression::Operation { ref name, .. } if name == "scalar_multiply" => {
            if let Some(tokens) = extract_literal_tokens(&expr) {
                Expression::Operation {
                    name: "literal_chain".to_string(),
                    args: tokens,
                }
            } else {
                expr
            }
        }
        _ => expr,
    }
}

fn extract_literal_tokens(expr: &Expression) -> Option<Vec<Expression>> {
    match expr {
        Expression::Const(_) | Expression::Object(_) => Some(vec![expr.clone()]),
        Expression::Operation { name, args } if name == "scalar_multiply" && args.len() == 2 => {
            let mut left_tokens = extract_literal_tokens(&args[0])?;
            let mut right_tokens = extract_literal_tokens(&args[1])?;
            left_tokens.append(&mut right_tokens);
            Some(left_tokens)
        }
        _ => None,
    }
}

impl Expression {
    fn as_string(&self) -> String {
        match self {
            Expression::Const(s) => s.clone(),
            Expression::Object(s) => s.clone(),
            Expression::Operation { .. } => "".to_string(),
            Expression::Placeholder { hint, .. } => hint.clone(),
            Expression::Match { .. } => "match".to_string(),
            Expression::List(_) => "list".to_string(),
        }
    }
}

// Public API
pub fn parse_latex(input: &str) -> Result<Expression, ParseError> {
    let mut parser = Parser::new(input);
    let flat_ast = parser.parse()?;

    // Apply template-based semantic inference
    // If inference fails, returns the original flat AST (graceful fallback)
    let structured_ast = crate::template_inference::infer_templates(flat_ast);

    Ok(structured_ast)
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
    fn parses_all_lowercase_greek() {
        let letters = vec![
            "\\alpha",
            "\\beta",
            "\\gamma",
            "\\delta",
            "\\epsilon",
            "\\zeta",
            "\\eta",
            "\\theta",
            "\\iota",
            "\\kappa",
            "\\lambda",
            "\\mu",
            "\\nu",
            "\\xi",
            "\\omicron",
            "\\pi",
            "\\rho",
            "\\sigma",
            "\\tau",
            "\\upsilon",
            "\\phi",
            "\\chi",
            "\\psi",
            "\\omega",
        ];
        for letter in letters {
            let result = parse_latex(letter);
            assert!(result.is_ok(), "Failed to parse {}", letter);
        }
    }

    #[test]
    fn parses_all_uppercase_greek() {
        let letters = vec![
            "\\Gamma",
            "\\Delta",
            "\\Theta",
            "\\Lambda",
            "\\Xi",
            "\\Pi",
            "\\Sigma",
            "\\Upsilon",
            "\\Phi",
            "\\Psi",
            "\\Omega",
        ];
        for letter in letters {
            let result = parse_latex(letter);
            assert!(result.is_ok(), "Failed to parse {}", letter);
        }
    }

    #[test]
    fn parses_greek_variants() {
        let variants = vec![
            "\\varepsilon",
            "\\vartheta",
            "\\varkappa",
            "\\varpi",
            "\\varrho",
            "\\varsigma",
            "\\varphi",
        ];
        for variant in variants {
            let result = parse_latex(variant);
            assert!(result.is_ok(), "Failed to parse {}", variant);
        }
    }

    #[test]
    fn parses_hebrew_letters() {
        let letters = vec!["\\aleph", "\\beth", "\\gimel", "\\daleth"];
        for letter in letters {
            let result = parse_latex(letter);
            assert!(result.is_ok(), "Failed to parse {}", letter);
        }
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
    fn parses_factorial_operator() {
        let expr = parse_latex("n!").expect("should parse factorial");
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "factorial");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected factorial operation"),
        }
    }

    #[test]
    fn preserves_literal_subscripts() {
        let expr = parse_latex("a_{1n}").expect("should parse subscript");
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "sub");
                match &args[1] {
                    Expression::Operation {
                        name,
                        args: chain_args,
                    } if name == "literal_chain" => {
                        assert_eq!(chain_args.len(), 2);
                        assert!(matches!(&chain_args[0], Expression::Const(s) if s == "1"));
                        assert!(matches!(&chain_args[1], Expression::Object(s) if s == "n"));
                    }
                    other => panic!("Expected literal_chain, got {:?}", other),
                }
            }
            _ => panic!("Expected subscript operation"),
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
            Expression::Operation { name, args } => {
                assert_eq!(name, "Matrix");
                // Check dimensions
                assert!(matches!(&args[0], Expression::Const(s) if s == "2"));
                assert!(matches!(&args[1], Expression::Const(s) if s == "2"));
            }
            _ => panic!("Expected matrix operation"),
        }
    }

    #[test]
    fn parses_matrix_with_fractions() {
        let result = parse_latex("\\begin{bmatrix}\\frac{a}{b}&c\\\\d&e\\end{bmatrix}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "Matrix");
                // First cell (after dimensions) should be a scalar_divide operation, not a string
                match &args[2] {
                    // Skip first two dimension args
                    Expression::Operation { name, .. } => {
                        assert_eq!(name, "scalar_divide");
                    }
                    _ => panic!("Expected first cell to be scalar_divide operation"),
                }
            }
            _ => panic!("Expected matrix operation"),
        }
    }

    #[test]
    fn parses_matrix_with_sqrt() {
        let result =
            parse_latex("\\begin{bmatrix}\\sqrt{2}&\\sqrt{3}\\\\\\sqrt{5}&\\sqrt{7}\\end{bmatrix}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "Matrix");
                // First cell (after dimensions) should be a sqrt operation
                match &args[2] {
                    // Skip first two dimension args
                    Expression::Operation { name, .. } => {
                        assert_eq!(name, "sqrt");
                    }
                    _ => panic!("Expected first cell to be sqrt operation"),
                }
            }
            _ => panic!("Expected matrix operation"),
        }
    }

    #[test]
    fn parses_matrix_with_trig() {
        let result =
            parse_latex("\\begin{bmatrix}\\sin{x}&\\cos{x}\\\\-\\cos{x}&\\sin{x}\\end{bmatrix}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "Matrix");
                // First cell (after dimensions) should be sin operation
                match &args[2] {
                    // Skip first two dimension args
                    Expression::Operation { name, .. } => {
                        assert_eq!(name, "sin");
                    }
                    _ => panic!("Expected first cell to be sin operation"),
                }
            }
            _ => panic!("Expected matrix operation"),
        }
    }

    #[test]
    fn parses_matrix_with_complex_nested() {
        let result = parse_latex(
            "\\begin{bmatrix}\\frac{1}{\\sqrt{2}}&0\\\\0&\\frac{1}{\\sqrt{2}}\\end{bmatrix}",
        );
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "Matrix");
                // First cell (after dimensions) should be scalar_divide with sqrt in denominator
                match &args[2] {
                    // Skip first two dimension args
                    Expression::Operation {
                        name,
                        args: inner_args,
                    } => {
                        assert_eq!(name, "scalar_divide");
                        // Check denominator is sqrt
                        match &inner_args[1] {
                            Expression::Operation { name, .. } => {
                                assert_eq!(name, "sqrt");
                            }
                            _ => panic!("Expected denominator to be sqrt"),
                        }
                    }
                    _ => panic!("Expected first cell to be scalar_divide operation"),
                }
            }
            _ => panic!("Expected matrix operation"),
        }
    }

    #[test]
    fn parses_matrix_with_ellipsis() {
        let result = parse_latex(
            "\\begin{bmatrix}a_{11} & \\cdots & a_{1n}\\\\\\vdots & \\ddots & \\vdots\\\\a_{m1} & \\cdots & a_{mn}\\end{bmatrix}",
        );
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "Matrix");
                // Check that ellipsis symbols are preserved
                // args[3] should be \cdots (args[0-1] are dimensions, args[2] is first element)
                match &args[3] {
                    Expression::Object(s) => {
                        assert_eq!(s, "\\cdots");
                    }
                    _ => panic!("Expected second cell to contain \\cdots"),
                }
            }
            _ => panic!("Expected matrix operation"),
        }
    }

    #[test]
    fn parses_cases_2() {
        let result = parse_latex("\\begin{cases}x^{2} & x \\geq 0\\\\0 & x < 0\\end{cases}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "cases2");
                assert_eq!(args.len(), 4); // expr1, cond1, expr2, cond2
            }
            _ => panic!("Expected cases2 operation"),
        }
    }

    #[test]
    fn parses_cases_3() {
        let result = parse_latex("\\begin{cases}-1 & x < 0\\\\0 & x = 0\\\\1 & x > 0\\end{cases}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "cases3");
                assert_eq!(args.len(), 6); // expr1, cond1, expr2, cond2, expr3, cond3
            }
            _ => panic!("Expected cases3 operation"),
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

    // === COMPREHENSIVE COVERAGE TESTS ===
    // Tests for all implemented features to achieve 100% case coverage

    // Untested lowercase Greek letters
    #[test]
    fn parses_remaining_lowercase_greek() {
        let letters = vec![
            "\\zeta",
            "\\eta",
            "\\iota",
            "\\xi",
            "\\omicron",
            "\\upsilon",
            "\\chi",
        ];
        for letter in letters {
            let result = parse_latex(letter);
            assert!(result.is_ok(), "Failed to parse {}", letter);
        }
    }

    // Untested uppercase Greek letters
    #[test]
    fn parses_remaining_uppercase_greek() {
        let letters = vec!["\\Xi", "\\Pi", "\\Upsilon"];
        for letter in letters {
            let result = parse_latex(letter);
            assert!(result.is_ok(), "Failed to parse {}", letter);
        }
    }

    // Relations: approx, equiv, propto
    #[test]
    fn parses_approximate() {
        let result = parse_latex("x \\approx y");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "approx");
            }
            _ => panic!("Expected approx operation"),
        }
    }

    #[test]
    fn parses_equivalent() {
        let result = parse_latex("x \\equiv y");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "equiv");
            }
            _ => panic!("Expected equiv operation"),
        }
    }

    #[test]
    fn parses_proportional() {
        let result = parse_latex("x \\propto y");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "proportional");
            }
            _ => panic!("Expected proportional operation"),
        }
    }

    // Set operations: intersection (union already tested)
    #[test]
    fn parses_intersection() {
        let result = parse_latex("A \\cap B");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "intersection");
            }
            _ => panic!("Expected intersection operation"),
        }
    }

    // Logical implications
    #[test]
    fn parses_implies() {
        let result = parse_latex("P \\Rightarrow Q");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_iff() {
        let result = parse_latex("P \\Leftrightarrow Q");
        assert!(result.is_ok());
    }

    // Logarithmic and exponential functions
    #[test]
    fn parses_natural_log() {
        let result = parse_latex("\\ln{x}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "ln");
            }
            _ => panic!("Expected ln operation"),
        }
    }

    #[test]
    fn parses_log() {
        let result = parse_latex("\\log{x}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "log");
            }
            _ => panic!("Expected log operation"),
        }
    }

    #[test]
    fn parses_exponential() {
        let result = parse_latex("\\exp{x}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "exp");
            }
            _ => panic!("Expected exp operation"),
        }
    }

    // Functions with parentheses (in addition to braces)
    #[test]
    fn parses_ln_with_parentheses() {
        let result = parse_latex("\\ln(x)");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_log_with_parentheses() {
        let result = parse_latex("\\log(x+1)");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_exp_with_parentheses() {
        let result = parse_latex("\\exp(i\\pi)");
        assert!(result.is_ok());
    }

    // Hat operator
    #[test]
    fn parses_hat_operator() {
        let result = parse_latex("\\hat{H}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "hat");
            }
            _ => panic!("Expected hat operation"),
        }
    }

    // Number sets (mathbb)
    #[test]
    fn parses_number_sets() {
        let sets = vec![
            "\\mathbb{R}",
            "\\mathbb{C}",
            "\\mathbb{N}",
            "\\mathbb{Z}",
            "\\mathbb{Q}",
        ];
        for set in sets {
            let result = parse_latex(set);
            assert!(result.is_ok(), "Failed to parse {}", set);
        }
    }

    // Min/max functions
    #[test]
    fn parses_min_function() {
        let result = parse_latex("\\min");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_max_function() {
        let result = parse_latex("\\max");
        assert!(result.is_ok());
    }

    // Text formatting
    #[test]
    fn parses_mathbf() {
        let result = parse_latex("\\mathbf{x}");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_boldsymbol() {
        let result = parse_latex("\\boldsymbol{v}");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_mathrm() {
        let result = parse_latex("\\mathrm{d}x");
        assert!(result.is_ok());
    }

    // Calculus symbols
    #[test]
    fn parses_integral_symbol() {
        let result = parse_latex("\\int f(x)");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_sum_symbol() {
        let result = parse_latex("\\sum_{i=1}^{n} i");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_product_symbol() {
        let result = parse_latex("\\prod_{i=1}^{n} i");
        assert!(result.is_ok());
    }

    // Integral with bounds
    #[test]
    fn parses_definite_integral() {
        let result = parse_latex("\\int_{0}^{1} x");
        assert!(result.is_ok());
    }

    // Arrow symbols
    #[test]
    fn parses_to_arrow() {
        let result = parse_latex("x \\to y");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_rightarrow() {
        let result = parse_latex("x \\rightarrow y");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_mapsto() {
        let result = parse_latex("x \\mapsto y");
        assert!(result.is_ok());
    }

    // Plain matrix environment
    #[test]
    fn parses_plain_matrix() {
        let result = parse_latex("\\begin{matrix}a&b\\\\c&d\\end{matrix}");
        assert!(result.is_ok());
    }

    // Nth root (sqrt with index)
    #[test]
    fn parses_nth_root() {
        let result = parse_latex("\\sqrt[3]{x}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "nth_root");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected nth_root operation"),
        }
    }

    #[test]
    fn parses_cube_root() {
        let result = parse_latex("\\sqrt[3]{8}");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_fourth_root() {
        let result = parse_latex("\\sqrt[4]{16}");
        assert!(result.is_ok());
    }

    // Operators in expressions
    #[test]
    fn parses_cdot_operator() {
        let result = parse_latex("a \\cdot b");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_times_operator() {
        let result = parse_latex("a \\times b");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_div_operator() {
        let result = parse_latex("a \\div b");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_pm_operator() {
        let result = parse_latex("x = 1 \\pm 0.1");
        assert!(result.is_ok());
    }

    // Differential operators
    #[test]
    fn parses_nabla() {
        let result = parse_latex("\\nabla f");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_partial() {
        let result = parse_latex("\\partial f");
        assert!(result.is_ok());
    }

    // Delimiters
    #[test]
    fn parses_left_right_parentheses() {
        let result = parse_latex("\\left(x\\right)");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_left_right_braces() {
        let result = parse_latex("\\left\\{x\\right\\}");
        assert!(result.is_ok());
    }

    // Spacing commands (should be handled gracefully)
    #[test]
    fn parses_thin_space() {
        let result = parse_latex("a \\, b");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_medium_space() {
        let result = parse_latex("a \\; b");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_quad_space() {
        let result = parse_latex("a \\quad b");
        assert!(result.is_ok());
    }

    // Complex nested expressions (edge cases)
    #[test]
    fn parses_deeply_nested_fractions() {
        let result = parse_latex("\\frac{\\frac{a}{b}}{\\frac{c}{d}}");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_nested_roots() {
        let result = parse_latex("\\sqrt{\\sqrt{x}}");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_mixed_subscript_superscript() {
        let result = parse_latex("x_{i}^{2}");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_complex_tensor_indices() {
        let result = parse_latex("T^{\\mu\\nu}_{\\rho\\sigma}");
        assert!(result.is_ok());
    }

    // Matrix variants
    #[test]
    fn parses_pmatrix_3x3() {
        let result = parse_latex("\\begin{pmatrix}1&2&3\\\\4&5&6\\\\7&8&9\\end{pmatrix}");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_vmatrix_2x2() {
        let result = parse_latex("\\begin{vmatrix}a&b\\\\c&d\\end{vmatrix}");
        assert!(result.is_ok());
    }

    // Forall and exists
    #[test]
    fn parses_forall() {
        let result = parse_latex("\\forall x");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_exists() {
        let result = parse_latex("\\exists x");
        assert!(result.is_ok());
    }

    // Ket vector (we have bra tested, ensure ket is too)
    #[test]
    fn parses_ket_vector() {
        let result = parse_latex("|\\psi\\rangle");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "ket");
            }
            _ => panic!("Expected ket operation"),
        }
    }

    // Inner product notation
    #[test]
    fn parses_inner_product() {
        let result = parse_latex("\\langle u, v \\rangle");
        assert!(result.is_ok());
    }

    // Real-world complex expressions
    #[test]
    fn parses_einstein_notation() {
        let result = parse_latex("G_{\\mu\\nu} = \\kappa T_{\\mu\\nu}");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_schrodinger_equation() {
        let result = parse_latex(
            "i\\hbar\\frac{\\partial}{\\partial t}|\\psi\\rangle = \\hat{H}|\\psi\\rangle",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn parses_maxwell_equation() {
        let result = parse_latex(
            "\\nabla \\times \\mathbf{E} = -\\frac{\\partial \\mathbf{B}}{\\partial t}",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn parses_euler_formula() {
        let result = parse_latex("e^{i\\pi} + 1 = 0");
        assert!(result.is_ok());
    }

    // === Accent Commands ===

    #[test]
    fn parses_bar_accent() {
        let result = parse_latex("\\bar{x}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "bar");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected bar operation"),
        }
    }

    #[test]
    fn parses_tilde_accent() {
        let result = parse_latex("\\tilde{x}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "tilde");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected tilde operation"),
        }
    }

    #[test]
    fn parses_overline_accent() {
        let result = parse_latex("\\overline{xy}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "overline");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected overline operation"),
        }
    }

    #[test]
    fn parses_dot_accent() {
        let result = parse_latex("\\dot{x}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "dot_accent");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected dot_accent operation"),
        }
    }

    #[test]
    fn parses_ddot_accent() {
        let result = parse_latex("\\ddot{x}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "ddot_accent");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected ddot_accent operation"),
        }
    }

    #[test]
    fn parses_accents_in_equations() {
        // Common physics notation: \bar{p} = m\bar{v}
        let result = parse_latex("\\bar{p} = m\\bar{v}");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_time_derivatives() {
        // \dot{x}, \ddot{x} for velocity and acceleration
        let result1 = parse_latex("\\dot{x}");
        let result2 = parse_latex("\\ddot{x}");
        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    // === Text Mode Support ===

    #[test]
    fn parses_text_simple() {
        let result = parse_latex("\\text{hello}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "text");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected text operation"),
        }
    }

    #[test]
    fn parses_text_with_spaces() {
        let result = parse_latex("\\text{if } x > 0");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_text_in_piecewise() {
        let result = parse_latex(
            "\\begin{cases}x^{2} & \\text{if } x \\geq 0\\\\0 & \\text{otherwise}\\end{cases}",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn parses_text_annotation() {
        let result = parse_latex("\\forall x \\in \\mathbb{R}\\text{, we have } x^{2} \\geq 0");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_text_with_punctuation() {
        let result = parse_latex("\\text{for all } x \\text{, we have:}");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_all_ellipsis_commands() {
        let ellipsis = vec!["\\cdots", "\\ldots", "\\vdots", "\\ddots", "\\iddots"];
        for dots in ellipsis {
            let result = parse_latex(dots);
            assert!(result.is_ok(), "Failed to parse {}", dots);
            // Verify it's an Object containing the command
            if let Ok(Expression::Object(s)) = result {
                assert_eq!(s, dots);
            } else {
                panic!("Expected Object for {}", dots);
            }
        }
    }

    #[test]
    fn parses_cdots() {
        let result = parse_latex("\\cdots");
        assert!(result.is_ok());
        if let Ok(Expression::Object(s)) = result {
            assert_eq!(s, "\\cdots");
        }
    }

    #[test]
    fn parses_vdots() {
        let result = parse_latex("\\vdots");
        assert!(result.is_ok());
        if let Ok(Expression::Object(s)) = result {
            assert_eq!(s, "\\vdots");
        }
    }

    #[test]
    fn parses_ddots() {
        let result = parse_latex("\\ddots");
        assert!(result.is_ok());
        if let Ok(Expression::Object(s)) = result {
            assert_eq!(s, "\\ddots");
        }
    }

    #[test]
    fn parses_ellipsis_in_matrix() {
        // Common use case: ellipsis in matrices
        let result = parse_latex(
            "\\begin{bmatrix}a_{11} & \\cdots & a_{1n}\\\\\\vdots & \\ddots & \\vdots\\\\a_{m1} & \\cdots & a_{mn}\\end{bmatrix}",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn parses_ldots_in_sequence() {
        // Ellipsis in sequence: 1, 2, 3, \ldots, n
        let result = parse_latex("1, 2, 3, \\ldots, n");
        assert!(result.is_ok());
    }

    // ===== BINOMIAL COEFFICIENT TESTS =====

    #[test]
    fn parses_binomial_coefficient() {
        let result = parse_latex("\\binom{n}{k}");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "binomial");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected binomial operation"),
        }
    }

    #[test]
    fn parses_binomial_with_numbers() {
        let result = parse_latex("\\binom{5}{2}");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_binomial_in_equation() {
        // Pascal's triangle formula
        let result = parse_latex("\\binom{n}{k} = \\binom{n-1}{k-1} + \\binom{n-1}{k}");
        assert!(result.is_ok());
    }

    // ===== FLOOR AND CEILING TESTS =====

    #[test]
    fn parses_floor_function() {
        let result = parse_latex("\\lfloor x \\rfloor");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "floor");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected floor operation"),
        }
    }

    #[test]
    fn parses_ceiling_function() {
        let result = parse_latex("\\lceil x \\rceil");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "ceiling");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected ceiling operation"),
        }
    }

    #[test]
    fn parses_floor_with_fraction() {
        // Common use case: floor of a fraction
        let result = parse_latex("\\lfloor \\frac{n}{2} \\rfloor");
        assert!(result.is_ok());
        let expr = result.unwrap();
        match expr {
            Expression::Operation { name, args } => {
                assert_eq!(name, "floor");
                assert_eq!(args.len(), 1);
                // Check that the argument is a division
                match &args[0] {
                    Expression::Operation {
                        name: inner_name, ..
                    } => {
                        assert_eq!(inner_name, "scalar_divide");
                    }
                    _ => panic!("Expected division inside floor"),
                }
            }
            _ => panic!("Expected floor operation"),
        }
    }

    #[test]
    fn parses_ceiling_with_division() {
        let result = parse_latex("\\lceil \\frac{n}{k} \\rceil");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_floor_with_subscript() {
        // Floor of indexed variable
        let result = parse_latex("\\lfloor x_i \\rfloor");
        assert!(result.is_ok());
    }

    #[test]
    fn parses_floor_and_ceiling_in_equation() {
        // Common in number theory: floor and ceiling relation
        let result = parse_latex("\\lceil x \\rceil = \\lfloor x \\rfloor + 1");
        assert!(result.is_ok());
    }
}
