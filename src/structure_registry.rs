//! Structure Registry for parametric structure types
//!
//! Maps structure definitions to enable generic handling of structure types
//! in type signatures and expressions.
//!
//! This complements DataTypeRegistry (for `data` types) by handling
//! structure types (defined with `structure` keyword).
//!
//! Example:
//! ```ignore
//! // In stdlib/matrices.kleis:
//! structure Matrix(m: Nat, n: Nat, T) {
//!     operation transpose : Matrix(n, m, T)
//! }
//!
//! // In user code:
//! structure Tensor(i: Nat, j: Nat, k: Nat, T) {
//!     operation contract : Tensor(i, j, k, T) → Scalar
//! }
//!
//! // StructureRegistry allows these to be looked up generically:
//! let mut registry = StructureRegistry::new();
//! registry.register(matrix_def)?;
//! registry.register(tensor_def)?;
//!
//! let matrix_def = registry.get("Matrix").unwrap();
//! assert_eq!(matrix_def.type_params.len(), 3); // m, n, T
//! ```
use crate::kleis_ast::StructureDef;
use std::collections::HashMap;

/// Registry of structure definitions loaded from Kleis code
///
/// Maps structure names to their definitions, enabling generic handling
/// of parametric structure types without hardcoding.
///
/// **Design:** Structures define type classes/interfaces with operations.
/// When a structure name appears in a type signature, we look it up here
/// to get its parameters and construct the appropriate Type.
#[derive(Debug, Clone)]
pub struct StructureRegistry {
    /// Maps structure name to its definition
    /// Example: "Matrix" → StructureDef { name: "Matrix", type_params: [m: Nat, n: Nat, T], ... }
    structures: HashMap<String, StructureDef>,

    /// Implements blocks for structures
    /// Maps structure name to all its implementations
    /// Example: "MatrixMultipliable" → [ImplementsDef with where Semiring(T)]
    implements: HashMap<String, Vec<crate::kleis_ast::ImplementsDef>>,

    /// Top-level operation declarations
    /// Example: "apply_kernel" → TypeExpr::Function(Product([GreenKernel, Flow]), FieldR4)
    toplevel_operations: HashMap<String, crate::kleis_ast::TypeExpr>,

    /// Data type definitions (ADTs)
    /// Example: "Channel" → DataDef { name: "Channel", variants: [Mass, EM, Spin, Color] }
    data_types: HashMap<String, crate::kleis_ast::DataDef>,

    /// Type aliases
    /// Example: "Vector" → (params, TypeExpr)
    type_aliases: HashMap<
        String,
        (
            Vec<crate::kleis_ast::TypeAliasParam>,
            crate::kleis_ast::TypeExpr,
        ),
    >,

    /// Function definitions
    /// Example: "double" → FunctionDef { params: ["x"], body: add(x, x) }
    functions: HashMap<String, crate::kleis_ast::FunctionDef>,
}

impl StructureRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        StructureRegistry {
            structures: HashMap::new(),
            implements: HashMap::new(),
            toplevel_operations: HashMap::new(),
            data_types: HashMap::new(),
            type_aliases: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// Register a data type (ADT) definition
    pub fn register_data_type(&mut self, data_def: crate::kleis_ast::DataDef) {
        self.data_types.insert(data_def.name.clone(), data_def);
    }

    /// Get a data type by name
    pub fn get_data_type(&self, name: &str) -> Option<&crate::kleis_ast::DataDef> {
        self.data_types.get(name)
    }

    /// Check if a name is a known data type
    pub fn has_data_type(&self, name: &str) -> bool {
        self.data_types.contains_key(name)
    }

    /// Register a type alias
    pub fn register_type_alias(
        &mut self,
        name: String,
        params: Vec<crate::kleis_ast::TypeAliasParam>,
        type_expr: crate::kleis_ast::TypeExpr,
    ) {
        self.type_aliases.insert(name, (params, type_expr));
    }

    /// Get a type alias by name
    pub fn get_type_alias(
        &self,
        name: &str,
    ) -> Option<&(
        Vec<crate::kleis_ast::TypeAliasParam>,
        crate::kleis_ast::TypeExpr,
    )> {
        self.type_aliases.get(name)
    }

    /// Check if a name is a known type alias
    pub fn has_type_alias(&self, name: &str) -> bool {
        self.type_aliases.contains_key(name)
    }

    /// Iterate over all registered data types
    pub fn data_types(&self) -> impl Iterator<Item = &crate::kleis_ast::DataDef> {
        self.data_types.values()
    }

    /// Get the number of registered data types
    pub fn data_type_count(&self) -> usize {
        self.data_types.len()
    }

    /// Iterate over all registered type aliases
    pub fn type_aliases(
        &self,
    ) -> impl Iterator<
        Item = (
            &String,
            &(
                Vec<crate::kleis_ast::TypeAliasParam>,
                crate::kleis_ast::TypeExpr,
            ),
        ),
    > {
        self.type_aliases.iter()
    }

    /// Get the number of registered type aliases
    pub fn type_alias_count(&self) -> usize {
        self.type_aliases.len()
    }

    /// Register a top-level operation declaration
    pub fn register_toplevel_operation(
        &mut self,
        name: String,
        type_sig: crate::kleis_ast::TypeExpr,
    ) {
        self.toplevel_operations.insert(name, type_sig);
    }

    /// Register a function definition
    pub fn register_function(&mut self, func_def: crate::kleis_ast::FunctionDef) {
        self.functions.insert(func_def.name.clone(), func_def);
    }

    /// Get a function definition by name
    pub fn get_function(&self, name: &str) -> Option<&crate::kleis_ast::FunctionDef> {
        self.functions.get(name)
    }

    /// Check if a function is registered
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get the number of registered functions
    pub fn function_count(&self) -> usize {
        self.functions.len()
    }

    /// Iterate over all function definitions
    pub fn functions(&self) -> impl Iterator<Item = &crate::kleis_ast::FunctionDef> {
        self.functions.values()
    }

    /// Load structures from a Kleis file
    ///
    /// Parses the file and registers all structures found.
    ///
    /// # Example
    /// ```ignore
    /// let mut registry = StructureRegistry::new();
    /// registry.load_from_file("stdlib/tensors.kleis")?;
    /// assert!(registry.has_structure("Tensor"));
    /// ```
    pub fn load_from_file(&mut self, path: &str) -> Result<usize, String> {
        use crate::kleis_ast::TopLevel;
        use crate::kleis_parser::KleisParser;

        // Read file
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;

        // Parse
        let mut parser = KleisParser::new(&content);
        let program = parser
            .parse_program()
            .map_err(|e| format!("Failed to parse {}: {}", path, e))?;

        // Register structures
        let mut count = 0;
        for top_level in program.items {
            if let TopLevel::StructureDef(def) = top_level {
                // Skip if already registered (allows multiple loads)
                if !self.has_structure(&def.name) {
                    self.register(def)?;
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Load all standard library files
    ///
    /// Loads structures from stdlib/*.kleis files in order.
    /// Returns the total number of structures loaded.
    pub fn load_stdlib(&mut self) -> Result<usize, String> {
        let stdlib_files = [
            "stdlib/types.kleis",
            "stdlib/minimal_prelude.kleis",
            "stdlib/matrices.kleis",
            "stdlib/tensors.kleis",
        ];

        let mut total = 0;
        for file in &stdlib_files {
            if std::path::Path::new(file).exists() {
                match self.load_from_file(file) {
                    Ok(count) => {
                        total += count;
                        println!("   📚 Loaded {} structures from {}", count, file);
                    }
                    Err(e) => {
                        eprintln!("   ⚠️ Warning: {}", e);
                    }
                }
            }
        }

        Ok(total)
    }

    /// Register a structure definition
    ///
    /// This adds the structure to the registry.
    /// Returns an error if the structure name is already registered.
    pub fn register(&mut self, def: StructureDef) -> Result<(), String> {
        // Check if structure name already exists
        if self.structures.contains_key(&def.name) {
            return Err(format!("Structure '{}' is already registered", def.name));
        }

        // Register the structure
        self.structures.insert(def.name.clone(), def);

        Ok(())
    }

    /// Get a structure definition by name
    ///
    /// Returns None if the structure doesn't exist.
    ///
    /// Example:
    /// ```ignore
    /// if let Some(structure) = registry.get("Matrix") {
    ///     println!("Matrix has {} type parameters", structure.type_params.len());
    /// }
    /// ```
    pub fn get(&self, name: &str) -> Option<&StructureDef> {
        self.structures.get(name)
    }

    /// Get all registered structure names
    pub fn structure_names(&self) -> Vec<&String> {
        self.structures.keys().collect()
    }

    /// Check if a structure is registered
    pub fn has_structure(&self, name: &str) -> bool {
        self.structures.contains_key(name)
    }

    /// Get the number of registered structures
    pub fn structure_count(&self) -> usize {
        self.structures.len()
    }

    /// Get the number of registered top-level operations
    pub fn operation_count(&self) -> usize {
        self.toplevel_operations.len()
    }

    // =========================================================================
    // Definition Removal (for :unload/:reload/:reset)
    // =========================================================================

    /// Remove a structure by name
    /// Returns true if the structure was found and removed
    pub fn remove_structure(&mut self, name: &str) -> bool {
        // Also remove any implements blocks for this structure
        self.implements.remove(name);
        self.structures.remove(name).is_some()
    }

    /// Remove all implements blocks for a structure
    /// Returns the number of implements blocks removed
    pub fn remove_implements_for_structure(&mut self, structure_name: &str) -> usize {
        if let Some(impls) = self.implements.remove(structure_name) {
            impls.len()
        } else {
            0
        }
    }

    /// Clear all structures and implements (for :reset)
    pub fn reset(&mut self) {
        self.structures.clear();
        self.implements.clear();
    }

    /// Get all axioms from a structure
    ///
    /// Returns a vector of (axiom_name, axiom_expression) pairs.
    ///
    /// Example:
    /// ```ignore
    /// let axioms = registry.get_axioms("Ring");
    /// for (name, expr) in axioms {
    ///     println!("Axiom {}: {:?}", name, expr);
    /// }
    /// ```
    pub fn get_axioms(&self, structure_name: &str) -> Vec<(String, &crate::ast::Expression)> {
        if let Some(structure) = self.get(structure_name) {
            structure
                .members
                .iter()
                .filter_map(|member| match member {
                    crate::kleis_ast::StructureMember::Axiom { name, proposition } => {
                        Some((name.clone(), proposition))
                    }
                    _ => None,
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all operations from a structure
    ///
    /// Returns a vector of (operation_name, type_signature) pairs.
    pub fn get_operations(
        &self,
        structure_name: &str,
    ) -> Vec<(String, &crate::kleis_ast::TypeExpr)> {
        if let Some(structure) = self.get(structure_name) {
            structure
                .members
                .iter()
                .filter_map(|member| match member {
                    crate::kleis_ast::StructureMember::Operation {
                        name,
                        type_signature,
                    } => Some((name.clone(), type_signature)),
                    _ => None,
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get a specific operation's type signature from any structure or top-level declaration
    ///
    /// Searches:
    /// 1. All registered structures for the operation
    /// 2. Top-level operation declarations
    ///
    /// Returns the TypeExpr (e.g., `ℂ × Flow → Flow` for `flow_smul`)
    ///
    /// Used by Z3 backend to determine correct sorts for function declarations.
    pub fn get_operation_signature(
        &self,
        operation_name: &str,
    ) -> Option<&crate::kleis_ast::TypeExpr> {
        // First check structures
        for structure in self.structures.values() {
            for member in &structure.members {
                if let crate::kleis_ast::StructureMember::Operation {
                    name,
                    type_signature,
                } = member
                {
                    if name == operation_name {
                        return Some(type_signature);
                    }
                }
            }
        }

        // Then check top-level operations
        self.toplevel_operations.get(operation_name)
    }

    /// Check if a structure has a specific axiom
    pub fn has_axiom(&self, structure_name: &str, axiom_name: &str) -> bool {
        self.get_axioms(structure_name)
            .iter()
            .any(|(name, _)| name == axiom_name)
    }

    /// Get all structures that have axioms
    pub fn structures_with_axioms(&self) -> Vec<&String> {
        self.structures
            .iter()
            .filter(|(_, def)| {
                def.members
                    .iter()
                    .any(|m| matches!(m, crate::kleis_ast::StructureMember::Axiom { .. }))
            })
            .map(|(name, _)| name)
            .collect()
    }

    /// Find which structures define a given operation
    ///
    /// Returns a vector of structure names that have this operation.
    /// Used by axiom verifier for dependency analysis.
    ///
    /// # Example
    /// ```ignore
    /// let owners = registry.get_operation_owners("transpose");
    /// // Returns: ["Matrix", "Tensor"] if both define transpose
    /// ```
    pub fn get_operation_owners(&self, operation_name: &str) -> Option<Vec<String>> {
        let mut owners = Vec::new();

        for (struct_name, def) in &self.structures {
            // Check if this structure has the operation
            let has_op = def.members.iter().any(|member| match member {
                crate::kleis_ast::StructureMember::Operation { name, .. } => name == operation_name,
                _ => false,
            });

            if has_op {
                owners.push(struct_name.clone());
            }
        }

        if owners.is_empty() {
            None
        } else {
            Some(owners)
        }
    }

    /// Register an implements block
    ///
    /// Stores the implements block so we can query where constraints later.
    /// Used by axiom verifier to load constrained structures.
    pub fn register_implements(&mut self, impl_def: crate::kleis_ast::ImplementsDef) {
        self.implements
            .entry(impl_def.structure_name.clone())
            .or_default()
            .push(impl_def);
    }

    /// Get where constraints for a structure
    ///
    /// Returns all where constraints from all implements blocks for this structure.
    /// Used by axiom verifier to load constrained structure axioms.
    ///
    /// # Example
    /// ```ignore
    /// // Given: implements MatrixMultipliable(...) where Semiring(T)
    /// let constraints = registry.get_where_constraints("MatrixMultipliable");
    /// // Returns constraints from all implements of MatrixMultipliable
    /// ```
    pub fn get_where_constraints(
        &self,
        structure_name: &str,
    ) -> Vec<&crate::kleis_ast::WhereConstraint> {
        let mut all_constraints = Vec::new();

        if let Some(impls) = self.implements.get(structure_name) {
            for impl_def in impls {
                if let Some(constraints) = &impl_def.where_clause {
                    all_constraints.extend(constraints.iter());
                }
            }
        }

        all_constraints
    }
}

impl Default for StructureRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kleis_ast::{StructureMember, TypeExpr, TypeParam};

    fn make_matrix_structure() -> StructureDef {
        StructureDef {
            name: "Matrix".to_string(),
            extends_clause: None,
            over_clause: None,
            type_params: vec![
                TypeParam {
                    name: "m".to_string(),
                    kind: Some("Nat".to_string()),
                },
                TypeParam {
                    name: "n".to_string(),
                    kind: Some("Nat".to_string()),
                },
                TypeParam {
                    name: "T".to_string(),
                    kind: None, // Type parameter
                },
            ],
            members: vec![StructureMember::Operation {
                name: "transpose".to_string(),
                type_signature: TypeExpr::Parametric(
                    "Matrix".to_string(),
                    vec![
                        TypeExpr::Named("n".to_string()),
                        TypeExpr::Named("m".to_string()),
                        TypeExpr::Named("T".to_string()),
                    ],
                ),
            }],
        }
    }

    fn make_ring_structure_with_axioms() -> StructureDef {
        use crate::ast::Expression;

        StructureDef {
            name: "Ring".to_string(),
            extends_clause: None,
            over_clause: None,
            type_params: vec![TypeParam {
                name: "R".to_string(),
                kind: None,
            }],
            members: vec![
                StructureMember::Operation {
                    name: "+".to_string(),
                    type_signature: TypeExpr::Named("R".to_string()),
                },
                StructureMember::Operation {
                    name: "×".to_string(),
                    type_signature: TypeExpr::Named("R".to_string()),
                },
                StructureMember::Axiom {
                    name: "distributivity".to_string(),
                    proposition: Expression::Object("axiom_expr".to_string()),
                },
                StructureMember::Axiom {
                    name: "commutativity".to_string(),
                    proposition: Expression::Object("axiom_expr2".to_string()),
                },
            ],
        }
    }

    #[test]
    fn test_register_structure() {
        let mut registry = StructureRegistry::new();
        let matrix = make_matrix_structure();

        assert!(registry.register(matrix.clone()).is_ok());
        assert_eq!(registry.structure_count(), 1);
        assert!(registry.has_structure("Matrix"));
    }

    #[test]
    fn test_get_structure() {
        let mut registry = StructureRegistry::new();
        let matrix = make_matrix_structure();
        registry.register(matrix).unwrap();

        let retrieved = registry.get("Matrix").unwrap();
        assert_eq!(retrieved.name, "Matrix");
        assert_eq!(retrieved.type_params.len(), 3);
    }

    #[test]
    fn test_duplicate_structure_fails() {
        let mut registry = StructureRegistry::new();
        let matrix = make_matrix_structure();

        registry.register(matrix.clone()).unwrap();
        let result = registry.register(matrix);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Structure 'Matrix' is already registered"));
    }

    #[test]
    fn test_unknown_structure_returns_none() {
        let registry = StructureRegistry::new();
        assert!(registry.get("NonExistent").is_none());
        assert!(!registry.has_structure("NonExistent"));
    }

    #[test]
    fn test_structure_names() {
        let mut registry = StructureRegistry::new();
        let matrix = make_matrix_structure();
        registry.register(matrix).unwrap();

        let names = registry.structure_names();
        assert_eq!(names.len(), 1);
        assert!(names.contains(&&"Matrix".to_string()));
    }

    #[test]
    fn test_get_axioms() {
        let mut registry = StructureRegistry::new();
        let ring = make_ring_structure_with_axioms();

        registry.register(ring).unwrap();

        let axioms = registry.get_axioms("Ring");
        assert_eq!(axioms.len(), 2);

        let axiom_names: Vec<String> = axioms.iter().map(|(name, _)| name.clone()).collect();
        assert!(axiom_names.contains(&"distributivity".to_string()));
        assert!(axiom_names.contains(&"commutativity".to_string()));
    }

    #[test]
    fn test_get_operations() {
        let mut registry = StructureRegistry::new();
        let ring = make_ring_structure_with_axioms();

        registry.register(ring).unwrap();

        let operations = registry.get_operations("Ring");
        assert_eq!(operations.len(), 2);

        let op_names: Vec<String> = operations.iter().map(|(name, _)| name.clone()).collect();
        assert!(op_names.contains(&"+".to_string()));
        assert!(op_names.contains(&"×".to_string()));
    }

    #[test]
    fn test_has_axiom() {
        let mut registry = StructureRegistry::new();
        let ring = make_ring_structure_with_axioms();

        registry.register(ring).unwrap();

        assert!(registry.has_axiom("Ring", "distributivity"));
        assert!(registry.has_axiom("Ring", "commutativity"));
        assert!(!registry.has_axiom("Ring", "nonexistent"));
        assert!(!registry.has_axiom("Nonexistent", "distributivity"));
    }

    #[test]
    fn test_structures_with_axioms() {
        let mut registry = StructureRegistry::new();
        let matrix = make_matrix_structure();
        let ring = make_ring_structure_with_axioms();

        registry.register(matrix).unwrap();
        registry.register(ring).unwrap();

        let with_axioms = registry.structures_with_axioms();
        assert_eq!(with_axioms.len(), 1);
        assert!(with_axioms.contains(&&"Ring".to_string()));
        assert!(!with_axioms.contains(&&"Matrix".to_string()));
    }

    #[test]
    fn test_get_axioms_nonexistent_structure() {
        let registry = StructureRegistry::new();
        let axioms = registry.get_axioms("Nonexistent");
        assert_eq!(axioms.len(), 0);
    }

    // =========================================================================
    // Tests for remove/reset methods (REPL unload/reload support)
    // =========================================================================

    #[test]
    fn test_remove_structure() {
        let mut registry = StructureRegistry::new();
        let matrix = make_matrix_structure();
        let ring = make_ring_structure_with_axioms();

        registry.register(matrix).unwrap();
        registry.register(ring).unwrap();

        assert_eq!(registry.structure_count(), 2);
        assert!(registry.has_structure("Matrix"));
        assert!(registry.has_structure("Ring"));

        // Remove Matrix
        assert!(registry.remove_structure("Matrix"));
        assert_eq!(registry.structure_count(), 1);
        assert!(!registry.has_structure("Matrix"));
        assert!(registry.has_structure("Ring"));

        // Removing again returns false
        assert!(!registry.remove_structure("Matrix"));
    }

    #[test]
    fn test_reset() {
        let mut registry = StructureRegistry::new();
        let matrix = make_matrix_structure();
        let ring = make_ring_structure_with_axioms();

        registry.register(matrix).unwrap();
        registry.register(ring).unwrap();

        assert_eq!(registry.structure_count(), 2);

        registry.reset();

        assert_eq!(registry.structure_count(), 0);
        assert!(!registry.has_structure("Matrix"));
        assert!(!registry.has_structure("Ring"));
    }

    #[test]
    fn test_remove_implements_for_structure() {
        let mut registry = StructureRegistry::new();
        let ring = make_ring_structure_with_axioms();

        registry.register(ring).unwrap();

        // Create a mock implements block
        let impl_def = crate::kleis_ast::ImplementsDef {
            structure_name: "Ring".to_string(),
            type_args: vec![TypeExpr::Named("ℤ".to_string())],
            over_clause: None,
            where_clause: None,
            members: vec![],
        };
        registry.register_implements(impl_def);

        // Remove implements - should return 1 (one block removed)
        let removed = registry.remove_implements_for_structure("Ring");
        assert_eq!(removed, 1);

        // Removing again should return 0 (nothing left)
        let removed_again = registry.remove_implements_for_structure("Ring");
        assert_eq!(removed_again, 0);
    }
}
