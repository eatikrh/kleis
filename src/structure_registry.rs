///! Structure Registry for parametric structure types
///!
///! Maps structure definitions to enable generic handling of structure types
///! in type signatures and expressions.
///!
///! This complements DataTypeRegistry (for `data` types) by handling
///! structure types (defined with `structure` keyword).
///!
///! Example:
///! ```ignore
///! // In stdlib/matrices.kleis:
///! structure Matrix(m: Nat, n: Nat, T) {
///!     operation transpose : Matrix(n, m, T)
///! }
///!
///! // In user code:
///! structure Tensor(i: Nat, j: Nat, k: Nat, T) {
///!     operation contract : Tensor(i, j, k, T) → Scalar
///! }
///!
///! // StructureRegistry allows these to be looked up generically:
///! let mut registry = StructureRegistry::new();
///! registry.register(matrix_def)?;
///! registry.register(tensor_def)?;
///!
///! let matrix_def = registry.get("Matrix").unwrap();
///! assert_eq!(matrix_def.type_params.len(), 3); // m, n, T
///! ```
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
}

impl StructureRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        StructureRegistry {
            structures: HashMap::new(),
        }
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
        assert!(
            result
                .unwrap_err()
                .contains("Structure 'Matrix' is already registered")
        );
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
}
