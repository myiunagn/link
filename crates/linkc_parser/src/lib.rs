use linkc_lexer::{SpannedToken, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Program {
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnSignature {
    pub name: String,
    pub params: Vec<(String, TypeAnnotation)>,
    pub return_type: Option<TypeAnnotation>,
    pub is_async: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    FnDecl {
        name: String,
        params: Vec<(String, TypeAnnotation)>,
        return_type: Option<TypeAnnotation>,
        body: Block,
    },
    LetDecl {
        name: String,
        type_annotation: Option<TypeAnnotation>,
        value: Option<Expr>,
    },
    Assign {
        target: String,
        value: Expr,
    },
    Expr(Expr),
    Return(Option<Expr>),
    If {
        condition: Expr,
        then_branch: Block,
        else_branch: Option<Block>,
    },
    While {
        condition: Expr,
        body: Block,
    },
    For {
        var_name: String,
        start: Expr,
        end: Expr,
        body: Block,
    },
    Loop(Block),
    Break,
    Continue,
    ExternDecl {
        language: String,
        module: Option<String>,
        decls: Vec<FnSignature>,
    },
    ExportDecl {
        language: String,
        module: Option<String>,
        decls: Vec<FnSignature>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    None,
    Ident(String),
    List(Vec<Expr>),
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Call {
        callee: String,
        args: Vec<Expr>,
    },
    IfExpr {
        condition: Box<Expr>,
        then_value: Box<Expr>,
        else_value: Box<Expr>,
    },
    BlockExpr(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Neq, Lt, Gt, LtEq, GtEq,
    And, Or, Pipe,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeAnnotation {
    I32, I64, F32, F64, Bool, Str, Unit, Named(String),
    U8, U16, U32, U64, USize, I8, I16,
    Void,
    Ptr(Box<TypeAnnotation>),
    Stream(Box<TypeAnnotation>),
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Int(n) => write!(f, "{}", n),
            Expr::Float(n) => write!(f, "{}", n),
            Expr::Str(s) => write!(f, "\"{}\"", s),
            Expr::Bool(b) => write!(f, "{}", b),
            Expr::None => write!(f, "none"),
            Expr::Ident(s) => write!(f, "{}", s),
            Expr::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Expr::Index { target, index } => write!(f, "{}[{}]", target, index),
            Expr::Binary { op, left, right } => {
                let op_str = match op {
                    BinOp::Add => "+", BinOp::Sub => "-", BinOp::Mul => "*",
                    BinOp::Div => "/", BinOp::Mod => "%", BinOp::Eq => "==",
                    BinOp::Neq => "!=", BinOp::Lt => "<", BinOp::Gt => ">",
                    BinOp::LtEq => "<=", BinOp::GtEq => ">=",
                    BinOp::And => "&&", BinOp::Or => "||", BinOp::Pipe => "|",
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
            TypeAnnotation::U8 => write!(f, "u8"),
            TypeAnnotation::U16 => write!(f, "u16"),
            TypeAnnotation::U32 => write!(f, "u32"),
            TypeAnnotation::U64 => write!(f, "u64"),
            TypeAnnotation::USize => write!(f, "usize"),
            TypeAnnotation::I8 => write!(f, "i8"),
            TypeAnnotation::I16 => write!(f, "i16"),
            TypeAnnotation::Void => write!(f, "void"),
            TypeAnnotation::Ptr(inner) => write!(f, "*mut {}", inner),
            TypeAnnotation::Stream(inner) => write!(f, "stream<{}>", inner),
        }
    }
}

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
    fn check(&self, token: Token) -> bool {
        self.peek().map_or(false, |t| t.token == token)
    }
    fn eat(&mut self, token: Token) -> bool {
        if self.check(token) { self.advance(); true } else { false }
    }
    fn expect(&mut self, token: Token) -> Result<(), String> {
        if self.eat(token.clone()) { Ok(()) } else {
            Err(format!("Expected {}, found {}", token, self.current_token()))
        }
    }
    fn advance(&mut self) {
        if self.pos < self.tokens.len() - 1 { self.pos += 1; }
    }

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
            Token::Extern => self.parse_extern_decl(),
            Token::Export => self.parse_export_decl(),
            Token::LeftBrace => {
                let block = self.parse_block()?;
                Ok(Stmt::Expr(Expr::BlockExpr(block)))
            }
            Token::Ident(name) => {
                if self.tokens.get(self.pos + 1).map_or(false, |t| t.token == Token::Assign) {
                    self.advance();
                    self.advance();
                    let value = self.parse_expr()?;
                    Ok(Stmt::Assign { target: name, value })
                } else {
                    Ok(Stmt::Expr(self.parse_expr()?))
                }
            }
            _ => Ok(Stmt::Expr(self.parse_expr()?)),
        }
    }

    fn parse_extern_decl(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Extern)?;
        let language = self.parse_string_literal()?;
        let module = if let Token::Ident(ref s) = self.current_token().clone() {
            if s == "module" {
                self.advance();
                Some(self.parse_string_literal()?)
            } else {
                None
            }
        } else {
            None
        };
        let decls = self.parse_fn_signatures()?;
        Ok(Stmt::ExternDecl { language, module, decls })
    }

    fn parse_export_decl(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Export)?;
        let language = self.parse_string_literal()?;
        let module = if let Token::Ident(ref s) = self.current_token().clone() {
            if s == "module" {
                self.advance();
                Some(self.parse_string_literal()?)
            } else {
                None
            }
        } else {
            None
        };
        let decls = self.parse_fn_signatures()?;
        Ok(Stmt::ExportDecl { language, module, decls })
    }

    fn parse_string_literal(&mut self) -> Result<String, String> {
        match self.current_token().clone() {
            Token::Str(s) => { self.advance(); Ok(s) }
            other => Err(format!("Expected string literal, found {}", other)),
        }
    }

    fn parse_fn_signatures(&mut self) -> Result<Vec<FnSignature>, String> {
        self.expect(Token::LeftBrace)?;
        let mut decls = Vec::new();
        while !self.check(Token::RightBrace) && !self.check(Token::Eof) {
            let is_async = if self.check(Token::Async) {
                self.advance();
                true
            } else {
                false
            };
            self.expect(Token::Fn)?;
            let name = match self.current_token().clone() {
                Token::Ident(s) => { self.advance(); s }
                other => return Err(format!("Expected function name, found {}", other)),
            };
            self.expect(Token::LeftParen)?;
            let mut params = Vec::new();
            if !self.check(Token::RightParen) {
                loop {
                    let param_name = match self.current_token().clone() {
                        Token::Ident(s) => { self.advance(); s }
                        other => return Err(format!("Expected parameter name, found {}", other)),
                    };
                    self.expect(Token::Colon)?;
                    let param_type = self.parse_type_annotation()?;
                    params.push((param_name, param_type));
                    if !self.eat(Token::Comma) { break; }
                }
            }
            self.expect(Token::RightParen)?;
            let return_type = if self.eat(Token::Arrow) {
                Some(self.parse_type_annotation()?)
            } else {
                None
            };
            self.eat(Token::Semicolon);
            decls.push(FnSignature { name, params, return_type, is_async });
        }
        self.expect(Token::RightBrace)?;
        Ok(decls)
    }

    fn parse_expr(&mut self) -> Result<Expr, String> { self.parse_pipe() }

    fn parse_pipe(&mut self) -> Result<Expr, String> {
        let left = self.parse_or()?;
        if self.check(Token::Pipe) {
            self.advance();
            let right = self.parse_pipe()?;
            Ok(Expr::Binary { op: BinOp::Pipe, left: Box::new(left), right: Box::new(right) })
        } else {
            Ok(left)
        }
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.check(Token::Or) {
            self.advance();
            left = Expr::Binary { op: BinOp::Or, left: Box::new(left), right: Box::new(self.parse_and()?) };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        while self.check(Token::And) {
            self.advance();
            left = Expr::Binary { op: BinOp::And, left: Box::new(left), right: Box::new(self.parse_equality()?) };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        loop {
            if self.eat(Token::Eq) {
                left = Expr::Binary { op: BinOp::Eq, left: Box::new(left), right: Box::new(self.parse_comparison()?) };
            } else if self.eat(Token::NotEq) {
                left = Expr::Binary { op: BinOp::Neq, left: Box::new(left), right: Box::new(self.parse_comparison()?) };
            } else { break; }
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_addition()?;
        loop {
            if self.eat(Token::Lt) {
                left = Expr::Binary { op: BinOp::Lt, left: Box::new(left), right: Box::new(self.parse_addition()?) };
            } else if self.eat(Token::Gt) {
                left = Expr::Binary { op: BinOp::Gt, left: Box::new(left), right: Box::new(self.parse_addition()?) };
            } else if self.eat(Token::LtEq) {
                left = Expr::Binary { op: BinOp::LtEq, left: Box::new(left), right: Box::new(self.parse_addition()?) };
            } else if self.eat(Token::GtEq) {
                left = Expr::Binary { op: BinOp::GtEq, left: Box::new(left), right: Box::new(self.parse_addition()?) };
            } else { break; }
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplication()?;
        loop {
            if self.eat(Token::Plus) {
                left = Expr::Binary { op: BinOp::Add, left: Box::new(left), right: Box::new(self.parse_multiplication()?) };
            } else if self.eat(Token::Minus) {
                left = Expr::Binary { op: BinOp::Sub, left: Box::new(left), right: Box::new(self.parse_multiplication()?) };
            } else { break; }
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            if self.eat(Token::Star) {
                left = Expr::Binary { op: BinOp::Mul, left: Box::new(left), right: Box::new(self.parse_unary()?) };
            } else if self.eat(Token::Slash) {
                left = Expr::Binary { op: BinOp::Div, left: Box::new(left), right: Box::new(self.parse_unary()?) };
            } else if self.eat(Token::Percent) {
                left = Expr::Binary { op: BinOp::Mod, left: Box::new(left), right: Box::new(self.parse_unary()?) };
            } else { break; }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.eat(Token::Minus) {
            let op = self.parse_unary()?;
            return Ok(Expr::Unary { op: UnaryOp::Neg, operand: Box::new(op) });
        }
        if self.eat(Token::Not) {
            let op = self.parse_unary()?;
            return Ok(Expr::Unary { op: UnaryOp::Not, operand: Box::new(op) });
        }
        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.check(Token::LeftParen) {
                if let Expr::Ident(name) = expr {
                    self.advance();
                    let mut args = Vec::new();
                    if !self.check(Token::RightParen) {
                        args.push(self.parse_expr()?);
                        while self.eat(Token::Comma) {
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.expect(Token::RightParen)?;
                    expr = Expr::Call { callee: name, args };
                } else {
                    break;
                }
            } else if self.check(Token::LeftBracket) {
                self.advance();
                let index = self.parse_expr()?;
                self.expect(Token::RightBracket)?;
                expr = Expr::Index { target: Box::new(expr), index: Box::new(index) };
            } else {
                break;
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
            Token::Stream => { self.advance(); Ok(Expr::Ident("stream".to_string())) }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            Token::LeftBracket => {
                self.advance();
                let mut items = Vec::new();
                if !self.check(Token::RightBracket) {
                    items.push(self.parse_expr()?);
                    while self.eat(Token::Comma) {
                        if self.check(Token::RightBracket) { break; }
                        items.push(self.parse_expr()?);
                    }
                }
                self.expect(Token::RightBracket)?;
                Ok(Expr::List(items))
            }
            Token::LeftBrace => {
                let block = self.parse_block()?;
                Ok(Expr::BlockExpr(block))
            }
            _ => Err(format!("Unexpected token: {}", token)),
        }
    }

    fn parse_fn_decl(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Fn)?;
        let name = if let Token::Ident(ref s) = self.current_token() {
            s.clone()
        } else {
            return Err(format!("Expected function name, found {}", self.current_token()));
        };
        self.advance();
        self.expect(Token::LeftParen)?;
        let mut params = Vec::new();
        while !self.check(Token::RightParen) {
            let param_name = if let Token::Ident(ref s) = self.current_token() {
                s.clone()
            } else {
                return Err(format!("Expected parameter name, found {}", self.current_token()));
            };
            self.advance();
            self.expect(Token::Colon)?;
            let type_ann = self.parse_type_annotation()?;
            params.push((param_name, type_ann));
            if !self.eat(Token::Comma) { break; }
        }
        self.expect(Token::RightParen)?;
        let return_type = if self.eat(Token::Arrow) {
            Some(self.parse_type_annotation()?)
        } else { None };
        let body = self.parse_block()?;
        Ok(Stmt::FnDecl { name, params, return_type, body })
    }

    fn parse_let_decl(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Let)?;
        let name = if let Token::Ident(ref s) = self.current_token() {
            s.clone()
        } else {
            return Err(format!("Expected variable name, found {}", self.current_token()));
        };
        self.advance();
        let type_annotation = if self.eat(Token::Colon) {
            Some(self.parse_type_annotation()?)
        } else { None };
        let value = if self.eat(Token::Assign) {
            Some(self.parse_expr()?)
        } else { None };
        Ok(Stmt::LetDecl { name, type_annotation, value })
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Return)?;
        let value = if self.check(Token::Semicolon) || self.check(Token::RightBrace) || self.check(Token::Eof) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        Ok(Stmt::Return(value))
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.expect(Token::If)?;
        let condition = self.parse_expr()?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.eat(Token::Else) {
            Some(if self.check(Token::If) {
                let s = self.parse_if()?;
                Block { stmts: vec![s] }
            } else {
                self.parse_block()?
            })
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
        let var_name = if let Token::Ident(ref s) = self.current_token() {
            s.clone()
        } else {
            return Err(format!("Expected variable name, found {}", self.current_token()));
        };
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
        let inner = match token {
            Token::Ident(ref s) => {
                let s = s.clone();
                self.advance();
                Ok(match s.as_str() {
                    "i8" => TypeAnnotation::I8,
                    "i16" => TypeAnnotation::I16,
                    "i32" => TypeAnnotation::I32,
                    "i64" => TypeAnnotation::I64,
                    "u8" => TypeAnnotation::U8,
                    "u16" => TypeAnnotation::U16,
                    "u32" => TypeAnnotation::U32,
                    "u64" => TypeAnnotation::U64,
                    "usize" => TypeAnnotation::USize,
                    "f32" => TypeAnnotation::F32,
                    "f64" => TypeAnnotation::F64,
                    "bool" => TypeAnnotation::Bool,
                    "str" => TypeAnnotation::Str,
                    "void" => TypeAnnotation::Void,
                    _ => TypeAnnotation::Named(s),
                })
            }
            Token::Stream => {
                self.advance();
                self.expect(Token::Lt)?;
                let inner = self.parse_type_annotation()?;
                self.expect(Token::Gt)?;
                Ok(TypeAnnotation::Stream(Box::new(inner)))
            }
            Token::LeftParen => {
                self.advance();
                self.expect(Token::RightParen)?;
                Ok(TypeAnnotation::Unit)
            }
            _ => Err(format!("Expected type annotation, found {}", token)),
        }?;

        // 解析指针类型: *mut T
        if self.check(Token::Star) {
            self.advance();
            // 可选的 mut 关键字
            if let Token::Ident(ref s) = self.current_token().clone() {
                if s == "mut" {
                    self.advance();
                }
            }
            Ok(TypeAnnotation::Ptr(Box::new(inner)))
        } else {
            Ok(inner)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linkc_lexer::lex;

    fn parse(source: &str) -> Result<Program, String> {
        let tokens = lex(source);
        let mut parser = Parser::new(tokens);
        parser.parse_program()
    }

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

    #[test]
    fn test_parse_integer() {
        let program = parse("42").unwrap();
        let Program::Block(stmts) = program;
        assert!(matches!(&stmts[0], Stmt::Expr(Expr::Int(42))));
    }

    #[test]
    fn test_parse_binary_add() {
        let program = parse("1 + 2").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::Expr(Expr::Binary { op, .. }) = &stmts[0] {
            assert!(matches!(op, BinOp::Add));
            return;
        }
        panic!("Expected binary expression");
    }

    #[test]
    fn test_parse_precedence() {
        let program = parse("1 + 2 * 3").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::Expr(Expr::Binary { op, right, .. }) = &stmts[0] {
            assert!(matches!(op, BinOp::Add));
            if let Expr::Binary { op: inner_op, .. } = right.as_ref() {
                assert!(matches!(inner_op, BinOp::Mul));
                return;
            }
        }
        panic!("Expected (1 + (2 * 3))");
    }

    #[test]
    fn test_parse_paren_group() {
        let program = parse("(1 + 2) * 3").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::Expr(Expr::Binary { op, .. }) = &stmts[0] {
            assert!(matches!(op, BinOp::Mul));
            return;
        }
        panic!("Expected ((1 + 2) * 3)");
    }

    #[test]
    fn test_parse_unary_neg() {
        let program = parse("-42").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::Expr(Expr::Unary { op, operand }) = &stmts[0] {
            assert!(matches!(op, UnaryOp::Neg));
            assert!(matches!(operand.as_ref(), Expr::Int(42)));
            return;
        }
        panic!("Expected unary negation");
    }

    #[test]
    fn test_parse_fn_decl() {
        let program = parse("fn add(a: i32, b: i32) -> i32 { return a + b; }").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::FnDecl { name, params, return_type, .. } = &stmts[0] {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert!(matches!(return_type, Some(TypeAnnotation::I32)));
            return;
        }
        panic!("Expected function declaration");
    }

    #[test]
    fn test_parse_let_decl() {
        let program = parse("let x: i32 = 42").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::LetDecl { name, type_annotation, value } = &stmts[0] {
            assert_eq!(name, "x");
            assert!(matches!(type_annotation, Some(TypeAnnotation::I32)));
            assert!(value.is_some());
            return;
        }
        panic!("Expected let declaration");
    }

    #[test]
    fn test_parse_if_else() {
        let program = parse("if true { let x = 1; } else { let x = 2; }").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::If { condition, else_branch, .. } = &stmts[0] {
            assert!(matches!(condition, Expr::Bool(true)));
            assert!(else_branch.is_some());
            return;
        }
        panic!("Expected if statement");
    }

    #[test]
    fn test_parse_while_loop() {
        let program = parse("while true { let x = 1; }").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::While { condition, .. } = &stmts[0] {
            assert!(matches!(condition, Expr::Bool(true)));
            return;
        }
        panic!("Expected while loop");
    }

    #[test]
    fn test_parse_for_loop() {
        let program = parse("for i in 0..10 { let x = 1; }").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::For { var_name, .. } = &stmts[0] {
            assert_eq!(var_name, "i");
            return;
        }
        panic!("Expected for loop");
    }

    #[test]
    fn test_parse_multiple_statements() {
        let program = parse("let x = 1; let y = 2; x + y").unwrap();
        let Program::Block(stmts) = program;
        assert_eq!(stmts.len(), 3);
    }

    #[test]
    fn test_parse_function_call() {
        let program = parse("add(1, 2)").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::Expr(Expr::Call { callee, args }) = &stmts[0] {
            assert_eq!(callee, "add");
            assert_eq!(args.len(), 2);
            return;
        }
        panic!("Expected function call");
    }
}
