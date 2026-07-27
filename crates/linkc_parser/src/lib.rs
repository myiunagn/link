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
pub struct StructField {
    pub name: String,
    pub type_ann: TypeAnnotation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariantDecl {
    pub name: String,
    pub payload: Vec<TypeAnnotation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// 通配符 `_`
    Wildcard,
    /// 字面量: int/float/str/bool/none
    Literal(Expr),
    /// 绑定变量: `name`
    Bind(String),
    /// 枚举变体无参数: `Color::Red`
    EnumVariant { type_name: String, variant: String },
    /// 枚举变体带参数: `Color::RGB(r, g, b)`
    EnumVariantWithPayload {
        type_name: String,
        variant: String,
        bindings: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    FnDecl {
        name: String,
        params: Vec<(String, TypeAnnotation)>,
        return_type: Option<TypeAnnotation>,
        body: Block,
        is_async: bool,
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
    StructDecl {
        name: String,
        fields: Vec<StructField>,
    },
    EnumDecl {
        name: String,
        variants: Vec<EnumVariantDecl>,
    },
    Match {
        scrutinee: Expr,
        arms: Vec<MatchArm>,
    },
    /// flow 声明块:声明式数据流定义
    /// `flow Name "description" { source: <expr>; pipeline: <expr>; }`
    FlowDecl {
        name: String,
        description: Option<String>,
        source: Option<Expr>,
        pipeline: Expr,
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
    /// 字段访问: `expr.field`
    FieldAccess {
        target: Box<Expr>,
        field: String,
    },
    /// 路径表达式: `Type::name`，用于枚举变体如 `Color::Red`
    Path {
        base: String,
        segment: String,
    },
    /// 结构体初始化: `Name { field: value, ... }`
    StructInit {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    /// 路径调用: `Type::name(args)`，用于枚举变体带参数如 `Color::RGB(1,2,3)`
    PathCall {
        base: String,
        segment: String,
        args: Vec<Expr>,
    },
    /// match 表达式（可作为表达式使用）
    MatchExpr {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    /// await 表达式: `await <expr>`
    /// v0.1 树漫游解释器中等价于直接求值(阻塞语义)
    Await(Box<Expr>),
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
            Expr::FieldAccess { target, field } => write!(f, "{}.{}", target, field),
            Expr::Path { base, segment } => write!(f, "{}::{}", base, segment),
            Expr::StructInit { name, fields } => {
                write!(f, "{} {{ ", name)?;
                for (i, (fname, fval)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", fname, fval)?;
                }
                write!(f, " }}")
            }
            Expr::PathCall { base, segment, args } => {
                write!(f, "{}::{}(", base, segment)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expr::MatchExpr { scrutinee, arms } => {
                write!(f, "match {} {{ ... }} ({} arms)", scrutinee, arms.len())
            }
            Expr::Await(inner) => write!(f, "await {}", inner),
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
            Token::Async => self.parse_fn_decl(),
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
            Token::Struct => self.parse_struct_decl(),
            Token::Enum => self.parse_enum_decl(),
            Token::Match => self.parse_match_stmt(),
            Token::Flow => self.parse_flow_decl(),
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
        // await <expr> —— 前缀运算符,优先级与一元运算符相同
        if self.eat(Token::Await) {
            let inner = self.parse_unary()?;
            return Ok(Expr::Await(Box::new(inner)));
        }
        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.check(Token::LeftParen) {
                if let Expr::Ident(name) = &expr {
                    let name = name.clone();
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
            } else if self.check(Token::Dot) {
                // 检查是否是范围操作符 `..`，如果是则不在这里处理字段访问
                let next_is_dot = self.tokens.get(self.pos + 1)
                    .map_or(false, |t| t.token == Token::Dot);
                if next_is_dot {
                    break;
                }
                self.advance();
                let field = match self.current_token().clone() {
                    Token::Ident(s) => { self.advance(); s }
                    other => return Err(format!("Expected field name after '.', found {}", other)),
                };
                expr = Expr::FieldAccess { target: Box::new(expr), field };
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
            // `source` 在 flow 块内是字段关键字,但在表达式中作为变量名使用
            Token::Source => { self.advance(); Ok(Expr::Ident("source".to_string())) }
            Token::Ident(name) => {
                self.advance();
                // 路径: Type::Variant 或 Type::Variant(args)
                if self.check(Token::DoubleColon) {
                    self.advance();
                    let segment = match self.current_token().clone() {
                        Token::Ident(s) => { self.advance(); s }
                        other => return Err(format!("Expected identifier after '::', found {}", other)),
                    };
                    // 如果紧跟 `(`，则是带参数的枚举变体调用
                    if self.check(Token::LeftParen) {
                        self.advance();
                        let mut args = Vec::new();
                        if !self.check(Token::RightParen) {
                            args.push(self.parse_expr()?);
                            while self.eat(Token::Comma) {
                                args.push(self.parse_expr()?);
                            }
                        }
                        self.expect(Token::RightParen)?;
                        Ok(Expr::PathCall { base: name, segment, args })
                    } else {
                        Ok(Expr::Path { base: name, segment })
                    }
                } else if self.check(Token::LeftBrace) {
                    // 仅当符合 `Name { field: ... }` 或 `Name {}` 模式时才识别为 struct init
                    // 避免与块表达式冲突，例如 `match c { ... }` 中的 `c {`
                    let is_struct_init = {
                        let next = self.tokens.get(self.pos + 1).map(|t| &t.token);
                        match next {
                            Some(Token::RightBrace) => true,
                            Some(Token::Ident(_)) => {
                                let after = self.tokens.get(self.pos + 2).map(|t| &t.token);
                                matches!(after, Some(Token::Colon))
                            }
                            _ => false,
                        }
                    };
                    if is_struct_init {
                        self.advance();
                        let mut fields = Vec::new();
                        if !self.check(Token::RightBrace) {
                            loop {
                                let field_name = match self.current_token().clone() {
                                    Token::Ident(s) => { self.advance(); s }
                                    other => return Err(format!("Expected field name, found {}", other)),
                                };
                                self.expect(Token::Colon)?;
                                let field_val = self.parse_expr()?;
                                fields.push((field_name, field_val));
                                if !self.eat(Token::Comma) { break; }
                            }
                        }
                        self.expect(Token::RightBrace)?;
                        Ok(Expr::StructInit { name, fields })
                    } else {
                        Ok(Expr::Ident(name))
                    }
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            Token::Stream => { self.advance(); Ok(Expr::Ident("stream".to_string())) }
            Token::Match => self.parse_match_expr(),
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
        // 可选 async 前缀
        let is_async = self.eat(Token::Async);
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
        Ok(Stmt::FnDecl { name, params, return_type, body, is_async })
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

    /// 解析结构体声明: `struct Name { field: T, ... }`
    fn parse_struct_decl(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Struct)?;
        let name = match self.current_token().clone() {
            Token::Ident(s) => { self.advance(); s }
            other => return Err(format!("Expected struct name, found {}", other)),
        };
        self.expect(Token::LeftBrace)?;
        let mut fields = Vec::new();
        while !self.check(Token::RightBrace) && !self.check(Token::Eof) {
            let field_name = match self.current_token().clone() {
                Token::Ident(s) => { self.advance(); s }
                other => return Err(format!("Expected field name, found {}", other)),
            };
            self.expect(Token::Colon)?;
            let type_ann = self.parse_type_annotation()?;
            fields.push(StructField { name: field_name, type_ann });
            if !self.eat(Token::Comma) { break; }
        }
        self.expect(Token::RightBrace)?;
        Ok(Stmt::StructDecl { name, fields })
    }

    /// 解析枚举声明: `enum Name { Variant, Variant2(T, T), ... }`
    fn parse_enum_decl(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Enum)?;
        let name = match self.current_token().clone() {
            Token::Ident(s) => { self.advance(); s }
            other => return Err(format!("Expected enum name, found {}", other)),
        };
        self.expect(Token::LeftBrace)?;
        let mut variants = Vec::new();
        while !self.check(Token::RightBrace) && !self.check(Token::Eof) {
            let variant_name = match self.current_token().clone() {
                Token::Ident(s) => { self.advance(); s }
                other => return Err(format!("Expected variant name, found {}", other)),
            };
            // 可选的载荷类型列表: (T, T, ...)
            let payload = if self.check(Token::LeftParen) {
                self.advance();
                let mut types = Vec::new();
                if !self.check(Token::RightParen) {
                    types.push(self.parse_type_annotation()?);
                    while self.eat(Token::Comma) {
                        types.push(self.parse_type_annotation()?);
                    }
                }
                self.expect(Token::RightParen)?;
                types
            } else {
                Vec::new()
            };
            variants.push(EnumVariantDecl { name: variant_name, payload });
            if !self.eat(Token::Comma) { break; }
        }
        self.expect(Token::RightBrace)?;
        Ok(Stmt::EnumDecl { name, variants })
    }

    /// 解析 match 语句: `match scrutinee { pattern => body, ... }`
    fn parse_match_stmt(&mut self) -> Result<Stmt, String> {
        let expr = self.parse_match_expr()?;
        if let Expr::MatchExpr { scrutinee, arms } = expr {
            Ok(Stmt::Match { scrutinee: *scrutinee, arms })
        } else {
            unreachable!("parse_match_expr should return MatchExpr")
        }
    }

    /// 解析 flow 声明块
    ///
    /// 语法:
    /// ```text
    /// flow Name "description" {
    ///     source: <expr>;
    ///     sample: every 1s;     // 解析但当前忽略(v0.1 无异步调度)
    ///     pipeline:
    ///         <expr>;
    /// }
    /// ```
    /// `description` 与 `source` / `sample` 字段都是可选的;`pipeline:` 必须存在。
    fn parse_flow_decl(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Flow)?;
        let name = match self.current_token().clone() {
            Token::Ident(s) => { self.advance(); s }
            other => return Err(format!("Expected flow name, found {}", other)),
        };
        // 可选描述字符串
        let description = if let Token::Str(_) = self.current_token() {
            match self.current_token().clone() {
                Token::Str(s) => { self.advance(); Some(s) }
                _ => None,
            }
        } else {
            None
        };
        self.expect(Token::LeftBrace)?;

        let mut source: Option<Expr> = None;
        let mut pipeline: Option<Expr> = None;

        while !self.check(Token::RightBrace) && !self.check(Token::Eof) {
            match self.current_token().clone() {
                Token::Source => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    let expr = self.parse_expr()?;
                    self.eat(Token::Semicolon);
                    source = Some(expr);
                }
                Token::Sample => {
                    // v0.1 暂不实现时间调度,跳过整个 sample 字段
                    self.advance();
                    self.expect(Token::Colon)?;
                    // 跳过到下一个分号
                    while !self.check(Token::Semicolon) && !self.check(Token::RightBrace) && !self.check(Token::Eof) {
                        self.advance();
                    }
                    self.eat(Token::Semicolon);
                }
                Token::Pipeline => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    let expr = self.parse_expr()?;
                    self.eat(Token::Semicolon);
                    pipeline = Some(expr);
                }
                other => return Err(format!("Expected 'source' / 'sample' / 'pipeline' in flow block, found {}", other)),
            }
        }
        self.expect(Token::RightBrace)?;

        let pipeline = pipeline.ok_or_else(|| format!("flow {} missing 'pipeline:' section", name))?;
        Ok(Stmt::FlowDecl { name, description, source, pipeline })
    }

    /// 解析 match 表达式
    fn parse_match_expr(&mut self) -> Result<Expr, String> {
        self.expect(Token::Match)?;
        let scrutinee = self.parse_expr()?;
        self.expect(Token::LeftBrace)?;
        let mut arms = Vec::new();
        while !self.check(Token::RightBrace) && !self.check(Token::Eof) {
            let pattern = self.parse_pattern()?;
            self.expect(Token::FatArrow)?;
            let body = self.parse_block()?;
            arms.push(MatchArm { pattern, body });
            self.eat(Token::Comma);
        }
        self.expect(Token::RightBrace)?;
        Ok(Expr::MatchExpr { scrutinee: Box::new(scrutinee), arms })
    }

    /// 解析单个模式
    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        let token = self.current_token().clone();
        match token {
            Token::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Token::Int(n) => { self.advance(); Ok(Pattern::Literal(Expr::Int(n))) }
            Token::Float(n) => { self.advance(); Ok(Pattern::Literal(Expr::Float(n))) }
            Token::Str(s) => { self.advance(); Ok(Pattern::Literal(Expr::Str(s))) }
            Token::Bool(b) => { self.advance(); Ok(Pattern::Literal(Expr::Bool(b))) }
            Token::None => { self.advance(); Ok(Pattern::Literal(Expr::None)) }
            Token::Ident(name) => {
                self.advance();
                // 检查是否是路径: Type::Variant
                if self.check(Token::DoubleColon) {
                    self.advance();
                    let variant = match self.current_token().clone() {
                        Token::Ident(s) => { self.advance(); s }
                        other => return Err(format!("Expected variant name after '::', found {}", other)),
                    };
                    // 检查是否有载荷绑定: (a, b, c)
                    if self.check(Token::LeftParen) {
                        self.advance();
                        let mut bindings = Vec::new();
                        if !self.check(Token::RightParen) {
                            loop {
                                match self.current_token().clone() {
                                    Token::Ident(s) => { self.advance(); bindings.push(s) }
                                    Token::Underscore => { self.advance(); bindings.push("_".to_string()) }
                                    other => return Err(format!("Expected binding in pattern, found {}", other)),
                                }
                                if !self.eat(Token::Comma) { break; }
                            }
                        }
                        self.expect(Token::RightParen)?;
                        Ok(Pattern::EnumVariantWithPayload {
                            type_name: name,
                            variant,
                            bindings,
                        })
                    } else {
                        Ok(Pattern::EnumVariant { type_name: name, variant })
                    }
                } else {
                    // 单纯标识符: 绑定变量
                    Ok(Pattern::Bind(name))
                }
            }
            other => Err(format!("Unexpected pattern token: {}", other)),
        }
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
    fn test_parse_async_fn_decl() {
        let program = parse("async fn fetch(url: str) -> str { return url; }").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::FnDecl { name, is_async, .. } = &stmts[0] {
            assert_eq!(name, "fetch");
            assert!(*is_async);
            return;
        }
        panic!("Expected async function declaration");
    }

    #[test]
    fn test_parse_await_expression() {
        let program = parse("await fetch(\"https://example.com\")").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::Expr(expr) = &stmts[0] {
            assert!(matches!(expr, Expr::Await(_)));
            return;
        }
        panic!("Expected await expression");
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

    #[test]
    fn test_parse_struct_decl() {
        let program = parse("struct Point { x: i32, y: i32 }").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::StructDecl { name, fields } = &stmts[0] {
            assert_eq!(name, "Point");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].name, "x");
            return;
        }
        panic!("Expected struct declaration");
    }

    #[test]
    fn test_parse_enum_decl() {
        let program = parse("enum Color { Red, Green, Blue, RGB(i32, i32, i32) }").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::EnumDecl { name, variants } = &stmts[0] {
            assert_eq!(name, "Color");
            assert_eq!(variants.len(), 4);
            assert_eq!(variants[0].name, "Red");
            assert_eq!(variants[3].name, "RGB");
            assert_eq!(variants[3].payload.len(), 3);
            return;
        }
        panic!("Expected enum declaration");
    }

    #[test]
    fn test_parse_struct_init_and_field_access() {
        let program = parse("let p = Point { x: 1, y: 2 }; p.x").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::LetDecl { value: Some(Expr::StructInit { name, fields }), .. } = &stmts[0] {
            assert_eq!(name, "Point");
            assert_eq!(fields.len(), 2);
        } else {
            panic!("Expected struct init");
        }
        if let Stmt::Expr(Expr::FieldAccess { target, field }) = &stmts[1] {
            assert_eq!(field, "x");
            assert!(matches!(target.as_ref(), Expr::Ident(_)));
        } else {
            panic!("Expected field access");
        }
    }

    #[test]
    fn test_parse_enum_path() {
        let program = parse("let c = Color::Red").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::LetDecl { value: Some(Expr::Path { base, segment }), .. } = &stmts[0] {
            assert_eq!(base, "Color");
            assert_eq!(segment, "Red");
            return;
        }
        panic!("Expected enum path");
    }

    #[test]
    fn test_parse_enum_path_call() {
        let program = parse("let c = Color::RGB(255, 0, 0)").unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::LetDecl { value: Some(Expr::PathCall { base, segment, args }), .. } = &stmts[0] {
            assert_eq!(base, "Color");
            assert_eq!(segment, "RGB");
            assert_eq!(args.len(), 3);
            return;
        }
        panic!("Expected enum path call");
    }

    #[test]
    fn test_parse_match_stmt() {
        let src = "match c { Color::Red => { 1 } Color::RGB(r, g, b) => { r } _ => { 0 } }";
        let program = parse(src).unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::Match { scrutinee, arms } = &stmts[0] {
            assert!(matches!(scrutinee, Expr::Ident(_)));
            assert_eq!(arms.len(), 3);
            assert!(matches!(arms[0].pattern, Pattern::EnumVariant { .. }));
            assert!(matches!(arms[1].pattern, Pattern::EnumVariantWithPayload { .. }));
            assert!(matches!(arms[2].pattern, Pattern::Wildcard));
            return;
        }
        panic!("Expected match statement");
    }

    #[test]
    fn test_parse_flow_decl_basic() {
        let src = r#"
            flow MyFlow "描述" {
                source: stream([1, 2, 3]);
                pipeline: source | collect;
            }
        "#;
        let program = parse(src).unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::FlowDecl { name, description, source, pipeline } = &stmts[0] {
            assert_eq!(name, "MyFlow");
            assert_eq!(description.as_deref(), Some("描述"));
            assert!(source.is_some());
            assert!(matches!(pipeline, Expr::Binary { op: BinOp::Pipe, .. }));
            return;
        }
        panic!("Expected flow declaration");
    }

    #[test]
    fn test_parse_flow_decl_minimal() {
        let src = r#"
            flow Bare {
                pipeline: stream([1]) | collect;
            }
        "#;
        let program = parse(src).unwrap();
        let Program::Block(stmts) = program;
        if let Stmt::FlowDecl { name, description, source, pipeline } = &stmts[0] {
            assert_eq!(name, "Bare");
            assert!(description.is_none());
            assert!(source.is_none());
            assert!(matches!(pipeline, Expr::Binary { op: BinOp::Pipe, .. }));
            return;
        }
        panic!("Expected flow declaration");
    }

    #[test]
    fn test_parse_flow_decl_with_sample() {
        // sample 字段应被解析(不报错),其内容当前被忽略
        let src = r#"
            flow Sampled {
                source: stream([1]);
                sample: every 1s;
                pipeline: source | collect;
            }
        "#;
        let program = parse(src).unwrap();
        let Program::Block(stmts) = program;
        assert!(matches!(&stmts[0], Stmt::FlowDecl { name, .. } if name == "Sampled"));
    }

    #[test]
    fn test_parse_flow_decl_missing_pipeline_errors() {
        let src = r#"
            flow Bad {
                source: stream([1]);
            }
        "#;
        let result = parse(src);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("missing 'pipeline:'"));
    }
}
