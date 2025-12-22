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
//! ## Rational Number Operations (ℚ)
//!
//! | Operator | Arg Types | Lowered To |
//! |----------|-----------|------------|
//! | `plus`   | ℚ × ℚ     | `rational_add` |
//! | `plus`   | ℚ × ℤ     | `rational_add(_, lift)` |
//! | `plus`   | ℤ × ℚ     | `rational_add(lift, _)` |
//! | `times`  | ℚ × ℚ     | `rational_mul` |
//! | `minus`  | ℚ × ℚ     | `rational_sub` |
//! | `divide` | ℚ × ℚ     | `rational_div` |
//! | `neg`    | ℚ         | `neg_rational` |
//!
//! Where `lift(n : ℤ) = rational(n, 1)`
//!
//! ## Type Promotion Hierarchy
//!
//! ℕ → ℤ → ℚ → ℝ → ℂ
//!
//! When mixing types, the result is promoted to the "larger" type.
//!
//! See: docs/plans/operator-overloading.md

use crate::ast::Expression;
use crate::type_context::TypeContextBuilder;
use crate::type_inference::Type;
use crate::typed_ast::TypedExpr;

/// Semantic Lowering transformer
///
/// Rewrites generic operators to type-specific operations based on operand types.
///
/// ADR-016 Refactoring: Now supports registry-based lowering via `lower_with_context`.
/// The `lower` method uses hardcoded patterns for backward compatibility.
pub struct SemanticLowering {
    /// Optional context for registry-based lowering
    context: Option<TypeContextBuilder>,
}

impl SemanticLowering {
    /// Create a new semantic lowering transformer (backward compatible, uses hardcoded patterns)
    pub fn new() -> Self {
        SemanticLowering { context: None }
    }

    /// Create a semantic lowering transformer with registry context (ADR-016 compliant)
    pub fn with_context(context: TypeContextBuilder) -> Self {
        SemanticLowering {
            context: Some(context),
        }
    }

    /// Generic arithmetic lowering using registry (ADR-016)
    ///
    /// Instead of 60+ hardcoded patterns, uses:
    /// 1. find_common_supertype(t1, t2) → target type
    /// 2. get_lift_function(from, to) → lift function name
    /// 3. get_lowered_op_name(op, type) → type-specific operation
    fn lower_arithmetic_generic(
        &self,
        name: &str,
        t1: &Type,
        t2: &Type,
        arg1: &Expression,
        arg2: &Expression,
    ) -> Option<Expression> {
        let ctx = self.context.as_ref()?;

        // Get type constructors
        let c1 = self.get_type_constructor(t1)?;
        let c2 = self.get_type_constructor(t2)?;

        // Find common supertype
        let target = ctx.find_common_supertype(&c1, &c2)?;

        // Lift arguments if needed
        let lifted_arg1 = if c1 != target {
            if let Some(lift_fn) = ctx.get_lift_function(&c1, &target) {
                Expression::Operation {
                    name: lift_fn,
                    args: vec![arg1.clone()],
                }
            } else {
                arg1.clone()
            }
        } else {
            arg1.clone()
        };

        let lifted_arg2 = if c2 != target {
            if let Some(lift_fn) = ctx.get_lift_function(&c2, &target) {
                Expression::Operation {
                    name: lift_fn,
                    args: vec![arg2.clone()],
                }
            } else {
                arg2.clone()
            }
        } else {
            arg2.clone()
        };

        // Get type-specific operation name
        let lowered_op = ctx.get_lowered_op_name(name, &target);

        Some(Expression::Operation {
            name: lowered_op,
            args: vec![lifted_arg1, lifted_arg2],
        })
    }

    /// Extract type constructor name from Type
    fn get_type_constructor(&self, ty: &Type) -> Option<String> {
        match ty {
            Type::Data { constructor, .. } => Some(constructor.clone()),
            _ => None,
        }
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

        // ADR-016: Try generic registry-based lowering first for binary arithmetic
        if matches!(
            name,
            "plus" | "minus" | "times" | "divide" | "scalar_divide"
        ) && arg_types.len() == 2
        {
            if let Some(lowered) = self.lower_arithmetic_generic(
                name,
                arg_types[0],
                arg_types[1],
                &lowered_args[0],
                &lowered_args[1],
            ) {
                return lowered;
            }
            // Fall through to hardcoded patterns if generic fails
        }

        match (name, arg_types.as_slice()) {
            // ============================================
            // COMPLEX NUMBER OPERATIONS (fallback for no context)
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
            // RATIONAL NUMBER OPERATIONS
            // ============================================

            // plus(ℚ, ℚ) → rational_add
            ("plus", [t1, t2]) if self.is_rational(t1) && self.is_rational(t2) => {
                Expression::Operation {
                    name: "rational_add".to_string(),
                    args: lowered_args.to_vec(),
                }
            }

            // plus(ℚ, ℤ) or plus(ℚ, ℕ) → rational_add(_, lift)
            ("plus", [t1, t2]) if self.is_rational(t1) && (self.is_int(t2) || self.is_nat(t2)) => {
                Expression::Operation {
                    name: "rational_add".to_string(),
                    args: vec![
                        lowered_args[0].clone(),
                        self.lift_to_rational(&lowered_args[1]),
                    ],
                }
            }

            // plus(ℤ, ℚ) or plus(ℕ, ℚ) → rational_add(lift, _)
            ("plus", [t1, t2]) if (self.is_int(t1) || self.is_nat(t1)) && self.is_rational(t2) => {
                Expression::Operation {
                    name: "rational_add".to_string(),
                    args: vec![
                        self.lift_to_rational(&lowered_args[0]),
                        lowered_args[1].clone(),
                    ],
                }
            }

            // plus(ℚ, ℝ) → plus as real (Z3 Real = ℚ, so this is compatible)
            ("plus", [t1, t2]) if self.is_rational(t1) && self.is_real(t2) => {
                Expression::Operation {
                    name: "plus".to_string(),
                    args: lowered_args.to_vec(),
                }
            }

            // plus(ℝ, ℚ) → plus as real
            ("plus", [t1, t2]) if self.is_real(t1) && self.is_rational(t2) => {
                Expression::Operation {
                    name: "plus".to_string(),
                    args: lowered_args.to_vec(),
                }
            }

            // plus(ℚ, ℂ) → complex_add(lift_to_complex(lift_to_real), _)
            ("plus", [t1, t2]) if self.is_rational(t1) && self.is_complex(t2) => {
                Expression::Operation {
                    name: "complex_add".to_string(),
                    args: vec![
                        self.lift_to_complex(&lowered_args[0]),
                        lowered_args[1].clone(),
                    ],
                }
            }

            // plus(ℂ, ℚ) → complex_add(_, lift_to_complex)
            ("plus", [t1, t2]) if self.is_complex(t1) && self.is_rational(t2) => {
                Expression::Operation {
                    name: "complex_add".to_string(),
                    args: vec![
                        lowered_args[0].clone(),
                        self.lift_to_complex(&lowered_args[1]),
                    ],
                }
            }

            // minus(ℚ, ℚ) → rational_sub
            ("minus", [t1, t2]) if self.is_rational(t1) && self.is_rational(t2) => {
                Expression::Operation {
                    name: "rational_sub".to_string(),
                    args: lowered_args.to_vec(),
                }
            }

            // minus(ℚ, ℤ/ℕ) → rational_sub(_, lift)
            ("minus", [t1, t2]) if self.is_rational(t1) && (self.is_int(t2) || self.is_nat(t2)) => {
                Expression::Operation {
                    name: "rational_sub".to_string(),
                    args: vec![
                        lowered_args[0].clone(),
                        self.lift_to_rational(&lowered_args[1]),
                    ],
                }
            }

            // minus(ℤ/ℕ, ℚ) → rational_sub(lift, _)
            ("minus", [t1, t2]) if (self.is_int(t1) || self.is_nat(t1)) && self.is_rational(t2) => {
                Expression::Operation {
                    name: "rational_sub".to_string(),
                    args: vec![
                        self.lift_to_rational(&lowered_args[0]),
                        lowered_args[1].clone(),
                    ],
                }
            }

            // times(ℚ, ℚ) → rational_mul
            ("times", [t1, t2]) if self.is_rational(t1) && self.is_rational(t2) => {
                Expression::Operation {
                    name: "rational_mul".to_string(),
                    args: lowered_args.to_vec(),
                }
            }

            // times(ℚ, ℤ/ℕ) → rational_mul(_, lift)
            ("times", [t1, t2]) if self.is_rational(t1) && (self.is_int(t2) || self.is_nat(t2)) => {
                Expression::Operation {
                    name: "rational_mul".to_string(),
                    args: vec![
                        lowered_args[0].clone(),
                        self.lift_to_rational(&lowered_args[1]),
                    ],
                }
            }

            // times(ℤ/ℕ, ℚ) → rational_mul(lift, _)
            ("times", [t1, t2]) if (self.is_int(t1) || self.is_nat(t1)) && self.is_rational(t2) => {
                Expression::Operation {
                    name: "rational_mul".to_string(),
                    args: vec![
                        self.lift_to_rational(&lowered_args[0]),
                        lowered_args[1].clone(),
                    ],
                }
            }

            // times(ℚ, ℂ) → complex_mul(lift, _)
            ("times", [t1, t2]) if self.is_rational(t1) && self.is_complex(t2) => {
                Expression::Operation {
                    name: "complex_mul".to_string(),
                    args: vec![
                        self.lift_to_complex(&lowered_args[0]),
                        lowered_args[1].clone(),
                    ],
                }
            }

            // times(ℂ, ℚ) → complex_mul(_, lift)
            ("times", [t1, t2]) if self.is_complex(t1) && self.is_rational(t2) => {
                Expression::Operation {
                    name: "complex_mul".to_string(),
                    args: vec![
                        lowered_args[0].clone(),
                        self.lift_to_complex(&lowered_args[1]),
                    ],
                }
            }

            // divide(ℚ, ℚ) → rational_div
            ("divide" | "scalar_divide", [t1, t2])
                if self.is_rational(t1) && self.is_rational(t2) =>
            {
                Expression::Operation {
                    name: "rational_div".to_string(),
                    args: lowered_args.to_vec(),
                }
            }

            // divide(ℚ, ℤ/ℕ) → rational_div(_, lift)
            ("divide" | "scalar_divide", [t1, t2])
                if self.is_rational(t1) && (self.is_int(t2) || self.is_nat(t2)) =>
            {
                Expression::Operation {
                    name: "rational_div".to_string(),
                    args: vec![
                        lowered_args[0].clone(),
                        self.lift_to_rational(&lowered_args[1]),
                    ],
                }
            }

            // divide(ℤ/ℕ, ℚ) → rational_div(lift, _)
            ("divide" | "scalar_divide", [t1, t2])
                if (self.is_int(t1) || self.is_nat(t1)) && self.is_rational(t2) =>
            {
                Expression::Operation {
                    name: "rational_div".to_string(),
                    args: vec![
                        self.lift_to_rational(&lowered_args[0]),
                        lowered_args[1].clone(),
                    ],
                }
            }

            // neg(ℚ) → neg_rational
            ("neg" | "negate", [t1]) if self.is_rational(t1) => Expression::Operation {
                name: "neg_rational".to_string(),
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

    /// Check if a type is Real/Scalar (or promotable to Real: Nat, Int, Rational)
    /// Used for lowering mixed Real+Complex operations
    fn is_real(&self, ty: &Type) -> bool {
        match ty {
            Type::Data { constructor, .. } => {
                // All numeric types below Complex in the hierarchy
                matches!(constructor.as_str(), "Scalar" | "Int" | "Nat" | "Rational")
            }
            _ => false,
        }
    }

    /// Check if a type is Rational
    fn is_rational(&self, ty: &Type) -> bool {
        match ty {
            Type::Data { constructor, .. } => constructor == "Rational",
            _ => false,
        }
    }

    /// Check if a type is Int
    fn is_int(&self, ty: &Type) -> bool {
        match ty {
            Type::Data { constructor, .. } => constructor == "Int",
            _ => false,
        }
    }

    /// Check if a type is Nat
    fn is_nat(&self, ty: &Type) -> bool {
        match ty {
            Type::Data { constructor, .. } => constructor == "Nat",
            _ => false,
        }
    }

    /// Lift an integer/natural expression to rational: n → rational(n, 1)
    fn lift_to_rational(&self, expr: &Expression) -> Expression {
        Expression::Operation {
            name: "rational".to_string(),
            args: vec![expr.clone(), Expression::Const("1".to_string())],
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
