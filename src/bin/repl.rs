//! Kleis REPL - Standalone binary wrapper
//!
//! This is a thin wrapper around the kleis::repl module.
//! All REPL logic lives in src/repl.rs to avoid duplication.
//!
//! Usage: repl
//!        kleis repl   (via unified binary)

fn main() {
    let _config = kleis::config::load();

    if let Err(e) = kleis::repl::run_repl() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
