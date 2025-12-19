//! Typed Abstract Syntax Tree for Operator Overloading
//!
//! This module provides a typed AST representation where each expression node
//! is annotated with its inferred type. This enables type-directed transformations
//! like operator overloading (semantic lowering).
//!
//! ## Architecture
//!
//! ```text
//! Parser → Expression (untyped)
//!                ↓
//! Type Inference → TypedExpr (typed)
//!                ↓
//! Semantic Lowering → Expression (lowered, explicit operations)
//!                ↓
//! Backend (Z3, Evaluator)
//! ```
//!
//! ## Design Decision
//!
//! We use a wrapper struct (`TypedExpr`) rather than adding type annotations
//! directly to `Expression`. This keeps the core AST clean and allows typed
//! and untyped representations to coexist during the transition.
//!
//! See: docs/plans/operator-overloading.md

use crate::ast::Expression;
use crate::type_inference::Type;

/// A typed expression: pairs an Expression with its inferred Type
///
/// The `children` field contains typed versions of all sub-expressions,
/// enabling type-directed transformations that need to inspect operand types.
///
/// ## Example
///
/// For the expression `3 + 4*i`:
///
/// ```text
/// TypedExpr {
///     expr: Operation { name: "plus", args: [3, times(4, i)] },
///     ty: Complex,
///     children: [
///         TypedExpr { expr: Const("3"), ty: Real, children: [] },
///         TypedExpr {
///             expr: Operation { name: "times", args: [4, i] },
///             ty: Complex,
///             children: [
///                 TypedExpr { expr: Const("4"), ty: Real, children: [] },
///                 TypedExpr { expr: Object("i"), ty: Complex, children: [] },
///             ]
///         }
///     ]
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TypedExpr {
    /// The original expression
    pub expr: Expression,
    /// The inferred type of this expression
    pub ty: Type,
    /// Typed sub-expressions (for operations, lists, etc.)
    pub children: Vec<TypedExpr>,
}

impl TypedExpr {
    /// Create a new typed expression with no children (leaf node)
    pub fn leaf(expr: Expression, ty: Type) -> Self {
        TypedExpr {
            expr,
            ty,
            children: vec![],
        }
    }

    /// Create a new typed expression with children
    pub fn node(expr: Expression, ty: Type, children: Vec<TypedExpr>) -> Self {
        TypedExpr { expr, ty, children }
    }

    /// Check if this is a leaf node (no children)
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Get the type of this expression
    pub fn get_type(&self) -> &Type {
        &self.ty
    }

    /// Check if this expression has Complex type
    pub fn is_complex(&self) -> bool {
        matches!(
            &self.ty,
            Type::Data { constructor, .. } if constructor == "Complex"
        )
    }

    /// Check if this expression has Real/Scalar type
    pub fn is_real(&self) -> bool {
        matches!(
            &self.ty,
            Type::Data { constructor, .. } if constructor == "Scalar"
        )
    }

    /// Check if this is an operation node
    pub fn is_operation(&self) -> bool {
        matches!(&self.expr, Expression::Operation { .. })
    }

    /// Get operation name if this is an operation node
    pub fn operation_name(&self) -> Option<&str> {
        match &self.expr {
            Expression::Operation { name, .. } => Some(name),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_expr_leaf() {
        let expr = Expression::Const("42".to_string());
        let ty = Type::scalar();
        let typed = TypedExpr::leaf(expr.clone(), ty.clone());

        assert!(typed.is_leaf());
        assert_eq!(typed.get_type(), &ty);
        assert!(typed.is_real());
        assert!(!typed.is_complex());
    }

    #[test]
    fn test_typed_expr_node() {
        let const_expr = Expression::Const("1".to_string());
        let child = TypedExpr::leaf(const_expr.clone(), Type::scalar());

        let op_expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![const_expr.clone(), const_expr],
        };
        let typed = TypedExpr::node(op_expr, Type::scalar(), vec![child.clone(), child]);

        assert!(!typed.is_leaf());
        assert_eq!(typed.children.len(), 2);
        assert!(typed.is_operation());
        assert_eq!(typed.operation_name(), Some("plus"));
    }

    #[test]
    fn test_typed_expr_complex() {
        let expr = Expression::Object("z".to_string());
        let ty = Type::Data {
            type_name: "Type".to_string(),
            constructor: "Complex".to_string(),
            args: vec![],
        };
        let typed = TypedExpr::leaf(expr, ty);

        assert!(typed.is_complex());
        assert!(!typed.is_real());
    }

    /// Test that infer_typed produces correct typed AST for an operation
    #[test]
    fn test_infer_typed_addition() {
        use crate::type_inference::TypeInference;

        // Create: 1 + 2
        let expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Const("1".to_string()),
                Expression::Const("2".to_string()),
            ],
        };

        let mut inference = TypeInference::new();

        let typed = inference.infer_typed(&expr, None).unwrap();

        // The expression should be an operation
        assert!(typed.is_operation());
        assert_eq!(typed.operation_name(), Some("plus"));

        // Should have 2 children (the operands)
        assert_eq!(typed.children.len(), 2);

        // Both children should be scalars (constants infer to Scalar)
        assert!(
            typed.children[0].is_real(),
            "First operand should be Scalar"
        );
        assert!(
            typed.children[1].is_real(),
            "Second operand should be Scalar"
        );

        // Note: For operations, type may be a fresh type variable initially,
        // which gets resolved after constraint solving. Here we just check
        // that a type was assigned and children are correct.
        println!(
            "✅ infer_typed correctly types: 1 + 2 (result type: {:?})",
            typed.ty
        );
    }

    /// Test infer_typed with nested operations
    #[test]
    fn test_infer_typed_nested() {
        use crate::type_inference::TypeInference;

        // Create: (1 + 2) * 3
        let inner = Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Const("1".to_string()),
                Expression::Const("2".to_string()),
            ],
        };
        let expr = Expression::Operation {
            name: "times".to_string(),
            args: vec![inner, Expression::Const("3".to_string())],
        };

        let mut inference = TypeInference::new();

        let typed = inference.infer_typed(&expr, None).unwrap();

        // Outer operation: times
        assert_eq!(typed.operation_name(), Some("times"));
        assert_eq!(typed.children.len(), 2);

        // First child should be the inner operation (plus)
        assert_eq!(typed.children[0].operation_name(), Some("plus"));
        assert_eq!(typed.children[0].children.len(), 2);

        // Second child should be a constant (leaf)
        assert!(typed.children[1].is_leaf());

        println!("✅ infer_typed correctly types nested: (1 + 2) * 3");
    }

    /// Test infer_typed preserves expression structure
    #[test]
    fn test_infer_typed_preserves_expression() {
        use crate::type_inference::TypeInference;

        let expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Const("42".to_string()),
                Expression::Object("x".to_string()),
            ],
        };

        let mut inference = TypeInference::new();

        let typed = inference.infer_typed(&expr, None).unwrap();

        // The original expression should be preserved
        assert_eq!(typed.expr, expr);

        // Children expressions should also be preserved
        assert_eq!(typed.children[0].expr, Expression::Const("42".to_string()));
        assert_eq!(typed.children[1].expr, Expression::Object("x".to_string()));

        println!("✅ infer_typed preserves original expression structure");
    }
}
