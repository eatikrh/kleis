// ============================================================================
// KLEIS RUNTIME LIBRARY
// ============================================================================
//
// This module provides the core runtime types for Kleis cross-compiled code:
//
//   - Sym<T>: Symbolic values (concrete or unknown)
//   - SymReal, SymInt, SymBool: Type aliases
//   - KleisSolver: Z3-backed solver for constraints
//   - module_abi: COM-like ABI for module discovery (generated)
//
// Generated Kleis modules export C-compatible functions:
//   - kleis_manifest()       : Discover functions/axioms
//   - kleis_call()           : Call function by name
//   - kleis_get_axiom()      : Get axiom formula
//   - kleis_function_names() : List function names
//
// The REPL loads modules via dlopen/LoadLibrary - no central registry.
//
// ============================================================================

pub mod symbolic;
pub mod solver;
pub mod module_abi;
pub mod fib_module;  // Example generated module

// Re-export for convenience
pub use symbolic::*;
pub use solver::*;
pub use module_abi::*;

