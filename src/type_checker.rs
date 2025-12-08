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
use crate::kleis_ast::TypeExpr;
use crate::type_context::TypeContextBuilder;
use crate::type_inference::{Type, TypeInference};

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
    /// Create new type checker (empty context)
    pub fn new() -> Self {
        TypeChecker {
            context_builder: TypeContextBuilder::new(),
            inference: TypeInference::new(),
        }
    }

    /// Create type checker with standard library loaded
    /// This is the recommended way to create a type checker for most use cases.
    ///
    /// **ADR-021 Update:** Now loads data types FIRST, then structures.
    ///
    /// **Loading Order:**
    /// 1. stdlib/types.kleis (data type definitions) - NEW in ADR-021
    /// 2. stdlib/minimal_prelude.kleis (structures and operations)
    /// 3. stdlib/matrices.kleis (matrix operations)
    ///
    /// **Note:** Currently loads `minimal_prelude.kleis` because the full `prelude.kleis`
    /// uses advanced syntax (operator symbols, axioms with ∀) that the parser doesn't
    /// support yet. Once the parser is extended (Phase 2), this will load the full stdlib.
    pub fn with_stdlib() -> Result<Self, String> {
        let mut checker = Self::new();

        // PHASE 1: Load data type definitions (ADR-021)
        // This must happen FIRST so structures can reference these types
        // TODO: Uncomment when stdlib/types.kleis exists (Step 8)
        // let types_def = include_str!("../stdlib/types.kleis");
        // checker
        //     .load_data_types(types_def)
        //     .map_err(|e| format!("Failed to load stdlib/types.kleis: {}", e))?;

        // PHASE 2: Load structures and operations
        // Load minimal prelude (subset that parser can handle)
        // TODO: Switch to full prelude.kleis once parser supports:
        //   - Operator symbols in parens: (•), (⊗)
        //   - Axioms with universal quantifiers: ∀(x y z : S)
        //   - Nested structures
        let minimal_prelude = include_str!("../stdlib/minimal_prelude.kleis");
        checker
            .load_kleis(minimal_prelude)
            .map_err(|e| format!("Failed to load stdlib/minimal_prelude.kleis: {}", e))?;

        // Load matrices
        let matrices = include_str!("../stdlib/matrices.kleis");
        checker
            .load_kleis(matrices)
            .map_err(|e| format!("Failed to load stdlib/matrices.kleis: {}", e))?;

        Ok(checker)
    }

    /// Load data type definitions from Kleis code (ADR-021)
    ///
    /// Parses data type definitions and registers them in the inference engine's
    /// data registry. This must be called BEFORE loading structures that reference
    /// these types.
    ///
    /// Example:
    /// ```ignore
    /// let types = "data Bool = True | False";
    /// checker.load_data_types(types)?;
    /// // Now Bool constructors are available
    /// ```
    pub fn load_data_types(&mut self, code: &str) -> Result<(), String> {
        use crate::kleis_parser::parse_kleis_program;

        // Parse the Kleis code
        let program = parse_kleis_program(code).map_err(|e| format!("Parse error: {}", e))?;

        // Extract and register all data definitions
        for item in program.items {
            if let crate::kleis_ast::TopLevel::DataDef(data_def) = item {
                self.inference
                    .data_registry_mut()
                    .register(data_def)
                    .map_err(|e| format!("Failed to register data type: {}", e))?;
            }
        }

        Ok(())
    }

    /// Load Kleis code into the type checker context
    /// This can be used to load additional libraries or user-defined types.
    pub fn load_kleis(&mut self, code: &str) -> Result<(), String> {
        use crate::kleis_parser::parse_kleis_program;

        // Parse the Kleis code
        let program = parse_kleis_program(code).map_err(|e| format!("Parse error: {}", e))?;

        // Build context from program
        let new_context = TypeContextBuilder::from_program(program)?;

        // Merge into existing context
        self.context_builder.merge(new_context)?;

        Ok(())
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
                    "ℝ" | "Real" => Type::scalar(),
                    // TODO: Add more mappings
                    _ => Type::scalar(), // Default for now
                }
            }
            TypeExpr::Parametric(name, _params) => {
                // TODO: Handle parametric types
                match name.as_str() {
                    "Vector" => Type::vector(3), // Default dimension
                    "Matrix" => Type::matrix(3, 3),
                    _ => Type::scalar(),
                }
            }
            _ => Type::scalar(), // Default
        }
    }

    /// Type check an expression with helpful error messages
    pub fn check(&mut self, expr: &Expression) -> TypeCheckResult {
        // Try HM inference with context_builder (ADR-016 compliant!)
        match self
            .inference
            .infer_and_solve(expr, Some(&self.context_builder))
        {
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
            if let Some(structure) = self
                .context_builder
                .registry()
                .structure_for_operation(name)
            {
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
        self.context_builder
            .supports_operation(type_name, operation_name)
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
    use crate::kleis_parser::{parse_kleis, parse_kleis_program};

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

    // ===== Data Type Loading Tests (ADR-021) =====

    #[test]
    fn test_load_data_types_simple() {
        let mut checker = TypeChecker::new();

        let code = "data Bool = True | False";
        checker.load_data_types(code).unwrap();

        // Verify data types were loaded
        let registry = checker.inference.data_registry();
        assert!(registry.has_type("Bool"));
        assert!(registry.has_variant("True"));
        assert!(registry.has_variant("False"));
    }

    #[test]
    fn test_load_data_types_parametric() {
        let mut checker = TypeChecker::new();

        let code = "data Option(T) = None | Some(T)";
        checker.load_data_types(code).unwrap();

        let registry = checker.inference.data_registry();
        assert!(registry.has_type("Option"));
        assert!(registry.has_variant("None"));
        assert!(registry.has_variant("Some"));
    }

    #[test]
    fn test_load_multiple_data_types() {
        let mut checker = TypeChecker::new();

        let code = r#"
            data Bool = True | False
            data Option(T) = None | Some(T)
            data Type = Scalar | Vector(n: Nat)
        "#;
        checker.load_data_types(code).unwrap();

        let registry = checker.inference.data_registry();
        assert_eq!(registry.type_count(), 3);
        assert_eq!(registry.variant_count(), 6); // True, False, None, Some, Scalar, Vector
    }

    #[test]
    fn test_load_data_types_with_named_fields() {
        let mut checker = TypeChecker::new();

        let code = "data Type = Scalar | Matrix(m: Nat, n: Nat)";
        checker.load_data_types(code).unwrap();

        let registry = checker.inference.data_registry();
        assert!(registry.has_variant("Scalar"));
        assert!(registry.has_variant("Matrix"));

        // Check Matrix variant has correct structure
        let (type_name, variant) = registry.lookup_variant("Matrix").unwrap();
        assert_eq!(type_name, "Type");
        assert_eq!(variant.fields.len(), 2);
    }

    #[test]
    fn test_load_data_types_error_duplicate() {
        let mut checker = TypeChecker::new();

        let code = r#"
            data Bool = True | False
            data Bool = Yes | No
        "#;

        let result = checker.load_data_types(code);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already registered"));
    }

    #[test]
    fn test_with_stdlib_still_works() {
        // Ensure with_stdlib() still works after our changes
        let result = TypeChecker::with_stdlib();
        assert!(result.is_ok());

        let checker = result.unwrap();
        // Should have loaded minimal_prelude and matrices
        assert!(checker.type_supports_operation("ℝ", "plus"));
    }
}
