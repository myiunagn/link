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
        target: Box<Expr>,
        value: Box<Expr>,
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
    ForIterable {
        var_name: String,
        iterable: Expr,
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
    /// 模块声明: `module foo::bar;`
    ModDecl {
        name: String,
    },
    /// 导入声明: `import foo::bar;` 或 `import foo::bar as baz;`
    UseDecl {
        path: Vec<String>,
        alias: Option<String>,
    },
    /// 域声明: `domain Name { key: value, ... }`
    /// 用于定义游戏后端域配置 (tick_rate, max_players, 事件处理函数等)
    DomainDecl {
        name: String,
        config: Vec<(String, Expr)>,
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
    Tuple(Vec<Expr>),
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
    /// try! 表达式: `try! <expr>`
    Try(Box<Expr>),
    /// 匿名函数/lambda: `fn(x: i64) -> i64 { ... }`
    Lambda {
        params: Vec<(String, TypeAnnotation)>,
        return_type: Option<TypeAnnotation>,
        body: Block,
    },
    /// 借用表达式: `&expr` 或 `&mut expr`
    Ref(Box<Expr>, bool),
    /// 解引用表达式: `*expr`
    Deref(Box<Expr>),
    /// 类型转换: `expr as Type`
    AsCast(Box<Expr>, TypeAnnotation),
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
    Ref(Box<TypeAnnotation>, bool),
    Generic(Box<TypeAnnotation>, Vec<TypeAnnotation>),
    Array(Box<TypeAnnotation>, u64),
    Tuple(Vec<TypeAnnotation>),
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
            Expr::Try(inner) => write!(f, "try! {}", inner),
            Expr::Lambda { params, return_type, .. } => {
                let params_str: Vec<String> = params.iter()
                    .map(|(n, t)| format!("{}: {}", n, t))
                    .collect();
                if let Some(rt) = return_type {
                    write!(f, "fn({}) -> {} {{ ... }}", params_str.join(", "), rt)
                } else {
                    write!(f, "fn({}) {{ ... }}", params_str.join(", "))
                }
            }
            Expr::Ref(inner, is_mut) => {
                if *is_mut {
                    write!(f, "&mut {}", inner)
                } else {
                    write!(f, "&{}", inner)
                }
            }
            Expr::Deref(inner) => write!(f, "*{}", inner),
            Expr::AsCast(expr, ty) => write!(f, "({} as {})", expr, ty),
            Expr::Tuple(elems) => {
                write!(f, "(")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
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
            TypeAnnotation::Ref(inner, is_mut) => {
                if *is_mut {
                    write!(f, "&mut {}", inner)
                } else {
                    write!(f, "&{}", inner)
                }
            }
            TypeAnnotation::Generic(base, args) => {
                write!(f, "{}<", base)?;
                for (i, a) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", a)?;
                }
                write!(f, ">")
            }
            TypeAnnotation::Array(elem, size) => {
                write!(f, "[{}; {}]", elem, size)
            }
            TypeAnnotation::Tuple(elems) => {
                write!(f, "(")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
        }
    }
}

pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
    suppress_call_suffix_block: bool,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, pos: 0, suppress_call_suffix_block: false }
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

    fn skip_angle_bracket_placeholder(&mut self) {
        if !self.eat(Token::Lt) { return; }
        let mut d = 1;
        while d > 0 && !self.check(Token::Eof) {
            if self.check(Token::Lt) { d += 1; }
            if self.check(Token::Gt) { d -= 1; }
            self.advance();
        }
    }

    fn skip_bracket_placeholder(&mut self) {
        if !self.eat(Token::LeftBracket) { return; }
        let mut d = 1;
        while d > 0 && !self.check(Token::Eof) {
            if self.check(Token::LeftBracket) { d += 1; }
            if self.check(Token::RightBracket) { d -= 1; }
            self.advance();
        }
    }

    fn skip_ellipsis_if_present(&mut self) -> bool {
        if self.check(Token::Dot) {
            let saved = self.pos;
            self.advance();
            if self.check(Token::Dot) {
                self.advance();
                if self.check(Token::Dot) {
                    self.advance();
                    return true;
                }
            }
            self.pos = saved;
        }
        false
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
        // ... 三重点占位符: 伪代码 D 类, 跳过到分号/结束
        if self.check(Token::Dot) {
            let saved = self.pos;
            self.advance();
            if self.check(Token::Dot) {
                self.advance();
                if self.check(Token::Dot) {
                    self.advance();
                    // 吃掉可选分号, 然后返回空表达式占位
                    self.eat(Token::Semicolon);
                    return Ok(Stmt::Expr(Expr::None));
                }
            }
            self.pos = saved;
        }
        // pub 修饰符前缀: 跳过 pub, 递归解析后续语句
        if let Token::Ident(ref s) = self.current_token().clone() {
            if s == "pub" {
                self.advance();
                return self.parse_stmt();
            }
            // const 声明: const NAME: Type = value;
            if s == "const" {
                self.advance();
                let _name = if let Token::Ident(s2) = self.current_token().clone() { self.advance(); s2 } else { "".to_string() };
                if self.eat(Token::Colon) { let _ = self.parse_type_annotation(); }
                if self.eat(Token::Assign) { let _ = self.parse_expr(); }
                self.eat(Token::Semicolon);
                return Ok(Stmt::Expr(Expr::None));
            }
            // impl 声明块: impl Trait for Type { ... } 或 impl Type { ... }
            if s == "impl" {
                self.advance();
                // 吞到匹配的 {, 然后吞整个块
                let mut brace_found = false;
                while !self.check(Token::Eof) {
                    if self.check(Token::LeftBrace) {
                        brace_found = true;
                        break;
                    }
                    self.advance();
                }
                if brace_found {
                    self.advance(); // {
                    let mut depth = 1;
                    while depth > 0 && !self.check(Token::Eof) {
                        match self.current_token() {
                            Token::LeftBrace => { depth += 1; self.advance(); }
                            Token::RightBrace => { depth -= 1; if depth > 0 { self.advance(); } }
                            _ => { self.advance(); }
                        }
                    }
                    self.eat(Token::RightBrace);
                }
                self.eat(Token::Semicolon);
                return Ok(Stmt::Expr(Expr::None));
            }
            // type alias: type Name = Type :> Constraint :> ...;
            if s == "type" {
                self.advance();
                let _name = if let Token::Ident(s2) = self.current_token().clone() { self.advance(); s2 } else { "".to_string() };
                if self.check(Token::Lt) {
                    let mut depth = 1;
                    self.advance();
                    while depth > 0 && !self.check(Token::Eof) {
                        match self.current_token() {
                            Token::Lt => { depth += 1; self.advance(); }
                            Token::Gt => { depth -= 1; if depth > 0 { self.advance(); } }
                            _ => { self.advance(); }
                        }
                    }
                    self.eat(Token::Gt);
                }
                if self.eat(Token::Assign) {
                    while !self.check(Token::Semicolon) && !self.check(Token::Eof) {
                        self.advance();
                    }
                }
                self.eat(Token::Semicolon);
                return Ok(Stmt::Expr(Expr::None));
            }
            // module / mod 声明: module foo::bar::baz;
            if s == "module" || s == "mod" {
                self.advance();
                let mut parts = Vec::new();
                match self.current_token().clone() {
                    Token::Ident(s2) => { parts.push(s2); self.advance(); }
                    other => return Err(format!("Expected module name after '{}', found {}", s, other)),
                }
                while self.check(Token::DoubleColon) {
                    self.advance();
                    match self.current_token().clone() {
                        Token::Ident(s2) => { parts.push(s2); self.advance(); }
                        other => return Err(format!("Expected identifier after '::' in module path, found {}", other)),
                    }
                }
                self.eat(Token::Semicolon);
                return Ok(Stmt::ModDecl { name: parts.join("::") });
            }
            // use 声明: 当 Token::Use 不存在时 Ident("use") 兜底
            if s == "use" {
                self.advance();
                let mut path = Vec::new();
                match self.current_token().clone() {
                    Token::Ident(s2) => { path.push(s2); self.advance(); }
                    other => return Err(format!("Expected identifier after 'use', found {}", other)),
                }
                while self.check(Token::DoubleColon) {
                    self.advance();
                    match self.current_token().clone() {
                        Token::Ident(s2) => { path.push(s2); self.advance(); }
                        Token::LeftBrace => {
                            self.advance();
                            while !self.check(Token::RightBrace) && !self.check(Token::Eof) {
                                if let Token::Ident(s3) = self.current_token().clone() {
                                    self.advance();
                                    if self.check(Token::As) { self.advance(); if let Token::Ident(_) = self.current_token().clone() { self.advance(); } }
                                }
                                if !self.eat(Token::Comma) { break; }
                            }
                            self.expect(Token::RightBrace)?;
                            break;
                        }
                        other => return Err(format!("Expected identifier after '::' in use path, found {}", other)),
                    }
                }
                let alias = if self.check(Token::As) {
                    self.advance();
                    match self.current_token().clone() {
                        Token::Ident(s2) => { self.advance(); Some(s2) }
                        other => return Err(format!("Expected identifier after 'as', found {}", other)),
                    }
                } else { None };
                self.eat(Token::Semicolon);
                return Ok(Stmt::UseDecl { path, alias });
            }
            // room / device / domain 扩展: room BattleArena { ... } / device X { ... }
            // 作为伪代码类型块: 先 save pos, 确认后续符合声明模式再吞, 否则回退 (避免误吞变量名调用)
            if s == "room" || s == "device" || s == "component" || s == "entity" || s == "state" ||
               s == "thing" || s == "adapter" || s == "matchmaker" || s == "server" || s == "client" ||
               s == "player" || s == "domain" {
                let saved_kw_pos = self.pos;
                self.advance();
                let mut looks_like_decl = false;
                // 可选泛型参数 <T, U: Bound> - 简化吞到匹配 >
                if self.eat(Token::Lt) {
                    looks_like_decl = true;
                    let mut depth = 1;
                    while depth > 0 && !self.check(Token::Eof) {
                        if self.check(Token::Lt) { depth += 1; }
                        if self.check(Token::Gt) { depth -= 1; }
                        self.advance();
                    }
                }
                // 吃名称
                if let Token::Ident(_) = self.current_token().clone() {
                    looks_like_decl = true;
                    self.advance();
                }
                // 可选 :> Supertype (如 player :> Endpoint)
                if self.check(Token::Colon) {
                    let saved_pos = self.pos;
                    self.advance(); // :
                    if self.check(Token::Gt) {
                        looks_like_decl = true;
                        self.advance(); // >
                        // 吃 Supertype 标识符或路径 (A::B::C)
                        loop {
                            if let Token::Ident(_) = self.current_token().clone() { self.advance(); } else { break; }
                            if self.check(Token::DoubleColon) { self.advance(); } else { break; }
                        }
                    } else {
                        self.pos = saved_pos;
                    }
                }
                // 吃可选描述字符串
                if let Token::Str(_) = self.current_token().clone() {
                    looks_like_decl = true;
                    self.advance();
                }
                // 如果接下来是 {, 吞到匹配的 }
                if self.check(Token::LeftBrace) {
                    looks_like_decl = true;
                    self.advance();
                    let mut depth = 1;
                    while depth > 0 && !self.check(Token::Eof) {
                        match self.current_token() {
                            Token::LeftBrace => { depth += 1; self.advance(); }
                            Token::RightBrace => { depth -= 1; if depth > 0 { self.advance(); } }
                            _ => { self.advance(); }
                        }
                    }
                    self.eat(Token::RightBrace);
                }
                // 关键: 只有确实像是声明块 (泛型/名称/子类型/描述/{/分号) 才 return None 占位
                // 否则回退 saved_kw_pos, 让后续走正常 Token::Ident 表达式分支
                if looks_like_decl || self.check(Token::Semicolon) || self.check(Token::Eof) {
                    self.eat(Token::Semicolon);
                    return Ok(Stmt::Expr(Expr::None));
                } else {
                    self.pos = saved_kw_pos;
                }
            }
            // inline / consteval 修饰符前缀后跟 fn
            if (s == "inline" || s == "consteval") {
                let lookahead = self.tokens.get(self.pos + 1).map(|t| t.token.clone());
                if matches!(lookahead, Some(Token::Fn) | Some(Token::Async)) {
                    self.advance();
                    return self.parse_fn_decl();
                }
            }
            // import 关键字别名到 use
            if s == "import" {
                self.advance();
                let mut path = Vec::new();
                match self.current_token().clone() {
                    Token::Ident(s2) => { path.push(s2); self.advance(); }
                    other => return Err(format!("Expected identifier after 'import', found {}", other)),
                }
                while self.check(Token::DoubleColon) {
                    self.advance();
                    match self.current_token().clone() {
                        Token::Ident(s2) => { path.push(s2); self.advance(); }
                        Token::LeftBrace => {
                            // 分组导入
                            self.advance();
                            while !self.check(Token::RightBrace) && !self.check(Token::Eof) {
                                if let Token::Ident(s3) = self.current_token().clone() {
                                    self.advance();
                                    if self.check(Token::As) { self.advance(); if let Token::Ident(_) = self.current_token().clone() { self.advance(); } }
                                }
                                if !self.eat(Token::Comma) { break; }
                            }
                            self.expect(Token::RightBrace)?;
                            break;
                        }
                        other => return Err(format!("Expected identifier after '::' in import path, found {}", other)),
                    }
                }
                let alias = if self.check(Token::As) {
                    self.advance();
                    match self.current_token().clone() {
                        Token::Ident(s2) => { self.advance(); Some(s2) }
                        other => return Err(format!("Expected identifier after 'as', found {}", other)),
                    }
                } else {
                    None
                };
                self.expect(Token::Semicolon)?;
                return Ok(Stmt::UseDecl { path, alias });
            }
        }
        match self.current_token().clone() {
            Token::Pub => {
                self.advance();
                self.parse_stmt()
            }
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
            Token::Mod => self.parse_mod_decl(),
            Token::Use => self.parse_use_decl(),
            Token::Domain => self.parse_domain_decl(),
            Token::LeftBrace => {
                let block = self.parse_block()?;
                Ok(Stmt::Expr(Expr::BlockExpr(block)))
            }
            token @ (Token::Ident(_) | Token::Stream | Token::Pipeline | Token::Source | Token::Sample) => {
                let name = match token {
                    Token::Ident(s) => s,
                    Token::Stream => "stream".to_string(),
                    Token::Pipeline => "pipeline".to_string(),
                    Token::Source => "source".to_string(),
                    Token::Sample => "sample".to_string(),
                    _ => unreachable!(),
                };
                // 检测类型块声明或子类型声明:
                // name<T> { ... } / name :> Base { ... } / name { ... }
                // (D 类伪代码: stream<T>/player/room/interface/component/state/entity/device/thing/adapter/matchmaker 等)
                let saved_pos = self.pos;
                self.advance();
                // 声明关键字 (interface/room/device/...) 后面允许跟可选名称 Ident
                let decl_keywords = ["interface", "component", "state", "entity", "device", "thing", "adapter", "matchmaker", "server", "client", "domain", "player", "room", "memory", "function", "matchmaker", "service", "adapter", "protocol"];
                let name_lower = name.to_lowercase();
                let is_decl_keyword = decl_keywords.iter().any(|k| k == &&name_lower[..]);
                let mut extra_name_taken = false;
                if is_decl_keyword {
                    // 尝试吃可选的名称 Ident
                    if let Token::Ident(_) = self.current_token().clone() {
                        self.advance();
                        extra_name_taken = true;
                    }
                }
                // 如果自身是 stream，没在上面的 decl_keywords，手动强制 is_decl_keyword = true 来吃泛型后 block
                let forced_block = name_lower == "stream" || name_lower == "pipeline" || name_lower == "source" || name_lower == "sample";
                // 可选泛型 <...>
                if self.eat(Token::Lt) {
                    let mut depth = 1;
                    while depth > 0 && !self.check(Token::Eof) {
                        if self.check(Token::Lt) { depth += 1; }
                        if self.check(Token::Gt) { depth -= 1; }
                        self.advance();
                    }
                }
                // 可选子类型后缀 :> Base
                if self.check(Token::Colon) {
                    let save2 = self.pos;
                    self.advance();
                    if self.check(Token::Gt) {
                        // :> 语法, 吞掉 Base 标识符 (可能有多个用冒号间隔?)
                        self.advance();
                        // 吃 Base 名
                        if let Token::Ident(_) = self.current_token().clone() {
                            self.advance();
                        }
                    } else {
                        self.pos = save2;
                    }
                }
                // 如果接下来是 {, 判断是 struct init fields 还是伪代码类型声明块
                if self.check(Token::LeftBrace) {
                    // 检查: { 里的第一个非空 token 模式: struct init fields?
                    let next1 = self.tokens.get(self.pos + 1).map(|t| &t.token);
                    let next2 = self.tokens.get(self.pos + 2).map(|t| &t.token);
                    let looks_like_struct_init = matches!(next1, Some(Token::RightBrace)) ||
                        (matches!(next1, Some(Token::Ident(_))) && matches!(next2, Some(Token::Colon))) ||
                        (matches!(next1, Some(Token::Ident(_))) && matches!(next2, Some(Token::Comma)));
                    if !looks_like_struct_init || is_decl_keyword || forced_block {
                        // 吞到匹配的 }
                        self.advance(); // {
                        let mut depth = 1;
                        while depth > 0 && !self.check(Token::Eof) {
                            match self.current_token() {
                                Token::LeftBrace => { depth += 1; self.advance(); }
                                Token::RightBrace => { depth -= 1; if depth > 0 { self.advance(); } }
                                _ => { self.advance(); }
                            }
                        }
                        self.eat(Token::RightBrace);
                        self.eat(Token::Semicolon);
                        return Ok(Stmt::Expr(Expr::None));
                    }
                }
                // 否则回退到普通表达式解析
                self.pos = saved_pos;

                let saved_pos2 = self.pos;
                let lhs_result = self.parse_expr();
                match lhs_result {
                    Ok(lhs_expr) => {
                        if self.check(Token::Assign) {
                            self.advance();
                            let value = self.parse_expr()?;
                            Ok(Stmt::Assign { target: Box::new(lhs_expr), value: Box::new(value) })
                        } else {
                            Ok(Stmt::Expr(lhs_expr))
                        }
                    }
                    Err(_) => {
                        self.pos = saved_pos2;
                        Ok(Stmt::Expr(self.parse_expr()?))
                    }
                }
            }
            Token::LeftParen | Token::LeftBracket | Token::Star => {
                let saved_pos = self.pos;
                let lhs_result = self.parse_expr();
                match lhs_result {
                    Ok(lhs_expr) => {
                        if self.check(Token::Assign) {
                            self.advance();
                            let value = self.parse_expr()?;
                            Ok(Stmt::Assign { target: Box::new(lhs_expr), value: Box::new(value) })
                        } else {
                            Ok(Stmt::Expr(lhs_expr))
                        }
                    }
                    Err(_) => {
                        self.pos = saved_pos;
                        Ok(Stmt::Expr(self.parse_expr()?))
                    }
                }
            }
            _ => Ok(Stmt::Expr(self.parse_expr()?)),
        }
    }

    fn parse_extern_decl(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Extern)?;
        let language = if self.check(Token::Lt) {
            // D类占位符: extern "<language>" - 跳过 <...> 并使用占位语言名
            self.skip_angle_bracket_placeholder();
            "__placeholder_lang__".to_string()
        } else {
            self.parse_string_literal()?
        };
        // D类占位符: [module "..."] - 跳过方括号可选语法
        if self.check(Token::LeftBracket) {
            self.skip_bracket_placeholder();
        }
        let module = if let Token::Ident(ref s) = self.current_token().clone() {
            if s == "module" || s == "crate" {
                self.advance();
                Some(if self.check(Token::Lt) {
                    self.skip_angle_bracket_placeholder();
                    "__placeholder_module__".to_string()
                } else {
                    self.parse_string_literal()?
                })
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
        if self.check(Token::Fn) || self.check(Token::Async) {
            return self.parse_fn_decl();
        }
        let language = self.parse_string_literal()?;
        let module = if let Token::Ident(ref s) = self.current_token().clone() {
            if s == "module" || s == "crate" {
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
            // D类占位符: ... (三重点省略号) 跳到分号/右花括号后 continue
            if self.skip_ellipsis_if_present() {
                while !self.check(Token::Semicolon) && !self.check(Token::RightBrace) && !self.check(Token::Eof) {
                    if self.check(Token::LeftBrace) {
                        let mut d = 1;
                        self.advance();
                        while d > 0 && !self.check(Token::Eof) {
                            match self.current_token() {
                                Token::LeftBrace => { d += 1; self.advance(); }
                                Token::RightBrace => { d -= 1; if d > 0 { self.advance(); } }
                                _ => { self.advance(); }
                            }
                        }
                    } else {
                        self.advance();
                    }
                }
                self.eat(Token::Semicolon);
                continue;
            }
            if let Token::Ident(ref s) = self.current_token().clone() {
                // 跳过 memory / class / interface / property / function 非函数声明
                if s == "memory" || s == "class" || s == "interface" || s == "property" || s == "function" {
                    // class/interface: 吞到匹配的 } 为止 (可能带嵌套 {})
                    if s == "class" || s == "interface" {
                        self.advance();
                        // 吞类名
                        if let Token::Ident(_) = self.current_token().clone() { self.advance(); }
                        // 可选泛型
                        if self.eat(Token::Lt) {
                            let mut d = 1;
                            while d > 0 && !self.check(Token::Eof) {
                                if self.check(Token::Lt) { d += 1; }
                                if self.check(Token::Gt) { d -= 1; }
                                self.advance();
                            }
                        }
                        // 如果接下来是 {, 吞到匹配 }
                        if self.check(Token::LeftBrace) {
                            self.advance();
                            let mut d = 1;
                            while d > 0 && !self.check(Token::Eof) {
                                match self.current_token() {
                                    Token::LeftBrace => { d += 1; self.advance(); }
                                    Token::RightBrace => { d -= 1; if d > 0 { self.advance(); } }
                                    _ => { self.advance(); }
                                }
                            }
                            self.eat(Token::RightBrace);
                        }
                        self.eat(Token::Semicolon);
                        continue;
                    } else {
                        // property/function/memory: 跳到分号或右花括号
                        while !self.check(Token::Semicolon) && !self.check(Token::RightBrace) && !self.check(Token::Eof) {
                            // 如果遇到 {, 吞到匹配的 }
                            if self.check(Token::LeftBrace) {
                                self.advance();
                                let mut d = 1;
                                while d > 0 && !self.check(Token::Eof) {
                                    match self.current_token() {
                                        Token::LeftBrace => { d += 1; self.advance(); }
                                        Token::RightBrace => { d -= 1; if d > 0 { self.advance(); } }
                                        _ => { self.advance(); }
                                    }
                                }
                                self.eat(Token::RightBrace);
                            } else {
                                self.advance();
                            }
                        }
                        self.eat(Token::Semicolon);
                        continue;
                    }
                }
            }
            let is_async = if self.check(Token::Async) {
                self.advance();
                true
            } else {
                false
            };
            self.expect(Token::Fn)?;
            let name = match self.current_token().clone() {
                Token::Ident(s) => { self.advance(); s }
                Token::Lt => {
                    self.skip_angle_bracket_placeholder();
                    "__placeholder_fn_name__".to_string()
                }
                other => return Err(format!("Expected function name, found {}", other)),
            };
            // 可选泛型 <...>
            if self.eat(Token::Lt) {
                let mut d = 1;
                while d > 0 && !self.check(Token::Eof) {
                    if self.check(Token::Lt) { d += 1; }
                    if self.check(Token::Gt) { d -= 1; }
                    self.advance();
                }
            }
            self.expect(Token::LeftParen)?;
            let mut params = Vec::new();
            if !self.check(Token::RightParen) {
                loop {
                    // D类占位符: ... 省略参数
                    if self.skip_ellipsis_if_present() {
                        // 吃掉逗号分隔的剩余内容直到 )
                        while !self.check(Token::RightParen) && !self.check(Token::Eof) {
                            self.advance();
                        }
                        break;
                    }
                    match self.current_token().clone() {
                        Token::Ident(s) => {
                            self.advance();
                            self.expect(Token::Colon)?;
                            let param_type = self.parse_type_annotation()?;
                            params.push((s, param_type));
                        }
                        Token::Lt => {
                            self.skip_angle_bracket_placeholder();
                            if self.eat(Token::Colon) {
                                let param_type = self.parse_type_annotation()?;
                                params.push(("__placeholder_param__".to_string(), param_type));
                            } else if self.check(Token::Comma) || self.check(Token::RightParen) {
                                params.push(("__placeholder_param__".to_string(), TypeAnnotation::Void));
                            } else {
                                while !self.check(Token::Comma) && !self.check(Token::RightParen) && !self.check(Token::Eof) {
                                    self.advance();
                                }
                                params.push(("__placeholder_param__".to_string(), TypeAnnotation::Void));
                            }
                        }
                        other => return Err(format!("Expected parameter name, found {}", other)),
                    }
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

    fn parse_expr(&mut self) -> Result<Expr, String> { self.parse_pipe(false) }

    fn parse_pipe(&mut self, in_pipeline: bool) -> Result<Expr, String> {
        let mut left = self.parse_or()?;
        if in_pipeline {
            if let Expr::Call { callee: _, ref mut args } = left {
                if self.check(Token::LeftBrace) {
                    let next = self.tokens.get(self.pos + 1).map(|t| &t.token);
                    let is_struct_init = matches!(next, Some(Token::RightBrace)) || (matches!(next, Some(Token::Ident(_))) && matches!(self.tokens.get(self.pos + 2).map(|t| &t.token), Some(Token::Colon)));
                    if !is_struct_init {
                        let block = self.parse_block()?;
                        args.push(Expr::Lambda { params: vec![], return_type: None, body: block });
                    }
                }
            }
        }
        if self.check(Token::Pipe) {
            self.advance();
            let right = self.parse_pipe(true)?;
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
        let mut left = self.parse_cast()?;
        loop {
            if self.eat(Token::Star) {
                left = Expr::Binary { op: BinOp::Mul, left: Box::new(left), right: Box::new(self.parse_cast()?) };
            } else if self.eat(Token::Slash) {
                left = Expr::Binary { op: BinOp::Div, left: Box::new(left), right: Box::new(self.parse_cast()?) };
            } else if self.eat(Token::Percent) {
                left = Expr::Binary { op: BinOp::Mod, left: Box::new(left), right: Box::new(self.parse_cast()?) };
            } else { break; }
        }
        Ok(left)
    }

    fn parse_cast(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            if self.eat(Token::As) {
                let ty = self.parse_type_annotation()?;
                left = Expr::AsCast(Box::new(left), ty);
            } else { break; }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        // try! <expr> —— 前缀 try! 运算符
        if let Token::Ident(ref s) = self.current_token().clone() {
            let next_token = self.tokens.get(self.pos + 1).map(|t| &t.token);
            if s == "try" && next_token == Some(&Token::Not) {
                self.advance(); // "try"
                self.advance(); // "!" (Token::Not)
                let inner = self.parse_unary()?;
                return Ok(Expr::Try(Box::new(inner)));
            }
        }
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
        // 借用: &expr 或 &mut expr
        if self.eat(Token::Ampersand) {
            let mut is_mut = false;
            if self.eat(Token::Mut) {
                is_mut = true;
            }
            let inner = self.parse_unary()?;
            return Ok(Expr::Ref(Box::new(inner), is_mut));
        }
        // 解引用: *expr
        if self.eat(Token::Star) {
            let inner = self.parse_unary()?;
            return Ok(Expr::Deref(Box::new(inner)));
        }
        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.check(Token::LeftParen) {
                let callee = match &expr {
                    Expr::Ident(name) => Some(name.clone()),
                    Expr::FieldAccess { target: _, field } => Some(field.clone()),
                    Expr::Path { base, segment } => Some(format!("{}::{}", base, segment)),
                    _ => None,
                };
                if let Some(name) = callee {
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
            } else if self.check(Token::DoubleColon) {
                self.advance();
                match self.current_token().clone() {
                    Token::Ident(right) => {
                        self.advance();
                        match expr {
                            Expr::Ident(left_str) => {
                                expr = Expr::Path { base: left_str, segment: right };
                            }
                            Expr::Path { base, segment } => {
                                expr = Expr::Ident(format!("{}::{}::{}", base, segment, right));
                            }
                            _ => break,
                        }
                    }
                    other => return Err(format!("Expected identifier after '::', found {}", other)),
                }
            } else if self.check(Token::LeftBrace) {
                let mut handled_call = false;
                if !self.suppress_call_suffix_block {
                    if let Expr::Call { callee: _, ref mut args } = expr {
                        let next = self.tokens.get(self.pos + 1).map(|t| &t.token);
                        let is_struct_init = matches!(next, Some(Token::RightBrace)) || (matches!(next, Some(Token::Ident(_))) && matches!(self.tokens.get(self.pos + 2).map(|t| &t.token), Some(Token::Colon)));
                        if !is_struct_init {
                            let block = self.parse_block()?;
                            args.push(Expr::Lambda { params: vec![], return_type: None, body: block });
                            handled_call = true;
                        }
                    }
                }
                if !handled_call {
                    if let Expr::Ident(name) = &expr {
                        let next = self.tokens.get(self.pos + 1).map(|t| &t.token);
                        let is_rightbrace = matches!(next, Some(Token::RightBrace));
                        let t2 = self.tokens.get(self.pos + 2).map(|t| &t.token);
                        let ident_then_colon = matches!(next, Some(Token::Ident(_))) && matches!(t2, Some(Token::Colon));
                        let ident_then_comma = matches!(next, Some(Token::Ident(_))) && matches!(t2, Some(Token::Comma));
                        if is_rightbrace || ident_then_colon || ident_then_comma {
                            let name = name.clone();
                            self.advance();
                            let mut fields = Vec::new();
                            while !self.check(Token::RightBrace) && !self.check(Token::Eof) {
                                let fname = match self.current_token().clone() {
                                    Token::Ident(s) => { self.advance(); s }
                                    Token::Return | Token::Let | Token::If | Token::While | Token::For | Token::Loop |
                                    Token::Break | Token::Continue | Token::Match => {
                                        break;
                                    }
                                    other => return Err(format!("Expected field name in struct init, found {}", other)),
                                };
                                if self.check(Token::Colon) {
                                    self.advance();
                                    let value = self.parse_expr()?;
                                    fields.push((fname, value));
                                } else {
                                    fields.push((fname.clone(), Expr::Ident(fname)));
                                }
                                if !self.eat(Token::Comma) { break; }
                            }
                            self.expect(Token::RightBrace)?;
                            expr = Expr::StructInit { name, fields };
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let token = self.current_token().clone();
        match token {
            Token::Int(n) => {
                self.advance();
                // 可选时间/单位后缀: 5s, 5min, 5h, 5ms, 5us, 5ns
                if let Some(&Token::Ident(ref unit)) = self.tokens.get(self.pos).map(|t| &t.token) {
                    if matches!(unit.as_str(), "s" | "min" | "h" | "ms" | "us" | "ns" | "Hz" | "kHz" | "MHz") {
                        self.advance();
                    }
                }
                Ok(Expr::Int(n))
            }
            Token::Float(n) => {
                self.advance();
                // 可选单位后缀
                if let Some(&Token::Ident(ref unit)) = self.tokens.get(self.pos).map(|t| &t.token) {
                    if matches!(unit.as_str(), "s" | "min" | "h" | "ms" | "us" | "ns" | "Hz" | "kHz" | "MHz") {
                        self.advance();
                    }
                }
                Ok(Expr::Float(n))
            }
            Token::Str(s) => { self.advance(); Ok(Expr::Str(s)) }
            Token::Bool(b) => { self.advance(); Ok(Expr::Bool(b)) }
            Token::None => { self.advance(); Ok(Expr::None) }
            Token::Ok => { self.advance(); Ok(Expr::Ident("ok".to_string())) }
            Token::Err => { self.advance(); Ok(Expr::Ident("err".to_string())) }
            Token::Some => { self.advance(); Ok(Expr::Ident("some".to_string())) }
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
                    // 仅当符合 `Name { field: ... }` 或 `Name {}` 或 `Name { f1, f2, ... }` 模式时才识别为 struct init
                    // 避免与块表达式冲突，例如 `match c { ... }` 中的 `c {` 或 `if cond { ... }`
                    let is_struct_init = {
                        let next = self.tokens.get(self.pos + 1).map(|t| &t.token);
                        match next {
                            Some(Token::RightBrace) => true,
                            Some(Token::Ident(_)) => {
                                let after = self.tokens.get(self.pos + 2).map(|t| &t.token);
                                matches!(after, Some(Token::Colon)) || matches!(after, Some(Token::Comma))
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
                                if self.check(Token::Colon) {
                                    self.advance();
                                    let field_val = self.parse_expr()?;
                                    fields.push((field_name, field_val));
                                } else {
                                    fields.push((field_name.clone(), Expr::Ident(field_name)));
                                }
                                if !self.eat(Token::Comma) { break; }
                                if self.check(Token::RightBrace) || self.check(Token::Eof) { break; }
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
            Token::Fn => self.parse_lambda(),
            Token::If => self.parse_if_expr(),
            Token::Loop => {
                self.advance();
                let body = self.parse_block()?;
                Ok(Expr::BlockExpr(Block { stmts: vec![Stmt::Loop(body)] }))
            }
            Token::While => {
                self.advance();
                self.suppress_call_suffix_block = true;
                let condition = self.parse_expr()?;
                self.suppress_call_suffix_block = false;
                let body = self.parse_block()?;
                Ok(Expr::BlockExpr(Block { stmts: vec![Stmt::While { condition, body }] }))
            }
            Token::LeftParen => {
                self.advance();
                if self.check(Token::RightParen) {
                    self.advance();
                    Ok(Expr::None)
                } else {
                    let first = self.parse_expr()?;
                    let mut elements = vec![first];
                    while self.eat(Token::Comma) {
                        if self.check(Token::RightParen) { break; }
                        elements.push(self.parse_expr()?);
                    }
                    self.expect(Token::RightParen)?;
                    if elements.len() == 1 {
                        Ok(elements.into_iter().next().unwrap())
                    } else {
                        Ok(Expr::Tuple(elements))
                    }
                }
            }
            Token::LeftBracket => {
                self.advance();
                let mut items = Vec::new();
                if !self.check(Token::RightBracket) {
                    let first = self.parse_expr()?;
                    if self.eat(Token::Semicolon) {
                        // [value; count] 形式: 重复 count 次
                        let count_expr = self.parse_expr()?;
                        let count = match count_expr {
                            Expr::Int(n) => n as usize,
                            _ => 0,
                        };
                        for _ in 0..count {
                            items.push(first.clone());
                        }
                    } else {
                        items.push(first);
                        while self.eat(Token::Comma) {
                            if self.check(Token::RightBracket) { break; }
                            items.push(self.parse_expr()?);
                        }
                    }
                }
                self.expect(Token::RightBracket)?;
                Ok(Expr::List(items))
            }
            Token::LeftBrace => {
                let block = self.parse_block()?;
                Ok(Expr::BlockExpr(block))
            }
            // 占位符语法: <expr> / <pipeline expr> 等,D类伪代码中用于语法模板
            // 解析为占位 None 表达式,仅保证 parser/sema 通过(实际不执行)
            Token::Lt => {
                self.advance();
                let mut depth = 1;
                while depth > 0 && !self.check(Token::Eof) {
                    match self.current_token() {
                        Token::Lt => { depth += 1; self.advance(); }
                        Token::Gt => { depth -= 1; if depth > 0 { self.advance(); } }
                        _ => { self.advance(); }
                    }
                }
                if self.check(Token::Gt) { self.advance(); }
                Ok(Expr::None)
            }
            _ => Err(format!("Unexpected token: {}", token)),
        }
    }

    fn parse_fn_decl(&mut self) -> Result<Stmt, String> {
        // 可选 async 前缀
        let is_async = self.eat(Token::Async);
        self.expect(Token::Fn)?;
        let name = match self.current_token().clone() {
            Token::Ident(ref s) => {
                let n = s.clone();
                self.advance();
                n
            }
            Token::Lt => {
                self.skip_angle_bracket_placeholder();
                "__placeholder_fn_name__".to_string()
            }
            other => return Err(format!("Expected function name, found {}", other)),
        };
        // 可选泛型参数 <T: Ord, U> - 简化吞到匹配 >
        if self.eat(Token::Lt) {
            let mut depth = 1;
            while depth > 0 && !self.check(Token::Eof) {
                if self.check(Token::Lt) { depth += 1; }
                if self.check(Token::Gt) { depth -= 1; }
                self.advance();
            }
        }
        self.expect(Token::LeftParen)?;
        let mut params = Vec::new();
        while !self.check(Token::RightParen) {
            if self.skip_ellipsis_if_present() {
                while !self.check(Token::RightParen) && !self.check(Token::Eof) {
                    self.advance();
                }
                break;
            }
            let param_name = match self.current_token().clone() {
                Token::Ident(ref s) => {
                    let n = s.clone();
                    self.advance();
                    // 必须有 : 类型
                    self.expect(Token::Colon)?;
                    let type_ann = self.parse_type_annotation()?;
                    params.push((n.clone(), type_ann));
                    n
                }
                Token::Lt => {
                    self.skip_angle_bracket_placeholder();
                    // 如果接下来是 : 则解析类型，否则当做整个参数（没有类型注解）
                    if self.eat(Token::Colon) {
                        let type_ann = self.parse_type_annotation()?;
                        params.push(("__placeholder_param__".to_string(), type_ann));
                    } else if self.check(Token::Comma) || self.check(Token::RightParen) {
                        // 没有类型，直接作为占位，不 push（或 push 默认类型）
                        params.push(("__placeholder_param__".to_string(), TypeAnnotation::Void));
                    } else {
                        // 可能还未跳过的内容，直接 skip 到 , 或 )
                        while !self.check(Token::Comma) && !self.check(Token::RightParen) && !self.check(Token::Eof) {
                            self.advance();
                        }
                        params.push(("__placeholder_param__".to_string(), TypeAnnotation::Void));
                    }
                    "__placeholder_param__".to_string()
                }
                other => return Err(format!("Expected parameter name, found {}", other)),
            };
            if !self.eat(Token::Comma) { break; }
        }
        self.expect(Token::RightParen)?;
        let return_type = if self.eat(Token::Arrow) {
            Some(self.parse_type_annotation()?)
        } else { None };
        // 跳过可能的 :> SubtypeConstraint（如 :> Normalized）
        if self.check(Token::Colon) {
            let saved_pos = self.pos;
            self.advance();
            if self.check(Token::Gt) {
                self.advance();
                while !self.check(Token::LeftBrace) && !self.check(Token::Semicolon) && !self.check(Token::Eof) {
                    self.advance();
                }
            } else {
                self.pos = saved_pos;
            }
        }
        let body = if self.eat(Token::Semicolon) {
            Block { stmts: Vec::new() }
        } else {
            self.parse_block()?
        };
        Ok(Stmt::FnDecl { name, params, return_type, body, is_async })
    }

    /// 解析匿名函数表达式: `fn(x: i64) -> i64 { ... }` 或 `fn(x) { ... }`
    fn parse_lambda(&mut self) -> Result<Expr, String> {
        self.expect(Token::Fn)?;
        self.expect(Token::LeftParen)?;
        let mut params = Vec::new();
        while !self.check(Token::RightParen) {
            let param_name = if let Token::Ident(ref s) = self.current_token() {
                s.clone()
            } else {
                return Err(format!("Expected parameter name, found {}", self.current_token()));
            };
            self.advance();
            // 类型标注可选: `x: i64` 或 `x`
            let type_ann = if self.eat(Token::Colon) {
                self.parse_type_annotation()?
            } else {
                TypeAnnotation::Named("i64".to_string())
            };
            params.push((param_name, type_ann));
            if !self.eat(Token::Comma) { break; }
        }
        self.expect(Token::RightParen)?;
        let return_type = if self.eat(Token::Arrow) {
            Some(self.parse_type_annotation()?)
        } else { None };
        let body = self.parse_block()?;
        Ok(Expr::Lambda { params, return_type, body })
    }

    fn parse_let_decl(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Let)?;
        // 可选 `mut` 关键字 (v0.1 中可变性不强制,仅语法兼容)
        self.eat(Token::Mut);
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

    fn block_to_expr(&self, block: Block) -> Expr {
        if block.stmts.len() == 1 {
            if let Stmt::Expr(e) = &block.stmts[0] {
                return e.clone();
            }
        }
        Expr::BlockExpr(block)
    }

    fn parse_if_expr(&mut self) -> Result<Expr, String> {
        self.expect(Token::If)?;
        self.suppress_call_suffix_block = true;
        let condition = self.parse_expr()?;
        self.suppress_call_suffix_block = false;
        let then_block = self.parse_block()?;
        if !self.eat(Token::Else) {
            return Err("'if' expression requires an else branch".to_string());
        }
        let else_value: Box<Expr> = if self.check(Token::If) {
            Box::new(self.parse_if_expr()?)
        } else {
            let else_block = self.parse_block()?;
            Box::new(self.block_to_expr(else_block))
        };
        let then_value = Box::new(self.block_to_expr(then_block));
        Ok(Expr::IfExpr { condition: Box::new(condition), then_value, else_value })
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.expect(Token::If)?;
        self.suppress_call_suffix_block = true;
        let condition = self.parse_expr()?;
        self.suppress_call_suffix_block = false;
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
        self.suppress_call_suffix_block = true;
        let condition = self.parse_expr()?;
        self.suppress_call_suffix_block = false;
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
        self.suppress_call_suffix_block = true;
        let first_expr = self.parse_expr()?;
        self.suppress_call_suffix_block = false;
        if self.check(Token::Dot) {
            let saved_pos = self.pos;
            self.advance();
            if self.check(Token::Dot) {
                self.advance();
                self.suppress_call_suffix_block = true;
                let end = self.parse_expr()?;
                self.suppress_call_suffix_block = false;
                let body = self.parse_block()?;
                return Ok(Stmt::For { var_name, start: first_expr, end, body });
            } else {
                self.pos = saved_pos;
            }
        }
        let body = self.parse_block()?;
        Ok(Stmt::ForIterable { var_name, iterable: first_expr, body })
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

    /// 解析结构体声明: `struct Name { field: T, ... }` or `struct Name<T> { ... }`
    fn parse_struct_decl(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Struct)?;
        let name = match self.current_token().clone() {
            Token::Ident(s) => { self.advance(); s }
            other => return Err(format!("Expected struct name, found {}", other)),
        };
        // 可选泛型参数: <T, U, const N: usize> - 简化处理，吞到匹配 >
        if self.eat(Token::Lt) {
            let mut depth = 1;
            while depth > 0 && !self.check(Token::Eof) {
                if self.check(Token::Lt) { depth += 1; }
                if self.check(Token::Gt) { depth -= 1; }
                self.advance();
            }
        }
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
            let has_comma = self.eat(Token::Comma);
            let has_semicolon = self.eat(Token::Semicolon);
            if !has_comma && !has_semicolon { break; }
            if self.check(Token::RightBrace) || self.check(Token::Eof) { break; }
        }
        self.expect(Token::RightBrace)?;
        Ok(Stmt::StructDecl { name, fields })
    }

    /// 解析枚举声明: `enum Name { Variant, Variant2(T, T), ... }` or `enum Name<T> { ... }`
    fn parse_enum_decl(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Enum)?;
        let name = match self.current_token().clone() {
            Token::Ident(s) => { self.advance(); s }
            other => return Err(format!("Expected enum name, found {}", other)),
        };
        // 可选泛型参数: <T> - 简化处理，吞到匹配 >
        if self.eat(Token::Lt) {
            let mut depth = 1;
            while depth > 0 && !self.check(Token::Eof) {
                if self.check(Token::Lt) { depth += 1; }
                if self.check(Token::Gt) { depth -= 1; }
                self.advance();
            }
        }
        self.expect(Token::LeftBrace)?;
        let mut variants = Vec::new();
        while !self.check(Token::RightBrace) && !self.check(Token::Eof) {
            let variant_name = match self.current_token().clone() {
                Token::Ident(s) => { self.advance(); s }
                other => return Err(format!("Expected variant name, found {}", other)),
            };
            // 可选的载荷类型列表: (T, T, ...) or (name: T, ...)
            let payload = if self.check(Token::LeftParen) {
                self.advance();
                let mut types = Vec::new();
                if !self.check(Token::RightParen) {
                    // 支持命名: name: T 或匿名: T - save+restore 方式
                    let saved = self.pos;
                    let mut named_style = false;
                    if let Token::Ident(_) = self.current_token().clone() {
                        self.advance(); // name
                        if self.eat(Token::Colon) {
                            named_style = true;
                        }
                    }
                    if !named_style {
                        self.pos = saved;
                    }
                    types.push(self.parse_type_annotation()?);
                    while self.eat(Token::Comma) {
                        if self.check(Token::RightParen) { break; }
                        let saved2 = self.pos;
                        let mut named2 = false;
                        if let Token::Ident(_) = self.current_token().clone() {
                            self.advance();
                            if self.eat(Token::Colon) { named2 = true; }
                        }
                        if !named2 { self.pos = saved2; }
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
            if self.check(Token::RightBrace) || self.check(Token::Eof) { break; }
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
    /// `description` 与 `source` / `sample` 字段都是可选的;`pipeline:` 必须存在或简写形式。
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

        // 检查是否是简写 flow { 表达式/语句 }（非关键字开头）
        let is_shorthand = !matches!(self.current_token(), Token::Source | Token::Sample | Token::Pipeline | Token::RightBrace | Token::Eof);
        if is_shorthand {
            // 简写: 支持语句块（flow x { let a = 1; a + 2 }）或单表达式
            let mut stmts: Vec<Stmt> = Vec::new();
            let stmt_start_tokens: &[Token] = &[
                Token::Let, Token::Fn, Token::If, Token::While, Token::For,
                Token::Loop, Token::Return, Token::Break, Token::Continue,
            ];
            while !self.check(Token::RightBrace) && !self.check(Token::Eof) {
                let cur = self.current_token().clone();
                if stmt_start_tokens.contains(&cur) {
                    stmts.push(self.parse_stmt()?);
                    self.eat(Token::Semicolon);
                } else {
                    // 表达式作为最后一条
                    let expr = self.parse_expr()?;
                    stmts.push(Stmt::Expr(expr));
                    self.eat(Token::Semicolon);
                }
            }
            // 取最后一个表达式（或语句块）作为 pipeline
            let last_expr = if stmts.is_empty() {
                Expr::None
            } else if let Stmt::Expr(e) = stmts.remove(stmts.len() - 1) {
                e
            } else {
                // 如果最后一条是非表达式语句，用 BlockExpr 包装全部
                let block = Block { stmts };
                Expr::BlockExpr(block)
            };
            pipeline = Some(last_expr);
        } else {
            while !self.check(Token::RightBrace) && !self.check(Token::Eof) {
                match self.current_token().clone() {
                    Token::Source => {
                        self.advance();
                        self.expect(Token::Colon)?;
                        // 可选: source: <TypeAnnotation> = <expr>
                        // 先尝试解析 TypeAnnotation, 如果有 = 号
                        let saved = self.pos;
                        let maybe_type = self.parse_type_annotation();
                        if maybe_type.is_ok() && self.check(Token::Assign) {
                            // 有类型+赋值, 跳过 TypeAnnotation（已解析）和 = 号
                            self.advance(); // =
                            let expr = self.parse_expr()?;
                            self.eat(Token::Semicolon);
                            source = Some(expr);
                        } else {
                            // 无类型或解析失败, 回退为直接 parse_expr
                            self.pos = saved;
                            let expr = self.parse_expr()?;
                            self.eat(Token::Semicolon);
                            source = Some(expr);
                        }
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
                    Token::Ident(field_name) => {
                        // 非关键字字段: on_player_join(p) { ... } 或 field: Type = value
                        // 简化处理：吞到分号或匹配右花括号后（嵌套 {} 保护）
                        self.advance(); // field_name
                        // 如果接下来是 ( 则可能是事件处理器函数
                        if self.check(Token::LeftParen) {
                            // 事件处理器: on_xxx(args) { ... } - 吞到匹配的 }
                            // 先吞 ( ... )
                            self.advance(); // (
                            let mut depth = 1;
                            while depth > 0 && !self.check(Token::Eof) {
                                match self.current_token() {
                                    Token::LeftParen => { depth += 1; self.advance(); }
                                    Token::RightParen => { depth -= 1; if depth > 0 { self.advance(); } }
                                    _ => { self.advance(); }
                                }
                            }
                            self.eat(Token::RightParen);
                        } else if self.check(Token::Colon) {
                            // field: <maybeType> = <value>;
                            self.advance(); // :
                            let _ = self.parse_type_annotation();
                            if self.check(Token::Assign) { self.advance(); let _ = self.parse_expr(); }
                            self.eat(Token::Semicolon);
                            continue;
                        }
                        // 如果接下来是 { 吞到匹配的 }
                        if self.check(Token::LeftBrace) {
                            self.advance();
                            let mut depth = 1;
                            while depth > 0 && !self.check(Token::Eof) {
                                match self.current_token() {
                                    Token::LeftBrace => { depth += 1; self.advance(); }
                                    Token::RightBrace => { depth -= 1; if depth > 0 { self.advance(); } }
                                    _ => { self.advance(); }
                                }
                            }
                            self.eat(Token::RightBrace);
                        }
                        self.eat(Token::Semicolon);
                    }
                    other => return Err(format!("Expected 'source' / 'sample' / 'pipeline' in flow block, found {}", other)),
                }
            }
        }
        self.expect(Token::RightBrace)?;

        if !is_shorthand {
            if pipeline.is_none() {
                return Err(format!("missing 'pipeline:' in flow block '{}'", name));
            }
        }

        // 允许简写 flow 中没有 pipeline（此时用 source 替代，或用 Expr::None 占位）
        let pipeline = pipeline.unwrap_or_else(|| source.clone().unwrap_or(Expr::None));
        Ok(Stmt::FlowDecl { name, description, source, pipeline })
    }

    /// 解析模块声明: `module foo::bar::baz;`
    fn parse_mod_decl(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Mod)?;
        let mut parts = Vec::new();
        // 第一个标识符
        match self.current_token().clone() {
            Token::Ident(s) => { parts.push(s); self.advance(); }
            other => return Err(format!("Expected module name after 'mod', found {}", other)),
        }
        // 后续 `::ident`
        while self.check(Token::DoubleColon) {
            self.advance();
            match self.current_token().clone() {
                Token::Ident(s) => { parts.push(s); self.advance(); }
                other => return Err(format!("Expected identifier after '::' in module path, found {}", other)),
            }
        }
        self.expect(Token::Semicolon)?;
        Ok(Stmt::ModDecl { name: parts.join("::") })
    }

    /// 解析导入声明: `use foo::bar::baz;` 或 `use foo::bar as baz;`
    fn parse_use_decl(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Use)?;
        let mut path = Vec::new();
        match self.current_token().clone() {
            Token::Ident(s) => { path.push(s); self.advance(); }
            other => return Err(format!("Expected identifier after 'use', found {}", other)),
        }
        while self.check(Token::DoubleColon) {
            self.advance();
            match self.current_token().clone() {
                Token::Ident(s) => { path.push(s); self.advance(); }
                Token::LeftBrace => {
                    // 分组导入: use a::b::{X, Y, Z}
                    self.advance();
                    while !self.check(Token::RightBrace) && !self.check(Token::Eof) {
                        if let Token::Ident(s2) = self.current_token().clone() {
                            self.advance();
                            // 可选 as alias
                            if self.check(Token::As) { self.advance(); if let Token::Ident(_) = self.current_token().clone() { self.advance(); } }
                        }
                        if !self.eat(Token::Comma) { break; }
                    }
                    self.expect(Token::RightBrace)?;
                    break;
                }
                other => return Err(format!("Expected identifier after '::' in use path, found {}", other)),
            }
        }
        // 可选 `as alias`
        let alias = if self.check(Token::As) {
            self.advance();
            match self.current_token().clone() {
                Token::Ident(s) => { self.advance(); Some(s) }
                other => return Err(format!("Expected identifier after 'as', found {}", other)),
            }
        } else {
            None
        };
        self.expect(Token::Semicolon)?;
        Ok(Stmt::UseDecl { path, alias })
    }

    /// 解析域声明: `domain Name { key: value, key2: value2 }`
    fn parse_domain_decl(&mut self) -> Result<Stmt, String> {
        self.expect(Token::Domain)?;
        let name = match self.current_token().clone() {
            Token::Ident(s) => { self.advance(); s }
            other => return Err(format!("Expected domain name after 'domain', found {}", other)),
        };
        self.expect(Token::LeftBrace)?;
        let mut config = Vec::new();
        while !self.check(Token::RightBrace) {
            // 解析 key: value
            let key = match self.current_token().clone() {
                Token::Ident(s) => { self.advance(); s }
                Token::Async | Token::Await | Token::Break | Token::Continue |
                Token::Else | Token::Enum | Token::Extern | Token::Export |
                Token::False | Token::Flow | Token::Fn | Token::For |
                Token::If | Token::In | Token::Let | Token::Loop |
                Token::Match | Token::Mod | Token::Mut | Token::None |
                Token::Return | Token::Struct | Token::True | Token::Use |
                Token::While | Token::Domain | Token::Stream | Token::Source |
                Token::Pipeline | Token::Sample => {
                    let s = format!("{}", self.current_token());
                    self.advance();
                    s
                }
                other => return Err(format!("Expected config key in domain block, found {}", other)),
            };
            self.expect(Token::Colon)?;
            let value = self.parse_expr()?;
            config.push((key, value));
            // 可选的逗号或分号分隔
            if self.check(Token::Comma) || self.check(Token::Semicolon) {
                self.advance();
            }
        }
        self.expect(Token::RightBrace)?;
        Ok(Stmt::DomainDecl { name, config })
    }

    /// 解析 match 表达式
    fn parse_match_expr(&mut self) -> Result<Expr, String> {
        self.expect(Token::Match)?;
        self.suppress_call_suffix_block = true;
        let scrutinee = self.parse_expr()?;
        self.suppress_call_suffix_block = false;
        self.expect(Token::LeftBrace)?;
        let mut arms = Vec::new();
        while !self.check(Token::RightBrace) && !self.check(Token::Eof) {
            let pattern = self.parse_pattern()?;
            self.expect(Token::FatArrow)?;
            // 支持单行 arm: => expr (无大括号) 或 => { block }
            let body = if self.check(Token::LeftBrace) {
                self.parse_block()?
            } else {
                // 单行表达式 arm: 解析到逗号/右花括号为止, 包装成 Block
                let saved = self.pos;
                let mut stmts = Vec::new();
                // 表达式可能包含逗号 (如 println("a", b)), 因此需要小心:
                // 用简单策略: parse_expr() 直到表达式完成
                let expr = self.parse_expr()?;
                stmts.push(Stmt::Expr(expr));
                Block { stmts }
            };
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
            Token::Ok | Token::Err | Token::Some => {
                let name = format!("{}", token);
                self.advance();
                // ok(err(some) 都当作 Ident(name) 来处理: 支持 ok(v) / Err(e) / Some(x) 构造
                if self.check(Token::DoubleColon) {
                    self.advance();
                    let variant = match self.current_token().clone() {
                        Token::Ident(s) => { self.advance(); s }
                        other => return Err(format!("Expected variant name after '::', found {}", other)),
                    };
                    if self.check(Token::LeftParen) {
                        self.advance();
                        let mut bindings = Vec::new();
                        if !self.check(Token::RightParen) {
                            loop {
                                match self.current_token().clone() {
                                    Token::Ident(s) => { self.advance(); bindings.push(s) }
                                    Token::Underscore => { self.advance(); bindings.push("_".to_string()) }
                                    Token::Ok | Token::Err | Token::Some => {
                                        let s = format!("{}", self.current_token());
                                        self.advance();
                                        bindings.push(s);
                                    }
                                    other => return Err(format!("Expected binding in pattern, found {}", other)),
                                }
                                if !self.eat(Token::Comma) { break; }
                                if self.check(Token::RightParen) || self.check(Token::Eof) { break; }
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
                } else if self.check(Token::LeftParen) {
                    // 构造函数/元组结构模式: Variant(bindings...) 或 Ok(v)/Err(e)
                    self.advance();
                    let mut bindings = Vec::new();
                    if !self.check(Token::RightParen) {
                        loop {
                            match self.current_token().clone() {
                                Token::Ident(s) => { self.advance(); bindings.push(s) }
                                Token::Underscore => { self.advance(); bindings.push("_".to_string()) }
                                Token::Ok | Token::Err | Token::Some => {
                                    let s = format!("{}", self.current_token());
                                    self.advance();
                                    bindings.push(s);
                                }
                                other => return Err(format!("Expected binding in pattern, found {}", other)),
                            }
                            if !self.eat(Token::Comma) { break; }
                            if self.check(Token::RightParen) || self.check(Token::Eof) { break; }
                        }
                    }
                    self.expect(Token::RightParen)?;
                    Ok(Pattern::EnumVariantWithPayload {
                        type_name: String::new(),
                        variant: name,
                        bindings,
                    })
                } else {
                    // 单纯标识符: 绑定变量
                    Ok(Pattern::Bind(name))
                }
            }
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
                                if self.check(Token::RightParen) || self.check(Token::Eof) { break; }
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
                } else if self.check(Token::LeftParen) {
                    // 构造函数/元组结构模式: Variant(bindings...) 或 Ok(v)/Err(e)
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
                            if self.check(Token::RightParen) || self.check(Token::Eof) { break; }
                        }
                    }
                    self.expect(Token::RightParen)?;
                    Ok(Pattern::EnumVariantWithPayload {
                        type_name: String::new(),
                        variant: name,
                        bindings,
                    })
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
        // 指针类型开头: *mut T / *T
        if matches!(token, Token::Star) {
            self.advance();
            let _is_mut = self.eat(Token::Mut);
            let inner = self.parse_type_annotation()?;
            return Ok(TypeAnnotation::Ptr(Box::new(inner)));
        }
        let inner = match token {
            Token::Fn => {
                // 函数指针类型: fn(x: f32, y: f32) -> f32 或 fn() -> void
                self.advance();
                self.expect(Token::LeftParen)?;
                while !self.check(Token::RightParen) && !self.check(Token::Eof) {
                    // 吞参数 (可能 x: Type 或只有 Type)
                    if let Token::Ident(_) = self.current_token().clone() {
                        let saved = self.pos;
                        self.advance();
                        if !self.check(Token::Colon) {
                            self.pos = saved; // 只有 Type, 不是 name: Type
                            let _ = self.parse_type_annotation();
                        } else {
                            self.advance(); // :
                            let _ = self.parse_type_annotation();
                        }
                    } else {
                        let _ = self.parse_type_annotation();
                    }
                    if !self.eat(Token::Comma) { break; }
                }
                self.expect(Token::RightParen)?;
                if self.eat(Token::Arrow) {
                    let _ret = self.parse_type_annotation();
                }
                return Ok(TypeAnnotation::Named("fn".to_string()));
            }
            Token::Ampersand => {
                self.advance();
                let mut is_mut = false;
                if self.eat(Token::Mut) {
                    is_mut = true;
                }
                let inner = self.parse_type_annotation()?;
                return Ok(TypeAnnotation::Ref(Box::new(inner), is_mut));
            }
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
                if self.check(Token::RightParen) {
                    self.advance();
                    Ok(TypeAnnotation::Unit)
                } else {
                    let first = self.parse_type_annotation()?;
                    let mut elements = vec![first];
                    while self.eat(Token::Comma) {
                        if self.check(Token::RightParen) { break; }
                        elements.push(self.parse_type_annotation()?);
                    }
                    self.expect(Token::RightParen)?;
                    if elements.len() == 1 {
                        Ok(elements.into_iter().next().unwrap())
                    } else {
                        Ok(TypeAnnotation::Tuple(elements))
                    }
                }
            }
            Token::LeftBracket => {
                // 数组类型: [T; N]
                self.advance();
                let elem_type = self.parse_type_annotation()?;
                let size = if self.eat(Token::Semicolon) {
                    match self.current_token().clone() {
                        Token::Int(n) => { self.advance(); n as u64 }
                        _ => {
                            // 非字面量大小：吞到 ] 为止，默认 0
                            let mut sz = 0u64;
                            while !self.check(Token::RightBracket) && !self.check(Token::Eof) {
                                self.advance();
                            }
                            sz
                        }
                    }
                } else { 0 };
                self.expect(Token::RightBracket)?;
                Ok(TypeAnnotation::Array(Box::new(elem_type), size))
            }
            Token::Lt => {
                // D类占位符: <type> 占位语法
                self.skip_angle_bracket_placeholder();
                Ok(TypeAnnotation::Named("__Placeholder".to_string()))
            }
            Token::Dot => {
                // D类占位符: ... 返回类型
                self.skip_ellipsis_if_present();
                Ok(TypeAnnotation::Named("__Ellipsis".to_string()))
            }
            _ => Err(format!("Expected type annotation, found {}", token)),
        }?;

        // 处理泛型: Named<T> / Named<K, V>
        let inner = if self.check(Token::Lt) {
            self.advance();
            let mut args = Vec::new();
            if !self.check(Token::Gt) {
                args.push(self.parse_type_annotation()?);
                while self.eat(Token::Comma) {
                    if self.check(Token::Gt) { break; }
                    args.push(self.parse_type_annotation()?);
                }
            }
            self.expect(Token::Gt)?;
            TypeAnnotation::Generic(Box::new(inner), args)
        } else {
            inner
        };

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

    // ===== Phase 2.12: 模块系统测试 =====

    #[test]
    fn test_parse_mod_decl_simple() {
        let program = parse("mod foo;").unwrap();
        let Program::Block(stmts) = program;
        assert!(matches!(&stmts[0], Stmt::ModDecl { name } if name == "foo"));
    }

    #[test]
    fn test_parse_mod_decl_nested() {
        let program = parse("mod foo::bar::baz;").unwrap();
        let Program::Block(stmts) = program;
        assert!(matches!(&stmts[0], Stmt::ModDecl { name } if name == "foo::bar::baz"));
    }

    #[test]
    fn test_parse_use_decl_simple() {
        let program = parse("use foo;").unwrap();
        let Program::Block(stmts) = program;
        assert!(matches!(&stmts[0], Stmt::UseDecl { path, alias: None } if path == &vec!["foo".to_string()]));
    }

    #[test]
    fn test_parse_use_decl_nested() {
        let program = parse("use foo::bar::baz;").unwrap();
        let Program::Block(stmts) = program;
        assert!(matches!(&stmts[0], Stmt::UseDecl { path, alias: None } if path.len() == 3 && path[0] == "foo" && path[2] == "baz"));
    }

    #[test]
    fn test_parse_use_decl_with_alias() {
        let program = parse("use foo::bar as baz;").unwrap();
        let Program::Block(stmts) = program;
        assert!(matches!(&stmts[0], Stmt::UseDecl { path, alias: Some(a) } if path.len() == 2 && a == "baz"));
    }

    #[test]
    fn test_parse_let_mut() {
        let program = parse("let mut x = 42;").unwrap();
        let Program::Block(stmts) = program;
        assert!(matches!(&stmts[0], Stmt::LetDecl { name, .. } if name == "x"));
    }

    #[test]
    fn test_parse_let_mut_with_type() {
        let program = parse("let mut x: i64 = 42;").unwrap();
        let Program::Block(stmts) = program;
        assert!(matches!(&stmts[0], Stmt::LetDecl { name, type_annotation: Some(_), .. } if name == "x"));
    }

    #[test]
    fn test_parse_mod_missing_semicolon_errors() {
        let result = parse("mod foo");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_use_missing_name_errors() {
        let result = parse("use ;");
        assert!(result.is_err());
    }
}
