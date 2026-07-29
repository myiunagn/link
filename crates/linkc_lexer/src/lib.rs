#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // 字面量
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Ident(String),

    // 关键字
    Fn, Let, Return, If, Else, Match, For, While, Loop, In,
    True, False, None, Some, Ok, Err, As,
    Break, Continue,
    Extern, Export, Async, Await, Struct, Enum, Impl, Trait, Use, Mod, Pub, Mut, Domain,
    Stream, Flow, Pipeline, Source, Sample,

    // 运算符
    Plus, Minus, Star, Slash, Percent,
    Assign, Eq, NotEq, Lt, Gt, LtEq, GtEq,
    And, Or, Not, Ampersand, Pipe, Arrow, FatArrow,
    Dot, Colon, DoubleColon, Semicolon, Comma, Underscore,

    // 分隔符
    LeftParen, RightParen, LeftBrace, RightBrace, LeftBracket, RightBracket,

    // 特殊
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
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),
            Token::Extern => write!(f, "extern"),
            Token::Export => write!(f, "export"),
            Token::Async => write!(f, "async"),
            Token::Await => write!(f, "await"),
            Token::Struct => write!(f, "struct"),
            Token::Enum => write!(f, "enum"),
            Token::Impl => write!(f, "impl"),
            Token::Trait => write!(f, "trait"),
            Token::Use => write!(f, "use"),
            Token::Mod => write!(f, "mod"),
            Token::Pub => write!(f, "pub"),
            Token::Mut => write!(f, "mut"),
            Token::Domain => write!(f, "domain"),
            Token::Stream => write!(f, "stream"),
            Token::Flow => write!(f, "flow"),
            Token::Pipeline => write!(f, "pipeline"),
            Token::Source => write!(f, "source"),
            Token::Sample => write!(f, "sample"),
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
            Token::DoubleColon => write!(f, "::"),
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
                tokens.push(SpannedToken {
                    token: Token::Eof,
                    span: Span::new(self.pos, self.pos),
                });
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
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '/' && self.peek_next() == Some('/') {
                while let Some(c) = self.peek() {
                    if c == '\n' { break; }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Token {
        let ch = self.advance().unwrap();
        match ch {
            '+' => Token::Plus,
            '-' => {
                if self.peek() == Some('>') { self.advance(); Token::Arrow }
                else { Token::Minus }
            }
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '=' => {
                if self.peek() == Some('=') { self.advance(); Token::Eq }
                else if self.peek() == Some('>') { self.advance(); Token::FatArrow }
                else { Token::Assign }
            }
            '!' => {
                if self.peek() == Some('=') { self.advance(); Token::NotEq }
                else { Token::Not }
            }
            '<' => {
                if self.peek() == Some('=') { self.advance(); Token::LtEq }
                else { Token::Lt }
            }
            '>' => {
                if self.peek() == Some('=') { self.advance(); Token::GtEq }
                else { Token::Gt }
            }
            '&' => {
                if self.peek() == Some('&') { self.advance(); Token::And }
                else { Token::Ampersand }
            }
            '|' => {
                if self.peek() == Some('|') { self.advance(); Token::Or }
                else { Token::Pipe }
            }
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            ';' => Token::Semicolon,
            ',' => Token::Comma,
            ':' => {
                if self.peek() == Some(':') {
                    self.advance();
                    Token::DoubleColon
                } else {
                    Token::Colon
                }
            }
            '.' => Token::Dot,
            '_' => {
                if self.peek().map(|c| c.is_alphanumeric()).unwrap_or(false) {
                    self.lex_ident_or_keyword("_".to_string())
                } else {
                    Token::Underscore
                }
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
            } else {
                s.push(ch);
            }
        }
        panic!("Unterminated string literal")
    }

    fn lex_number(&mut self, first: char) -> Token {
        let mut num_str = String::new();
        num_str.push(first);
        let mut is_float = false;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !is_float && self.peek_next() != Some('.') {
                is_float = true;
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        if let Some(ch) = self.peek() {
            if ch == 'e' || ch == 'E' {
                is_float = true;
                num_str.push(ch);
                self.advance();
                if let Some(sign) = self.peek() {
                    if sign == '+' || sign == '-' {
                        num_str.push(sign);
                        self.advance();
                    }
                }
                let mut found_exp_digit = false;
                while let Some(d) = self.peek() {
                    if d.is_ascii_digit() {
                        num_str.push(d);
                        self.advance();
                        found_exp_digit = true;
                    } else {
                        break;
                    }
                }
                if !found_exp_digit {
                    panic!("Invalid float literal: missing exponent digits");
                }
            }
        }
        if is_float {
            Token::Float(num_str.parse().expect("Invalid float literal"))
        } else {
            Token::Int(num_str.parse().expect("Invalid integer literal"))
        }
    }

    fn lex_ident_or_keyword(&mut self, start: String) -> Token {
        let mut ident = start;
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
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
            "break" => Token::Break,
            "continue" => Token::Continue,
            "extern" => Token::Extern,
            "export" => Token::Export,
            "async" => Token::Async,
            "await" => Token::Await,
            "struct" => Token::Struct,
            "enum" => Token::Enum,
            "impl" => Token::Impl,
            "trait" => Token::Trait,
            "use" => Token::Use,
            "mod" => Token::Mod,
            "pub" => Token::Pub,
            "mut" => Token::Mut,
            "domain" => Token::Domain,
            "stream" => Token::Stream,
            "flow" => Token::Flow,
            "pipeline" => Token::Pipeline,
            "source" => Token::Source,
            "sample" => Token::Sample,
            _ => Token::Ident(ident),
        }
    }
}

pub fn lex(source: &str) -> Vec<SpannedToken> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_display() {
        assert_eq!(format!("{}", Token::Int(42)), "42");
        assert_eq!(format!("{}", Token::Ident("foo".to_string())), "foo");
        assert_eq!(format!("{}", Token::Plus), "+");
    }

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

    #[test]
    fn test_lex_stream_keyword() {
        let tokens = lex("stream stream_of_ints");
        assert_eq!(tokens[0].token, Token::Stream);
        assert_eq!(tokens[1].token, Token::Ident("stream_of_ints".to_string()));
    }

    #[test]
    fn test_lex_flow_keywords() {
        let tokens = lex("flow pipeline source sample");
        assert_eq!(tokens[0].token, Token::Flow);
        assert_eq!(tokens[1].token, Token::Pipeline);
        assert_eq!(tokens[2].token, Token::Source);
        assert_eq!(tokens[3].token, Token::Sample);
    }

    #[test]
    fn test_lex_async_await_keywords() {
        let tokens = lex("async fn await");
        assert_eq!(tokens[0].token, Token::Async);
        assert_eq!(tokens[1].token, Token::Fn);
        assert_eq!(tokens[2].token, Token::Await);
    }
}
