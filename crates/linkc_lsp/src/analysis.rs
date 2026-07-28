//! Document analysis: lex → parse → semantic check.
//!
//! Captures symbols (functions, structs, enums, top-level lets) with their
//! source ranges so completion / hover / definition can reuse one analysis
//! pass per document version.

use std::collections::HashMap;
use linkc_lexer::{lex, SpannedToken, Token};
use linkc_parser::{Program, Stmt, StructField, EnumVariantDecl, TypeAnnotation};

/// A symbol declared in a document, with its location and kind.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    /// 0-based line, 0-based column (LSP convention).
    pub line: u32,
    pub col: u32,
    /// End column (exclusive). Best-effort from token spans.
    pub end_col: u32,
    /// Optional signature text for hover.
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Variable,
    Module,
}

impl SymbolKind {
    /// Maps to LSP SymbolKind integer values.
    pub fn lsp_kind(self) -> i64 {
        match self {
            SymbolKind::Function => 12,  // Function
            SymbolKind::Struct => 23,    // Struct
            SymbolKind::Enum => 10,      // Enum
            SymbolKind::Variable => 13,  // Variable
            SymbolKind::Module => 2,     // Module
        }
    }

    /// Maps to LSP CompletionItemKind integer values.
    pub fn lsp_completion_kind(self) -> i64 {
        match self {
            SymbolKind::Function => 3,   // Function
            SymbolKind::Struct => 22,    // Struct
            SymbolKind::Enum => 13,      // Enum
            SymbolKind::Variable => 6,   // Variable
            SymbolKind::Module => 9,     // Module
        }
    }
}

/// A diagnostic produced by analysis (lexer / parser / sema).
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: u32,
    pub col: u32,
    pub end_line: u32,
    pub end_col: u32,
    pub message: String,
    pub severity: DiagnosticSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

impl DiagnosticSeverity {
    pub fn lsp_value(self) -> i64 {
        match self {
            DiagnosticSeverity::Error => 1,
            DiagnosticSeverity::Warning => 2,
        }
    }
}

/// Full analysis result for a document.
pub struct Analysis {
    pub tokens: Vec<SpannedToken>,
    pub program: Option<Program>,
    pub diagnostics: Vec<Diagnostic>,
    pub symbols: Vec<Symbol>,
    /// Map from name → symbol index, for fast lookup.
    pub by_name: HashMap<String, usize>,
    /// Source text kept for range / line computations.
    pub source: String,
    /// Precomputed line start offsets for O(1) offset→line/col.
    pub line_starts: Vec<usize>,
}

impl Analysis {
    pub fn analyze(source: &str) -> Self {
        let line_starts = compute_line_starts(source);

        // 1) Lex. Lexer may panic on invalid char; catch with a thread boundary
        //    is overkill, so we pre-scan for recoverable lexing instead.
        let (tokens, lex_diags) = lex_safe(source, &line_starts);

        // 2) Parse.
        let mut program = None;
        let mut diagnostics = lex_diags;
        match ParserAdapter::parse(&tokens) {
            Ok(p) => {
                program = Some(p.clone());
                // 3) Semantic analysis.
                let sema_errs = linkc_sema::check_program(&p);
                for e in sema_errs {
                    // SemaError line/col are 1-based.
                    let line = (e.line.saturating_sub(1)) as u32;
                    let col = (e.col.saturating_sub(1)) as u32;
                    diagnostics.push(Diagnostic {
                        line,
                        col,
                        end_line: line,
                        end_col: col + 1,
                        message: e.message,
                        severity: DiagnosticSeverity::Error,
                    });
                }
            }
            Err(msg) => {
                // Parse error: try to extract a line number from the message.
                let (line, col) = extract_pos(&msg, &line_starts);
                diagnostics.push(Diagnostic {
                    line,
                    col,
                    end_line: line,
                    end_col: col + 1,
                    message: msg,
                    severity: DiagnosticSeverity::Error,
                });
            }
        }

        // 4) Collect symbols.
        let mut symbols = Vec::new();
        if let Some(prog) = &program {
            collect_symbols(prog, &tokens, &line_starts, &mut symbols);
        }

        let mut by_name = HashMap::new();
        for (i, s) in symbols.iter().enumerate() {
            by_name.entry(s.name.clone()).or_insert(i);
        }

        Analysis {
            tokens,
            program,
            diagnostics,
            symbols,
            by_name,
            source: source.to_string(),
            line_starts,
        }
    }

    /// Convert a byte offset to (0-based line, 0-based column).
    pub fn offset_to_pos(&self, offset: usize) -> (u32, u32) {
        offset_to_pos(offset, &self.line_starts)
    }

    /// Locate the token covering `offset`, if any.
    pub fn token_at(&self, offset: usize) -> Option<&SpannedToken> {
        // Skip the trailing Eof sentinel.
        self.tokens.iter().find(|t| {
            t.token != Token::Eof && t.span.start <= offset && offset < t.span.end
        })
    }

    /// Name at the given offset, if it is an identifier token.
    pub fn name_at(&self, offset: usize) -> Option<String> {
        if let Some(t) = self.token_at(offset) {
            if let Token::Ident(s) = &t.token {
                return Some(s.clone());
            }
        }
        None
    }
}

/// Compute byte offsets where each line starts (0-based).
pub fn compute_line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, b) in source.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

pub fn offset_to_pos(offset: usize, line_starts: &[usize]) -> (u32, u32) {
    // Binary search for the last line_start <= offset.
    let line_idx = match line_starts.binary_search(&offset) {
        Ok(i) => i,
        Err(i) => i.saturating_sub(1),
    };
    let line_start = line_starts.get(line_idx).copied().unwrap_or(0);
    let col = offset.saturating_sub(line_start);
    (line_idx as u32, col as u32)
}

/// Lex wrapper that converts lexer panics into diagnostics instead of crashing
/// the server. The stock lexer uses `panic!` on invalid characters / bad
/// escapes / unterminated strings; we pre-scan the source and sanitize it so
/// the real lexer always succeeds.
fn lex_safe(source: &str, line_starts: &[usize]) -> (Vec<SpannedToken>, Vec<Diagnostic>) {
    let mut diags = Vec::new();
    let bytes = source.as_bytes();
    let mut cleaned = String::with_capacity(source.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];

        // String literal: walk to the closing quote. If we hit EOF first,
        // synthesize a closing quote so the lexer doesn't panic, and emit a
        // diagnostic at the opening quote's position.
        if b == b'"' {
            let start = i;
            cleaned.push('"');
            i += 1;
            let mut closed = false;
            while i < bytes.len() {
                let c = bytes[i];
                if c == b'\\' {
                    // Escape: copy backslash + next byte (if any) verbatim.
                    cleaned.push('\\');
                    i += 1;
                    if i < bytes.len() {
                        cleaned.push(bytes[i] as char);
                        i += 1;
                    }
                    continue;
                }
                cleaned.push(c as char);
                i += 1;
                if c == b'"' {
                    closed = true;
                    break;
                }
            }
            if !closed {
                // Append a synthetic close quote to keep the lexer happy.
                cleaned.push('"');
                let (l, c) = offset_to_pos(start, line_starts);
                diags.push(Diagnostic {
                    line: l,
                    col: c,
                    end_line: l,
                    end_col: c + 1,
                    message: "unterminated string literal".to_string(),
                    severity: DiagnosticSeverity::Error,
                });
            }
            continue;
        }

        // Line comment: copy through to end of line.
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                cleaned.push(bytes[i] as char);
                i += 1;
            }
            continue;
        }

        // ASCII printable or common whitespace: lexer accepts these.
        if (b as char).is_ascii_graphic() || b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
            cleaned.push(b as char);
            i += 1;
        } else {
            // Non-ASCII or control character: replace with space and diagnose.
            let (l, c) = offset_to_pos(i, line_starts);
            diags.push(Diagnostic {
                line: l,
                col: c,
                end_line: l,
                end_col: c + 1,
                message: "unexpected character in source".to_string(),
                severity: DiagnosticSeverity::Error,
            });
            cleaned.push(' ');
            i += 1;
        }
    }

    let tokens = lex(&cleaned);
    (tokens, diags)
}

/// Adapter that runs the parser and returns a `Result`.
struct ParserAdapter;

impl ParserAdapter {
    fn parse(tokens: &[SpannedToken]) -> Result<Program, String> {
        let mut parser = linkc_parser::Parser::new(tokens.to_vec());
        parser.parse_program()
    }
}

/// Best-effort extraction of a line number from a parser error message.
fn extract_pos(msg: &str, line_starts: &[usize]) -> (u32, u32) {
    // Parser error messages currently embed "line N" without col; default to col 0.
    if let Some(idx) = msg.find("line ") {
        let rest = &msg[idx + 5..];
        let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
        if let Ok(n) = rest[..end].parse::<usize>() {
            if n > 0 {
                return ((n - 1) as u32, 0u32);
            }
        }
    }
    let _ = line_starts;
    (0, 0)
}

/// Walk top-level statements collecting declared symbols.
fn collect_symbols(
    program: &Program,
    tokens: &[SpannedToken],
    line_starts: &[usize],
    out: &mut Vec<Symbol>,
) {
    let Program::Block(stmts) = program;
    for stmt in stmts {
        collect_stmt(stmt, tokens, line_starts, out);
    }
}

fn collect_stmt(
    stmt: &Stmt,
    tokens: &[SpannedToken],
    line_starts: &[usize],
    out: &mut Vec<Symbol>,
) {
    match stmt {
        Stmt::FnDecl { name, params, return_type, is_async, .. } => {
            let (line, col) = find_ident_pos(name, tokens, line_starts);
            let detail = format_fn_signature(name, params, return_type, *is_async);
            out.push(Symbol {
                name: name.clone(),
                kind: SymbolKind::Function,
                line,
                col,
                end_col: col + name.len() as u32,
                detail: Some(detail),
            });
        }
        Stmt::StructDecl { name, fields } => {
            let (line, col) = find_ident_pos(name, tokens, line_starts);
            let detail = format_struct(name, fields);
            out.push(Symbol {
                name: name.clone(),
                kind: SymbolKind::Struct,
                line,
                col,
                end_col: col + name.len() as u32,
                detail: Some(detail),
            });
        }
        Stmt::EnumDecl { name, variants } => {
            let (line, col) = find_ident_pos(name, tokens, line_starts);
            let detail = format_enum(name, variants);
            out.push(Symbol {
                name: name.clone(),
                kind: SymbolKind::Enum,
                line,
                col,
                end_col: col + name.len() as u32,
                detail: Some(detail),
            });
        }
        Stmt::LetDecl { name, .. } => {
            let (line, col) = find_ident_pos(name, tokens, line_starts);
            out.push(Symbol {
                name: name.clone(),
                kind: SymbolKind::Variable,
                line,
                col,
                end_col: col + name.len() as u32,
                detail: None,
            });
        }
        Stmt::ModDecl { name } => {
            let (line, col) = find_ident_pos(name, tokens, line_starts);
            out.push(Symbol {
                name: name.clone(),
                kind: SymbolKind::Module,
                line,
                col,
                end_col: col + name.len() as u32,
                detail: None,
            });
        }
        // Other statements don't introduce top-level symbols.
        _ => {}
    }
}

/// Find the first occurrence of an identifier in the token stream and return
/// its (line, col). Falls back to (0, 0) if not found.
fn find_ident_pos(name: &str, tokens: &[SpannedToken], line_starts: &[usize]) -> (u32, u32) {
    for t in tokens {
        if let Token::Ident(s) = &t.token {
            if s == name {
                return offset_to_pos(t.span.start, line_starts);
            }
        }
    }
    (0, 0)
}

fn format_fn_signature(
    name: &str,
    params: &[(String, TypeAnnotation)],
    ret: &Option<TypeAnnotation>,
    is_async: bool,
) -> String {
    let prefix = if is_async { "async " } else { "" };
    let p = params.iter()
        .map(|(n, t)| format!("{}: {}", n, t))
        .collect::<Vec<_>>()
        .join(", ");
    let r = match ret {
        Some(t) => format!(" -> {}", t),
        None => String::new(),
    };
    format!("{}fn {}({}){}", prefix, name, p, r)
}

fn format_struct(name: &str, fields: &[StructField]) -> String {
    let body = fields.iter()
        .map(|f| format!("{}: {}", f.name, f.type_ann))
        .collect::<Vec<_>>()
        .join("; ");
    format!("struct {} {{ {} }}", name, body)
}

fn format_enum(name: &str, variants: &[EnumVariantDecl]) -> String {
    let body = variants.iter()
        .map(|v| {
            if v.payload.is_empty() {
                v.name.clone()
            } else {
                let tys = v.payload.iter()
                    .map(|t| format!("{}", t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", v.name, tys)
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("enum {} {{ {} }}", name, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyze_clean_program() {
        let src = "fn add(a: i64, b: i64) -> i64 {\n    return a + b;\n}\n";
        let a = Analysis::analyze(src);
        assert!(a.diagnostics.is_empty(), "unexpected diags: {:?}", a.diagnostics);
        assert_eq!(a.symbols.len(), 1);
        assert_eq!(a.symbols[0].name, "add");
        assert_eq!(a.symbols[0].kind, SymbolKind::Function);
        assert_eq!(a.symbols[0].line, 0);
    }

    #[test]
    fn analyze_parse_error() {
        let src = "fn add(a: i64 { }\n";
        let a = Analysis::analyze(src);
        assert!(!a.diagnostics.is_empty());
    }

    #[test]
    fn analyze_struct_and_enum() {
        let src = "struct Point { x: i64, y: i64 }\nenum Color { Red, Green, Blue }\n";
        let a = Analysis::analyze(src);
        assert_eq!(a.symbols.len(), 2);
        assert_eq!(a.symbols[0].kind, SymbolKind::Struct);
        assert_eq!(a.symbols[1].kind, SymbolKind::Enum);
    }

    #[test]
    fn analyze_type_error() {
        let src = "fn bad(x: i64) -> i64 {\n    return x + true;\n}\n";
        let a = Analysis::analyze(src);
        // Type checker should flag bool + int.
        assert!(a.diagnostics.iter().any(|d| d.severity == DiagnosticSeverity::Error));
    }

    #[test]
    fn offset_to_pos_basic() {
        let src = "abc\ndef\n";
        let starts = compute_line_starts(src);
        assert_eq!(offset_to_pos(0, &starts), (0, 0));
        assert_eq!(offset_to_pos(3, &starts), (0, 3));
        assert_eq!(offset_to_pos(4, &starts), (1, 0));
        assert_eq!(offset_to_pos(7, &starts), (1, 3));
    }

    #[test]
    fn name_at_finds_identifier() {
        let src = "fn foo() { return 1; }\n";
        let a = Analysis::analyze(src);
        // 'foo' starts at offset 3.
        let name = a.name_at(3);
        assert_eq!(name.as_deref(), Some("foo"));
    }

    #[test]
    fn unterminated_string_is_diagnostic_not_panic() {
        let src = "fn main() { let s = \"oops;\n }\n";
        // Should not panic.
        let a = Analysis::analyze(src);
        assert!(a.diagnostics.iter().any(|d| d.message.contains("string")));
    }
}
