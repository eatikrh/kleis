///! Data Type Registry for ADR-021
///!
///! Maps data type definitions and their variants to enable:
///! - Looking up variant definitions by constructor name
///! - Retrieving data type definitions by type name
///! - Validating uniqueness of variant names
///!
///! This is the foundation for dynamic type system where types
///! are defined in Kleis files rather than hardcoded in Rust.
use crate::kleis_ast::{DataDef, DataVariant};
use std::collections::HashMap;

/// Registry of data type definitions loaded from Kleis code
///
/// Maps type names to their definitions and variant names to their
/// containing type and definition.
///
/// Example usage:
/// ```ignore
/// let mut registry = DataTypeRegistry::new();
/// registry.register(bool_data_def)?;
/// registry.register(option_data_def)?;
///
/// // Lookup by variant name
/// let (type_name, variant) = registry.lookup_variant("Some")?;
/// assert_eq!(type_name, "Option");
/// ```
#[derive(Debug, Clone)]
pub struct DataTypeRegistry {
    /// Maps data type name to its definition
    /// Example: "Bool" → DataDef { variants: [True, False] }
    types: HashMap<String, DataDef>,

    /// Maps variant/constructor name to (type name, variant definition)
    /// Example: "True" → ("Bool", DataVariant { name: "True", fields: [] })
    ///          "Some" → ("Option", DataVariant { name: "Some", fields: [...] })
    variants: HashMap<String, (String, DataVariant)>,
}

impl DataTypeRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        DataTypeRegistry {
            types: HashMap::new(),
            variants: HashMap::new(),
        }
    }

    /// Register a data type definition
    ///
    /// This adds the type to the registry and registers all its variants.
    /// Returns an error if:
    /// - The type name is already registered
    /// - Any variant name conflicts with an existing variant
    pub fn register(&mut self, def: DataDef) -> Result<(), String> {
        // Check if type name already exists
        if self.types.contains_key(&def.name) {
            return Err(format!("Data type '{}' is already registered", def.name));
        }

        // Check for variant name conflicts before registering
        for variant in &def.variants {
            if let Some((existing_type, _)) = self.variants.get(&variant.name) {
                return Err(format!(
                    "Variant '{}' conflicts with existing variant in type '{}'",
                    variant.name, existing_type
                ));
            }
        }

        // Register all variants
        for variant in &def.variants {
            self.variants
                .insert(variant.name.clone(), (def.name.clone(), variant.clone()));
        }

        // Register the type
        self.types.insert(def.name.clone(), def);

        Ok(())
    }

    /// Look up a variant by its constructor name
    ///
    /// Returns the type name it belongs to and the variant definition,
    /// or None if the variant doesn't exist.
    ///
    /// Example:
    /// ```ignore
    /// if let Some((type_name, variant)) = registry.lookup_variant("Some") {
    ///     assert_eq!(type_name, "Option");
    ///     assert_eq!(variant.fields.len(), 1);
    /// }
    /// ```
    pub fn lookup_variant(&self, name: &str) -> Option<&(String, DataVariant)> {
        self.variants.get(name)
    }

    /// Get a data type definition by name
    ///
    /// Returns None if the type doesn't exist.
    pub fn get_type(&self, name: &str) -> Option<&DataDef> {
        self.types.get(name)
    }

    /// Get all registered type names
    pub fn type_names(&self) -> Vec<&String> {
        self.types.keys().collect()
    }

    /// Get all registered variant names
    pub fn variant_names(&self) -> Vec<&String> {
        self.variants.keys().collect()
    }

    /// Check if a type is registered
    pub fn has_type(&self, name: &str) -> bool {
        self.types.contains_key(name)
    }

    /// Check if a variant is registered
    pub fn has_variant(&self, name: &str) -> bool {
        self.variants.contains_key(name)
    }

    /// Get the number of registered types
    pub fn type_count(&self) -> usize {
        self.types.len()
    }

    /// Get the number of registered variants
    pub fn variant_count(&self) -> usize {
        self.variants.len()
    }
}

impl Default for DataTypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kleis_ast::{DataField, TypeExpr, TypeParam};

    fn make_bool_data_def() -> DataDef {
        DataDef {
            name: "Bool".to_string(),
            type_params: vec![],
            variants: vec![
                DataVariant {
                    name: "True".to_string(),
                    fields: vec![],
                },
                DataVariant {
                    name: "False".to_string(),
                    fields: vec![],
                },
            ],
        }
    }

    fn make_option_data_def() -> DataDef {
        DataDef {
            name: "Option".to_string(),
            type_params: vec![TypeParam {
                name: "T".to_string(),
                kind: None,
            }],
            variants: vec![
                DataVariant {
                    name: "None".to_string(),
                    fields: vec![],
                },
                DataVariant {
                    name: "Some".to_string(),
                    fields: vec![DataField {
                        name: None,
                        type_expr: TypeExpr::Named("T".to_string()),
                    }],
                },
            ],
        }
    }

    #[test]
    fn test_new_registry() {
        let registry = DataTypeRegistry::new();
        assert_eq!(registry.type_count(), 0);
        assert_eq!(registry.variant_count(), 0);
    }

    #[test]
    fn test_register_simple_type() {
        let mut registry = DataTypeRegistry::new();
        let bool_def = make_bool_data_def();

        let result = registry.register(bool_def);
        assert!(result.is_ok());

        assert_eq!(registry.type_count(), 1);
        assert_eq!(registry.variant_count(), 2);
        assert!(registry.has_type("Bool"));
        assert!(registry.has_variant("True"));
        assert!(registry.has_variant("False"));
    }

    #[test]
    fn test_lookup_variant() {
        let mut registry = DataTypeRegistry::new();
        registry.register(make_bool_data_def()).unwrap();

        let result = registry.lookup_variant("True");
        assert!(result.is_some());

        let (type_name, variant) = result.unwrap();
        assert_eq!(type_name, "Bool");
        assert_eq!(variant.name, "True");
        assert!(variant.fields.is_empty());
    }

    #[test]
    fn test_lookup_nonexistent_variant() {
        let registry = DataTypeRegistry::new();
        let result = registry.lookup_variant("NonExistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_type() {
        let mut registry = DataTypeRegistry::new();
        let bool_def = make_bool_data_def();
        registry.register(bool_def.clone()).unwrap();

        let result = registry.get_type("Bool");
        assert!(result.is_some());

        let retrieved = result.unwrap();
        assert_eq!(retrieved.name, "Bool");
        assert_eq!(retrieved.variants.len(), 2);
    }

    #[test]
    fn test_register_multiple_types() {
        let mut registry = DataTypeRegistry::new();

        registry.register(make_bool_data_def()).unwrap();
        registry.register(make_option_data_def()).unwrap();

        assert_eq!(registry.type_count(), 2);
        assert_eq!(registry.variant_count(), 4); // True, False, None, Some

        assert!(registry.has_type("Bool"));
        assert!(registry.has_type("Option"));
        assert!(registry.has_variant("True"));
        assert!(registry.has_variant("Some"));
    }

    #[test]
    fn test_error_duplicate_type() {
        let mut registry = DataTypeRegistry::new();
        let bool_def = make_bool_data_def();

        registry.register(bool_def.clone()).unwrap();
        let result = registry.register(bool_def.clone());

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already registered"));
    }

    #[test]
    fn test_error_duplicate_variant() {
        let mut registry = DataTypeRegistry::new();

        registry.register(make_bool_data_def()).unwrap();

        // Try to register another type with "True" variant
        let bad_def = DataDef {
            name: "AnotherType".to_string(),
            type_params: vec![],
            variants: vec![DataVariant {
                name: "True".to_string(), // Conflicts!
                fields: vec![],
            }],
        };

        let result = registry.register(bad_def);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("conflicts"));
    }

    #[test]
    fn test_parametric_type() {
        let mut registry = DataTypeRegistry::new();
        registry.register(make_option_data_def()).unwrap();

        // Check type
        let option_type = registry.get_type("Option");
        assert!(option_type.is_some());
        assert_eq!(option_type.unwrap().type_params.len(), 1);

        // Check variants
        let none_variant = registry.lookup_variant("None");
        assert!(none_variant.is_some());
        assert_eq!(none_variant.unwrap().0, "Option");

        let some_variant = registry.lookup_variant("Some");
        assert!(some_variant.is_some());
        assert_eq!(some_variant.unwrap().0, "Option");
        assert_eq!(some_variant.unwrap().1.fields.len(), 1);
    }

    #[test]
    fn test_type_names() {
        let mut registry = DataTypeRegistry::new();
        registry.register(make_bool_data_def()).unwrap();
        registry.register(make_option_data_def()).unwrap();

        let names = registry.type_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&&"Bool".to_string()));
        assert!(names.contains(&&"Option".to_string()));
    }

    #[test]
    fn test_variant_names() {
        let mut registry = DataTypeRegistry::new();
        registry.register(make_bool_data_def()).unwrap();

        let names = registry.variant_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&&"True".to_string()));
        assert!(names.contains(&&"False".to_string()));
    }

    #[test]
    fn test_clone_registry() {
        let mut registry1 = DataTypeRegistry::new();
        registry1.register(make_bool_data_def()).unwrap();

        let registry2 = registry1.clone();
        assert_eq!(registry2.type_count(), 1);
        assert!(registry2.has_variant("True"));
    }
}
