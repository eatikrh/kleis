//! Type Context Builder - Connects parsed structures to type inference
//!
//! This module builds a TypeContext from parsed Kleis programs:
//! 1. Loads structure definitions (abstract operations)
//! 2. Loads implements blocks (concrete bindings)
//! 3. Builds operation registry
//! 4. Provides query interface for type checking
//!
//! Example:
//! ```ignore
//! let program = parse_kleis_program("structure Numeric(N) { ... }")?;
//! let ctx = TypeContextBuilder::from_program(program)?;
//! ctx.supports_operation("abs", &Type::Real); // true
//! ```
use crate::kleis_ast::{
    ImplMember, ImplementsDef, Program, StructureDef, StructureMember, TopLevel, TypeExpr,
};
use crate::signature_interpreter::SignatureInterpreter;
use crate::type_inference::{Type, TypeContext};
use std::collections::HashMap;

/// Tracks which structures define which operations
#[derive(Debug, Clone)]
pub struct OperationRegistry {
    /// Maps operation name → structure name
    /// Example: "abs" → "Numeric"
    operation_to_structure: HashMap<String, String>,

    /// Maps structure name → operations it defines
    /// Example: "Numeric" → ["abs", "floor"]
    structure_to_operations: HashMap<String, Vec<String>>,

    /// Maps (type, operation) → implementation
    /// Example: (ℝ, "abs") → "builtin_abs_real"
    concrete_implementations: HashMap<(String, String), String>,

    /// Maps type → structures it implements
    /// Example: ℝ → ["Numeric", "Ordered", "Field"]
    pub type_to_structures: HashMap<String, Vec<String>>,

    /// Maps structure → parent structure (from extends clause)
    /// Example: "Group" → "Monoid", "Ring" → "Group"
    /// This enables automatic type promotion based on structure hierarchy
    structure_extends: HashMap<String, String>,
}

impl Default for OperationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl OperationRegistry {
    pub fn new() -> Self {
        OperationRegistry {
            operation_to_structure: HashMap::new(),
            structure_to_operations: HashMap::new(),
            concrete_implementations: HashMap::new(),
            type_to_structures: HashMap::new(),
            structure_extends: HashMap::new(),
        }
    }

    /// Register that a structure extends another structure
    /// Example: register_extension("Group", "Monoid") means Group extends Monoid
    pub fn register_extension(&mut self, child: &str, parent: &str) {
        self.structure_extends
            .insert(child.to_string(), parent.to_string());
    }

    /// Check if child_structure extends ancestor_structure (transitively)
    pub fn extends(&self, child: &str, ancestor: &str) -> bool {
        if child == ancestor {
            return true;
        }
        let mut current = child;
        while let Some(parent) = self.structure_extends.get(current) {
            if parent == ancestor {
                return true;
            }
            current = parent;
        }
        false
    }

    /// Get the parent structure of a given structure (if any)
    pub fn get_parent(&self, structure: &str) -> Option<&String> {
        self.structure_extends.get(structure)
    }

    /// Register that a structure defines an operation
    pub fn register_operation(&mut self, structure_name: &str, operation_name: &str) {
        self.operation_to_structure
            .insert(operation_name.to_string(), structure_name.to_string());

        self.structure_to_operations
            .entry(structure_name.to_string())
            .or_default()
            .push(operation_name.to_string());
    }

    /// Register that a type implements a structure
    pub fn register_implementation(
        &mut self,
        type_name: &str,
        structure_name: &str,
        operation_name: &str,
        implementation: &str,
    ) {
        // Record that this type implements this structure
        self.type_to_structures
            .entry(type_name.to_string())
            .or_default()
            .push(structure_name.to_string());

        // Record the concrete implementation
        self.concrete_implementations.insert(
            (type_name.to_string(), operation_name.to_string()),
            implementation.to_string(),
        );
    }

    /// Check if a type supports an operation
    pub fn supports_operation(&self, type_name: &str, operation_name: &str) -> bool {
        // Find which structure defines this operation
        if let Some(structure_name) = self.operation_to_structure.get(operation_name) {
            // Check if this type implements that structure
            if let Some(structures) = self.type_to_structures.get(type_name) {
                return structures.contains(structure_name);
            }
        }
        false
    }

    /// Get all types that support an operation
    pub fn types_supporting(&self, operation_name: &str) -> Vec<String> {
        let mut types = Vec::new();

        // Find which structure defines this operation
        if let Some(structure_name) = self.operation_to_structure.get(operation_name) {
            // Find all types that implement that structure
            for (type_name, structures) in &self.type_to_structures {
                if structures.contains(structure_name) {
                    types.push(type_name.clone());
                }
            }
        }

        types
    }

    /// Get the structure that defines an operation
    pub fn structure_for_operation(&self, operation_name: &str) -> Option<&String> {
        self.operation_to_structure.get(operation_name)
    }

    /// Merge another OperationRegistry into this one
    pub fn merge(&mut self, other: OperationRegistry) -> Result<(), String> {
        // Merge operation_to_structure
        for (op, structure) in other.operation_to_structure {
            if let Some(existing) = self.operation_to_structure.get(&op) {
                if existing != &structure {
                    return Err(format!(
                        "Operation '{}' defined in both '{}' and '{}'",
                        op, existing, structure
                    ));
                }
            } else {
                self.operation_to_structure.insert(op, structure);
            }
        }

        // Merge structure_to_operations
        for (structure, ops) in other.structure_to_operations {
            self.structure_to_operations
                .entry(structure)
                .or_default()
                .extend(ops);
        }

        // Merge concrete_implementations
        for ((ty, op), impl_name) in other.concrete_implementations {
            self.concrete_implementations.insert((ty, op), impl_name);
        }

        // Merge type_to_structures
        for (ty, structures) in other.type_to_structures {
            self.type_to_structures
                .entry(ty)
                .or_default()
                .extend(structures);
        }

        Ok(())
    }
}

/// Builds TypeContext from parsed Kleis programs
pub struct TypeContextBuilder {
    /// Parsed structures (abstract)
    structures: HashMap<String, StructureDef>,

    /// Parsed implements (concrete)
    implements: Vec<ImplementsDef>,

    /// Operation registry
    registry: OperationRegistry,

    /// Type context (for inference)
    context: TypeContext,

    /// Type aliases: name -> underlying type expression
    type_aliases: HashMap<String, TypeExpr>,
}

impl TypeContextBuilder {
    pub fn new() -> Self {
        TypeContextBuilder {
            structures: HashMap::new(),
            implements: Vec::new(),
            registry: OperationRegistry::new(),
            context: TypeContext::new(),
            type_aliases: HashMap::new(),
        }
    }

    /// Merge another TypeContextBuilder into this one
    /// This allows incremental loading of Kleis libraries
    pub fn merge(&mut self, other: TypeContextBuilder) -> Result<(), String> {
        // Merge structures (check for conflicts)
        for (name, structure) in other.structures {
            #[allow(clippy::map_entry)]
            if self.structures.contains_key(&name) {
                // Structure already exists - this is OK if they're identical
                // For now, we'll just warn and skip
                eprintln!("Warning: Structure '{}' already defined, skipping", name);
            } else {
                self.structures.insert(name, structure);
            }
        }

        // Merge implements (just append, duplicates are OK)
        self.implements.extend(other.implements);

        // Merge operation registry
        self.registry.merge(other.registry)?;

        // Merge aliases
        for (name, aliased) in other.type_aliases {
            self.type_aliases.entry(name).or_insert(aliased);
        }

        // Context merging is not needed (it's ephemeral)

        Ok(())
    }

    /// Build type context from a parsed program
    pub fn from_program(program: Program) -> Result<Self, String> {
        let mut builder = Self::new();

        // Phase 0: Register type aliases
        for item in &program.items {
            if let TopLevel::TypeAlias(alias) = item {
                builder
                    .type_aliases
                    .insert(alias.name.clone(), alias.type_expr.clone());
            }
        }

        // Phase 1: Register all structures (abstract operations)
        for item in &program.items {
            if let TopLevel::StructureDef(structure) = item {
                builder.register_structure(structure)?;
            }
        }

        // Phase 2: Register all implements (concrete bindings)
        for item in &program.items {
            if let TopLevel::ImplementsDef(impl_def) = item {
                builder.register_implements(impl_def)?;
            }
        }

        // Phase 3: Register top-level operations (utilities)
        for item in &program.items {
            if let TopLevel::OperationDecl(op_decl) = item {
                builder.register_toplevel_operation(op_decl)?;
            }
        }

        Ok(builder)
    }

    fn register_structure(&mut self, structure: &StructureDef) -> Result<(), String> {
        // Register operations from this structure (including nested)
        self.register_operations_recursive(&structure.name, &structure.members);

        // Register extends clause (structure hierarchy for type promotion)
        if let Some(ref extends_type) = structure.extends_clause {
            // Extract parent structure name from type expression
            let parent_name = match extends_type {
                TypeExpr::Named(name) => name.clone(),
                TypeExpr::Parametric(name, _) => name.clone(),
                _ => format!("{:?}", extends_type), // Fallback for complex types
            };
            self.registry
                .register_extension(&structure.name, &parent_name);
        }

        let normalized = self.normalize_structure(structure)?;
        self.structures.insert(structure.name.clone(), normalized);
        Ok(())
    }

    /// Recursively register operations from structure members
    /// Handles nested structures by flattening their operations
    /// Grammar v0.6: Also registers function definitions as operations
    fn register_operations_recursive(&mut self, structure_name: &str, members: &[StructureMember]) {
        for member in members {
            match member {
                StructureMember::Operation { name, .. } => {
                    self.registry.register_operation(structure_name, name);
                }
                StructureMember::FunctionDef(func_def) => {
                    // Grammar v0.6: Register function as available operation
                    self.registry
                        .register_operation(structure_name, &func_def.name);
                }
                StructureMember::NestedStructure { members, .. } => {
                    // Recursively register operations from nested structure
                    self.register_operations_recursive(structure_name, members);
                }
                _ => {
                    // Field or Axiom - no operation to register
                }
            }
        }
    }

    fn register_implements(&mut self, impl_def: &ImplementsDef) -> Result<(), String> {
        // Find the structure this implements (validation check)
        let _structure = self
            .structures
            .get(&impl_def.structure_name)
            .ok_or_else(|| format!("Unknown structure: {}", impl_def.structure_name))?;

        // Validate where constraints if present
        if let Some(constraints) = &impl_def.where_clause {
            self.validate_where_constraints(constraints)?;
        }

        // Extract type name from type_args (use first arg for now, TODO: handle multiple)
        let type_name = if let Some(first_arg) = impl_def.type_args.first() {
            let norm = self.normalize_type_expr(first_arg)?;
            self.type_expr_to_string(&norm)
        } else {
            return Err(format!(
                "Implements block for {} has no type arguments",
                impl_def.structure_name
            ));
        };

        // Register each operation implementation
        for member in &impl_def.members {
            if let ImplMember::Operation {
                name,
                implementation,
            } = member
            {
                let impl_name = match implementation {
                    crate::kleis_ast::Implementation::Builtin(s) => s.clone(),
                    crate::kleis_ast::Implementation::Inline { .. } => {
                        format!("inline_{}", name)
                    }
                };

                self.registry.register_implementation(
                    &type_name,
                    &impl_def.structure_name,
                    name,
                    &impl_name,
                );
            }
        }

        self.implements.push(impl_def.clone());
        Ok(())
    }

    fn register_toplevel_operation(
        &mut self,
        _op_decl: &crate::kleis_ast::OperationDecl,
    ) -> Result<(), String> {
        // Top-level operations (like frac for display mode)
        // These are utility operations, not tied to structures
        // TODO: Register these separately if needed
        Ok(())
    }

    /// Validate where constraints in an implements block
    ///
    /// This checks that the constrained structures exist.
    /// Future: Could also check that the type arguments satisfy the constraints
    /// (e.g., verify with Z3 that T actually implements Semiring)
    fn validate_where_constraints(
        &self,
        constraints: &[crate::kleis_ast::WhereConstraint],
    ) -> Result<(), String> {
        for constraint in constraints {
            // Check that the constrained structure exists
            if !self.structures.contains_key(&constraint.structure_name) {
                return Err(format!(
                    "Unknown structure in where clause: {}",
                    constraint.structure_name
                ));
            }

            // TODO (future): Validate that type arguments actually satisfy the constraint
            // This would involve checking:
            // 1. Do the type arguments implement the required structure?
            // 2. If using Z3, verify axioms hold
            //
            // For now, we just check the structure exists (compile-time check)
            // Runtime/proof-time checking would be next phase
        }

        Ok(())
    }

    #[allow(clippy::only_used_in_recursion)]
    fn type_expr_to_string(&self, type_expr: &TypeExpr) -> String {
        match type_expr {
            TypeExpr::Named(s) => s.clone(),
            TypeExpr::Parametric(name, params) => {
                let params_str: Vec<String> =
                    params.iter().map(|p| self.type_expr_to_string(p)).collect();
                format!("{}({})", name, params_str.join(", "))
            }
            TypeExpr::Function(from, to) => {
                format!(
                    "{} → {}",
                    self.type_expr_to_string(from),
                    self.type_expr_to_string(to)
                )
            }
            TypeExpr::Product(types) => {
                let types_str: Vec<String> =
                    types.iter().map(|t| self.type_expr_to_string(t)).collect();
                format!("({})", types_str.join(" × "))
            }
            TypeExpr::Var(v) => v.clone(),
            TypeExpr::ForAll { vars, body } => {
                // Format: ∀(n : ℕ, T). Vector(n) → ℝ
                let vars_str: Vec<String> = vars
                    .iter()
                    .map(|(name, ty)| format!("{} : {}", name, self.type_expr_to_string(ty)))
                    .collect();
                format!(
                    "∀({}). {}",
                    vars_str.join(", "),
                    self.type_expr_to_string(body)
                )
            }
        }
    }

    /// Normalize a TypeExpr by expanding type aliases (with cycle guard)
    fn normalize_type_expr(&self, ty: &TypeExpr) -> Result<TypeExpr, String> {
        fn helper(
            ty: &TypeExpr,
            aliases: &HashMap<String, TypeExpr>,
            stack: &mut Vec<String>,
        ) -> Result<TypeExpr, String> {
            match ty {
                TypeExpr::Named(n) => {
                    if let Some(alias) = aliases.get(n) {
                        if stack.contains(n) {
                            return Err(format!("Cyclic type alias detected: {}", n));
                        }
                        stack.push(n.clone());
                        let expanded = helper(alias, aliases, stack)?;
                        stack.pop();
                        Ok(expanded)
                    } else {
                        Ok(TypeExpr::Named(n.clone()))
                    }
                }
                TypeExpr::Parametric(name, params) => {
                    let params_norm = params
                        .iter()
                        .map(|p| helper(p, aliases, stack))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(TypeExpr::Parametric(name.clone(), params_norm))
                }
                TypeExpr::Function(from, to) => {
                    let from_n = helper(from, aliases, stack)?;
                    let to_n = helper(to, aliases, stack)?;
                    Ok(TypeExpr::Function(Box::new(from_n), Box::new(to_n)))
                }
                TypeExpr::Product(types) => {
                    let types_norm = types
                        .iter()
                        .map(|t| helper(t, aliases, stack))
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(TypeExpr::Product(types_norm))
                }
                TypeExpr::Var(v) => Ok(TypeExpr::Var(v.clone())),
                TypeExpr::ForAll { vars, body } => {
                    let vars_norm = vars
                        .iter()
                        .map(|(n, t)| {
                            let nt = helper(t, aliases, stack)?;
                            Ok((n.clone(), nt))
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                    let body_norm = helper(body, aliases, stack)?;
                    Ok(TypeExpr::ForAll {
                        vars: vars_norm,
                        body: Box::new(body_norm),
                    })
                }
            }
        }

        let mut stack = Vec::new();
        helper(ty, &self.type_aliases, &mut stack)
    }

    /// Normalize all type expressions inside a structure definition
    fn normalize_structure(&self, structure: &StructureDef) -> Result<StructureDef, String> {
        let mut s = structure.clone();

        if let Some(ext) = &s.extends_clause {
            s.extends_clause = Some(self.normalize_type_expr(ext)?);
        }
        if let Some(over) = &s.over_clause {
            s.over_clause = Some(self.normalize_type_expr(over)?);
        }

        // Normalize members in place
        for member in s.members.iter_mut() {
            match member {
                StructureMember::Operation { type_signature, .. } => {
                    *type_signature = self.normalize_type_expr(type_signature)?;
                }
                StructureMember::Field { type_expr, .. } => {
                    *type_expr = self.normalize_type_expr(type_expr)?;
                }
                StructureMember::Axiom { .. } => {}
                StructureMember::NestedStructure {
                    structure_type,
                    members,
                    ..
                } => {
                    *structure_type = self.normalize_type_expr(structure_type)?;
                    // Recurse into nested members
                    let nested = StructureDef {
                        name: "".to_string(),
                        type_params: vec![],
                        members: members.clone(),
                        extends_clause: None,
                        over_clause: None,
                    };
                    let normalized = self.normalize_structure(&nested)?;
                    *members = normalized.members;
                }
                StructureMember::FunctionDef(func_def) => {
                    if let Some(annotation) = &func_def.type_annotation {
                        let norm = self.normalize_type_expr(annotation)?;
                        func_def.type_annotation = Some(norm);
                    }
                }
            }
        }

        Ok(s)
    }

    /// Get the operation registry
    pub fn registry(&self) -> &OperationRegistry {
        &self.registry
    }

    /// Get the type signature for an operation from its structure definition
    /// This is pure ADR-016: Read the signature from Kleis code!
    pub fn get_operation_signature(&self, op_name: &str) -> Option<&TypeExpr> {
        // Find which structure defines this operation
        let structure_name = self.registry.structure_for_operation(op_name)?;
        let structure = self.structures.get(structure_name)?;

        // Find the operation member
        for member in &structure.members {
            if let StructureMember::Operation {
                name,
                type_signature,
            } = member
            {
                if name == op_name {
                    return Some(type_signature);
                }
            }
        }

        None
    }

    /// Get the structure that defines an operation
    pub fn get_structure(&self, structure_name: &str) -> Option<&StructureDef> {
        self.structures.get(structure_name)
    }

    /// Helper: Check binary operation argument count
    fn check_binary_args(&self, op_name: &str, arg_types: &[Type]) -> Result<(), String> {
        if arg_types.len() != 2 {
            return Err(format!("{} requires 2 arguments", op_name));
        }
        Ok(())
    }

    /// Helper: Convert Type to type name for registry lookup
    #[allow(clippy::only_used_in_recursion)]
    fn type_to_name(&self, ty: &Type) -> Option<String> {
        match ty {
            // Bootstrap types
            Type::Nat => Some("Nat".to_string()),
            Type::NatValue(n) => Some(n.to_string()),
            Type::String => Some("String".to_string()),
            Type::StringValue(s) => Some(format!("\"{}\"", s)),
            Type::Bool => Some("Bool".to_string()),
            Type::Unit => Some("Unit".to_string()),

            // User-defined data types
            Type::Data {
                constructor, args, ..
            } => {
                if args.is_empty() {
                    // Map Scalar to ℝ for backward compatibility
                    if constructor == "Scalar" {
                        Some("ℝ".to_string())
                    } else {
                        Some(constructor.clone())
                    }
                } else {
                    // Construct type name with arguments
                    // Generic format works for Matrix, Vector, and any parametric type
                    let arg_names: Vec<String> = args
                        .iter()
                        .filter_map(|arg| self.type_to_name(arg))
                        .collect();

                    // General format: Constructor(arg1, arg2, ...)
                    // Examples: Matrix(2, 3, ℝ), Vector(3, ℝ), Tensor(2, 3, 4, ℝ)
                    Some(format!("{}({})", constructor, arg_names.join(", ")))
                }
            }

            // Meta-level types
            Type::Var(_) => None, // Type variables can't be validated (polymorphic)
            Type::ForAll(_, _) => None, // Polymorphic types handled elsewhere
        }
    }

    /// Validate that argument types implement the required structure
    /// This is the PROPER ADR-016 way: Check registry, not hardcoded types!
    fn validate_structure_implementation(
        &self,
        structure_name: &str,
        op_name: &str,
        arg_types: &[Type],
    ) -> Result<(), String> {
        for arg_type in arg_types {
            // Skip type variables (they're polymorphic, checked at instantiation)
            if matches!(arg_type, Type::Var(_)) {
                continue;
            }

            // Convert type to name for registry lookup
            let type_name = self.type_to_name(arg_type).ok_or_else(|| {
                format!(
                    "Cannot validate structure implementation for type: {:?}",
                    arg_type
                )
            })?;

            // Check if this type implements the required structure
            if !self.registry.supports_operation(&type_name, op_name) {
                // Get structures this type actually implements
                let type_structures = self.structures_for_type(&type_name);
                let available_ops: Vec<String> = type_structures
                    .iter()
                    .flat_map(|s| {
                        self.registry
                            .structure_to_operations
                            .get(s)
                            .cloned()
                            .unwrap_or_default()
                    })
                    .collect();

                return Err(format!(
                    "Type '{}' does not support operation '{}'.\n\
                     \n\
                     The operation '{}' is defined in structure '{}', \n\
                     but type '{}' does not implement this structure.\n\
                     \n\
                     Type '{}' implements: {}\n\
                     Available operations: {}",
                    type_name,
                    op_name,
                    op_name,
                    structure_name,
                    type_name,
                    type_name,
                    if type_structures.is_empty() {
                        "no structures".to_string()
                    } else {
                        type_structures.join(", ")
                    },
                    if available_ops.is_empty() {
                        "none".to_string()
                    } else {
                        available_ops.join(", ")
                    }
                ));
            }
        }
        Ok(())
    }

    /// Infer the type of an operation applied to given argument types
    /// This is the ADR-016 compliant way: query structures, don't hardcode!
    pub fn infer_operation_type(
        &self,
        op_name: &str,
        arg_types: &[Type],
        data_registry: &crate::data_registry::DataTypeRegistry,
    ) -> Result<Type, String> {
        // Handle formatting-only operations (display, not semantic)
        // These return the type of their first argument (the base)
        match op_name {
            // Display annotations
            "subsup" | "subscript" | "tilde" | "hat" | "bar" | "vec" | "dot" | "ddot" |
            "vector_arrow" | "vector_bold" | "overline" | "underline" |
            // Tensor index formatting (legacy palette operations)
            "index_mixed" | "index_pair" | "tensor_lower_pair" | 
            "tensor_1up_3down" | "tensor_2up_2down" | "tensor_upper_pair" => {
                // Formatting operations: type is the type of the base (first arg)
                if arg_types.is_empty() {
                    // No args - use Scalar data type
                    return Ok(Type::Data {
                        type_name: "Type".to_string(),
                        constructor: "Scalar".to_string(),
                        args: vec![],
                    });
                }
                return Ok(arg_types[0].clone());
            }
            _ => {}
        }

        // Query registry for operation
        if let Some(structure_name) = self.registry.structure_for_operation(op_name) {
            // Found the structure that defines this operation
            // Check if operation needs special handling or can use SignatureInterpreter directly

            match op_name {
                // Special semantics: equals returns RHS type (for definitions like I = Matrix(...))
                // This is SEMANTIC, not type-specific: The type of "x = y" is the type of y
                // This can't be expressed in a simple signature, so needs special handling
                "equals" | "not_equals" => {
                    self.check_binary_args(op_name, arg_types)?;
                    // For equals/not_equals, return the type of RHS (second argument)
                    // This handles definitions like: I = Matrix(2,2,...)
                    // Type of equation is the type of what's defined
                    Ok(arg_types[1].clone())
                }

                // Ordering operations: Validate that types implement Ordered
                // This is the GENERIC solution: Works for ANY user-defined type!
                "less_than" | "greater_than" | "less_equal" | "greater_equal" => {
                    self.check_binary_args(op_name, arg_types)?;

                    // GENERIC validation: Check registry for structure implementation
                    // Works for built-in types (ℝ, Matrix) AND user-defined types!
                    self.validate_structure_implementation(structure_name, op_name, arg_types)?;

                    // Then delegate to SignatureInterpreter for type inference
                    let structure = self
                        .get_structure(structure_name)
                        .ok_or_else(|| format!("Structure '{}' not found", structure_name))?;

                    let structure_registry = self.build_structure_registry();
                    SignatureInterpreter::new(data_registry.clone(), structure_registry)
                        .interpret_signature(structure, op_name, arg_types)
                }

                // ALL other operations use SignatureInterpreter
                _ => {
                    // Operation found in registry - try SignatureInterpreter as fallback!
                    // This is the ADR-016 ideal: Just interpret the signature from the structure
                    let structure = self
                        .get_structure(structure_name)
                        .ok_or_else(|| format!("Structure '{}' not found", structure_name))?;

                    let structure_registry = self.build_structure_registry();
                    let mut interpreter =
                        SignatureInterpreter::new(data_registry.clone(), structure_registry);
                    interpreter
                        .interpret_signature(structure, op_name, arg_types)
                        .map_err(|e| {
                            // Show actual error for debugging
                            format!(
                                "Operation '{}' found in structure '{}' but type inference failed: {}\n\
                                 This might mean the operation signature is complex or the structure\n\
                                 definition needs more information.",
                                op_name, structure_name, e
                            )
                        })
                }
            }
        } else {
            // Operation not in registry at all
            Err(format!(
                "Unknown operation: '{}'\n\
                 Hint: This operation is not defined in any loaded structure.\n\
                 Check stdlib or define it in a custom structure.",
                op_name
            ))
        }
    }

    /// Get the underlying type context
    pub fn context(&self) -> &TypeContext {
        &self.context
    }

    /// Check if a type supports an operation
    pub fn supports_operation(&self, type_name: &str, operation_name: &str) -> bool {
        self.registry.supports_operation(type_name, operation_name)
    }

    /// Get all types that support an operation
    pub fn types_supporting(&self, operation_name: &str) -> Vec<String> {
        self.registry.types_supporting(operation_name)
    }

    /// Get structures that a type implements
    pub fn structures_for_type(&self, type_name: &str) -> Vec<String> {
        self.registry
            .type_to_structures
            .get(type_name)
            .cloned()
            .unwrap_or_else(Vec::new)
    }

    /// Check if type From can be promoted to type To
    /// This queries the Promotes(From, To) structure implementations
    #[allow(dead_code)]
    pub fn can_promote(&self, from_type: &str, to_type: &str) -> bool {
        // The structure name would be "Promotes" with type arguments
        // Check if Promotes(from_type, to_type) is implemented
        let structure_key = format!("Promotes({}, {})", from_type, to_type);
        self.registry.type_to_structures.contains_key(&structure_key)
            || self.registry.type_to_structures.iter().any(|(_k, structs)| {
                // Check if any type has Promotes structure with matching args
                structs.iter().any(|s| {
                    s.starts_with("Promotes(")
                        && s.contains(from_type)
                        && s.contains(to_type)
                })
            })
    }

    /// Find the common supertype for two types using the Promotes hierarchy
    /// Returns the smallest type both can be promoted to
    /// Type hierarchy: ℕ → ℤ → ℚ → ℝ → ℂ
    pub fn find_common_supertype(&self, t1: &str, t2: &str) -> Option<String> {
        // If same type, that's the common type
        if t1 == t2 {
            return Some(t1.to_string());
        }

        // Normalize type names to canonical form
        fn normalize(t: &str) -> &'static str {
            match t {
                "ℕ" | "Nat" => "Nat",
                "ℤ" | "Int" => "Int",
                "ℚ" | "Rational" => "Rational",
                "ℝ" | "Scalar" | "Real" => "Scalar",
                "ℂ" | "Complex" => "Complex",
                _ => "Unknown",
            }
        }

        // Define the type hierarchy (order from smallest to largest)
        const HIERARCHY: [&str; 5] = ["Nat", "Int", "Rational", "Scalar", "Complex"];

        let t1_norm = normalize(t1);
        let t2_norm = normalize(t2);

        // Find positions in hierarchy
        let pos1 = HIERARCHY.iter().position(|&h| h == t1_norm);
        let pos2 = HIERARCHY.iter().position(|&h| h == t2_norm);

        match (pos1, pos2) {
            (Some(p1), Some(p2)) => {
                // Return the larger type (higher in hierarchy)
                let max_pos = p1.max(p2);
                Some(HIERARCHY[max_pos].to_string())
            }
            _ => None, // One or both types not in hierarchy
        }
    }

    /// Generate helpful error message when operation not supported
    pub fn suggest_operation(&self, type_name: &str, attempted_operation: &str) -> Option<String> {
        // Get structures this type implements
        let structures = self.structures_for_type(type_name);

        if structures.is_empty() {
            return Some(format!("{} doesn't implement any structures", type_name));
        }

        // Get operations available for these structures
        let mut available_ops = Vec::new();
        for structure in &structures {
            if let Some(ops) = self.registry.structure_to_operations.get(structure) {
                available_ops.extend(ops.clone());
            }
        }

        if available_ops.is_empty() {
            return Some(format!(
                "{} implements {:?} but no operations available",
                type_name, structures
            ));
        }

        // Suggest similar operation
        // Simple: suggest first available operation
        // TODO: Use edit distance to find most similar
        Some(format!(
            "{} doesn't support '{}'. Available operations: {}. Try: {}?",
            type_name,
            attempted_operation,
            available_ops.join(", "),
            available_ops[0]
        ))
    }

    /// Build a StructureRegistry from loaded structures
    /// This enables generic handling of parametric structure types in signatures
    pub fn build_structure_registry(&self) -> crate::structure_registry::StructureRegistry {
        use crate::structure_registry::StructureRegistry;
        let mut registry = StructureRegistry::new();

        for structure in self.structures.values() {
            // Only register parametric structures (those with type parameters)
            // Non-parametric structures don't need registry lookup
            if !structure.type_params.is_empty() {
                if let Err(e) = registry.register(structure.clone()) {
                    eprintln!(
                        "Warning: Failed to register structure '{}': {}",
                        structure.name, e
                    );
                }
            }
        }

        // Also register implements blocks (for where constraint tracking)
        for impl_def in &self.implements {
            registry.register_implements(impl_def.clone());
        }

        registry
    }
}

impl Default for TypeContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kleis_ast::Implementation;
    use crate::kleis_parser::parse_kleis_program;

    fn builder_with_aliases(map: &[(&str, TypeExpr)]) -> TypeContextBuilder {
        let mut b = TypeContextBuilder::new();
        for (k, v) in map {
            b.type_aliases.insert((*k).to_string(), v.clone());
        }
        b
    }

    #[test]
    fn test_build_context_from_numeric() {
        let code = r#"
            structure Numeric(N) {
                operation abs : N → N
            }
            
            implements Numeric(ℝ) {
                operation abs = builtin_abs
            }
        "#;

        let program = parse_kleis_program(code).unwrap();
        let builder = TypeContextBuilder::from_program(program).unwrap();

        // Check registry
        assert!(builder.supports_operation("ℝ", "abs"));
        assert!(!builder.supports_operation("ℝ", "card"));

        // Check types supporting abs
        let types = builder.types_supporting("abs");
        assert_eq!(types.len(), 1);
        assert_eq!(types[0], "ℝ");
    }

    #[test]
    fn test_polymorphic_operation() {
        let code = r#"
            structure Numeric(N) {
                operation abs : N → N
            }
            
            implements Numeric(ℝ) {
                operation abs = builtin_abs_real
            }
            
            implements Numeric(ℂ) {
                operation abs = complex_modulus
            }
        "#;

        let program = parse_kleis_program(code).unwrap();
        let builder = TypeContextBuilder::from_program(program).unwrap();

        // abs works for both ℝ and ℂ
        assert!(builder.supports_operation("ℝ", "abs"));
        assert!(builder.supports_operation("ℂ", "abs"));

        // Both types support abs
        let types = builder.types_supporting("abs");
        assert_eq!(types.len(), 2);
        assert!(types.contains(&"ℝ".to_string()));
        assert!(types.contains(&"ℂ".to_string()));
    }

    #[test]
    fn test_multiple_structures() {
        let code = r#"
            structure Numeric(N) {
                operation abs : N → N
            }
            
            structure Finite(C) {
                operation card : C → ℕ
            }
            
            implements Numeric(ℝ) {
                operation abs = builtin_abs
            }
            
            implements Finite(Set(T)) {
                operation card = builtin_card
            }
        "#;

        let program = parse_kleis_program(code).unwrap();
        let builder = TypeContextBuilder::from_program(program).unwrap();

        // ℝ supports abs but not card
        assert!(builder.supports_operation("ℝ", "abs"));
        assert!(!builder.supports_operation("ℝ", "card"));

        // Set supports card but not abs
        assert!(builder.supports_operation("Set(T)", "card"));
        assert!(!builder.supports_operation("Set(T)", "abs"));
    }

    #[test]
    fn test_error_suggestions() {
        let code = r#"
            structure Numeric(N) {
                operation abs : N → N
            }
            
            structure Finite(C) {
                operation card : C → ℕ
            }
            
            implements Numeric(ℝ) {
                operation abs = builtin_abs
            }
            
            implements Finite(Set(T)) {
                operation card = builtin_card
            }
        "#;

        let program = parse_kleis_program(code).unwrap();
        let builder = TypeContextBuilder::from_program(program).unwrap();

        // Try to use card on ℝ (wrong!)
        let suggestion = builder.suggest_operation("ℝ", "card");
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("abs")); // Should suggest abs

        // Try to use abs on Set (wrong!)
        let suggestion = builder.suggest_operation("Set(T)", "abs");
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("card")); // Should suggest card
    }

    #[test]
    fn normalize_simple_alias() {
        let b = builder_with_aliases(&[("Real", TypeExpr::Named("ℝ".to_string()))]);
        let norm = b
            .normalize_type_expr(&TypeExpr::Named("Real".to_string()))
            .unwrap();
        assert_eq!(norm, TypeExpr::Named("ℝ".to_string()));
    }

    #[test]
    fn normalize_parametric_alias_chain() {
        let b = builder_with_aliases(&[
            ("Real", TypeExpr::Named("ℝ".to_string())),
            (
                "VecReal",
                TypeExpr::Parametric(
                    "Vector".to_string(),
                    vec![TypeExpr::Named("Real".to_string())],
                ),
            ),
        ]);
        let norm = b
            .normalize_type_expr(&TypeExpr::Named("VecReal".to_string()))
            .unwrap();
        assert_eq!(
            norm,
            TypeExpr::Parametric("Vector".to_string(), vec![TypeExpr::Named("ℝ".to_string())])
        );
    }

    #[test]
    fn normalize_detects_cycle() {
        let mut b = TypeContextBuilder::new();
        b.type_aliases
            .insert("A".to_string(), TypeExpr::Named("B".to_string()));
        b.type_aliases
            .insert("B".to_string(), TypeExpr::Named("A".to_string()));
        let err = b.normalize_type_expr(&TypeExpr::Named("A".to_string()));
        assert!(err.is_err());
    }

    #[test]
    fn implements_uses_normalized_type_name() {
        // alias Real = ℝ; implements Numeric(Real) { operation abs = builtin_abs }
        let mut b = builder_with_aliases(&[("Real", TypeExpr::Named("ℝ".to_string()))]);

        // minimal structure placeholder so register_implements can validate
        let numeric = StructureDef {
            name: "Numeric".to_string(),
            type_params: vec![],
            members: vec![],
            extends_clause: None,
            over_clause: None,
        };
        b.structures.insert("Numeric".to_string(), numeric);

        let impl_def = ImplementsDef {
            structure_name: "Numeric".to_string(),
            type_args: vec![TypeExpr::Named("Real".to_string())],
            members: vec![ImplMember::Operation {
                name: "abs".to_string(),
                implementation: Implementation::Builtin("builtin_abs".to_string()),
            }],
            over_clause: None,
            where_clause: None,
        };

        b.register_implements(&impl_def).unwrap();

        // Registry should store the expanded type name (ℝ) implementing Numeric
        let structures = b.registry.type_to_structures.get("ℝ");
        assert!(structures.is_some());
        assert!(structures.unwrap().contains(&"Numeric".to_string()));
    }
}
