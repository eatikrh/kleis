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
}
