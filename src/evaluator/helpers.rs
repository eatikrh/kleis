use crate::ast::Expression;

use super::{eval_numeric, Evaluator};

impl Evaluator {
    pub(crate) fn extract_bool(&self, expr: &Expression) -> Result<bool, String> {
        let evaluated = self.eval_concrete(expr)?;
        let s = match &evaluated {
            Expression::Const(s) | Expression::Object(s) => s.to_lowercase(),
            _ => return Err(format!("Expected boolean, got: {:?}", evaluated)),
        };
        match s.as_str() {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            _ => Err(format!("Expected boolean, got: {}", s)),
        }
    }

    pub(crate) fn extract_f64_matrix(&self, expr: &Expression) -> Result<Vec<Vec<f64>>, String> {
        let evaluated = self.eval_concrete(expr)?;

        let extract_row = |row: &Expression| -> Result<Vec<f64>, String> {
            if let Expression::List(elems) = row {
                let mut row_data = Vec::new();
                for elem in elems {
                    if let Some(n) = self.as_number(elem) {
                        row_data.push(n);
                    } else {
                        return Err(format!("Expected number in matrix, got: {:?}", elem));
                    }
                }
                return Ok(row_data);
            }
            if let Some(elems) = self.extract_flat_list(row) {
                let mut row_data = Vec::new();
                for elem in elems {
                    if let Some(n) = self.as_number(&elem) {
                        row_data.push(n);
                    } else {
                        return Err(format!("Expected number in matrix, got: {:?}", elem));
                    }
                }
                return Ok(row_data);
            }
            Err(format!("Expected row to be a list, got: {:?}", row))
        };

        if let Expression::List(rows) = &evaluated {
            let mut matrix = Vec::new();
            for row in rows {
                matrix.push(extract_row(row)?);
            }
            return Ok(matrix);
        }

        if let Some(rows) = self.extract_flat_list(&evaluated) {
            let mut matrix = Vec::new();
            for row in rows {
                matrix.push(extract_row(&row)?);
            }
            return Ok(matrix);
        }

        Err(format!(
            "Expected matrix (list of lists), got: {:?}",
            evaluated
        ))
    }

    pub(crate) fn extract_string(&self, expr: &Expression) -> Result<String, String> {
        match expr {
            Expression::Const(s) => {
                let s = s.trim_matches('"');
                Ok(s.to_string())
            }
            Expression::String(s) => Ok(s.clone()),
            Expression::Object(s) => Ok(s.clone()),
            _ => Err(format!("Expected string, got: {:?}", expr)),
        }
    }

    pub(crate) fn extract_string_list(&self, expr: &Expression) -> Result<Vec<String>, String> {
        let evaluated = self.eval_concrete(expr)?;
        if let Expression::List(elements) = evaluated {
            elements.iter().map(|e| self.extract_string(e)).collect()
        } else {
            Err(format!("Expected list of strings, got: {:?}", expr))
        }
    }

    pub(crate) fn extract_f64_list_from_diagram_option(
        &self,
        expr: &Expression,
    ) -> Result<Vec<f64>, String> {
        let evaluated = self.eval_concrete(expr)?;
        if let Expression::List(elements) = evaluated {
            elements.iter().map(|e| self.extract_f64(e)).collect()
        } else {
            Err(format!("Expected list of numbers, got: {:?}", expr))
        }
    }

    pub(crate) fn extract_number(&self, expr: &Expression) -> Result<f64, String> {
        let evaluated = self.eval_concrete(expr)?;
        if let Some(n) = self.as_number(&evaluated) {
            Ok(n)
        } else if let Expression::Const(s) = &evaluated {
            s.parse::<f64>()
                .map_err(|_| format!("Expected number, got: {}", s))
        } else {
            Err(format!("Expected number, got: {:?}", evaluated))
        }
    }

    pub(crate) fn extract_f64(&self, expr: &Expression) -> Result<f64, String> {
        self.extract_number(expr)
    }

    pub(crate) fn extract_flat_list(&self, expr: &Expression) -> Option<Vec<Expression>> {
        match expr {
            Expression::Operation { name, args, .. } if name == "list" || name == "List" => {
                Some(args.clone())
            }
            Expression::Operation { name, args, .. } if name == "Cons" => {
                if args.len() == 2 {
                    let head = args[0].clone();
                    if let Some(mut tail) = self.extract_flat_list(&args[1]) {
                        let mut result = vec![head];
                        result.append(&mut tail);
                        return Some(result);
                    }
                }
                None
            }
            Expression::Object(s) if s == "Nil" => Some(vec![]),
            _ => None,
        }
    }

    pub(crate) fn extract_number_list_v2(&self, expr: &Expression) -> Result<Vec<f64>, String> {
        let evaluated = self.eval_concrete(expr)?;

        if let Expression::List(elements) = &evaluated {
            let mut result = Vec::new();
            for elem in elements {
                if let Some(n) = self.as_number(elem) {
                    result.push(n);
                } else {
                    return Err(format!("Expected number in list, got: {:?}", elem));
                }
            }
            return Ok(result);
        }

        if let Expression::Operation { name, args, .. } = &evaluated {
            if name == "List" || name == "list" {
                let mut result = Vec::new();
                for arg in args {
                    if let Some(n) = self.as_number(arg) {
                        result.push(n);
                    } else {
                        return Err(format!("Expected number in list, got: {:?}", arg));
                    }
                }
                return Ok(result);
            }
        }

        if let Some(elems) = self.extract_flat_list(&evaluated) {
            let mut result = Vec::new();
            for elem in elems {
                if let Some(n) = self.as_number(&elem) {
                    result.push(n);
                } else {
                    return Err(format!("Expected number in list, got: {:?}", elem));
                }
            }
            return Ok(result);
        }

        Err(format!("Expected list of numbers, got: {:?}", evaluated))
    }

    pub(crate) fn builtin_comparison<F>(
        &self,
        args: &[Expression],
        op: F,
    ) -> Result<Option<Expression>, String>
    where
        F: Fn(f64, f64) -> bool,
    {
        if args.len() != 2 {
            return Ok(None);
        }
        if let (Some(a), Some(b)) = (self.as_number(&args[0]), self.as_number(&args[1])) {
            Ok(Some(Expression::Object(
                if op(a, b) { "true" } else { "false" }.to_string(),
            )))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn as_number(&self, expr: &Expression) -> Option<f64> {
        eval_numeric(expr)
    }

    pub(crate) fn as_integer(&self, expr: &Expression) -> Option<i64> {
        match expr {
            Expression::Const(s) => s.parse().ok(),
            _ => None,
        }
    }

    pub(crate) const ZERO_TOLERANCE: f64 = 1e-15;

    pub(crate) fn format_number(v: f64) -> String {
        if v.abs() < Self::ZERO_TOLERANCE {
            "0".to_string()
        } else {
            format!("{}", v)
        }
    }

    pub(crate) fn const_from_f64(v: f64) -> Expression {
        Expression::Const(Self::format_number(v))
    }

    pub(crate) fn pretty_print_value(&self, expr: &Expression) -> String {
        match expr {
            Expression::Const(s) => s.clone(),
            Expression::String(s) => format!("\"{}\"", s),
            Expression::Object(s) => s.clone(),
            Expression::List(elements) => {
                let is_matrix = elements.iter().all(|e| matches!(e, Expression::List(_)));
                if is_matrix && !elements.is_empty() {
                    self.pretty_print_matrix(elements)
                } else {
                    let items: Vec<String> = elements
                        .iter()
                        .map(|e| self.pretty_print_value(e))
                        .collect();
                    format!("[{}]", items.join(", "))
                }
            }
            Expression::Operation { name, args, .. } => {
                if name == "typst_raw" && args.len() == 1 {
                    if let Ok(
                        Expression::String(s) | Expression::Const(s) | Expression::Object(s),
                    ) = self.eval_concrete(&args[0])
                    {
                        return s;
                    }
                    if let Ok(s) = self.extract_string(&args[0]) {
                        return s;
                    }
                }
                if name == "concat" {
                    let mut parts = Vec::with_capacity(args.len());
                    let mut all_ok = true;
                    for a in args {
                        match self.eval_concrete(a) {
                            Ok(Expression::String(s))
                            | Ok(Expression::Const(s))
                            | Ok(Expression::Object(s)) => parts.push(s),
                            _ => {
                                all_ok = false;
                                break;
                            }
                        }
                    }
                    if all_ok {
                        return parts.join("");
                    }
                }
                if (name == "Matrix" || name == "matrix") && args.len() == 3 {
                    if let (Some(rows), Some(cols)) =
                        (self.as_integer(&args[0]), self.as_integer(&args[1]))
                    {
                        if let Expression::List(elements) = &args[2] {
                            return self.pretty_print_flat_matrix(
                                rows as usize,
                                cols as usize,
                                elements,
                            );
                        }
                    }
                }
                if args.is_empty() {
                    name.clone()
                } else {
                    let args_str: Vec<String> =
                        args.iter().map(|a| self.pretty_print_value(a)).collect();
                    format!("{}({})", name, args_str.join(", "))
                }
            }
            _ => format!("{:?}", expr),
        }
    }

    pub(crate) fn pretty_print_matrix(&self, rows: &[Expression]) -> String {
        let string_rows: Vec<Vec<String>> = rows
            .iter()
            .map(|row| {
                if let Expression::List(cols) = row {
                    cols.iter().map(|e| self.pretty_print_value(e)).collect()
                } else {
                    vec![self.pretty_print_value(row)]
                }
            })
            .collect();

        if string_rows.is_empty() {
            return "[]".to_string();
        }

        let num_cols = string_rows[0].len();
        let col_widths: Vec<usize> = (0..num_cols)
            .map(|c| {
                string_rows
                    .iter()
                    .map(|row| row.get(c).map(|s| s.len()).unwrap_or(0))
                    .max()
                    .unwrap_or(0)
            })
            .collect();

        let mut lines = Vec::new();
        lines.push(
            "┌".to_string() + &" ".repeat(col_widths.iter().sum::<usize>() + num_cols * 2) + "┐",
        );

        for row in &string_rows {
            let cells: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(i, s)| format!("{:>width$}", s, width = col_widths[i]))
                .collect();
            lines.push(format!("│ {} │", cells.join("  ")));
        }

        lines.push(
            "└".to_string() + &" ".repeat(col_widths.iter().sum::<usize>() + num_cols * 2) + "┘",
        );
        lines.join("\n")
    }

    pub(crate) fn pretty_print_flat_matrix(
        &self,
        rows: usize,
        cols: usize,
        elements: &[Expression],
    ) -> String {
        if elements.len() != rows * cols {
            return format!("Matrix({}, {}, {:?})", rows, cols, elements);
        }

        let string_rows: Vec<Vec<String>> = (0..rows)
            .map(|r| {
                (0..cols)
                    .map(|c| self.pretty_print_value(&elements[r * cols + c]))
                    .collect()
            })
            .collect();

        let col_widths: Vec<usize> = (0..cols)
            .map(|c| {
                string_rows
                    .iter()
                    .map(|row| row.get(c).map(|s| s.len()).unwrap_or(0))
                    .max()
                    .unwrap_or(0)
            })
            .collect();

        let inner_width = col_widths.iter().sum::<usize>() + (cols - 1) * 2;
        let mut lines = Vec::new();
        lines.push(format!("┌{}┐", " ".repeat(inner_width + 2)));

        for row in &string_rows {
            let cells: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(i, s)| format!("{:>width$}", s, width = col_widths[i]))
                .collect();
            lines.push(format!("│ {} │", cells.join("  ")));
        }

        lines.push(format!("└{}┘", " ".repeat(inner_width + 2)));
        lines.join("\n")
    }

    pub(crate) fn as_string(&self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::String(s) => Some(s.clone()),
            Expression::Const(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub(crate) fn as_bool(&self, expr: &Expression) -> Option<bool> {
        match expr {
            Expression::Object(s) => match s.as_str() {
                "true" | "True" => Some(true),
                "false" | "False" => Some(false),
                _ => None,
            },
            Expression::Const(s) => match s.as_str() {
                "true" | "True" => Some(true),
                "false" | "False" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn values_equal(&self, a: &Expression, b: &Expression) -> bool {
        match (a, b) {
            (Expression::Const(x), Expression::Const(y)) => x == y,
            (Expression::String(x), Expression::String(y)) => x == y,
            (Expression::Object(x), Expression::Object(y)) => x == y,
            (Expression::Const(x), Expression::String(y)) => x == y,
            (Expression::String(x), Expression::Const(y)) => x == y,
            _ => false,
        }
    }

    // === Matrix helper methods ===

    pub(crate) fn extract_matrix(
        &self,
        expr: &Expression,
    ) -> Option<(usize, usize, Vec<Expression>)> {
        match expr {
            Expression::List(rows) if !rows.is_empty() => {
                let first_row = match &rows[0] {
                    Expression::List(cols) => cols,
                    _ => return None,
                };

                let m = rows.len();
                let n = first_row.len();

                if n == 0 {
                    return None;
                }

                let mut elements = Vec::with_capacity(m * n);
                for row in rows {
                    match row {
                        Expression::List(cols) => {
                            if cols.len() != n {
                                return None;
                            }
                            elements.extend(cols.clone());
                        }
                        _ => return None,
                    }
                }

                Some((m, n, elements))
            }

            Expression::Operation { name, args, .. } if name == "Matrix" && args.len() >= 3 => {
                let m = self.as_integer(&args[0])? as usize;
                let n = self.as_integer(&args[1])? as usize;

                let elements = match &args[2] {
                    Expression::List(elems) => elems.clone(),
                    Expression::Operation {
                        name: list_name,
                        args: list_args,
                        ..
                    } if list_name == "List" => list_args.clone(),
                    _ if args.len() > 3 => args[2..].to_vec(),
                    other => vec![other.clone()],
                };

                if elements.len() == m * n {
                    Some((m, n, elements))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(crate) fn make_matrix(
        &self,
        m: usize,
        n: usize,
        elements: Vec<Expression>,
    ) -> Expression {
        Expression::Operation {
            name: "Matrix".to_string(),
            args: vec![
                Expression::Const(format!("{}", m)),
                Expression::Const(format!("{}", n)),
                Expression::List(elements),
            ],
            span: None,
        }
    }

    // === Symbolic arithmetic helpers ===

    pub(crate) fn add_expressions(&self, a: &Expression, b: &Expression) -> Expression {
        match (self.as_number(a), self.as_number(b)) {
            (Some(x), Some(y)) => {
                let sum = x + y;
                if sum.fract() == 0.0 && sum.abs() < 1e15 {
                    Expression::Const(format!("{}", sum as i64))
                } else {
                    Expression::Const(format!("{}", sum))
                }
            }
            (Some(0.0), None) => b.clone(),
            (None, Some(0.0)) => a.clone(),
            _ => Expression::Operation {
                name: "plus".to_string(),
                args: vec![a.clone(), b.clone()],
                span: None,
            },
        }
    }

    pub(crate) fn sub_expressions(&self, a: &Expression, b: &Expression) -> Expression {
        match (self.as_number(a), self.as_number(b)) {
            (Some(x), Some(y)) => {
                let diff = x - y;
                if diff.fract() == 0.0 && diff.abs() < 1e15 {
                    Expression::Const(format!("{}", diff as i64))
                } else {
                    Expression::Const(format!("{}", diff))
                }
            }
            (None, Some(0.0)) => a.clone(),
            _ => Expression::Operation {
                name: "minus".to_string(),
                args: vec![a.clone(), b.clone()],
                span: None,
            },
        }
    }

    pub(crate) fn mul_expressions(&self, a: &Expression, b: &Expression) -> Expression {
        match (self.as_number(a), self.as_number(b)) {
            (Some(x), Some(y)) => {
                let prod = x * y;
                if prod.fract() == 0.0 && prod.abs() < 1e15 {
                    Expression::Const(format!("{}", prod as i64))
                } else {
                    Expression::Const(format!("{}", prod))
                }
            }
            (Some(0.0), _) => Expression::Const("0".to_string()),
            (_, Some(0.0)) => Expression::Const("0".to_string()),
            (Some(1.0), None) => b.clone(),
            (None, Some(1.0)) => a.clone(),
            _ => Expression::Operation {
                name: "times".to_string(),
                args: vec![a.clone(), b.clone()],
                span: None,
            },
        }
    }

    pub(crate) fn negate_expression(&self, a: &Expression) -> Expression {
        match self.as_number(a) {
            Some(x) => {
                let neg = -x;
                if neg.fract() == 0.0 && neg.abs() < 1e15 {
                    Expression::Const(format!("{}", neg as i64))
                } else {
                    Expression::Const(format!("{}", neg))
                }
            }
            None if matches!(a, Expression::Const(s) if s == "0") => {
                Expression::Const("0".to_string())
            }
            None => Expression::Operation {
                name: "negate".to_string(),
                args: vec![a.clone()],
                span: None,
            },
        }
    }

    // === Complex number helpers ===

    pub(crate) fn extract_complex(&self, expr: &Expression) -> Option<(Expression, Expression)> {
        match expr {
            Expression::Operation { name, args, .. }
                if (name == "complex" || name == "Complex") && args.len() == 2 =>
            {
                Some((args[0].clone(), args[1].clone()))
            }
            _ => None,
        }
    }

    pub(crate) fn make_complex(&self, re: Expression, im: Expression) -> Expression {
        Expression::Operation {
            name: "complex".to_string(),
            args: vec![re, im],
            span: None,
        }
    }

    pub(crate) fn extract_complex_matrix(
        &self,
        expr: &Expression,
    ) -> Option<(Expression, Expression)> {
        match expr {
            Expression::List(parts) if parts.len() == 2 => {
                if self.extract_matrix(&parts[0]).is_some()
                    && self.extract_matrix(&parts[1]).is_some()
                {
                    Some((parts[0].clone(), parts[1].clone()))
                } else {
                    None
                }
            }
            Expression::Operation { name, args, .. }
                if (name == "pair" || name == "tuple" || name == "Pair" || name == "Tuple")
                    && args.len() == 2 =>
            {
                if self.extract_matrix(&args[0]).is_some()
                    && self.extract_matrix(&args[1]).is_some()
                {
                    Some((args[0].clone(), args[1].clone()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub(crate) fn as_nat(&self, expr: &Expression) -> Option<usize> {
        match expr {
            Expression::Const(s) => s.parse::<usize>().ok(),
            _ => {
                if let Ok(Expression::Const(s)) = self.eval_concrete(expr) {
                    s.parse::<usize>().ok()
                } else {
                    None
                }
            }
        }
    }
}
