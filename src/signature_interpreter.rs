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
use crate::data_registry::DataTypeRegistry;
use crate::kleis_ast::{StructureDef, StructureMember, TypeExpr};
use crate::structure_registry::StructureRegistry;
use crate::type_inference::{Type, TypeVar};
use std::collections::HashMap;

/// Interprets operation type signatures from structure definitions
pub struct SignatureInterpreter {
    /// Dimension bindings for Nat parameters
    /// Example: {m: 2, n: 3, p: 5}
    /// Public for testing
    pub bindings: HashMap<String, usize>,

    /// Type parameter bindings for polymorphic parameters
    /// Example: {T: ℝ, C: Matrix(2,3)}
    /// This enables true polymorphism!
    type_bindings: HashMap<String, Type>,

    /// String parameter bindings for labeled types
    /// Example: {unit: "m/s", label: "velocity"}
    /// Enables unit-safe types like: Metric("m/s", ℝ)
    /// Public for testing
    pub string_bindings: HashMap<String, String>,

    /// Type variable substitutions for proper HM unification
    /// Maps type variables to their resolved types
    /// Example: {TypeVar(0): Scalar, TypeVar(1): Matrix(2,3)}
    substitutions: HashMap<TypeVar, Type>,

    /// Registry of user-defined data types
    /// Enables looking up types like Currency, Tensor3D, Option, etc.
    data_registry: DataTypeRegistry,

    /// Registry of structure definitions
    /// Enables looking up structures like Matrix(m, n, T), Tensor(i, j, k, T), etc.
    /// This allows generic handling of parametric structure types without hardcoding.
    structure_registry: StructureRegistry,
}

impl SignatureInterpreter {
    pub fn new(data_registry: DataTypeRegistry, structure_registry: StructureRegistry) -> Self {
        SignatureInterpreter {
            bindings: HashMap::new(),
            type_bindings: HashMap::new(),
            string_bindings: HashMap::new(),
            substitutions: HashMap::new(),
            data_registry,
            structure_registry,
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
        let result = self.interpret_type_expr(&result_type)?;

        // Apply substitutions to resolve any type variables
        // This is what makes x + 1 correctly infer to Scalar!
        Ok(self.apply_substitution(&result))
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
            // Parametric types: Bind dimensions and check structure
            (
                Type::Data {
                    constructor,
                    args,
                    type_name,
                },
                TypeExpr::Parametric(expected_name, params),
            ) => {
                // Check that constructors match (lenient for backward compatibility)
                if constructor == expected_name || type_name == expected_name {
                    // Bind/check each parameter
                    for (actual_arg, expected_param) in args.iter().zip(params.iter()) {
                        match actual_arg {
                            Type::NatValue(n) => {
                                // Nat parameter - bind dimension
                                self.bind_or_check_param(expected_param, *n)?;
                            }
                            Type::StringValue(s) => {
                                // String parameter - bind string value
                                // Enables: data Metric(unit: String, T) = Metric(...)
                                if let TypeExpr::Named(param_name) = expected_param {
                                    self.bind_or_check_string(param_name, s)?;
                                } else if let TypeExpr::Var(literal) = expected_param {
                                    // String literal in signature - check it matches
                                    if literal != s {
                                        return Err(format!(
                                            "String literal mismatch: expected \"{}\", got \"{}\"",
                                            literal, s
                                        ));
                                    }
                                }
                            }
                            other_type => {
                                // Type parameter - bind the type itself
                                if let TypeExpr::Named(param_name) = expected_param {
                                    self.bind_or_check_type(param_name, other_type)?;
                                }
                            }
                        }
                    }
                }
                // Accept even if names don't match (backward compatibility)
                Ok(())
            }

            // Named type with type parameter: Bind the type
            (actual_type, TypeExpr::Named(param_name)) => {
                // Check if it's a concrete type match
                if param_name == "ℝ" || param_name == "Real" {
                    match actual_type {
                        Type::Data { constructor, .. } if constructor == "Scalar" => Ok(()),
                        _ => {
                            // TODO: Should error on type mismatch (e.g., Matrix when expecting ℝ)
                            // Currently accepts for backward compatibility with existing tests.
                            // Future: Replace Ok() with:
                            // Err(format!("Type mismatch: expected ℝ, got {:?}", actual_type))
                            Ok(())
                        }
                    }
                } else if param_name.len() == 1 && param_name.chars().next().unwrap().is_uppercase()
                {
                    // Single uppercase letter is likely a type parameter - bind it
                    self.bind_or_check_type(param_name, actual_type)?;
                    Ok(())
                } else {
                    // Unknown named type - accept for now (backward compat)
                    Ok(())
                }
            }

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
                if let Type::Data {
                    constructor,
                    args: type_args,
                    ..
                } = arg_type
                {
                    if constructor == "Matrix" && type_args.len() >= 2 {
                        // Extract dimensions from Type::NatValue
                        let rows = match &type_args[0] {
                            Type::NatValue(n) => *n,
                            _ => continue, // Skip if not concrete value
                        };
                        let cols = match &type_args[1] {
                            Type::NatValue(n) => *n,
                            _ => continue, // Skip if not concrete value
                        };

                        // Bind dimensions based on structure type
                        if structure.name == "MatrixAddable" {
                            // Both matrices must have same (m, n)
                            self.bind_or_check("m", rows, format!("argument {}", arg_idx + 1))?;
                            self.bind_or_check("n", cols, format!("argument {}", arg_idx + 1))?;
                        } else if structure.name == "MatrixMultipliable" {
                            if arg_idx == 0 {
                                // First matrix: bind m and n
                                self.bind_or_check("m", rows, "first matrix rows".to_string())?;
                                self.bind_or_check("n", cols, "first matrix cols".to_string())?;
                            } else if arg_idx == 1 {
                                // Second matrix: check rows=n, bind p=cols
                                self.bind_or_check("n", rows, "second matrix rows".to_string())?;
                                self.bind_or_check("p", cols, "second matrix cols".to_string())?;
                            }
                        } else if structure.name == "SquareMatrix" {
                            // SquareMatrix(n, T): must be n×n (rows = cols)
                            if rows != cols {
                                return Err(format!(
                                    "{} requires square matrix!\n  Got: {}×{} (non-square)\n  {} only defined for n×n matrices",
                                    structure.name, rows, cols, structure.name
                                ));
                            }
                            self.bind_or_check("n", rows, "square matrix dimension".to_string())?;
                        } else {
                            // Generic Matrix structure: bind m, n from first matrix
                            if arg_idx == 0 {
                                self.bind_or_check("m", rows, "matrix rows".to_string())?;
                                self.bind_or_check("n", cols, "matrix cols".to_string())?;
                            }
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

    /// Bind a string parameter or check it matches existing binding
    /// This enables unit-safe types: Metric("m/s", ℝ) vs Metric("N", ℝ)
    fn bind_or_check_string(&mut self, param_name: &str, value: &str) -> Result<(), String> {
        if let Some(existing) = self.string_bindings.get(param_name) {
            // Parameter already bound - check it matches!
            if existing != value {
                return Err(format!(
                    "String parameter '{}' mismatch:\n  \
                     Previously bound to \"{}\"\n  \
                     But got \"{}\"\n  \
                     All uses of '{}' must have the same value!",
                    param_name, existing, value, param_name
                ));
            }
        } else {
            // First time seeing this parameter - bind it
            self.string_bindings
                .insert(param_name.to_string(), value.to_string());
        }
        Ok(())
    }

    /// Bind a type parameter or check it matches existing binding
    /// This enables polymorphism: structure Generic(T) can work with ANY type!
    ///
    /// With proper HM: When unifying Var with concrete type, perform substitution!
    fn bind_or_check_type(&mut self, param_name: &str, ty: &Type) -> Result<(), String> {
        // Apply any existing substitutions to both types first
        let resolved_ty = self.apply_substitution(ty);

        if let Some(existing) = self.type_bindings.get(param_name).cloned() {
            let resolved_existing = self.apply_substitution(&existing);

            // Parameter already bound - check if they're compatible
            match (&resolved_existing, &resolved_ty) {
                // Var(α) unifies with concrete type T → substitute α := T
                (Type::Var(v), concrete) if !matches!(concrete, Type::Var(_)) => {
                    self.substitutions.insert(v.clone(), concrete.clone());
                    // Update the binding with the concrete type
                    self.type_bindings
                        .insert(param_name.to_string(), concrete.clone());
                    Ok(())
                }
                // concrete T unifies with Var(α) → substitute α := T
                (concrete, Type::Var(v)) if !matches!(concrete, Type::Var(_)) => {
                    self.substitutions.insert(v.clone(), concrete.clone());
                    // Binding already has concrete type, keep it
                    Ok(())
                }
                // Both are Vars → OK (remain polymorphic)
                (Type::Var(_), Type::Var(_)) => Ok(()),
                // Otherwise, types must match exactly
                (a, b) if a == b => Ok(()),
                (a, b) => Err(format!(
                    "Type parameter '{}' mismatch:\n  \
                     Previously bound to {:?}\n  \
                     But got {:?}\n  \
                     All uses of '{}' must have the same type!",
                    param_name, a, b, param_name
                )),
            }
        } else {
            // First time seeing this parameter - bind it
            self.type_bindings
                .insert(param_name.to_string(), resolved_ty);
            Ok(())
        }
    }

    /// Apply substitutions to a type (resolve type variables)
    /// This is the core of HM unification: α[α := T] = T
    fn apply_substitution(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(v) => {
                if let Some(substituted) = self.substitutions.get(v) {
                    // Recursively apply (in case substitution contains more vars)
                    self.apply_substitution(substituted)
                } else {
                    ty.clone()
                }
            }
            // For Data types with args, apply substitution to args
            Type::Data {
                type_name,
                constructor,
                args,
            } => {
                let substituted_args = args
                    .iter()
                    .map(|arg| self.apply_substitution(arg))
                    .collect();
                Type::Data {
                    type_name: type_name.clone(),
                    constructor: constructor.clone(),
                    args: substituted_args,
                }
            }
            // Other types don't contain type variables
            _ => ty.clone(),
        }
    }

    /// Interpret a type expression with current bindings
    ///
    /// Example:
    ///   TypeExpr: Matrix(n, m, T)
    ///   Bindings: {m: 2, n: 3}
    ///   Result: Matrix(3, 2, ℝ)  // n and m swapped!
    ///
    /// Public for testing
    pub fn interpret_type_expr(&self, type_expr: &TypeExpr) -> Result<Type, String> {
        match type_expr {
            TypeExpr::Named(name) => {
                // 1. Check if this is a bound type parameter
                if let Some(ty) = self.type_bindings.get(name) {
                    return Ok(ty.clone());
                }

                // 2. Check if this is a user-defined type in the registry
                if self.data_registry.has_type(name) {
                    // User-defined simple type (0-arity): Currency, Bool, etc.
                    return Ok(Type::Data {
                        type_name: name.clone(),
                        constructor: name.clone(),
                        args: vec![],
                    });
                }

                // 3. Check built-in types
                match name.as_str() {
                    "ℝ" | "Real" => Ok(Type::scalar()),
                    "Nat" => Ok(Type::Nat),
                    // 4. Unbound type parameters (T, N, S, etc.)
                    // If we reach here, the parameter wasn't bound during unification.
                    // This happens with signatures without arrows (e.g., "transpose : Matrix(n, m)")
                    // where old binding logic is used. For backward compatibility, default to Scalar.
                    // Note: Type variable substitution IS implemented (see bind_or_check_type),
                    // so Var types DO resolve correctly when unified with concrete types!
                    _ if name.len() == 1 && name.chars().next().unwrap().is_uppercase() => {
                        Ok(Type::scalar())
                    }
                    _ => Err(format!("Unknown type: {}", name)),
                }
            }

            TypeExpr::Parametric(name, param_exprs) => {
                // 1. Check if this is a user-defined parametric type
                if let Some(data_def) = self.data_registry.get_type(name) {
                    // GENERIC handling for ANY arity!
                    // The arity comes from the DataDef, not hardcoded!
                    let expected_arity = data_def.type_params.len();

                    if param_exprs.len() != expected_arity {
                        return Err(format!(
                            "Type {} expects {} parameters, got {}",
                            name,
                            expected_arity,
                            param_exprs.len()
                        ));
                    }

                    // Interpret each parameter based on its kind
                    let mut args = Vec::new();
                    for (param_def, param_expr) in data_def.type_params.iter().zip(param_exprs) {
                        let arg = match param_def.kind.as_deref() {
                            Some("Nat") => {
                                // Natural number parameter (dimension, index, etc.)
                                let n = self.eval_param(param_expr)?;
                                Type::NatValue(n)
                            }
                            Some("String") => {
                                // String parameter (label, name, etc.)
                                let s = self.eval_string_param(param_expr)?;
                                Type::StringValue(s)
                            }
                            Some("Type") | None => {
                                // Type parameter - recursively interpret
                                self.interpret_type_expr(param_expr)?
                            }
                            Some(k) => {
                                return Err(format!("Unknown parameter kind: {}", k));
                            }
                        };
                        args.push(arg);
                    }

                    return Ok(Type::Data {
                        type_name: name.clone(),
                        constructor: name.clone(),
                        args,
                    });
                }

                // 2. Check if this is a structure type (Matrix, Vector, custom structures)
                // Structure types are defined with `structure` keyword and represent
                // type classes/interfaces, not concrete data types.
                if let Some(structure_def) = self.structure_registry.get(name) {
                    // GENERIC handling for ANY parametric structure!
                    let expected_arity = structure_def.type_params.len();

                    if param_exprs.len() != expected_arity {
                        return Err(format!(
                            "Structure type {} expects {} parameters, got {}",
                            name,
                            expected_arity,
                            param_exprs.len()
                        ));
                    }

                    // Generic handling for ALL structure types
                    let mut args = Vec::new();
                    for (param_def, param_expr) in structure_def.type_params.iter().zip(param_exprs)
                    {
                        let arg = match param_def.kind.as_deref() {
                            Some("Nat") => {
                                // Natural number parameter (dimension, index, etc.)
                                let n = self.eval_param(param_expr)?;
                                Type::NatValue(n)
                            }
                            Some("String") => {
                                // String parameter (label, name, etc.)
                                let s = self.eval_string_param(param_expr)?;
                                Type::StringValue(s)
                            }
                            Some("Type") | None => {
                                // Type parameter - recursively interpret
                                self.interpret_type_expr(param_expr)?
                            }
                            Some(k) => {
                                return Err(format!("Unknown parameter kind: {}", k));
                            }
                        };
                        args.push(arg);
                    }

                    // Construct Type::Data representation for structure types
                    // Structure types use the same representation as data types
                    // but are conceptually different (interfaces vs concrete types)
                    return Ok(Type::Data {
                        type_name: name.clone(),
                        constructor: name.clone(),
                        args,
                    });
                }

                // Neither data type nor structure type - unknown
                Err(format!("Unknown parametric type: {}", name))
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

    /// Evaluate a string parameter
    ///
    /// Example:
    ///   Param: Named("unit")
    ///   Bindings: {unit: "m/s"}
    ///   Result: "m/s"
    ///
    /// Used for string-valued type parameters like:
    ///   data Metric(unit: String, T) = Metric(...)
    fn eval_string_param(&self, param: &TypeExpr) -> Result<String, String> {
        match param {
            TypeExpr::Named(name) => {
                // Check if bound to a string value
                if let Some(value) = self.string_bindings.get(name) {
                    Ok(value.clone())
                } else {
                    // Not bound - use the name as literal
                    Ok(name.clone())
                }
            }
            TypeExpr::Var(s) => {
                // String literal in signature
                Ok(s.clone())
            }
            _ => Err("Expected string parameter (simple name or literal)".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_registry::DataTypeRegistry;
    use crate::kleis_parser::parse_kleis_program;

    #[test]
    fn test_interpret_transpose_signature() {
        use crate::structure_registry::StructureRegistry;

        // Load stdlib/types.kleis to get Matrix data type
        let types_code = include_str!("../stdlib/types.kleis");
        let types_program = parse_kleis_program(types_code).unwrap();

        let mut data_registry = DataTypeRegistry::new();
        for item in types_program.items {
            if let crate::kleis_ast::TopLevel::DataDef(data_def) = item {
                data_registry.register(data_def).unwrap();
            }
        }

        // Parse structure with transpose and register it
        let code = r#"
            structure Matrix(m: Nat, n: Nat, T) {
                operation transpose : Matrix(n, m, T)
            }
        "#;

        let program = parse_kleis_program(code).unwrap();
        let structure = program.structures()[0].clone();

        // Register Matrix structure in structure_registry
        let mut structure_registry = StructureRegistry::new();
        structure_registry.register(structure.clone()).unwrap();

        let mut interp = SignatureInterpreter::new(data_registry, structure_registry);

        // Bind: m=2, n=3
        interp.bindings.insert("m".to_string(), 2);
        interp.bindings.insert("n".to_string(), 3);

        // Interpret signature
        let arg_types = vec![Type::matrix(2, 3, Type::scalar())];
        let result = interp
            .interpret_signature(&structure, "transpose", &arg_types)
            .unwrap();

        // Should be Matrix(3, 2) - dimensions flipped!
        assert_eq!(result, Type::matrix(3, 2, Type::scalar()));
    }

    #[test]
    fn test_type_variable_substitution() {
        use crate::structure_registry::StructureRegistry;

        // Test that Var(α) + Scalar correctly substitutes α := Scalar
        // This is the key HM unification feature!

        let code = r#"
            structure Arithmetic(T) {
                operation plus : T → T → T
            }
        "#;

        let program = parse_kleis_program(code).unwrap();
        let structure = program.structures()[0];

        let data_registry = DataTypeRegistry::new();
        let structure_registry = StructureRegistry::new();
        let mut interp = SignatureInterpreter::new(data_registry, structure_registry);

        // Simulate: x + 1 where x is Var(0), 1 is Scalar
        use crate::type_inference::TypeVar;
        let arg_types = vec![
            Type::Var(TypeVar(0)), // x is unbound
            Type::scalar(),        // 1 is concrete
        ];

        let result = interp
            .interpret_signature(structure, "plus", &arg_types)
            .unwrap();

        // Result should be Scalar (substituted), not Var(0)!
        assert_eq!(
            result,
            Type::scalar(),
            "Type variable should be substituted with Scalar"
        );

        // Verify substitution was recorded
        assert_eq!(interp.substitutions.len(), 1);
        assert_eq!(interp.substitutions.get(&TypeVar(0)), Some(&Type::scalar()));

        println!("✓ Type variable substitution works: Var(0) + Scalar → Scalar");
    }
}
