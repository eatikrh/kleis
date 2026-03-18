//! Signature Interpreter - Parse and Apply Type Signatures
//!
//! This is the KEY to ADR-016 compliance!
//!
//! Instead of hardcoding operation rules in Rust,
//! we READ type signatures from Kleis structures and INTERPRET them.
//!
//! Example from stdlib/matrices.kleis:
//!   structure MatrixMultipliable(m: Nat, n: Nat, p: Nat, T) {
//!       operation multiply : Matrix(m, p, T)
//!   }
//!
//! Given: multiply(Matrix(2, 3), Matrix(3, 5))
//!
//! Interpretation:
//! 1. Structure params: m, n, p, T
//! 2. Unify with args: m=2, n=3, p=5, T=ℝ
//! 3. Result type: Matrix(m, p, T) = Matrix(2, 5, ℝ)
//!
//! This is SELF-HOSTING: Kleis defines Kleis!
use crate::data_registry::DataTypeRegistry;
use crate::kleis_ast::{DimExpr, StructureDef, StructureMember, TypeExpr};
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

    /// Fresh type variable counter for polymorphic type constructors
    next_type_var: usize,

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
            next_type_var: 0,
            data_registry,
            structure_registry,
        }
    }

    fn fresh_type_var(&mut self) -> Type {
        let var = TypeVar::new(self.next_type_var);
        self.next_type_var += 1;
        Type::Var(var)
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
    #[allow(clippy::only_used_in_recursion)]
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
                            // NOTE: This currently accepts Matrix when ℝ is expected.
                            // This is intentional because:
                            // 1. Polymorphic definitions (sin : T → T) are loaded and take precedence
                            // 2. Matrix transcendentals are mathematically valid (e.g., e^(A-sI) in control theory)
                            // 3. The limitation is in backends (Z3 treats as uninterpreted), not types
                            //
                            // See: docs/session-2024-12-12/TRANSCENDENTAL_FUNCTIONS.md
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
        // Legacy fallback for old-style signatures without arrows
        // Modern signatures should use arrows and unify_arguments instead
        //
        // This extracts Nat parameters from argument types and binds them
        // to the structure's type parameters based on positional matching

        if structure.type_params.is_empty() {
            return Ok(()); // No params to bind
        }

        // For each argument, try to extract Nat values and bind to structure params
        for (arg_idx, arg_type) in arg_types.iter().enumerate() {
            if let Type::Data {
                args: type_args, ..
            } = arg_type
            {
                // Extract Nat values from the argument type
                let nat_values: Vec<usize> = type_args
                    .iter()
                    .filter_map(|arg| {
                        if let Type::NatValue(n) = arg {
                            Some(*n)
                        } else {
                            None
                        }
                    })
                    .collect();

                // Bind to structure's Nat parameters positionally
                // This is a simple heuristic for backwards compatibility
                let mut nat_param_idx = 0;
                for (param_idx, param) in structure.type_params.iter().enumerate() {
                    if matches!(param.kind, Some(crate::kleis_ast::KindExpr::Nat))
                        && nat_param_idx < nat_values.len()
                    {
                        let value = nat_values[nat_param_idx];
                        self.bind_or_check(
                            &param.name,
                            value,
                            format!("argument {} parameter {}", arg_idx + 1, param_idx + 1),
                        )?;
                        nat_param_idx += 1;
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
                // Both are Data types with same constructor → unify arguments recursively
                (
                    Type::Data {
                        constructor: c1,
                        args: args1,
                        ..
                    },
                    Type::Data {
                        constructor: c2,
                        args: args2,
                        ..
                    },
                ) if c1 == c2 && args1.len() == args2.len() => {
                    // Recursively unify each type argument
                    for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                        // Create a temporary type expression for recursive unification
                        // For now, if both are Vars, add substitution
                        match (arg1, arg2) {
                            (Type::Var(v1), Type::Var(v2)) if v1 != v2 => {
                                // Unify the two variables
                                self.substitutions.insert(v2.clone(), arg1.clone());
                            }
                            (Type::Var(v), concrete) | (concrete, Type::Var(v))
                                if !matches!(concrete, Type::Var(_)) =>
                            {
                                self.substitutions.insert(v.clone(), concrete.clone());
                            }
                            _ if arg1 == arg2 => {
                                // Equal - OK
                            }
                            _ => {
                                return Err(format!(
                                    "Type parameter '{}' has mismatched nested types in {:?} vs {:?}",
                                    param_name, arg1, arg2
                                ));
                            }
                        }
                    }
                    Ok(())
                }
                // Type application vs parametric data type: normalize to App form
                (Type::App(_, _), Type::Data { .. }) | (Type::Data { .. }, Type::App(_, _)) => {
                    let to_app = |ty: &Type| match ty {
                        Type::Data {
                            type_name,
                            constructor,
                            args,
                        } if !args.is_empty() => {
                            let mut app = Type::Data {
                                type_name: type_name.clone(),
                                constructor: constructor.clone(),
                                args: vec![],
                            };
                            for arg in args {
                                app = Type::App(Box::new(app), Box::new(arg.clone()));
                            }
                            Some(app)
                        }
                        _ => None,
                    };

                    let left = to_app(&resolved_existing).unwrap_or(resolved_existing.clone());
                    let right = to_app(&resolved_ty).unwrap_or(resolved_ty.clone());

                    match (left, right) {
                        (Type::App(f1, a1), Type::App(f2, a2)) => {
                            self.bind_or_check_type(param_name, &f1)?;
                            self.bind_or_check_type(param_name, &a1)?;
                            self.bind_or_check_type(param_name, &f2)?;
                            self.bind_or_check_type(param_name, &a2)?;
                            Ok(())
                        }
                        (a, b) if a == b => Ok(()),
                        (a, b) => Err(format!(
                            "Type parameter '{}' mismatch:\n  \
                             Previously bound to {:?}\n  \
                             But got {:?}\n  \
                             All uses of '{}' must have the same type!",
                            param_name, a, b, param_name
                        )),
                    }
                }
                // Both are type applications → unify constructor and argument
                (Type::App(f1, a1), Type::App(f2, a2)) => {
                    self.bind_or_check_type(param_name, f1)?;
                    self.bind_or_check_type(param_name, a1)?;
                    self.bind_or_check_type(param_name, f2)?;
                    self.bind_or_check_type(param_name, a2)?;
                    Ok(())
                }
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
            Type::App(func, arg) => Type::App(
                Box::new(self.apply_substitution(func)),
                Box::new(self.apply_substitution(arg)),
            ),
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
    pub fn interpret_type_expr(&mut self, type_expr: &TypeExpr) -> Result<Type, String> {
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
                    // Numeric types
                    "ℕ" | "Nat" => Ok(Type::Nat),
                    "ℤ" | "Int" | "Integer" => Ok(Type::Data {
                        type_name: "Type".to_string(),
                        constructor: "Int".to_string(),
                        args: vec![],
                    }),
                    "ℚ" | "Rational" => Ok(Type::Data {
                        type_name: "Type".to_string(),
                        constructor: "Rational".to_string(),
                        args: vec![],
                    }),
                    "ℝ" | "Real" | "Scalar" => Ok(Type::scalar()),
                    "ℂ" | "Complex" => Ok(Type::Data {
                        type_name: "Type".to_string(),
                        constructor: "Complex".to_string(),
                        args: vec![],
                    }),
                    // Boolean
                    "Bool" | "𝔹" => Ok(Type::Bool),
                    // String
                    "String" => Ok(Type::String),
                    // Unit
                    "Unit" | "()" => Ok(Type::Unit),
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
                // 0. If this is a bound type constructor, apply it to parameters
                if let Some(binding) = self.type_bindings.get(name).cloned() {
                    let args: Result<Vec<Type>, String> = param_exprs
                        .iter()
                        .map(|param_expr| self.interpret_type_expr(param_expr))
                        .collect();
                    let args = args?;

                    if let Type::Data {
                        type_name,
                        constructor,
                        args: existing_args,
                    } = &binding
                    {
                        if existing_args.is_empty() {
                            return Ok(Type::Data {
                                type_name: type_name.clone(),
                                constructor: constructor.clone(),
                                args,
                            });
                        }
                    }

                    let mut applied = binding;
                    for arg in args {
                        applied = Type::App(Box::new(applied), Box::new(arg));
                    }
                    return Ok(applied);
                }

                // 0b. If this is an unbound type constructor variable (M, F, G),
                // create a fresh type variable and bind it, then apply.
                if name.len() == 1 && name.chars().next().unwrap().is_uppercase() {
                    let entry = if let Some(existing) = self.type_bindings.get(name).cloned() {
                        existing
                    } else {
                        let fresh = self.fresh_type_var();
                        self.type_bindings.insert(name.clone(), fresh.clone());
                        fresh
                    };
                    let args: Result<Vec<Type>, String> = param_exprs
                        .iter()
                        .map(|param_expr| self.interpret_type_expr(param_expr))
                        .collect();
                    let args = args?;
                    let mut applied = entry;
                    for arg in args {
                        applied = Type::App(Box::new(applied), Box::new(arg));
                    }
                    return Ok(applied);
                }

                // 0. Handle built-in parametric types first
                match name.as_str() {
                    // Set(T) - built-in set type backed by Z3
                    "Set" => {
                        if param_exprs.len() != 1 {
                            return Err(format!(
                                "Set expects 1 type parameter, got {}",
                                param_exprs.len()
                            ));
                        }
                        let element_type = self.interpret_type_expr(&param_exprs[0])?;
                        return Ok(Type::Data {
                            type_name: "Set".to_string(),
                            constructor: "Set".to_string(),
                            args: vec![element_type],
                        });
                    }
                    // BitVec(n) - bit-vector type
                    "BitVec" => {
                        if param_exprs.len() != 1 {
                            return Err(format!(
                                "BitVec expects 1 size parameter, got {}",
                                param_exprs.len()
                            ));
                        }
                        let n = self.eval_param(&param_exprs[0])?;
                        return Ok(Type::Data {
                            type_name: "BitVec".to_string(),
                            constructor: "BitVec".to_string(),
                            args: vec![Type::NatValue(n)],
                        });
                    }
                    _ => {}
                }

                // 1. Check if this is a user-defined parametric type
                if let Some(data_def) = self.data_registry.get_type(name) {
                    let type_params = data_def.type_params.clone();
                    // GENERIC handling for ANY arity!
                    // The arity comes from the DataDef, not hardcoded!
                    let expected_arity = type_params.len();

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
                    for (param_def, param_expr) in type_params.iter().zip(param_exprs) {
                        let arg = match &param_def.kind {
                            Some(crate::kleis_ast::KindExpr::Nat) => {
                                // Natural number parameter (dimension, index, etc.)
                                match self.eval_param(param_expr) {
                                    Ok(n) => Type::NatValue(n),
                                    Err(msg) if msg.starts_with("Unbound parameter: ") => {
                                        match param_expr {
                                            TypeExpr::Named(name) => {
                                                Type::NatExpr(DimExpr::Var(name.clone()))
                                            }
                                            TypeExpr::DimExpr(expr) => Type::NatExpr(expr.clone()),
                                            _ => {
                                                return Err(msg);
                                            }
                                        }
                                    }
                                    Err(msg) => {
                                        return Err(msg);
                                    }
                                }
                            }
                            Some(crate::kleis_ast::KindExpr::String) => {
                                // String parameter (label, name, etc.)
                                let s = self.eval_string_param(param_expr)?;
                                Type::StringValue(s)
                            }
                            Some(crate::kleis_ast::KindExpr::Type)
                            | Some(crate::kleis_ast::KindExpr::Named(_))
                            | Some(crate::kleis_ast::KindExpr::Arrow(_, _))
                            | None => {
                                // Type parameter - recursively interpret
                                self.interpret_type_expr(param_expr)?
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
                    let type_params = structure_def.type_params.clone();
                    // GENERIC handling for ANY parametric structure!
                    let expected_arity = type_params.len();

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
                    for (param_def, param_expr) in type_params.iter().zip(param_exprs) {
                        let arg = match &param_def.kind {
                            Some(crate::kleis_ast::KindExpr::Nat) => {
                                // Natural number parameter (dimension, index, etc.)
                                match self.eval_param(param_expr) {
                                    Ok(n) => Type::NatValue(n),
                                    Err(msg) if msg.starts_with("Unbound parameter: ") => {
                                        match param_expr {
                                            TypeExpr::Named(name) => {
                                                Type::NatExpr(DimExpr::Var(name.clone()))
                                            }
                                            TypeExpr::DimExpr(expr) => Type::NatExpr(expr.clone()),
                                            _ => {
                                                return Err(msg);
                                            }
                                        }
                                    }
                                    Err(msg) => {
                                        return Err(msg);
                                    }
                                }
                            }
                            Some(crate::kleis_ast::KindExpr::String) => {
                                // String parameter (label, name, etc.)
                                let s = self.eval_string_param(param_expr)?;
                                Type::StringValue(s)
                            }
                            Some(crate::kleis_ast::KindExpr::Type)
                            | Some(crate::kleis_ast::KindExpr::Named(_))
                            | Some(crate::kleis_ast::KindExpr::Arrow(_, _))
                            | None => {
                                // Type parameter - recursively interpret
                                self.interpret_type_expr(param_expr)?
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
            TypeExpr::DimExpr(expr) => self.eval_dim_expr(expr),
            _ => Err("Complex parameter evaluation not yet supported".to_string()),
        }
    }

    fn eval_dim_expr(&self, expr: &DimExpr) -> Result<usize, String> {
        match expr {
            DimExpr::Lit(n) => Ok(*n),
            DimExpr::Var(name) => self
                .bindings
                .get(name)
                .copied()
                .ok_or_else(|| format!("Unbound parameter: {}", name)),
            DimExpr::Add(l, r) => Ok(self.eval_dim_expr(l)? + self.eval_dim_expr(r)?),
            DimExpr::Sub(l, r) => {
                let left = self.eval_dim_expr(l)?;
                let right = self.eval_dim_expr(r)?;
                left.checked_sub(right)
                    .ok_or_else(|| format!("Dimension underflow: {} - {}", left, right))
            }
            DimExpr::Mul(l, r) => Ok(self.eval_dim_expr(l)? * self.eval_dim_expr(r)?),
            DimExpr::Div(l, r) => {
                let left = self.eval_dim_expr(l)?;
                let right = self.eval_dim_expr(r)?;
                if right == 0 {
                    return Err("Division by zero in dimension expression".to_string());
                }
                if left % right != 0 {
                    return Err(format!(
                        "Non-integer dimension division: {} / {}",
                        left, right
                    ));
                }
                Ok(left / right)
            }
            DimExpr::Pow(l, r) => {
                let base = self.eval_dim_expr(l)?;
                let exp = self.eval_dim_expr(r)?;
                Ok(base.pow(exp as u32))
            }
            DimExpr::Call(name, args) => {
                let vals: Result<Vec<_>, _> =
                    args.iter().map(|arg| self.eval_dim_expr(arg)).collect();
                let vals = vals?;
                match name.as_str() {
                    "min" => vals
                        .into_iter()
                        .min()
                        .ok_or_else(|| "min requires at least one argument".to_string()),
                    "max" => vals
                        .into_iter()
                        .max()
                        .ok_or_else(|| "max requires at least one argument".to_string()),
                    "gcd" => {
                        let mut iter = vals.into_iter();
                        let mut acc = iter
                            .next()
                            .ok_or_else(|| "gcd requires at least one argument".to_string())?;
                        for v in iter {
                            acc = gcd(acc, v);
                        }
                        Ok(acc)
                    }
                    "lcm" => {
                        let mut iter = vals.into_iter();
                        let mut acc = iter
                            .next()
                            .ok_or_else(|| "lcm requires at least one argument".to_string())?;
                        for v in iter {
                            acc = lcm(acc, v)?;
                        }
                        Ok(acc)
                    }
                    _ => Err(format!("Unknown dimension function: {}", name)),
                }
            }
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

fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let tmp = b;
        b = a % b;
        a = tmp;
    }
    a
}

fn lcm(a: usize, b: usize) -> Result<usize, String> {
    if a == 0 || b == 0 {
        return Ok(0);
    }
    let g = gcd(a, b);
    a.checked_div(g)
        .and_then(|v| v.checked_mul(b))
        .ok_or_else(|| "lcm overflow in dimension expression".to_string())
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
            Type::Var(TypeVar::new(0)), // x is unbound
            Type::scalar(),             // 1 is concrete
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
        assert_eq!(
            interp.substitutions.get(&TypeVar::new(0)),
            Some(&Type::scalar())
        );

        println!("✓ Type variable substitution works: Var(0) + Scalar → Scalar");
    }

    #[test]
    fn test_interpret_type_constructor_application() {
        use crate::structure_registry::StructureRegistry;

        let data_registry = DataTypeRegistry::new();
        let structure_registry = StructureRegistry::new();
        let mut interp = SignatureInterpreter::new(data_registry, structure_registry);

        let type_expr = crate::kleis_parser::parse_type_expr("F(ℝ)").unwrap();
        let result = interp.interpret_type_expr(&type_expr).unwrap();

        match result {
            Type::App(func, arg) => {
                assert!(matches!(*func, Type::Var(TypeVar { id: 0, .. })));
                assert!(matches!(
                    *arg,
                    Type::Data {
                        constructor,
                        ..
                    } if constructor == "Scalar"
                ));
            }
            other => panic!("Expected Type::App, got {:?}", other),
        }
    }

    #[test]
    fn test_interpret_type_expr_dimexpr_pow() {
        use crate::structure_registry::StructureRegistry;

        let types_code = include_str!("../stdlib/types.kleis");
        let types_program = parse_kleis_program(types_code).unwrap();

        let mut data_registry = DataTypeRegistry::new();
        for item in types_program.items {
            if let crate::kleis_ast::TopLevel::DataDef(data_def) = item {
                data_registry.register(data_def).unwrap();
            }
        }

        let matrix_structure_code = r#"
            structure Matrix(m: Nat, n: Nat, T) {
                operation transpose : Matrix(n, m, T)
            }
        "#;
        let matrix_program = parse_kleis_program(matrix_structure_code).unwrap();
        let matrix_structure = matrix_program.structures()[0].clone();
        let mut structure_registry = StructureRegistry::new();
        structure_registry.register(matrix_structure).unwrap();

        let mut interp = SignatureInterpreter::new(data_registry, structure_registry);
        interp.bindings.insert("n".to_string(), 3);

        let pow_expr = TypeExpr::DimExpr(DimExpr::Pow(
            Box::new(DimExpr::Lit(2)),
            Box::new(DimExpr::Var("n".to_string())),
        ));

        let ty = interp
            .interpret_type_expr(&TypeExpr::Parametric(
                "Matrix".to_string(),
                vec![pow_expr.clone(), pow_expr, TypeExpr::Named("ℝ".to_string())],
            ))
            .unwrap();

        match ty {
            Type::Data {
                constructor, args, ..
            } => {
                assert_eq!(constructor, "Matrix");
                assert_eq!(args.len(), 3);
                assert_eq!(args[0], Type::NatValue(8));
                assert_eq!(args[1], Type::NatValue(8));
            }
            other => panic!("Expected Matrix type, got {:?}", other),
        }
    }
}
