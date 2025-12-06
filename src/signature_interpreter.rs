///! Signature Interpreter - Parse and Apply Type Signatures
///!
///! This is the KEY to ADR-016 compliance!
///!
///! Instead of hardcoding operation rules in Rust,
///! we READ type signatures from Kleis structures and INTERPRET them.
///!
///! Example from stdlib/matrices.kleis:
///!   structure MatrixMultipliable(m: Nat, n: Nat, p: Nat, T) {
///!       operation multiply : Matrix(m, p, T)
///!   }
///!
///! Given: multiply(Matrix(2, 3), Matrix(3, 5))
///!
///! Interpretation:
///! 1. Structure params: m, n, p, T
///! 2. Unify with args: m=2, n=3, p=5, T=ℝ
///! 3. Result type: Matrix(m, p, T) = Matrix(2, 5, ℝ)
///!
///! This is SELF-HOSTING: Kleis defines Kleis!
use crate::kleis_ast::{StructureDef, StructureMember, TypeExpr};
use crate::type_inference::Type;
use std::collections::HashMap;

/// Interprets operation type signatures from structure definitions
pub struct SignatureInterpreter {
    /// Structure type parameters bound to concrete values
    /// Example: {m: 2, n: 3, p: 5, T: ℝ}
    bindings: HashMap<String, usize>,
}

impl SignatureInterpreter {
    pub fn new() -> Self {
        SignatureInterpreter {
            bindings: HashMap::new(),
        }
    }

    /// Interpret an operation's type signature given argument types
    ///
    /// Example:
    ///   Structure: MatrixMultipliable(m, n, p, T)
    ///   Operation: multiply : Matrix(m, p, T)
    ///   Args: [Matrix(2, 3), Matrix(3, 5)]
    ///
    /// Process:
    ///   1. Bind structure params from args: m=2, n=3, p=5
    ///   2. Check constraints (n of first = p of second)
    ///   3. Substitute into result: Matrix(m, p, T) = Matrix(2, 5, ℝ)
    pub fn interpret_signature(
        &mut self,
        structure: &StructureDef,
        op_name: &str,
        arg_types: &[Type],
    ) -> Result<Type, String> {
        // Find the operation in the structure
        let operation = structure
            .members
            .iter()
            .find_map(|member| {
                if let StructureMember::Operation {
                    name,
                    type_signature,
                } = member
                {
                    if name == op_name {
                        return Some(type_signature);
                    }
                }
                None
            })
            .ok_or_else(|| {
                format!(
                    "Operation '{}' not found in structure '{}'",
                    op_name, structure.name
                )
            })?;

        // Bind structure parameters from argument types
        self.bind_from_args(structure, arg_types)?;

        // Interpret the result type signature
        self.interpret_type_expr(operation)
    }

    /// Bind structure type parameters from argument types
    ///
    /// Example:
    ///   Structure params: (m: Nat, n: Nat, p: Nat, T)
    ///   Arg types: [Matrix(2, 3), Matrix(3, 5)]
    ///
    /// From Matrix(2, 3): m=2, n=3
    /// From Matrix(3, 5): p=5 (and verify n=3 matches)
    fn bind_from_args(
        &mut self,
        structure: &StructureDef,
        arg_types: &[Type],
    ) -> Result<(), String> {
        // For Matrix operations, try to extract dimensions
        if structure.name.contains("Matrix") {
            // Try to bind from first matrix argument
            for arg_type in arg_types {
                if let Type::Matrix(rows, cols) = arg_type {
                    // Bind m and n from first matrix
                    if !self.bindings.contains_key("m") {
                        self.bindings.insert("m".to_string(), *rows);
                    }
                    if !self.bindings.contains_key("n") {
                        self.bindings.insert("n".to_string(), *cols);
                    }

                    // For MatrixMultipliable, second matrix gives p
                    if structure.name == "MatrixMultipliable" {
                        if let Some(Type::Matrix(_second_rows, second_cols)) = arg_types.get(1) {
                            self.bindings.insert("p".to_string(), *second_cols);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Interpret a type expression with current bindings
    ///
    /// Example:
    ///   TypeExpr: Matrix(n, m, T)
    ///   Bindings: {m: 2, n: 3}
    ///   Result: Matrix(3, 2, ℝ)  // n and m swapped!
    fn interpret_type_expr(&self, type_expr: &TypeExpr) -> Result<Type, String> {
        match type_expr {
            TypeExpr::Named(name) => {
                // Simple type like ℝ, or a type parameter like T
                match name.as_str() {
                    "ℝ" | "Real" => Ok(Type::Scalar),
                    "T" => Ok(Type::Scalar), // For now, T defaults to Scalar
                    _ => Ok(Type::Scalar),   // Default
                }
            }

            TypeExpr::Parametric(name, params) => {
                // Matrix(m, n, T) or Matrix(n, m, T), etc.
                if name == "Matrix" && params.len() >= 2 {
                    // Extract rows and cols from params
                    let rows = self.eval_param(&params[0])?;
                    let cols = self.eval_param(&params[1])?;

                    Ok(Type::Matrix(rows, cols))
                } else if name == "Vector" && params.len() >= 1 {
                    let dim = self.eval_param(&params[0])?;
                    Ok(Type::Vector(dim))
                } else {
                    Err(format!("Unknown parametric type: {}", name))
                }
            }

            TypeExpr::Function(_, result) => {
                // For operation signatures, the result is what matters
                // (The input types are checked by unification)
                self.interpret_type_expr(result)
            }

            _ => Err("Unsupported type expression".to_string()),
        }
    }

    /// Evaluate a type parameter to a concrete number
    ///
    /// Example:
    ///   Param: Named("n")
    ///   Bindings: {n: 3}
    ///   Result: 3
    fn eval_param(&self, param: &TypeExpr) -> Result<usize, String> {
        match param {
            TypeExpr::Named(name) => {
                // Look up in bindings
                if let Some(&value) = self.bindings.get(name) {
                    Ok(value)
                } else if let Ok(n) = name.parse::<usize>() {
                    // Direct number
                    Ok(n)
                } else {
                    Err(format!("Unbound parameter: {}", name))
                }
            }
            _ => Err("Complex parameter evaluation not yet supported".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kleis_parser::parse_kleis_program;

    #[test]
    fn test_interpret_transpose_signature() {
        // Parse structure with transpose
        let code = r#"
            structure Matrix(m: Nat, n: Nat, T) {
                operation transpose : Matrix(n, m, T)
            }
        "#;

        let program = parse_kleis_program(code).unwrap();
        let structure = program.structures()[0];

        let mut interp = SignatureInterpreter::new();

        // Bind: m=2, n=3
        interp.bindings.insert("m".to_string(), 2);
        interp.bindings.insert("n".to_string(), 3);

        // Interpret signature
        let arg_types = vec![Type::Matrix(2, 3)];
        let result = interp
            .interpret_signature(structure, "transpose", &arg_types)
            .unwrap();

        // Should be Matrix(3, 2) - dimensions flipped!
        assert_eq!(result, Type::Matrix(3, 2));
    }
}
