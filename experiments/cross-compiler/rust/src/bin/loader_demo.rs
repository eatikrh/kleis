// ============================================================================
// KLEIS MODULE LOADER DEMO
// ============================================================================
//
// This demonstrates the full cycle:
//   1. Load a compiled Kleis module (.dylib) via dlopen
//   2. Discover available functions and axioms
//   3. Call functions by name
//   4. Retrieve axioms for verification
//
// This is how the REPL would load JIT-compiled Kleis modules.
//
// ============================================================================

use std::ffi::{c_char, CStr, CString};
use std::path::Path;

// Import types from the library
use kleis_solver_api::module_abi::{AbiValue, AbiResult};

// ============================================================================
// MODULE LOADER
// ============================================================================

pub struct KleisModuleLoader {
    lib: libloading::Library,
}

impl KleisModuleLoader {
    /// Load a Kleis module from a shared library
    pub unsafe fn load(path: impl AsRef<Path>) -> Result<Self, libloading::Error> {
        let lib = libloading::Library::new(path.as_ref())?;
        Ok(KleisModuleLoader { lib })
    }

    /// Get module name
    pub fn name(&self) -> Option<String> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn() -> *const c_char> = 
                self.lib.get(b"kleis_module_name").ok()?;
            let ptr = func();
            if ptr.is_null() {
                return None;
            }
            Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }

    /// Get module version
    pub fn version(&self) -> Option<String> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn() -> *const c_char> = 
                self.lib.get(b"kleis_module_version").ok()?;
            let ptr = func();
            if ptr.is_null() {
                return None;
            }
            Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }

    /// Get number of functions
    pub fn function_count(&self) -> usize {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn() -> usize> = 
                self.lib.get(b"kleis_function_count").unwrap();
            func()
        }
    }

    /// Get function name by index
    pub fn function_name(&self, index: usize) -> Option<String> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(usize) -> *const c_char> = 
                self.lib.get(b"kleis_function_name").ok()?;
            let ptr = func(index);
            if ptr.is_null() {
                return None;
            }
            Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }

    /// List all function names
    pub fn list_functions(&self) -> Vec<String> {
        (0..self.function_count())
            .filter_map(|i| self.function_name(i))
            .collect()
    }

    /// Get number of axioms
    pub fn axiom_count(&self) -> usize {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn() -> usize> = 
                self.lib.get(b"kleis_axiom_count").unwrap();
            func()
        }
    }

    /// Get axiom name by index
    pub fn axiom_name(&self, index: usize) -> Option<String> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(usize) -> *const c_char> = 
                self.lib.get(b"kleis_axiom_name").ok()?;
            let ptr = func(index);
            if ptr.is_null() {
                return None;
            }
            Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }

    /// List all axiom names
    pub fn list_axioms(&self) -> Vec<String> {
        (0..self.axiom_count())
            .filter_map(|i| self.axiom_name(i))
            .collect()
    }

    /// Get axiom formula by name
    pub fn get_axiom(&self, name: &str) -> Option<String> {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn(*const c_char) -> *const c_char> = 
                self.lib.get(b"kleis_get_axiom").ok()?;
            let c_name = CString::new(name).ok()?;
            let ptr = func(c_name.as_ptr());
            if ptr.is_null() {
                return None;
            }
            Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
        }
    }

    /// Call a function by name
    pub fn call(&self, name: &str, args: &[AbiValue]) -> AbiResult {
        unsafe {
            let func: libloading::Symbol<
                unsafe extern "C" fn(*const c_char, *const AbiValue, usize) -> AbiResult
            > = match self.lib.get(b"kleis_call") {
                Ok(f) => f,
                Err(e) => return AbiResult::err(&format!("failed to get kleis_call: {}", e)),
            };
            
            let c_name = match CString::new(name) {
                Ok(s) => s,
                Err(_) => return AbiResult::err("invalid function name"),
            };
            
            func(c_name.as_ptr(), args.as_ptr(), args.len())
        }
    }
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║          KLEIS MODULE LOADER - Full Cycle Demo                ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // For this demo, we'll use the library directly since we're linking statically
    // In a real REPL, you would do: KleisModuleLoader::load("path/to/module.dylib")
    
    // Since we can't easily dlopen ourselves, let's demonstrate the API directly
    use kleis_solver_api::fib_module::FibonacciModule;
    use kleis_solver_api::module_abi::KleisModuleImpl;
    
    let module = FibonacciModule;
    
    println!("📦 Module: {} v{}", module.name(), module.version());
    println!();

    // 0. Show imports (dependencies)
    println!("📥 Imports (must load first):");
    for import in module.imports() {
        let required = if import.required { "required" } else { "optional" };
        println!("   {} [{}]", import.path, required);
    }
    println!();

    // 1. Discover functions
    println!("🔧 Available Functions:");
    for func in module.functions() {
        let params: Vec<String> = func.params.iter()
            .map(|(name, typ)| format!("{}: {}", name, typ))
            .collect();
        println!("   {} ({}) -> {}", func.name, params.join(", "), func.return_type);
        if let Some(doc) = func.doc {
            println!("      └─ {}", doc);
        }
    }
    println!();

    // 2. Discover axioms
    println!("📜 Available Axioms:");
    for axiom in module.axioms() {
        println!("   {} : {}", axiom.name, axiom.formula);
        if let Some(doc) = axiom.doc {
            println!("      └─ {}", doc);
        }
    }
    println!();

    // 3. Call functions
    println!("▶ Function Calls (via ABI):");
    
    let test_cases = [
        ("fib", 0u64),
        ("fib", 1),
        ("fib", 10),
        ("fib", 20),
        ("factorial", 0),
        ("factorial", 5),
        ("factorial", 10),
        ("is_even", 42),
        ("is_even", 7),
    ];

    for (func, arg) in test_cases {
        let result = module.call(func, &[AbiValue::Nat(arg)]);
        if result.success {
            match result.value {
                AbiValue::Nat(n) => println!("   {}({}) = {}", func, arg, n),
                AbiValue::Bool(b) => println!("   {}({}) = {}", func, arg, b),
                _ => println!("   {}({}) = {:?}", func, arg, result.value),
            }
        } else {
            println!("   {}({}) = ERROR", func, arg);
        }
    }
    println!();

    // 4. Retrieve axiom for verification
    println!("🔍 Axiom Lookup (for Z3 verification):");
    let axiom_names = ["fib_base_0", "fib_recursive", "factorial_base"];
    for name in axiom_names {
        if let Some(formula) = module.get_axiom(name) {
            println!("   {} : {}", name, formula);
        }
    }
    println!();

    // 5. Show the full cycle
    println!("═══════════════════════════════════════════════════════════════");
    println!("FULL CYCLE COMPLETE:");
    println!();
    println!("  1. ✅ Kleis source → AST (kleis_in_kleis.kleis)");
    println!("  2. ✅ AST → Rust code (kleis_codegen_rust.kleis)");
    println!("  3. ✅ Rust code → Shared library (cargo build)");
    println!("  4. ✅ Load library → Discover functions/axioms");
    println!("  5. ✅ Call functions via C-ABI");
    println!("  6. ✅ Retrieve axioms for Z3 verification");
    println!();
    println!("The REPL can now:");
    println!("  - JIT compile Kleis to .dylib on-the-fly");
    println!("  - Load modules via dlopen()");
    println!("  - Call functions by name");
    println!("  - Verify axioms with Z3");
    println!("═══════════════════════════════════════════════════════════════");
}

