//! Pretty-Printer for Kleis Expressions
//!
//! Converts AST back to human-readable Kleis source code.
//! Used for `:export` command in REPL and debugging.
//!
//! ## Example
//!
//! ```ignore
//! use kleis::pretty_print::PrettyPrinter;
//!
//! let expr = Expression::Operation {
//!     name: "plus".to_string(),
//!     args: vec![Expression::Object("x".into()), Expression::Const("1".into())],
//! };
//!
//! let pp = PrettyPrinter::new();
//! println!("{}", pp.format_expression(&expr));
//! // Output: x + 1
//! ```

use crate::ast::{Expression, MatchCase, Pattern, QuantifiedVar, QuantifierKind};
use crate::evaluator::Closure;
use crate::kleis_ast::{DataDef, StructureDef, StructureMember};

/// Pretty-printer configuration
pub struct PrettyPrinter {
    /// Indentation string (default: 4 spaces)
    indent: String,
}

impl Default for PrettyPrinter {
    fn default() -> Self {
        Self::new()
    }
}

impl PrettyPrinter {
    /// Create a new pretty-printer with default settings
    pub fn new() -> Self {
        PrettyPrinter {
            indent: "    ".to_string(),
        }
    }

    /// Create a pretty-printer with custom indentation
    pub fn with_indent(indent: &str) -> Self {
        PrettyPrinter {
            indent: indent.to_string(),
        }
    }

    /// Format a function definition
    /// Note: Always use single-line format for valid round-trip parsing.
    pub fn format_function(&self, name: &str, closure: &Closure) -> String {
        let params = closure.params.join(", ");
        let body = self.format_expression(&closure.body);

        // Use multi-line format if body contains newlines
        if body.contains('\n') {
            // Indent the body lines
            let indented_body = body
                .lines()
                .enumerate()
                .map(|(i, line)| {
                    if i == 0 {
                        line.to_string()
                    } else {
                        format!("{}{}", self.indent, line)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "define {}({}) =\n{}{}",
                name, params, self.indent, indented_body
            )
        } else {
            format!("define {}({}) = {}", name, params, body)
        }
    }

    /// Format an expression to Kleis source code
    pub fn format_expression(&self, expr: &Expression) -> String {
        self.format_at_depth(expr, 0)
    }

    /// Format an expression at a given indentation depth
    /// This is the core recursive method that tracks nesting level
    fn format_at_depth(&self, expr: &Expression, depth: usize) -> String {
        match expr {
            Expression::Const(s) => s.clone(),

            Expression::Object(name) => name.clone(),

            Expression::Operation { name, args } => {
                self.format_operation_at_depth(name, args, depth)
            }

            Expression::Placeholder { hint, .. } => {
                format!("□{}", hint)
            }

            Expression::Match { scrutinee, cases } => {
                self.format_match_at_depth(scrutinee, cases, depth)
            }

            Expression::List(items) => {
                let formatted: Vec<String> = items
                    .iter()
                    .map(|e| self.format_at_depth(e, depth))
                    .collect();
                format!("[{}]", formatted.join(", "))
            }

            Expression::Quantifier {
                quantifier,
                variables,
                where_clause,
                body,
            } => self.format_quantifier_at_depth(
                quantifier,
                variables,
                where_clause.as_deref(),
                body,
                depth,
            ),

            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => self.format_conditional_at_depth(condition, then_branch, else_branch, depth),

            Expression::Let {
                name,
                type_annotation,
                value,
                body,
            } => self.format_let_at_depth(name, type_annotation.as_deref(), value, body, depth),
        }
    }

    /// Format an operation (handles infix operators specially)
    fn format_operation_at_depth(&self, name: &str, args: &[Expression], depth: usize) -> String {
        // Infix binary operators
        // Note: Use ASCII versions for comparison operators for valid round-trip parsing
        let infix_op = match name {
            "plus" | "add" => Some("+"),
            "minus" | "subtract" => Some("-"),
            "times" | "multiply" => Some("*"),
            "divide" | "div" => Some("/"),
            "power" | "pow" => Some("^"),
            "equals" | "eq" => Some("="),
            "not_equals" | "neq" => Some("!="),
            "less_than" | "lt" => Some("<"),
            "greater_than" | "gt" => Some(">"),
            "leq" => Some("<="),
            "geq" => Some(">="),
            "and" | "logical_and" => Some("and"),
            "or" | "logical_or" => Some("or"),
            "implies" => Some("⟹"),
            "compose" => Some("∘"),
            "bullet" | "op" => Some("*"),
            _ => None,
        };

        if let Some(op) = infix_op {
            if args.len() == 2 {
                let left = self.format_at_depth(&args[0], depth);
                let right = self.format_at_depth(&args[1], depth);
                return format!(
                    "{} {} {}",
                    self.maybe_paren(&args[0], &left),
                    op,
                    self.maybe_paren(&args[1], &right)
                );
            }
        }

        // Prefix unary operators
        match name {
            "neg" | "negate" if args.len() == 1 => {
                let arg = self.format_at_depth(&args[0], depth);
                return format!("-{}", self.maybe_paren(&args[0], &arg));
            }
            "not" | "logical_not" if args.len() == 1 => {
                let arg = self.format_at_depth(&args[0], depth);
                return format!("¬{}", self.maybe_paren(&args[0], &arg));
            }
            _ => {}
        }

        // Generic function call
        if args.is_empty() {
            name.to_string()
        } else {
            let formatted_args: Vec<String> = args
                .iter()
                .map(|a| self.format_at_depth(a, depth))
                .collect();
            format!("{}({})", name, formatted_args.join(", "))
        }
    }

    /// Format a match expression with proper indentation
    fn format_match_at_depth(
        &self,
        scrutinee: &Expression,
        cases: &[MatchCase],
        depth: usize,
    ) -> String {
        let scrutinee_str = self.format_at_depth(scrutinee, depth);

        // For many cases, use multi-line format
        if cases.len() > 3 {
            let indent = self.indent.repeat(depth + 1);
            let cases_str: Vec<String> = cases
                .iter()
                .map(|c| {
                    let pattern = self.format_pattern(&c.pattern);
                    let body = self.format_at_depth(&c.body, depth + 1);
                    format!("{}  {} => {}", indent, pattern, body)
                })
                .collect();
            format!(
                "match {} {{\n{}\n{}}}",
                scrutinee_str,
                cases_str.join("\n"),
                self.indent.repeat(depth)
            )
        } else {
            // Short match: single line
            let cases_str: Vec<String> = cases
                .iter()
                .map(|c| {
                    let pattern = self.format_pattern(&c.pattern);
                    let body = self.format_at_depth(&c.body, depth);
                    format!("{} => {}", pattern, body)
                })
                .collect();
            format!("match {} {{ {} }}", scrutinee_str, cases_str.join(" | "))
        }
    }

    /// Format a pattern
    fn format_pattern(&self, pattern: &Pattern) -> String {
        Self::format_pattern_inner(pattern)
    }

    /// Inner pattern formatting (static to avoid clippy warning about self in recursion)
    fn format_pattern_inner(pattern: &Pattern) -> String {
        match pattern {
            Pattern::Wildcard => "_".to_string(),
            Pattern::Variable(name) => name.clone(),
            Pattern::Constant(val) => val.clone(),
            Pattern::Constructor { name, args } => {
                if args.is_empty() {
                    name.clone()
                } else {
                    let formatted_args: Vec<String> =
                        args.iter().map(Self::format_pattern_inner).collect();
                    format!("{}({})", name, formatted_args.join(", "))
                }
            }
        }
    }

    /// Format a quantifier with depth tracking
    fn format_quantifier_at_depth(
        &self,
        quantifier: &QuantifierKind,
        variables: &[QuantifiedVar],
        where_clause: Option<&Expression>,
        body: &Expression,
        depth: usize,
    ) -> String {
        let quant_sym = match quantifier {
            QuantifierKind::ForAll => "∀",
            QuantifierKind::Exists => "∃",
        };

        let vars_str: Vec<String> = variables
            .iter()
            .map(|v| {
                if let Some(ty) = &v.type_annotation {
                    format!("{} : {}", v.name, ty)
                } else {
                    v.name.clone()
                }
            })
            .collect();

        let vars_part = format!("{}({})", quant_sym, vars_str.join(", "));

        let where_part = if let Some(cond) = where_clause {
            format!(" where {}", self.format_at_depth(cond, depth))
        } else {
            String::new()
        };

        let body_str = self.format_at_depth(body, depth);

        format!("{}{}. {}", vars_part, where_part, body_str)
    }

    /// Format a conditional with hierarchical indentation
    /// The AST structure drives the formatting:
    /// - Simple conditionals stay on one line
    /// - Chained if-else uses aligned format
    /// - Deeply nested expressions get properly indented
    fn format_conditional_at_depth(
        &self,
        condition: &Expression,
        then_branch: &Expression,
        else_branch: &Expression,
        depth: usize,
    ) -> String {
        let cond = self.format_at_depth(condition, depth);
        let then_str = self.format_at_depth(then_branch, depth);

        // Check if this is a chained if-else (else branch is another conditional)
        let is_chained_else = matches!(else_branch, Expression::Conditional { .. });

        // Format else branch - chained conditionals stay at same depth for alignment
        let else_str = self.format_at_depth(else_branch, depth);

        // Check if we should use multi-line format
        let total_len = cond.len() + then_str.len() + else_str.len();
        let indent_str = self.indent.repeat(depth);

        if is_chained_else {
            // Chained if-else: put each branch on its own line
            // Format: if cond then result\nelse if cond2 then result2\nelse result3
            format!(
                "if {} then {}\n{}else {}",
                cond, then_str, indent_str, else_str
            )
        } else if total_len > 60 {
            // Long expression: use multi-line with proper indentation
            format!(
                "if {} then {}\n{}else {}",
                cond, then_str, indent_str, else_str
            )
        } else {
            // Short expression: single line
            format!("if {} then {} else {}", cond, then_str, else_str)
        }
    }

    /// Format a let binding with depth tracking
    fn format_let_at_depth(
        &self,
        name: &str,
        type_annotation: Option<&str>,
        value: &Expression,
        body: &Expression,
        depth: usize,
    ) -> String {
        let value_str = self.format_at_depth(value, depth);
        let body_str = self.format_at_depth(body, depth);

        // Build the let binding with optional type annotation
        let binding = match type_annotation {
            Some(ty) => format!("{} : {}", name, ty),
            None => name.to_string(),
        };

        // For complex bodies, use multi-line format
        if body_str.contains('\n') {
            let indent_str = self.indent.repeat(depth);
            format!(
                "let {} = {} in\n{}{}",
                binding, value_str, indent_str, body_str
            )
        } else {
            format!("let {} = {} in {}", binding, value_str, body_str)
        }
    }

    /// Add parentheses around complex expressions when needed
    fn maybe_paren(&self, expr: &Expression, formatted: &str) -> String {
        match expr {
            Expression::Operation { name, args } if args.len() == 2 => {
                // Check if it's an infix operation that might need parens
                let needs_parens = matches!(
                    name.as_str(),
                    "plus" | "minus" | "times" | "divide" | "and" | "or"
                );
                if needs_parens {
                    format!("({})", formatted)
                } else {
                    formatted.to_string()
                }
            }
            Expression::Conditional { .. } => format!("({})", formatted),
            _ => formatted.to_string(),
        }
    }

    /// Indent all lines of a string
    /// Reserved for future multi-line formatting when parser supports it
    #[allow(dead_code)]
    fn indent_lines(&self, s: &str, levels: usize) -> String {
        let prefix: String = self.indent.repeat(levels);
        s.lines()
            .map(|line| format!("{}{}", prefix, line))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format a data type definition
    /// Example: data Bool = True | False
    /// Example: data Option(T) = None | Some(T)
    pub fn format_data_def(&self, data_def: &DataDef) -> String {
        let name = &data_def.name;

        // Format type parameters if any
        let params = if data_def.type_params.is_empty() {
            String::new()
        } else {
            let param_strs: Vec<String> = data_def
                .type_params
                .iter()
                .map(|p| {
                    if let Some(ref kind) = p.kind {
                        format!("{}: {}", p.name, kind)
                    } else {
                        p.name.clone()
                    }
                })
                .collect();
            format!("({})", param_strs.join(", "))
        };

        // Format variants
        let variants: Vec<String> = data_def
            .variants
            .iter()
            .map(|v| {
                if v.fields.is_empty() {
                    v.name.clone()
                } else {
                    let fields: Vec<String> = v
                        .fields
                        .iter()
                        .map(|f| Self::format_type_expr(&f.type_expr))
                        .collect();
                    format!("{}({})", v.name, fields.join(", "))
                }
            })
            .collect();

        format!("data {}{} = {}", name, params, variants.join(" | "))
    }

    /// Format a type expression
    fn format_type_expr(type_expr: &crate::kleis_ast::TypeExpr) -> String {
        use crate::kleis_ast::TypeExpr;
        match type_expr {
            TypeExpr::Named(name) => name.clone(),
            TypeExpr::Parametric(name, args) => {
                let args_str: Vec<String> = args.iter().map(Self::format_type_expr).collect();
                format!("{}({})", name, args_str.join(", "))
            }
            TypeExpr::Function(input, output) => {
                format!(
                    "{} → {}",
                    Self::format_type_expr(input),
                    Self::format_type_expr(output)
                )
            }
            TypeExpr::Product(types) => {
                let types_str: Vec<String> = types.iter().map(Self::format_type_expr).collect();
                format!("({})", types_str.join(", "))
            }
            TypeExpr::Var(name) => name.clone(),
            TypeExpr::ForAll { vars, body } => {
                let vars_str: Vec<String> = vars
                    .iter()
                    .map(|(name, ty)| format!("{}: {}", name, Self::format_type_expr(ty)))
                    .collect();
                format!(
                    "∀({}). {}",
                    vars_str.join(", "),
                    Self::format_type_expr(body)
                )
            }
        }
    }

    /// Format a structure definition
    /// Example:
    /// ```kleis
    /// structure Monoid(M) {
    ///     element identity : M
    ///     operation op : M -> M -> M
    ///     axiom left_identity : ∀(x : M). op(identity, x) = x
    /// }
    /// ```
    pub fn format_structure(&self, structure: &StructureDef) -> String {
        let name = &structure.name;

        // Format type parameters if any
        let params = if structure.type_params.is_empty() {
            String::new()
        } else {
            let param_strs: Vec<String> = structure
                .type_params
                .iter()
                .map(|p| {
                    if let Some(ref kind) = p.kind {
                        format!("{}: {}", p.name, kind)
                    } else {
                        p.name.clone()
                    }
                })
                .collect();
            format!("({})", param_strs.join(", "))
        };

        // Format extends clause if any
        let extends = if let Some(ref parent) = structure.extends_clause {
            format!(" extends {}", Self::format_type_expr(parent))
        } else {
            String::new()
        };

        // Format over clause if any
        let over = if let Some(ref field_type) = structure.over_clause {
            format!(" over {}", Self::format_type_expr(field_type))
        } else {
            String::new()
        };

        // Format members
        let members_str = self.format_structure_members(&structure.members, 1);

        format!(
            "structure {}{}{}{} {{\n{}\n}}",
            name, params, extends, over, members_str
        )
    }

    /// Format structure members with proper indentation
    fn format_structure_members(&self, members: &[StructureMember], level: usize) -> String {
        let indent = self.indent.repeat(level);
        let lines: Vec<String> = members
            .iter()
            .map(|m| format!("{}{}", indent, self.format_structure_member(m, level)))
            .collect();
        lines.join("\n")
    }

    /// Format a single structure member
    fn format_structure_member(&self, member: &StructureMember, level: usize) -> String {
        match member {
            StructureMember::Field { name, type_expr } => {
                format!("element {} : {}", name, Self::format_type_expr(type_expr))
            }
            StructureMember::Operation {
                name,
                type_signature,
            } => {
                format!(
                    "operation {} : {}",
                    name,
                    Self::format_type_expr(type_signature)
                )
            }
            StructureMember::Axiom { name, proposition } => {
                format!("axiom {} : {}", name, self.format_expression(proposition))
            }
            StructureMember::NestedStructure {
                name,
                structure_type,
                members,
            } => {
                let nested_members = self.format_structure_members(members, level + 1);
                format!(
                    "structure {} : {} {{\n{}\n{}}}",
                    name,
                    Self::format_type_expr(structure_type),
                    nested_members,
                    self.indent.repeat(level)
                )
            }
            StructureMember::FunctionDef(func_def) => {
                let params = func_def.params.join(", ");
                let body = self.format_expression(&func_def.body);
                format!("define {}({}) = {}", func_def.name, params, body)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_const() {
        let pp = PrettyPrinter::new();
        let expr = Expression::Const("42".to_string());
        assert_eq!(pp.format_expression(&expr), "42");
    }

    #[test]
    fn test_format_object() {
        let pp = PrettyPrinter::new();
        let expr = Expression::Object("x".to_string());
        assert_eq!(pp.format_expression(&expr), "x");
    }

    #[test]
    fn test_format_infix_plus() {
        let pp = PrettyPrinter::new();
        let expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Object("x".to_string()),
                Expression::Const("1".to_string()),
            ],
        };
        assert_eq!(pp.format_expression(&expr), "x + 1");
    }

    #[test]
    fn test_format_function_call() {
        let pp = PrettyPrinter::new();
        let expr = Expression::Operation {
            name: "sin".to_string(),
            args: vec![Expression::Object("x".to_string())],
        };
        assert_eq!(pp.format_expression(&expr), "sin(x)");
    }

    #[test]
    fn test_format_conditional() {
        let pp = PrettyPrinter::new();
        let expr = Expression::Conditional {
            condition: Box::new(Expression::Operation {
                name: "gt".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Const("0".to_string()),
                ],
            }),
            then_branch: Box::new(Expression::Object("x".to_string())),
            else_branch: Box::new(Expression::Operation {
                name: "neg".to_string(),
                args: vec![Expression::Object("x".to_string())],
            }),
        };
        assert_eq!(pp.format_expression(&expr), "if x > 0 then x else -x");
    }

    #[test]
    fn test_format_let() {
        let pp = PrettyPrinter::new();
        let expr = Expression::Let {
            name: "y".to_string(),
            type_annotation: None,
            value: Box::new(Expression::Const("5".to_string())),
            body: Box::new(Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Object("y".to_string()),
                    Expression::Object("y".to_string()),
                ],
            }),
        };
        assert_eq!(pp.format_expression(&expr), "let y = 5 in y + y");
    }

    #[test]
    fn test_format_let_with_type() {
        let pp = PrettyPrinter::new();
        let expr = Expression::Let {
            name: "x".to_string(),
            type_annotation: Some("ℝ".to_string()),
            value: Box::new(Expression::Const("5".to_string())),
            body: Box::new(Expression::Object("x".to_string())),
        };
        assert_eq!(pp.format_expression(&expr), "let x : ℝ = 5 in x");
    }

    #[test]
    fn test_format_match() {
        let pp = PrettyPrinter::new();
        let expr = Expression::Match {
            scrutinee: Box::new(Expression::Object("x".to_string())),
            cases: vec![
                MatchCase {
                    pattern: Pattern::Constant("0".to_string()),
                    body: Expression::Const("1".to_string()),
                },
                MatchCase {
                    pattern: Pattern::Wildcard,
                    body: Expression::Const("0".to_string()),
                },
            ],
        };
        assert_eq!(pp.format_expression(&expr), "match x { 0 => 1 | _ => 0 }");
    }

    #[test]
    fn test_format_function() {
        let pp = PrettyPrinter::new();
        let closure = Closure {
            params: vec!["x".to_string()],
            body: Expression::Operation {
                name: "plus".to_string(),
                args: vec![
                    Expression::Object("x".to_string()),
                    Expression::Object("x".to_string()),
                ],
            },
            env: std::collections::HashMap::new(),
        };
        assert_eq!(
            pp.format_function("double", &closure),
            "define double(x) = x + x"
        );
    }
}
