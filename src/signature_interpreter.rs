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
    ///   1. Parse signature to get expected argument types
    ///   2. Unify actual args with expected (bind params, check constraints)
    ///   3. Substitute into result type
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

        // Parse the function signature to get expected argument types
        let (expected_arg_types, result_type) = self.parse_function_signature(operation)?;

        // If no expected args (signature is just result type), use old binding method
        if expected_arg_types.is_empty() {
            // Fallback to old behavior for signatures without arrows
            self.bind_from_args(structure, arg_types)?;
        } else {
            // Unify actual arguments with expected types
            // This validates constraints like "both Matrix(m, n, T)"
            self.unify_arguments(arg_types, &expected_arg_types)?;
        }

        // Now interpret the result type with bound parameters
        self.interpret_type_expr(&result_type)
    }

    /// Parse a function signature into (argument types, result type)
    /// Example: Matrix(m, n, T) → Matrix(m, n, T) → Matrix(m, n, T)
    /// Returns: ([Matrix(m,n,T), Matrix(m,n,T)], Matrix(m,n,T))
    ///
    /// For single-arg operations: Matrix(m, n, T) (no arrow)
    /// Returns: ([], Matrix(m,n,T)) - result is the type itself
    fn parse_function_signature(
        &self,
        sig: &TypeExpr,
    ) -> Result<(Vec<TypeExpr>, TypeExpr), String> {
        match sig {
            TypeExpr::Function(from, to) => {
                // Recursively parse nested functions
                let (mut args, result) = self.parse_function_signature(to)?;
                args.insert(0, (**from).clone());
                Ok((args, result))
            }
            _ => {
                // Base case: no arrows, this is the result type
                // For operations like: transpose : Matrix(n, m, T)
                // This means the operation is already bound and returns this type
                Ok((vec![], sig.clone()))
            }
        }
    }

    /// Unify actual argument types with expected types from signature
    /// This is where dimension constraints get checked!
    fn unify_arguments(
        &mut self,
        actual_types: &[Type],
        expected_types: &[TypeExpr],
    ) -> Result<(), String> {
        if actual_types.len() != expected_types.len() {
            return Err(format!(
                "Argument count mismatch: got {}, expected {}",
                actual_types.len(),
                expected_types.len()
            ));
        }

        for (i, (actual, expected)) in actual_types.iter().zip(expected_types.iter()).enumerate() {
            self.unify_with_expected(actual, expected)
                .map_err(|e| format!("Argument {}: {}", i + 1, e))?;
        }

        Ok(())
    }

    /// Unify an actual type with an expected TypeExpr
    /// Binds parameters and checks constraints
    fn unify_with_expected(&mut self, actual: &Type, expected: &TypeExpr) -> Result<(), String> {
        match (actual, expected) {
            // Matrix(m, n) unifies with Matrix(m, n, T) by binding m, n
            (Type::Matrix(m, n), TypeExpr::Parametric(name, params)) if name == "Matrix" => {
                if params.len() >= 2 {
                    self.bind_or_check_param(&params[0], *m)?;
                    self.bind_or_check_param(&params[1], *n)?;
                }
                Ok(())
            }

            // Scalar unifies with any type parameter T or ℝ
            (Type::Scalar, TypeExpr::Named(name)) if name == "T" || name == "ℝ" => Ok(()),

            // Type variables unify with anything (unknown type)
            (Type::Var(_), _) => Ok(()),

            _ => Ok(()), // For now, accept other combinations
        }
    }

    /// Bind a parameter or check it matches existing binding
    /// This is where "both Matrix(m, n, T)" constraint gets enforced!
    fn bind_or_check_param(&mut self, param_expr: &TypeExpr, value: usize) -> Result<(), String> {
        match param_expr {
            TypeExpr::Named(name) => {
                if let Some(&existing) = self.bindings.get(name) {
                    // Parameter already bound - check it matches!
                    if existing != value {
                        return Err(format!(
                            "Dimension constraint violated: {} was bound to {}, but got {}",
                            name, existing, value
                        ));
                    }
                } else {
                    // First time seeing this parameter - bind it
                    self.bindings.insert(name.clone(), value);
                }
                Ok(())
            }
            TypeExpr::Var(_) => Ok(()), // Type variable, no constraint
            _ => Ok(()),                // Complex expression, handle later
        }
    }

    /// Bind structure type parameters from argument types
    ///
    /// Example:
    ///   Structure params: (m: Nat, n: Nat, p: Nat, T)
    ///   Arg types: [Matrix(2, 3), Matrix(3, 5)]
    ///
    /// From Matrix(2, 3): m=2, n=3
    /// From Matrix(3, 5): CHECKS m=2?, n=3?, then binds p=5
    ///
    /// This is where dimension constraints get enforced!
    fn bind_from_args(
        &mut self,
        structure: &StructureDef,
        arg_types: &[Type],
    ) -> Result<(), String> {
        // For Matrix operations, extract and validate dimensions from ALL args
        if structure.name.contains("Matrix") {
            let param_names = &structure.type_params;

            // Process each matrix argument
            for (arg_idx, arg_type) in arg_types.iter().enumerate() {
                if let Type::Matrix(rows, cols) = arg_type {
                    // For MatrixAddable(m, n, T): both matrices must have same m, n
                    // For MatrixMultipliable(m, n, p, T): first is (m,n), second is (n,p)

                    if structure.name == "MatrixAddable" {
                        // Both matrices must have same (m, n)
                        self.bind_or_check("m", *rows, format!("argument {}", arg_idx + 1))?;
                        self.bind_or_check("n", *cols, format!("argument {}", arg_idx + 1))?;
                    } else if structure.name == "MatrixMultipliable" {
                        if arg_idx == 0 {
                            // First matrix: bind m and n
                            self.bind_or_check("m", *rows, "first matrix rows".to_string())?;
                            self.bind_or_check("n", *cols, "first matrix cols".to_string())?;
                        } else if arg_idx == 1 {
                            // Second matrix: check rows=n, bind p=cols
                            self.bind_or_check("n", *rows, "second matrix rows".to_string())?;
                            self.bind_or_check("p", *cols, "second matrix cols".to_string())?;
                        }
                    } else if structure.name == "SquareMatrix" {
                        // SquareMatrix(n, T): must be n×n (rows = cols)
                        if rows != cols {
                            return Err(format!(
                                "{} requires square matrix!\n  Got: {}×{} (non-square)\n  {} only defined for n×n matrices",
                                structure.name, rows, cols, structure.name
                            ));
                        }
                        self.bind_or_check("n", *rows, "square matrix dimension".to_string())?;
                    } else {
                        // Generic Matrix structure: bind m, n from first matrix
                        if arg_idx == 0 {
                            self.bind_or_check("m", *rows, "matrix rows".to_string())?;
                            self.bind_or_check("n", *cols, "matrix cols".to_string())?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Bind a parameter value or check it matches existing binding
    /// This enforces that all uses of 'm' have the same value!
    fn bind_or_check(
        &mut self,
        param_name: &str,
        value: usize,
        context: String,
    ) -> Result<(), String> {
        if let Some(&existing) = self.bindings.get(param_name) {
            if existing != value {
                return Err(format!(
                    "Dimension constraint violated for parameter '{}':\n  \
                     Previously bound to {} \n  \
                     But {} has {} \n  \
                     All uses of '{}' must have the same value!",
                    param_name, existing, context, value, param_name
                ));
            }
        } else {
            self.bindings.insert(param_name.to_string(), value);
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
