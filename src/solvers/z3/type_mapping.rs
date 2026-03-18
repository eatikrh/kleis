//! Z3 Type Mapping - Consolidates all Type → Z3 Sort mappings
//!
//! This module is the SINGLE PLACE where Kleis types map to Z3 sorts.
//! All Z3-specific type logic is consolidated here.
//!
//! ## Design Principles
//!
//! 1. **Single source of truth** - All Type → Z3 Sort mappings in one file
//! 2. **Extensible** - Easy to add new type mappings
//! 3. **Data-driven** - No hardcoding of specific types like Matrix, Tensor
//! 4. **Clear separation** - EditorTypeTranslator is Z3-agnostic; this is Z3-specific
//!
//! ## Key Mappings
//!
//! | Kleis Type | Z3 Sort |
//! |------------|---------|
//! | Nat, NatValue(_) | Int |
//! | Bool | Bool |
//! | String | String |
//! | Real, Scalar | Real |
//! | Complex | ComplexDatatype (ADT) |
//! | User-defined | Uninterpreted sort |
//!
//! ## Usage
//!
//! The module provides free functions for use by Z3Backend:
//! - `get_type_dispatch_info(op_name, ty)` - Determines if an operation needs type-based dispatch
//! - `get_builtin_sort_kind(type_name)` - Checks if type maps to Z3 built-in
//! - `get_parameterized_sort_name(type_name, args)` - Builds unique sort name for user types

use crate::type_inference::Type;

// ============================================================================
// Type Dispatch Information
// ============================================================================

/// Type dispatch information for operations
#[derive(Debug, Clone)]
pub struct TypedOperationInfo {
    /// The Z3 function name to use (e.g., "Matrix_add" instead of "plus")
    pub z3_func_name: String,
    /// The result sort name for this operation
    pub result_sort_name: String,
}

/// Get type dispatch information for an operation.
///
/// This is the SINGLE PLACE where we decide if an operation needs
/// type-based dispatch (e.g., Matrix_add instead of plus).
///
/// Returns Some(info) if the type requires dispatched operations,
/// None if the default Z3 built-in should be used.
///
/// ## Logic
///
/// - Built-in types (Real, Int, Bool, etc.) → use Z3's native operations
/// - User-defined types (Matrix, Tensor, custom, etc.) → use uninterpreted functions
pub fn get_type_dispatch_info(op_name: &str, ty: &Type) -> Option<TypedOperationInfo> {
    // Only dispatch operations that are type-polymorphic
    const DISPATCHABLE_OPS: &[&str] = &[
        "plus", "add", "minus", "subtract", "times", "multiply", "matmul", "div", "divide",
        "negate", "neg",
    ];

    if !DISPATCHABLE_OPS.contains(&op_name) {
        return None;
    }

    // Get the type name for dispatch
    let type_name = match ty {
        Type::Data { type_name, .. } => type_name.as_str(),
        _ => return None, // Primitive types use Z3 built-ins
    };

    // Check if this type uses Z3 built-ins (no dispatch needed)
    if is_builtin_type(type_name) {
        return None;
    }

    // All other types get dispatched operations
    let op_suffix = match op_name {
        "plus" | "add" => "add",
        "minus" | "subtract" => "sub",
        "times" | "multiply" | "matmul" => "mul",
        "div" | "divide" => "div",
        "negate" | "neg" => "neg",
        _ => return None,
    };

    let z3_func_name = format!("{}_{}", type_name, op_suffix);

    Some(TypedOperationInfo {
        z3_func_name,
        result_sort_name: type_name.to_string(),
    })
}

// ============================================================================
// Type to Sort Mapping
// ============================================================================

/// Types that map to Z3 built-in sorts.
///
/// Operations on these types use Z3's native operations, not uninterpreted functions.
///
/// ## Z3 Sort Mapping
///
/// | Kleis Type | Z3 Sort |
/// |------------|---------|
/// | Real, Scalar, ℝ | Real |
/// | Rational, ℚ | Real (approximated) |
/// | Int, Integer, ℤ | Int |
/// | Nat, Natural, ℕ | Int (non-negative by convention) |
/// | Bool, Boolean | Bool |
/// | String, Str | String (QF_SLIA theory) |
/// | Complex, ℂ | ADT (handled by ComplexDatatype in backend) |
/// | Set, Set(T) | Set (parameterized by element type) |
/// | BitVec, BV, BitVec8, etc. | BitVec (parameterized by width) |
/// | Array | Array (key → value mapping) |
const BUILTIN_TYPES: &[&str] = &[
    // Numeric types
    "Real", "Scalar", "ℝ", "Int", "Integer", "ℤ", "Nat", "Natural", "ℕ", "Rational", "ℚ",
    // Complex numbers (special handling in backend)
    "Complex", "ℂ", // Boolean
    "Bool", "Boolean", // String
    "String", "Str", // Set (element type as parameter)
    "Set", // Bitvector (width as parameter)
    "BitVec", "BV", "BitVec8", "BitVec16", "BitVec32", "BitVec64",
    // Array (key/value types as parameters)
    "Array",
];

/// Check if a type name is a Z3 built-in type.
pub fn is_builtin_type(type_name: &str) -> bool {
    BUILTIN_TYPES.contains(&type_name) || is_parameterized_builtin(type_name)
}

/// Check if a type name is a parameterized built-in (e.g., BitVec(8), Set(Int))
fn is_parameterized_builtin(type_name: &str) -> bool {
    type_name.starts_with("BitVec")
        || type_name.starts_with("BV")
        || type_name.starts_with("Set")
        || type_name.starts_with("Array")
}

/// Z3 sort information for built-in types
#[derive(Debug, Clone, PartialEq)]
pub enum Z3SortKind {
    /// Z3 Real sort
    Real,
    /// Z3 Int sort
    Int,
    /// Z3 Bool sort
    Bool,
    /// Z3 String sort
    String,
    /// Z3 Complex ADT (handled specially in backend)
    Complex,
    /// Z3 Set sort with element type
    Set { element_sort: Box<Z3SortKind> },
    /// Z3 BitVec sort with width
    BitVec { width: u32 },
    /// Z3 Array sort with key and value types
    Array {
        key_sort: Box<Z3SortKind>,
        value_sort: Box<Z3SortKind>,
    },
    /// User-defined type (needs uninterpreted sort)
    UserDefined,
}

/// Get the Z3 sort kind for a type name.
///
/// Returns the Z3SortKind for the type, or UserDefined if it's not built-in.
pub fn get_sort_kind(type_name: &str, args: &[Type]) -> Z3SortKind {
    match type_name {
        // Numeric types
        "Real" | "Scalar" | "ℝ" | "Rational" | "ℚ" => Z3SortKind::Real,
        "Int" | "Integer" | "ℤ" | "Nat" | "Natural" | "ℕ" => Z3SortKind::Int,

        // Boolean
        "Bool" | "Boolean" => Z3SortKind::Bool,

        // String
        "String" | "Str" => Z3SortKind::String,

        // Complex numbers
        "Complex" | "ℂ" => Z3SortKind::Complex,

        // Set - extract element type from args
        "Set" => {
            let elem_sort = if !args.is_empty() {
                Box::new(get_sort_kind_for_type(&args[0]))
            } else {
                Box::new(Z3SortKind::Int) // Default to Set(Int)
            };
            Z3SortKind::Set {
                element_sort: elem_sort,
            }
        }

        // BitVec - extract width from name or args
        "BitVec" | "BV" => {
            let width = if !args.is_empty() {
                match &args[0] {
                    Type::NatValue(n) => *n as u32,
                    _ => 8, // Default to 8-bit
                }
            } else {
                8
            };
            Z3SortKind::BitVec { width }
        }
        "BitVec8" => Z3SortKind::BitVec { width: 8 },
        "BitVec16" => Z3SortKind::BitVec { width: 16 },
        "BitVec32" => Z3SortKind::BitVec { width: 32 },
        "BitVec64" => Z3SortKind::BitVec { width: 64 },

        // Array - extract key/value types from args
        "Array" => {
            let key_sort = if !args.is_empty() {
                Box::new(get_sort_kind_for_type(&args[0]))
            } else {
                Box::new(Z3SortKind::Int)
            };
            let value_sort = if args.len() > 1 {
                Box::new(get_sort_kind_for_type(&args[1]))
            } else {
                Box::new(Z3SortKind::Int)
            };
            Z3SortKind::Array {
                key_sort,
                value_sort,
            }
        }

        // User-defined type
        _ => Z3SortKind::UserDefined,
    }
}

/// Get Z3 sort kind for a Type (used for nested types like Set(Real))
fn get_sort_kind_for_type(ty: &Type) -> Z3SortKind {
    match ty {
        Type::Nat | Type::NatValue(_) | Type::NatExpr(_) => Z3SortKind::Int,
        Type::Bool => Z3SortKind::Bool,
        Type::String | Type::StringValue(_) => Z3SortKind::String,
        Type::Data {
            type_name, args, ..
        } => get_sort_kind(type_name, args),
        Type::App(_, _) => {
            if let Some((base, args)) = flatten_type_app(ty) {
                get_sort_kind(&base, &args)
            } else {
                Z3SortKind::UserDefined
            }
        }
        _ => Z3SortKind::Int, // Default fallback
    }
}

/// Get the Z3 built-in sort kind for a type name (simple string version).
///
/// Returns the sort kind as a string if it's a simple built-in type,
/// None if it needs more complex handling or is user-defined.
pub fn get_builtin_sort_kind(type_name: &str) -> Option<&'static str> {
    match type_name {
        "Real" | "Scalar" | "ℝ" | "Rational" | "ℚ" => Some("Real"),
        "Int" | "Integer" | "ℤ" | "Nat" | "Natural" | "ℕ" => Some("Int"),
        "Bool" | "Boolean" => Some("Bool"),
        "String" | "Str" => Some("String"),
        "Complex" | "ℂ" => Some("Complex"),
        "Set" => Some("Set"),
        "BitVec" | "BV" | "BitVec8" | "BitVec16" | "BitVec32" | "BitVec64" => Some("BitVec"),
        "Array" => Some("Array"),
        _ => None, // User-defined type
    }
}

/// Get the unique sort name for a parameterized type.
///
/// For user-defined types, this creates a name like "Matrix_2x3".
/// This name is used to create an uninterpreted Z3 sort.
pub fn get_parameterized_sort_name(type_name: &str, args: &[Type]) -> String {
    if args.is_empty() {
        type_name.to_string()
    } else {
        let param_strs: Vec<String> = args
            .iter()
            .map(|arg| match arg {
                Type::NatValue(n) => n.to_string(),
                Type::Data { type_name, .. } => type_name.clone(),
                Type::Nat => "n".to_string(),
                Type::Bool => "bool".to_string(),
                Type::App(_, _) => {
                    if let Some((base, args)) = flatten_type_app(arg) {
                        get_parameterized_sort_name(&base, &args)
                    } else {
                        "_".to_string()
                    }
                }
                _ => "_".to_string(),
            })
            .collect();
        format!("{}_{}", type_name, param_strs.join("x"))
    }
}

fn flatten_type_app(ty: &Type) -> Option<(String, Vec<Type>)> {
    match ty {
        Type::App(func, arg) => {
            let (base, mut args) = flatten_type_app(func)?;
            args.push((**arg).clone());
            Some((base, args))
        }
        Type::Data {
            constructor, args, ..
        } => Some((constructor.clone(), args.clone())),
        _ => None,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Built-in Type Recognition Tests
    // ========================================================================

    #[test]
    fn test_builtin_numeric_types() {
        // Real types
        assert!(is_builtin_type("Real"));
        assert!(is_builtin_type("Scalar"));
        assert!(is_builtin_type("ℝ"));
        assert!(is_builtin_type("Rational"));
        assert!(is_builtin_type("ℚ"));

        // Integer types
        assert!(is_builtin_type("Int"));
        assert!(is_builtin_type("Integer"));
        assert!(is_builtin_type("ℤ"));
        assert!(is_builtin_type("Nat"));
        assert!(is_builtin_type("Natural"));
        assert!(is_builtin_type("ℕ"));
    }

    #[test]
    fn test_builtin_complex_type() {
        assert!(is_builtin_type("Complex"));
        assert!(is_builtin_type("ℂ"));
    }

    #[test]
    fn test_builtin_boolean_and_string() {
        assert!(is_builtin_type("Bool"));
        assert!(is_builtin_type("Boolean"));
        assert!(is_builtin_type("String"));
        assert!(is_builtin_type("Str"));
    }

    #[test]
    fn test_builtin_set_types() {
        assert!(is_builtin_type("Set"));
        // Parameterized sets should also be recognized
        assert!(is_parameterized_builtin("Set(Int)"));
        assert!(is_parameterized_builtin("Set(Real)"));
    }

    #[test]
    fn test_builtin_bitvector_types() {
        assert!(is_builtin_type("BitVec"));
        assert!(is_builtin_type("BV"));
        assert!(is_builtin_type("BitVec8"));
        assert!(is_builtin_type("BitVec16"));
        assert!(is_builtin_type("BitVec32"));
        assert!(is_builtin_type("BitVec64"));
        // Parameterized bitvectors
        assert!(is_parameterized_builtin("BitVec(128)"));
        assert!(is_parameterized_builtin("BV(256)"));
    }

    #[test]
    fn test_builtin_array_type() {
        assert!(is_builtin_type("Array"));
        assert!(is_parameterized_builtin("Array(Int, Real)"));
    }

    #[test]
    fn test_user_defined_types_not_builtin() {
        assert!(!is_builtin_type("Matrix"));
        assert!(!is_builtin_type("Vector"));
        assert!(!is_builtin_type("Tensor"));
        assert!(!is_builtin_type("MyCustomType"));
        assert!(!is_builtin_type("Quaternion"));
    }

    // ========================================================================
    // Z3SortKind Tests
    // ========================================================================

    #[test]
    fn test_sort_kind_numeric() {
        assert_eq!(get_sort_kind("Real", &[]), Z3SortKind::Real);
        assert_eq!(get_sort_kind("ℝ", &[]), Z3SortKind::Real);
        assert_eq!(get_sort_kind("Rational", &[]), Z3SortKind::Real);
        assert_eq!(get_sort_kind("Int", &[]), Z3SortKind::Int);
        assert_eq!(get_sort_kind("ℤ", &[]), Z3SortKind::Int);
        assert_eq!(get_sort_kind("Nat", &[]), Z3SortKind::Int);
    }

    #[test]
    fn test_sort_kind_complex() {
        assert_eq!(get_sort_kind("Complex", &[]), Z3SortKind::Complex);
        assert_eq!(get_sort_kind("ℂ", &[]), Z3SortKind::Complex);
    }

    #[test]
    fn test_sort_kind_set() {
        // Default Set → Set(Int)
        let set_int = get_sort_kind("Set", &[]);
        assert!(
            matches!(set_int, Z3SortKind::Set { element_sort } if *element_sort == Z3SortKind::Int)
        );

        // Set(Real)
        let args = vec![Type::Data {
            type_name: "Real".to_string(),
            constructor: "Real".to_string(),
            args: vec![],
        }];
        let set_real = get_sort_kind("Set", &args);
        assert!(
            matches!(set_real, Z3SortKind::Set { element_sort } if *element_sort == Z3SortKind::Real)
        );
    }

    #[test]
    fn test_sort_kind_bitvector() {
        // Default BitVec → 8-bit
        let bv_default = get_sort_kind("BitVec", &[]);
        assert!(matches!(bv_default, Z3SortKind::BitVec { width: 8 }));

        // BitVec(32)
        let args = vec![Type::NatValue(32)];
        let bv32 = get_sort_kind("BitVec", &args);
        assert!(matches!(bv32, Z3SortKind::BitVec { width: 32 }));

        // Named bitvectors
        assert!(matches!(
            get_sort_kind("BitVec8", &[]),
            Z3SortKind::BitVec { width: 8 }
        ));
        assert!(matches!(
            get_sort_kind("BitVec16", &[]),
            Z3SortKind::BitVec { width: 16 }
        ));
        assert!(matches!(
            get_sort_kind("BitVec32", &[]),
            Z3SortKind::BitVec { width: 32 }
        ));
        assert!(matches!(
            get_sort_kind("BitVec64", &[]),
            Z3SortKind::BitVec { width: 64 }
        ));
    }

    #[test]
    fn test_sort_kind_array() {
        // Default Array → Array(Int, Int)
        let arr_default = get_sort_kind("Array", &[]);
        assert!(
            matches!(arr_default, Z3SortKind::Array { key_sort, value_sort }
            if *key_sort == Z3SortKind::Int && *value_sort == Z3SortKind::Int)
        );

        // Array(String, Real)
        let args = vec![
            Type::Data {
                type_name: "String".to_string(),
                constructor: "String".to_string(),
                args: vec![],
            },
            Type::Data {
                type_name: "Real".to_string(),
                constructor: "Real".to_string(),
                args: vec![],
            },
        ];
        let arr_str_real = get_sort_kind("Array", &args);
        assert!(
            matches!(arr_str_real, Z3SortKind::Array { key_sort, value_sort }
            if *key_sort == Z3SortKind::String && *value_sort == Z3SortKind::Real)
        );
    }

    #[test]
    fn test_sort_kind_user_defined() {
        assert_eq!(get_sort_kind("Matrix", &[]), Z3SortKind::UserDefined);
        assert_eq!(get_sort_kind("Tensor", &[]), Z3SortKind::UserDefined);
        assert_eq!(get_sort_kind("Quaternion", &[]), Z3SortKind::UserDefined);
    }

    // ========================================================================
    // Parameterized Sort Name Tests
    // ========================================================================

    #[test]
    fn test_parameterized_sort_name_simple() {
        assert_eq!(get_parameterized_sort_name("MyType", &[]), "MyType");
    }

    #[test]
    fn test_parameterized_sort_name_matrix() {
        let args = vec![Type::NatValue(2), Type::NatValue(3)];
        assert_eq!(get_parameterized_sort_name("Matrix", &args), "Matrix_2x3");

        // Different dimensions
        let args_4x4 = vec![Type::NatValue(4), Type::NatValue(4)];
        assert_eq!(
            get_parameterized_sort_name("Matrix", &args_4x4),
            "Matrix_4x4"
        );
    }

    #[test]
    fn test_parameterized_sort_name_with_type_args() {
        // Vector(Real)
        let args = vec![Type::Data {
            type_name: "Real".to_string(),
            constructor: "Real".to_string(),
            args: vec![],
        }];
        assert_eq!(get_parameterized_sort_name("Vector", &args), "Vector_Real");

        // Matrix(Complex)
        let args_complex = vec![Type::Data {
            type_name: "Complex".to_string(),
            constructor: "Complex".to_string(),
            args: vec![],
        }];
        assert_eq!(
            get_parameterized_sort_name("Matrix", &args_complex),
            "Matrix_Complex"
        );
    }

    #[test]
    fn test_parameterized_sort_name_complex_matrix() {
        // Matrix(3, 3, Complex) - 3x3 complex matrix
        let args = vec![
            Type::NatValue(3),
            Type::NatValue(3),
            Type::Data {
                type_name: "Complex".to_string(),
                constructor: "Complex".to_string(),
                args: vec![],
            },
        ];
        assert_eq!(
            get_parameterized_sort_name("Matrix", &args),
            "Matrix_3x3xComplex"
        );
    }

    #[test]
    fn test_parameterized_sort_name_with_type_app() {
        let matrix_ctor = Type::Data {
            type_name: "Matrix".to_string(),
            constructor: "Matrix".to_string(),
            args: vec![],
        };
        let matrix_app = Type::App(Box::new(matrix_ctor), Box::new(Type::NatValue(2)));
        assert_eq!(
            get_parameterized_sort_name("Wrapper", &[matrix_app]),
            "Wrapper_Matrix_2"
        );
    }

    // ========================================================================
    // Type Dispatch Tests
    // ========================================================================

    #[test]
    fn test_type_dispatch_builtin_no_dispatch() {
        // Built-in types should NOT get dispatched (use Z3 native)
        let types_to_test = vec![
            ("Real", vec![]),
            ("ℝ", vec![]),
            ("Int", vec![]),
            ("ℤ", vec![]),
            ("Nat", vec![]),
            ("Rational", vec![]),
            ("ℚ", vec![]),
            ("Complex", vec![]),
            ("ℂ", vec![]),
            ("Bool", vec![]),
            ("String", vec![]),
        ];

        for (type_name, args) in types_to_test {
            let ty = Type::Data {
                type_name: type_name.to_string(),
                constructor: type_name.to_string(),
                args,
            };
            assert!(
                get_type_dispatch_info("plus", &ty).is_none(),
                "Type {} should not be dispatched",
                type_name
            );
        }
    }

    #[test]
    fn test_type_dispatch_set_no_dispatch() {
        // Sets use Z3 built-in set operations
        let set_ty = Type::Data {
            type_name: "Set".to_string(),
            constructor: "Set".to_string(),
            args: vec![Type::Data {
                type_name: "Int".to_string(),
                constructor: "Int".to_string(),
                args: vec![],
            }],
        };
        assert!(get_type_dispatch_info("plus", &set_ty).is_none());
    }

    #[test]
    fn test_type_dispatch_bitvector_no_dispatch() {
        // Bitvectors use Z3 built-in BV operations
        let bv_ty = Type::Data {
            type_name: "BitVec".to_string(),
            constructor: "BitVec".to_string(),
            args: vec![Type::NatValue(32)],
        };
        assert!(get_type_dispatch_info("plus", &bv_ty).is_none());
    }

    #[test]
    fn test_type_dispatch_matrix() {
        let matrix_ty = Type::Data {
            type_name: "Matrix".to_string(),
            constructor: "Matrix".to_string(),
            args: vec![Type::NatValue(2), Type::NatValue(2)],
        };

        let dispatch_add = get_type_dispatch_info("plus", &matrix_ty);
        assert!(dispatch_add.is_some());
        assert_eq!(dispatch_add.unwrap().z3_func_name, "Matrix_add");

        let dispatch_mul = get_type_dispatch_info("times", &matrix_ty);
        assert!(dispatch_mul.is_some());
        assert_eq!(dispatch_mul.unwrap().z3_func_name, "Matrix_mul");

        let dispatch_sub = get_type_dispatch_info("minus", &matrix_ty);
        assert!(dispatch_sub.is_some());
        assert_eq!(dispatch_sub.unwrap().z3_func_name, "Matrix_sub");
    }

    #[test]
    fn test_type_dispatch_tensor() {
        let tensor_ty = Type::Data {
            type_name: "Tensor".to_string(),
            constructor: "Tensor".to_string(),
            args: vec![Type::NatValue(3), Type::NatValue(4)],
        };

        let dispatch = get_type_dispatch_info("plus", &tensor_ty);
        assert!(dispatch.is_some());
        assert_eq!(dispatch.unwrap().z3_func_name, "Tensor_add");
    }

    #[test]
    fn test_type_dispatch_custom_adt() {
        // User-defined algebraic data type
        let quaternion_ty = Type::Data {
            type_name: "Quaternion".to_string(),
            constructor: "Quaternion".to_string(),
            args: vec![],
        };

        let dispatch = get_type_dispatch_info("times", &quaternion_ty);
        assert!(dispatch.is_some());
        assert_eq!(dispatch.unwrap().z3_func_name, "Quaternion_mul");
    }

    #[test]
    fn test_type_dispatch_complex_matrix() {
        // Matrix over complex numbers
        let complex_matrix_ty = Type::Data {
            type_name: "ComplexMatrix".to_string(),
            constructor: "ComplexMatrix".to_string(),
            args: vec![Type::NatValue(3), Type::NatValue(3)],
        };

        let dispatch = get_type_dispatch_info("plus", &complex_matrix_ty);
        assert!(dispatch.is_some());
        assert_eq!(dispatch.unwrap().z3_func_name, "ComplexMatrix_add");
    }

    #[test]
    fn test_type_dispatch_different_sizes() {
        // Different matrix sizes get different dispatch names (sort names differ)
        let matrix_2x2 = Type::Data {
            type_name: "Matrix".to_string(),
            constructor: "Matrix".to_string(),
            args: vec![Type::NatValue(2), Type::NatValue(2)],
        };
        let matrix_3x4 = Type::Data {
            type_name: "Matrix".to_string(),
            constructor: "Matrix".to_string(),
            args: vec![Type::NatValue(3), Type::NatValue(4)],
        };

        // Both dispatch to Matrix_add, but sorts differ
        let dispatch_2x2 = get_type_dispatch_info("plus", &matrix_2x2).unwrap();
        let dispatch_3x4 = get_type_dispatch_info("plus", &matrix_3x4).unwrap();

        assert_eq!(dispatch_2x2.z3_func_name, "Matrix_add");
        assert_eq!(dispatch_3x4.z3_func_name, "Matrix_add");

        // Sort names differ
        assert_eq!(
            get_parameterized_sort_name("Matrix", &[Type::NatValue(2), Type::NatValue(2)]),
            "Matrix_2x2"
        );
        assert_eq!(
            get_parameterized_sort_name("Matrix", &[Type::NatValue(3), Type::NatValue(4)]),
            "Matrix_3x4"
        );
    }

    #[test]
    fn test_non_dispatchable_operations() {
        let matrix_ty = Type::Data {
            type_name: "Matrix".to_string(),
            constructor: "Matrix".to_string(),
            args: vec![],
        };

        // Non-arithmetic operations should never be dispatched
        assert!(get_type_dispatch_info("equals", &matrix_ty).is_none());
        assert!(get_type_dispatch_info("and", &matrix_ty).is_none());
        assert!(get_type_dispatch_info("or", &matrix_ty).is_none());
        assert!(get_type_dispatch_info("implies", &matrix_ty).is_none());
        assert!(get_type_dispatch_info("lt", &matrix_ty).is_none());
    }

    #[test]
    fn test_primitive_types_no_dispatch() {
        // Primitive types (not Data) use Z3 built-ins
        assert!(get_type_dispatch_info("plus", &Type::Nat).is_none());
        assert!(get_type_dispatch_info("times", &Type::Bool).is_none());
        assert!(get_type_dispatch_info("plus", &Type::String).is_none());
    }

    // ========================================================================
    // Edge Cases
    // ========================================================================

    #[test]
    fn test_empty_type_name() {
        assert!(!is_builtin_type(""));
        assert_eq!(get_sort_kind("", &[]), Z3SortKind::UserDefined);
    }

    #[test]
    fn test_nested_parameterized_types() {
        // Matrix(Vector(3), 4) - unusual but should handle gracefully
        let args = vec![
            Type::Data {
                type_name: "Vector".to_string(),
                constructor: "Vector".to_string(),
                args: vec![Type::NatValue(3)],
            },
            Type::NatValue(4),
        ];
        assert_eq!(
            get_parameterized_sort_name("Matrix", &args),
            "Matrix_Vectorx4"
        );
    }
}
