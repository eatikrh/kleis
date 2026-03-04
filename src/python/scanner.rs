//! Python line scanner that emits Kleis AST.
//!
//! A hand-written, zero-dependency scanner that tracks indentation to build
//! a nested AST for Python source code. Produces the same Expression types
//! as a Kleis-native parser would.
//!
//! Reference: Python 3 grammar (https://docs.python.org/3/reference/grammar.html)
//! and Ruff parser source (MIT) for edge-case patterns.

use crate::ast::Expression;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Parse Python source and return a Kleis `PyModule(items, line_count, parse_errors)`.
pub fn scan_python(source: &str) -> Result<Expression, String> {
    let lines: Vec<&str> = source.lines().collect();
    let line_count = lines.len();
    let mut scanner = Scanner::new(&lines);
    let items = scanner.scan_block(0);
    let parse_errors = scanner.errors;

    let kleis_items: Vec<Expression> = items.into_iter().map(|n| n.to_item_expr()).collect();

    Ok(mk_op(
        "PyModule",
        vec![
            mk_list(kleis_items),
            mk_int(line_count as i64),
            mk_int(parse_errors as i64),
        ],
    ))
}

// ---------------------------------------------------------------------------
// Internal AST (intermediate, before conversion to Kleis Expression)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum PyNode {
    Import {
        names: Vec<String>,
        line: usize,
    },
    FromImport {
        module: String,
        names: Vec<String>,
        is_wildcard: bool,
        line: usize,
    },
    Function {
        name: String,
        params: String,
        return_type: String,
        is_async: bool,
        body: Vec<PyNode>,
        decorators: Vec<(String, usize)>,
        line: usize,
    },
    Class {
        name: String,
        bases: Vec<String>,
        body: Vec<PyNode>,
        decorators: Vec<(String, usize)>,
        line: usize,
    },
    Assign {
        target: String,
        has_type_hint: bool,
        line: usize,
    },
    Return {
        line: usize,
    },
    Raise {
        exception: String,
        line: usize,
    },
    Yield {
        line: usize,
    },
    If {
        body: Vec<PyNode>,
        else_body: Vec<PyNode>,
        line: usize,
    },
    For {
        target: String,
        body: Vec<PyNode>,
        line: usize,
    },
    While {
        body: Vec<PyNode>,
        line: usize,
    },
    Try {
        body: Vec<PyNode>,
        handlers: Vec<ExceptHandler>,
        finally_body: Vec<PyNode>,
        line: usize,
    },
    With {
        line: usize,
    },
    Pass {
        line: usize,
    },
    Break {
        line: usize,
    },
    Continue {
        line: usize,
    },
    Global {
        names: Vec<String>,
        line: usize,
    },
    Nonlocal {
        names: Vec<String>,
        line: usize,
    },
    Expr {
        text: String,
        line: usize,
    },
}

#[derive(Debug, Clone)]
struct ExceptHandler {
    exception_type: String,
    name: String,
    body: Vec<PyNode>,
    line: usize,
}

// ---------------------------------------------------------------------------
// Conversion from internal AST to Kleis Expression
// ---------------------------------------------------------------------------

impl PyNode {
    fn to_item_expr(&self) -> Expression {
        match self {
            PyNode::Import { .. } => mk_op("PyItemImport", vec![self.to_expr()]),
            PyNode::FromImport { .. } => mk_op("PyItemFromImport", vec![self.to_expr()]),
            PyNode::Function { .. } => mk_op("PyItemFunction", vec![self.to_expr()]),
            PyNode::Class { .. } => mk_op("PyItemClass", vec![self.to_expr()]),
            _ => mk_op("PyItemStmt", vec![self.to_expr()]),
        }
    }

    fn to_stmt_expr(&self) -> Expression {
        match self {
            PyNode::Import { .. } => mk_op("PyStmtImport", vec![self.to_expr()]),
            PyNode::FromImport { .. } => mk_op("PyStmtFromImport", vec![self.to_expr()]),
            PyNode::Function { .. } => mk_op("PyStmtFunctionDef", vec![self.to_expr()]),
            PyNode::Class { .. } => mk_op("PyStmtClassDef", vec![self.to_expr()]),
            _ => self.to_expr(),
        }
    }

    fn to_expr(&self) -> Expression {
        match self {
            PyNode::Import { names, line } => mk_op(
                "PyImport",
                vec![
                    mk_list(
                        names
                            .iter()
                            .map(|n| Expression::String(n.clone()))
                            .collect(),
                    ),
                    mk_int(*line as i64),
                ],
            ),
            PyNode::FromImport {
                module,
                names,
                is_wildcard,
                line,
            } => mk_op(
                "PyFromImport",
                vec![
                    Expression::String(module.clone()),
                    mk_list(
                        names
                            .iter()
                            .map(|n| Expression::String(n.clone()))
                            .collect(),
                    ),
                    mk_bool(*is_wildcard),
                    mk_int(*line as i64),
                ],
            ),
            PyNode::Function {
                name,
                params,
                return_type,
                is_async,
                body,
                decorators,
                line,
            } => mk_op(
                "PyFunction",
                vec![
                    Expression::String(name.clone()),
                    Expression::String(params.clone()),
                    Expression::String(return_type.clone()),
                    mk_bool(*is_async),
                    mk_list(body.iter().map(|n| n.to_stmt_expr()).collect()),
                    mk_list(decorators.iter().map(|(n, l)| dec_expr(n, *l)).collect()),
                    mk_int(*line as i64),
                ],
            ),
            PyNode::Class {
                name,
                bases,
                body,
                decorators,
                line,
            } => {
                let methods: Vec<Expression> = body
                    .iter()
                    .filter_map(|n| match n {
                        PyNode::Function { .. } => Some(n.to_expr()),
                        _ => None,
                    })
                    .collect();
                let body_exprs: Vec<Expression> = body.iter().map(|n| n.to_stmt_expr()).collect();
                mk_op(
                    "PyClass",
                    vec![
                        Expression::String(name.clone()),
                        mk_list(
                            bases
                                .iter()
                                .map(|b| Expression::String(b.clone()))
                                .collect(),
                        ),
                        mk_list(methods),
                        mk_list(body_exprs),
                        mk_list(decorators.iter().map(|(n, l)| dec_expr(n, *l)).collect()),
                        mk_int(*line as i64),
                    ],
                )
            }
            PyNode::Assign {
                target,
                has_type_hint,
                line,
            } => mk_op(
                "PyStmtAssign",
                vec![
                    Expression::String(target.clone()),
                    mk_bool(*has_type_hint),
                    mk_int(*line as i64),
                ],
            ),
            PyNode::Return { line } => mk_op("PyStmtReturn", vec![mk_int(*line as i64)]),
            PyNode::Raise { exception, line } => mk_op(
                "PyStmtRaise",
                vec![Expression::String(exception.clone()), mk_int(*line as i64)],
            ),
            PyNode::Yield { line } => mk_op("PyStmtYield", vec![mk_int(*line as i64)]),
            PyNode::If {
                body,
                else_body,
                line,
            } => mk_op(
                "PyStmtIf",
                vec![
                    mk_list(body.iter().map(|n| n.to_stmt_expr()).collect()),
                    mk_list(else_body.iter().map(|n| n.to_stmt_expr()).collect()),
                    mk_int(*line as i64),
                ],
            ),
            PyNode::For { target, body, line } => mk_op(
                "PyStmtFor",
                vec![
                    Expression::String(target.clone()),
                    mk_list(body.iter().map(|n| n.to_stmt_expr()).collect()),
                    mk_int(*line as i64),
                ],
            ),
            PyNode::While { body, line } => mk_op(
                "PyStmtWhile",
                vec![
                    mk_list(body.iter().map(|n| n.to_stmt_expr()).collect()),
                    mk_int(*line as i64),
                ],
            ),
            PyNode::Try {
                body,
                handlers,
                finally_body,
                line,
            } => mk_op(
                "PyStmtTry",
                vec![
                    mk_list(body.iter().map(|n| n.to_stmt_expr()).collect()),
                    mk_list(handlers.iter().map(|h| h.to_expr()).collect()),
                    mk_list(finally_body.iter().map(|n| n.to_stmt_expr()).collect()),
                    mk_int(*line as i64),
                ],
            ),
            PyNode::With { line } => mk_op("PyStmtWith", vec![mk_int(*line as i64)]),
            PyNode::Pass { line } => mk_op("PyStmtPass", vec![mk_int(*line as i64)]),
            PyNode::Break { line } => mk_op("PyStmtBreak", vec![mk_int(*line as i64)]),
            PyNode::Continue { line } => mk_op("PyStmtContinue", vec![mk_int(*line as i64)]),
            PyNode::Global { names, line } => mk_op(
                "PyStmtGlobal",
                vec![
                    mk_list(
                        names
                            .iter()
                            .map(|n| Expression::String(n.clone()))
                            .collect(),
                    ),
                    mk_int(*line as i64),
                ],
            ),
            PyNode::Nonlocal { names, line } => mk_op(
                "PyStmtNonlocal",
                vec![
                    mk_list(
                        names
                            .iter()
                            .map(|n| Expression::String(n.clone()))
                            .collect(),
                    ),
                    mk_int(*line as i64),
                ],
            ),
            PyNode::Expr { text, line } => mk_op(
                "PyStmtExpr",
                vec![Expression::String(text.clone()), mk_int(*line as i64)],
            ),
        }
    }
}

impl ExceptHandler {
    fn to_expr(&self) -> Expression {
        mk_op(
            "PyExceptHandler",
            vec![
                Expression::String(self.exception_type.clone()),
                Expression::String(self.name.clone()),
                mk_list(self.body.iter().map(|n| n.to_stmt_expr()).collect()),
                mk_int(self.line as i64),
            ],
        )
    }
}

// ---------------------------------------------------------------------------
// Expression helpers
// ---------------------------------------------------------------------------

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

/// Convert a Vec<Expression> into Cons(x, Cons(y, ... Nil)) for Kleis pattern matching.
fn mk_list(items: Vec<Expression>) -> Expression {
    items
        .into_iter()
        .rev()
        .fold(mk_op("Nil", vec![]), |acc, item| {
            mk_op("Cons", vec![item, acc])
        })
}

fn dec_expr(name: &str, line: usize) -> Expression {
    mk_op(
        "PyDecorator",
        vec![Expression::String(name.to_string()), mk_int(line as i64)],
    )
}

// ---------------------------------------------------------------------------
// Scanner — indent-based line scanner
// ---------------------------------------------------------------------------

struct Scanner<'a> {
    lines: &'a [&'a str],
    pos: usize,
    errors: usize,
    in_triple_quote: bool,
}

impl<'a> Scanner<'a> {
    fn new(lines: &'a [&'a str]) -> Self {
        Scanner {
            lines,
            pos: 0,
            errors: 0,
            in_triple_quote: false,
        }
    }

    /// Scan a block of statements at a given indentation level.
    /// Stops when a line has indent < min_indent or EOF.
    fn scan_block(&mut self, min_indent: usize) -> Vec<PyNode> {
        let mut nodes = Vec::new();
        let mut pending_decorators: Vec<(String, usize)> = Vec::new();

        while self.pos < self.lines.len() {
            let raw_line = self.lines[self.pos];
            let indent = leading_spaces(raw_line);
            let trimmed = raw_line.trim();

            // Handle triple-quote strings spanning multiple lines
            if self.in_triple_quote {
                if contains_triple_quote(trimmed) {
                    self.in_triple_quote = false;
                }
                self.pos += 1;
                continue;
            }

            // Skip blank lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                self.pos += 1;
                continue;
            }

            // Check for triple-quote opening (docstrings, multi-line strings)
            if starts_triple_quote(trimmed) && !closes_triple_quote_same_line(trimmed) {
                self.in_triple_quote = true;
                self.pos += 1;
                continue;
            }

            // Dedent — block is over
            if indent < min_indent {
                break;
            }

            // Only process lines at our indent level; deeper lines belong to sub-blocks
            if indent > min_indent && nodes.is_empty() && pending_decorators.is_empty() {
                // First line is deeper than expected — adjust min_indent
                // This handles cases like class bodies after `class Foo:` on same indent
                return self.scan_block(indent);
            }
            if indent > min_indent {
                // Stray deeper line outside a block — treat as expression
                self.pos += 1;
                continue;
            }

            let line_num = self.pos + 1; // 1-based

            // Decorators
            if trimmed.starts_with('@') {
                let dec_name = extract_decorator_name(trimmed);
                pending_decorators.push((dec_name, line_num));
                self.pos += 1;
                continue;
            }

            // async def
            if trimmed.starts_with("async def ") {
                let node = self.parse_function(trimmed, true, &pending_decorators, line_num);
                pending_decorators.clear();
                nodes.push(node);
                continue;
            }

            // def
            if trimmed.starts_with("def ") {
                let node = self.parse_function(trimmed, false, &pending_decorators, line_num);
                pending_decorators.clear();
                nodes.push(node);
                continue;
            }

            // class
            if trimmed.starts_with("class ") {
                let node = self.parse_class(trimmed, &pending_decorators, line_num);
                pending_decorators.clear();
                nodes.push(node);
                continue;
            }

            // Discard stale decorators before non-def/class
            if !pending_decorators.is_empty() {
                pending_decorators.clear();
            }

            // import
            if trimmed.starts_with("import ") {
                nodes.push(self.parse_import(trimmed, line_num));
                self.pos += 1;
                continue;
            }

            // from ... import
            if trimmed.starts_with("from ") {
                nodes.push(self.parse_from_import(trimmed, line_num));
                self.pos += 1;
                continue;
            }

            // try:
            if trimmed == "try:" {
                nodes.push(self.parse_try(indent, line_num));
                continue;
            }

            // if / elif
            if trimmed.starts_with("if ") || trimmed.starts_with("elif ") {
                nodes.push(self.parse_if(indent, line_num));
                continue;
            }

            // for
            if trimmed.starts_with("for ") {
                nodes.push(self.parse_for(trimmed, indent, line_num));
                continue;
            }

            // while
            if trimmed.starts_with("while ") {
                nodes.push(self.parse_while(indent, line_num));
                continue;
            }

            // with
            if trimmed.starts_with("with ") {
                nodes.push(self.parse_with(indent, line_num));
                continue;
            }

            // Simple statements
            if trimmed == "pass" || trimmed == "pass " {
                nodes.push(PyNode::Pass { line: line_num });
                self.pos += 1;
                continue;
            }
            if trimmed.starts_with("return") {
                nodes.push(PyNode::Return { line: line_num });
                self.pos += 1;
                continue;
            }
            if trimmed.starts_with("raise") {
                let exc = trimmed
                    .strip_prefix("raise")
                    .unwrap_or("")
                    .trim()
                    .to_string();
                nodes.push(PyNode::Raise {
                    exception: exc,
                    line: line_num,
                });
                self.pos += 1;
                continue;
            }
            if trimmed.starts_with("yield") {
                nodes.push(PyNode::Yield { line: line_num });
                self.pos += 1;
                continue;
            }
            if trimmed == "break" {
                nodes.push(PyNode::Break { line: line_num });
                self.pos += 1;
                continue;
            }
            if trimmed == "continue" {
                nodes.push(PyNode::Continue { line: line_num });
                self.pos += 1;
                continue;
            }
            if trimmed.starts_with("global ") {
                let names = parse_name_list(trimmed.strip_prefix("global ").unwrap_or(""));
                nodes.push(PyNode::Global {
                    names,
                    line: line_num,
                });
                self.pos += 1;
                continue;
            }
            if trimmed.starts_with("nonlocal ") {
                let names = parse_name_list(trimmed.strip_prefix("nonlocal ").unwrap_or(""));
                nodes.push(PyNode::Nonlocal {
                    names,
                    line: line_num,
                });
                self.pos += 1;
                continue;
            }

            // Assignment (with optional type hint)
            if let Some(assign) = try_parse_assign(trimmed, line_num) {
                nodes.push(assign);
                self.pos += 1;
                continue;
            }

            // Fallback: expression statement
            nodes.push(PyNode::Expr {
                text: trimmed.to_string(),
                line: line_num,
            });
            self.pos += 1;
        }

        nodes
    }

    // --- Statement parsers ---

    fn parse_function(
        &mut self,
        trimmed: &str,
        is_async: bool,
        decorators: &[(String, usize)],
        line: usize,
    ) -> PyNode {
        let after_def = if is_async {
            trimmed.strip_prefix("async def ").unwrap_or("")
        } else {
            trimmed.strip_prefix("def ").unwrap_or("")
        };

        let name = extract_identifier(after_def);
        let params = extract_parens(after_def);
        let return_type = extract_return_type(after_def);

        let body_indent = self.advance_into_block();
        let body = if body_indent > 0 {
            self.scan_block(body_indent)
        } else {
            Vec::new()
        };

        PyNode::Function {
            name,
            params,
            return_type,
            is_async,
            body,
            decorators: decorators.to_vec(),
            line,
        }
    }

    fn parse_class(
        &mut self,
        trimmed: &str,
        decorators: &[(String, usize)],
        line: usize,
    ) -> PyNode {
        let after_class = trimmed.strip_prefix("class ").unwrap_or("");
        let name = extract_identifier(after_class);
        let bases = extract_bases(after_class);

        let body_indent = self.advance_into_block();
        let body = if body_indent > 0 {
            self.scan_block(body_indent)
        } else {
            Vec::new()
        };

        PyNode::Class {
            name,
            bases,
            body,
            decorators: decorators.to_vec(),
            line,
        }
    }

    fn parse_import(&self, trimmed: &str, line: usize) -> PyNode {
        let after = trimmed.strip_prefix("import ").unwrap_or("");
        let names = parse_name_list(after);
        PyNode::Import { names, line }
    }

    fn parse_from_import(&self, trimmed: &str, line: usize) -> PyNode {
        // from module import names
        let after_from = trimmed.strip_prefix("from ").unwrap_or("");
        let (module, rest) = if let Some(idx) = after_from.find(" import ") {
            (
                after_from[..idx].trim().to_string(),
                after_from[idx + 8..].trim(),
            )
        } else {
            (after_from.trim().to_string(), "")
        };

        let is_wildcard = rest == "*";
        let names = if is_wildcard {
            vec!["*".to_string()]
        } else {
            parse_name_list(rest)
        };

        PyNode::FromImport {
            module,
            names,
            is_wildcard,
            line,
        }
    }

    fn parse_try(&mut self, indent: usize, line: usize) -> PyNode {
        // Consume `try:` line
        self.pos += 1;
        let body_indent = self.find_block_indent(indent);
        let body = if body_indent > indent {
            self.scan_block(body_indent)
        } else {
            Vec::new()
        };

        let mut handlers = Vec::new();
        let mut finally_body = Vec::new();

        // Parse except / else / finally clauses
        while self.pos < self.lines.len() {
            let raw = self.lines[self.pos];
            let ind = leading_spaces(raw);
            let t = raw.trim();

            if ind != indent {
                break;
            }

            if t.starts_with("except") {
                let handler = self.parse_except_handler(indent);
                handlers.push(handler);
            } else if t == "else:" {
                self.pos += 1;
                let else_indent = self.find_block_indent(indent);
                if else_indent > indent {
                    // else body — we don't store it separately, just scan past it
                    let _else_body = self.scan_block(else_indent);
                }
            } else if t == "finally:" {
                self.pos += 1;
                let fin_indent = self.find_block_indent(indent);
                if fin_indent > indent {
                    finally_body = self.scan_block(fin_indent);
                }
            } else {
                break;
            }
        }

        PyNode::Try {
            body,
            handlers,
            finally_body,
            line,
        }
    }

    fn parse_except_handler(&mut self, parent_indent: usize) -> ExceptHandler {
        let raw = self.lines[self.pos];
        let trimmed = raw.trim();
        let line = self.pos + 1;

        let (exception_type, name) = parse_except_header(trimmed);

        self.pos += 1;
        let body_indent = self.find_block_indent(parent_indent);
        let body = if body_indent > parent_indent {
            self.scan_block(body_indent)
        } else {
            Vec::new()
        };

        ExceptHandler {
            exception_type,
            name,
            body,
            line,
        }
    }

    fn parse_if(&mut self, indent: usize, line: usize) -> PyNode {
        // Consume if/elif line
        self.pos += 1;
        let body_indent = self.find_block_indent(indent);
        let body = if body_indent > indent {
            self.scan_block(body_indent)
        } else {
            Vec::new()
        };

        let mut else_body = Vec::new();

        // Check for elif / else
        if self.pos < self.lines.len() {
            let raw = self.lines[self.pos];
            let ind = leading_spaces(raw);
            let t = raw.trim();

            if ind == indent {
                if t.starts_with("elif ") {
                    // elif becomes a nested If in the else_body
                    let elif_line = self.pos + 1;
                    let elif_node = self.parse_if(indent, elif_line);
                    else_body.push(elif_node);
                } else if t == "else:" {
                    self.pos += 1;
                    let else_indent = self.find_block_indent(indent);
                    if else_indent > indent {
                        else_body = self.scan_block(else_indent);
                    }
                }
            }
        }

        PyNode::If {
            body,
            else_body,
            line,
        }
    }

    fn parse_for(&mut self, trimmed: &str, indent: usize, line: usize) -> PyNode {
        let after_for = trimmed.strip_prefix("for ").unwrap_or("");
        let target = if let Some(idx) = after_for.find(" in ") {
            after_for[..idx].trim().to_string()
        } else {
            after_for.trim().to_string()
        };

        self.pos += 1;
        let body_indent = self.find_block_indent(indent);
        let body = if body_indent > indent {
            self.scan_block(body_indent)
        } else {
            Vec::new()
        };

        PyNode::For { target, body, line }
    }

    fn parse_while(&mut self, indent: usize, line: usize) -> PyNode {
        self.pos += 1;
        let body_indent = self.find_block_indent(indent);
        let body = if body_indent > indent {
            self.scan_block(body_indent)
        } else {
            Vec::new()
        };

        PyNode::While { body, line }
    }

    fn parse_with(&mut self, indent: usize, line: usize) -> PyNode {
        self.pos += 1;
        let body_indent = self.find_block_indent(indent);
        if body_indent > indent {
            let _body = self.scan_block(body_indent);
        }
        PyNode::With { line }
    }

    // --- Helpers ---

    /// Advance past the current line (which ends with `:`) and return the
    /// indentation level of the body block.
    fn advance_into_block(&mut self) -> usize {
        self.pos += 1;
        self.find_block_indent(0)
    }

    /// Find the indent of the next non-blank, non-comment line.
    fn find_block_indent(&self, _parent_indent: usize) -> usize {
        let mut i = self.pos;
        while i < self.lines.len() {
            let raw = self.lines[i];
            let t = raw.trim();
            if !t.is_empty() && !t.starts_with('#') {
                return leading_spaces(raw);
            }
            i += 1;
        }
        0
    }
}

// ---------------------------------------------------------------------------
// String helpers
// ---------------------------------------------------------------------------

fn leading_spaces(line: &str) -> usize {
    let mut count = 0;
    for ch in line.chars() {
        match ch {
            ' ' => count += 1,
            '\t' => count += 4, // treat tab as 4 spaces
            _ => break,
        }
    }
    count
}

fn extract_identifier(s: &str) -> String {
    s.chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

fn extract_parens(s: &str) -> String {
    if let Some(open) = s.find('(') {
        let mut depth = 0;
        let mut end = None;
        for (i, ch) in s[open..].char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(open + i);
                        break;
                    }
                }
                _ => {}
            }
        }
        match end {
            Some(e) => s[open + 1..e].to_string(),
            None => s[open + 1..].trim_end_matches(':').trim().to_string(),
        }
    } else {
        String::new()
    }
}

fn extract_return_type(s: &str) -> String {
    // Look for -> after closing paren
    if let Some(arrow) = s.find("->") {
        let after = s[arrow + 2..].trim();
        // Take until `:` (the block start)
        if let Some(colon) = after.find(':') {
            after[..colon].trim().to_string()
        } else {
            after.to_string()
        }
    } else {
        String::new()
    }
}

fn extract_bases(s: &str) -> Vec<String> {
    if s.find('(').is_some() {
        let inner = extract_parens(s);
        if inner.is_empty() {
            return Vec::new();
        }
        inner.split(',').map(|b| b.trim().to_string()).collect()
    } else {
        Vec::new()
    }
}

fn extract_decorator_name(trimmed: &str) -> String {
    let after_at = &trimmed[1..]; // skip '@'
                                  // Take until '(' or end of line
    let name: String = after_at
        .chars()
        .take_while(|c| *c != '(' && *c != '\n' && *c != ' ')
        .collect();
    name.trim().to_string()
}

fn parse_name_list(s: &str) -> Vec<String> {
    s.split(',')
        .map(|n| {
            let name = n.trim();
            // Handle `foo as bar` — keep the alias part
            if let Some(idx) = name.find(" as ") {
                name[..idx].trim().to_string()
            } else {
                name.to_string()
            }
        })
        .filter(|n| !n.is_empty())
        .collect()
}

fn parse_except_header(trimmed: &str) -> (String, String) {
    // except ExceptionType as name:
    // except ExceptionType:
    // except:
    let content = trimmed
        .strip_prefix("except*")
        .or_else(|| trimmed.strip_prefix("except"))
        .unwrap_or("")
        .trim()
        .trim_end_matches(':')
        .trim();

    if content.is_empty() {
        return (String::new(), String::new());
    }

    if let Some(idx) = content.find(" as ") {
        let exc_type = content[..idx].trim().to_string();
        let name = content[idx + 4..].trim().to_string();
        (exc_type, name)
    } else {
        (content.to_string(), String::new())
    }
}

fn try_parse_assign(trimmed: &str, line: usize) -> Option<PyNode> {
    // Annotated assignment: x: int = 5  or  x: int
    // Simple assignment: x = 5
    // Skip comparisons: if x == 5, x != 5, x >= 5, x <= 5
    // Skip augmented: x += 1, x -= 1, etc.

    // Check for `:` before `=` (type hint)
    if let Some(colon_idx) = trimmed.find(':') {
        // Make sure this isn't a dict literal or slice
        let before_colon = &trimmed[..colon_idx];
        if is_simple_target(before_colon) {
            let target = before_colon.trim().to_string();
            return Some(PyNode::Assign {
                target,
                has_type_hint: true,
                line,
            });
        }
    }

    // Simple assignment: look for `=` that's not `==`, `!=`, `>=`, `<=`, `+=`, `-=`, etc.
    if let Some(eq_idx) = trimmed.find('=') {
        if eq_idx == 0 {
            return None;
        }
        let before = trimmed.as_bytes()[eq_idx - 1];
        let after = trimmed.as_bytes().get(eq_idx + 1).copied().unwrap_or(0);
        if before == b'!'
            || before == b'<'
            || before == b'>'
            || before == b'+'
            || before == b'-'
            || before == b'*'
            || before == b'/'
            || before == b'%'
            || before == b'&'
            || before == b'|'
            || before == b'^'
            || after == b'='
        {
            return None;
        }
        let target_str = trimmed[..eq_idx].trim();
        if is_simple_target(target_str) {
            return Some(PyNode::Assign {
                target: target_str.to_string(),
                has_type_hint: false,
                line,
            });
        }
    }

    None
}

fn is_simple_target(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
    // Simple identifier, possibly with dots (self.x) or tuple unpacking (a, b)
    trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == ',' || c == ' ')
}

fn starts_triple_quote(trimmed: &str) -> bool {
    trimmed.starts_with("\"\"\"")
        || trimmed.starts_with("'''")
        || trimmed.starts_with("r\"\"\"")
        || trimmed.starts_with("r'''")
        || trimmed.starts_with("f\"\"\"")
        || trimmed.starts_with("f'''")
        || trimmed.starts_with("b\"\"\"")
        || trimmed.starts_with("b'''")
}

fn contains_triple_quote(s: &str) -> bool {
    s.contains("\"\"\"") || s.contains("'''")
}

fn closes_triple_quote_same_line(trimmed: &str) -> bool {
    // Check if a triple-quote that opens also closes on the same line.
    // e.g. """docstring""" or '''text'''
    for prefix in &[
        "\"\"\"", "'''", "r\"\"\"", "r'''", "f\"\"\"", "f'''", "b\"\"\"", "b'''",
    ] {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            let closer = &prefix[prefix.len() - 3..]; // last 3 chars
            if rest.contains(closer) {
                return true;
            }
        }
    }
    false
}

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

    #[test]
    fn test_simple_function() {
        let source = "def foo(x, y):\n    return x + y\n";
        let result = scan_python(source).unwrap();
        if let Expression::Operation { name, args, .. } = &result {
            assert_eq!(name, "PyModule");
            assert_eq!(args.len(), 3);
            assert_eq!(cons_len(&args[0]), 1);
        } else {
            panic!("Expected Operation");
        }
    }

    #[test]
    fn test_class_with_methods() {
        let source = "\
class Foo(Bar):
    def __init__(self):
        self.x = 1

    def method(self) -> str:
        return 'hello'
";
        let result = scan_python(source).unwrap();
        if let Expression::Operation { name, args, .. } = &result {
            assert_eq!(name, "PyModule");
            assert_eq!(cons_len(&args[0]), 1);
        }
    }

    #[test]
    fn test_imports() {
        let source = "\
import os
import sys
from pathlib import Path
from typing import *
";
        let result = scan_python(source).unwrap();
        if let Expression::Operation { args, .. } = &result {
            assert_eq!(cons_len(&args[0]), 4);
        }
    }

    #[test]
    fn test_try_except() {
        let source = "\
try:
    x = 1
except ValueError as e:
    pass
except:
    pass
finally:
    cleanup()
";
        let result = scan_python(source).unwrap();
        if let Expression::Operation { args, .. } = &result {
            assert_eq!(cons_len(&args[0]), 1);
        }
    }

    #[test]
    fn test_decorators() {
        let source = "\
@staticmethod
@override
def my_method(cls):
    pass
";
        let result = scan_python(source).unwrap();
        if let Expression::Operation { args, .. } = &result {
            assert_eq!(cons_len(&args[0]), 1);
        }
    }

    #[test]
    fn test_leading_spaces() {
        assert_eq!(leading_spaces("    hello"), 4);
        assert_eq!(leading_spaces("hello"), 0);
        assert_eq!(leading_spaces("\thello"), 4);
        assert_eq!(leading_spaces("  \thello"), 6);
    }

    #[test]
    fn test_extract_identifier() {
        assert_eq!(extract_identifier("foo(x)"), "foo");
        assert_eq!(extract_identifier("__init__(self)"), "__init__");
        assert_eq!(extract_identifier("MyClass(Base):"), "MyClass");
    }

    #[test]
    fn test_extract_return_type() {
        assert_eq!(extract_return_type("foo(x) -> str:"), "str");
        assert_eq!(extract_return_type("foo(x):"), "");
        assert_eq!(
            extract_return_type("foo(x, y) -> Optional[int]:"),
            "Optional[int]"
        );
    }

    #[test]
    fn test_parse_except_header() {
        assert_eq!(
            parse_except_header("except ValueError as e:"),
            ("ValueError".to_string(), "e".to_string())
        );
        assert_eq!(
            parse_except_header("except:"),
            (String::new(), String::new())
        );
        assert_eq!(
            parse_except_header("except KeyError:"),
            ("KeyError".to_string(), String::new())
        );
    }
}
