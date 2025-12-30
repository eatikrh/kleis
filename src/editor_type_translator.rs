//! Editor Type Translator - Converts EditorNode types to Kleis type_inference::Type
//!
//! This module provides a GENERIC translation layer between:
//! - EditorNode (Visual Editor AST with rendering metadata)
//! - type_inference::Type (Canonical Kleis type system)
//!
//! ## Design Principles
//!
//! 1. **No hardcoded types** - The translator is agnostic to specific types like Matrix, Tensor, etc.
//! 2. **Data-driven** - Type information is extracted from EditorNode metadata generically
//! 3. **User-defined types** - Works with any type, including custom user-defined types
//! 4. **Z3-agnostic** - No references to Z3 here; Z3 mappings live in `solvers/z3/type_mapping.rs`
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ Equation Editor              │ Kleis "example" blocks       │
//! ├──────────────────────────────┼──────────────────────────────┤
//! │ EditorNode                   │ Expression                   │
//! │ kind: "matrix"               │ TypeChecker.check()          │
//! │ metadata: {rows, cols}       │                              │
//! │     ↓                        │     ↓                        │
//! │ EditorTypeTranslator         │ (already Type)               │
//! │ (GENERIC extraction)         │     │                        │
//! └─────────────────────────────────────────────────────────────┘
//!                     ↓
//!            type_inference::Type  ← CANONICAL
//!                     ↓
//!     solvers/z3/type_mapping.rs  ← Z3-SPECIFIC MAPPINGS
//!                     ↓
//!               Z3Backend
//! ```

use crate::editor_ast::{EditorNode, OperationData};
use crate::type_inference::Type;

/// Translates EditorNode type information to Kleis canonical types.
///
/// This translator is completely generic - it extracts type information
/// from EditorNode metadata without knowing what specific types mean.
/// Z3-specific mappings are handled separately in `solvers/z3/type_mapping.rs`.
pub struct EditorTypeTranslator;

impl EditorTypeTranslator {
    /// Extract type from an EditorNode based on its kind and metadata.
    ///
    /// This is a GENERIC extraction - we don't hardcode specific types.
    /// We simply convert EditorNode metadata into the canonical Type representation.
    ///
    /// Returns Some(Type) if the EditorNode has explicit type information,
    /// None if it should fall back to standard type inference.
    pub fn translate(node: &EditorNode) -> Option<Type> {
        match node {
            EditorNode::Operation { operation } => Self::translate_operation(operation),
            EditorNode::Const { value } => Self::translate_const(value),
            EditorNode::Object { .. } => None, // Variables - let type inference handle
            EditorNode::Placeholder { .. } => None, // Placeholders have no type yet
            EditorNode::List { .. } => None,   // Lists need element type inference
        }
    }

    /// Translate an operation node based on its kind and metadata.
    ///
    /// This is completely data-driven:
    /// - kind: determines the type_name
    /// - metadata: provides type parameters (dimensions, etc.)
    fn translate_operation(op: &OperationData) -> Option<Type> {
        // If the operation has an explicit kind, use it as the type name
        if let Some(kind) = &op.kind {
            return Some(Self::build_type_from_kind(kind, op));
        }

        // No explicit kind - let standard type inference handle it
        None
    }

    /// Build a Type from the kind and metadata.
    ///
    /// This is generic - we don't interpret what the kind means,
    /// we just extract the data from metadata and build a Type.
    fn build_type_from_kind(kind: &str, op: &OperationData) -> Type {
        // Extract type arguments from metadata
        let type_args = Self::extract_type_args_from_metadata(op);

        // Build the Type - kind becomes type_name AND constructor
        // (for simple types they're the same; for ADTs constructor may differ)
        Type::Data {
            type_name: kind.to_string(),
            constructor: kind.to_string(),
            args: type_args,
        }
    }

    /// Extract type arguments from operation metadata.
    ///
    /// This is generic - we extract numeric values and convert them to NatValue.
    fn extract_type_args_from_metadata(op: &OperationData) -> Vec<Type> {
        let mut args = Vec::new();

        if let Some(meta) = &op.metadata {
            // Common dimension fields (generic - not specific to any type)
            for dim_key in ["rows", "cols", "size", "rank", "dim", "n", "m"] {
                if let Some(val) = meta.get(dim_key).and_then(|v| v.as_u64()) {
                    args.push(Type::NatValue(val as usize));
                }
            }

            // Index structure (for tensors and similar)
            if let Some(idx_struct) = meta.get("indexStructure").and_then(|v| v.as_array()) {
                let upper = idx_struct
                    .iter()
                    .filter(|v| v.as_str() == Some("up"))
                    .count();
                let lower = idx_struct
                    .iter()
                    .filter(|v| v.as_str() == Some("down"))
                    .count();
                args.push(Type::NatValue(upper));
                args.push(Type::NatValue(lower));
            }

            // Element type (if specified)
            if let Some(elem_type) = meta.get("elementType").and_then(|v| v.as_str()) {
                args.push(Type::Data {
                    type_name: elem_type.to_string(),
                    constructor: elem_type.to_string(),
                    args: vec![],
                });
            }

            // Custom type name (for user-defined types)
            if let Some(custom_type) = meta.get("typeName").and_then(|v| v.as_str()) {
                // This allows user-defined types to specify their own name
                args.push(Type::Data {
                    type_name: custom_type.to_string(),
                    constructor: custom_type.to_string(),
                    args: vec![],
                });
            }
        }

        args
    }

    /// Translate a constant value to a type.
    fn translate_const(value: &str) -> Option<Type> {
        // Try to determine type from constant value format
        if value.parse::<i64>().is_ok() {
            Some(Type::Nat) // Integer constant
        } else if value.parse::<f64>().is_ok() {
            // Real number - represented as Data type
            Some(Type::Data {
                type_name: "Real".to_string(),
                constructor: "Real".to_string(),
                args: vec![],
            })
        } else if value == "true" || value == "false" {
            Some(Type::Bool)
        } else {
            None // String or unknown
        }
    }
}

/// TypeEnvironment stores type information for expressions.
/// Used to pass type info from type checking to solvers.
#[derive(Debug, Clone, Default)]
pub struct TypeEnvironment {
    /// Maps expression path/identifiers to their types
    types: std::collections::HashMap<String, Type>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a type for an identifier/path
    pub fn insert(&mut self, name: String, ty: Type) {
        self.types.insert(name, ty);
    }

    /// Look up type for an identifier/path
    pub fn get(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }

    /// Check if we have type info for an identifier
    pub fn contains(&self, name: &str) -> bool {
        self.types.contains_key(name)
    }

    /// Get all type mappings
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Type)> {
        self.types.iter()
    }

    /// Number of type entries
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Get underlying map (for passing to Z3Backend)
    pub fn into_map(self) -> std::collections::HashMap<String, Type> {
        self.types
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor_ast::PlaceholderData;
    use std::collections::HashMap;

    #[test]
    fn test_translate_generic_type_with_kind() {
        let mut metadata = HashMap::new();
        metadata.insert("rows".to_string(), serde_json::json!(3));
        metadata.insert("cols".to_string(), serde_json::json!(4));

        let node = EditorNode::Operation {
            operation: OperationData {
                name: "my_matrix".to_string(),
                args: vec![],
                kind: Some("matrix".to_string()), // Kind determines type
                metadata: Some(metadata),
            },
        };

        let ty = EditorTypeTranslator::translate(&node);
        assert!(ty.is_some());

        if let Some(Type::Data {
            type_name,
            constructor,
            args,
        }) = ty
        {
            assert_eq!(type_name, "matrix"); // Kind becomes type_name
            assert_eq!(constructor, "matrix");
            assert!(args.len() >= 2);
            assert_eq!(args[0], Type::NatValue(3)); // rows
            assert_eq!(args[1], Type::NatValue(4)); // cols
        } else {
            panic!("Expected Data type");
        }
    }

    #[test]
    fn test_translate_user_defined_type() {
        let mut metadata = HashMap::new();
        metadata.insert("size".to_string(), serde_json::json!(5));
        metadata.insert("typeName".to_string(), serde_json::json!("MyCustomType"));

        let node = EditorNode::Operation {
            operation: OperationData {
                name: "custom_op".to_string(),
                args: vec![],
                kind: Some("custom".to_string()),
                metadata: Some(metadata),
            },
        };

        let ty = EditorTypeTranslator::translate(&node);
        assert!(ty.is_some());

        if let Some(Type::Data { type_name, .. }) = ty {
            assert_eq!(type_name, "custom"); // User-defined type
        }
    }

    #[test]
    fn test_no_kind_returns_none() {
        let node = EditorNode::Operation {
            operation: OperationData {
                name: "plus".to_string(),
                args: vec![],
                kind: None, // No kind - fall through to type inference
                metadata: None,
            },
        };

        let ty = EditorTypeTranslator::translate(&node);
        assert!(ty.is_none()); // Should return None for standard ops
    }

    #[test]
    fn test_constant_translation() {
        let int_node = EditorNode::Const {
            value: "42".to_string(),
        };
        assert_eq!(EditorTypeTranslator::translate(&int_node), Some(Type::Nat));

        let bool_node = EditorNode::Const {
            value: "true".to_string(),
        };
        assert_eq!(
            EditorTypeTranslator::translate(&bool_node),
            Some(Type::Bool)
        );
    }

    #[test]
    fn test_type_environment() {
        let mut env = TypeEnvironment::new();
        env.insert("x".to_string(), Type::Nat);
        env.insert(
            "M".to_string(),
            Type::Data {
                type_name: "Matrix".to_string(),
                constructor: "Matrix".to_string(),
                args: vec![Type::NatValue(2), Type::NatValue(2)],
            },
        );

        assert!(env.contains("x"));
        assert!(env.contains("M"));
        assert!(!env.contains("y"));
        assert_eq!(env.len(), 2);
    }
}
