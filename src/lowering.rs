//! Semantic Lowering Pass for Operator Overloading
//!
//! Transforms typed AST by rewriting generic operators to type-specific operations.
//! This enables natural arithmetic syntax (`z1 + z2`) to work with complex numbers
//! (and future numeric types like matrices, vectors, quaternions).
//!
//! ## Architecture
//!
//! ```text
//! Parser → Expression (untyped)
//!              ↓
//! Type Inference → TypedExpr (typed)
//!              ↓
//! Semantic Lowering → Expression (lowered, explicit operations)  ← THIS MODULE
//!              ↓
//! Backend (Z3, Evaluator)
//! ```
//!
//! ## Example
//!
//! Input: `3 + 4*i` (after type inference)
//! ```text
//! TypedExpr {
//!     expr: plus(3, times(4, i)),
//!     ty: Complex,
//!     children: [
//!         TypedExpr { ty: Real, ... },   // 3
//!         TypedExpr { ty: Complex, ... } // 4*i
//!     ]
//! }
//! ```
//!
//! Output: `complex_add(complex(3, 0), complex_mul(complex(4, 0), i))`
//!
//! ## Operator Mapping
//!
//! | Operator | Arg Types | Lowered To |
//! |----------|-----------|------------|
//! | `plus`   | ℂ × ℂ     | `complex_add` |
//! | `plus`   | ℝ × ℂ     | `complex_add(lift, _)` |
//! | `plus`   | ℂ × ℝ     | `complex_add(_, lift)` |
//! | `times`  | ℂ × ℂ     | `complex_mul` |
//! | `times`  | ℝ × ℂ     | `complex_mul(lift, _)` |
//! | `times`  | ℂ × ℝ     | `complex_mul(_, lift)` |
//! | `minus`  | ℂ × ℂ     | `complex_sub` |
//! | `divide` | ℂ × ℂ     | `complex_div` |
//! | `neg`    | ℂ         | `neg_complex` |
//!
//! Where `lift(r : ℝ) = complex(r, 0)`
//!
//! See: docs/plans/operator-overloading.md

use crate::ast::Expression;
use crate::type_inference::Type;
use crate::typed_ast::TypedExpr;

/// Semantic Lowering transformer
///
/// Rewrites generic operators to type-specific operations based on operand types.
pub struct SemanticLowering;

impl SemanticLowering {
    /// Create a new semantic lowering transformer
    pub fn new() -> Self {
        SemanticLowering
    }

    /// Lower a typed expression to an untyped expression with explicit operations
    ///
    /// This is the main entry point. It traverses the typed AST and rewrites
    /// operations based on the types of their operands.
    pub fn lower(&self, typed: &TypedExpr) -> Expression {
        match &typed.expr {
            Expression::Operation { name, args: _ } => {
                // First, recursively lower all children
                let lowered_children: Vec<Expression> =
                    typed.children.iter().map(|c| self.lower(c)).collect();

                // Then check if we need to rewrite based on operand types
                self.lower_operation(name, &typed.children, &lowered_children, &typed.ty)
            }

            // Leaf nodes: just clone the expression
            Expression::Const(_)
            | Expression::String(_)
            | Expression::Object(_)
            | Expression::Placeholder { .. } => typed.expr.clone(),

            // List: recursively lower elements
            Expression::List(_) => {
                let lowered_elements: Vec<Expression> =
                    typed.children.iter().map(|c| self.lower(c)).collect();
                Expression::List(lowered_elements)
            }

            // Match: lower scrutinee and case bodies
            Expression::Match {
                scrutinee: _,
                cases,
            } => {
                // children[0] is scrutinee, children[1..] are case bodies
                let lowered_scrutinee = Box::new(self.lower(&typed.children[0]));

                let lowered_cases: Vec<crate::ast::MatchCase> = cases
                    .iter()
                    .zip(typed.children.iter().skip(1))
                    .map(|(case, typed_body)| crate::ast::MatchCase {
                        pattern: case.pattern.clone(),
                        guard: case.guard.clone(),
                        body: self.lower(typed_body),
                    })
                    .collect();

                Expression::Match {
                    scrutinee: lowered_scrutinee,
                    cases: lowered_cases,
                }
            }

            // Conditional: lower all branches
            Expression::Conditional { .. } => {
                let lowered_cond = Box::new(self.lower(&typed.children[0]));
                let lowered_then = Box::new(self.lower(&typed.children[1]));
                let lowered_else = Box::new(self.lower(&typed.children[2]));

                Expression::Conditional {
                    condition: lowered_cond,
                    then_branch: lowered_then,
                    else_branch: lowered_else,
                }
            }

            // Let: lower value and body
            Expression::Let {
                pattern,
                type_annotation,
                ..
            } => {
                let lowered_value = Box::new(self.lower(&typed.children[0]));
                let lowered_body = Box::new(self.lower(&typed.children[1]));

                Expression::Let {
                    pattern: pattern.clone(),
                    type_annotation: type_annotation.clone(),
                    value: lowered_value,
                    body: lowered_body,
                }
            }

            // Ascription: lower inner expression
            Expression::Ascription {
                type_annotation, ..
            } => {
                let lowered_inner = Box::new(self.lower(&typed.children[0]));
                Expression::Ascription {
                    expr: lowered_inner,
                    type_annotation: type_annotation.clone(),
                }
            }

            // Lambda: lower body
            Expression::Lambda { params, .. } => {
                let lowered_body = Box::new(self.lower(&typed.children[0]));
                Expression::Lambda {
                    params: params.clone(),
                    body: lowered_body,
                }
            }

            // Quantifier: lower body
            Expression::Quantifier {
                quantifier,
                variables,
                where_clause,
                ..
            } => {
                let lowered_body = Box::new(self.lower(&typed.children[0]));
                Expression::Quantifier {
                    quantifier: quantifier.clone(),
                    variables: variables.clone(),
                    where_clause: where_clause.clone(),
                    body: lowered_body,
                }
            }
        }
    }

    /// Lower an operation based on operand types
    fn lower_operation(
        &self,
        name: &str,
        typed_args: &[TypedExpr],
        lowered_args: &[Expression],
        _result_ty: &Type,
    ) -> Expression {
        // Get operand types
        let arg_types: Vec<&Type> = typed_args.iter().map(|a| &a.ty).collect();

        match (name, arg_types.as_slice()) {
            // ============================================
            // COMPLEX NUMBER OPERATIONS
            // ============================================

            // plus(ℂ, ℂ) → complex_add
            ("plus", [t1, t2]) if self.is_complex(t1) && self.is_complex(t2) => {
                Expression::Operation {
                    name: "complex_add".to_string(),
                    args: lowered_args.to_vec(),
                }
            }

            // plus(ℝ, ℂ) → complex_add(lift, _)
            ("plus", [t1, t2]) if self.is_real(t1) && self.is_complex(t2) => {
                Expression::Operation {
                    name: "complex_add".to_string(),
                    args: vec![
                        self.lift_to_complex(&lowered_args[0]),
                        lowered_args[1].clone(),
                    ],
                }
            }

            // plus(ℂ, ℝ) → complex_add(_, lift)
            ("plus", [t1, t2]) if self.is_complex(t1) && self.is_real(t2) => {
                Expression::Operation {
                    name: "complex_add".to_string(),
                    args: vec![
                        lowered_args[0].clone(),
                        self.lift_to_complex(&lowered_args[1]),
                    ],
                }
            }

            // minus(ℂ, ℂ) → complex_sub
            ("minus", [t1, t2]) if self.is_complex(t1) && self.is_complex(t2) => {
                Expression::Operation {
                    name: "complex_sub".to_string(),
                    args: lowered_args.to_vec(),
                }
            }

            // minus(ℝ, ℂ) → complex_sub(lift, _)
            ("minus", [t1, t2]) if self.is_real(t1) && self.is_complex(t2) => {
                Expression::Operation {
                    name: "complex_sub".to_string(),
                    args: vec![
                        self.lift_to_complex(&lowered_args[0]),
                        lowered_args[1].clone(),
                    ],
                }
            }

            // minus(ℂ, ℝ) → complex_sub(_, lift)
            ("minus", [t1, t2]) if self.is_complex(t1) && self.is_real(t2) => {
                Expression::Operation {
                    name: "complex_sub".to_string(),
                    args: vec![
                        lowered_args[0].clone(),
                        self.lift_to_complex(&lowered_args[1]),
                    ],
                }
            }

            // times(ℂ, ℂ) → complex_mul
            ("times", [t1, t2]) if self.is_complex(t1) && self.is_complex(t2) => {
                Expression::Operation {
                    name: "complex_mul".to_string(),
                    args: lowered_args.to_vec(),
                }
            }

            // times(ℝ, ℂ) → complex_mul(lift, _)
            ("times", [t1, t2]) if self.is_real(t1) && self.is_complex(t2) => {
                Expression::Operation {
                    name: "complex_mul".to_string(),
                    args: vec![
                        self.lift_to_complex(&lowered_args[0]),
                        lowered_args[1].clone(),
                    ],
                }
            }

            // times(ℂ, ℝ) → complex_mul(_, lift)
            ("times", [t1, t2]) if self.is_complex(t1) && self.is_real(t2) => {
                Expression::Operation {
                    name: "complex_mul".to_string(),
                    args: vec![
                        lowered_args[0].clone(),
                        self.lift_to_complex(&lowered_args[1]),
                    ],
                }
            }

            // divide(ℂ, ℂ) → complex_div
            ("divide" | "scalar_divide", [t1, t2])
                if self.is_complex(t1) && self.is_complex(t2) =>
            {
                Expression::Operation {
                    name: "complex_div".to_string(),
                    args: lowered_args.to_vec(),
                }
            }

            // divide(ℝ, ℂ) → complex_div(lift, _)
            ("divide" | "scalar_divide", [t1, t2]) if self.is_real(t1) && self.is_complex(t2) => {
                Expression::Operation {
                    name: "complex_div".to_string(),
                    args: vec![
                        self.lift_to_complex(&lowered_args[0]),
                        lowered_args[1].clone(),
                    ],
                }
            }

            // divide(ℂ, ℝ) → complex_div(_, lift)
            ("divide" | "scalar_divide", [t1, t2]) if self.is_complex(t1) && self.is_real(t2) => {
                Expression::Operation {
                    name: "complex_div".to_string(),
                    args: vec![
                        lowered_args[0].clone(),
                        self.lift_to_complex(&lowered_args[1]),
                    ],
                }
            }

            // neg(ℂ) → neg_complex
            ("neg" | "negate", [t1]) if self.is_complex(t1) => Expression::Operation {
                name: "neg_complex".to_string(),
                args: lowered_args.to_vec(),
            },

            // ============================================
            // DEFAULT: Keep operation as-is
            // ============================================
            _ => Expression::Operation {
                name: name.to_string(),
                args: lowered_args.to_vec(),
            },
        }
    }

    /// Lift a real expression to complex: r → complex(r, 0)
    fn lift_to_complex(&self, expr: &Expression) -> Expression {
        Expression::Operation {
            name: "complex".to_string(),
            args: vec![expr.clone(), Expression::Const("0".to_string())],
        }
    }

    /// Check if a type is Complex
    fn is_complex(&self, ty: &Type) -> bool {
        match ty {
            Type::Data { constructor, .. } => constructor == "Complex",
            _ => false,
        }
    }

    /// Check if a type is Real/Scalar
    fn is_real(&self, ty: &Type) -> bool {
        match ty {
            Type::Data { constructor, .. } => constructor == "Scalar",
            _ => false,
        }
    }
}

impl Default for SemanticLowering {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_inference::TypeInference;
    use crate::typed_ast::TypedExpr;

    /// Helper: create a TypedExpr for a complex-typed expression
    fn complex_typed(expr: Expression) -> TypedExpr {
        TypedExpr::leaf(
            expr,
            Type::Data {
                type_name: "Type".to_string(),
                constructor: "Complex".to_string(),
                args: vec![],
            },
        )
    }

    /// Helper: create a TypedExpr for a real-typed expression
    fn real_typed(expr: Expression) -> TypedExpr {
        TypedExpr::leaf(expr, Type::scalar())
    }

    #[test]
    fn test_lower_complex_addition() {
        let lowering = SemanticLowering::new();

        // Create: z1 + z2 (both complex)
        let z1 = Expression::Object("z1".to_string());
        let z2 = Expression::Object("z2".to_string());
        let expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![z1.clone(), z2.clone()],
        };

        // Create typed AST with complex types
        let typed = TypedExpr::node(
            expr,
            Type::Data {
                type_name: "Type".to_string(),
                constructor: "Complex".to_string(),
                args: vec![],
            },
            vec![complex_typed(z1), complex_typed(z2)],
        );

        let lowered = lowering.lower(&typed);

        // Should be rewritten to complex_add
        match lowered {
            Expression::Operation { name, args } => {
                assert_eq!(name, "complex_add");
                assert_eq!(args.len(), 2);
                println!("✅ plus(ℂ, ℂ) → complex_add");
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_lower_real_plus_complex() {
        let lowering = SemanticLowering::new();

        // Create: 3 + z (real + complex)
        let three = Expression::Const("3".to_string());
        let z = Expression::Object("z".to_string());
        let expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![three.clone(), z.clone()],
        };

        // Create typed AST
        let typed = TypedExpr::node(
            expr,
            Type::Data {
                type_name: "Type".to_string(),
                constructor: "Complex".to_string(),
                args: vec![],
            },
            vec![real_typed(three), complex_typed(z)],
        );

        let lowered = lowering.lower(&typed);

        // Should be: complex_add(complex(3, 0), z)
        match lowered {
            Expression::Operation { name, args } => {
                assert_eq!(name, "complex_add");
                assert_eq!(args.len(), 2);

                // First arg should be lifted: complex(3, 0)
                match &args[0] {
                    Expression::Operation {
                        name: lift_name,
                        args: lift_args,
                    } => {
                        assert_eq!(lift_name, "complex");
                        assert_eq!(lift_args.len(), 2);
                        assert_eq!(lift_args[0], Expression::Const("3".to_string()));
                        assert_eq!(lift_args[1], Expression::Const("0".to_string()));
                    }
                    _ => panic!("Expected lifted complex"),
                }

                println!("✅ plus(ℝ, ℂ) → complex_add(lift, _)");
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_lower_complex_multiplication() {
        let lowering = SemanticLowering::new();

        // Create: z1 * z2 (both complex)
        let z1 = Expression::Object("z1".to_string());
        let z2 = Expression::Object("z2".to_string());
        let expr = Expression::Operation {
            name: "times".to_string(),
            args: vec![z1.clone(), z2.clone()],
        };

        let typed = TypedExpr::node(
            expr,
            Type::Data {
                type_name: "Type".to_string(),
                constructor: "Complex".to_string(),
                args: vec![],
            },
            vec![complex_typed(z1), complex_typed(z2)],
        );

        let lowered = lowering.lower(&typed);

        match lowered {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "complex_mul");
                println!("✅ times(ℂ, ℂ) → complex_mul");
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_lower_complex_negation() {
        let lowering = SemanticLowering::new();

        // Create: -z (negate complex)
        let z = Expression::Object("z".to_string());
        let expr = Expression::Operation {
            name: "neg".to_string(),
            args: vec![z.clone()],
        };

        let typed = TypedExpr::node(
            expr,
            Type::Data {
                type_name: "Type".to_string(),
                constructor: "Complex".to_string(),
                args: vec![],
            },
            vec![complex_typed(z)],
        );

        let lowered = lowering.lower(&typed);

        match lowered {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "neg_complex");
                println!("✅ neg(ℂ) → neg_complex");
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_lower_preserves_real_operations() {
        let lowering = SemanticLowering::new();

        // Create: 1 + 2 (both real - should not be rewritten)
        let one = Expression::Const("1".to_string());
        let two = Expression::Const("2".to_string());
        let expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![one.clone(), two.clone()],
        };

        let typed = TypedExpr::node(
            expr.clone(),
            Type::scalar(),
            vec![real_typed(one), real_typed(two)],
        );

        let lowered = lowering.lower(&typed);

        // Should stay as "plus" (not rewritten to complex_add)
        match lowered {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "plus");
                println!("✅ plus(ℝ, ℝ) stays as plus (not rewritten)");
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_lower_nested_expression() {
        let lowering = SemanticLowering::new();

        // Create: (z1 + z2) * z3 (nested complex operations)
        let z1 = Expression::Object("z1".to_string());
        let z2 = Expression::Object("z2".to_string());
        let z3 = Expression::Object("z3".to_string());

        let inner = Expression::Operation {
            name: "plus".to_string(),
            args: vec![z1.clone(), z2.clone()],
        };

        let outer = Expression::Operation {
            name: "times".to_string(),
            args: vec![inner.clone(), z3.clone()],
        };

        let complex_ty = Type::Data {
            type_name: "Type".to_string(),
            constructor: "Complex".to_string(),
            args: vec![],
        };

        // Build typed AST
        let typed_inner = TypedExpr::node(
            inner,
            complex_ty.clone(),
            vec![complex_typed(z1), complex_typed(z2)],
        );

        let typed_outer = TypedExpr::node(
            outer,
            complex_ty.clone(),
            vec![typed_inner, complex_typed(z3)],
        );

        let lowered = lowering.lower(&typed_outer);

        // Outer should be complex_mul
        match &lowered {
            Expression::Operation { name, args } => {
                assert_eq!(name, "complex_mul");

                // Inner (first arg) should be complex_add
                match &args[0] {
                    Expression::Operation {
                        name: inner_name, ..
                    } => {
                        assert_eq!(inner_name, "complex_add");
                        println!("✅ Nested: (z1 + z2) * z3 → complex_mul(complex_add(...), z3)");
                    }
                    _ => panic!("Expected inner operation"),
                }
            }
            _ => panic!("Expected Operation"),
        }
    }

    #[test]
    fn test_integration_with_type_inference() {
        // This test shows the full pipeline: parse → infer → lower
        let mut inference = TypeInference::new();

        // Create: 1 + 2 (will infer as Scalar)
        let expr = Expression::Operation {
            name: "plus".to_string(),
            args: vec![
                Expression::Const("1".to_string()),
                Expression::Const("2".to_string()),
            ],
        };

        // Step 1: Type inference
        let typed = inference.infer_typed(&expr, None).unwrap();

        // Step 2: Lowering
        let lowering = SemanticLowering::new();
        let lowered = lowering.lower(&typed);

        // Should stay as "plus" since operands are Scalar
        match lowered {
            Expression::Operation { name, .. } => {
                assert_eq!(name, "plus");
                println!("✅ Full pipeline: 1 + 2 inferred and lowered correctly");
            }
            _ => panic!("Expected Operation"),
        }
    }
}
