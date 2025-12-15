//! Editor AST - Rich representation for visual equation editing
//!
//! This is separate from the Kleis Core AST (`Expression`) which represents
//! pure language semantics. EditorNode includes rendering metadata needed
//! for visual editing and display.
//!
//! ## Architecture
//!
//! ```text
//! Kleis Core AST (Expression)     Visual AST (EditorNode)
//! ├── Pure, grammar-based         ├── Rich, with rendering metadata
//! ├── No rendering concerns       ├── kind, indexStructure, display hints
//! ├── Source: Kleis Parser        ├── Source: Editor palette OR translated
//! └── Used for: Verification      └── Used for: Visual Rendering/Editing
//! ```
//!
//! ## Translation
//!
//! To visually edit Kleis code:
//! 1. Parse Kleis text → Expression (Core AST)
//! 2. translate_to_editor(Expression) → EditorNode (adds rendering metadata)
//! 3. Visual Editor works with EditorNode
//! 4. Render EditorNode to LaTeX/Typst/HTML

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Editor AST node for visual equation editing
///
/// Unlike `Expression` (Kleis Core), this type includes rendering metadata
/// like `kind` and `metadata` for proper visual display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EditorNode {
    /// Simple identifier or symbol
    #[serde(rename_all = "PascalCase")]
    Object { object: String },

    /// Numeric or string constant
    #[serde(rename_all = "PascalCase")]
    Const {
        #[serde(rename = "Const")]
        value: String,
    },

    /// Placeholder for user input (□)
    #[serde(rename_all = "PascalCase")]
    Placeholder { placeholder: PlaceholderData },

    /// Operation with optional semantic type and metadata
    #[serde(rename_all = "PascalCase")]
    Operation { operation: OperationData },

    /// List of expressions
    #[serde(rename_all = "PascalCase")]
    List { list: Vec<EditorNode> },
}

/// Placeholder data for structural editing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaceholderData {
    pub id: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// Operation data with rendering metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationData {
    /// Display name/symbol (e.g., "Γ", "∫", "sin")
    pub name: String,

    /// Arguments to the operation
    pub args: Vec<EditorNode>,

    /// Semantic type hint for rendering (e.g., "tensor", "integral", "derivative")
    /// When None, renderer uses name-based template lookup (backward compatible)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    /// Additional metadata for rendering
    /// Examples: indexStructure for tensors, bounds for integrals
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

// ============================================================================
// Constructor helpers
// ============================================================================

impl EditorNode {
    /// Create an object node
    pub fn object(s: impl Into<String>) -> Self {
        EditorNode::Object { object: s.into() }
    }

    /// Create a constant node
    pub fn constant(s: impl Into<String>) -> Self {
        EditorNode::Const { value: s.into() }
    }

    /// Create a placeholder node
    pub fn placeholder(id: usize, hint: Option<String>) -> Self {
        EditorNode::Placeholder {
            placeholder: PlaceholderData { id, hint },
        }
    }

    /// Create a simple operation (no kind/metadata)
    pub fn operation(name: impl Into<String>, args: Vec<EditorNode>) -> Self {
        EditorNode::Operation {
            operation: OperationData {
                name: name.into(),
                args,
                kind: None,
                metadata: None,
            },
        }
    }

    /// Create an operation with semantic kind
    pub fn operation_with_kind(
        name: impl Into<String>,
        args: Vec<EditorNode>,
        kind: impl Into<String>,
    ) -> Self {
        EditorNode::Operation {
            operation: OperationData {
                name: name.into(),
                args,
                kind: Some(kind.into()),
                metadata: None,
            },
        }
    }

    /// Create a tensor with full index structure
    ///
    /// # Example
    /// ```ignore
    /// let christoffel = EditorNode::tensor(
    ///     "Γ",
    ///     vec![lambda, mu, nu],
    ///     vec!["up", "down", "down"],
    /// );
    /// ```
    pub fn tensor(
        symbol: impl Into<String>,
        indices: Vec<EditorNode>,
        index_structure: Vec<&str>,
    ) -> Self {
        let mut metadata = HashMap::new();
        let structure: Vec<serde_json::Value> = index_structure
            .into_iter()
            .map(|s| serde_json::Value::String(s.to_string()))
            .collect();
        metadata.insert(
            "indexStructure".to_string(),
            serde_json::Value::Array(structure),
        );

        // New structure: name = "tensor", args[0] = symbol, args[1:] = indices
        let mut args = vec![EditorNode::object(symbol)];
        args.extend(indices);

        EditorNode::Operation {
            operation: OperationData {
                name: "tensor".to_string(),
                args,
                kind: Some("tensor".to_string()),
                metadata: Some(metadata),
            },
        }
    }

    /// Create a list node
    pub fn list(elements: Vec<EditorNode>) -> Self {
        EditorNode::List { list: elements }
    }
}

// ============================================================================
// Translation from Kleis Core AST
// ============================================================================

use crate::ast::Expression;

/// Translate Kleis Core AST to Editor AST
///
/// This enriches the pure Expression with rendering metadata by:
/// - Recognizing known tensor symbols (Γ, R, g, etc.)
/// - Inferring index structure from negate() wrappers
/// - Adding appropriate `kind` and `metadata`
pub fn translate_to_editor(expr: &Expression) -> EditorNode {
    translate_with_context(expr, &TranslationContext::default())
}

/// Context for translation (type information, known symbols, etc.)
#[derive(Default)]
pub struct TranslationContext {
    /// Known tensor symbols
    pub tensor_symbols: Vec<String>,
}

impl TranslationContext {
    /// Create context with default known tensors
    pub fn with_default_tensors() -> Self {
        TranslationContext {
            tensor_symbols: vec![
                "Γ".to_string(),
                "gamma".to_string(),
                "R".to_string(),
                "riemann".to_string(),
                "g".to_string(),
                "metric".to_string(),
                "T".to_string(),
                "christoffel".to_string(),
            ],
        }
    }

    /// Check if a symbol is a known tensor
    pub fn is_tensor(&self, name: &str) -> bool {
        self.tensor_symbols.iter().any(|s| s == name)
    }
}

fn translate_with_context(expr: &Expression, ctx: &TranslationContext) -> EditorNode {
    match expr {
        Expression::Object(s) => EditorNode::object(s),

        Expression::Const(s) => EditorNode::constant(s),

        Expression::Placeholder { id, hint } => EditorNode::placeholder(*id, Some(hint.clone())),

        Expression::Operation { name, args } => {
            let translated_args: Vec<EditorNode> = args
                .iter()
                .map(|a| translate_with_context(a, ctx))
                .collect();

            // Check if this is a known tensor
            if ctx.is_tensor(name) {
                // Infer index structure from negate() wrappers
                let index_structure = infer_index_structure(args);

                if !index_structure.is_empty() {
                    // Create tensor with inferred structure
                    let mut metadata = HashMap::new();
                    let structure: Vec<serde_json::Value> = index_structure
                        .into_iter()
                        .map(|s| serde_json::Value::String(s.to_string()))
                        .collect();
                    metadata.insert(
                        "indexStructure".to_string(),
                        serde_json::Value::Array(structure),
                    );

                    return EditorNode::Operation {
                        operation: OperationData {
                            name: name.clone(),
                            args: translated_args,
                            kind: Some("tensor".to_string()),
                            metadata: Some(metadata),
                        },
                    };
                }
            }

            // Default: simple operation
            EditorNode::operation(name, translated_args)
        }

        Expression::List(elements) => {
            let translated: Vec<EditorNode> = elements
                .iter()
                .map(|e| translate_with_context(e, ctx))
                .collect();
            EditorNode::list(translated)
        }

        // For other Expression variants, create simple representations
        Expression::Match { .. } => {
            // Match expressions don't have a visual editor representation yet
            EditorNode::object("[match]")
        }

        Expression::Quantifier { .. } => {
            // Quantifiers could be visualized, but keep simple for now
            EditorNode::object("[quantifier]")
        }

        Expression::Conditional { .. } => EditorNode::object("[if-then-else]"),

        Expression::Let { .. } => EditorNode::object("[let]"),
    }
}

/// Infer index structure from xAct-style negate() wrappers
///
/// In xAct notation: Γ(λ, -μ, -ν)
/// - Positive index → contravariant (up)
/// - Negative index (wrapped in negate()) → covariant (down)
fn infer_index_structure(args: &[Expression]) -> Vec<String> {
    args.iter()
        .map(|arg| {
            match arg {
                Expression::Operation { name, args: inner }
                    if name == "negate" && inner.len() == 1 =>
                {
                    "down".to_string() // Covariant
                }
                _ => "up".to_string(), // Contravariant
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_node_object() {
        let node = EditorNode::object("x");
        assert!(matches!(node, EditorNode::Object { object } if object == "x"));
    }

    #[test]
    fn test_editor_node_tensor() {
        let tensor = EditorNode::tensor(
            "Γ",
            vec![
                EditorNode::object("λ"),
                EditorNode::object("μ"),
                EditorNode::object("ν"),
            ],
            vec!["up", "down", "down"],
        );

        match tensor {
            EditorNode::Operation { operation } => {
                // New structure: name is "tensor", symbol is in args[0]
                assert_eq!(operation.name, "tensor");
                assert_eq!(operation.kind, Some("tensor".to_string()));
                assert!(operation.metadata.is_some());

                // Symbol should be first arg
                assert_eq!(operation.args.len(), 4); // symbol + 3 indices
                match &operation.args[0] {
                    EditorNode::Object { object } => assert_eq!(object, "Γ"),
                    _ => panic!("Expected symbol as Object"),
                }

                let metadata = operation.metadata.unwrap();
                let structure = metadata.get("indexStructure").unwrap();
                assert!(structure.is_array());
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_translate_simple_expression() {
        let expr = Expression::Object("x".to_string());
        let node = translate_to_editor(&expr);
        assert!(matches!(node, EditorNode::Object { object } if object == "x"));
    }

    #[test]
    fn test_translate_tensor_with_negate() {
        // Γ(λ, -μ, -ν) in xAct style
        let expr = Expression::Operation {
            name: "Γ".to_string(),
            args: vec![
                Expression::Object("λ".to_string()),
                Expression::Operation {
                    name: "negate".to_string(),
                    args: vec![Expression::Object("μ".to_string())],
                },
                Expression::Operation {
                    name: "negate".to_string(),
                    args: vec![Expression::Object("ν".to_string())],
                },
            ],
        };

        let ctx = TranslationContext::with_default_tensors();
        let node = translate_with_context(&expr, &ctx);

        match node {
            EditorNode::Operation { operation } => {
                assert_eq!(operation.name, "Γ");
                assert_eq!(operation.kind, Some("tensor".to_string()));

                let metadata = operation.metadata.unwrap();
                let structure = metadata.get("indexStructure").unwrap().as_array().unwrap();
                assert_eq!(structure[0], "up");
                assert_eq!(structure[1], "down");
                assert_eq!(structure[2], "down");
            }
            _ => panic!("Expected Operation"),
        }
    }
}
