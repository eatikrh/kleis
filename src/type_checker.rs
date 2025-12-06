///! Type Checker - Connects Type Context Registry to HM Inference
///!
///! This module bridges:
///! - TypeContextBuilder (knows which types support which operations)
///! - TypeInference (Hindley-Milner algorithm)
///!
///! Together they provide:
///! - Type checking with user-defined types
///! - Helpful error messages
///! - Suggestions based on available operations

use crate::ast::Expression;
use crate::type_context::TypeContextBuilder;
use crate::type_inference::{TypeInference, Type};
use crate::kleis_ast::TypeExpr;

/// Result of type checking
#[derive(Debug, Clone)]
pub enum TypeCheckResult {
    /// Successfully inferred type
    Success(Type),
    
    /// Type error with helpful message
    Error {
        message: String,
        suggestion: Option<String>,
    },
    
    /// Type is polymorphic (needs more context)
    Polymorphic {
        type_var: Type,
        available_types: Vec<String>,
    },
}

/// Type checker that combines registry and HM inference
pub struct TypeChecker {
    /// Type context with operation registry
    context_builder: TypeContextBuilder,
    
    /// HM inference engine
    inference: TypeInference,
}

impl TypeChecker {
    /// Create new type checker
    pub fn new() -> Self {
        TypeChecker {
            context_builder: TypeContextBuilder::new(),
            inference: TypeInference::new(),
        }
    }
    
    /// Create from parsed program
    pub fn from_program(program: crate::kleis_ast::Program) -> Result<Self, String> {
        let context_builder = TypeContextBuilder::from_program(program)?;
        
        Ok(TypeChecker {
            context_builder,
            inference: TypeInference::new(),
        })
    }
    
    /// Bind a variable to a type
    pub fn bind(&mut self, name: &str, type_expr: &TypeExpr) {
        // Convert TypeExpr to Type
        let ty = self.type_expr_to_type(type_expr);
        self.inference.bind(name.to_string(), ty);
    }
    
    fn type_expr_to_type(&self, type_expr: &TypeExpr) -> Type {
        match type_expr {
            TypeExpr::Named(name) => {
                // Map named types to Type enum
                match name.as_str() {
                    "ℝ" | "Real" => Type::Scalar,
                    // TODO: Add more mappings
                    _ => Type::Scalar, // Default for now
                }
            }
            TypeExpr::Parametric(name, _params) => {
                // TODO: Handle parametric types
                match name.as_str() {
                    "Vector" => Type::Vector(3), // Default dimension
                    "Matrix" => Type::Matrix(3, 3),
                    _ => Type::Scalar,
                }
            }
            _ => Type::Scalar, // Default
        }
    }
    
    /// Type check an expression with helpful error messages
    pub fn check(&mut self, expr: &Expression) -> TypeCheckResult {
        // First, try HM inference
        match self.inference.infer_and_solve(expr) {
            Ok(ty) => TypeCheckResult::Success(ty),
            Err(e) => {
                // Check if error is due to unsupported operation
                if let Some(suggestion) = self.generate_suggestion(expr, &e) {
                    TypeCheckResult::Error {
                        message: e,
                        suggestion: Some(suggestion),
                    }
                } else {
                    TypeCheckResult::Error {
                        message: e,
                        suggestion: None,
                    }
                }
            }
        }
    }
    
    /// Generate helpful suggestion based on error
    fn generate_suggestion(&self, expr: &Expression, _error: &str) -> Option<String> {
        // Check if it's an operation that's not supported for a type
        if let Expression::Operation { name, args } = expr {
            if args.is_empty() {
                return None;
            }
            
            // Get the first argument's type (if we can infer it)
            // For now, simplified: check if operation exists in registry
            if let Some(structure) = self.context_builder.registry().structure_for_operation(name) {
                let types = self.context_builder.types_supporting(name);
                if types.is_empty() {
                    return Some(format!(
                        "Operation '{}' defined in structure '{}' but no types implement it yet",
                        name, structure
                    ));
                }
                
                return Some(format!(
                    "Operation '{}' is available for types: {}",
                    name,
                    types.join(", ")
                ));
            }
        }
        
        None
    }
    
    /// Check if a specific type supports an operation
    pub fn type_supports_operation(&self, type_name: &str, operation_name: &str) -> bool {
        self.context_builder.supports_operation(type_name, operation_name)
    }
    
    /// Get types that support an operation
    pub fn types_supporting(&self, operation_name: &str) -> Vec<String> {
        self.context_builder.types_supporting(operation_name)
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kleis_parser::{parse_kleis_program, parse_kleis};
    
    #[test]
    fn test_basic_type_checking() {
        // Parse stdlib
        let stdlib = r#"
            structure Numeric(N) {
                operation abs : N → N
            }
            
            implements Numeric(ℝ) {
                operation abs = builtin_abs
            }
        "#;
        
        let program = parse_kleis_program(stdlib).unwrap();
        let mut checker = TypeChecker::from_program(program).unwrap();
        
        // Bind x to ℝ
        checker.bind("x", &TypeExpr::Named("ℝ".to_string()));
        
        // Check: abs(x)
        let expr = parse_kleis("abs(x)").unwrap();
        
        // Query support (doesn't actually type check yet, just queries registry)
        assert!(checker.type_supports_operation("ℝ", "abs"));
    }
    
    #[test]
    fn test_operation_support_query() {
        let stdlib = r#"
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
        
        let program = parse_kleis_program(stdlib).unwrap();
        let checker = TypeChecker::from_program(program).unwrap();
        
        // ℝ supports abs, not card
        assert!(checker.type_supports_operation("ℝ", "abs"));
        assert!(!checker.type_supports_operation("ℝ", "card"));
        
        // Set supports card, not abs
        assert!(checker.type_supports_operation("Set(T)", "card"));
        assert!(!checker.type_supports_operation("Set(T)", "abs"));
    }
    
    #[test]
    fn test_types_supporting_query() {
        let stdlib = r#"
            structure Numeric(N) {
                operation abs : N → N
            }
            
            implements Numeric(ℝ) {
                operation abs = builtin_abs
            }
            
            implements Numeric(ℂ) {
                operation abs = complex_modulus
            }
        "#;
        
        let program = parse_kleis_program(stdlib).unwrap();
        let checker = TypeChecker::from_program(program).unwrap();
        
        // Query which types support abs
        let types = checker.types_supporting("abs");
        assert_eq!(types.len(), 2);
        assert!(types.contains(&"ℝ".to_string()));
        assert!(types.contains(&"ℂ".to_string()));
    }
}

