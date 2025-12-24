//! LSP subcommand - Start the Language Server Protocol server
//!
//! Runs the LSP in-process using the library module.

use kleis::context::SharedContext;

pub fn run(ctx: SharedContext) {
    if let Err(e) = kleis::lsp::run_lsp_with_context(ctx) {
        // In stdio mode, we can't print to stderr as it may corrupt protocol
        // Write to log file instead
        let _ = std::fs::write("/tmp/kleis-lsp-error.log", format!("{}", e));
        std::process::exit(1);
    }
}
