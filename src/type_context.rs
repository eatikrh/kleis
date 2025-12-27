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
    ImplMember, ImplementsDef, Program, StructureDef, StructureMember, TopLevel, TypeAliasParam,
    TypeExpr,
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

    /// Type promotion graph: Maps (FromType, ToType) → lift function name
    /// Populated from `implements Promotes(From, To) { operation lift = ... }`
    /// Example: ("Int", "Rational") → "int_to_rational"
    type_promotions: HashMap<(String, String), String>,

    /// Top-level operation type signatures
    /// Populated from `operation sin : ℝ → ℝ` at file scope
    /// Example: "sin" → (vec!["Scalar"], "Scalar")  // (arg types, return type)
    toplevel_operation_types: HashMap<String, TypeExpr>,
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
            type_promotions: HashMap::new(),
            toplevel_operation_types: HashMap::new(),
        }
    }

    /// Register a type promotion path
    /// Example: register_promotion("Int", "Rational", "int_to_rational")
    pub fn register_promotion(&mut self, from: &str, to: &str, lift_fn: &str) {
        self.type_promotions
            .insert((from.to_string(), to.to_string()), lift_fn.to_string());
    }

    /// Get lift function for a direct promotion (if registered)
    pub fn get_promotion(&self, from: &str, to: &str) -> Option<&String> {
        self.type_promotions
            .get(&(from.to_string(), to.to_string()))
    }

    /// Check if a direct promotion exists
    pub fn has_promotion(&self, from: &str, to: &str) -> bool {
        self.type_promotions
            .contains_key(&(from.to_string(), to.to_string()))
    }

    /// Get all types that `from_type` can be directly promoted to
    pub fn get_promotion_targets(&self, from_type: &str) -> Vec<String> {
        self.type_promotions
            .keys()
            .filter(|(from, _)| from == from_type)
            .map(|(_, to)| to.clone())
            .collect()
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

    /// Register a top-level operation with its type signature
    /// Example: register_toplevel_operation("sin", TypeExpr::Function(Scalar, Scalar))
    pub fn register_toplevel_operation(&mut self, name: &str, type_sig: TypeExpr) {
        self.toplevel_operation_types
            .insert(name.to_string(), type_sig);
    }

    /// Get the type signature for a top-level operation
    pub fn get_toplevel_operation_type(&self, name: &str) -> Option<&TypeExpr> {
        self.toplevel_operation_types.get(name)
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

        // Merge structure_extends
        for (child, parent) in other.structure_extends {
            self.structure_extends.insert(child, parent);
        }

        // Merge type_promotions (Promotes implementations)
        for ((from, to), lift_fn) in other.type_promotions {
            self.type_promotions.insert((from, to), lift_fn);
        }

        // Merge toplevel_operation_types
        for (name, type_sig) in other.toplevel_operation_types {
            self.toplevel_operation_types.insert(name, type_sig);
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

    /// Type aliases: name -> (parameters, underlying type expression)
    /// v0.91: Supports parameterized type aliases like ComplexMatrix(m, n)
    type_aliases: HashMap<String, (Vec<TypeAliasParam>, TypeExpr)>,
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

        // Merge aliases (v0.91: with parameters)
        for (name, alias_def) in other.type_aliases {
            self.type_aliases.entry(name).or_insert(alias_def);
        }

        // Context merging is not needed (it's ephemeral)

        Ok(())
    }

    /// Build type context from a parsed program
    pub fn from_program(program: Program) -> Result<Self, String> {
        let mut builder = Self::new();

        // Phase 0: Register type aliases (v0.91: with parameters)
        for item in &program.items {
            if let TopLevel::TypeAlias(alias) = item {
                builder.type_aliases.insert(
                    alias.name.clone(),
                    (alias.params.clone(), alias.type_expr.clone()),
                );
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
        // Special handling for Promotes(From, To) - register type promotion
        if impl_def.structure_name == "Promotes" && impl_def.type_args.len() == 2 {
            let from_type_raw =
                self.type_expr_to_string(&self.normalize_type_expr(&impl_def.type_args[0])?);
            let to_type_raw =
                self.type_expr_to_string(&self.normalize_type_expr(&impl_def.type_args[1])?);

            // Normalize Unicode type names to canonical ASCII names
            // This ensures ℕ→ℤ is registered as Nat→Int
            let from_type = Self::normalize_type_name(&from_type_raw).to_string();
            let to_type = Self::normalize_type_name(&to_type_raw).to_string();

            // Find the lift operation implementation
            for member in &impl_def.members {
                if let ImplMember::Operation {
                    name,
                    implementation,
                } = member
                {
                    if name == "lift" {
                        let lift_fn = match implementation {
                            crate::kleis_ast::Implementation::Builtin(s) => s.clone(),
                            crate::kleis_ast::Implementation::Inline { .. } => {
                                format!(
                                    "{}_to_{}",
                                    from_type.to_lowercase(),
                                    to_type.to_lowercase()
                                )
                            }
                        };
                        self.registry
                            .register_promotion(&from_type, &to_type, &lift_fn);
                    }
                }
            }

            self.implements.push(impl_def.clone());
            return Ok(());
        }

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
        op_decl: &crate::kleis_ast::OperationDecl,
    ) -> Result<(), String> {
        // Top-level operations (like sin, cos, frac)
        // Register their type signatures for type inference
        self.registry
            .register_toplevel_operation(&op_decl.name, op_decl.type_signature.clone());
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
            TypeExpr::DimExpr(dim) => dim.to_string(),
        }
    }

    /// Normalize a TypeExpr by expanding type aliases (with cycle guard)
    /// Normalize a type expression by expanding type aliases
    ///
    /// v0.91: Supports parameterized type aliases
    /// - Simple alias: `type Real = ℝ` → `Real` expands to `ℝ`
    /// - Parameterized: `type ComplexMatrix(m, n) = (Matrix(m,n,ℝ), Matrix(m,n,ℝ))`
    ///   → `ComplexMatrix(2, 3)` expands to `(Matrix(2,3,ℝ), Matrix(2,3,ℝ))`
    fn normalize_type_expr(&self, ty: &TypeExpr) -> Result<TypeExpr, String> {
        fn substitute(
            ty: &TypeExpr,
            param_names: &[String],
            param_values: &[TypeExpr],
        ) -> TypeExpr {
            match ty {
                TypeExpr::Named(n) => {
                    // Check if this name is a parameter to substitute
                    for (i, param_name) in param_names.iter().enumerate() {
                        if n == param_name {
                            return param_values[i].clone();
                        }
                    }
                    TypeExpr::Named(n.clone())
                }
                TypeExpr::Var(v) => {
                    // Variable might also be a parameter
                    for (i, param_name) in param_names.iter().enumerate() {
                        if v == param_name {
                            return param_values[i].clone();
                        }
                    }
                    TypeExpr::Var(v.clone())
                }
                TypeExpr::Parametric(name, params) => {
                    let params_subst: Vec<TypeExpr> = params
                        .iter()
                        .map(|p| substitute(p, param_names, param_values))
                        .collect();
                    TypeExpr::Parametric(name.clone(), params_subst)
                }
                TypeExpr::Function(from, to) => TypeExpr::Function(
                    Box::new(substitute(from, param_names, param_values)),
                    Box::new(substitute(to, param_names, param_values)),
                ),
                TypeExpr::Product(types) => {
                    let types_subst: Vec<TypeExpr> = types
                        .iter()
                        .map(|t| substitute(t, param_names, param_values))
                        .collect();
                    TypeExpr::Product(types_subst)
                }
                TypeExpr::ForAll { vars, body } => {
                    let vars_subst: Vec<(String, TypeExpr)> = vars
                        .iter()
                        .map(|(n, t)| (n.clone(), substitute(t, param_names, param_values)))
                        .collect();
                    TypeExpr::ForAll {
                        vars: vars_subst,
                        body: Box::new(substitute(body, param_names, param_values)),
                    }
                }
                // v0.92: DimExpr is a dimension expression, handle variable substitution
                TypeExpr::DimExpr(dim) => {
                    TypeExpr::DimExpr(substitute_dim_expr(dim, param_names, param_values))
                }
            }
        }

        fn substitute_dim_expr(
            dim: &crate::kleis_ast::DimExpr,
            param_names: &[String],
            param_values: &[TypeExpr],
        ) -> crate::kleis_ast::DimExpr {
            use crate::kleis_ast::DimExpr;
            match dim {
                DimExpr::Lit(n) => DimExpr::Lit(*n),
                DimExpr::Var(v) => {
                    // Check if this variable is a parameter to substitute
                    for (i, param_name) in param_names.iter().enumerate() {
                        if v == param_name {
                            // Convert TypeExpr to DimExpr if possible
                            if let TypeExpr::Named(name) = &param_values[i] {
                                if let Ok(n) = name.parse::<usize>() {
                                    return DimExpr::Lit(n);
                                } else {
                                    return DimExpr::Var(name.clone());
                                }
                            } else if let TypeExpr::DimExpr(d) = &param_values[i] {
                                return d.clone();
                            }
                        }
                    }
                    DimExpr::Var(v.clone())
                }
                DimExpr::Add(left, right) => DimExpr::Add(
                    Box::new(substitute_dim_expr(left, param_names, param_values)),
                    Box::new(substitute_dim_expr(right, param_names, param_values)),
                ),
                DimExpr::Sub(left, right) => DimExpr::Sub(
                    Box::new(substitute_dim_expr(left, param_names, param_values)),
                    Box::new(substitute_dim_expr(right, param_names, param_values)),
                ),
                DimExpr::Mul(left, right) => DimExpr::Mul(
                    Box::new(substitute_dim_expr(left, param_names, param_values)),
                    Box::new(substitute_dim_expr(right, param_names, param_values)),
                ),
                DimExpr::Div(left, right) => DimExpr::Div(
                    Box::new(substitute_dim_expr(left, param_names, param_values)),
                    Box::new(substitute_dim_expr(right, param_names, param_values)),
                ),
                DimExpr::Pow(left, right) => DimExpr::Pow(
                    Box::new(substitute_dim_expr(left, param_names, param_values)),
                    Box::new(substitute_dim_expr(right, param_names, param_values)),
                ),
                DimExpr::Call(name, args) => {
                    let args_subst: Vec<DimExpr> = args
                        .iter()
                        .map(|a| substitute_dim_expr(a, param_names, param_values))
                        .collect();
                    DimExpr::Call(name.clone(), args_subst)
                }
            }
        }

        fn helper(
            ty: &TypeExpr,
            aliases: &HashMap<String, (Vec<TypeAliasParam>, TypeExpr)>,
            stack: &mut Vec<String>,
        ) -> Result<TypeExpr, String> {
            match ty {
                TypeExpr::Named(n) => {
                    if let Some((params, body)) = aliases.get(n) {
                        // Only expand if it's a non-parameterized alias
                        if params.is_empty() {
                            if stack.contains(n) {
                                return Err(format!("Cyclic type alias detected: {}", n));
                            }
                            stack.push(n.clone());
                            let expanded = helper(body, aliases, stack)?;
                            stack.pop();
                            Ok(expanded)
                        } else {
                            // Parameterized alias used without args - error or keep as-is
                            // For now, keep as-is (user might be referencing the type constructor)
                            Ok(TypeExpr::Named(n.clone()))
                        }
                    } else {
                        Ok(TypeExpr::Named(n.clone()))
                    }
                }
                TypeExpr::Parametric(name, args) => {
                    // First normalize the arguments
                    let args_norm: Vec<TypeExpr> = args
                        .iter()
                        .map(|p| helper(p, aliases, stack))
                        .collect::<Result<Vec<_>, _>>()?;

                    // Check if this is a parameterized type alias
                    if let Some((params, body)) = aliases.get(name) {
                        if !params.is_empty() {
                            // This is a parameterized alias - substitute!
                            if args_norm.len() != params.len() {
                                return Err(format!(
                                    "Type alias {} expects {} arguments, got {}",
                                    name,
                                    params.len(),
                                    args_norm.len()
                                ));
                            }
                            if stack.contains(name) {
                                return Err(format!("Cyclic type alias detected: {}", name));
                            }
                            stack.push(name.clone());

                            // Extract parameter names
                            let param_names: Vec<String> =
                                params.iter().map(|p| p.name.clone()).collect();

                            // Substitute parameters in the body
                            let substituted = substitute(body, &param_names, &args_norm);

                            // Recursively normalize the result
                            let expanded = helper(&substituted, aliases, stack)?;
                            stack.pop();
                            return Ok(expanded);
                        }
                    }

                    // Not a parameterized alias - keep as parametric type
                    Ok(TypeExpr::Parametric(name.clone(), args_norm))
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
                // v0.92: DimExpr doesn't need normalization - just pass through
                TypeExpr::DimExpr(dim) => Ok(TypeExpr::DimExpr(dim.clone())),
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

            // Symbolic dimension expression
            Type::NatExpr(dim) => Some(format!("{}", dim)),

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

        // Check for top-level operations (like sin : ℝ → ℝ)
        if let Some(type_sig) = self.registry.get_toplevel_operation_type(op_name) {
            // Found a top-level operation - interpret its type signature
            return self.interpret_toplevel_operation_type(type_sig, arg_types);
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

    /// Interpret a top-level operation's type signature
    ///
    /// For operations like `operation sin : ℝ → ℝ`, this extracts the return type.
    /// Handles:
    /// - Simple function types: `ℝ → ℝ` returns `ℝ`
    /// - Multi-arg functions: `ℝ × ℝ → ℝ` returns `ℝ`
    fn interpret_toplevel_operation_type(
        &self,
        type_sig: &TypeExpr,
        _arg_types: &[Type],
    ) -> Result<Type, String> {
        match type_sig {
            TypeExpr::Function(_from, to) => {
                // Function type: return the codomain
                self.type_expr_to_type(to)
            }
            _ => {
                // Non-function type: return as-is (e.g., constant operations)
                self.type_expr_to_type(type_sig)
            }
        }
    }

    /// Convert a TypeExpr to a Type (for return type extraction)
    fn type_expr_to_type(&self, type_expr: &TypeExpr) -> Result<Type, String> {
        match type_expr {
            TypeExpr::Named(name) => {
                // Normalize Unicode names
                let normalized = Self::normalize_type_name(name);
                Ok(Type::Data {
                    type_name: "Type".to_string(),
                    constructor: normalized.to_string(),
                    args: vec![],
                })
            }
            TypeExpr::Parametric(name, params) => {
                let param_types: Result<Vec<Type>, String> =
                    params.iter().map(|p| self.type_expr_to_type(p)).collect();
                Ok(Type::Data {
                    type_name: "Type".to_string(),
                    constructor: name.clone(),
                    args: param_types?,
                })
            }
            TypeExpr::Function(_from, to) => {
                // For now, return the codomain for function types
                // Full function type support would need Type::Function
                self.type_expr_to_type(to)
            }
            TypeExpr::Product(types) => {
                if types.len() == 1 {
                    self.type_expr_to_type(&types[0])
                } else {
                    // Product type - for now return first type
                    // TODO: Proper tuple/product type support
                    self.type_expr_to_type(&types[0])
                }
            }
            TypeExpr::Var(_) => {
                // Type variable - return as fresh variable
                Ok(Type::Var(crate::type_inference::TypeVar(0)))
            }
            TypeExpr::ForAll { body, .. } => {
                // Quantified type - unwrap and interpret the body
                self.type_expr_to_type(body)
            }
            TypeExpr::DimExpr(dim) => Ok(Type::NatExpr(dim.clone())),
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

    /// Get all operations defined by a structure
    pub fn operations_for_structure(&self, structure_name: &str) -> Vec<String> {
        self.registry
            .structure_to_operations
            .get(structure_name)
            .cloned()
            .unwrap_or_else(Vec::new)
    }

    /// Get all operations available for a type (via its implemented structures)
    pub fn operations_for_type(&self, type_name: &str) -> Vec<String> {
        let mut ops = Vec::new();
        if let Some(structures) = self.registry.type_to_structures.get(type_name) {
            for structure in structures {
                if let Some(structure_ops) = self.registry.structure_to_operations.get(structure) {
                    for op in structure_ops {
                        if !ops.contains(op) {
                            ops.push(op.clone());
                        }
                    }
                }
            }
        }
        ops
    }

    /// Check if a type supports any operation (i.e., implements at least one structure)
    pub fn supports_any_operation(&self, type_name: &str) -> bool {
        self.registry
            .type_to_structures
            .get(type_name)
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    }

    /// Get the signature of an operation as a string (for display)
    pub fn operation_signature(&self, operation_name: &str) -> Option<String> {
        if let Some(type_expr) = self.get_operation_signature(operation_name) {
            Some(format!(
                "operation {} : {}",
                operation_name,
                self.type_expr_to_string(type_expr)
            ))
        } else {
            // Check if it's known in the registry
            self.registry
                .structure_for_operation(operation_name)
                .map(|structure| format!("operation {} (from {})", operation_name, structure))
        }
    }

    /// Get all structure names (for LSP completions)
    pub fn all_structure_names(&self) -> Vec<String> {
        self.structures.keys().cloned().collect()
    }

    /// Get all operation names (for LSP completions)
    pub fn all_operation_names(&self) -> Vec<String> {
        self.registry
            .operation_to_structure
            .keys()
            .cloned()
            .collect()
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
        self.registry
            .type_to_structures
            .contains_key(&structure_key)
            || self
                .registry
                .type_to_structures
                .iter()
                .any(|(_k, structs)| {
                    // Check if any type has Promotes structure with matching args
                    structs.iter().any(|s| {
                        s.starts_with("Promotes(") && s.contains(from_type) && s.contains(to_type)
                    })
                })
    }

    /// Normalize type name to canonical form
    fn normalize_type_name(t: &str) -> &str {
        match t {
            "ℕ" | "Nat" => "Nat",
            "ℤ" | "Int" => "Int",
            "ℚ" | "Rational" => "Rational",
            "ℝ" | "Scalar" | "Real" => "Scalar",
            "ℂ" | "Complex" => "Complex",
            _ => t,
        }
    }

    /// Find the common supertype for two types using the Promotes hierarchy
    /// Returns the smallest type both can be promoted to
    ///
    /// Strategy:
    /// 1. First try registry-based lookup (user-defined promotions)
    /// 2. Fall back to built-in numeric hierarchy
    pub fn find_common_supertype(&self, t1: &str, t2: &str) -> Option<String> {
        let t1_norm = Self::normalize_type_name(t1);
        let t2_norm = Self::normalize_type_name(t2);

        // If same type, that's the common type
        if t1_norm == t2_norm {
            return Some(t1_norm.to_string());
        }

        // Try registry first: check if t1 → t2 or t2 → t1 is registered
        if self.registry.has_promotion(t1_norm, t2_norm) {
            return Some(t2_norm.to_string());
        }
        if self.registry.has_promotion(t2_norm, t1_norm) {
            return Some(t1_norm.to_string());
        }

        // Try to find common ancestor via BFS on promotion graph
        if let Some(common) = self.find_common_ancestor_bfs(t1_norm, t2_norm) {
            return Some(common);
        }

        // Fall back to built-in numeric hierarchy
        const HIERARCHY: [&str; 5] = ["Nat", "Int", "Rational", "Scalar", "Complex"];

        let pos1 = HIERARCHY.iter().position(|&h| h == t1_norm);
        let pos2 = HIERARCHY.iter().position(|&h| h == t2_norm);

        match (pos1, pos2) {
            (Some(p1), Some(p2)) => {
                let max_pos = p1.max(p2);
                Some(HIERARCHY[max_pos].to_string())
            }
            _ => None,
        }
    }

    /// Find common ancestor using BFS on the promotion graph
    fn find_common_ancestor_bfs(&self, t1: &str, t2: &str) -> Option<String> {
        use std::collections::{HashSet, VecDeque};

        // Get all types reachable from t1
        let mut reachable_from_t1: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        queue.push_back(t1.to_string());
        reachable_from_t1.insert(t1.to_string());

        while let Some(current) = queue.pop_front() {
            for target in self.registry.get_promotion_targets(&current) {
                if !reachable_from_t1.contains(&target) {
                    reachable_from_t1.insert(target.clone());
                    queue.push_back(target);
                }
            }
        }

        // BFS from t2, find first type also reachable from t1
        queue.clear();
        let mut visited: HashSet<String> = HashSet::new();
        queue.push_back(t2.to_string());
        visited.insert(t2.to_string());

        // Check t2 itself
        if reachable_from_t1.contains(t2) {
            return Some(t2.to_string());
        }

        while let Some(current) = queue.pop_front() {
            for target in self.registry.get_promotion_targets(&current) {
                if reachable_from_t1.contains(&target) {
                    return Some(target);
                }
                if !visited.contains(&target) {
                    visited.insert(target.clone());
                    queue.push_back(target);
                }
            }
        }

        None
    }

    /// Get the lift function name for promoting from one type to another
    ///
    /// Strategy:
    /// 1. First try registry-based lookup (user-defined promotions)
    /// 2. Try multi-step composition via registry
    /// 3. Fall back to built-in lift functions
    pub fn get_lift_function(&self, from_type: &str, to_type: &str) -> Option<String> {
        let from = Self::normalize_type_name(from_type);
        let to = Self::normalize_type_name(to_type);

        // If same type, no lift needed
        if from == to {
            return None;
        }

        // Try registry first (direct promotion)
        if let Some(lift_fn) = self.registry.get_promotion(from, to) {
            return Some(lift_fn.clone());
        }

        // Try to find multi-step promotion path and compose lifts
        let chain = self.get_lift_chain(from, to);
        if !chain.is_empty() {
            // For single-step, return the function directly
            if chain.len() == 1 {
                return Some(chain[0].clone());
            }
            // For multi-step, return a composed lift marker
            // Format: "compose_lifts:fn1,fn2,fn3" which lowering will parse
            return Some(format!("compose_lifts:{}", chain.join(",")));
        }

        // Fall back to built-in lift functions for numeric types
        match (from, to) {
            ("Nat", "Int") => Some("nat_to_int".to_string()),
            ("Int", "Rational") => Some("int_to_rational".to_string()),
            ("Rational", "Scalar") => Some("rational_to_real".to_string()),
            ("Scalar", "Complex") => Some("real_to_complex".to_string()),
            ("Nat", "Rational") => Some("nat_to_rational".to_string()),
            ("Nat", "Scalar") => Some("nat_to_real".to_string()),
            ("Nat", "Complex") => Some("nat_to_complex".to_string()),
            ("Int", "Scalar") => Some("int_to_real".to_string()),
            ("Int", "Complex") => Some("int_to_complex".to_string()),
            ("Rational", "Complex") => Some("rational_to_complex".to_string()),
            _ => None,
        }
    }

    /// Get the chain of lift functions needed to promote from one type to another
    ///
    /// Returns a vector of lift function names in order of application.
    /// E.g., Int → Complex might return ["int_to_real", "real_to_complex"]
    /// meaning: real_to_complex(int_to_real(x))
    pub fn get_lift_chain(&self, from_type: &str, to_type: &str) -> Vec<String> {
        let from = Self::normalize_type_name(from_type);
        let to = Self::normalize_type_name(to_type);

        if from == to {
            return vec![];
        }

        // Try to find promotion path in registry
        if let Some(path) = self.find_promotion_path(from, to) {
            let mut chain = Vec::new();
            let mut current = from.to_string();

            for next in &path {
                if let Some(lift_fn) = self.registry.get_promotion(&current, next) {
                    chain.push(lift_fn.clone());
                } else {
                    // Gap in registry - try fallback for this step
                    if let Some(fallback) = self.get_builtin_lift(&current, next) {
                        chain.push(fallback);
                    } else {
                        // Can't complete the chain
                        return vec![];
                    }
                }
                current = next.clone();
            }

            return chain;
        }

        // No path found in registry, try direct built-in
        if let Some(direct) = self.get_builtin_lift(from, to) {
            return vec![direct];
        }

        vec![]
    }

    /// Get built-in lift function for a single step
    fn get_builtin_lift(&self, from: &str, to: &str) -> Option<String> {
        match (from, to) {
            ("Nat", "Int") => Some("nat_to_int".to_string()),
            ("Int", "Rational") => Some("int_to_rational".to_string()),
            ("Rational", "Scalar") => Some("rational_to_real".to_string()),
            ("Scalar", "Complex") => Some("real_to_complex".to_string()),
            ("Nat", "Rational") => Some("nat_to_rational".to_string()),
            ("Nat", "Scalar") => Some("nat_to_real".to_string()),
            ("Nat", "Complex") => Some("nat_to_complex".to_string()),
            ("Int", "Scalar") => Some("int_to_real".to_string()),
            ("Int", "Complex") => Some("int_to_complex".to_string()),
            ("Rational", "Complex") => Some("rational_to_complex".to_string()),
            _ => None,
        }
    }

    /// Find a promotion path from one type to another using BFS
    fn find_promotion_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        use std::collections::{HashMap, VecDeque};

        if from == to {
            return Some(vec![]);
        }

        let mut queue: VecDeque<String> = VecDeque::new();
        let mut came_from: HashMap<String, String> = HashMap::new();

        queue.push_back(from.to_string());
        came_from.insert(from.to_string(), String::new());

        while let Some(current) = queue.pop_front() {
            for target in self.registry.get_promotion_targets(&current) {
                if !came_from.contains_key(&target) {
                    came_from.insert(target.clone(), current.clone());

                    if target == to {
                        // Reconstruct path
                        let mut path = vec![];
                        let mut node = to.to_string();
                        while node != from {
                            path.push(node.clone());
                            node = came_from.get(&node).unwrap().clone();
                        }
                        path.reverse();
                        return Some(path);
                    }

                    queue.push_back(target);
                }
            }
        }

        None
    }

    /// Get the type-specific operation name for a given generic operation and target type
    /// e.g., ("plus", "Complex") → "complex_add"
    pub fn get_lowered_op_name(&self, generic_op: &str, target_type: &str) -> String {
        fn normalize(t: &str) -> &str {
            match t {
                "ℕ" | "Nat" => "Nat",
                "ℤ" | "Int" => "Int",
                "ℚ" | "Rational" => "Rational",
                "ℝ" | "Scalar" | "Real" => "Scalar",
                "ℂ" | "Complex" => "Complex",
                _ => t,
            }
        }

        let ty = normalize(target_type);

        match (generic_op, ty) {
            // Complex operations
            ("plus", "Complex") => "complex_add".to_string(),
            ("minus", "Complex") => "complex_sub".to_string(),
            ("times", "Complex") => "complex_mul".to_string(),
            ("divide" | "scalar_divide", "Complex") => "complex_div".to_string(),
            ("negate" | "neg", "Complex") => "neg_complex".to_string(),

            // Rational operations
            ("plus", "Rational") => "rational_add".to_string(),
            ("minus", "Rational") => "rational_sub".to_string(),
            ("times", "Rational") => "rational_mul".to_string(),
            ("divide" | "scalar_divide", "Rational") => "rational_div".to_string(),
            ("negate" | "neg", "Rational") => "neg_rational".to_string(),

            // Default: keep original operation name for Scalar, Int, Nat
            _ => generic_op.to_string(),
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

    /// Helper to create a builder with simple (non-parameterized) type aliases
    fn builder_with_aliases(map: &[(&str, TypeExpr)]) -> TypeContextBuilder {
        let mut b = TypeContextBuilder::new();
        for (k, v) in map {
            // Simple aliases have empty params
            b.type_aliases.insert((*k).to_string(), (vec![], v.clone()));
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
            .insert("A".to_string(), (vec![], TypeExpr::Named("B".to_string())));
        b.type_aliases
            .insert("B".to_string(), (vec![], TypeExpr::Named("A".to_string())));
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
