//! Minimal JSON-RPC 2.0 transport for LSP over stdio.
//!
//! LSP messages are framed with a `Content-Length` header:
//! ```text
//! Content-Length: N\r\n
//! \r\n
//! <N bytes of JSON>
//! ```
//!
//! This module handles reading/writing frames from stdio. It is intentionally
//! dependency-free aside from `serde_json` to keep the LSP crate lightweight.

use std::io::{self, BufRead, Write};
use serde_json::Value;

/// A parsed JSON-RPC message (request, response, or notification).
pub struct Message {
    pub json: Value,
}

impl Message {
    pub fn parse(s: &str) -> Result<Self, String> {
        let json: Value = serde_json::from_str(s)
            .map_err(|e| format!("invalid JSON: {}", e))?;
        Ok(Message { json })
    }

    /// JSON-RPC method name, if this is a request or notification.
    pub fn method(&self) -> Option<&str> {
        self.json.get("method").and_then(|v| v.as_str())
    }

    /// Request id, if this is a request (has both `id` and `method`).
    pub fn id(&self) -> Option<Value> {
        if self.json.get("method").is_some() {
            self.json.get("id").cloned()
        } else {
            None
        }
    }

    /// Params object (object or array), or `None` if absent.
    pub fn params(&self) -> Option<&Value> {
        self.json.get("params")
    }
}

/// Reads one framed message from the given buffered reader.
///
/// Returns `Ok(None)` on clean EOF (client closed stdin).
pub fn read_message<R: BufRead>(reader: &mut R) -> Result<Option<Message>, String> {
    let mut content_length: Option<usize> = None;

    // Parse headers until blank line.
    loop {
        let mut header = String::new();
        let n = reader.read_line(&mut header)
            .map_err(|e| format!("read error: {}", e))?;
        if n == 0 {
            // EOF before any header byte: clean close.
            return Ok(None);
        }
        // Trim trailing \r\n.
        let trimmed = header.trim_end_matches(|c| c == '\r' || c == '\n');
        if trimmed.is_empty() {
            // Blank line: end of headers.
            break;
        }
        if let Some(rest) = trimmed.strip_prefix("Content-Length:") {
            let value = rest.trim();
            content_length = value.parse::<usize>().ok();
        }
        // Other headers (Content-Type, etc.) are ignored.
    }

    let len = content_length.ok_or_else(|| "missing Content-Length header".to_string())?;

    // Read exactly `len` bytes of JSON body.
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body)
        .map_err(|e| format!("body read error: {}", e))?;
    let text = String::from_utf8(body)
        .map_err(|e| format!("body is not utf-8: {}", e))?;
    Message::parse(&text).map(Some)
}

/// Writes a framed message to the given writer.
pub fn write_message<W: Write>(writer: &mut W, json: &Value) -> io::Result<()> {
    let body = serde_json::to_string(json)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    writer.flush()
}

/// Builds a JSON-RPC response.
pub fn response(id: Value, result: Value) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    })
}

/// Builds a JSON-RPC error response.
pub fn error_response(id: Value, code: i32, message: &str) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message },
    })
}

/// Builds a JSON-RPC notification (no id).
pub fn notification(method: &str, params: Value) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn round_trip_message() {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "test",
            "params": { "x": 1 },
            "id": 42,
        });
        let mut buf = Vec::new();
        write_message(&mut buf, &payload).unwrap();

        let mut reader = Cursor::new(buf);
        let mut br = io::BufReader::new(&mut reader);
        let msg = read_message(&mut br).unwrap().unwrap();
        assert_eq!(msg.method(), Some("test"));
        assert_eq!(msg.id(), Some(Value::from(42)));
    }

    #[test]
    fn eof_returns_none() {
        let mut br = io::BufReader::new(Cursor::new(Vec::<u8>::new()));
        let msg = read_message(&mut br).unwrap();
        assert!(msg.is_none());
    }

    #[test]
    fn response_helpers() {
        let r = response(Value::from(1), serde_json::json!({"ok": true}));
        assert_eq!(r["result"]["ok"], true);
        let e = error_response(Value::from(2), -32601, "not found");
        assert_eq!(e["error"]["code"], -32601);
        let n = notification("evt", serde_json::json!({"v": 1}));
        assert!(n.get("id").is_none());
        assert_eq!(n["method"], "evt");
    }
}
