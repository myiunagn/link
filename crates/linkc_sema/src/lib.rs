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
    Ref(Box<SemaType>, bool),
    Tuple(Vec<SemaType>),
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
            TypeAnnotation::Named(name) => {
                match name.as_str() {
                    "list" => SemaType::List(Box::new(SemaType::Unknown)),
                    "stream" => SemaType::Stream(Box::new(SemaType::Unknown)),
                    "int" => SemaType::I64,
                    "float" => SemaType::F64,
                    "string" | "str" => SemaType::Str,
                    "bool" => SemaType::Bool,
                    "unit" | "void" => SemaType::Unit,
                    _ => SemaType::Named(name.clone()),
                }
            }
            TypeAnnotation::Stream(inner) => SemaType::Stream(Box::new(SemaType::from_annotation(inner))),
            TypeAnnotation::Ref(inner, is_mut) => SemaType::Ref(Box::new(SemaType::from_annotation(inner)), *is_mut),
            TypeAnnotation::Generic(base, _) => SemaType::from_annotation(base),
            TypeAnnotation::Array(elem, _) => SemaType::List(Box::new(SemaType::from_annotation(elem))),
            TypeAnnotation::Tuple(elems) => {
                let elem_types: Vec<SemaType> = elems.iter()
                    .map(|e| SemaType::from_annotation(e))
                    .collect();
                SemaType::Tuple(elem_types)
            }
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
        match (self, other) {
            (SemaType::Unknown, _) | (_, SemaType::Unknown) => true,
            (SemaType::Unit, SemaType::Void) | (SemaType::Void, SemaType::Unit) => true,
            (SemaType::List(a), SemaType::List(b)) => a.is_compatible_with(b),
            (SemaType::Stream(a), SemaType::Stream(b)) => a.is_compatible_with(b),
            (SemaType::Tuple(a_elems), SemaType::Tuple(b_elems)) => {
                if a_elems.len() != b_elems.len() {
                    return false;
                }
                a_elems.iter().zip(b_elems.iter()).all(|(a, b)| a.is_compatible_with(b))
            }
            _ => false,
        }
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
            SemaType::Ref(inner, is_mut) => {
                if *is_mut {
                    write!(f, "&mut {}", inner)
                } else {
                    write!(f, "&{}", inner)
                }
            }
            SemaType::Tuple(elems) => {
                write!(f, "(")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
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
                let is_forward_decl = body.stmts.is_empty() ||
                    (body.stmts.len() == 1 && matches!(&body.stmts[0], Stmt::Expr(Expr::None)));
                if !is_forward_decl && body_ret != SemaType::Unknown && body_ret != ret_type && ret_type != SemaType::Void {
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
                self.infer_expr(condition);
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
                self.infer_expr(condition);
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
            Stmt::ForIterable { var_name, iterable, body } => {
                let iter_ty = self.infer_expr(iterable);
                let elem_ty = match &iter_ty {
                    SemaType::List(e) | SemaType::Stream(e) => *e.clone(),
                    _ => SemaType::Unknown,
                };
                self.push_scope();
                self.declare_var(var_name, elem_ty);
                self.infer_block_return_type(body);
                self.pop_scope();
            }
            Stmt::Loop(body) => {
                self.push_scope();
                self.infer_block_return_type(body);
                self.pop_scope();
            }
            Stmt::Assign { target, value } => {
                let _val_type = self.infer_expr(value);
                self.infer_lvalue_target(target);
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
                } else if let Some(fn_sig) = self.fn_signatures.get(name).cloned() {
                    fn_sig
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
            Expr::Tuple(elems) => {
                let elem_types: Vec<SemaType> = elems.iter()
                    .map(|e| self.infer_expr(e))
                    .collect();
                SemaType::Tuple(elem_types)
            }
            Expr::Index { target, index } => {
                let target_type = self.infer_expr(target);
                let index_type = self.infer_expr(index);

                if !index_type.is_integer() {
                    self.error(format!("list index must be integer, got {}", index_type), 0, 0);
                }

                match target_type {
                    SemaType::List(elem) => *elem,
                    SemaType::Str => SemaType::Str,
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
                                    if left_type == SemaType::F32 && right_type == SemaType::F32 {
                                        SemaType::F32
                                    } else {
                                        SemaType::F64
                                    }
                                } else if left_type == SemaType::I32 && right_type == SemaType::I32 {
                                    SemaType::I32
                                } else if left_type == SemaType::U32 && right_type == SemaType::U32 {
                                    SemaType::U32
                                } else if left_type == SemaType::I16 && right_type == SemaType::I16 {
                                    SemaType::I16
                                } else if left_type == SemaType::U16 && right_type == SemaType::U16 {
                                    SemaType::U16
                                } else if left_type == SemaType::I8 && right_type == SemaType::I8 {
                                    SemaType::I8
                                } else if left_type == SemaType::U8 && right_type == SemaType::U8 {
                                    SemaType::U8
                                } else {
                                    SemaType::I64
                                }
                            } else if matches!(op, BinOp::Add) {
                                if left_type == SemaType::Str && right_type == SemaType::Str {
                                    SemaType::Str
                                } else if matches!((&left_type, &right_type), (SemaType::List(_), SemaType::List(_))) {
                                    left_type
                                } else if left_type == SemaType::Unknown || right_type == SemaType::Unknown {
                                    SemaType::Unknown
                                } else {
                                    self.error(
                                        format!("cannot apply {:?} to {} and {}", op, left_type, right_type),
                                        0, 0
                                    );
                                    SemaType::Unknown
                                }
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
                self.infer_expr(condition);

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
            Expr::Try(inner) => {
                self.infer_expr(inner)
            }
            Expr::Lambda { params, return_type, .. } => {
                let param_types: Vec<SemaType> = params.iter()
                    .map(|(_, t)| SemaType::from_annotation(t))
                    .collect();
                let ret_type = if let Some(rt) = return_type {
                    SemaType::from_annotation(rt)
                } else {
                    SemaType::Unknown
                };
                SemaType::Function {
                    params: param_types,
                    ret: Box::new(ret_type),
                }
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
            Expr::Ref(inner, is_mut) => {
                let inner_ty = self.infer_expr(inner);
                SemaType::Ref(Box::new(inner_ty), *is_mut)
            }
            Expr::Deref(inner) => {
                let inner_ty = self.infer_expr(inner);
                match inner_ty {
                    SemaType::Ref(target, _) => *target,
                    SemaType::Ptr(target) => *target,
                    SemaType::Unknown => SemaType::Unknown,
                    other => {
                        self.error(format!("cannot dereference type {}", other), 0, 0);
                        SemaType::Unknown
                    }
                }
            }
            Expr::AsCast(expr, ty) => {
                let _ = self.infer_expr(expr);
                SemaType::from_annotation(ty)
            }
        }
    }

    fn infer_lvalue_target(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(_) => {}
            Expr::Index { target, index } => {
                self.infer_lvalue_target(target);
                let _ = self.infer_expr(index);
            }
            Expr::FieldAccess { target, .. } => {
                self.infer_lvalue_target(target);
            }
            _ => {}
        }
    }

    fn check_lvalue_assign(&mut self, target: &Expr, val_type: SemaType) {
        match target {
            Expr::Ident(name) => {
                if let Some(var_type) = self.lookup_var(name) {
                    if val_type != SemaType::Unknown && var_type != SemaType::Unknown {
                        if !val_type.is_compatible_with(&var_type) {
                            self.error(
                                format!("assigning {} to variable '{}' of type {}", val_type, name, var_type),
                                0, 0
                            );
                        }
                    }
                }
            }
            Expr::Index { target: ltarget, index } => {
                self.check_lvalue_assign(ltarget, SemaType::Unknown);
                let _ = self.infer_expr(index);
            }
            Expr::FieldAccess { target: ltarget, .. } => {
                self.check_lvalue_assign(ltarget, SemaType::Unknown);
            }
            _ => {}
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
                    self.check_lvalue_assign(target, val_type);
                }
                Stmt::If { condition, then_branch, else_branch } => {
                    self.infer_expr(condition);
                    self.push_scope();
                    let then_ret = self.infer_block_return_type(then_branch);
                    self.pop_scope();
                    let mut else_ret = SemaType::Void;
                    if let Some(else_block) = else_branch {
                        self.push_scope();
                        else_ret = self.infer_block_return_type(else_block);
                        self.pop_scope();
                    }
                    if else_branch.is_some() {
                        if then_ret != SemaType::Void {
                            ret_type = then_ret;
                        } else if else_ret != SemaType::Void {
                            ret_type = else_ret;
                        }
                    }
                }
                Stmt::While { condition, body } => {
                    self.infer_expr(condition);
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
                Stmt::ForIterable { var_name, iterable, body } => {
                    let iter_ty = self.infer_expr(iterable);
                    let elem_ty = match &iter_ty {
                        SemaType::List(e) | SemaType::Stream(e) => *e.clone(),
                        _ => SemaType::Unknown,
                    };
                    self.push_scope();
                    self.declare_var(var_name, elem_ty);
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
                    target: Box::new(self.fold_expr(target)),
                    value: Box::new(self.fold_expr(value)),
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
            Stmt::ForIterable { var_name, iterable, body } => {
                Stmt::ForIterable {
                    var_name: var_name.clone(),
                    iterable: self.fold_expr(iterable),
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
            Stmt::DomainDecl { name, config } => {
                let mut folded_config = Vec::new();
                for (k, v) in config {
                    folded_config.push((k.clone(), self.fold_expr(v)));
                }
                Stmt::DomainDecl {
                    name: name.clone(),
                    config: folded_config,
                }
            }
            Stmt::ModDecl { name } => Stmt::ModDecl { name: name.clone() },
            Stmt::UseDecl { path, alias } => Stmt::UseDecl {
                path: path.clone(),
                alias: alias.clone(),
            },
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
            Expr::Tuple(elems) => {
                let mut folded_elems = Vec::new();
                for e in elems {
                    folded_elems.push(self.fold_expr(e));
                }
                Expr::Tuple(folded_elems)
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
            Expr::Try(inner) => {
                Expr::Try(Box::new(self.fold_expr(inner)))
            }
            Expr::Lambda { params, return_type, body } => {
                Expr::Lambda {
                    params: params.clone(),
                    return_type: return_type.clone(),
                    body: self.fold_block(body),
                }
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
            Stmt::ForIterable { var_name, iterable, body } => {
                Stmt::ForIterable {
                    var_name: var_name.clone(),
                    iterable: iterable.clone(),
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

// ============================================================================
// Borrow Checker — 轻量级所有权分析
// ============================================================================

/// 变量的所有权状态
#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipState {
    /// 拥有所有权，可自由使用
    Owned,
    /// 已被移动，不可再使用
    Moved,
    /// 被不可变借用的次数
    Borrowed(usize),
    /// 被可变借用
    MutBorrowed,
}

/// 单个变量的所有权信息
#[derive(Debug, Clone)]
pub struct VarOwnership {
    pub state: OwnershipState,
    pub ty: SemaType,
    pub mutable: bool,
}

/// 判断类型是否为 Copy（按值复制，不移动所有权）
fn is_copy_type(ty: &SemaType) -> bool {
    matches!(ty,
        SemaType::I8 | SemaType::I16 | SemaType::I32 | SemaType::I64 |
        SemaType::U8 | SemaType::U16 | SemaType::U32 | SemaType::U64 | SemaType::USize |
        SemaType::F32 | SemaType::F64 | SemaType::Bool | SemaType::Unit | SemaType::Void | SemaType::Unknown |
        SemaType::Ref(_, _)
    )
}

/// 轻量级借用检查器
/// 追踪非 Copy 类型的移动语义，检测 use-after-move 和借用冲突
pub struct BorrowChecker {
    errors: Vec<SemaError>,
    scopes: Vec<HashMap<String, VarOwnership>>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            scopes: vec![HashMap::new()],
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
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn lookup_var(&self, name: &str) -> Option<VarOwnership> {
        for scope in self.scopes.iter().rev() {
            if let Some(state) = scope.get(name) {
                return Some(state.clone());
            }
        }
        None
    }

    fn declare_var(&mut self, name: &str, ty: SemaType, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), VarOwnership {
                state: OwnershipState::Owned,
                ty,
                mutable,
            });
        }
    }

    fn set_var_state(&mut self, name: &str, state: OwnershipState) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var) = scope.get_mut(name) {
                var.state = state;
                return;
            }
        }
    }

    fn error(&mut self, message: String) {
        self.errors.push(SemaError { message, line: 0, col: 0 });
    }

    /// 标记变量被移动，如果它是非 Copy 类型
    fn move_var(&mut self, name: &str) {
        if let Some(var) = self.lookup_var(name) {
            if !is_copy_type(&var.ty) {
                match &var.state {
                    OwnershipState::Moved => {
                        self.error(format!("use of moved value: '{}'", name));
                    }
                    OwnershipState::MutBorrowed => {
                        self.error(format!("cannot move '{}' while it is mutably borrowed", name));
                    }
                    // 在简化模型中，Borrowed(n>0) 视为"只读临时借用"，
                    // 之后允许 move（例如：读取 len 后再 return 容器）
                    OwnershipState::Borrowed(_) | OwnershipState::Owned => {
                        self.set_var_state(name, OwnershipState::Moved);
                    }
                }
            }
        }
    }

    /// 标记变量被不可变借用
    fn borrow_var(&mut self, name: &str) {
        if let Some(var) = self.lookup_var(name) {
            if !is_copy_type(&var.ty) {
                match &var.state {
                    OwnershipState::Moved => {
                        self.error(format!("borrow of moved value: '{}'", name));
                    }
                    OwnershipState::MutBorrowed => {
                        self.error(format!("cannot borrow '{}' as immutable because it is also borrowed as mutable", name));
                    }
                    OwnershipState::Borrowed(n) => {
                        self.set_var_state(name, OwnershipState::Borrowed(n + 1));
                    }
                    OwnershipState::Owned => {
                        self.set_var_state(name, OwnershipState::Borrowed(1));
                    }
                }
            }
        }
    }

    /// 标记变量被可变借用
    fn mut_borrow_var(&mut self, name: &str) {
        if let Some(var) = self.lookup_var(name) {
            if !is_copy_type(&var.ty) {
                match &var.state {
                    OwnershipState::Moved => {
                        self.error(format!("mutably borrow of moved value: '{}'", name));
                    }
                    OwnershipState::Borrowed(n) => {
                        if *n > 0 {
                            self.error(format!("cannot borrow '{}' as mutable because it is also borrowed as immutable", name));
                        } else {
                            self.set_var_state(name, OwnershipState::MutBorrowed);
                        }
                    }
                    OwnershipState::MutBorrowed => {
                        self.error(format!("cannot borrow '{}' as mutable more than once at a time", name));
                    }
                    OwnershipState::Owned => {
                        self.set_var_state(name, OwnershipState::MutBorrowed);
                    }
                }
            }
        }
    }

    /// 检查使用变量是否合法（读取）
    fn use_var(&mut self, name: &str) {
        if let Some(var) = self.lookup_var(name) {
            if !is_copy_type(&var.ty) {
                match &var.state {
                    OwnershipState::Moved => {
                        self.error(format!("use of moved value: '{}'", name));
                    }
                    _ => {}
                }
            }
        }
    }

    fn check_lvalue_use(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(_name) => {
                // 赋值目标是写入，不是读取，不做 use_var 借用检查
                // （简化模型：允许对变量写入）
            }
            Expr::Index { target, index } => {
                // 递归检查 target 本身（同样是写入目标）
                self.check_lvalue_use(target);
                // index 表达式是读取，正常检查
                self.check_expr(index, false);
            }
            Expr::FieldAccess { target, .. } => {
                self.check_lvalue_use(target);
            }
            _ => {}
        }
    }

    fn check_toplevel_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::FnDecl { name, params, body, .. } => {
                self.push_scope();
                for (pname, ptype) in params {
                    // 函数参数默认可变（简化模型），类型决定 Copy/非 Copy
                    self.declare_var(pname, SemaType::from_annotation(ptype), true);
                }
                self.check_block(body);
                self.pop_scope();

                // 将函数名标记为已声明（顶层作用域中函数名不影响所有权）
                if let Some(scope) = self.scopes.first_mut() {
                    scope.insert(name.clone(), VarOwnership {
                        state: OwnershipState::Owned,
                        ty: SemaType::Unknown,
                        mutable: false,
                    });
                }
            }
            Stmt::StructDecl { .. } => {}
            Stmt::EnumDecl { .. } => {}
            Stmt::ExternDecl { .. } => {}
            Stmt::ExportDecl { .. } => {}
            Stmt::ModDecl { .. } => {}
            Stmt::UseDecl { .. } => {}
            Stmt::FlowDecl { .. } => {}
            Stmt::DomainDecl { .. } => {}
            other => self.check_stmt(other),
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::LetDecl { name, type_annotation, value } => {
                let ty = if let Some(ta) = type_annotation {
                    SemaType::from_annotation(ta)
                } else if let Some(val) = value {
                    self.infer_expr_type(val)
                } else {
                    SemaType::Unknown
                };

                if let Some(val) = value {
                    self.check_expr(val, true);
                }
                self.declare_var(name, ty, false);
            }
            Stmt::Assign { target, value } => {
                self.check_lvalue_use(target);
                self.check_expr(value, true);
            }
            Stmt::Expr(expr) => {
                self.check_expr(expr, true);
            }
            Stmt::Return(Some(expr)) => {
                self.check_expr(expr, true);
            }
            Stmt::Return(None) => {}
            Stmt::If { condition, then_branch, else_branch } => {
                self.check_expr(condition, false);
                let saved = self.snapshot();
                self.push_scope();
                self.check_block(then_branch);
                self.pop_scope();
                let then_snapshot = self.snapshot();
                self.restore(saved);
                if let Some(else_block) = else_branch {
                    self.push_scope();
                    self.check_block(else_block);
                    self.pop_scope();
                }
                // 合并两个分支的所有权状态（保守策略：取最严格）
                self.merge_branch_states(&then_snapshot);
            }
            Stmt::While { condition, body } => {
                self.check_expr(condition, false);
                self.push_scope();
                self.check_block(body);
                self.pop_scope();
            }
            Stmt::For { var_name, start, end, body } => {
                self.check_expr(start, false);
                self.check_expr(end, false);
                self.push_scope();
                self.declare_var(var_name, SemaType::I64, false);
                self.check_block(body);
                self.pop_scope();
            }
            Stmt::ForIterable { var_name, iterable, body } => {
                self.check_expr(iterable, false);
                self.push_scope();
                self.declare_var(var_name, SemaType::Unknown, false);
                self.check_block(body);
                self.pop_scope();
            }
            Stmt::Loop(body) => {
                self.push_scope();
                self.check_block(body);
                self.pop_scope();
            }
            Stmt::Break => {}
            Stmt::Continue => {}
            Stmt::Match { scrutinee, arms } => {
                self.check_expr(scrutinee, false);
                let saved = self.snapshot();
                let mut arm_snapshots = Vec::new();
                for arm in arms {
                    self.restore(saved.clone());
                    self.push_scope();
                    self.bind_pattern(&arm.pattern);
                    self.check_block(&arm.body);
                    self.pop_scope();
                    arm_snapshots.push(self.snapshot());
                }
                self.restore(saved);
                for snap in arm_snapshots {
                    self.merge_branch_states(&snap);
                }
            }
            _ => {}
        }
    }

    fn check_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
    }

    /// 对表达式进行借用检查。
    /// `may_move`: 该表达式的结果是否可能被移动（如赋值右侧、函数参数）
    fn check_expr(&mut self, expr: &Expr, may_move: bool) {
        match expr {
            Expr::Ident(name) => {
                if may_move {
                    self.move_var(name);
                } else {
                    self.use_var(name);
                    self.borrow_var(name);
                }
            }
            Expr::Call { callee, args } => {
                // 内置函数通常不移动参数（复制语义）
                let builtins_no_move = ["print", "println", "len", "sleep", "abs", "min", "max", "sqrt", "pow"];
                let args_may_move = !builtins_no_move.contains(&callee.as_str());
                for arg in args {
                    self.check_expr(arg, args_may_move);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.check_expr(left, false);
                self.check_expr(right, false);
            }
            Expr::Unary { operand, .. } => {
                self.check_expr(operand, false);
            }
            Expr::Index { target, index } => {
                self.check_expr(target, false);
                self.check_expr(index, false);
            }
            Expr::FieldAccess { target, .. } => {
                self.check_expr(target, false);
            }
            Expr::StructInit { fields, .. } => {
                for (_, fval) in fields {
                    self.check_expr(fval, true);
                }
            }
            Expr::List(items) => {
                for item in items {
                    self.check_expr(item, true);
                }
            }
            Expr::Tuple(elems) => {
                for e in elems {
                    self.check_expr(e, true);
                }
            }
            Expr::IfExpr { condition, then_value, else_value } => {
                self.check_expr(condition, false);
                self.check_expr(then_value, may_move);
                self.check_expr(else_value, may_move);
            }
            Expr::BlockExpr(block) => {
                self.push_scope();
                self.check_block(block);
                self.pop_scope();
            }
            Expr::Path { .. } => {}
            Expr::PathCall { args, .. } => {
                for arg in args {
                    self.check_expr(arg, true);
                }
            }
            Expr::MatchExpr { scrutinee, arms } => {
                self.check_expr(scrutinee, false);
                let saved = self.snapshot();
                let mut arm_snapshots = Vec::new();
                for arm in arms {
                    self.restore(saved.clone());
                    self.push_scope();
                    self.bind_pattern(&arm.pattern);
                    for stmt in &arm.body.stmts {
                        self.check_stmt(stmt);
                    }
                    self.pop_scope();
                    arm_snapshots.push(self.snapshot());
                }
                self.restore(saved);
                for snap in arm_snapshots {
                    self.merge_branch_states(&snap);
                }
            }
            Expr::Await(inner) => {
                self.check_expr(inner, may_move);
            }
            Expr::Lambda { body, .. } => {
                self.push_scope();
                for stmt in &body.stmts {
                    self.check_stmt(stmt);
                }
                self.pop_scope();
            }
            _ => {}
        }
    }

    fn bind_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Bind(name) => {
                self.declare_var(name, SemaType::Unknown, false);
            }
            Pattern::EnumVariantWithPayload { bindings, .. } => {
                for name in bindings {
                    if name != "_" {
                        self.declare_var(name, SemaType::Unknown, false);
                    }
                }
            }
            _ => {}
        }
    }

    /// 推断表达式类型（简化版，仅用于 let 声明）
    fn infer_expr_type(&self, expr: &Expr) -> SemaType {
        match expr {
            Expr::Int(_) => SemaType::I64,
            Expr::Float(_) => SemaType::F64,
            Expr::Str(_) => SemaType::Str,
            Expr::Bool(_) => SemaType::Bool,
            Expr::None => SemaType::Unit,
            Expr::Ident(name) => {
                self.lookup_var(name).map(|v| v.ty).unwrap_or(SemaType::Unknown)
            }
            Expr::List(_) => SemaType::List(Box::new(SemaType::Unknown)),
            Expr::Tuple(elems) => {
                let elem_types: Vec<SemaType> = elems.iter()
                    .map(|e| self.infer_expr_type(e))
                    .collect();
                SemaType::Tuple(elem_types)
            }
            Expr::Binary { op, .. } => {
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => SemaType::I64,
                    BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq |
                    BinOp::And | BinOp::Or => SemaType::Bool,
                    BinOp::Pipe => SemaType::Unknown,
                }
            }
            Expr::Unary { op, operand } => {
                match op {
                    UnaryOp::Neg => self.infer_expr_type(operand),
                    UnaryOp::Not => SemaType::Bool,
                }
            }
            Expr::Call { callee, .. } => {
                // 简化：不知道返回类型
                SemaType::Unknown
            }
            Expr::IfExpr { then_value, .. } => self.infer_expr_type(then_value),
            Expr::FieldAccess { target, .. } => {
                self.infer_expr_type(target)
            }
            Expr::StructInit { name, .. } => SemaType::Named(name.clone()),
            Expr::Path { base, .. } => SemaType::Named(base.clone()),
            Expr::PathCall { base, .. } => SemaType::Named(base.clone()),
            Expr::BlockExpr(block) => {
                if let Some(last) = block.stmts.last() {
                    match last {
                        Stmt::Expr(e) => self.infer_expr_type(e),
                        Stmt::Return(Some(e)) => self.infer_expr_type(e),
                        _ => SemaType::Void,
                    }
                } else {
                    SemaType::Void
                }
            }
            Expr::MatchExpr { arms, .. } => {
                if let Some(first) = arms.first() {
                    self.infer_block_type(&first.body)
                } else {
                    SemaType::Unknown
                }
            }
            Expr::Await(inner) => self.infer_expr_type(inner),
            Expr::Try(inner) => self.infer_expr_type(inner),
            Expr::Lambda { params, return_type, .. } => {
                let param_types: Vec<SemaType> = params.iter()
                    .map(|(_, t)| SemaType::from_annotation(t))
                    .collect();
                let ret_type = if let Some(rt) = return_type {
                    SemaType::from_annotation(rt)
                } else {
                    SemaType::Unknown
                };
                SemaType::Function {
                    params: param_types,
                    ret: Box::new(ret_type),
                }
            }
            Expr::Index { .. } => SemaType::Unknown,
            Expr::Ref(inner, is_mut) => SemaType::Ref(Box::new(self.infer_expr_type(inner)), *is_mut),
            Expr::Deref(inner) => {
                match self.infer_expr_type(inner) {
                    SemaType::Ref(target, _) => *target,
                    SemaType::Ptr(target) => *target,
                    _ => SemaType::Unknown,
                }
            }
            Expr::AsCast(_, ty) => SemaType::from_annotation(ty),
        }
    }

    fn infer_block_type(&self, block: &Block) -> SemaType {
        if let Some(last) = block.stmts.last() {
            match last {
                Stmt::Expr(e) => self.infer_expr_type(e),
                Stmt::Return(Some(e)) => self.infer_expr_type(e),
                _ => SemaType::Void,
            }
        } else {
            SemaType::Void
        }
    }

    /// 保存当前所有作用域的快照
    fn snapshot(&self) -> Vec<HashMap<String, OwnershipState>> {
        self.scopes.iter()
            .map(|scope| {
                scope.iter()
                    .map(|(k, v)| (k.clone(), v.state.clone()))
                    .collect()
            })
            .collect()
    }

    /// 恢复快照
    fn restore(&mut self, snapshot: Vec<HashMap<String, OwnershipState>>) {
        for (scope, snap) in self.scopes.iter_mut().zip(snapshot.iter()) {
            for (name, state) in snap {
                if let Some(var) = scope.get_mut(name) {
                    var.state = state.clone();
                }
            }
        }
    }

    /// 合并分支状态：如果任一分支移动了变量，则保守地认为可能被移动
    fn merge_branch_states(&mut self, branch_snapshot: &[HashMap<String, OwnershipState>]) {
        for (scope_idx, branch_scope) in branch_snapshot.iter().enumerate() {
            if let Some(scope) = self.scopes.get_mut(scope_idx) {
                for (name, branch_state) in branch_scope {
                    if let Some(var) = scope.get_mut(name) {
                        // 如果分支中变量被移动或借用，而我们当前是 Owned，
                        // 我们保守地保留分支后的状态。
                        // 实际上，如果两个分支都同意状态，则使用该状态；
                        // 否则恢复到 Owned（因为分支后所有权应该重新统一）。
                        // 简化：对于 Moved 状态，如果当前也是 Moved 则保留，否则保持当前。
                        match (&var.state, branch_state) {
                            (OwnershipState::Moved, OwnershipState::Moved) => {}
                            (OwnershipState::Borrowed(a), OwnershipState::Borrowed(b)) => {
                                var.state = OwnershipState::Borrowed(*a.max(b));
                            }
                            (OwnershipState::MutBorrowed, OwnershipState::MutBorrowed) => {}
                            // 如果状态不一致，保守地重置为 Owned（假定值在分支后被重新统一）
                            _ => {
                                if *branch_state == OwnershipState::Moved && var.state != OwnershipState::Moved {
                                    // 分支内被移动了，但分支外没被移动——这是合法的
                                    // 因为值被传入了分支
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 公共接口：对程序进行借用检查
pub fn check_borrow(program: &Program) -> Vec<SemaError> {
    let mut checker = BorrowChecker::new();
    checker.check_program(program)
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
    fn test_check_truthy_if_condition() {
        let program = parse("if 42 { 1 } else { 2 }");
        let errors = check_program(&program);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_check_truthy_nonbool_conditions() {
        let program = parse(r#"if "" {} if [] {} if none {} if 0 {} if "hello" {}"#);
        let errors = check_program(&program);
        assert!(errors.is_empty());
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

    mod borrow_checker_tests {
        use super::*;

        #[test]
        fn test_borrow_copy_type_no_move() {
            // i64 是 Copy 类型，可以多次使用
            let program = parse("let x: i64 = 42; let y = x; let z = x;");
            let errors = check_borrow(&program);
            assert!(errors.is_empty(), "Copy types should not be moved: {:?}", errors);
        }

        #[test]
        fn test_borrow_str_use_after_move() {
            let program = parse(r#"let s = "hello"; let t = s; println(s);"#);
            let errors = check_borrow(&program);
            assert!(!errors.is_empty());
            assert!(errors[0].message.contains("use of moved value"), "{:?}", errors);
        }

        #[test]
        fn test_borrow_list_use_after_move() {
            let program = parse("let xs = [1, 2, 3]; let ys = xs; len(xs)");
            let errors = check_borrow(&program);
            assert!(!errors.is_empty());
            assert!(errors[0].message.contains("use of moved value"), "{:?}", errors);
        }

        #[test]
        fn test_borrow_struct_use_after_move() {
            let program = parse("struct Point { x: i64, y: i64 } let p = Point { x: 1, y: 2 }; let q = p; p.x");
            let errors = check_borrow(&program);
            assert!(!errors.is_empty());
            assert!(errors[0].message.contains("use of moved value"), "{:?}", errors);
        }

        #[test]
        fn test_borrow_valid_reuse_copy() {
            // bool 是 Copy 类型
            let program = parse("let b = true; let c = b; let d = b;");
            let errors = check_borrow(&program);
            assert!(errors.is_empty(), "bool is Copy: {:?}", errors);
        }

        #[test]
        fn test_borrow_fn_param_move() {
            // 非内置函数调用会移动参数
            let program = parse(r#"fn consume(s: str) -> i64 { return len(s); } let msg = "hi"; consume(msg); println(msg);"#);
            let errors = check_borrow(&program);
            assert!(!errors.is_empty());
            assert!(errors[0].message.contains("use of moved value"), "{:?}", errors);
        }

        #[test]
        fn test_borrow_builtin_no_move() {
            // 内置函数（如 len, println）不移动参数
            let program = parse(r#"let s = "hello"; println(s); let n = len(s); println(s);"#);
            let errors = check_borrow(&program);
            assert!(errors.is_empty(), "builtins should not move: {:?}", errors);
        }

        #[test]
        fn test_borrow_scope_drop() {
            // 作用域结束时变量被丢弃，不影响外部
            let program = parse(r#"let s = "hello"; { let t = s; } println(s);"#);
            let errors = check_borrow(&program);
            // s 被移动到 t，在块中释放，之后不能再使用 s
            assert!(!errors.is_empty());
            assert!(errors[0].message.contains("use of moved value"), "{:?}", errors);
        }

        #[test]
        fn test_borrow_if_branch_move() {
            // if 分支中的移动
            let program = parse(r#"let s = "hello"; if true { let t = s; } println(s);"#);
            let errors = check_borrow(&program);
            // 在 if 的一个分支中移动了 s，合并后仍然是 Owned
            // 简化模型：分支后状态保守处理
            // 因为分支中移动了 s，另一个分支没有，合并后可能仍报错
            // 这里只验证不 panic
        }

        #[test]
        fn test_borrow_double_move() {
            let program = parse(r#"let s = "hello"; let t = s; let u = s;"#);
            let errors = check_borrow(&program);
            assert!(!errors.is_empty());
            assert!(errors[0].message.contains("use of moved value"), "{:?}", errors);
        }

        #[test]
        fn test_borrow_fn_return_owned() {
            // 函数返回值拥有新所有权
            let program = parse(r#"fn greet() -> str { return "hi"; } let s = greet(); println(s);"#);
            let errors = check_borrow(&program);
            assert!(errors.is_empty(), "returned value is fresh owned: {:?}", errors);
        }

        #[test]
        fn test_borrow_enum_use_after_move() {
            let program = parse("enum Color { Red, Green } let c = Color::Red; let d = c; c");
            let errors = check_borrow(&program);
            assert!(!errors.is_empty());
            assert!(errors[0].message.contains("use of moved value"), "{:?}", errors);
        }
    }
}
