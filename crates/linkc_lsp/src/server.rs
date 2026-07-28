//! LSP server: dispatches JSON-RPC requests/notifications to handlers.
//!
//! Document model: full-content sync (the simplest sync mode). Each open
//! document is re-analyzed on change; diagnostics are published after each
//! analysis. The server runs on stdio and exits cleanly on `shutdown`.

use std::collections::HashMap;
use std::io::{self, Write};

use crate::analysis::{Analysis, Diagnostic, Symbol, SymbolKind};
use crate::jsonrpc;

/// LSP Language Server entry point. Reads JSON-RPC from stdin, writes to stdout.
pub struct LanguageServer {
    documents: HashMap<String, Document>,
    shutdown_requested: bool,
}

struct Document {
    /// Current full source text.
    source: String,
    /// Cached analysis of `source`.
    analysis: Analysis,
    /// Monotonic version counter (for incremental sync, future use).
    version: i64,
}

impl LanguageServer {
    pub fn new() -> Self {
        LanguageServer {
            documents: HashMap::new(),
            shutdown_requested: false,
        }
    }

    /// Run the server until the client sends `exit` or closes stdin.
    pub fn run(&mut self) -> Result<(), String> {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let stdout = io::stdout();
        let mut writer = stdout.lock();

        loop {
            let msg = match jsonrpc::read_message(&mut reader)? {
                Some(m) => m,
                None => return Ok(()), // EOF
            };

            let method = match msg.method() {
                Some(m) => m.to_string(),
                None => continue, // responses to our notifications are not expected
            };

            match method.as_str() {
                "initialize" => {
                    let result = self.handle_initialize();
                    if let Some(id) = msg.id() {
                        let resp = jsonrpc::response(id, result);
                        jsonrpc::write_message(&mut writer, &resp)
                            .map_err(|e| format!("write error: {}", e))?;
                    }
                }
                "initialized" => {
                    // No-op: capabilities handshake complete.
                }
                "shutdown" => {
                    self.shutdown_requested = true;
                    if let Some(id) = msg.id() {
                        let resp = jsonrpc::response(id, serde_json::Value::Null);
                        jsonrpc::write_message(&mut writer, &resp)
                            .map_err(|e| format!("write error: {}", e))?;
                    }
                }
                "exit" => {
                    return Ok(());
                }
                "textDocument/didOpen" => {
                    if let Some(params) = msg.params() {
                        self.handle_did_open(params, &mut writer)?;
                    }
                }
                "textDocument/didChange" => {
                    if let Some(params) = msg.params() {
                        self.handle_did_change(params, &mut writer)?;
                    }
                }
                "textDocument/didClose" => {
                    if let Some(params) = msg.params() {
                        self.handle_did_close(params);
                    }
                }
                "textDocument/completion" => {
                    if let Some(id) = msg.id() {
                        let params = msg.params().cloned().unwrap_or(serde_json::Value::Null);
                        let result = self.handle_completion(&params);
                        let resp = jsonrpc::response(id, result);
                        jsonrpc::write_message(&mut writer, &resp)
                            .map_err(|e| format!("write error: {}", e))?;
                    }
                }
                "textDocument/hover" => {
                    if let Some(id) = msg.id() {
                        let params = msg.params().cloned().unwrap_or(serde_json::Value::Null);
                        let result = self.handle_hover(&params);
                        let resp = jsonrpc::response(id, result);
                        jsonrpc::write_message(&mut writer, &resp)
                            .map_err(|e| format!("write error: {}", e))?;
                    }
                }
                "textDocument/definition" => {
                    if let Some(id) = msg.id() {
                        let params = msg.params().cloned().unwrap_or(serde_json::Value::Null);
                        let result = self.handle_definition(&params);
                        let resp = jsonrpc::response(id, result);
                        jsonrpc::write_message(&mut writer, &resp)
                            .map_err(|e| format!("write error: {}", e))?;
                    }
                }
                "textDocument/documentSymbol" => {
                    if let Some(id) = msg.id() {
                        let params = msg.params().cloned().unwrap_or(serde_json::Value::Null);
                        let result = self.handle_document_symbol(&params);
                        let resp = jsonrpc::response(id, result);
                        jsonrpc::write_message(&mut writer, &resp)
                            .map_err(|e| format!("write error: {}", e))?;
                    }
                }
                _ => {
                    // Unknown method: respond with method-not-found if a request.
                    if let Some(id) = msg.id() {
                        let resp = jsonrpc::error_response(
                            id,
                            -32601,
                            &format!("method not found: {}", method),
                        );
                        jsonrpc::write_message(&mut writer, &resp)
                            .map_err(|e| format!("write error: {}", e))?;
                    }
                }
            }
        }
    }

    fn handle_initialize(&self) -> serde_json::Value {
        serde_json::json!({
            "capabilities": {
                "textDocumentSync": 1,  // Full content sync.
                "completionProvider": {
                    "resolveProvider": false,
                    "triggerCharacters": [".", ":"]
                },
                "hoverProvider": true,
                "definitionProvider": true,
                "documentSymbolProvider": true
            },
            "serverInfo": {
                "name": "linkc_lsp",
                "version": env!("CARGO_PKG_VERSION")
            }
        })
    }

    fn handle_did_open<W: Write>(
        &mut self,
        params: &serde_json::Value,
        writer: &mut W,
    ) -> Result<(), String> {
        let text_doc = params.get("textDocument")
            .ok_or_else(|| "missing textDocument".to_string())?;
        let uri = text_doc.get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing uri".to_string())?
            .to_string();
        let text = text_doc.get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing text".to_string())?
            .to_string();
        let version = text_doc.get("version")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let analysis = Analysis::analyze(&text);
        let diagnostics = analysis.diagnostics.clone();
        self.documents.insert(uri.clone(), Document {
            source: text,
            analysis,
            version,
        });

        self.publish_diagnostics(&uri, &diagnostics, writer)
    }

    fn handle_did_change<W: Write>(
        &mut self,
        params: &serde_json::Value,
        writer: &mut W,
    ) -> Result<(), String> {
        let text_doc = params.get("textDocument")
            .ok_or_else(|| "missing textDocument".to_string())?;
        let uri = text_doc.get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing uri".to_string())?
            .to_string();
        let version = text_doc.get("version")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // Full sync: take the last change's full text.
        let changes = params.get("contentChanges")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "missing contentChanges".to_string())?;
        let text = changes.last()
            .and_then(|c| c.get("text"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing text in change".to_string())?
            .to_string();

        let analysis = Analysis::analyze(&text);
        let diagnostics = analysis.diagnostics.clone();
        self.documents.insert(uri.clone(), Document {
            source: text,
            analysis,
            version,
        });

        self.publish_diagnostics(&uri, &diagnostics, writer)
    }

    fn handle_did_close(&mut self, params: &serde_json::Value) {
        if let Some(uri) = params.get("textDocument")
            .and_then(|v| v.get("uri"))
            .and_then(|v| v.as_str())
        {
            self.documents.remove(uri);
        }
    }

    fn publish_diagnostics<W: Write>(
        &self,
        uri: &str,
        diagnostics: &[Diagnostic],
        writer: &mut W,
    ) -> Result<(), String> {
        let items: Vec<serde_json::Value> = diagnostics.iter().map(|d| {
            serde_json::json!({
                "range": {
                    "start": { "line": d.line, "character": d.col },
                    "end":   { "line": d.end_line, "character": d.end_col }
                },
                "severity": d.severity.lsp_value(),
                "source": "link",
                "message": d.message
            })
        }).collect();

        let notification = jsonrpc::notification(
            "textDocument/publishDiagnostics",
            serde_json::json!({
                "uri": uri,
                "diagnostics": items
            }),
        );
        jsonrpc::write_message(writer, &notification)
            .map_err(|e| format!("write error: {}", e))
    }

    fn handle_completion(&self, params: &serde_json::Value) -> serde_json::Value {
        let (uri, line, character) = match parse_text_document_position(params) {
            Some(v) => v,
            None => return serde_json::Value::Null,
        };

        let doc = match self.documents.get(&uri) {
            Some(d) => d,
            None => return serde_json::Value::Null,
        };

        let items = build_completion_items(&doc.analysis, line, character);
        serde_json::json!({
            "isIncomplete": false,
            "items": items
        })
    }

    fn handle_hover(&self, params: &serde_json::Value) -> serde_json::Value {
        let (uri, line, character) = match parse_text_document_position(params) {
            Some(v) => v,
            None => return serde_json::Value::Null,
        };

        let doc = match self.documents.get(&uri) {
            Some(d) => d,
            None => return serde_json::Value::Null,
        };

        // Convert LSP (line, character) to a byte offset for token lookup.
        let offset = pos_to_offset(&doc.source, line, character);

        let content = if let Some(name) = doc.analysis.name_at(offset) {
            if let Some(&idx) = doc.analysis.by_name.get(&name) {
                let sym = &doc.analysis.symbols[idx];
                format_hover_content(sym)
            } else if let Some(builtin) = builtin_lookup(&name) {
                format_builtin_hover(builtin)
            } else {
                return serde_json::Value::Null;
            }
        } else {
            return serde_json::Value::Null;
        };

        serde_json::json!({
            "contents": { "kind": "markdown", "value": content }
        })
    }

    fn handle_definition(&self, params: &serde_json::Value) -> serde_json::Value {
        let (uri, line, character) = match parse_text_document_position(params) {
            Some(v) => v,
            None => return serde_json::Value::Null,
        };

        let doc = match self.documents.get(&uri) {
            Some(d) => d,
            None => return serde_json::Value::Null,
        };

        let offset = pos_to_offset(&doc.source, line, character);
        let name = match doc.analysis.name_at(offset) {
            Some(n) => n,
            None => return serde_json::Value::Null,
        };

        let idx = match doc.analysis.by_name.get(&name) {
            Some(&i) => i,
            None => return serde_json::Value::Null,
        };

        let sym = &doc.analysis.symbols[idx];
        serde_json::json!({
            "uri": uri,
            "range": {
                "start": { "line": sym.line, "character": sym.col },
                "end":   { "line": sym.line, "character": sym.end_col }
            }
        })
    }

    fn handle_document_symbol(&self, params: &serde_json::Value) -> serde_json::Value {
        let uri = match params.get("textDocument")
            .and_then(|v| v.get("uri"))
            .and_then(|v| v.as_str())
        {
            Some(u) => u.to_string(),
            None => return serde_json::Value::Null,
        };

        let doc = match self.documents.get(&uri) {
            Some(d) => d,
            None => return serde_json::Value::Null,
        };

        let items: Vec<serde_json::Value> = doc.analysis.symbols.iter().map(|s| {
            serde_json::json!({
                "name": s.name,
                "kind": s.kind.lsp_kind(),
                "range": {
                    "start": { "line": s.line, "character": s.col },
                    "end":   { "line": s.line, "character": s.end_col }
                },
                "selectionRange": {
                    "start": { "line": s.line, "character": s.col },
                    "end":   { "line": s.line, "character": s.end_col }
                },
                "detail": s.detail
            })
        }).collect();

        serde_json::Value::Array(items)
    }
}

/// Parse the standard LSP `{ textDocument: { uri }, position: { line, character } }`.
fn parse_text_document_position(params: &serde_json::Value) -> Option<(String, u32, u32)> {
    let uri = params.get("textDocument")?
        .get("uri")?
        .as_str()?
        .to_string();
    let pos = params.get("position")?;
    let line = pos.get("line").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let character = pos.get("character").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    Some((uri, line, character))
}

/// Convert (line, character) to a byte offset in the source. LSP positions are
/// UTF-16 code unit based, but Link source is mostly ASCII so we treat the
/// character column as a byte offset within the line for simplicity.
fn pos_to_offset(source: &str, line: u32, character: u32) -> usize {
    let mut current_line = 0u32;
    let mut line_start = 0usize;
    for (i, b) in source.bytes().enumerate() {
        if current_line == line {
            return line_start + (character as usize).min(source[line_start..].len());
        }
        if b == b'\n' {
            current_line += 1;
            line_start = i + 1;
        }
    }
    if current_line == line {
        return line_start + (character as usize).min(source[line_start..].len());
    }
    source.len()
}

/// Build completion items for a cursor position: keywords, builtins, and
/// document symbols whose name doesn't shadow keyword/builtin entries.
fn build_completion_items(analysis: &Analysis, _line: u32, _col: u32) -> Vec<serde_json::Value> {
    let mut items = Vec::new();

    // 1) Keywords.
    for kw in KEYWORDS {
        items.push(serde_json::json!({
            "label": kw,
            "kind": 14,  // Keyword
            "detail": "keyword"
        }));
    }

    // 2) Builtins (type info in detail).
    for b in BUILTIN_FUNCTIONS {
        items.push(serde_json::json!({
            "label": b.name,
            "kind": 3,  // Function
            "detail": b.signature,
            "documentation": b.doc
        }));
    }

    // 3) Built-in types.
    for ty in BUILTIN_TYPES {
        items.push(serde_json::json!({
            "label": ty,
            "kind": 21,  // Struct (closest match for type name)
            "detail": "type"
        }));
    }

    // 4) Document symbols (functions, structs, enums, variables).
    for sym in &analysis.symbols {
        items.push(serde_json::json!({
            "label": sym.name,
            "kind": sym.kind.lsp_completion_kind(),
            "detail": sym.detail
        }));
    }

    items
}

fn format_hover_content(sym: &Symbol) -> String {
    let kind_str = match sym.kind {
        SymbolKind::Function => "function",
        SymbolKind::Struct => "struct",
        SymbolKind::Enum => "enum",
        SymbolKind::Variable => "variable",
        SymbolKind::Module => "module",
    };
    let detail = sym.detail.as_deref().unwrap_or(sym.name.as_str());
    format!("```link\n{}\n```\n\n_{}_ defined at line {}_", detail, kind_str, sym.line + 1)
}

fn format_builtin_hover(b: &Builtin) -> String {
    format!("```link\n{}\n```\n\n{}", b.signature, b.doc)
}

/// Description of a Link built-in function.
struct Builtin {
    name: &'static str,
    signature: &'static str,
    doc: &'static str,
}

const BUILTIN_FUNCTIONS: &[Builtin] = &[
    Builtin { name: "println", signature: "fn println(...args: any) -> ()", doc: "Print arguments to stdout with a trailing newline. Supports `{}` placeholders in the first string argument." },
    Builtin { name: "print",   signature: "fn print(...args: any) -> ()",   doc: "Print arguments to stdout without a trailing newline." },
    Builtin { name: "len",     signature: "fn len(x: str | list) -> i64",   doc: "Return the length of a string or list." },
    Builtin { name: "sleep",   signature: "fn sleep(ms: i64) -> ()",        doc: "Suspend the current task for `ms` milliseconds." },
    // Stdlib: math
    Builtin { name: "abs",     signature: "fn abs(x: i64 | f64) -> i64 | f64", doc: "Absolute value." },
    Builtin { name: "min",     signature: "fn min(a: T, b: T) -> T",        doc: "Return the smaller of two values." },
    Builtin { name: "max",     signature: "fn max(a: T, b: T) -> T",        doc: "Return the larger of two values." },
    Builtin { name: "sqrt",    signature: "fn sqrt(x: f64) -> f64",         doc: "Square root." },
    Builtin { name: "pow",     signature: "fn pow(base: f64, exp: f64) -> f64", doc: "`base` raised to `exp`." },
    // Stdlib: strings
    Builtin { name: "str_upper",  signature: "fn str_upper(s: str) -> str", doc: "Uppercase a string." },
    Builtin { name: "str_lower",  signature: "fn str_lower(s: str) -> str", doc: "Lowercase a string." },
    Builtin { name: "str_trim",   signature: "fn str_trim(s: str) -> str",  doc: "Strip leading/trailing whitespace." },
    Builtin { name: "str_split",  signature: "fn str_split(s: str, sep: str) -> list<str>", doc: "Split `s` by `sep`." },
    Builtin { name: "str_contains", signature: "fn str_contains(s: str, sub: str) -> bool", doc: "Test substring presence." },
    Builtin { name: "str_len",    signature: "fn str_len(s: str) -> i64",   doc: "Length in bytes of a string." },
    // Stdlib: lists
    Builtin { name: "list_push",  signature: "fn list_push(lst: list<T>, v: T) -> ()", doc: "Append an element to a list." },
    Builtin { name: "list_pop",   signature: "fn list_pop(lst: list<T>) -> T", doc: "Remove and return the last element." },
    Builtin { name: "list_sort",  signature: "fn list_sort(lst: list<T>) -> list<T>", doc: "Return a sorted copy." },
    Builtin { name: "list_reverse", signature: "fn list_reverse(lst: list<T>) -> list<T>", doc: "Return a reversed copy." },
    Builtin { name: "list_len",   signature: "fn list_len(lst: list<T>) -> i64", doc: "Length of a list." },
    // Stdlib: IO
    Builtin { name: "file_read",  signature: "fn file_read(path: str) -> str", doc: "Read a file as UTF-8 text." },
    Builtin { name: "file_write", signature: "fn file_write(path: str, content: str) -> ()", doc: "Write text to a file." },
    // Stdlib: time / conversion
    Builtin { name: "time_now",   signature: "fn time_now() -> i64", doc: "Unix epoch milliseconds." },
    Builtin { name: "int",        signature: "fn int(x: any) -> i64", doc: "Convert to integer." },
    Builtin { name: "float",      signature: "fn float(x: any) -> f64", doc: "Convert to float." },
    Builtin { name: "bool",       signature: "fn bool(x: any) -> bool", doc: "Convert to boolean." },
    Builtin { name: "str",        signature: "fn str(x: any) -> str", doc: "Convert to string." },
    // Stream operators
    Builtin { name: "stream",     signature: "fn stream<T>(items: list<T>) -> stream<T>", doc: "Construct a stream from a list." },
    Builtin { name: "map",        signature: "fn map<T, U>(s: stream<T>, f: fn(T) -> U) -> stream<U>", doc: "Transform stream elements." },
    Builtin { name: "filter",     signature: "fn filter<T>(s: stream<T>, pred: fn(T) -> bool) -> stream<T>", doc: "Keep elements matching a predicate." },
    Builtin { name: "for_each",   signature: "fn for_each<T>(s: stream<T>, f: fn(T)) -> ()", doc: "Consume each stream element." },
    Builtin { name: "collect",    signature: "fn collect<T>(s: stream<T>) -> list<T>", doc: "Materialize a stream into a list." },
];

const BUILTIN_TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64",
    "u8", "u16", "u32", "u64", "usize",
    "f32", "f64",
    "bool", "str", "string", "char", "void",
];

const KEYWORDS: &[&str] = &[
    "fn", "let", "return", "if", "else", "match", "for", "while", "loop", "in",
    "break", "continue",
    "extern", "export", "async", "await",
    "struct", "enum", "impl", "trait",
    "use", "mod", "pub", "mut",
    "stream", "flow", "pipeline", "source", "sample",
    "true", "false", "none", "some", "ok", "err", "as",
];

fn builtin_lookup(name: &str) -> Option<&'static Builtin> {
    BUILTIN_FUNCTIONS.iter().find(|b| b.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pos_to_offset_basic() {
        let src = "abc\ndef\n";
        assert_eq!(pos_to_offset(src, 0, 0), 0);
        assert_eq!(pos_to_offset(src, 0, 2), 2);
        assert_eq!(pos_to_offset(src, 1, 0), 4);
        assert_eq!(pos_to_offset(src, 1, 3), 7);
    }

    #[test]
    fn completion_includes_keywords_and_builtins() {
        let analysis = Analysis::analyze("fn main() { return 1; }\n");
        let items = build_completion_items(&analysis, 0, 0);
        let labels: Vec<&str> = items.iter()
            .filter_map(|v| v.get("label").and_then(|l| l.as_str()))
            .collect();
        assert!(labels.contains(&"fn"));
        assert!(labels.contains(&"println"));
        assert!(labels.contains(&"i64"));
        assert!(labels.contains(&"main"));
    }

    #[test]
    fn document_symbols_returned() {
        let mut server = LanguageServer::new();
        // Simulate open by inserting directly.
        let src = "fn foo() {}\nstruct Bar { x: i64 }\n";
        let analysis = Analysis::analyze(src);
        server.documents.insert("file:///test.link".to_string(), Document {
            source: src.to_string(),
            analysis,
            version: 0,
        });
        let params = serde_json::json!({
            "textDocument": { "uri": "file:///test.link" }
        });
        let result = server.handle_document_symbol(&params);
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], "foo");
        assert_eq!(arr[1]["name"], "Bar");
    }

    #[test]
    fn definition_finds_function() {
        let mut server = LanguageServer::new();
        let src = "fn add(a: i64, b: i64) -> i64 {\n    return a + b;\n}\n";
        let analysis = Analysis::analyze(src);
        server.documents.insert("file:///test.link".to_string(), Document {
            source: src.to_string(),
            analysis,
            version: 0,
        });
        // 'add' starts at offset 3 (line 0, char 3).
        let params = serde_json::json!({
            "textDocument": { "uri": "file:///test.link" },
            "position": { "line": 0, "character": 4 }
        });
        let result = server.handle_definition(&params);
        assert_eq!(result["uri"], "file:///test.link");
        assert_eq!(result["range"]["start"]["line"], 0);
        assert_eq!(result["range"]["start"]["character"], 3);
    }

    #[test]
    fn hover_returns_signature() {
        let mut server = LanguageServer::new();
        let src = "fn add(a: i64, b: i64) -> i64 {\n    return a + b;\n}\n";
        let analysis = Analysis::analyze(src);
        server.documents.insert("file:///test.link".to_string(), Document {
            source: src.to_string(),
            analysis,
            version: 0,
        });
        let params = serde_json::json!({
            "textDocument": { "uri": "file:///test.link" },
            "position": { "line": 0, "character": 4 }
        });
        let result = server.handle_hover(&params);
        let content = result["contents"]["value"].as_str().unwrap();
        assert!(content.contains("fn add(a: i64, b: i64) -> i64"));
    }

    #[test]
    fn hover_for_builtin() {
        let mut server = LanguageServer::new();
        let src = "fn main() { println(\"hi\"); }\n";
        let analysis = Analysis::analyze(src);
        server.documents.insert("file:///test.link".to_string(), Document {
            source: src.to_string(),
            analysis,
            version: 0,
        });
        // 'println' starts at offset 11 (line 0, char 11).
        let params = serde_json::json!({
            "textDocument": { "uri": "file:///test.link" },
            "position": { "line": 0, "character": 12 }
        });
        let result = server.handle_hover(&params);
        let content = result["contents"]["value"].as_str().unwrap();
        assert!(content.contains("println"));
    }

    #[test]
    fn initialize_advertises_capabilities() {
        let server = LanguageServer::new();
        let result = server.handle_initialize();
        assert_eq!(result["capabilities"]["textDocumentSync"], 1);
        assert_eq!(result["capabilities"]["hoverProvider"], true);
        assert_eq!(result["capabilities"]["definitionProvider"], true);
        assert_eq!(result["capabilities"]["documentSymbolProvider"], true);
    }
}
