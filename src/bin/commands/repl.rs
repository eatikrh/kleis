//! REPL subcommand - Interactive Read-Eval-Print Loop
//!
//! Runs the REPL in-process using the library module.

use kleis::context::SharedContext;

pub fn run(ctx: SharedContext) {
    if let Err(e) = kleis::repl::run_repl_with_context(ctx) {
        eprintln!("REPL error: {}", e);
        std::process::exit(1);
    }
}
