//! Procedural macros for Kleis testing
//!
//! Provides `#[requires_kleis("path/to/file.kleis")]` attribute
//! that loads Kleis files and their axioms before running a test.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Load Kleis files and their axioms before running a test
///
/// This attribute loads the specified Kleis file(s) into the registry
/// and asserts their axioms into Z3 before the test runs.
///
/// # Usage
///
/// ```ignore
/// #[requires_kleis("stdlib/tensors_concrete.kleis")]
/// #[test]
/// fn test_concrete_tensor() {
///     // registry and backend are available with axioms loaded
/// }
/// ```
///
/// Multiple files can be specified:
/// ```ignore
/// #[requires_kleis("stdlib/tensors.kleis", "stdlib/tensors_concrete.kleis")]
/// #[test]
/// fn test_with_multiple_files() {
///     // Both files loaded
/// }
/// ```
///
/// # Generated Code
///
/// The macro wraps your test function to:
/// 1. Create a StructureRegistry
/// 2. Load stdlib (types, prelude, matrices, tensors)
/// 3. Load the specified file(s)
/// 4. Create a Z3Backend
/// 5. Assert all axioms from the registry
/// 6. Make `registry` and `backend` available to your test
#[proc_macro_attribute]
pub fn requires_kleis(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Parse the attribute arguments (comma-separated string literals)
    let files: Vec<String> = if attr.is_empty() {
        vec![]
    } else {
        // Parse as comma-separated string literals
        let attr_str = attr.to_string();
        attr_str
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };

    let fn_name = &input.sig.ident;
    let fn_block = &input.block;
    let fn_attrs = &input.attrs;
    let fn_vis = &input.vis;

    // Generate file loading code
    let file_loads: Vec<_> = files
        .iter()
        .map(|file| {
            quote! {
                registry.load_from_file(#file)
                    .expect(&format!("Failed to load {}", #file));
                println!("   📚 Loaded {}", #file);
            }
        })
        .collect();

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis fn #fn_name() {
            use kleis::structure_registry::StructureRegistry;
            use kleis::solvers::z3::backend::Z3Backend;
            use kleis::solvers::backend::SolverBackend;

            // Setup: Create registry and load files
            let mut registry = StructureRegistry::new();

            // Load stdlib first
            if let Err(e) = registry.load_stdlib() {
                println!("   ⚠️ Warning: Failed to load stdlib: {}", e);
            }

            // Load required files
            #(#file_loads)*

            // Create Z3 backend and load axioms
            let mut backend = Z3Backend::new(&registry)
                .expect("Failed to create Z3 backend");

            let axiom_count = backend.assert_axioms_from_registry()
                .expect("Failed to assert axioms");
            println!("   ✅ Loaded {} axioms into Z3", axiom_count);

            // Run the actual test
            #fn_block
        }
    };

    TokenStream::from(expanded)
}
