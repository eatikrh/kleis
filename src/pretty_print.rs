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
use crate::kleis_ast::DataDef;

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

        // Always use single-line format for valid Kleis syntax
        format!("define {}({}) = {}", name, params, body)
    }

    /// Format an expression to Kleis source code
    pub fn format_expression(&self, expr: &Expression) -> String {
        match expr {
            Expression::Const(s) => s.clone(),

            Expression::Object(name) => name.clone(),

            Expression::Operation { name, args } => self.format_operation(name, args),

            Expression::Placeholder { hint, .. } => {
                format!("□{}", hint)
            }

            Expression::Match { scrutinee, cases } => self.format_match(scrutinee, cases),

            Expression::List(items) => {
                let formatted: Vec<String> =
                    items.iter().map(|e| self.format_expression(e)).collect();
                format!("[{}]", formatted.join(", "))
            }

            Expression::Quantifier {
                quantifier,
                variables,
                where_clause,
                body,
            } => self.format_quantifier(quantifier, variables, where_clause.as_deref(), body),

            Expression::Conditional {
                condition,
                then_branch,
                else_branch,
            } => self.format_conditional(condition, then_branch, else_branch),

            Expression::Let { name, value, body } => self.format_let(name, value, body),
        }
    }

    /// Format an operation (handles infix operators specially)
    fn format_operation(&self, name: &str, args: &[Expression]) -> String {
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
            "implies" => Some("->"),
            "compose" => Some("∘"),
            "bullet" | "op" => Some("*"),
            _ => None,
        };

        if let Some(op) = infix_op {
            if args.len() == 2 {
                let left = self.format_expression(&args[0]);
                let right = self.format_expression(&args[1]);
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
                let arg = self.format_expression(&args[0]);
                return format!("-{}", self.maybe_paren(&args[0], &arg));
            }
            "not" | "logical_not" if args.len() == 1 => {
                let arg = self.format_expression(&args[0]);
                return format!("¬{}", self.maybe_paren(&args[0], &arg));
            }
            _ => {}
        }

        // Generic function call
        if args.is_empty() {
            name.to_string()
        } else {
            let formatted_args: Vec<String> =
                args.iter().map(|a| self.format_expression(a)).collect();
            format!("{}({})", name, formatted_args.join(", "))
        }
    }

    /// Format a match expression
    /// Note: Always use single-line format for valid round-trip parsing.
    fn format_match(&self, scrutinee: &Expression, cases: &[MatchCase]) -> String {
        let scrutinee_str = self.format_expression(scrutinee);

        let cases_str: Vec<String> = cases
            .iter()
            .map(|c| {
                let pattern = self.format_pattern(&c.pattern);
                let body = self.format_expression(&c.body);
                format!("{} => {}", pattern, body)
            })
            .collect();

        // Always use single-line format with | separator for valid Kleis syntax
        format!("match {} {{ {} }}", scrutinee_str, cases_str.join(" | "))
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

    /// Format a quantifier
    fn format_quantifier(
        &self,
        quantifier: &QuantifierKind,
        variables: &[QuantifiedVar],
        where_clause: Option<&Expression>,
        body: &Expression,
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
            format!(" where {}", self.format_expression(cond))
        } else {
            String::new()
        };

        let body_str = self.format_expression(body);

        format!("{}{}. {}", vars_part, where_part, body_str)
    }

    /// Format a conditional
    /// Note: Kleis parser requires if/then/else on a single logical line,
    /// so we always use single-line format for valid round-trip parsing.
    fn format_conditional(
        &self,
        condition: &Expression,
        then_branch: &Expression,
        else_branch: &Expression,
    ) -> String {
        let cond = self.format_expression(condition);
        let then_str = self.format_expression(then_branch);
        let else_str = self.format_expression(else_branch);

        // Always use single-line format for valid Kleis syntax
        format!("if {} then {} else {}", cond, then_str, else_str)
    }

    /// Format a let binding
    fn format_let(&self, name: &str, value: &Expression, body: &Expression) -> String {
        let value_str = self.format_expression(value);
        let body_str = self.format_expression(body);

        format!("let {} = {} in {}", name, value_str, body_str)
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
                    "{} -> {}",
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
