//! Check subcommand - Parse and type-check a Kleis file

use kleis::context::SharedContext;
use std::path::Path;

pub fn run(file: &Path, verbose: bool, ctx: SharedContext) {
    // Use shared context to open and parse the document
    let mut ctx_guard = ctx.write().expect("Failed to acquire context lock");

    let path = file.to_path_buf();

    if verbose {
        println!("Parsing {}...", file.display());
    }

    // Open document in shared context
    if let Err(e) = ctx_guard.open_document(path.clone()) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    // Check for parse errors
    let doc = ctx_guard.get_document(&path).unwrap();

    if !doc.diagnostics.is_empty() {
        eprintln!("Parse errors in {}:", file.display());
        for diag in &doc.diagnostics {
            eprintln!("  position {}: {}", diag.start, diag.message);
        }
        std::process::exit(1);
    }

    if verbose {
        if let Some(ref program) = doc.program {
            println!("  ✓ Parsed {} top-level items", program.items.len());
        }
    }

    println!("✓ {} - OK", file.display());

    // TODO: Add type checking when TypeChecker is integrated into context
}
