//! End-to-end LSP test: spawns `link lsp`, exchanges JSON-RPC over stdio.
//!
//! Verifies the full protocol stack: framing, initialize handshake, document
//! sync, completion, hover, definition, document symbols, and diagnostics.
//!
//! These tests require the `link` binary to be built first (`cargo build`).
//! They are ignored by default to avoid coupling `cargo test -p linkc_lsp`
//! with a workspace build; run with `cargo test -p linkc_lsp -- --ignored`.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn link_exe() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let exe_name = if cfg!(windows) { "link.exe" } else { "link" };
    let path = manifest_dir
        .parent().unwrap()
        .parent().unwrap()
        .join("target")
        .join("debug")
        .join(exe_name);
    if !path.exists() {
        panic!("`link` binary not found at {}. Run `cargo build` first.", path.display());
    }
    path
}

fn send<W: Write>(w: &mut W, obj: serde_json::Value) {
    let body = serde_json::to_string(&obj).unwrap();
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    w.write_all(header.as_bytes()).unwrap();
    w.write_all(body.as_bytes()).unwrap();
    w.flush().unwrap();
}

fn read<R: Read>(r: &mut R) -> serde_json::Value {
    let mut content_length: Option<usize> = None;
    let mut buf = Vec::new();
    loop {
        let mut byte = [0u8; 1];
        let n = r.read(&mut byte).unwrap();
        if n == 0 { panic!("unexpected EOF while reading headers"); }
        buf.push(byte[0]);
        if buf.len() >= 4 && &buf[buf.len() - 4..] == b"\r\n\r\n" {
            break;
        }
    }
    let header_str = String::from_utf8_lossy(&buf);
    for line in header_str.lines() {
        if let Some(rest) = line.strip_prefix("Content-Length:") {
            content_length = rest.trim().parse().ok();
        }
    }
    let len = content_length.expect("missing Content-Length");
    let mut body = vec![0u8; len];
    r.read_exact(&mut body).unwrap();
    serde_json::from_slice(&body).unwrap()
}

fn spawn_server() -> std::process::Child {
    Command::new(link_exe())
        .arg("lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn `link lsp`")
}

/// initialize + initialized + didOpen; drain the diagnostics notification.
fn handshake_and_open(
    stdin: &mut impl Write,
    stdout: &mut impl Read,
    uri: &str,
    src: &str,
) {
    send(stdin, serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": { "processId": null, "rootUri": null, "capabilities": {} }
    }));
    let _ = read(stdout);
    send(stdin, serde_json::json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));
    send(stdin, serde_json::json!({
        "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": { "textDocument": { "uri": uri, "languageId": "link", "version": 0, "text": src } }
    }));
    let _ = read(stdout);
}

fn shutdown(stdin: &mut impl Write, stdout: &mut impl Read) {
    send(stdin, serde_json::json!({ "jsonrpc": "2.0", "id": 99, "method": "shutdown" }));
    let _ = read(stdout);
    send(stdin, serde_json::json!({ "jsonrpc": "2.0", "method": "exit" }));
}

#[test]
#[ignore = "requires `cargo build` to produce the link binary"]
fn lsp_initialize_handshake() {
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    send(&mut stdin, serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": { "processId": null, "rootUri": null, "capabilities": {} }
    }));
    let resp = read(&mut stdout);
    assert_eq!(resp["id"], 1);
    assert_eq!(resp["result"]["capabilities"]["textDocumentSync"], 1);
    assert_eq!(resp["result"]["capabilities"]["hoverProvider"], true);
    assert_eq!(resp["result"]["capabilities"]["definitionProvider"], true);
    assert_eq!(resp["result"]["capabilities"]["documentSymbolProvider"], true);
    assert!(resp["result"]["capabilities"]["completionProvider"].is_object());

    shutdown(&mut stdin, &mut stdout);
    child.wait().unwrap();
}

#[test]
#[ignore = "requires `cargo build` to produce the link binary"]
fn lsp_did_open_publishes_diagnostics() {
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    let src = "fn add(a: i64, b: i64) -> i64 {\n    return a + b;\n}\n";
    send(&mut stdin, serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": { "processId": null, "rootUri": null, "capabilities": {} }
    }));
    let _ = read(&mut stdout);
    send(&mut stdin, serde_json::json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));
    send(&mut stdin, serde_json::json!({
        "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": { "textDocument": { "uri": "file:///test.link", "languageId": "link", "version": 0, "text": src } }
    }));
    let diag = read(&mut stdout);
    assert_eq!(diag["method"], "textDocument/publishDiagnostics");
    assert_eq!(diag["params"]["uri"], "file:///test.link");
    assert_eq!(diag["params"]["diagnostics"].as_array().unwrap().len(), 0);

    shutdown(&mut stdin, &mut stdout);
    child.wait().unwrap();
}

#[test]
#[ignore = "requires `cargo build` to produce the link binary"]
fn lsp_completion_returns_keywords_builtins_and_symbols() {
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    let src = "fn add(a: i64, b: i64) -> i64 {\n    return a + b;\n}\n";
    handshake_and_open(&mut stdin, &mut stdout, "file:///test.link", src);

    send(&mut stdin, serde_json::json!({
        "jsonrpc": "2.0", "id": 2, "method": "textDocument/completion",
        "params": { "textDocument": { "uri": "file:///test.link" }, "position": { "line": 0, "character": 4 } }
    }));
    let resp = read(&mut stdout);
    assert_eq!(resp["id"], 2);
    let items = resp["result"]["items"].as_array().unwrap();
    let labels: Vec<&str> = items.iter().map(|i| i["label"].as_str().unwrap()).collect();
    assert!(labels.contains(&"fn"), "missing keyword 'fn'");
    assert!(labels.contains(&"println"), "missing builtin 'println'");
    assert!(labels.contains(&"i64"), "missing builtin type 'i64'");
    assert!(labels.contains(&"add"), "missing document symbol 'add'");

    shutdown(&mut stdin, &mut stdout);
    child.wait().unwrap();
}

#[test]
#[ignore = "requires `cargo build` to produce the link binary"]
fn lsp_hover_returns_signature() {
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    let src = "fn add(a: i64, b: i64) -> i64 {\n    return a + b;\n}\n";
    handshake_and_open(&mut stdin, &mut stdout, "file:///test.link", src);

    send(&mut stdin, serde_json::json!({
        "jsonrpc": "2.0", "id": 3, "method": "textDocument/hover",
        "params": { "textDocument": { "uri": "file:///test.link" }, "position": { "line": 0, "character": 4 } }
    }));
    let resp = read(&mut stdout);
    assert_eq!(resp["id"], 3);
    let content = resp["result"]["contents"]["value"].as_str().unwrap();
    assert!(content.contains("fn add(a: i64, b: i64) -> i64"), "hover content: {}", content);

    shutdown(&mut stdin, &mut stdout);
    child.wait().unwrap();
}

#[test]
#[ignore = "requires `cargo build` to produce the link binary"]
fn lsp_definition_jumps_to_declaration() {
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    let src = "fn add(a: i64, b: i64) -> i64 {\n    return a + b;\n}\n";
    handshake_and_open(&mut stdin, &mut stdout, "file:///test.link", src);

    send(&mut stdin, serde_json::json!({
        "jsonrpc": "2.0", "id": 4, "method": "textDocument/definition",
        "params": { "textDocument": { "uri": "file:///test.link" }, "position": { "line": 0, "character": 4 } }
    }));
    let resp = read(&mut stdout);
    assert_eq!(resp["id"], 4);
    assert_eq!(resp["result"]["range"]["start"]["line"], 0);
    assert_eq!(resp["result"]["range"]["start"]["character"], 3);

    shutdown(&mut stdin, &mut stdout);
    child.wait().unwrap();
}

#[test]
#[ignore = "requires `cargo build` to produce the link binary"]
fn lsp_document_symbols_outline() {
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    let src = "fn add(a: i64, b: i64) -> i64 {\n    return a + b;\n}\nstruct Point { x: i64, y: i64 }\n";
    handshake_and_open(&mut stdin, &mut stdout, "file:///test.link", src);

    send(&mut stdin, serde_json::json!({
        "jsonrpc": "2.0", "id": 5, "method": "textDocument/documentSymbol",
        "params": { "textDocument": { "uri": "file:///test.link" } }
    }));
    let resp = read(&mut stdout);
    assert_eq!(resp["id"], 5);
    let symbols = resp["result"].as_array().unwrap();
    assert_eq!(symbols.len(), 2);
    assert_eq!(symbols[0]["name"], "add");
    assert_eq!(symbols[0]["kind"], 12); // Function
    assert_eq!(symbols[1]["name"], "Point");
    assert_eq!(symbols[1]["kind"], 23); // Struct

    shutdown(&mut stdin, &mut stdout);
    child.wait().unwrap();
}

#[test]
#[ignore = "requires `cargo build` to produce the link binary"]
fn lsp_type_error_diagnostic() {
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    let src = "fn bad(x: i64) -> i64 {\n    return x + true;\n}\n";
    send(&mut stdin, serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": { "processId": null, "rootUri": null, "capabilities": {} }
    }));
    let _ = read(&mut stdout);
    send(&mut stdin, serde_json::json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }));
    send(&mut stdin, serde_json::json!({
        "jsonrpc": "2.0", "method": "textDocument/didOpen",
        "params": { "textDocument": { "uri": "file:///test.link", "languageId": "link", "version": 0, "text": src } }
    }));
    let diag = read(&mut stdout);
    let diags = diag["params"]["diagnostics"].as_array().unwrap();
    assert!(!diags.is_empty(), "expected type error diagnostics");
    assert_eq!(diags[0]["severity"], 1); // Error

    shutdown(&mut stdin, &mut stdout);
    child.wait().unwrap();
}
