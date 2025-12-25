//! Simple file-based logging for Kleis
//!
//! Avoids eprintln which can interfere with DAP/LSP protocol over stdio.
//! Similar to SLF4J: a simple facade that writes to a log file.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

/// Global log file handle
static LOG_FILE: OnceLock<Mutex<Option<File>>> = OnceLock::new();

/// Initialize logging to a file
pub fn init_file_logging(path: impl Into<PathBuf>) {
    let path = path.into();
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok();
    
    let _ = LOG_FILE.set(Mutex::new(file));
}

/// Initialize logging with default path
pub fn init_default_logging() {
    let path = std::env::temp_dir().join("kleis-debug.log");
    init_file_logging(path);
}

/// Log a message to the file
pub fn log(level: &str, component: &str, message: &str) {
    if let Some(mutex) = LOG_FILE.get() {
        if let Ok(mut guard) = mutex.lock() {
            if let Some(ref mut file) = *guard {
                let timestamp = chrono_lite_timestamp();
                let _ = writeln!(file, "[{}] [{}] [{}] {}", timestamp, level, component, message);
                let _ = file.flush();
            }
        }
    }
}

/// Simple timestamp without chrono dependency
fn chrono_lite_timestamp() -> String {
    use std::time::SystemTime;
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => {
            let secs = d.as_secs();
            let millis = d.subsec_millis();
            format!("{}.{:03}", secs, millis)
        }
        Err(_) => "0.000".to_string()
    }
}

/// Convenience macros for logging
#[macro_export]
macro_rules! dap_log {
    ($($arg:tt)*) => {
        $crate::logging::log("DEBUG", "dap", &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! eval_log {
    ($($arg:tt)*) => {
        $crate::logging::log("DEBUG", "eval", &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! hook_log {
    ($($arg:tt)*) => {
        $crate::logging::log("DEBUG", "hook", &format!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging() {
        init_default_logging();
        log("INFO", "test", "Test message");
    }
}

