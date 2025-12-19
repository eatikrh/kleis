// ============================================================================
// KLEIS ABI - Application Binary Interface for Kleis Modules
// ============================================================================
//
// Each compiled Kleis module is a shared library (.dylib/.so/.dll) that
// exports C-compatible functions for discovery and invocation.
//
// The REPL loads modules directly via dlopen/LoadLibrary - NO central registry.
//
// ┌─────────────┐     dlopen/LoadLibrary     ┌──────────────────┐
// │   REPL      │ ──────────────────────────▶│  my_module.dylib │
// │             │                             │                  │
// │  "load X"   │◀──── kleis_manifest() ─────│  [exported fns]  │
// │             │                             │                  │
// │  "fib(5)"   │──── kleis_call("fib",[5])──▶│                  │
// │             │◀──── 5 ────────────────────│                  │
// └─────────────┘                             └──────────────────┘
//
// CALLING CONVENTIONS:
//
//   All functions use `extern "C"` for cross-language compatibility.
//
// VALUE PASSING:
//
//   Kleis Type    Rust Type        Convention
//   ------------- ---------------- --------------------------------
//   ℕ (Nat)       u64              By value (8 bytes)
//   ℤ (Int)       i64              By value (8 bytes)
//   ℝ (Real)      f64              By value (8 bytes)
//   Bool          bool             By value (1 byte)
//   String        *const c_char    By pointer (null-terminated UTF-8)
//   List(T)       *const AbiValue  By pointer + length
//   Option(T)     AbiValue         By value (tagged union)
//   Constructor   AbiValue         By value (tag + pointer)
//
// OWNERSHIP RULES:
//
//   1. CALLER OWNS INPUT: Allocates and frees input memory
//   2. CALLEE OWNS OUTPUT: Static strings borrowed from module
//   3. ERROR STRINGS: Static lifetime, do not free
//
// THREAD SAFETY:
//
//   - NOT thread-safe by default
//   - Static data (names, axioms) safe for concurrent reads
//
// ============================================================================

use std::ffi::{c_char, CStr, CString};
use std::ptr;

// ============================================================================
// 1. ABI VALUE TYPE (C-compatible)
// ============================================================================

/// C-compatible value for cross-boundary calls
#[repr(C)]
#[derive(Debug, Clone)]
pub enum AbiValue {
    Null,
    Nat(u64),
    Int(i64),
    Real(f64),
    Bool(bool),
    // Strings are passed as pointers
    String { ptr: *const c_char, len: usize },
    // Lists are passed as arrays
    List { ptr: *const AbiValue, len: usize },
    // Constructor with tag
    Constructor { tag: u32, args_ptr: *const AbiValue, args_len: usize },
    // Function reference (for HOF support)
    Function { 
        module_ptr: *const c_char,   // Module name
        name_ptr: *const c_char,     // Function name
        arity: usize,                // Number of arguments
    },
    // Partial application (function with some args bound)
    PartialApp {
        module_ptr: *const c_char,
        name_ptr: *const c_char,
        arity: usize,
        bound_args_ptr: *const AbiValue,
        bound_args_len: usize,
    },
}

impl Default for AbiValue {
    fn default() -> Self {
        AbiValue::Null
    }
}

// ============================================================================
// 2. ABI RESULT TYPE
// ============================================================================

#[repr(C)]
pub struct AbiResult {
    pub success: bool,
    pub value: AbiValue,
    pub error_ptr: *const c_char,  // null if success
}

impl AbiResult {
    pub fn ok(value: AbiValue) -> Self {
        AbiResult {
            success: true,
            value,
            error_ptr: ptr::null(),
        }
    }

    pub fn err(msg: &str) -> Self {
        let c_str = CString::new(msg).unwrap_or_default();
        AbiResult {
            success: false,
            value: AbiValue::Null,
            error_ptr: c_str.into_raw(),
        }
    }
}

// ============================================================================
// 3. MANIFEST STRUCTURES (C-compatible)
// ============================================================================

#[repr(C)]
pub struct AbiParamSig {
    pub name: *const c_char,
    pub type_name: *const c_char,
}

#[repr(C)]
pub struct AbiFunctionSig {
    pub name: *const c_char,
    pub params: *const AbiParamSig,
    pub params_len: usize,
    pub return_type: *const c_char,
    pub doc: *const c_char,  // null if none
}

#[repr(C)]
pub struct AbiAxiom {
    pub name: *const c_char,
    pub formula: *const c_char,
    pub doc: *const c_char,  // null if none
}

#[repr(C)]
pub struct AbiManifest {
    pub name: *const c_char,
    pub version: *const c_char,
    pub functions: *const AbiFunctionSig,
    pub functions_len: usize,
    pub axioms: *const AbiAxiom,
    pub axioms_len: usize,
}

// ============================================================================
// 4. MODULE TRAIT (what generated code implements)
// ============================================================================

/// Trait that generated modules implement internally
pub trait KleisModuleImpl: Send + Sync {
    /// Get module name
    fn name(&self) -> &'static str;
    
    /// Get module version string (e.g., "1.0.0")
    fn version(&self) -> &'static str;
    
    /// Get version info for compatibility checking
    fn version_info(&self) -> ModuleVersion {
        ModuleVersion {
            source_hash: "",
            signature_hash: "",
            compile_time: 0,
        }
    }
    
    /// Get function signatures
    fn functions(&self) -> &'static [StaticFunctionSig];
    
    /// Get axioms
    fn axioms(&self) -> &'static [StaticAxiom];
    
    /// Get imports (dependencies with version tracking)
    fn imports(&self) -> &'static [StaticImport] { &[] }
    
    /// Call a function by name
    fn call(&self, name: &str, args: &[AbiValue]) -> AbiResult;
    
    /// Get axiom formula by name
    fn get_axiom(&self, name: &str) -> Option<&'static str>;
    
    /// Check if a dependency is compatible
    fn is_import_compatible(&self, import_path: &str, current_sig_hash: &str) -> bool {
        self.imports()
            .iter()
            .find(|i| i.path == import_path)
            .map(|i| i.expected_hash == current_sig_hash)
            .unwrap_or(true) // Not a dependency, so compatible
    }
}

/// Static function signature (compile-time)
pub struct StaticFunctionSig {
    pub name: &'static str,
    pub params: &'static [(&'static str, &'static str)],  // (name, type)
    pub return_type: &'static str,
    pub doc: Option<&'static str>,
}

/// Static axiom (compile-time)
pub struct StaticAxiom {
    pub name: &'static str,
    pub formula: &'static str,
    pub doc: Option<&'static str>,
}

/// Import dependency with version tracking
pub struct StaticImport {
    pub path: &'static str,           // Module path (e.g., "stdlib/prelude.kleis")
    pub expected_hash: &'static str,  // Signature hash at compile time
    pub required: bool,               // Must be loaded first
}

/// Module version info
pub struct ModuleVersion {
    pub source_hash: &'static str,    // SHA256 of .kleis source
    pub signature_hash: &'static str, // Hash of exported signatures
    pub compile_time: u64,            // Unix timestamp
}

// ============================================================================
// 5. MACRO FOR GENERATING EXPORTED FUNCTIONS
// ============================================================================

/// Macro to generate the C-compatible exports for a module
#[macro_export]
macro_rules! kleis_module_exports {
    ($module_impl:expr) => {
        use std::ffi::{c_char, CStr};
        use $crate::module_abi::*;
        
        static MODULE: std::sync::LazyLock<Box<dyn KleisModuleImpl>> = 
            std::sync::LazyLock::new(|| Box::new($module_impl));
        
        /// Get the module's manifest (for discovery)
        #[no_mangle]
        pub extern "C" fn kleis_manifest() -> *const AbiManifest {
            // Build manifest from static data
            static MANIFEST: std::sync::LazyLock<AbiManifest> = 
                std::sync::LazyLock::new(|| {
                    // This would be populated from MODULE
                    AbiManifest {
                        name: std::ptr::null(),
                        version: std::ptr::null(),
                        functions: std::ptr::null(),
                        functions_len: 0,
                        axioms: std::ptr::null(),
                        axioms_len: 0,
                    }
                });
            &*MANIFEST
        }
        
        /// Call a function by name
        #[no_mangle]
        pub extern "C" fn kleis_call(
            name: *const c_char,
            args: *const AbiValue,
            args_len: usize,
        ) -> AbiResult {
            let name = unsafe {
                if name.is_null() {
                    return AbiResult::err("null function name");
                }
                match CStr::from_ptr(name).to_str() {
                    Ok(s) => s,
                    Err(_) => return AbiResult::err("invalid UTF-8 in function name"),
                }
            };
            
            let args = if args.is_null() || args_len == 0 {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(args, args_len) }
            };
            
            MODULE.call(name, args)
        }
        
        /// Get an axiom's formula
        #[no_mangle]
        pub extern "C" fn kleis_get_axiom(name: *const c_char) -> *const c_char {
            let name = unsafe {
                if name.is_null() {
                    return std::ptr::null();
                }
                match CStr::from_ptr(name).to_str() {
                    Ok(s) => s,
                    Err(_) => return std::ptr::null(),
                }
            };
            
            match MODULE.get_axiom(name) {
                Some(formula) => formula.as_ptr() as *const c_char,
                None => std::ptr::null(),
            }
        }
        
        /// Get all function names (null-terminated array)
        #[no_mangle]
        pub extern "C" fn kleis_function_names() -> *const *const c_char {
            static NAMES: std::sync::LazyLock<Vec<*const c_char>> = 
                std::sync::LazyLock::new(|| {
                    let mut names: Vec<*const c_char> = MODULE.functions()
                        .iter()
                        .map(|f| f.name.as_ptr() as *const c_char)
                        .collect();
                    names.push(std::ptr::null());  // null terminator
                    names
                });
            NAMES.as_ptr()
        }
        
        /// Get all axiom names (null-terminated array)
        #[no_mangle]
        pub extern "C" fn kleis_axiom_names() -> *const *const c_char {
            static NAMES: std::sync::LazyLock<Vec<*const c_char>> = 
                std::sync::LazyLock::new(|| {
                    let mut names: Vec<*const c_char> = MODULE.axioms()
                        .iter()
                        .map(|a| a.name.as_ptr() as *const c_char)
                        .collect();
                    names.push(std::ptr::null());  // null terminator
                    names
                });
            NAMES.as_ptr()
        }
    };
}

// ============================================================================
// 6. EXAMPLE: WHAT GENERATED CODE LOOKS LIKE
// ============================================================================

/// Example of what codegen produces for a simple module
pub mod example_generated {
    use super::*;

    pub struct FibonacciModule;

    impl KleisModuleImpl for FibonacciModule {
        fn name(&self) -> &'static str { "fibonacci" }
        fn version(&self) -> &'static str { "1.0.0" }
        
        fn functions(&self) -> &'static [StaticFunctionSig] {
            static FUNCS: &[StaticFunctionSig] = &[
                StaticFunctionSig {
                    name: "fib",
                    params: &[("n", "ℕ")],
                    return_type: "ℕ",
                    doc: Some("Compute nth Fibonacci number"),
                },
                StaticFunctionSig {
                    name: "factorial",
                    params: &[("n", "ℕ")],
                    return_type: "ℕ",
                    doc: Some("Compute n factorial"),
                },
            ];
            FUNCS
        }
        
        fn axioms(&self) -> &'static [StaticAxiom] {
            static AXIOMS: &[StaticAxiom] = &[
                StaticAxiom {
                    name: "fib_base_0",
                    formula: "fib(0) = 0",
                    doc: None,
                },
                StaticAxiom {
                    name: "fib_base_1",
                    formula: "fib(1) = 1",
                    doc: None,
                },
                StaticAxiom {
                    name: "fib_recursive",
                    formula: "∀ n : ℕ . n ≥ 2 → fib(n) = fib(n-1) + fib(n-2)",
                    doc: Some("Fibonacci recurrence relation"),
                },
            ];
            AXIOMS
        }
        
        fn call(&self, name: &str, args: &[AbiValue]) -> AbiResult {
            match name {
                "fib" => {
                    let n = match args.first() {
                        Some(AbiValue::Nat(n)) => *n,
                        _ => return AbiResult::err("fib expects Nat argument"),
                    };
                    AbiResult::ok(AbiValue::Nat(fib(n)))
                }
                "factorial" => {
                    let n = match args.first() {
                        Some(AbiValue::Nat(n)) => *n,
                        _ => return AbiResult::err("factorial expects Nat argument"),
                    };
                    AbiResult::ok(AbiValue::Nat(factorial(n)))
                }
                _ => AbiResult::err(&format!("unknown function: {}", name)),
            }
        }
        
        fn get_axiom(&self, name: &str) -> Option<&'static str> {
            match name {
                "fib_base_0" => Some("fib(0) = 0"),
                "fib_base_1" => Some("fib(1) = 1"),
                "fib_recursive" => Some("∀ n : ℕ . n ≥ 2 → fib(n) = fib(n-1) + fib(n-2)"),
                _ => None,
            }
        }
    }

    // The actual functions
    fn fib(n: u64) -> u64 {
        if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
    }

    fn factorial(n: u64) -> u64 {
        if n == 0 { 1 } else { n * factorial(n - 1) }
    }
}

// ============================================================================
// 7. TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::example_generated::FibonacciModule;

    #[test]
    fn test_module_discovery() {
        let module = FibonacciModule;
        
        assert_eq!(module.name(), "fibonacci");
        assert_eq!(module.functions().len(), 2);
        assert_eq!(module.axioms().len(), 3);
    }

    #[test]
    fn test_module_call() {
        let module = FibonacciModule;
        
        let result = module.call("fib", &[AbiValue::Nat(10)]);
        assert!(result.success);
        match result.value {
            AbiValue::Nat(n) => assert_eq!(n, 55),
            _ => panic!("expected Nat"),
        }
    }

    #[test]
    fn test_axiom_lookup() {
        let module = FibonacciModule;
        
        assert_eq!(module.get_axiom("fib_base_0"), Some("fib(0) = 0"));
        assert_eq!(module.get_axiom("nonexistent"), None);
    }
}

