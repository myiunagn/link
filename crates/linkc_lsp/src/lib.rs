//! Link Language Server Protocol implementation.
//!
//! Provides language intelligence for `.link` files via JSON-RPC over stdio:
//! - Diagnostics (lexer / parser / type checker)
//! - Completion (keywords, builtins, document symbols)
//! - Hover (type information)
//! - Go-to-definition (function / variable location)
//! - Document symbols (outline)
//!
//! Integration: `link lsp` starts the server; editors connect via stdio.

pub mod analysis;
pub mod jsonrpc;
pub mod server;

pub use server::LanguageServer;
