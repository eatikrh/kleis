//! Isabelle Server Connection Management
//!
//! Handles TCP connection to Isabelle server, authentication, and message passing.
//!
//! # Protocol
//!
//! Isabelle server uses a simple text-based protocol over TCP:
//! 1. Client connects to server port
//! 2. Client sends password for authentication
//! 3. Server responds with "OK" + JSON server info
//! 4. Client sends commands, server responds with JSON
//!
//! See: Isabelle System Manual, Chapter 4
//! <https://isabelle.in.tum.de/dist/Isabelle2025-1/doc/system.pdf>

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Default connection timeout
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Default read timeout for responses
const READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Isabelle server connection
///
/// Manages a TCP connection to an Isabelle server instance.
/// Handles authentication, message sending, and response parsing.
pub struct IsabelleConnection {
    /// TCP stream to server
    stream: TcpStream,

    /// Buffered reader for responses
    reader: BufReader<TcpStream>,

    /// Server info received after authentication
    server_info: Option<ServerInfo>,

    /// Whether connection is authenticated
    authenticated: bool,
}

/// Server information returned after authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Isabelle version name (e.g., "Isabelle2025-1")
    #[serde(default)]
    pub isabelle_name: String,

    /// Isabelle version ID
    #[serde(default)]
    pub isabelle_id: String,

    /// Available tasks/commands
    #[serde(default)]
    pub tasks: Vec<String>,
}

/// Result of an Isabelle server command
#[derive(Debug, Clone)]
pub enum CommandResult {
    /// Command succeeded with optional response data
    Ok(Option<serde_json::Value>),

    /// Command failed with error message
    Error(String),

    /// Command is still running (async task)
    Running { task_id: String },
}

impl IsabelleConnection {
    /// Connect to an Isabelle server
    ///
    /// # Arguments
    /// * `host` - Server hostname (usually "127.0.0.1")
    /// * `port` - Server port
    /// * `password` - Authentication password
    ///
    /// # Returns
    /// Connected and authenticated connection, or error
    pub fn connect(host: &str, port: u16, password: &str) -> Result<Self, String> {
        let addr = format!("{}:{}", host, port);

        // Connect with timeout
        let stream = TcpStream::connect_timeout(
            &addr
                .parse()
                .map_err(|e| format!("Invalid address: {}", e))?,
            CONNECT_TIMEOUT,
        )
        .map_err(|e| format!("Failed to connect to Isabelle server at {}: {}", addr, e))?;

        // Set read timeout
        stream
            .set_read_timeout(Some(READ_TIMEOUT))
            .map_err(|e| format!("Failed to set read timeout: {}", e))?;

        // Clone stream for reader
        let reader_stream = stream
            .try_clone()
            .map_err(|e| format!("Failed to clone stream: {}", e))?;

        let mut conn = IsabelleConnection {
            stream,
            reader: BufReader::new(reader_stream),
            server_info: None,
            authenticated: false,
        };

        // Authenticate
        conn.authenticate(password)?;

        Ok(conn)
    }

    /// Authenticate with the server
    fn authenticate(&mut self, password: &str) -> Result<(), String> {
        // Send password
        self.send_raw(password)?;

        // Read response
        let response = self.read_line()?;

        // Parse response: "OK <json>" or "ERROR <message>"
        if response.starts_with("OK") {
            // Parse server info from JSON after "OK "
            if let Some(json_str) = response.strip_prefix("OK ") {
                if let Ok(info) = serde_json::from_str::<ServerInfo>(json_str.trim()) {
                    self.server_info = Some(info);
                }
            }
            self.authenticated = true;
            Ok(())
        } else if response.starts_with("ERROR") {
            Err(format!(
                "Authentication failed: {}",
                response.strip_prefix("ERROR ").unwrap_or(&response)
            ))
        } else {
            Err(format!("Unexpected response: {}", response))
        }
    }

    /// Send a command to the server
    ///
    /// # Arguments
    /// * `command` - Command name (e.g., "echo", "session_start", "use_theories")
    /// * `args` - Command arguments as JSON value
    ///
    /// # Returns
    /// Command result
    pub fn send_command(
        &mut self,
        command: &str,
        args: &serde_json::Value,
    ) -> Result<CommandResult, String> {
        if !self.authenticated {
            return Err("Not authenticated".to_string());
        }

        // Format: command <json_args>
        let msg = format!("{} {}", command, serde_json::to_string(args).unwrap());
        self.send_raw(&msg)?;

        // Read response
        let response = self.read_line()?;

        // Parse response
        self.parse_response(&response)
    }

    /// Send a simple command without JSON args
    pub fn send_simple(&mut self, command: &str) -> Result<CommandResult, String> {
        if !self.authenticated {
            return Err("Not authenticated".to_string());
        }

        self.send_raw(command)?;
        let response = self.read_line()?;
        self.parse_response(&response)
    }

    /// Parse a server response
    fn parse_response(&self, response: &str) -> Result<CommandResult, String> {
        if response.starts_with("OK") {
            // Extract JSON after "OK "
            let json_part = response.strip_prefix("OK").unwrap_or("").trim();
            if json_part.is_empty() {
                Ok(CommandResult::Ok(None))
            } else {
                let value = serde_json::from_str(json_part)
                    .map_err(|e| format!("Failed to parse JSON: {}", e))?;
                Ok(CommandResult::Ok(Some(value)))
            }
        } else if response.starts_with("ERROR") {
            let msg = response.strip_prefix("ERROR").unwrap_or(response).trim();
            Ok(CommandResult::Error(msg.to_string()))
        } else if response.starts_with("RUNNING") {
            // Async task started
            let task_id = response
                .strip_prefix("RUNNING")
                .unwrap_or("")
                .trim()
                .to_string();
            Ok(CommandResult::Running { task_id })
        } else {
            // Unknown response format - treat as OK with raw data
            Ok(CommandResult::Ok(Some(serde_json::Value::String(
                response.to_string(),
            ))))
        }
    }

    /// Send raw text to server
    fn send_raw(&mut self, msg: &str) -> Result<(), String> {
        writeln!(self.stream, "{}", msg).map_err(|e| format!("Failed to send: {}", e))?;
        self.stream
            .flush()
            .map_err(|e| format!("Failed to flush: {}", e))?;
        Ok(())
    }

    /// Read a line from server
    fn read_line(&mut self) -> Result<String, String> {
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .map_err(|e| format!("Failed to read: {}", e))?;
        Ok(line.trim().to_string())
    }

    /// Get server info (available after authentication)
    pub fn server_info(&self) -> Option<&ServerInfo> {
        self.server_info.as_ref()
    }

    /// Check if connection is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    /// Close the connection
    pub fn close(self) -> Result<(), String> {
        // Connection closes when dropped, but we can try to send shutdown
        drop(self.stream);
        Ok(())
    }

    /// Set read timeout
    pub fn set_timeout(&mut self, timeout: Duration) -> Result<(), String> {
        self.stream
            .set_read_timeout(Some(timeout))
            .map_err(|e| format!("Failed to set timeout: {}", e))
    }

    /// Read the next message from the server (for async responses)
    ///
    /// Isabelle server sends async messages in the format:
    /// - OK <json> - Success response
    /// - NOTE <json> - Informational message  
    /// - FINISHED <json> - Async task completed
    /// - FAILED <json> - Async task failed
    /// - <length>\n<data> - Length-prefixed message (alternative format)
    ///
    /// Returns (kind, json) where kind is OK, NOTE, FINISHED, etc.
    pub fn read_next_message(&mut self) -> Result<Option<(String, serde_json::Value)>, String> {
        let line = match self.read_line_nonblocking() {
            Ok(Some(line)) => line,
            Ok(None) => return Ok(None),
            Err(e) => return Err(e),
        };

        if line.is_empty() {
            return Ok(None);
        }

        // Check for known message types
        for prefix in &["OK", "NOTE", "FINISHED", "FAILED", "ERROR", "RUNNING"] {
            if line.starts_with(prefix) {
                let json_part = line.strip_prefix(prefix).unwrap_or("").trim();
                if json_part.is_empty() {
                    return Ok(Some((prefix.to_string(), serde_json::Value::Null)));
                }
                match serde_json::from_str(json_part) {
                    Ok(json) => return Ok(Some((prefix.to_string(), json))),
                    Err(_) => {
                        // Not valid JSON, return as string
                        return Ok(Some((
                            prefix.to_string(),
                            serde_json::Value::String(json_part.to_string()),
                        )));
                    }
                }
            }
        }

        // Check if it's a length prefix (number followed by newline)
        if let Ok(_length) = line.parse::<usize>() {
            // Read the actual message on the next line
            if let Ok(Some(data)) = self.read_line_nonblocking() {
                // Try to parse as message
                return self.parse_message_line(&data);
            }
        }

        // Unknown format - try to parse as JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
            return Ok(Some(("UNKNOWN".to_string(), json)));
        }

        Ok(None)
    }

    /// Parse a message line into (kind, json)
    fn parse_message_line(&self, line: &str) -> Result<Option<(String, serde_json::Value)>, String> {
        for prefix in &["OK", "NOTE", "FINISHED", "FAILED", "ERROR", "RUNNING"] {
            if line.starts_with(prefix) {
                let json_part = line.strip_prefix(prefix).unwrap_or("").trim();
                if json_part.is_empty() {
                    return Ok(Some((prefix.to_string(), serde_json::Value::Null)));
                }
                match serde_json::from_str(json_part) {
                    Ok(json) => return Ok(Some((prefix.to_string(), json))),
                    Err(_) => continue,
                }
            }
        }
        Ok(None)
    }

    /// Read a line from server without blocking (returns None if no data)
    fn read_line_nonblocking(&mut self) -> Result<Option<String>, String> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => Ok(None), // EOF
            Ok(_) => Ok(Some(line.trim().to_string())),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(None),
            Err(e) => Err(format!("Failed to read: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_info_deserialize() {
        let json = r#"{"isabelle_name":"Isabelle2025-1","isabelle_id":"abc123","tasks":["echo","session_start"]}"#;
        let info: ServerInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.isabelle_name, "Isabelle2025-1");
        assert_eq!(info.tasks.len(), 2);
    }

    #[test]
    fn test_server_info_partial() {
        // Server may not return all fields
        let json = r#"{"isabelle_name":"Isabelle2025"}"#;
        let info: ServerInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.isabelle_name, "Isabelle2025");
        assert!(info.tasks.is_empty());
    }
}
