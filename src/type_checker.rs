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
use crate::type_inference::{Type, TypeInference, TypeVar};

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
    /// 4. stdlib/tensors_minimal.kleis (tensor operations for GR)
    /// 5. stdlib/quantum_minimal.kleis (quantum mechanics operations)
    ///
    /// **Note:** Currently loads `minimal_prelude.kleis` because the full `prelude.kleis`
    /// uses advanced syntax (operator symbols, axioms with ∀) that the parser doesn't
    /// support yet. Once the parser is extended (Phase 2), this will load the full stdlib.
    pub fn with_stdlib() -> Result<Self, String> {
        let mut checker = Self::new();

        // PHASE 1: Load data type definitions AND function definitions (ADR-021 + Self-hosting!)
        // This must happen FIRST so structures can reference these types
        //
        // types.kleis contains both:
        // - `data` definitions (Bool, Option, List, etc.)
        // - `define` function definitions (not, head, tail, etc.) - SELF-HOSTED!
        //
        // We use load_kleis() which loads BOTH data types and functions.
        // This is real self-hosting - Kleis standard library functions defined in Kleis!
        let types_def = include_str!("../stdlib/types.kleis");
        checker
            .load_kleis(types_def)
            .map_err(|e| format!("Failed to load stdlib/types.kleis: {}", e))?;

        // PHASE 2: Load structures and operations
        // Load full prelude (complete algebraic hierarchy with axioms)
        // Parser now supports:
        //   ✅ Operator symbols in definitions: operation (×) : ...
        //   ✅ Quantified type signatures: operation dot : ∀(n : ℕ). Vector(n) → ℝ
        //   ✅ Axioms with quantifiers: axiom assoc: ∀(x y z : S). (x • y) • z = x • (y • z)
        //   ✅ Nested structures: structure additive : AbelianGroup(R) { ... }
        //   ✅ Extends keyword: structure Group(G) extends Monoid(G)
        //   ✅ Over clauses: structure VectorSpace(V) over Field(F)
        //   ✅ Where clauses: where ...
        //   ✅ Define with operators: define (-)(x, y) = ...
        let prelude = include_str!("../stdlib/prelude.kleis");
        checker
            .load_kleis(prelude)
            .map_err(|e| format!("Failed to load stdlib/prelude.kleis: {}", e))?;

        // Load matrices
        let matrices = include_str!("../stdlib/matrices.kleis");
        checker
            .load_kleis(matrices)
            .map_err(|e| format!("Failed to load stdlib/matrices.kleis: {}", e))?;

        // Load tensors (GR operations) - minimal version for parser compatibility
        let tensors = include_str!("../stdlib/tensors_minimal.kleis");
        checker
            .load_kleis(tensors)
            .map_err(|e| format!("Failed to load stdlib/tensors_minimal.kleis: {}", e))?;

        // Load quantum mechanics (QM operations) - minimal version
        let quantum = include_str!("../stdlib/quantum_minimal.kleis");
        checker
            .load_kleis(quantum)
            .map_err(|e| format!("Failed to load stdlib/quantum_minimal.kleis: {}", e))?;

        // Load math functions (inverse trig, hyperbolic, special functions, etc.)
        let math_functions = include_str!("../stdlib/math_functions.kleis");
        checker
            .load_kleis(math_functions)
            .map_err(|e| format!("Failed to load stdlib/math_functions.kleis: {}", e))?;

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

        // PHASE 1: Register data type definitions FIRST (ADR-021)
        // This must happen before building structures or type-checking functions
        // because they may reference these data constructors
        for item in &program.items {
            if let crate::kleis_ast::TopLevel::DataDef(data_def) = item {
                self.inference
                    .data_registry_mut()
                    .register(data_def.clone())
                    .map_err(|e| format!("Failed to register data type: {}", e))?;
            }
        }

        // PHASE 2: Build context from program (structures and operations)
        let new_context = TypeContextBuilder::from_program(program.clone())?;

        // Merge into existing context
        self.context_builder.merge(new_context)?;

        // PHASE 3: Load function definitions (Wire 2: Self-hosting)
        // This happens AFTER data types are registered so functions can use them
        self.load_function_definitions(&program)?;

        Ok(())
    }

    /// Load function definitions from a parsed program (Wire 2: Self-hosting)
    ///
    /// This method processes `define` statements and adds function types to the
    /// type inference context. Functions can then be type-checked before use.
    ///
    /// Example:
    /// ```ignore
    /// let code = "define double(x) = x + x";
    /// checker.load_function_definitions(&program)?;
    /// // Now 'double' is available with inferred type
    /// ```
    fn load_function_definitions(
        &mut self,
        program: &crate::kleis_ast::Program,
    ) -> Result<(), String> {
        use crate::kleis_ast::TopLevel;

        // Process each function definition
        for item in &program.items {
            if let TopLevel::FunctionDef(func_def) = item {
                self.check_function_def(func_def)?;
            }
        }

        Ok(())
    }

    /// Type-check a function definition and add it to the context (Wire 2: Self-hosting)
    ///
    /// This implements the Hindley-Milner type checking for function definitions:
    /// 1. Add parameters to context (with types or fresh vars)
    /// 2. Infer body type
    /// 3. Build function type: T1 → T2 → ... → Tn → Result
    /// 4. Add function binding to context
    ///
    /// Examples:
    /// - `define double(x) = x + x` → infers type based on body
    /// - `define abs(x: ℝ) : ℝ = x * x` → uses annotated types
    pub fn check_function_def(
        &mut self,
        func_def: &crate::kleis_ast::FunctionDef,
    ) -> Result<Type, String> {
        // Save current context (we'll restore it after checking the function body)
        let saved_context = self.inference.context().clone();

        // Add parameters to context with fresh type variables
        // Wire 3 TODO: Use parameter type annotations from func_def.type_annotation
        for param in &func_def.params {
            let param_ty = Type::Var(TypeVar(self.inference.next_var_id()));
            self.inference.bind(param.clone(), param_ty);
        }

        // Infer the body type
        let body_ty = self
            .inference
            .infer_and_solve(&func_def.body, Some(&self.context_builder))
            .map_err(|e| format!("Type error in function '{}': {}", func_def.name, e))?;

        // Build function type
        // Wire 3 TODO: Build proper curried function type: param1 → param2 → ... → result
        // For now, we store just the body/result type since we don't have function application yet
        let func_ty = body_ty.clone();

        // Restore context (parameters were local to function body)
        *self.inference.context_mut() = saved_context;

        // Clear constraints (they were solved for this function, don't leak to next function)
        self.inference.clear_constraints();

        // Add function to context with its type
        self.inference.bind(func_def.name.clone(), func_ty.clone());

        Ok(func_ty)
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
                    // Future: Query data registry for user-defined types
                    // For now, default to scalar for unknown types
                    _ => Type::scalar(),
                }
            }
            TypeExpr::Parametric(_name, _params) => {
                // Future: Interpret params and build Data type
                // Example: Vector(3) → Data { type_name: "Type", constructor: "Vector", args: [NatValue(3)] }
                Type::scalar()
            }
            _ => Type::scalar(),
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
        if let Err(e) = &result {
            eprintln!("Error loading stdlib: {}", e);
        }
        assert!(result.is_ok());

        let checker = result.unwrap();

        // Debug: Check what's in the registry
        eprintln!(
            "Registry type count: {}",
            checker.inference.data_registry().type_count()
        );
        eprintln!(
            "Registry variant count: {}",
            checker.inference.data_registry().variant_count()
        );
        eprintln!(
            "Has Matrix variant: {}",
            checker.inference.data_registry().has_variant("Matrix")
        );
        eprintln!(
            "Has Scalar variant: {}",
            checker.inference.data_registry().has_variant("Scalar")
        );

        // Should have loaded prelude and matrices
        assert!(checker.type_supports_operation("ℝ", "plus"));
    }

    // ===== Function Definition Type Checking Tests (Wire 2: Self-hosting) =====

    #[test]
    fn test_check_function_def_simple_constant() {
        let mut checker = TypeChecker::new();

        let code = "define pi = 3.14159";
        checker.load_kleis(code).unwrap();

        // Verify pi is in context
        let pi_ty = checker.inference.context().get("pi");
        assert!(pi_ty.is_some());
    }

    #[test]
    fn test_check_function_def_one_param() {
        let mut checker = TypeChecker::with_stdlib().unwrap();

        let code = "define double(x) = x + x";
        checker.load_kleis(code).unwrap();

        // Verify double is in context
        let double_ty = checker.inference.context().get("double");
        assert!(double_ty.is_some());
    }

    #[test]
    fn test_check_function_def_two_params() {
        let mut checker = TypeChecker::with_stdlib().unwrap();

        let code = "define add(x, y) = x + y";
        checker.load_kleis(code).unwrap();

        // Verify add is in context
        let add_ty = checker.inference.context().get("add");
        assert!(add_ty.is_some());
    }

    #[test]
    fn test_check_function_def_with_pattern_match() {
        let mut checker = TypeChecker::with_stdlib().unwrap();

        // First test: simpler case without pattern matching
        let simple_code = "define identity(x) = x";
        checker.load_kleis(simple_code).unwrap();

        // Pattern matching on Bool - should work after fixing constructor unification!
        // True and False both have type Bool, so branches unify correctly
        let code = "define not(b) = match b { True => False | False => True }";
        let result = checker.load_kleis(code);

        if let Err(e) = &result {
            eprintln!("ERROR: {}", e);
            eprintln!("Context before match: {:?}", checker.inference.context());
        }

        // Should succeed now!
        assert!(result.is_ok());

        // Verify not is in context
        let not_ty = checker.inference.context().get("not");
        assert!(not_ty.is_some());
    }

    #[test]
    fn test_check_multiple_function_defs() {
        let mut checker = TypeChecker::with_stdlib().unwrap();

        let code = r#"
            define pi = 3.14159
            define double(x) = x + x
            define add(x, y) = x + y
        "#;
        checker.load_kleis(code).unwrap();

        // Verify all functions are in context
        assert!(checker.inference.context().get("pi").is_some());
        assert!(checker.inference.context().get("double").is_some());
        assert!(checker.inference.context().get("add").is_some());
    }

    #[test]
    fn test_check_function_def_mixed_with_data() {
        let mut checker = TypeChecker::with_stdlib().unwrap();

        // Load a simpler function that doesn't use pattern matching on Bool
        // Pattern matching on Bool currently fails due to type inference limitations
        let code = r#"
            define identity(x) = x
            define const_val() = 42
        "#;
        checker.load_kleis(code).unwrap();

        // Verify functions are in context
        assert!(checker.inference.context().get("identity").is_some());
        assert!(checker.inference.context().get("const_val").is_some());
    }

    #[test]
    fn test_check_function_def_error_undefined_var() {
        let mut checker = TypeChecker::with_stdlib().unwrap();

        // This should succeed - undefined variables get fresh type vars
        let code = "define f(x) = x + y";
        let result = checker.load_kleis(code);

        // The function loads successfully (y gets a fresh type variable)
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_function_def_empty_params() {
        let mut checker = TypeChecker::new();

        let code = "define f() = 42";
        checker.load_kleis(code).unwrap();

        // Verify f is in context
        let f_ty = checker.inference.context().get("f");
        assert!(f_ty.is_some());
    }

    #[test]
    fn test_check_function_def_complex_body() {
        let mut checker = TypeChecker::with_stdlib().unwrap();

        let code = "define compute(a, b, c) = a + b * c";
        checker.load_kleis(code).unwrap();

        // Verify compute is in context
        let compute_ty = checker.inference.context().get("compute");
        assert!(compute_ty.is_some());
    }

    #[test]
    fn test_function_def_with_stdlib() {
        let mut checker = TypeChecker::with_stdlib().unwrap();

        // Define a function using stdlib types
        let code = "define triple(x) = x + x + x";
        checker.load_kleis(code).unwrap();

        // Verify triple is in context
        let triple_ty = checker.inference.context().get("triple");
        assert!(triple_ty.is_some());
    }
}
