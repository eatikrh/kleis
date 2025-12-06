///! Type Context Builder - Connects parsed structures to type inference
///!
///! This module builds a TypeContext from parsed Kleis programs:
///! 1. Loads structure definitions (abstract operations)
///! 2. Loads implements blocks (concrete bindings)
///! 3. Builds operation registry
///! 4. Provides query interface for type checking
///!
///! Example:
///! ```
///! let program = parse_kleis_program("structure Numeric(N) { ... }")?;
///! let ctx = TypeContextBuilder::from_program(program)?;
///! ctx.supports_operation("abs", &Type::Real); // true
///! ```
use crate::kleis_ast::{
    ImplMember, ImplementsDef, Program, StructureDef, StructureMember, TopLevel, TypeExpr,
};
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
    type_to_structures: HashMap<String, Vec<String>>,
}

impl OperationRegistry {
    pub fn new() -> Self {
        OperationRegistry {
            operation_to_structure: HashMap::new(),
            structure_to_operations: HashMap::new(),
            concrete_implementations: HashMap::new(),
            type_to_structures: HashMap::new(),
        }
    }

    /// Register that a structure defines an operation
    pub fn register_operation(&mut self, structure_name: &str, operation_name: &str) {
        self.operation_to_structure
            .insert(operation_name.to_string(), structure_name.to_string());

        self.structure_to_operations
            .entry(structure_name.to_string())
            .or_insert_with(Vec::new)
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
            .or_insert_with(Vec::new)
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
}

impl TypeContextBuilder {
    pub fn new() -> Self {
        TypeContextBuilder {
            structures: HashMap::new(),
            implements: Vec::new(),
            registry: OperationRegistry::new(),
            context: TypeContext::new(),
        }
    }

    /// Build type context from a parsed program
    pub fn from_program(program: Program) -> Result<Self, String> {
        let mut builder = Self::new();

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
        // Register operations from this structure
        for member in &structure.members {
            if let StructureMember::Operation { name, .. } = member {
                self.registry.register_operation(&structure.name, name);
            }
        }

        self.structures
            .insert(structure.name.clone(), structure.clone());
        Ok(())
    }

    fn register_implements(&mut self, impl_def: &ImplementsDef) -> Result<(), String> {
        // Find the structure this implements
        let structure = self
            .structures
            .get(&impl_def.structure_name)
            .ok_or_else(|| format!("Unknown structure: {}", impl_def.structure_name))?;

        // Extract type name from type_args (use first arg for now, TODO: handle multiple)
        let type_name = if let Some(first_arg) = impl_def.type_args.first() {
            self.type_expr_to_string(first_arg)
        } else {
            return Err(format!("Implements block for {} has no type arguments", impl_def.structure_name));
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
        }
    }

    /// Get the operation registry
    pub fn registry(&self) -> &OperationRegistry {
        &self.registry
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
}

impl Default for TypeContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kleis_parser::parse_kleis_program;

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
}
