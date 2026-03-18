//! Run subcommand - Execute a Kleis file

use kleis::context::SharedContext;
use std::path::Path;

pub fn run(file: &Path, ctx: SharedContext) {
    let path = file.to_path_buf();

    // Use shared context
    let mut ctx_guard = ctx.write().expect("Failed to acquire context lock");

    // Open document in shared context
    if let Err(e) = ctx_guard.open_document(path.clone()) {
        eprintln!("Error reading {}: {}", file.display(), e);
        std::process::exit(1);
    }

    // Check for parse errors
    {
        let doc = ctx_guard.get_document(&path).unwrap();
        if !doc.diagnostics.is_empty() {
            eprintln!("Parse errors in {}:", file.display());
            for diag in &doc.diagnostics {
                eprintln!("  position {}: {}", diag.start, diag.message);
            }
            std::process::exit(1);
        }
    }

    // Load program into evaluator
    if let Err(e) = ctx_guard.load_program(&path) {
        eprintln!("Error loading program: {}", e);
        std::process::exit(1);
    }

    println!("✓ {} loaded successfully", file.display());

    // Print what was loaded
    let doc = ctx_guard.get_document(&path).unwrap();
    if let Some(ref program) = doc.program {
        let functions = program.functions();
        if !functions.is_empty() {
            println!(
                "Defined functions: {}",
                functions
                    .iter()
                    .map(|f| f.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
}
