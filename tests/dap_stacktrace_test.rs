//! Integration test for DAP stackTrace - debugging the "unknown" source issue

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn send_lsp_message(stdin: &mut impl Write, msg: &str) {
    let header = format!("Content-Length: {}\r\n\r\n", msg.len());
    stdin.write_all(header.as_bytes()).unwrap();
    stdin.write_all(msg.as_bytes()).unwrap();
    stdin.flush().unwrap();
}

fn read_lsp_response(stdout: &mut BufReader<impl Read>) -> Option<String> {
    let mut header = String::new();
    stdout.read_line(&mut header).ok()?;

    if !header.starts_with("Content-Length:") {
        return None;
    }

    let len: usize = header
        .trim_start_matches("Content-Length:")
        .trim()
        .parse()
        .ok()?;

    // Read empty line
    let mut empty = String::new();
    stdout.read_line(&mut empty).ok()?;

    // Read content
    let mut content = vec![0u8; len];
    stdout.read_exact(&mut content).ok()?;

    String::from_utf8(content).ok()
}

fn send_dap_message(stream: &mut TcpStream, msg: &str) {
    let header = format!("Content-Length: {}\r\n\r\n", msg.len());
    stream.write_all(header.as_bytes()).unwrap();
    stream.write_all(msg.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn read_dap_response(stream: &mut TcpStream) -> Option<serde_json::Value> {
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok()?;

    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut header = String::new();
    reader.read_line(&mut header).ok()?;

    if !header.starts_with("Content-Length:") {
        eprintln!("Bad header: {}", header);
        return None;
    }

    let len: usize = header
        .trim_start_matches("Content-Length:")
        .trim()
        .parse()
        .ok()?;

    // Read empty line
    let mut empty = String::new();
    reader.read_line(&mut empty).ok()?;

    // Read content
    let mut content = vec![0u8; len];
    reader.read_exact(&mut content).ok()?;

    let json_str = String::from_utf8(content).ok()?;
    serde_json::from_str(&json_str).ok()
}

#[test]
fn test_dap_stacktrace_has_source() {
    // Start the kleis server (use debug build for tests)
    let mut server = Command::new("./target/debug/kleis")
        .arg("server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start kleis server");

    let mut stdin = server.stdin.take().unwrap();
    let stdout = server.stdout.take().unwrap();
    let stderr = server.stderr.take().unwrap();
    let mut stdout_reader = BufReader::new(stdout);

    // Spawn a thread to read stderr (for debugging)
    let stderr_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        let mut logs = Vec::new();
        for line in reader.lines().map_while(Result::ok) {
            logs.push(line);
        }
        logs
    });

    // Give server time to start
    thread::sleep(Duration::from_millis(500));

    // Send LSP initialize
    let init_msg = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#;
    send_lsp_message(&mut stdin, init_msg);
    thread::sleep(Duration::from_millis(300));

    // Read response (we don't care about contents)
    let _ = read_lsp_response(&mut stdout_reader);

    // Send initialized notification
    let initialized = r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#;
    send_lsp_message(&mut stdin, initialized);
    thread::sleep(Duration::from_millis(100));

    // Request DAP server start - use absolute path
    let program_path = std::env::current_dir()
        .unwrap()
        .join("examples/debug_main.kleis")
        .to_string_lossy()
        .to_string();

    let debug_cmd = format!(
        r#"{{"jsonrpc":"2.0","id":2,"method":"workspace/executeCommand","params":{{"command":"kleis.startDebugSession","arguments":["{}"]}}}}"#,
        program_path.replace("\\", "\\\\").replace("\"", "\\\"")
    );
    send_lsp_message(&mut stdin, &debug_cmd);
    thread::sleep(Duration::from_millis(500));

    // Read response to get DAP port
    let response = read_lsp_response(&mut stdout_reader);
    println!("LSP response: {:?}", response);

    let port: u16 = response
        .as_ref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.get("result")?.get("port")?.as_u64())
        .map(|p| p as u16)
        .expect("Failed to get DAP port from response");

    println!("DAP port: {}", port);

    // Connect to DAP server
    let mut dap =
        TcpStream::connect(format!("127.0.0.1:{}", port)).expect("Failed to connect to DAP server");

    // DAP initialize
    let dap_init =
        r#"{"seq":1,"type":"request","command":"initialize","arguments":{"adapterID":"kleis"}}"#;
    send_dap_message(&mut dap, dap_init);
    thread::sleep(Duration::from_millis(300));
    let init_resp = read_dap_response(&mut dap);
    println!("DAP initialize response: {:?}", init_resp);

    // DAP launch with absolute path
    let launch_msg = format!(
        r#"{{"seq":2,"type":"request","command":"launch","arguments":{{"program":"{}","stopOnEntry":true}}}}"#,
        program_path.replace("\\", "\\\\").replace("\"", "\\\"")
    );
    send_dap_message(&mut dap, &launch_msg);
    thread::sleep(Duration::from_millis(300));
    let launch_resp = read_dap_response(&mut dap);
    println!("DAP launch response: {:?}", launch_resp);

    // DAP configurationDone
    let cfg_done = r#"{"seq":3,"type":"request","command":"configurationDone"}"#;
    send_dap_message(&mut dap, cfg_done);
    thread::sleep(Duration::from_millis(500));

    // Read multiple responses (response + events)
    for _ in 0..3 {
        if let Some(resp) = read_dap_response(&mut dap) {
            println!("DAP configurationDone response/event: {:?}", resp);
        }
    }

    // DAP stackTrace - THIS IS THE KEY TEST
    let stack_trace =
        r#"{"seq":4,"type":"request","command":"stackTrace","arguments":{"threadId":1}}"#;
    send_dap_message(&mut dap, stack_trace);
    thread::sleep(Duration::from_millis(300));
    let stack_resp = read_dap_response(&mut dap);

    println!("\n=== STACK TRACE RESPONSE ===");
    println!(
        "{}",
        serde_json::to_string_pretty(&stack_resp).unwrap_or_default()
    );

    // Check the response
    let stack_frames = stack_resp
        .as_ref()
        .and_then(|v| v.get("body"))
        .and_then(|b| b.get("stackFrames"))
        .and_then(|sf| sf.as_array());

    if let Some(frames) = stack_frames {
        for (i, frame) in frames.iter().enumerate() {
            let source_name = frame
                .get("source")
                .and_then(|s| s.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("(no source name)");
            let source_path = frame
                .get("source")
                .and_then(|s| s.get("path"))
                .and_then(|p| p.as_str())
                .unwrap_or("(no source path)");
            let line = frame.get("line").and_then(|l| l.as_u64()).unwrap_or(0);

            println!(
                "Frame {}: name={}, path={}, line={}",
                i, source_name, source_path, line
            );

            // Assert that source is not "unknown"
            assert_ne!(
                source_name, "unknown",
                "Source name should not be 'unknown'"
            );
            assert!(!source_path.is_empty(), "Source path should not be empty");
        }
    } else {
        panic!("No stack frames in response");
    }

    // Cleanup
    drop(dap);
    drop(stdin);
    let _ = server.kill();
    let _ = server.wait(); // Wait for process to avoid zombie

    // Print stderr logs
    let logs = stderr_handle.join().unwrap_or_default();
    println!("\n=== SERVER STDERR LOGS ===");
    for log in logs {
        println!("{}", log);
    }
}
