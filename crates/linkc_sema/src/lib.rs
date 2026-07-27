use linkc_parser::*;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct SemaError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for SemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error: {} (line {}, col {})", self.message, self.line, self.col)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SemaType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    USize,
    F32,
    F64,
    Bool,
    Str,
    Unit,
    Void,
    Ptr(Box<SemaType>),
    Named(String),
    List(Box<SemaType>),
    Stream(Box<SemaType>),
    Function {
        params: Vec<SemaType>,
        ret: Box<SemaType>,
    },
    Unknown,
}

impl SemaType {
    pub fn from_annotation(ann: &TypeAnnotation) -> Self {
        match ann {
            TypeAnnotation::I8 => SemaType::I8,
            TypeAnnotation::I16 => SemaType::I16,
            TypeAnnotation::I32 => SemaType::I32,
            TypeAnnotation::I64 => SemaType::I64,
            TypeAnnotation::U8 => SemaType::U8,
            TypeAnnotation::U16 => SemaType::U16,
            TypeAnnotation::U32 => SemaType::U32,
            TypeAnnotation::U64 => SemaType::U64,
            TypeAnnotation::USize => SemaType::USize,
            TypeAnnotation::F32 => SemaType::F32,
            TypeAnnotation::F64 => SemaType::F64,
            TypeAnnotation::Bool => SemaType::Bool,
            TypeAnnotation::Str => SemaType::Str,
            TypeAnnotation::Unit => SemaType::Unit,
            TypeAnnotation::Void => SemaType::Void,
            TypeAnnotation::Ptr(inner) => SemaType::Ptr(Box::new(SemaType::from_annotation(inner))),
            TypeAnnotation::Named(name) => SemaType::Named(name.clone()),
            TypeAnnotation::Stream(inner) => SemaType::Stream(Box::new(SemaType::from_annotation(inner))),
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, SemaType::I8 | SemaType::I16 | SemaType::I32 | SemaType::I64
                | SemaType::U8 | SemaType::U16 | SemaType::U32 | SemaType::U64 | SemaType::USize)
    }

    pub fn is_float(&self) -> bool {
        matches!(self, SemaType::F32 | SemaType::F64)
    }

    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, SemaType::Bool)
    }

    pub fn is_compatible_with(&self, other: &SemaType) -> bool {
        if self == other {
            return true;
        }
        if self.is_numeric() && other.is_numeric() {
            return true;
        }
        matches!((self, other), (SemaType::Unknown, _) | (_, SemaType::Unknown))
    }

    pub fn default_for_literal(expr: &Expr) -> Self {
        match expr {
            Expr::Int(_) => SemaType::I64,
            Expr::Float(_) => SemaType::F64,
            Expr::Bool(_) => SemaType::Bool,
            Expr::Str(_) => SemaType::Str,
            Expr::None => SemaType::Unit,
            _ => SemaType::Unknown,
        }
    }
}

impl fmt::Display for SemaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemaType::I8 => write!(f, "i8"),
            SemaType::I16 => write!(f, "i16"),
            SemaType::I32 => write!(f, "i32"),
            SemaType::I64 => write!(f, "i64"),
            SemaType::U8 => write!(f, "u8"),
            SemaType::U16 => write!(f, "u16"),
            SemaType::U32 => write!(f, "u32"),
            SemaType::U64 => write!(f, "u64"),
            SemaType::USize => write!(f, "usize"),
            SemaType::F32 => write!(f, "f32"),
            SemaType::F64 => write!(f, "f64"),
            SemaType::Bool => write!(f, "bool"),
            SemaType::Str => write!(f, "str"),
            SemaType::Unit => write!(f, "unit"),
            SemaType::Void => write!(f, "void"),
            SemaType::Ptr(inner) => write!(f, "*{}", inner),
            SemaType::Named(name) => write!(f, "{}", name),
            SemaType::List(inner) => write!(f, "[{}]", inner),
            SemaType::Stream(inner) => write!(f, "stream<{}>", inner),
            SemaType::Function { params, ret } => {
                let params_str: Vec<String> = params.iter().map(|p| p.to_string()).collect();
                write!(f, "fn({}) -> {}", params_str.join(", "), ret)
            }
            SemaType::Unknown => write!(f, "unknown"),
        }
    }
}

pub struct TypeChecker {
    errors: Vec<SemaError>,
    var_types: Vec<HashMap<String, SemaType>>,
    fn_signatures: HashMap<String, SemaType>,
    struct_fields: HashMap<String, Vec<(String, SemaType)>>,
    enum_variants: HashMap<String, Vec<(String, Vec<SemaType>)>>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut fn_signatures = HashMap::new();

        fn_signatures.insert("print".to_string(), SemaType::Function {
            params: vec![SemaType::Unknown],
            ret: Box::new(SemaType::Void),
        });
        fn_signatures.insert("println".to_string(), SemaType::Function {
            params: vec![SemaType::Unknown],
            ret: Box::new(SemaType::Void),
        });
        fn_signatures.insert("len".to_string(), SemaType::Function {
            params: vec![SemaType::Unknown],
            ret: Box::new(SemaType::I64),
        });
        fn_signatures.insert("sleep".to_string(), SemaType::Function {
            params: vec![SemaType::I64],
            ret: Box::new(SemaType::Void),
        });
        fn_signatures.insert("stream".to_string(), SemaType::Function {
            params: vec![SemaType::Unknown],
            ret: Box::new(SemaType::Stream(Box::new(SemaType::Unknown))),
        });
        fn_signatures.insert("map".to_string(), SemaType::Function {
            params: vec![SemaType::Unknown, SemaType::Unknown],
            ret: Box::new(SemaType::Stream(Box::new(SemaType::Unknown))),
        });
        fn_signatures.insert("filter".to_string(), SemaType::Function {
            params: vec![SemaType::Unknown, SemaType::Unknown],
            ret: Box::new(SemaType::Stream(Box::new(SemaType::Unknown))),
        });
        fn_signatures.insert("for_each".to_string(), SemaType::Function {
            params: vec![SemaType::Unknown, SemaType::Unknown],
            ret: Box::new(SemaType::Void),
        });
        fn_signatures.insert("collect".to_string(), SemaType::Function {
            params: vec![SemaType::Unknown],
            ret: Box::new(SemaType::List(Box::new(SemaType::Unknown))),
        });

        Self {
            errors: Vec::new(),
            var_types: vec![HashMap::new()],
            fn_signatures,
            struct_fields: HashMap::new(),
            enum_variants: HashMap::new(),
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Vec<SemaError> {
        let Program::Block(stmts) = program;

        for stmt in stmts {
            self.check_toplevel_stmt(stmt);
        }

        self.errors.clone()
    }

    fn push_scope(&mut self) {
        self.var_types.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.var_types.pop();
    }

    fn lookup_var(&self, name: &str) -> Option<SemaType> {
        for scope in self.var_types.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(t.clone());
            }
        }
        None
    }

    fn declare_var(&mut self, name: &str, ty: SemaType) {
        if let Some(scope) = self.var_types.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    fn bind_pattern_vars(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Wildcard => {}
            Pattern::Literal(_) => {}
            Pattern::Bind(name) => {
                self.declare_var(name, SemaType::Unknown);
            }
            Pattern::EnumVariant { .. } => {}
            Pattern::EnumVariantWithPayload { bindings, .. } => {
                for name in bindings {
                    self.declare_var(name, SemaType::Unknown);
                }
            }
        }
    }

    fn error(&mut self, message: String, line: usize, col: usize) {
        self.errors.push(SemaError { message, line, col });
    }

    fn check_toplevel_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::StructDecl { name, fields } => {
                let mut field_types = Vec::new();
                for field in fields {
                    field_types.push((field.name.clone(), SemaType::from_annotation(&field.type_ann)));
                }
                self.struct_fields.insert(name.clone(), field_types);
            }
            Stmt::EnumDecl { name, variants } => {
                let mut variant_types = Vec::new();
                for variant in variants {
                    let payload: Vec<SemaType> = variant.payload.iter()
                        .map(|p| SemaType::from_annotation(p))
                        .collect();
                    variant_types.push((variant.name.clone(), payload));
                }
                self.enum_variants.insert(name.clone(), variant_types);
            }
            Stmt::FnDecl { name, params, return_type, body, .. } => {
                let param_types: Vec<SemaType> = params.iter()
                    .map(|(_, t)| SemaType::from_annotation(t))
                    .collect();
                let ret_type = if let Some(rt) = return_type {
                    SemaType::from_annotation(rt)
                } else {
                    SemaType::Void
                };
                let ret_type_clone = ret_type.clone();

                let fn_type = SemaType::Function {
                    params: param_types,
                    ret: Box::new(ret_type_clone),
                };
                self.fn_signatures.insert(name.clone(), fn_type);

                self.push_scope();
                for (pname, ptype) in params {
                    self.declare_var(pname, SemaType::from_annotation(ptype));
                }

                let body_ret = self.infer_block_return_type(body);
                if body_ret != SemaType::Unknown && body_ret != ret_type && ret_type != SemaType::Void {
                    if !body_ret.is_compatible_with(&ret_type) {
                        self.error(
                            format!("function '{}' returns {}, but declared return type is {}",
                                name, body_ret, ret_type),
                            1, 1
                        );
                    }
                }

                self.pop_scope();
            }
            Stmt::ExternDecl { decls, .. } => {
                for decl in decls {
                    let param_types: Vec<SemaType> = decl.params.iter()
                        .map(|(_, t)| SemaType::from_annotation(t))
                        .collect();
                    let ret_type = if let Some(rt) = &decl.return_type {
                        SemaType::from_annotation(rt)
                    } else {
                        SemaType::Void
                    };
                    let fn_type = SemaType::Function {
                        params: param_types,
                        ret: Box::new(ret_type),
                    };
                    self.fn_signatures.insert(decl.name.clone(), fn_type);
                }
            }
            Stmt::LetDecl { name, type_annotation, value } => {
                let inferred = if let Some(val) = value {
                    self.infer_expr(val)
                } else {
                    SemaType::Unknown
                };

                let declared = if let Some(ta) = type_annotation {
                    SemaType::from_annotation(ta)
                } else {
                    inferred.clone()
                };

                if inferred != SemaType::Unknown && declared != SemaType::Unknown {
                    if !inferred.is_compatible_with(&declared) {
                        self.error(
                            format!("variable '{}' has type {}, but assigned {}", name, declared, inferred),
                            1, 1
                        );
                    }
                }

                self.declare_var(name, declared);
            }
            Stmt::Expr(expr) => {
                self.infer_expr(expr);
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let cond_type = self.infer_expr(condition);
                if !cond_type.is_bool() && cond_type != SemaType::Unknown {
                    self.error(format!("if condition must be bool, got {}", cond_type), 0, 0);
                }
                self.push_scope();
                self.infer_block_return_type(then_branch);
                self.pop_scope();
                if let Some(else_block) = else_branch {
                    self.push_scope();
                    self.infer_block_return_type(else_block);
                    self.pop_scope();
                }
            }
            Stmt::While { condition, body } => {
                let cond_type = self.infer_expr(condition);
                if !cond_type.is_bool() && cond_type != SemaType::Unknown {
                    self.error(format!("while condition must be bool, got {}", cond_type), 0, 0);
                }
                self.push_scope();
                self.infer_block_return_type(body);
                self.pop_scope();
            }
            Stmt::For { var_name, start, end, body } => {
                self.infer_expr(start);
                self.infer_expr(end);
                self.push_scope();
                self.declare_var(var_name, SemaType::I64);
                self.infer_block_return_type(body);
                self.pop_scope();
            }
            Stmt::Loop(body) => {
                self.push_scope();
                self.infer_block_return_type(body);
                self.pop_scope();
            }
            Stmt::Assign { target, value } => {
                let val_type = self.infer_expr(value);
                if let Some(var_type) = self.lookup_var(target) {
                    if val_type != SemaType::Unknown && var_type != SemaType::Unknown {
                        if !val_type.is_compatible_with(&var_type) {
                            self.error(
                                format!("assigning {} to variable '{}' of type {}", val_type, target, var_type),
                                0, 0
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn infer_expr(&mut self, expr: &Expr) -> SemaType {
        match expr {
            Expr::Int(_) => SemaType::I64,
            Expr::Float(_) => SemaType::F64,
            Expr::Str(_) => SemaType::Str,
            Expr::Bool(_) => SemaType::Bool,
            Expr::None => SemaType::Unit,
            Expr::Ident(name) => {
                if let Some(t) = self.lookup_var(name) {
                    t
                } else {
                    self.error(format!("undefined variable '{}'", name), 0, 0);
                    SemaType::Unknown
                }
            }
            Expr::List(items) => {
                if items.is_empty() {
                    SemaType::List(Box::new(SemaType::Unknown))
                } else {
                    let elem_type = self.infer_expr(&items[0]);
                    SemaType::List(Box::new(elem_type))
                }
            }
            Expr::Index { target, index } => {
                let target_type = self.infer_expr(target);
                let index_type = self.infer_expr(index);

                if !index_type.is_integer() {
                    self.error(format!("list index must be integer, got {}", index_type), 0, 0);
                }

                match target_type {
                    SemaType::List(elem) => *elem,
                    SemaType::Unknown => SemaType::Unknown,
                    _ => {
                        self.error(format!("cannot index type {}", target_type), 0, 0);
                        SemaType::Unknown
                    }
                }
            }
            Expr::Binary { op, left, right } => {
                let left_type = self.infer_expr(left);
                let right_type = self.infer_expr(right);

                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        if left_type.is_numeric() && right_type.is_numeric() {
                            if left_type.is_float() || right_type.is_float() {
                                SemaType::F64
                            } else {
                                SemaType::I64
                            }
                        } else if matches!(op, BinOp::Add) && left_type == SemaType::Str && right_type == SemaType::Str {
                            SemaType::Str
                        } else if left_type == SemaType::Unknown || right_type == SemaType::Unknown {
                            SemaType::Unknown
                        } else {
                            self.error(
                                format!("cannot apply {:?} to {} and {}", op, left_type, right_type),
                                0, 0
                            );
                            SemaType::Unknown
                        }
                    }
                    BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => {
                        if left_type.is_compatible_with(&right_type) {
                            SemaType::Bool
                        } else if left_type == SemaType::Unknown || right_type == SemaType::Unknown {
                            SemaType::Bool
                        } else {
                            self.error(
                                format!("cannot compare {} and {}", left_type, right_type),
                                0, 0
                            );
                            SemaType::Bool
                        }
                    }
                    BinOp::And | BinOp::Or => {
                        if left_type.is_bool() && right_type.is_bool() {
                            SemaType::Bool
                        } else if left_type == SemaType::Unknown || right_type == SemaType::Unknown {
                            SemaType::Bool
                        } else {
                            self.error(
                                format!("logical operator requires bool, got {} and {}", left_type, right_type),
                                0, 0
                            );
                            SemaType::Bool
                        }
                    }
                    BinOp::Pipe => {
                        SemaType::Unknown
                    }
                }
            }
            Expr::Unary { op, operand } => {
                let op_type = self.infer_expr(operand);
                match op {
                    UnaryOp::Neg => {
                        if op_type.is_numeric() || op_type == SemaType::Unknown {
                            op_type
                        } else {
                            self.error(format!("cannot negate type {}", op_type), 0, 0);
                            SemaType::Unknown
                        }
                    }
                    UnaryOp::Not => {
                        if op_type.is_bool() || op_type == SemaType::Unknown {
                            SemaType::Bool
                        } else {
                            self.error(format!("logical not requires bool, got {}", op_type), 0, 0);
                            SemaType::Bool
                        }
                    }
                }
            }
            Expr::Call { callee, args } => {
                if let Some(fn_sig) = self.fn_signatures.get(callee).cloned() {
                    if let SemaType::Function { params, ret } = fn_sig {
                        for (i, arg) in args.iter().enumerate() {
                            let arg_type = self.infer_expr(arg);
                            if let Some(param_type) = params.get(i) {
                                if *param_type != SemaType::Unknown && arg_type != SemaType::Unknown {
                                    if !arg_type.is_compatible_with(param_type) {
                                        self.error(
                                            format!("argument {} to '{}' has type {}, expected {}",
                                                i + 1, callee, arg_type, param_type),
                                            0, 0
                                        );
                                    }
                                }
                            }
                        }
                        *ret
                    } else {
                        SemaType::Unknown
                    }
                } else {
                    SemaType::Unknown
                }
            }
            Expr::IfExpr { condition, then_value, else_value } => {
                let cond_type = self.infer_expr(condition);
                if !cond_type.is_bool() && cond_type != SemaType::Unknown {
                    self.error(format!("if condition must be bool, got {}", cond_type), 0, 0);
                }

                let then_type = self.infer_expr(then_value);
                let else_type = self.infer_expr(else_value);

                if then_type == else_type {
                    then_type
                } else if then_type.is_compatible_with(&else_type) {
                    if then_type.is_float() { then_type } else { else_type }
                } else if then_type == SemaType::Unknown {
                    else_type
                } else if else_type == SemaType::Unknown {
                    then_type
                } else {
                    self.error(
                        format!("if branches have different types: {} and {}", then_type, else_type),
                        0, 0
                    );
                    SemaType::Unknown
                }
            }
            Expr::FieldAccess { target, field } => {
                let target_type = self.infer_expr(target);
                match target_type {
                    SemaType::Named(struct_name) => {
                        if let Some(fields) = self.struct_fields.get(&struct_name) {
                            if let Some((_, field_type)) = fields.iter().find(|(n, _)| n == field) {
                                field_type.clone()
                            } else {
                                self.error(
                                    format!("struct '{}' has no field '{}'", struct_name, field),
                                    0, 0
                                );
                                SemaType::Unknown
                            }
                        } else {
                            SemaType::Named(struct_name)
                        }
                    }
                    SemaType::Unknown => SemaType::Unknown,
                    _ => {
                        self.error(format!("cannot access field '{}' on type {}", field, target_type), 0, 0);
                        SemaType::Unknown
                    }
                }
            }
            Expr::StructInit { name, fields } => {
                let struct_fields_opt = self.struct_fields.get(name).cloned();
                if let Some(struct_fields) = struct_fields_opt {
                    for (fname, fval) in fields {
                        let fval_type = self.infer_expr(fval);
                        if let Some((_, expected)) = struct_fields.iter().find(|(n, _)| n == fname) {
                            if fval_type != SemaType::Unknown && *expected != SemaType::Unknown {
                                if !fval_type.is_compatible_with(expected) {
                                    self.error(
                                        format!("field '{}' of struct '{}' has type {}, expected {}",
                                            fname, name, fval_type, expected),
                                        0, 0
                                    );
                                }
                            }
                        } else {
                            self.error(
                                format!("struct '{}' has no field '{}'", name, fname),
                                0, 0
                            );
                        }
                    }
                    SemaType::Named(name.clone())
                } else {
                    SemaType::Named(name.clone())
                }
            }
            Expr::Path { base, segment: _ } => {
                if self.enum_variants.contains_key(base) {
                    SemaType::Named(base.clone())
                } else if self.struct_fields.contains_key(base) {
                    SemaType::Named(base.clone())
                } else {
                    SemaType::Unknown
                }
            }
            Expr::PathCall { base, segment: _, args } => {
                for arg in args {
                    self.infer_expr(arg);
                }
                if self.enum_variants.contains_key(base) {
                    SemaType::Named(base.clone())
                } else {
                    SemaType::Unknown
                }
            }
            Expr::BlockExpr(block) => {
                self.push_scope();
                let ret = self.infer_block_return_type(block);
                self.pop_scope();
                ret
            }
            Expr::Await(inner) => {
                self.infer_expr(inner)
            }
            Expr::MatchExpr { scrutinee, arms } => {
                let _scrut_type = self.infer_expr(scrutinee);
                let mut result_type = SemaType::Unknown;
                for arm in arms {
                    self.push_scope();
                    self.bind_pattern_vars(&arm.pattern);
                    let arm_type = self.infer_block_return_type(&arm.body);
                    self.pop_scope();
                    if result_type == SemaType::Unknown {
                        result_type = arm_type;
                    }
                }
                result_type
            }
        }
    }

    fn infer_block_return_type(&mut self, block: &Block) -> SemaType {
        let mut ret_type = SemaType::Void;
        for stmt in &block.stmts {
            match stmt {
                Stmt::Return(Some(expr)) => {
                    ret_type = self.infer_expr(expr);
                }
                Stmt::Return(None) => {
                    ret_type = SemaType::Void;
                }
                Stmt::Expr(expr) => {
                    ret_type = self.infer_expr(expr);
                }
                Stmt::LetDecl { name, type_annotation, value } => {
                    let inferred = if let Some(val) = value {
                        self.infer_expr(val)
                    } else {
                        SemaType::Unknown
                    };
                    let declared = if let Some(ta) = type_annotation {
                        SemaType::from_annotation(ta)
                    } else {
                        inferred.clone()
                    };
                    self.declare_var(name, declared);
                }
                Stmt::Assign { target, value } => {
                    let val_type = self.infer_expr(value);
                    if let Some(var_type) = self.lookup_var(target) {
                        if val_type != SemaType::Unknown && var_type != SemaType::Unknown {
                            if !val_type.is_compatible_with(&var_type) {
                                self.error(
                                    format!("assigning {} to variable '{}' of type {}", val_type, target, var_type),
                                    0, 0
                                );
                            }
                        }
                    }
                }
                Stmt::If { condition, then_branch, else_branch } => {
                    let cond_type = self.infer_expr(condition);
                    if !cond_type.is_bool() && cond_type != SemaType::Unknown {
                        self.error(format!("if condition must be bool, got {}", cond_type), 0, 0);
                    }
                    self.push_scope();
                    self.infer_block_return_type(then_branch);
                    self.pop_scope();
                    if let Some(else_block) = else_branch {
                        self.push_scope();
                        self.infer_block_return_type(else_block);
                        self.pop_scope();
                    }
                }
                Stmt::While { condition, body } => {
                    let cond_type = self.infer_expr(condition);
                    if !cond_type.is_bool() && cond_type != SemaType::Unknown {
                        self.error(format!("while condition must be bool, got {}", cond_type), 0, 0);
                    }
                    self.push_scope();
                    self.infer_block_return_type(body);
                    self.pop_scope();
                }
                Stmt::For { var_name, start, end, body } => {
                    self.infer_expr(start);
                    self.infer_expr(end);
                    self.push_scope();
                    self.declare_var(var_name, SemaType::I64);
                    self.infer_block_return_type(body);
                    self.pop_scope();
                }
                Stmt::Loop(body) => {
                    self.push_scope();
                    self.infer_block_return_type(body);
                    self.pop_scope();
                }
                Stmt::Match { scrutinee, arms } => {
                    self.infer_expr(scrutinee);
                    let mut match_ret = SemaType::Unknown;
                    for arm in arms {
                        self.push_scope();
                        self.bind_pattern_vars(&arm.pattern);
                        let arm_ret = self.infer_block_return_type(&arm.body);
                        self.pop_scope();
                        if match_ret == SemaType::Unknown {
                            match_ret = arm_ret;
                        }
                    }
                    if match_ret != SemaType::Void {
                        ret_type = match_ret;
                    }
                }
                _ => {}
            }
        }
        ret_type
    }
}

pub fn check_program(program: &Program) -> Vec<SemaError> {
    let mut checker = TypeChecker::new();
    checker.check_program(program)
}

pub struct ConstFolder;

impl ConstFolder {
    pub fn new() -> Self {
        ConstFolder
    }

    pub fn fold_program(&mut self, program: &Program) -> Program {
        match program {
            Program::Block(stmts) => {
                let mut folded_stmts = Vec::new();
                for stmt in stmts {
                    folded_stmts.push(self.fold_stmt(stmt));
                }
                Program::Block(folded_stmts)
            }
        }
    }

    fn fold_stmt(&mut self, stmt: &Stmt) -> Stmt {
        match stmt {
            Stmt::LetDecl { name, type_annotation, value } => {
                let folded_value = value.as_ref().map(|e| self.fold_expr(e));
                Stmt::LetDecl {
                    name: name.clone(),
                    type_annotation: type_annotation.clone(),
                    value: folded_value,
                }
            }
            Stmt::Assign { target, value } => {
                Stmt::Assign {
                    target: target.clone(),
                    value: self.fold_expr(value),
                }
            }
            Stmt::Expr(expr) => {
                Stmt::Expr(self.fold_expr(expr))
            }
            Stmt::Return(Some(expr)) => {
                Stmt::Return(Some(self.fold_expr(expr)))
            }
            Stmt::Return(None) => Stmt::Return(None),
            Stmt::If { condition, then_branch, else_branch } => {
                let folded_cond = self.fold_expr(condition);
                Stmt::If {
                    condition: folded_cond,
                    then_branch: self.fold_block(then_branch),
                    else_branch: else_branch.as_ref().map(|b| self.fold_block(b)),
                }
            }
            Stmt::While { condition, body } => {
                let folded_cond = self.fold_expr(condition);
                Stmt::While {
                    condition: folded_cond,
                    body: self.fold_block(body),
                }
            }
            Stmt::For { var_name, start, end, body } => {
                let folded_start = self.fold_expr(start);
                let folded_end = self.fold_expr(end);
                Stmt::For {
                    var_name: var_name.clone(),
                    start: folded_start,
                    end: folded_end,
                    body: self.fold_block(body),
                }
            }
            Stmt::Loop(body) => {
                Stmt::Loop(self.fold_block(body))
            }
            Stmt::Break => Stmt::Break,
            Stmt::Continue => Stmt::Continue,
            Stmt::FnDecl { name, params, return_type, body, is_async } => {
                Stmt::FnDecl {
                    name: name.clone(),
                    params: params.clone(),
                    return_type: return_type.clone(),
                    body: self.fold_block(body),
                    is_async: *is_async,
                }
            }
            Stmt::StructDecl { name, fields } => {
                Stmt::StructDecl {
                    name: name.clone(),
                    fields: fields.clone(),
                }
            }
            Stmt::EnumDecl { name, variants } => {
                Stmt::EnumDecl {
                    name: name.clone(),
                    variants: variants.clone(),
                }
            }
            Stmt::ExternDecl { language, module, decls } => {
                Stmt::ExternDecl {
                    language: language.clone(),
                    module: module.clone(),
                    decls: decls.clone(),
                }
            }
            Stmt::ExportDecl { language, module, decls } => {
                Stmt::ExportDecl {
                    language: language.clone(),
                    module: module.clone(),
                    decls: decls.clone(),
                }
            }
            Stmt::Match { scrutinee, arms } => {
                let folded_scrutinee = self.fold_expr(scrutinee);
                let mut folded_arms = Vec::new();
                for arm in arms {
                    folded_arms.push(MatchArm {
                        pattern: arm.pattern.clone(),
                        body: self.fold_block(&arm.body),
                    });
                }
                Stmt::Match {
                    scrutinee: folded_scrutinee,
                    arms: folded_arms,
                }
            }
            Stmt::FlowDecl { name, description, source, pipeline } => {
                Stmt::FlowDecl {
                    name: name.clone(),
                    description: description.clone(),
                    source: source.as_ref().map(|e| self.fold_expr(e)),
                    pipeline: self.fold_expr(pipeline),
                }
            }
        }
    }

    fn fold_block(&mut self, block: &Block) -> Block {
        let mut folded_stmts = Vec::new();
        for stmt in &block.stmts {
            folded_stmts.push(self.fold_stmt(stmt));
        }
        Block { stmts: folded_stmts }
    }

    fn fold_expr(&mut self, expr: &Expr) -> Expr {
        match expr {
            Expr::Binary { op, left, right } => {
                let folded_left = self.fold_expr(left);
                let folded_right = self.fold_expr(right);

                match (&folded_left, &folded_right) {
                    (Expr::Int(l), Expr::Int(r)) => {
                        match op {
                            BinOp::Add => return Expr::Int(l + r),
                            BinOp::Sub => return Expr::Int(l - r),
                            BinOp::Mul => return Expr::Int(l * r),
                            BinOp::Div if *r != 0 => return Expr::Int(l / r),
                            BinOp::Mod if *r != 0 => return Expr::Int(l % r),
                            BinOp::Eq => return Expr::Bool(l == r),
                            BinOp::Neq => return Expr::Bool(l != r),
                            BinOp::Lt => return Expr::Bool(l < r),
                            BinOp::Gt => return Expr::Bool(l > r),
                            BinOp::LtEq => return Expr::Bool(l <= r),
                            BinOp::GtEq => return Expr::Bool(l >= r),
                            _ => {}
                        }
                    }
                    (Expr::Float(l), Expr::Float(r)) => {
                        match op {
                            BinOp::Add => return Expr::Float(l + r),
                            BinOp::Sub => return Expr::Float(l - r),
                            BinOp::Mul => return Expr::Float(l * r),
                            BinOp::Div if *r != 0.0 => return Expr::Float(l / r),
                            BinOp::Eq => return Expr::Bool((l - r).abs() < f64::EPSILON),
                            BinOp::Neq => return Expr::Bool((l - r).abs() >= f64::EPSILON),
                            BinOp::Lt => return Expr::Bool(l < r),
                            BinOp::Gt => return Expr::Bool(l > r),
                            BinOp::LtEq => return Expr::Bool(l <= r),
                            BinOp::GtEq => return Expr::Bool(l >= r),
                            _ => {}
                        }
                    }
                    (Expr::Bool(l), Expr::Bool(r)) => {
                        match op {
                            BinOp::And => return Expr::Bool(*l && *r),
                            BinOp::Or => return Expr::Bool(*l || *r),
                            BinOp::Eq => return Expr::Bool(l == r),
                            BinOp::Neq => return Expr::Bool(l != r),
                            _ => {}
                        }
                    }
                    (Expr::Str(l), Expr::Str(r)) => {
                        if let BinOp::Add = op {
                            return Expr::Str(format!("{}{}", l, r));
                        }
                    }
                    _ => {}
                }

                Expr::Binary {
                    op: op.clone(),
                    left: Box::new(folded_left),
                    right: Box::new(folded_right),
                }
            }
            Expr::Unary { op, operand } => {
                let folded_operand = self.fold_expr(operand);
                match (&op, &folded_operand) {
                    (UnaryOp::Neg, Expr::Int(n)) => return Expr::Int(-n),
                    (UnaryOp::Neg, Expr::Float(f)) => return Expr::Float(-f),
                    (UnaryOp::Not, Expr::Bool(b)) => return Expr::Bool(!b),
                    _ => {}
                }
                Expr::Unary {
                    op: op.clone(),
                    operand: Box::new(folded_operand),
                }
            }
            Expr::IfExpr { condition, then_value, else_value } => {
                let folded_cond = self.fold_expr(condition);
                if let Expr::Bool(true) = &folded_cond {
                    return self.fold_expr(then_value);
                }
                if let Expr::Bool(false) = &folded_cond {
                    return self.fold_expr(else_value);
                }
                Expr::IfExpr {
                    condition: Box::new(folded_cond),
                    then_value: Box::new(self.fold_expr(then_value)),
                    else_value: Box::new(self.fold_expr(else_value)),
                }
            }
            Expr::Call { callee, args } => {
                let mut folded_args = Vec::new();
                for arg in args {
                    folded_args.push(self.fold_expr(arg));
                }

                if callee == "len" && folded_args.len() == 1 {
                    if let Expr::Str(s) = &folded_args[0] {
                        return Expr::Int(s.len() as i64);
                    }
                    if let Expr::List(items) = &folded_args[0] {
                        return Expr::Int(items.len() as i64);
                    }
                }

                Expr::Call {
                    callee: callee.clone(),
                    args: folded_args,
                }
            }
            Expr::List(items) => {
                let mut folded_items = Vec::new();
                for item in items {
                    folded_items.push(self.fold_expr(item));
                }
                Expr::List(folded_items)
            }
            Expr::Index { target, index } => {
                let folded_target = self.fold_expr(target);
                let folded_index = self.fold_expr(index);
                if let (Expr::List(items), Expr::Int(i)) = (&folded_target, &folded_index) {
                    if *i >= 0 && (*i as usize) < items.len() {
                        return items[*i as usize].clone();
                    }
                }
                Expr::Index {
                    target: Box::new(folded_target),
                    index: Box::new(folded_index),
                }
            }
            Expr::BlockExpr(block) => {
                Expr::BlockExpr(self.fold_block(block))
            }
            Expr::StructInit { name, fields } => {
                let mut folded_fields = Vec::new();
                for (fname, fval) in fields {
                    folded_fields.push((fname.clone(), self.fold_expr(fval)));
                }
                Expr::StructInit {
                    name: name.clone(),
                    fields: folded_fields,
                }
            }
            Expr::FieldAccess { target, field } => {
                Expr::FieldAccess {
                    target: Box::new(self.fold_expr(target)),
                    field: field.clone(),
                }
            }
            Expr::MatchExpr { scrutinee, arms } => {
                let folded_scrutinee = self.fold_expr(scrutinee);
                let mut folded_arms = Vec::new();
                for arm in arms {
                    folded_arms.push(MatchArm {
                        pattern: arm.pattern.clone(),
                        body: self.fold_block(&arm.body),
                    });
                }
                Expr::MatchExpr {
                    scrutinee: Box::new(folded_scrutinee),
                    arms: folded_arms,
                }
            }
            Expr::Await(inner) => {
                Expr::Await(Box::new(self.fold_expr(inner)))
            }
            _ => expr.clone(),
        }
    }
}

pub fn const_fold(program: &Program) -> Program {
    let mut folder = ConstFolder::new();
    folder.fold_program(program)
}

pub struct DeadCodeEliminator;

impl DeadCodeEliminator {
    pub fn new() -> Self {
        DeadCodeEliminator
    }

    pub fn eliminate_program(&mut self, program: &Program) -> Program {
        match program {
            Program::Block(stmts) => {
                Program::Block(self.eliminate_stmts(stmts))
            }
        }
    }

    fn eliminate_stmts(&mut self, stmts: &[Stmt]) -> Vec<Stmt> {
        let mut result = Vec::new();
        let mut unreachable = false;

        for stmt in stmts {
            if unreachable {
                continue;
            }

            let eliminated = self.eliminate_stmt(stmt);
            result.push(eliminated);

            match stmt {
                Stmt::Return(_) | Stmt::Break | Stmt::Continue => {
                    unreachable = true;
                }
                _ => {}
            }
        }

        result
    }

    fn eliminate_stmt(&mut self, stmt: &Stmt) -> Stmt {
        match stmt {
            Stmt::FnDecl { name, params, return_type, body, is_async } => {
                Stmt::FnDecl {
                    name: name.clone(),
                    params: params.clone(),
                    return_type: return_type.clone(),
                    body: self.eliminate_block(body),
                    is_async: *is_async,
                }
            }
            Stmt::If { condition, then_branch, else_branch } => {
                Stmt::If {
                    condition: condition.clone(),
                    then_branch: self.eliminate_block(then_branch),
                    else_branch: else_branch.as_ref().map(|b| self.eliminate_block(b)),
                }
            }
            Stmt::While { condition, body } => {
                Stmt::While {
                    condition: condition.clone(),
                    body: self.eliminate_block(body),
                }
            }
            Stmt::For { var_name, start, end, body } => {
                Stmt::For {
                    var_name: var_name.clone(),
                    start: start.clone(),
                    end: end.clone(),
                    body: self.eliminate_block(body),
                }
            }
            Stmt::Loop(body) => {
                Stmt::Loop(self.eliminate_block(body))
            }
            Stmt::Match { scrutinee, arms } => {
                let mut new_arms = Vec::new();
                for arm in arms {
                    new_arms.push(MatchArm {
                        pattern: arm.pattern.clone(),
                        body: self.eliminate_block(&arm.body),
                    });
                }
                Stmt::Match {
                    scrutinee: scrutinee.clone(),
                    arms: new_arms,
                }
            }
            Stmt::Return(Some(expr)) => {
                Stmt::Return(Some(expr.clone()))
            }
            _ => stmt.clone(),
        }
    }

    fn eliminate_block(&mut self, block: &Block) -> Block {
        Block {
            stmts: self.eliminate_stmts(&block.stmts),
        }
    }
}

pub fn eliminate_dead_code(program: &Program) -> Program {
    let mut eliminator = DeadCodeEliminator::new();
    eliminator.eliminate_program(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use linkc_lexer::lex;

    fn parse(source: &str) -> Program {
        let tokens = lex(source);
        let mut parser = Parser::new(tokens);
        parser.parse_program().unwrap()
    }

    #[test]
    fn test_check_int_literal() {
        let program = parse("42");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_binary_add() {
        let program = parse("1 + 2");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_undefined_variable() {
        let program = parse("x");
        let errors = check_program(&program);
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("undefined variable"));
    }

    #[test]
    fn test_check_let_decl() {
        let program = parse("let x: i64 = 42; x");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_function_call() {
        let program = parse("fn add(a: i64, b: i64) -> i64 { return a + b; } add(1, 2)");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_if_condition() {
        let program = parse("if true { 1 } else { 2 }");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_struct() {
        let program = parse("struct Point { x: i32, y: i32 } let p = Point { x: 1, y: 2 }; p.x");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_list() {
        let program = parse("let xs = [1, 2, 3]; xs[0]");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_extern() {
        let program = parse(r#"extern "C" { fn abs(n: i32) -> i32; } abs(-42)"#);
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_string_concat() {
        let program = parse(r#""hello" + "world""#);
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_while_loop() {
        let program = parse("let i = 0; while i < 10 { i = i + 1; }");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_for_loop() {
        let program = parse("for i in 0..10 { println(i); }");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_enum() {
        let program = parse("enum Color { Red, Green, Blue } let c = Color::Red;");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_enum_with_payload() {
        let program = parse("enum Result { Ok(i64), Err(str) } let r = Result::Ok(42);");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_recursive_function() {
        let program = parse("fn fib(n: i64) -> i64 { if n < 2 { return n; } return fib(n - 1) + fib(n - 2); } fib(10)");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_bad_if_condition() {
        let program = parse("if 42 { 1 } else { 2 }");
        let errors = check_program(&program);
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("if condition must be bool"));
    }

    #[test]
    fn test_check_len_function() {
        let program = parse(r#"len("hello")"#);
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_println() {
        let program = parse(r#"println("hello")"#);
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_nested_blocks() {
        let program = parse("let x = 1; { let y = 2; x + y }");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_variable_shadowing() {
        let program = parse("let x = 1; let x = 2; x");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_struct_field_access() {
        let program = parse("struct Vec2 { x: f64, y: f64 } let v = Vec2 { x: 1.0, y: 2.0 }; v.x + v.y");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_list_len() {
        let program = parse("let xs = [1, 2, 3]; len(xs)");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_match_enum_return() {
        let program = parse("enum Shape { Circle(f64), Square(f64) } fn area(s: Shape) -> f64 { match s { Shape::Circle(r) => { return r; } Shape::Square(x) => { return x; } } }");
        let errors = check_program(&program);
        for e in &errors {
            eprintln!("ERROR: {}", e.message);
        }
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_match_literal_return() {
        let program = parse("fn test(x: i64) -> i64 { match x { 1 => { return 10; } 2 => { return 20; } _ => { return 30; } } }");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    mod const_fold_tests {
        use super::*;

        #[test]
        fn test_fold_int_add() {
            let program = parse("1 + 2");
            let folded = const_fold(&program);
            let Program::Block(stmts) = &folded;
            if let Stmt::Expr(Expr::Int(n)) = &stmts[0] {
                assert_eq!(*n, 3);
                return;
            }
            panic!("expected folded int");
        }

        #[test]
        fn test_fold_int_mul() {
            let program = parse("3 * 4");
            let folded = const_fold(&program);
            let Program::Block(stmts) = &folded;
            if let Stmt::Expr(Expr::Int(n)) = &stmts[0] {
                assert_eq!(*n, 12);
                return;
            }
            panic!("expected folded int");
        }

        #[test]
        fn test_fold_float_add() {
            let program = parse("1.5 + 2.5");
            let folded = const_fold(&program);
            let Program::Block(stmts) = &folded;
            if let Stmt::Expr(Expr::Float(f)) = &stmts[0] {
                assert!((f - 4.0).abs() < f64::EPSILON);
                return;
            }
            panic!("expected folded float");
        }

        #[test]
        fn test_fold_bool_and() {
            let program = parse("true && false");
            let folded = const_fold(&program);
            let Program::Block(stmts) = &folded;
            if let Stmt::Expr(Expr::Bool(b)) = &stmts[0] {
                assert!(!b);
                return;
            }
            panic!("expected folded bool");
        }

        #[test]
        fn test_fold_neg() {
            let program = parse("-42");
            let folded = const_fold(&program);
            let Program::Block(stmts) = &folded;
            if let Stmt::Expr(Expr::Int(n)) = &stmts[0] {
                assert_eq!(*n, -42);
                return;
            }
            panic!("expected folded neg");
        }

        #[test]
        fn test_fold_not() {
            let program = parse("!true");
            let folded = const_fold(&program);
            let Program::Block(stmts) = &folded;
            if let Stmt::Expr(Expr::Bool(b)) = &stmts[0] {
                assert!(!b);
                return;
            }
            panic!("expected folded not");
        }

        #[test]
        fn test_fold_string_concat() {
            let program = parse(r#""hello" + "world""#);
            let folded = const_fold(&program);
            let Program::Block(stmts) = &folded;
            if let Stmt::Expr(Expr::Str(s)) = &stmts[0] {
                assert_eq!(s, "helloworld");
                return;
            }
            panic!("expected folded string");
        }

        #[test]
        fn test_fold_len_str() {
            let program = parse(r#"len("hello")"#);
            let folded = const_fold(&program);
            let Program::Block(stmts) = &folded;
            if let Stmt::Expr(Expr::Int(n)) = &stmts[0] {
                assert_eq!(*n, 5);
                return;
            }
            panic!("expected folded len");
        }

        #[test]
        fn test_fold_len_list() {
            let program = parse("len([1, 2, 3])");
            let folded = const_fold(&program);
            let Program::Block(stmts) = &folded;
            if let Stmt::Expr(Expr::Int(n)) = &stmts[0] {
                assert_eq!(*n, 3);
                return;
            }
            panic!("expected folded len");
        }

        #[test]
        fn test_fold_nested() {
            let program = parse("(2 + 3) * 4");
            let folded = const_fold(&program);
            let Program::Block(stmts) = &folded;
            if let Stmt::Expr(Expr::Int(n)) = &stmts[0] {
                assert_eq!(*n, 20);
                return;
            }
            panic!("expected folded nested");
        }

        #[test]
        fn test_fold_comparison() {
            let program = parse("5 > 3");
            let folded = const_fold(&program);
            let Program::Block(stmts) = &folded;
            if let Stmt::Expr(Expr::Bool(b)) = &stmts[0] {
                assert!(*b);
                return;
            }
            panic!("expected folded comparison");
        }

        #[test]
        fn test_fold_list_index() {
            let program = parse("[10, 20, 30][1]");
            let folded = const_fold(&program);
            let Program::Block(stmts) = &folded;
            if let Stmt::Expr(Expr::Int(n)) = &stmts[0] {
                assert_eq!(*n, 20);
                return;
            }
            panic!("expected folded list index");
        }
    }

    mod dead_code_tests {
        use super::*;

        #[test]
        fn test_eliminate_after_return() {
            let program = parse("fn test() -> i64 { return 1; let x = 2; }");
            let eliminated = eliminate_dead_code(&program);
            let Program::Block(stmts) = &eliminated;
            if let Stmt::FnDecl { body, .. } = &stmts[0] {
                assert_eq!(body.stmts.len(), 1);
                assert!(matches!(body.stmts[0], Stmt::Return(Some(Expr::Int(1)))));
                return;
            }
            panic!("expected fn decl");
        }

        #[test]
        fn test_eliminate_if_still_there() {
            let program = parse("if true { 1 } else { 2 }");
            let eliminated = eliminate_dead_code(&program);
            let Program::Block(stmts) = &eliminated;
            assert!(matches!(stmts[0], Stmt::If { .. }));
        }

        #[test]
        fn test_eliminate_while_still_there() {
            let program = parse("while false { println(1); }");
            let eliminated = eliminate_dead_code(&program);
            let Program::Block(stmts) = &eliminated;
            assert!(matches!(stmts[0], Stmt::While { .. }));
        }

        #[test]
        fn test_eliminate_preserves_useful_code() {
            let program = parse("let x = 1; let y = 2; x + y");
            let eliminated = eliminate_dead_code(&program);
            let Program::Block(stmts) = &eliminated;
            assert_eq!(stmts.len(), 3);
        }

        #[test]
        fn test_eliminate_nested_blocks() {
            let program = parse("fn test() -> i64 { if true { return 1; } let x = 2; return x; }");
            let eliminated = eliminate_dead_code(&program);
            let Program::Block(stmts) = &eliminated;
            if let Stmt::FnDecl { body, .. } = &stmts[0] {
                assert_eq!(body.stmts.len(), 3);
                return;
            }
            panic!("expected fn decl");
        }

        #[test]
        fn test_eliminate_break() {
            let program = parse("fn test() { loop { break; let x = 1; } }");
            let eliminated = eliminate_dead_code(&program);
            let Program::Block(stmts) = &eliminated;
            if let Stmt::FnDecl { body, .. } = &stmts[0] {
                if let Stmt::Loop(loop_body) = &body.stmts[0] {
                    assert_eq!(loop_body.stmts.len(), 1);
                    return;
                }
            }
            panic!("expected loop with 1 stmt");
        }
    }
}
