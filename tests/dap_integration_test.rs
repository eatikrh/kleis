//! DAP Integration Tests
//!
//! These tests verify the Debug Adapter Protocol implementation by simulating
//! a DAP client (like VS Code) and checking the server's responses.

// These utility functions are for future TCP-based testing
#![allow(dead_code)]

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Helper to send a DAP message over a stream
fn send_dap_message<W: Write>(writer: &mut W, msg: &serde_json::Value) -> std::io::Result<()> {
    let body = msg.to_string();
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    writer.write_all(header.as_bytes())?;
    writer.write_all(body.as_bytes())?;
    writer.flush()
}

/// Helper to receive a DAP message from a stream
fn recv_dap_message<R: BufRead>(reader: &mut R) -> std::io::Result<Option<serde_json::Value>> {
    // Read Content-Length header
    let mut header = String::new();
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            return Ok(None); // EOF
        }
        header.push_str(&line);
        if header.ends_with("\r\n\r\n") {
            break;
        }
    }

    // Parse Content-Length
    let content_length: usize = header
        .lines()
        .find(|l| l.to_lowercase().starts_with("content-length:"))
        .and_then(|l| l.split(':').nth(1))
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);

    if content_length == 0 {
        return Ok(None);
    }

    // Read body
    let mut body = vec![0u8; content_length];
    std::io::Read::read_exact(reader, &mut body)?;

    let msg: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    Ok(Some(msg))
}

/// A DAP test client that connects to a server
struct DapTestClient {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
    seq: i32,
}

impl DapTestClient {
    fn connect(port: u16) -> std::io::Result<Self> {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port))?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        let reader = BufReader::new(stream.try_clone()?);
        let writer = stream;
        Ok(Self {
            reader,
            writer,
            seq: 0,
        })
    }

    fn next_seq(&mut self) -> i32 {
        self.seq += 1;
        self.seq
    }

    fn send_request(
        &mut self,
        command: &str,
        arguments: Option<serde_json::Value>,
    ) -> std::io::Result<()> {
        let mut msg = serde_json::json!({
            "seq": self.next_seq(),
            "type": "request",
            "command": command,
        });
        if let Some(args) = arguments {
            msg["arguments"] = args;
        }
        send_dap_message(&mut self.writer, &msg)
    }

    fn recv(&mut self) -> std::io::Result<Option<serde_json::Value>> {
        recv_dap_message(&mut self.reader)
    }

    fn recv_response(&mut self, command: &str) -> std::io::Result<serde_json::Value> {
        loop {
            match self.recv()? {
                Some(msg) if msg["type"] == "response" && msg["command"] == command => {
                    return Ok(msg);
                }
                Some(msg) => {
                    eprintln!("Skipping non-matching message: {:?}", msg);
                    continue;
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        format!("No response for {}", command),
                    ));
                }
            }
        }
    }

    fn recv_event(&mut self, event_name: &str) -> std::io::Result<serde_json::Value> {
        loop {
            match self.recv()? {
                Some(msg) if msg["type"] == "event" && msg["event"] == event_name => {
                    return Ok(msg);
                }
                Some(msg) => {
                    eprintln!("Skipping: {:?}", msg);
                    continue;
                }
                None => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        format!("No event {}", event_name),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use kleis::dap::DapDebugger;

    /// Test the DAP message parsing and handling without network
    #[test]
    fn test_dap_initialize_response() {
        let mut debugger = DapDebugger::new(None);

        // Send initialize request
        let response = debugger.handle_message(
            r#"{"seq":1,"type":"request","command":"initialize","arguments":{"clientID":"test"}}"#,
        );

        assert!(response.is_some(), "Should return a response");
        let resp: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();

        assert_eq!(resp["type"], "response");
        assert_eq!(resp["command"], "initialize");
        assert_eq!(resp["success"], true);
        assert!(resp["body"]["supportsConfigurationDoneRequest"]
            .as_bool()
            .unwrap_or(false));
    }

    #[test]
    fn test_dap_initialized_event_is_queued() {
        let mut debugger = DapDebugger::new(None);

        // Send initialize request
        let _ = debugger.handle_message(
            r#"{"seq":1,"type":"request","command":"initialize","arguments":{"clientID":"test"}}"#,
        );

        // Check that initialized event was queued
        assert!(
            !debugger.pending_events.is_empty(),
            "Should have pending events"
        );

        let event: serde_json::Value = serde_json::from_str(&debugger.pending_events[0]).unwrap();
        assert_eq!(event["type"], "event");
        assert_eq!(event["event"], "initialized");
    }

    #[test]
    fn test_dap_full_launch_sequence() {
        let mut debugger = DapDebugger::new(None);

        // 1. Initialize
        let resp = debugger.handle_message(
            r#"{"seq":1,"type":"request","command":"initialize","arguments":{"clientID":"test"}}"#,
        );
        assert!(resp.is_some());
        let resp_json: serde_json::Value = serde_json::from_str(&resp.unwrap()).unwrap();
        assert_eq!(resp_json["success"], true, "initialize should succeed");

        // Check initialized event was queued
        assert!(
            !debugger.pending_events.is_empty(),
            "Should have initialized event"
        );
        let init_event: serde_json::Value =
            serde_json::from_str(&debugger.pending_events[0]).unwrap();
        assert_eq!(init_event["event"], "initialized");
        debugger.pending_events.clear();

        // 2. Launch
        let program = std::env::current_dir()
            .unwrap()
            .join("examples/example_blocks.kleis");
        let launch_msg = format!(
            r#"{{"seq":2,"type":"request","command":"launch","arguments":{{"program":"{}","stopOnEntry":true}}}}"#,
            program.display()
        );
        let resp = debugger.handle_message(&launch_msg);
        assert!(resp.is_some());
        let resp_json: serde_json::Value = serde_json::from_str(&resp.unwrap()).unwrap();
        assert_eq!(resp_json["success"], true, "launch should succeed");

        // 3. ConfigurationDone
        let resp =
            debugger.handle_message(r#"{"seq":3,"type":"request","command":"configurationDone"}"#);
        assert!(resp.is_some());
        let resp_json: serde_json::Value = serde_json::from_str(&resp.unwrap()).unwrap();
        assert_eq!(
            resp_json["success"], true,
            "configurationDone should succeed"
        );

        // Check that stopped event was queued
        let has_stopped = debugger.pending_events.iter().any(|e| {
            let event: serde_json::Value = serde_json::from_str(e).unwrap_or_default();
            event["event"] == "stopped"
        });
        assert!(
            has_stopped,
            "Should have stopped event after configurationDone"
        );

        // 4. Threads
        let resp = debugger.handle_message(r#"{"seq":4,"type":"request","command":"threads"}"#);
        assert!(resp.is_some());
        let resp_json: serde_json::Value = serde_json::from_str(&resp.unwrap()).unwrap();
        assert_eq!(resp_json["success"], true);
        assert!(!resp_json["body"]["threads"].as_array().unwrap().is_empty());

        // 5. StackTrace
        let resp = debugger.handle_message(
            r#"{"seq":5,"type":"request","command":"stackTrace","arguments":{"threadId":1}}"#,
        );
        assert!(resp.is_some());
        let resp_json: serde_json::Value = serde_json::from_str(&resp.unwrap()).unwrap();
        assert_eq!(resp_json["success"], true);
        let frames = resp_json["body"]["stackFrames"].as_array().unwrap();
        assert!(!frames.is_empty(), "Should have at least one stack frame");
        assert!(
            frames[0]["line"].as_u64().unwrap() > 0,
            "Stack frame should have a line number"
        );
    }

    #[test]
    fn test_dap_stepping() {
        let mut debugger = DapDebugger::new(None);

        // Initialize and launch
        debugger
            .handle_message(r#"{"seq":1,"type":"request","command":"initialize","arguments":{}}"#);
        debugger.pending_events.clear();

        let program = std::env::current_dir()
            .unwrap()
            .join("examples/example_blocks.kleis");
        let launch_msg = format!(
            r#"{{"seq":2,"type":"request","command":"launch","arguments":{{"program":"{}"}}}}"#,
            program.display()
        );
        debugger.handle_message(&launch_msg);
        debugger.handle_message(r#"{"seq":3,"type":"request","command":"configurationDone"}"#);
        debugger.pending_events.clear();

        // Step next
        let resp = debugger.handle_message(
            r#"{"seq":4,"type":"request","command":"next","arguments":{"threadId":1}}"#,
        );
        assert!(resp.is_some());
        let resp_json: serde_json::Value = serde_json::from_str(&resp.unwrap()).unwrap();
        assert_eq!(resp_json["success"], true);

        // Check that stopped event was queued
        let has_stopped = debugger.pending_events.iter().any(|e| {
            let event: serde_json::Value = serde_json::from_str(e).unwrap_or_default();
            event["event"] == "stopped" && event["body"]["reason"] == "step"
        });
        assert!(
            has_stopped,
            "Should have stopped event with reason 'step' after next"
        );
    }
}
