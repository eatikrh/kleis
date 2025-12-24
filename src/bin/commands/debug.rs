//! Debug subcommand - Start the Debug Adapter Protocol (DAP) server
//!
//! This implements the Debug Adapter Protocol for debugging Kleis programs
//! in IDEs like VS Code.
//!
//! **Important:** In stdio mode, NO output to stdout/stderr is allowed
//! as it would corrupt the DAP protocol.

use kleis::context::SharedContext;

pub fn run(port: Option<u16>, ctx: SharedContext) {
    match port {
        Some(p) => {
            // TCP mode: console output is safe
            eprintln!("🐛 Starting Kleis DAP server on port {}", p);
            if let Err(e) = kleis::dap::run_tcp_server_with_context(p, ctx) {
                eprintln!("DAP server error: {}", e);
                std::process::exit(1);
            }
        }
        None => {
            // stdio mode: NO console output allowed!
            // The DAP protocol uses stdin/stdout for communication.
            if let Err(e) = kleis::dap::run_stdio_server_with_context(ctx) {
                // Write error to a log file instead of stderr
                let _ = std::fs::write("/tmp/kleis-dap-error.log", format!("{}", e));
                std::process::exit(1);
            }
        }
    }
}
