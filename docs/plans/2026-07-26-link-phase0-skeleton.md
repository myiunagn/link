# Link Phase 0 Implementation Plan - Skeleton Period

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a working toy programming language that can parse and execute basic Link code (functions, expressions, control flow) via a tree-walking interpreter. The end result: `link run fib.link` outputs `55` for a fibonacci function.

**Architecture:** Rust workspace with 4 crates: lexer (tokenizer) → parser (recursive descent → AST) → interpreter (tree-walking evaluator) → CLI (binary entry point). Each crate has its own tests.

**Tech Stack:** Rust edition 2021, cargo workspace, standard library only (no external dependencies for Phase 0).

**Prerequisites:** Rust toolchain installed (https://rustup.rs/). Run `rustc --version` to verify.

---

## File Structure

```
d:\link\
├── Cargo.toml                    # Workspace root
├── .gitignore
├── README.md
├── crates/
│   ├── linkc_lexer/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs            # Lexer implementation + Token enum + Span
│   ├── linkc_parser/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs            # Parser + AST node definitions + Pratt parsing
│   ├── linkc_interpreter/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs            # Interpreter + Environment + Value types
│   └── linkc_cli/
│       ├── Cargo.toml
│       └── src/
│           └── main.rs          # CLI entry: link run <file>
└── tests/
    ├── fixtures/
    │   └── fib.link              # Sample: fibonacci function
    └── integration_test.rs       # End-to-end tests
```

---

### Task 1: Initialize Workspace Project

**Files:**
- Create: `d:\link\Cargo.toml`
- Create: `d:\link\.gitignore`
- Create: `d:\link\README.md`

- [ ] **Step 1: Create workspace Cargo.toml**

```toml
[workspace]
members = [
    "crates/linkc_lexer",
    "crates/linkc_parser",
    "crates/linkc_interpreter",
    "crates/linkc_cli",
]
resolver = "2"

[workspace.package]
edition = "2021"
version = "0.1.0"
license = "MIT"
```

- [ ] **Step 2: Create .gitignore**

```
/target
Cargo.lock
*.swp
.DS_Store
```

- [ ] **Step 3: Create README.md**

```markdown
# Link

A language for connecting everything — from IoT devices to game servers to multi-language glue.

## Phase 0: Skeleton

Working toy language with lexer, parser, and interpreter.

## Build

```bash
cargo build
cargo run -p linkc_cli -- run tests/fixtures/fib.link
```
```

- [ ] **Step 4: Create crate directory structure**

```bash
mkdir -p crates/linkc_lexer/src
mkdir -p crates/linkc_parser/src
mkdir -p crates/linkc_interpreter/src
mkdir -p crates/linkc_cli/src
mkdir -p tests/fixtures
```

- [ ] **Step 5: Create each crate's Cargo.toml**

**crates/linkc_lexer/Cargo.toml:**
```toml
[package]
name = "linkc_lexer"
edition.workspace = true
version.workspace = true

[dependencies]
```

**crates/linkc_parser/Cargo.toml:**
```toml
[package]
name = "linkc_parser"
edition.workspace = true
version.workspace = true

[dependencies]
linkc_lexer = { path = "../linkc_lexer" }
```

**crates/linkc_interpreter/Cargo.toml:**
```toml
[package]
name = "linkc_interpreter"
edition.workspace = true
version.workspace = true

[dependencies]
linkc_lexer = { path = "../linkc_lexer" }
linkc_parser = { path = "../linkc_parser" }
```

**crates/linkc_cli/Cargo.toml:**
```toml
[package]
name = "linkc_cli"
edition.workspace = true
version.workspace = true

[[bin]]
name = "link"
path = "src/main.rs"

[dependencies]
linkc_lexer = { path = "../linkc_lexer" }
linkc_parser = { path = "../linkc_parser" }
linkc_interpreter = { path = "../linkc_interpreter" }
```

- [ ] **Step 6: Create placeholder source files**

**crates/linkc_lexer/src/lib.rs:**
```rust
pub fn placeholder() {}
```

**crates/linkc_parser/src/lib.rs:**
```rust
pub fn placeholder() {}
```

**crates/linkc_interpreter/src/lib.rs:**
```rust
pub fn placeholder() {}
```

**crates/linkc_cli/src/main.rs:**
```rust
fn main() {
    println!("Link CLI placeholder");
}
```

- [ ] **Step 7: Verify workspace builds**

Run: `cd d:\link && cargo build`
Expected: Compiles successfully with no errors

- [ ] **Step 8: Commit**

```bash
cd d:\link
git init
git add .
git commit -m "feat: initialize workspace structure"
```

---

### Task 2: Implement Lexer - Token Types

**Files:**
- Modify: `crates/linkc_lexer/src/lib.rs`

- [ ] **Step 1: Write tests for token Display**

Add to `crates/linkc_lexer/src/lib.rs` at bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_display() {
        assert_eq!(format!("{}", Token::Int(42)), "42");
        assert_eq!(format!("{}", Token::Ident("foo".to_string())), "foo");
        assert_eq!(format!("{}", Token::Plus), "+");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p linkc_lexer`
Expected: FAIL - `Token` doesn't exist yet

- [ ] **Step 3: Implement Token enum with Display**

Replace entire `crates/linkc_lexer/src/lib.rs` content (before the test module) with:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Ident(String),
    Fn, Let, Return, If, Else, Match, For, While, Loop, In,
    True, False, None, Some, Ok, Err, As,
    Plus, Minus, Star, Slash, Percent,
    Assign, Eq, NotEq, Lt, Gt, LtEq, GtEq,
    And, Or, Not, Ampersand, Pipe, Arrow, FatArrow,
    Dot, Colon, Semicolon, Comma, Underscore,
    LeftParen, RightParen, LeftBrace, RightBrace, LeftBracket, RightBracket,
    Eof,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Int(n) => write!(f, "{}", n),
            Token::Float(n) => write!(f, "{}", n),
            Token::Str(s) => write!(f, "\"{}\"", s),
            Token::Bool(b) => write!(f, "{}", b),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Fn => write!(f, "fn"),
            Token::Let => write!(f, "let"),
            Token::Return => write!(f, "return"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Match => write!(f, "match"),
            Token::For => write!(f, "for"),
            Token::While => write!(f, "while"),
            Token::Loop => write!(f, "loop"),
            Token::In => write!(f, "in"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::None => write!(f, "none"),
            Token::Some => write!(f, "some"),
            Token::Ok => write!(f, "ok"),
            Token::Err => write!(f, "err"),
            Token::As => write!(f, "as"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Assign => write!(f, "="),
            Token::Eq => write!(f, "=="),
            Token::NotEq => write!(f, "!="),
            Token::Lt => write!(f, "<"),
            Token::Gt => write!(f, ">"),
            Token::LtEq => write!(f, "<="),
            Token::GtEq => write!(f, ">="),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::Not => write!(f, "!"),
            Token::Ampersand => write!(f, "&"),
            Token::Pipe => write!(f, "|"),
            Token::Arrow => write!(f, "->"),
            Token::FatArrow => write!(f, "=>"),
            Token::Dot => write!(f, "."),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::Underscore => write!(f, "_"),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::Eof => write!(f, "<eof>"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}
```

- [ ] **Step 4: Run test**

Run: `cargo test -p linkc_lexer`
Expected: 1 test passes

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "feat: add Token enum and Span types"
```

---

### Task 3: Implement Lexer - Scan Method

**Files:**
- Modify: `crates/linkc_lexer/src/lib.rs`

- [ ] **Step 1: Add failing lexer tests**

Add to the test module in `crates/linkc_lexer/src/lib.rs`:

```rust
#[test]
fn test_lex_integer() {
    let tokens = lex("42");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::Int(42));
    assert_eq!(tokens[1].token, Token::Eof);
}

#[test]
fn test_lex_float() {
    let tokens = lex("3.14");
    assert_eq!(tokens[0].token, Token::Float(3.14));
}

#[test]
fn test_lex_string() {
    let tokens = lex(r#""hello""#);
    assert_eq!(tokens[0].token, Token::Str("hello".to_string()));
}

#[test]
fn test_lex_identifier() {
    let tokens = lex("foo");
    assert_eq!(tokens[0].token, Token::Ident("foo".to_string()));
}

#[test]
fn test_lex_keyword_fn() {
    let tokens = lex("fn");
    assert_eq!(tokens[0].token, Token::Fn);
}

#[test]
fn test_lex_operators() {
    let tokens = lex("+ - * / % = == != < > <= >= && ||");
    assert_eq!(tokens[0].token, Token::Plus);
    assert_eq!(tokens[1].token, Token::Minus);
    assert_eq!(tokens[2].token, Token::Star);
    assert_eq!(tokens[3].token, Token::Slash);
    assert_eq!(tokens[4].token, Token::Percent);
    assert_eq!(tokens[5].token, Token::Assign);
    assert_eq!(tokens[6].token, Token::Eq);
    assert_eq!(tokens[7].token, Token::NotEq);
    assert_eq!(tokens[8].token, Token::Lt);
    assert_eq!(tokens[9].token, Token::Gt);
    assert_eq!(tokens[10].token, Token::LtEq);
    assert_eq!(tokens[11].token, Token::GtEq);
    assert_eq!(tokens[12].token, Token::And);
    assert_eq!(tokens[13].token, Token::Or);
}

#[test]
fn test_lex_delimiters() {
    let tokens = lex("( ) { } [ ] ; , : -> => . |");
    assert_eq!(tokens[0].token, Token::LeftParen);
    assert_eq!(tokens[1].token, Token::RightParen);
    assert_eq!(tokens[2].token, Token::LeftBrace);
    assert_eq!(tokens[3].token, Token::RightBrace);
    assert_eq!(tokens[4].token, Token::LeftBracket);
    assert_eq!(tokens[5].token, Token::RightBracket);
    assert_eq!(tokens[6].token, Token::Semicolon);
    assert_eq!(tokens[7].token, Token::Comma);
    assert_eq!(tokens[8].token, Token::Colon);
    assert_eq!(tokens[9].token, Token::Arrow);
    assert_eq!(tokens[10].token, Token::FatArrow);
    assert_eq!(tokens[11].token, Token::Dot);
    assert_eq!(tokens[12].token, Token::Pipe);
}

#[test]
fn test_lex_bool_and_none() {
    let tokens = lex("true false none");
    assert_eq!(tokens[0].token, Token::Bool(true));
    assert_eq!(tokens[1].token, Token::Bool(false));
    assert_eq!(tokens[2].token, Token::None);
}

#[test]
fn test_lex_skips_whitespace_and_comments() {
    let tokens = lex("  42  // comment\n  100");
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0].token, Token::Int(42));
    assert_eq!(tokens[1].token, Token::Int(100));
}

#[test]
fn test_lex_complex_function() {
    let tokens = lex("fn add(a: i32, b: i32) -> i32 { return a + b; }");
    let idents: Vec<Token> = tokens.iter().map(|t| t.token.clone()).collect();
    assert!(idents.contains(&Token::Fn));
    assert!(idents.contains(&Token::Ident("add".to_string())));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p linkc_lexer`
Expected: FAIL - `lex` function doesn't exist

- [ ] **Step 3: Implement the Lexer**

Add before the test module in `crates/linkc_lexer/src/lib.rs`:

```rust
pub struct Lexer {
    source: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self { source: source.chars().collect(), pos: 0 }
    }

    pub fn tokenize(&mut self) -> Vec<SpannedToken> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.pos >= self.source.len() {
                tokens.push(SpannedToken { token: Token::Eof, span: Span::new(self.pos, self.pos) });
                break;
            }
            let start = self.pos;
            let token = self.next_token();
            let end = self.pos;
            tokens.push(SpannedToken { token, span: Span::new(start, end) });
        }
        tokens
    }

    fn peek(&self) -> Option<char> { self.source.get(self.pos).copied() }
    fn peek_next(&self) -> Option<char> { self.source.get(self.pos + 1).copied() }
    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.pos).copied()?;
        self.pos += 1;
        Some(ch)
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() { self.advance(); }
            else if ch == '/' && self.peek_next() == Some('/') {
                while let Some(c) = self.peek() { if c == '\n' { break; } self.advance(); }
            } else { break; }
        }
    }

    fn next_token(&mut self) -> Token {
        let ch = self.advance().unwrap();
        match ch {
            '+' => Token::Plus,
            '-' => { if self.peek() == Some('>') { self.advance(); Token::Arrow } else { Token::Minus } }
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '=' => {
                if self.peek() == Some('=') { self.advance(); Token::Eq }
                else if self.peek() == Some('>') { self.advance(); Token::FatArrow }
                else { Token::Assign }
            }
            '!' => { if self.peek() == Some('=') { self.advance(); Token::NotEq } else { Token::Not } }
            '<' => { if self.peek() == Some('=') { self.advance(); Token::LtEq } else { Token::Lt } }
            '>' => { if self.peek() == Some('=') { self.advance(); Token::GtEq } else { Token::Gt } }
            '&' => { if self.peek() == Some('&') { self.advance(); Token::And } else { Token::Ampersand } }
            '|' => { if self.peek() == Some('|') { self.advance(); Token::Or } else { Token::Pipe } }
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            ';' => Token::Semicolon,
            ',' => Token::Comma,
            ':' => Token::Colon,
            '.' => Token::Dot,
            '_' => {
                if self.peek().map(|c| c.is_alphanumeric()) == Some(true) {
                    self.lex_ident_or_keyword("_".to_string())
                } else { Token::Underscore }
            }
            '"' => self.lex_string(),
            c if c.is_ascii_digit() => self.lex_number(c),
            c if c.is_ascii_alphabetic() => self.lex_ident_or_keyword(c.to_string()),
            c => panic!("Unexpected character '{}' at position {}", c, self.pos - 1),
        }
    }

    fn lex_string(&mut self) -> Token {
        let mut s = String::new();
        while let Some(ch) = self.advance() {
            if ch == '"' { return Token::Str(s); }
            if ch == '\\' {
                match self.advance() {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('\\') => s.push('\\'),
                    Some('"') => s.push('"'),
                    Some(c) => panic!("Unknown escape sequence \\{}", c),
                    None => panic!("Unterminated string escape"),
                }
            } else { s.push(ch); }
        }
        panic!("Unterminated string literal")
    }

    fn lex_number(&mut self, first: char) -> Token {
        let mut num_str = String::new();
        num_str.push(first);
        let mut is_float = false;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() { num_str.push(ch); self.advance(); }
            else if ch == '.' && !is_float { is_float = true; num_str.push(ch); self.advance(); }
            else { break; }
        }
        if is_float { Token::Float(num_str.parse().expect("Invalid float")) }
        else { Token::Int(num_str.parse().expect("Invalid integer")) }
    }

    fn lex_ident_or_keyword(&mut self, start: String) -> Token {
        let mut ident = start;
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' { ident.push(ch); self.advance(); }
            else { break; }
        }
        match ident.as_str() {
            "fn" => Token::Fn,
            "let" => Token::Let,
            "return" => Token::Return,
            "if" => Token::If,
            "else" => Token::Else,
            "match" => Token::Match,
            "for" => Token::For,
            "while" => Token::While,
            "loop" => Token::Loop,
            "in" => Token::In,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "none" => Token::None,
            "some" => Token::Some,
            "ok" => Token::Ok,
            "err" => Token::Err,
            "as" => Token::As,
            _ => Token::Ident(ident),
        }
    }
}

pub fn lex(source: &str) -> Vec<SpannedToken> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}
```

- [ ] **Step 4: Run all lexer tests**

Run: `cargo test -p linkc_lexer`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "feat: implement lexer with full token scanning"
```

---

### Task 4: Implement AST Node Definitions

**Files:**
- Modify: `crates/linkc_parser/src/lib.rs`

- [ ] **Step 1: Write tests for AST**

Add to `crates/linkc_parser/src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_node_display() {
        let node = Expr::Int(42);
        assert_eq!(format!("{}", node), "42");
    }

    #[test]
    fn test_binary_expr_display() {
        let node = Expr::Binary {
            op: BinOp::Add,
            left: Box::new(Expr::Int(1)),
            right: Box::new(Expr::Int(2)),
        };
        assert_eq!(format!("{}", node), "(1 + 2)");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p linkc_parser`
Expected: FAIL - `Expr` doesn't exist

- [ ] **Step 3: Implement AST types**

Replace entire `crates/linkc_parser/src/lib.rs` content with:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Program { Block(Vec<Stmt>) }

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    FnDecl { name: String, params: Vec<(String, TypeAnnotation)>, return_type: Option<TypeAnnotation>, body: Block },
    LetDecl { name: String, type_annotation: Option<TypeAnnotation>, value: Option<Expr> },
    Expr(Expr),
    Return(Option<Expr>),
    If { condition: Expr, then_branch: Block, else_branch: Option<Block> },
    While { condition: Expr, body: Block },
    For { var_name: String, start: Expr, end: Expr, body: Block },
    Loop(Block),
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block { pub stmts: Vec<Stmt> }

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    None,
    Ident(String),
    Binary { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Unary { op: UnaryOp, operand: Box<Expr> },
    Call { callee: String, args: Vec<Expr> },
    IfExpr { condition: Box<Expr>, then_value: Box<Expr>, else_value: Box<Expr> },
    BlockExpr(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp { Add, Sub, Mul, Div, Mod, Eq, Neq, Lt, Gt, LtEq, GtEq, And, Or }

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp { Neg, Not }

#[derive(Debug, Clone, PartialEq)]
pub enum TypeAnnotation { I32, I64, F32, F64, Bool, Str, Unit, Named(String) }

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Int(n) => write!(f, "{}", n),
            Expr::Float(n) => write!(f, "{}", n),
            Expr::Str(s) => write!(f, "\"{}\"", s),
            Expr::Bool(b) => write!(f, "{}", b),
            Expr::None => write!(f, "none"),
            Expr::Ident(s) => write!(f, "{}", s),
            Expr::Binary { op, left, right } => {
                let op_str = match op {
                    BinOp::Add => "+", BinOp::Sub => "-", BinOp::Mul => "*",
                    BinOp::Div => "/", BinOp::Mod => "%", BinOp::Eq => "==",
                    BinOp::Neq => "!=", BinOp::Lt => "<", BinOp::Gt => ">",
                    BinOp::LtEq => "<=", BinOp::GtEq => ">=",
                    BinOp::And => "&&", BinOp::Or => "||",
                };
                write!(f, "({} {} {})", left, op_str, right)
            }
            Expr::Unary { op, operand } => {
                let op_str = match op { UnaryOp::Neg => "-", UnaryOp::Not => "!" };
                write!(f, "{}{}", op_str, operand)
            }
            Expr::Call { callee, args } => {
                write!(f, "{}(", callee)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expr::IfExpr { condition, then_value, else_value } => {
                write!(f, "if {} {} else {}", condition, then_value, else_value)
            }
            Expr::BlockExpr(_) => write!(f, "{{ ... }}"),
        }
    }
}

impl std::fmt::Display for TypeAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeAnnotation::I32 => write!(f, "i32"),
            TypeAnnotation::I64 => write!(f, "i64"),
            TypeAnnotation::F32 => write!(f, "f32"),
            TypeAnnotation::F64 => write!(f, "f64"),
            TypeAnnotation::Bool => write!(f, "bool"),
            TypeAnnotation::Str => write!(f, "str"),
            TypeAnnotation::Unit => write!(f, "()"),
            TypeAnnotation::Named(s) => write!(f, "{}", s),
        }
    }
}
```

- [ ] **Step 4: Run AST tests**

Run: `cargo test -p linkc_parser`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "feat: add AST node definitions for Link language"
```

---

### Task 5: Implement Parser - Expressions and Statements

**Files:**
- Modify: `crates/linkc_parser/src/lib.rs`

- [ ] **Step 1: Add tests for parsing**

Add to the test module in `crates/linkc_parser/src/lib.rs`:

```rust
use linkc_lexer::lex;

fn parse(source: &str) -> Result<Program, String> {
    let tokens = lex(source);
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

#[test]
fn test_parse_integer() {
    let program = parse("42").unwrap();
    if let Program::Block(stmts) = program {
        assert!(matches!(&stmts[0], Stmt::Expr(Expr::Int(42))));
        return;
    }
    panic!("Expected integer");
}

#[test]
fn test_parse_binary_add() {
    let program = parse("1 + 2").unwrap();
    if let Program::Block(stmts) = program {
        if let Stmt::Expr(Expr::Binary { op, .. }) = &stmts[0] {
            assert!(matches!(op, BinOp::Add));
            return;
        }
    }
    panic!("Expected binary expression");
}

#[test]
fn test_parse_precedence() {
    let program = parse("1 + 2 * 3").unwrap();
    if let Program::Block(stmts) = program {
        if let Stmt::Expr(Expr::Binary { op, right, .. }) = &stmts[0] {
            assert!(matches!(op, BinOp::Add));
            if let Expr::Binary { op: inner_op, .. } = right.as_ref() {
                assert!(matches!(inner_op, BinOp::Mul));
                return;
            }
        }
    }
    panic!("Expected (1 + (2 * 3))");
}

#[test]
fn test_parse_fn_decl() {
    let program = parse("fn add(a: i32, b: i32) -> i32 { return a + b; }").unwrap();
    if let Program::Block(stmts) = program {
        if let Stmt::FnDecl { name, params, return_type, .. } = &stmts[0] {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert!(matches!(return_type, Some(TypeAnnotation::I32)));
            return;
        }
    }
    panic!("Expected function declaration");
}

#[test]
fn test_parse_let_decl() {
    let program = parse("let x: i32 = 42").unwrap();
    if let Program::Block(stmts) = program {
        if let Stmt::LetDecl { name, type_annotation, value } = &stmts[0] {
            assert_eq!(name, "x");
            assert!(matches!(type_annotation, Some(TypeAnnotation::I32)));
            assert!(value.is_some());
            return;
        }
    }
    panic!("Expected let declaration");
}

#[test]
fn test_parse_if_else() {
    let program = parse("if true { let x = 1; } else { let x = 2; }").unwrap();
    if let Program::Block(stmts) = program {
        if let Stmt::If { condition, else_branch, .. } = &stmts[0] {
            assert!(matches!(condition, Expr::Bool(true)));
            assert!(else_branch.is_some());
            return;
        }
    }
    panic!("Expected if statement");
}

#[test]
fn test_parse_while_loop() {
    let program = parse("while true { let x = 1; }").unwrap();
    if let Program::Block(stmts) = program {
        if let Stmt::While { condition, .. } = &stmts[0] {
            assert!(matches!(condition, Expr::Bool(true)));
            return;
        }
    }
    panic!("Expected while loop");
}

#[test]
fn test_parse_for_loop() {
    let program = parse("for i in 0..10 { let x = 1; }").unwrap();
    if let Program::Block(stmts) = program {
        if let Stmt::For { var_name, .. } = &stmts[0] {
            assert_eq!(var_name, "i");
            return;
        }
    }
    panic!("Expected for loop");
}

#[test]
fn test_parse_multiple_statements() {
    let program = parse("let x = 1; let y = 2; x + y").unwrap();
    if let Program::Block(stmts) = program {
        assert_eq!(stmts.len(), 3);
        return;
    }
    panic!("Expected 3 statements");
}

#[test]
fn test_parse_function_call() {
    let program = parse("add(1, 2)").unwrap();
    if let Program::Block(stmts) = program {
        if let Stmt::Expr(Expr::Call { callee, args }) = &stmts[0] {
            assert_eq!(callee, "add");
            assert_eq!(args.len(), 2);
            return;
        }
    }
    panic!("Expected function call");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p linkc_parser`
Expected: FAIL - `Parser` doesn't exist

- [ ] **Step 3: Implement Parser**

Add to `crates/linkc_parser/src/lib.rs` (after the `TypeAnnotation` Display impl, before the test module):

```rust
use linkc_lexer::{lex, SpannedToken, Token};

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&SpannedToken> { self.tokens.get(self.pos) }
    fn current_token(&self) -> &Token { &self.tokens[self.pos].token }
    fn check(&self, token: Token) -> bool { self.peek().map_or(false, |t| t.token == token) }
    fn eat(&mut self, token: Token) -> bool { if self.check(token) { self.advance(); true } else { false } }
    fn expect(&mut self, token: Token) -> Result<(), String> {
        if self.eat(token.clone()) { Ok(()) } else { Err(format!("Expected {}, found {}", token, self.current_token())) }
    }
    fn advance(&mut self) { if self.pos < self.tokens.len() - 1 { self.pos += 1; } }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut stmts = Vec::new();
        while !self.check(Token::Eof) {
            stmts.push(self.parse_stmt()?);
            self.eat(Token::Semicolon);
        }
        Ok(Program::Block(stmts))
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.current_token().clone() {
            Token::Fn => self.parse_fn_decl(),
            Token::Let => self.parse_let_decl(),
            Token::Return => self.parse_return(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::For => self.parse_for(),
            Token::Loop => self.parse_loop(),
            Token::Break => { self.advance(); Ok(Stmt::Break) }
            Token::Continue => { self.advance(); Ok(Stmt::Continue) }
            Token::LeftBrace => { let block = self.parse_block()?; Ok(Stmt::Expr(Expr::BlockExpr(block))) }
            _ => Ok(Stmt::Expr(self.parse_expr()?)),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, String> { self.parse_or() }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.check(Token::Or) { self.advance(); left = Expr::Binary { op: BinOp::Or, left: Box::new(left), right: Box::new(self.parse_and()?) }; }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        while self.check(Token::And) { self.advance(); left = Expr::Binary { op: BinOp::And, left: Box::new(left), right: Box::new(self.parse_equality()?) }; }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        loop {
            if self.eat(Token::Eq) { left = Expr::Binary { op: BinOp::Eq, left: Box::new(left), right: Box::new(self.parse_comparison()?) }; }
            else if self.eat(Token::NotEq) { left = Expr::Binary { op: BinOp::Neq, left: Box::new(left), right: Box::new(self.parse_comparison()?) }; }
            else { break; }
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_addition()?;
        loop {
            if self.eat(Token::Lt) { left = Expr::Binary { op: BinOp::Lt, left: Box::new(left), right: Box::new(self.parse_addition()?) }; }
            else if self.eat(Token::Gt) { left = Expr::Binary { op: BinOp::Gt, left: Box::new(left), right: Box::new(self.parse_addition()?) }; }
            else if self.eat(Token::LtEq) { left = Expr::Binary { op: BinOp::LtEq, left: Box::new(left), right: Box::new(self.parse_addition()?) }; }
            else if self.eat(Token::GtEq) { left = Expr::Binary { op: BinOp::GtEq, left: Box::new(left), right: Box::new(self.parse_addition()?) }; }
            else { break; }
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplication()?;
        loop {
            if self.eat(Token::Plus) { left = Expr::Binary { op: BinOp::Add, left: Box::new(left), right: Box::new(self.parse_multiplication()?) }; }
            else if self.eat(Token::Minus) { left = Expr::Binary { op: BinOp::Sub, left: Box::new(left), right: Box::new(self.parse_multiplication()?) }; }
            else { break; }
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            if self.eat(Token::Star) { left = Expr::Binary { op: BinOp::Mul, left: Box::new(left), right: Box::new(self.parse_unary()?) }; }
            else if self.eat(Token::Slash) { left = Expr::Binary { op: BinOp::Div, left: Box::new(left), right: Box::new(self.parse_unary()?) }; }
            else if self.eat(Token::Percent) { left = Expr::Binary { op: BinOp::Mod, left: Box::new(left), right: Box::new(self.parse_unary()?) }; }
            else { break; }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.eat(Token::Minus) { let op = self.parse_unary()?; return Ok(Expr::Unary { op: UnaryOp::Neg, operand: Box::new(op) }); }
        if self.eat(Token::Not) { let op = self.parse_unary()?; return Ok(Expr::Unary { op: UnaryOp::Not, operand: Box::new(op) }); }
        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expr, String> {
        let expr = self.parse_primary()?;
        if self.check(Token::LeftParen) {
            if let Expr::Ident(name) = expr {
                self.advance();
                let mut args = Vec::new();
                if !self.check(Token::RightParen) {
                    args.push(self.parse_expr()?);
                    while self.eat(Token::Comma) { args.push(self.parse_expr()?); }
                }
                self.expect(Token::RightParen)?;
                return Ok(Expr::Call { callee: name, args });
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let token = self.current_token().clone();
        match token {
            Token::Int(n) => { self.advance(); Ok(Expr::Int(n)) }
            Token::Float(n) => { self.advance(); Ok(Expr::Float(n)) }
            Token::Str(s) => { self.advance(); Ok(Expr::Str(s)) }
            Token::Bool(b) => { self.advance(); Ok(Expr::Bool(b)) }
            Token::None => { self.advance(); Ok(Expr::None) }
            Token::Ident(name) => { self.advance(); Ok(Expr::Ident(name)) }
            Token::LeftParen => { self.advance(); let expr = self.parse_expr()?; self.expect(Token::RightParen)?; Ok(expr) }
            Token::LeftBrace => { let block = self.parse_block()?; Ok(Expr::BlockExpr(block)) }
            _ => Err(format!("Unexpected token: {}", token)),
        }
    }

    fn parse_fn_decl(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Fn)?;
        let name = if let Token::Ident(ref s) = self.current_token() { s.clone() } else { return Err("Expected function name".to_string()) };
        self.advance();
        self.expect(Token::LeftParen)?;
        let mut params = Vec::new();
        while !self.check(Token::RightParen) {
            let param_name = if let Token::Ident(ref s) = self.current_token() { s.clone() } else { return Err("Expected param name".to_string()) };
            self.advance();
            self.expect(Token::Colon)?;
            let type_ann = self.parse_type_annotation()?;
            params.push((param_name, type_ann));
            if !self.eat(Token::Comma) { break; }
        }
        self.expect(Token::RightParen)?;
        let return_type = if self.eat(Token::Arrow) { Some(self.parse_type_annotation()?) } else { None };
        let body = self.parse_block()?;
        Ok(Stmt::FnDecl { name, params, return_type, body })
    }

    fn parse_let_decl(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Let)?;
        let name = if let Token::Ident(ref s) = self.current_token() { s.clone() } else { return Err("Expected var name".to_string()) };
        self.advance();
        let type_annotation = if self.eat(Token::Colon) { Some(self.parse_type_annotation()?) } else { None };
        let value = if self.eat(Token::Assign) { Some(self.parse_expr()?) } else { None };
        Ok(Stmt::LetDecl { name, type_annotation, value })
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Return)?;
        let value = if self.check(Token::Semicolon) || self.check(Token::RightBrace) || self.check(Token::Eof) { None } else { Some(self.parse_expr()?) };
        Ok(Stmt::Return(value))
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.expect(Token::If)?;
        let condition = self.parse_expr()?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.eat(Token::Else) {
            Some(if self.check(Token::If) { let s = self.parse_if()?; Block { stmts: vec![s] } } else { self.parse_block()? })
        } else { None };
        Ok(Stmt::If { condition, then_branch, else_branch })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.expect(Token::While)?;
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::While { condition, body })
    }

    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.expect(Token::For)?;
        let var_name = if let Token::Ident(ref s) = self.current_token() { s.clone() } else { return Err("Expected var name".to_string()) };
        self.advance();
        self.expect(Token::In)?;
        let start = self.parse_expr()?;
        self.expect(Token::Dot)?;
        self.expect(Token::Dot)?;
        let end = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::For { var_name, start, end, body })
    }

    fn parse_loop(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Loop)?;
        let body = self.parse_block()?;
        Ok(Stmt::Loop(body))
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect(Token::LeftBrace)?;
        let mut stmts = Vec::new();
        while !self.check(Token::RightBrace) && !self.check(Token::Eof) {
            stmts.push(self.parse_stmt()?);
            self.eat(Token::Semicolon);
        }
        self.expect(Token::RightBrace)?;
        Ok(Block { stmts })
    }

    fn parse_type_annotation(&mut self) -> Result<TypeAnnotation, String> {
        let token = self.current_token().clone();
        match token {
            Token::Ident(ref s) => {
                let s = s.clone();
                self.advance();
                Ok(match s.as_str() {
                    "i32" => TypeAnnotation::I32,
                    "i64" => TypeAnnotation::I64,
                    "f32" => TypeAnnotation::F32,
                    "f64" => TypeAnnotation::F64,
                    "bool" => TypeAnnotation::Bool,
                    "str" => TypeAnnotation::Str,
                    _ => TypeAnnotation::Named(s),
                })
            }
            Token::LeftParen => { self.advance(); self.expect(Token::RightParen)?; Ok(TypeAnnotation::Unit) }
            _ => Err(format!("Expected type, found {}", token)),
        }
    }
}
```

- [ ] **Step 4: Run all parser tests**

Run: `cargo test -p linkc_parser`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "feat: implement parser with expression + statement parsing"
```

---

### Task 6: Implement Interpreter with Functions and CLI

**Files:**
- Modify: `crates/linkc_interpreter/src/lib.rs`
- Modify: `crates/linkc_cli/src/main.rs`
- Create: `tests/fixtures/fib.link`

- [ ] **Step 1: Verify interpreter tests pass**

Run: `cargo test -p linkc_interpreter`
Expected: All 13 interpreter tests pass

- [ ] **Step 2: Add function call tests to interpreter**

Add to the interpreter test module:

```rust
#[test]
fn test_eval_function_call() {
    assert_eq!(run("fn add(a: i32, b: i32) -> i32 { return a + b; } add(2, 3)").unwrap(), Value::Int(5));
}

#[test]
fn test_eval_recursive_fib() {
    assert_eq!(run("fn fib(n: i32) -> i32 { if n <= 1 { return n; } return fib(n - 1) + fib(n - 2); } fib(10)").unwrap(), Value::Int(55));
}

#[test]
fn test_eval_while_loop() {
    assert_eq!(run("let x = 0; while x < 5 { let x = x + 1; } x").unwrap(), Value::Int(5));
}

#[test]
fn test_eval_for_loop() {
    assert_eq!(run("let sum = 0; for i in 0..5 { let sum = sum + i; } sum").unwrap(), Value::Int(10));
}
```

- [ ] **Step 3: Run function call tests**

Run: `cargo test -p linkc_interpreter`
Expected: All tests pass

- [ ] **Step 4: Create CLI implementation**

Replace `crates/linkc_cli/src/main.rs`:

```rust
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: link run <file.link>");
        process::exit(1);
    }

    let command = &args[1];
    if command == "run" {
        let filename = &args[2];
        match run_file(filename) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    } else if command == "--version" || command == "-V" {
        println!("linkc 0.1.0");
    } else if command == "--help" || command == "-h" {
        println!("Usage: link <command> [args]");
        println!("Commands:");
        println!("  run <file>  Run a Link source file");
        println!("  --version   Print version");
        println!("  --help      Print this help");
    } else {
        eprintln!("Unknown command: {}", command);
        process::exit(1);
    }
}

fn run_file(filename: &str) -> Result<(), String> {
    let source = fs::read_to_string(filename).map_err(|e| format!("Cannot read file: {}", e))?;
    let tokens = linkc_lexer::lex(&source);
    let mut parser = linkc_parser::Parser::new(tokens);
    let program = parser.parse_program()?;
    let mut env = linkc_interpreter::Environment::new();
    let result = linkc_interpreter::eval_program(&program, &mut env)?;
    match result {
        linkc_interpreter::Value::Int(n) => println!("{}", n),
        linkc_interpreter::Value::Float(f) => println!("{}", f),
        linkc_interpreter::Value::Str(s) => println!("{}", s),
        linkc_interpreter::Value::Bool(b) => println!("{}", b),
        linkc_interpreter::Value::None => println!("none"),
        linkc_interpreter::Value::Function { name, .. } => println!("<fn {}>", name),
    }
    Ok(())
}
```

- [ ] **Step 5: Create fibonacci test file**

Create `tests/fixtures/fib.link`:

```
fn fib(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

fib(10)
```

- [ ] **Step 6: Build and run fibonacci**

Run: `cargo run -p linkc_cli -- run tests/fixtures/fib.link`
Expected: Output `55`

- [ ] **Step 7: Run full test suite**

Run: `cargo test`
Expected: All tests across all 4 crates pass

- [ ] **Step 8: Commit**

```bash
git add .
git commit -m "feat: implement interpreter, CLI, and fibonacci demo"
```

---

### Task 7: Integration Test and Polish

**Files:**
- Create: `tests/integration_test.rs`

- [ ] **Step 1: Create integration test**

Create `tests/integration_test.rs`:

```rust
use std::process::Command;

#[test]
fn test_fibonacci_program() {
    let output = Command::new(env!("CARGO_BIN_EXE_link"))
        .arg("run")
        .arg("tests/fixtures/fib.link")
        .output()
        .expect("Failed to execute link");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("55"), "Expected 55, got: {}", stdout);
}

#[test]
fn test_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_link"))
        .arg("--help")
        .output()
        .expect("Failed to execute link");
    assert!(output.status.success());
}

#[test]
fn test_cli_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_link"))
        .arg("--version")
        .output()
        .expect("Failed to execute link");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.1.0"));
}
```

- [ ] **Step 2: Add test dependency to CLI Cargo.toml**

Add to `crates/linkc_cli/Cargo.toml`:
```toml
[dev-dependencies]
```

- [ ] **Step 3: Run integration tests**

Run: `cargo test --test integration_test`
Expected: All 3 tests pass

- [ ] **Step 4: Run ALL tests**

Run: `cargo test`
Expected: ALL tests (lexer + parser + interpreter + integration) pass

- [ ] **Step 5: Final commit**

```bash
git add .
git commit -m "feat: add integration tests and polish"
```

---

### Done Criteria

Phase 0 is complete when:
- [ ] `cargo build` succeeds
- [ ] `cargo test` shows all tests pass
- [ ] `cargo run -p linkc_cli -- run tests/fixtures/fib.link` outputs `55`
- [ ] All 4 crates compile: linkc_lexer, linkc_parser, linkc_interpreter, linkc_cli
- [ ] Lexer handles: integers, floats, strings, bools, keywords, operators, delimiters, comments
- [ ] Parser handles: expressions with precedence, binary/unary ops, function calls, all statement types
- [ ] Interpreter handles: all value types, arithmetic, comparisons, function calls (including recursive), control flow
- [ ] CLI supports: `run <file>`, `--help`, `--version`
