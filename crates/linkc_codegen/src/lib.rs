use linkc_parser::*;
use std::collections::HashMap;

pub trait CodeGenerator {
    fn generate(&mut self, program: &Program) -> Result<String, String>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptLevel {
    O0,
    O1,
    O2,
    O3,
}

impl OptLevel {
    pub fn as_c_flag(&self) -> &str {
        match self {
            OptLevel::O0 => "-O0",
            OptLevel::O1 => "-O1",
            OptLevel::O2 => "-O2",
            OptLevel::O3 => "-O3",
        }
    }

    pub fn as_msvc_flag(&self) -> &str {
        match self {
            OptLevel::O0 => "/Od",
            OptLevel::O1 => "/O1",
            OptLevel::O2 => "/O2",
            OptLevel::O3 => "/O3",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "0" | "O0" | "o0" => Ok(OptLevel::O0),
            "1" | "O1" | "o1" => Ok(OptLevel::O1),
            "2" | "O2" | "o2" => Ok(OptLevel::O2),
            "3" | "O3" | "o3" => Ok(OptLevel::O3),
            _ => Err(format!("Unknown optimization level: '{}' (use 0-3)", s)),
        }
    }
}

pub struct CBackend {
    indent: usize,
    functions: Vec<String>,
    globals: Vec<String>,
    struct_defs: Vec<String>,
    enum_defs: Vec<String>,
    var_map: HashMap<String, String>,
    var_type_map: HashMap<String, String>,
    struct_map: HashMap<String, Vec<(String, String)>>,
    enum_map: HashMap<String, Vec<(String, usize, Vec<String>)>>,
    fn_return_types: HashMap<String, String>,
    tmp_counter: usize,
    has_main: bool,
    #[allow(dead_code)]
    opt_level: OptLevel,
    debug_info: bool,
}

impl CBackend {
    pub fn new(opt_level: OptLevel, debug_info: bool) -> Self {
        Self {
            indent: 0,
            functions: Vec::new(),
            globals: Vec::new(),
            struct_defs: Vec::new(),
            enum_defs: Vec::new(),
            var_map: HashMap::new(),
            var_type_map: HashMap::new(),
            struct_map: HashMap::new(),
            enum_map: HashMap::new(),
            fn_return_types: HashMap::new(),
            tmp_counter: 0,
            has_main: false,
            opt_level,
            debug_info,
        }
    }

    pub fn new_with_defaults() -> Self {
        Self::new(OptLevel::O2, false)
    }

    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }

    fn map_type(type_ann: &TypeAnnotation) -> Result<String, String> {
        match type_ann {
            TypeAnnotation::I8 => Ok("int8_t".to_string()),
            TypeAnnotation::I16 => Ok("int16_t".to_string()),
            TypeAnnotation::I32 => Ok("int32_t".to_string()),
            TypeAnnotation::I64 => Ok("int64_t".to_string()),
            TypeAnnotation::U8 => Ok("uint8_t".to_string()),
            TypeAnnotation::U16 => Ok("uint16_t".to_string()),
            TypeAnnotation::U32 => Ok("uint32_t".to_string()),
            TypeAnnotation::U64 => Ok("uint64_t".to_string()),
            TypeAnnotation::USize => Ok("size_t".to_string()),
            TypeAnnotation::F32 => Ok("float".to_string()),
            TypeAnnotation::F64 => Ok("double".to_string()),
            TypeAnnotation::Bool => Ok("bool".to_string()),
            TypeAnnotation::Str => Ok("const char*".to_string()),
            TypeAnnotation::Unit => Ok("void".to_string()),
            TypeAnnotation::Void => Ok("void".to_string()),
            TypeAnnotation::Ptr(inner) => {
                let inner = Self::map_type(inner)?;
                Ok(format!("{}*", inner))
            }
            TypeAnnotation::Named(name) => Ok(name.clone()),
            TypeAnnotation::Stream(_) => {
                Ok("struct LinkStream*".to_string())
            }
            TypeAnnotation::Ref(inner, _) => Self::map_type(inner),
            TypeAnnotation::Generic(base, _) => Self::map_type(base),
            TypeAnnotation::Array(elem, _) => {
                let e = Self::map_type(elem)?;
                Ok(format!("{}*", e))
            }
            TypeAnnotation::Tuple(elems) => {
                let fields: Vec<String> = elems.iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let t = Self::map_type(e).unwrap_or_else(|_| "int64_t".to_string());
                        format!("{} f{}", t, i)
                    })
                    .collect();
                Ok(format!("struct {{ {} }}", fields.join("; ")))
            }
        }
    }

    fn default_type() -> String {
        "int64_t".to_string()
    }

    fn infer_type_from_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Int(_) => "int64_t".to_string(),
            Expr::Float(_) => "double".to_string(),
            Expr::Str(_) => "const char*".to_string(),
            Expr::Bool(_) => "bool".to_string(),
            Expr::None => "void*".to_string(),
            Expr::List(_) => "LinkList".to_string(),
            Expr::Ident(name) => {
                self.var_type_map.get(name).cloned().unwrap_or_else(Self::default_type)
            }
            Expr::StructInit { name, .. } => name.clone(),
            Expr::Path { base, .. } | Expr::PathCall { base, .. } => base.clone(),
            Expr::Binary { left, right, .. } => {
                let lt = self.infer_type_from_expr(left);
                let rt = self.infer_type_from_expr(right);
                if lt == "double" || rt == "double" {
                    "double".to_string()
                } else if lt == "const char*" || rt == "const char*" {
                    "const char*".to_string()
                } else {
                    "int64_t".to_string()
                }
            }
            Expr::FieldAccess { target, field } => {
                if let Expr::Ident(name) = target.as_ref() {
                    if let Some(c_type) = self.var_type_map.get(name) {
                        if let Some(fields) = self.struct_map.get(c_type) {
                            for (fname, ftype) in fields {
                                if fname == field {
                                    return ftype.clone();
                                }
                            }
                        }
                    }
                }
                Self::default_type()
            }
            Expr::Call { callee, .. } => {
                match callee.as_str() {
                    "len" => "int64_t".to_string(),
                    _ => self.fn_return_types.get(callee)
                        .cloned()
                        .unwrap_or_else(Self::default_type),
                }
            }
            _ => Self::default_type(),
        }
    }

    fn fresh_tmp(&mut self) -> String {
        self.tmp_counter += 1;
        format!("_tmp_{}", self.tmp_counter)
    }

    fn generate_struct_def(&mut self, name: &str, fields: &[StructField]) -> String {
        let mut def = format!("typedef struct {{\n");
        self.indent = 1;
        for field in fields {
            let c_type = Self::map_type(&field.type_ann).unwrap_or_else(|_| "int64_t".to_string());
            def.push_str(&format!("{}{} {};\n", self.indent_str(), c_type, field.name));
        }
        self.indent = 0;
        def.push_str(&format!("}} {};\n", name));
        def
    }

    fn generate_enum_def(&mut self, name: &str, variants: &[EnumVariantDecl]) -> String {
        let mut def = String::new();

        def.push_str(&format!("typedef struct {{\n"));
        def.push_str(&format!("    int32_t discriminant;\n"));

        let has_payload = variants.iter().any(|v| !v.payload.is_empty());
        if has_payload {
            def.push_str(&format!("    union {{\n"));
            self.indent = 2;
            for variant in variants {
                if variant.payload.is_empty() {
                    def.push_str(&format!("{}/* {} has no payload */\n", self.indent_str(), variant.name));
                } else if variant.payload.len() == 1 {
                    let c_type = Self::map_type(&variant.payload[0]).unwrap_or_else(|_| "int64_t".to_string());
                    def.push_str(&format!("{}struct {{ {} field0; }} {};\n", self.indent_str(), c_type, variant.name));
                } else {
                    let mut field_lines = Vec::new();
                    for (i, pt) in variant.payload.iter().enumerate() {
                        let c_type = Self::map_type(pt).unwrap_or_else(|_| "int64_t".to_string());
                        field_lines.push(format!("{} {};", c_type, format!("field{}", i)));
                    }
                    def.push_str(&format!("{}struct {{ {} }} {};\n", self.indent_str(), field_lines.join(" "), variant.name));
                }
            }
            self.indent = 0;
            def.push_str(&format!("    }} data;\n"));
        }
        def.push_str(&format!("}} {};\n", name));
        self.indent = 0;

        def.push_str(&format!("enum {}_variants {{\n", name));
        for (i, variant) in variants.iter().enumerate() {
            def.push_str(&format!("    {}_v_{} = {},\n", name, variant.name, i));
        }
        def.push_str(&format!("}};\n"));

        def
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Int(n) => Ok(format!("{}LL", n)),
            Expr::Float(f) => {
                let s = format!("{}", f);
                if s.contains('.') || s.contains('e') || s.contains('E') {
                    Ok(s)
                } else {
                    Ok(format!("{}.0", s))
                }
            }
            Expr::Str(s) => {
                let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                Ok(format!("\"{}\"", escaped))
            }
            Expr::Bool(b) => Ok(if *b { "true" } else { "false" }.to_string()),
            Expr::None => Ok("0".to_string()),
            Expr::Ident(name) => {
                if let Some(c_name) = self.var_map.get(name) {
                    Ok(c_name.clone())
                } else {
                    Err(format!("Undefined variable: {}", name))
                }
            }
            Expr::Binary { op, left, right } => {
                let left_str = self.generate_expr(left)?;
                let right_str = self.generate_expr(right)?;
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Mod => "%",
                    BinOp::Eq => "==",
                    BinOp::Neq => "!=",
                    BinOp::Lt => "<",
                    BinOp::Gt => ">",
                    BinOp::LtEq => "<=",
                    BinOp::GtEq => ">=",
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                    BinOp::Pipe => {
                        return Err("pipe operator not directly supported in C backend (use stream functions)".to_string());
                    }
                };
                Ok(format!("({} {} {})", left_str, op_str, right_str))
            }
            Expr::Unary { op, operand } => {
                let op_str = match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "!",
                };
                let operand_str = self.generate_expr(operand)?;
                Ok(format!("{}({})", op_str, operand_str))
            }
            Expr::Call { callee, args } => {
                match callee.as_str() {
                    "print" | "println" => {
                        let is_println = callee == "println";
                        if args.is_empty() {
                            if is_println {
                                Ok("puts(\"\")".to_string())
                            } else {
                                Ok("(void)0".to_string())
                            }
                        } else if args.len() >= 2 {
                            if let Expr::Str(fmt) = &args[0] {
                                if fmt.contains("{}") {
                                    let format_str = fmt.replace('\\', "\\\\").replace('"', "\\\"");
                                    let mut arg_strs = Vec::new();
                                    let mut arg_idx = 1;
                                    let mut result = String::new();
                                    let chars: Vec<char> = format_str.chars().collect();
                                    let mut i = 0;
                                    while i < chars.len() {
                                        if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '}' {
                                            if arg_idx < args.len() {
                                                let arg = &args[arg_idx];
                                                let arg_expr = self.generate_expr(arg)?;
                                                match arg {
                                                    Expr::Int(_) => {
                                                        result.push_str("%lld");
                                                        arg_strs.push(format!("(long long)({})", arg_expr));
                                                    }
                                                    Expr::Float(_) => {
                                                        result.push_str("%lf");
                                                        arg_strs.push(arg_expr);
                                                    }
                                                    Expr::Bool(_) => {
                                                        result.push_str("%s");
                                                        arg_strs.push(format!("({} ? \"true\" : \"false\")", arg_expr));
                                                    }
                                                    Expr::Str(_) => {
                                                        result.push_str("%s");
                                                        arg_strs.push(arg_expr);
                                                    }
                                                    _ => {
                                                        let t = self.infer_type_from_expr(arg);
                                                        if t == "double" {
                                                            result.push_str("%lf");
                                                            arg_strs.push(arg_expr);
                                                        } else if t == "const char*" {
                                                            result.push_str("%s");
                                                            arg_strs.push(arg_expr);
                                                        } else {
                                                            result.push_str("%lld");
                                                            arg_strs.push(format!("(long long)({})", arg_expr));
                                                        }
                                                    }
                                                }
                                                arg_idx += 1;
                                            }
                                            i += 2;
                                        } else {
                                            result.push(chars[i]);
                                            i += 1;
                                        }
                                    }
                                    if is_println {
                                        result.push_str("\\n");
                                    }
                                    if arg_strs.is_empty() {
                                        Ok(format!("printf(\"{}\")", result))
                                    } else {
                                        Ok(format!("printf(\"{}\", {})", result, arg_strs.join(", ")))
                                    }
                                } else {
                                    self.generate_print_simple(args, is_println)
                                }
                            } else {
                                self.generate_print_simple(args, is_println)
                            }
                        } else {
                            self.generate_print_simple(args, is_println)
                        }
                    }
                    "len" => {
                        if args.len() != 1 {
                            return Err("len() takes exactly 1 argument".to_string());
                        }
                        let arg = &args[0];
                        let arg_str = self.generate_expr(arg)?;
                        match arg {
                            Expr::Str(_) => {
                                Ok(format!("strlen({})", arg_str))
                            }
                            Expr::List(_) => {
                                Ok(format!("({}).count", arg_str))
                            }
                            _ => {
                                Ok(format!("({}).count", arg_str))
                            }
                        }
                    }
                    "sleep" => {
                        if args.len() != 1 {
                            return Err("sleep() takes exactly 1 argument".to_string());
                        }
                        let arg_str = self.generate_expr(&args[0])?;
                        Ok(format!("(Sleep((DWORD)({})), (void)0)", arg_str))
                    }
                    _ => {
                        let mut arg_strs = Vec::new();
                        for arg in args {
                            arg_strs.push(self.generate_expr(arg)?);
                        }
                        Ok(format!("{}({})", callee, arg_strs.join(", ")))
                    }
                }
            }
            Expr::IfExpr { condition, then_value, else_value } => {
                let cond_str = self.generate_expr(condition)?;
                let then_str = self.generate_expr(then_value)?;
                let else_str = self.generate_expr(else_value)?;
                Ok(format!("({} ? {} : {})", cond_str, then_str, else_str))
            }
            Expr::List(items) => {
                let _tmp = self.fresh_tmp();
                let count = items.len();
                let mut init_exprs = Vec::new();
                for item in items {
                    init_exprs.push(self.generate_expr(item)?);
                }
                Ok(format!("((LinkList){{ .count = {}, .items = {{ {} }} }})", count, init_exprs.join(", ")))
            }
            Expr::Index { target, index } => {
                let target_str = self.generate_expr(target)?;
                let index_str = self.generate_expr(index)?;
                Ok(format!("{}.items[{}]", target_str, index_str))
            }
            Expr::BlockExpr(block) => {
                let tmp = self.fresh_tmp();
                let tmp_type = Self::default_type();
                self.var_map.insert(tmp.clone(), tmp.clone());
                let mut code = format!("{} {};\n", tmp_type, tmp);
                let saved_indent = self.indent;
                self.indent = 1;
                code.push_str(&self.generate_block_with_assign(block, &tmp)?);
                self.indent = saved_indent;
                Ok(format!("({})", tmp))
            }
            Expr::StructInit { name, fields } => {
                let mut field_inits = Vec::new();
                for (fname, fval) in fields {
                    let val_str = self.generate_expr(fval)?;
                    field_inits.push(format!(".{} = {}", fname, val_str));
                }
                Ok(format!("(({}){{ {} }})", name, field_inits.join(", ")))
            }
            Expr::FieldAccess { target, field } => {
                let target_str = self.generate_expr(target)?;
                Ok(format!("{}.{}", target_str, field))
            }
            Expr::Path { base, segment } => {
                Ok(format!("(({base}){{ .discriminant = {variant_name}_v_{variant} }})",
                    base = base, variant_name = base, variant = segment))
            }
            Expr::PathCall { base, segment, args } => {
                let mut arg_strs = Vec::new();
                for arg in args {
                    arg_strs.push(self.generate_expr(arg)?);
                }
                Ok(format!("(({base}){{ .discriminant = {variant_name}_v_{variant}, .data.{variant} = {{ {fields} }} }})",
                    base = base, variant_name = base, variant = segment, fields = arg_strs.join(", ")))
            }
            Expr::MatchExpr { .. } => {
                Err("match expressions not supported in C backend yet".to_string())
            }
            Expr::Await(_) => {
                Err("await not supported in C backend (use async runtime)".to_string())
            }
            Expr::Try(_) => {
                Err("try! not supported in C backend yet".to_string())
            }
            Expr::Lambda { .. } => {
                Err("lambda/anonymous functions not supported in C backend".to_string())
            }
            Expr::Ref(inner, _) => self.generate_expr(inner),
            Expr::Deref(inner) => self.generate_expr(inner),
            Expr::AsCast(expr, _) => self.generate_expr(expr),
            Expr::Tuple(elems) => {
                let mut parts = Vec::new();
                for e in elems {
                    parts.push(self.generate_expr(e)?);
                }
                Ok(format!("((struct {{ /* tuple */ }}){{ /* {} */ }}", parts.join(", ")))
            }
        }
    }

    fn generate_print_simple(&mut self, args: &[Expr], is_println: bool) -> Result<String, String> {
        let mut format_str = String::new();
        let mut arg_strs = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                format_str.push_str(" ");
            }
            let arg_expr = self.generate_expr(arg)?;
            match arg {
                Expr::Int(_) => {
                    format_str.push_str("%lld");
                    arg_strs.push(format!("(long long)({})", arg_expr));
                }
                Expr::Float(_) => {
                    format_str.push_str("%lf");
                    arg_strs.push(arg_expr);
                }
                Expr::Bool(_) => {
                    format_str.push_str("%s");
                    arg_strs.push(format!("({} ? \"true\" : \"false\")", arg_expr));
                }
                Expr::Str(_) => {
                    format_str.push_str("%s");
                    arg_strs.push(arg_expr);
                }
                _ => {
                    let t = self.infer_type_from_expr(arg);
                    if t == "double" {
                        format_str.push_str("%lf");
                        arg_strs.push(arg_expr);
                    } else if t == "const char*" {
                        format_str.push_str("%s");
                        arg_strs.push(arg_expr);
                    } else {
                        format_str.push_str("%lld");
                        arg_strs.push(format!("(long long)({})", arg_expr));
                    }
                }
            }
        }
        if is_println {
            format_str.push_str("\\n");
        }
        if arg_strs.is_empty() {
            Ok(format!("printf(\"{}\")", format_str))
        } else {
            Ok(format!("printf(\"{}\", {})", format_str, arg_strs.join(", ")))
        }
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<Vec<String>, String> {
        let mut lines = Vec::new();

        match stmt {
            Stmt::LetDecl { name, type_annotation, value } => {
                let c_type = if let Some(ta) = type_annotation {
                    Self::map_type(ta)?
                } else if let Some(val) = value {
                    self.infer_type_from_expr(val)
                } else {
                    Self::default_type()
                };
                let c_name = name.clone();
                self.var_map.insert(name.clone(), c_name.clone());
                self.var_type_map.insert(name.clone(), c_type.clone());

                if let Some(val) = value {
                    let val_str = self.generate_expr(val)?;
                    lines.push(format!("{}{} {} = {};", self.indent_str(), c_type, c_name, val_str));
                } else {
                    lines.push(format!("{}{} {};", self.indent_str(), c_type, c_name));
                }
            }
            Stmt::Assign { target, value } => {
                let target_str = self.generate_expr(target)?;
                let val_str = self.generate_expr(value)?;
                lines.push(format!("{}{} = {};", self.indent_str(), target_str, val_str));
            }
            Stmt::Expr(expr) => {
                let expr_str = self.generate_expr(expr)?;
                lines.push(format!("{}{};", self.indent_str(), expr_str));
            }
            Stmt::Return(Some(expr)) => {
                let expr_str = self.generate_expr(expr)?;
                lines.push(format!("{}return {};", self.indent_str(), expr_str));
            }
            Stmt::Return(None) => {
                lines.push(format!("{}return;", self.indent_str()));
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let cond_str = self.generate_expr(condition)?;
                lines.push(format!("{}if ({}) {{", self.indent_str(), cond_str));
                self.indent += 1;
                let then_lines = self.generate_block_lines(then_branch)?;
                lines.extend(then_lines);
                self.indent -= 1;

                if let Some(else_block) = else_branch {
                    lines.push(format!("{}}} else {{", self.indent_str()));
                    self.indent += 1;
                    let else_lines = self.generate_block_lines(else_block)?;
                    lines.extend(else_lines);
                    self.indent -= 1;
                }
                lines.push(format!("{}}}", self.indent_str()));
            }
            Stmt::While { condition, body } => {
                let cond_str = self.generate_expr(condition)?;
                lines.push(format!("{}while ({}) {{", self.indent_str(), cond_str));
                self.indent += 1;
                let body_lines = self.generate_block_lines(body)?;
                lines.extend(body_lines);
                self.indent -= 1;
                lines.push(format!("{}}}", self.indent_str()));
            }
            Stmt::For { var_name, start, end, body } => {
                let start_str = self.generate_expr(start)?;
                let end_str = self.generate_expr(end)?;
                let c_name = var_name.clone();
                self.var_map.insert(var_name.clone(), c_name.clone());
                self.var_type_map.insert(var_name.clone(), "int64_t".to_string());

                lines.push(format!(
                    "{}for (int64_t {} = {}; {} < {}; {}++) {{",
                    self.indent_str(), c_name, start_str, c_name, end_str, c_name
                ));
                self.indent += 1;
                let body_lines = self.generate_block_lines(body)?;
                lines.extend(body_lines);
                self.indent -= 1;
                lines.push(format!("{}}}", self.indent_str()));
            }
            Stmt::ForIterable { var_name, iterable, body } => {
                let iter_str = self.generate_expr(iterable)?;
                let iter_cvar = format!("__iter_{}", self.tmp_counter);
                self.tmp_counter += 1;
                let idx_cvar = format!("__idx_{}", self.tmp_counter);
                self.tmp_counter += 1;
                let c_name = var_name.clone();
                self.var_map.insert(var_name.clone(), c_name.clone());
                self.var_type_map.insert(var_name.clone(), "LinkValue*".to_string());

                lines.push(format!("{}LinkValue* {} = {};", self.indent_str(), iter_cvar, iter_str));
                lines.push(format!("{}int64_t {} = 0;", self.indent_str(), idx_cvar));
                lines.push(format!("{}while ({} < list_len({})) {{", self.indent_str(), idx_cvar, iter_cvar));
                self.indent += 1;
                lines.push(format!("{}LinkValue* {} = list_get({}, {});", self.indent_str(), c_name, iter_cvar, idx_cvar));
                let body_lines = self.generate_block_lines(body)?;
                lines.extend(body_lines);
                lines.push(format!("{}{}++;", self.indent_str(), idx_cvar));
                self.indent -= 1;
                lines.push(format!("{}}}", self.indent_str()));
            }
            Stmt::Loop(body) => {
                lines.push(format!("{}while (1) {{", self.indent_str()));
                self.indent += 1;
                let body_lines = self.generate_block_lines(body)?;
                lines.extend(body_lines);
                self.indent -= 1;
                lines.push(format!("{}}}", self.indent_str()));
            }
            Stmt::Break => {
                lines.push(format!("{}break;", self.indent_str()));
            }
            Stmt::Continue => {
                lines.push(format!("{}continue;", self.indent_str()));
            }
            Stmt::FnDecl { name, params, return_type, body, is_async: _ } => {
                if name == "main" {
                    self.has_main = true;
                }

                let ret_type = if let Some(rt) = return_type {
                    Self::map_type(rt)?
                } else {
                    "void".to_string()
                };

                let mut param_strs = Vec::new();
                let mut fn_var_map = HashMap::new();
                let mut fn_var_type_map = HashMap::new();
                for (pname, ptype) in params {
                    let c_type = Self::map_type(ptype)?;
                    param_strs.push(format!("{} {}", c_type, pname));
                    fn_var_map.insert(pname.clone(), pname.clone());
                    fn_var_type_map.insert(pname.clone(), c_type);
                }

                let param_list = if param_strs.is_empty() {
                    "void".to_string()
                } else {
                    param_strs.join(", ")
                };

                let saved_var_map = std::mem::replace(&mut self.var_map, fn_var_map);
                let saved_var_type_map = std::mem::replace(&mut self.var_type_map, fn_var_type_map);
                let saved_indent = self.indent;
                self.indent = 1;

                let body_lines = self.generate_block_lines(body)?;

                self.var_map = saved_var_map;
                self.var_type_map = saved_var_type_map;
                self.indent = saved_indent;

                let mut fn_code = format!("{} {}({}) {{\n", ret_type, name, param_list);
                for line in &body_lines {
                    fn_code.push_str(line);
                    fn_code.push('\n');
                }
                fn_code.push_str("}\n");

                self.functions.push(fn_code);

                let forward_decl = format!("{} {}({});", ret_type, name, param_list);
                self.globals.push(forward_decl);
            }
            Stmt::StructDecl { name, fields } => {
                let mut field_info = Vec::new();
                for field in fields {
                    let c_type = Self::map_type(&field.type_ann).unwrap_or_else(|_| "int64_t".to_string());
                    field_info.push((field.name.clone(), c_type));
                }
                self.struct_map.insert(name.clone(), field_info.clone());

                let def = self.generate_struct_def(name, fields);
                self.struct_defs.push(def);
            }
            Stmt::EnumDecl { name, variants } => {
                let mut variant_info = Vec::new();
                for (i, variant) in variants.iter().enumerate() {
                    let payload_types: Vec<String> = variant.payload.iter()
                        .map(|pt| Self::map_type(pt).unwrap_or_else(|_| "int64_t".to_string()))
                        .collect();
                    variant_info.push((variant.name.clone(), i, payload_types));
                }
                self.enum_map.insert(name.clone(), variant_info);

                let def = self.generate_enum_def(name, variants);
                self.enum_defs.push(def);
            }
            Stmt::ExternDecl { language, module: _, decls } => {
                if language != "c" && language != "C" && language != "c++" && language != "cpp" {
                    return Err(format!("extern '{}' not supported in C backend (use 'c')", language));
                }
                for decl in decls {
                    let ret_type = if let Some(rt) = &decl.return_type {
                        Self::map_type(rt)?
                    } else {
                        "void".to_string()
                    };
                    let mut param_strs = Vec::new();
                    for (_, ptype) in &decl.params {
                        param_strs.push(Self::map_type(ptype)?);
                    }
                    let param_list = if param_strs.is_empty() {
                        "void".to_string()
                    } else {
                        param_strs.join(", ")
                    };
                    self.globals.push(format!("{} {}({});", ret_type, decl.name, param_list));
                }
            }
            Stmt::ExportDecl { .. } => {}
            Stmt::Match { scrutinee, arms } => {
                let tmp_scrutinee = self.fresh_tmp();
                let scrutinee_str = self.generate_expr(scrutinee)?;
                let scrut_type = self.infer_type_from_expr(scrutinee);
                self.var_map.insert(tmp_scrutinee.clone(), tmp_scrutinee.clone());
                lines.push(format!("{}{} {} = {};", self.indent_str(), scrut_type, tmp_scrutinee, scrutinee_str));

                let mut first = true;
                for arm in arms {
                    let arm_cond = self.generate_match_arm_condition(&arm.pattern, &tmp_scrutinee)?;
                    if first {
                        lines.push(format!("{}if ({}) {{", self.indent_str(), arm_cond));
                        first = false;
                    } else {
                        lines.push(format!("{}}} else if ({}) {{", self.indent_str(), arm_cond));
                    }
                    self.indent += 1;
                    let arm_lines = self.generate_block_lines(&arm.body)?;
                    lines.extend(arm_lines);
                    self.indent -= 1;
                }
                lines.push(format!("{}}}", self.indent_str()));
            }
            Stmt::FlowDecl { .. } => {
                return Err("flow not supported in C backend yet".to_string());
            }
            Stmt::DomainDecl { .. } => {
                // domain 声明是游戏后端配置,C 代码生成无需输出
            }
            Stmt::ModDecl { .. } | Stmt::UseDecl { .. } => {
                // 模块/导入声明是元数据,C 代码生成无需输出
            }
        }

        Ok(lines)
    }

    fn generate_match_arm_condition(&mut self, pattern: &Pattern, scrutinee: &str) -> Result<String, String> {
        match pattern {
            Pattern::Wildcard => Ok("1".to_string()),
            Pattern::Literal(expr) => {
                let expr_str = self.generate_expr(expr)?;
                Ok(format!("({} == {})", scrutinee, expr_str))
            }
            Pattern::Bind(name) => {
                self.var_map.insert(name.clone(), scrutinee.to_string());
                Ok("1".to_string())
            }
            Pattern::EnumVariant { type_name, variant } => {
                Ok(format!("({}.discriminant == {}_v_{})", scrutinee, type_name, variant))
            }
            Pattern::EnumVariantWithPayload { type_name, variant, bindings } => {
                let disc_check = format!("({}.discriminant == {}_v_{})", scrutinee, type_name, variant);
                let checks = vec![disc_check];
                for (i, binding) in bindings.iter().enumerate() {
                    if binding == "_" {
                        continue;
                    }
                    let accessor = if self.enum_payload_count(type_name, variant) <= 1 {
                        format!("{}.data.{}.field0", scrutinee, variant)
                    } else {
                        format!("{}.data.{}.field{}", scrutinee, variant, i)
                    };
                    self.var_map.insert(binding.clone(), accessor);
                }
                Ok(checks.join(" && "))
            }
        }
    }

    fn enum_payload_count(&self, type_name: &str, variant: &str) -> usize {
        self.enum_map.get(type_name)
            .and_then(|variants| {
                variants.iter()
                    .find(|(name, _, _)| name == variant)
                    .map(|(_, _, payloads)| payloads.len())
            })
            .unwrap_or(0)
    }

    fn generate_block_lines(&mut self, block: &Block) -> Result<Vec<String>, String> {
        let mut lines = Vec::new();
        for stmt in &block.stmts {
            let stmt_lines = self.generate_stmt(stmt)?;
            lines.extend(stmt_lines);
        }
        Ok(lines)
    }

    fn generate_block_with_assign(&mut self, block: &Block, _target: &str) -> Result<String, String> {
        let mut code = String::new();
        for stmt in &block.stmts {
            let stmt_lines = self.generate_stmt(stmt)?;
            for line in stmt_lines {
                code.push_str(&line);
                code.push('\n');
            }
        }
        Ok(code)
    }

    fn generate_program(&mut self, program: &Program) -> Result<String, String> {
        let Program::Block(stmts) = program;

        // Pre-scan: collect all function return types for accurate type inference
        for stmt in stmts {
            if let Stmt::FnDecl { name, return_type, .. } = stmt {
                let ret_type = if let Some(rt) = return_type {
                    Self::map_type(rt).unwrap_or_else(|_| Self::default_type())
                } else {
                    "void".to_string()
                };
                self.fn_return_types.insert(name.clone(), ret_type);
            }
        }

        let mut top_level_exprs = Vec::new();
        let mut main_body_lines = Vec::new();
        let mut has_toplevel_code = false;

        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr) => {
                    top_level_exprs.push(expr.clone());
                    has_toplevel_code = true;
                }
                Stmt::FnDecl { .. } | Stmt::ExternDecl { .. } | Stmt::ExportDecl { .. }
                | Stmt::StructDecl { .. } | Stmt::EnumDecl { .. } | Stmt::FlowDecl { .. }
                | Stmt::DomainDecl { .. } | Stmt::ModDecl { .. } | Stmt::UseDecl { .. } => {
                    self.generate_stmt(stmt)?;
                }
                Stmt::LetDecl { .. } | Stmt::Assign { .. } | Stmt::If { .. }
                | Stmt::While { .. } | Stmt::For { .. } | Stmt::ForIterable { .. } | Stmt::Loop(_)
                | Stmt::Break | Stmt::Continue | Stmt::Return(_) | Stmt::Match { .. } => {
                    has_toplevel_code = true;
                    let saved_indent = self.indent;
                    self.indent = 1;
                    let lines = self.generate_stmt(stmt)?;
                    self.indent = saved_indent;
                    main_body_lines.extend(lines);
                }
            }
        }

        if has_toplevel_code && !self.has_main {
            let saved_indent = self.indent;
            self.indent = 1;

            let mut main_body = main_body_lines;
            for expr in &top_level_exprs {
                let expr_str = self.generate_expr(expr)?;
                main_body.push(format!("{}printf(\"%lld\\n\", (long long)({}));", self.indent_str(), expr_str));
            }
            main_body.push(format!("{}return 0;", self.indent_str()));

            self.indent = saved_indent;

            let mut main_fn = "int main(void) {\n".to_string();
            for line in &main_body {
                main_fn.push_str(line);
                main_fn.push('\n');
            }
            main_fn.push_str("}\n");
            self.functions.push(main_fn);
            self.has_main = true;
        }

        let mut output = String::new();

        if self.debug_info {
            output.push_str("#line 1 \"<link>\"\n");
        }

        output.push_str("#include <stdint.h>\n");
        output.push_str("#include <stdbool.h>\n");
        output.push_str("#include <stdio.h>\n");
        output.push_str("#include <stdlib.h>\n");
        output.push_str("#include <string.h>\n");
        output.push_str("#include <stdbool.h>\n");
        output.push_str("#ifdef _WIN32\n");
        output.push_str("#include <windows.h>\n");
        output.push_str("#else\n");
        output.push_str("#include <unistd.h>\n");
        output.push_str("#define Sleep(x) usleep((x) * 1000)\n");
        output.push_str("#define DWORD unsigned int\n");
        output.push_str("#endif\n");
        output.push_str("\n");

        output.push_str("typedef struct {\n");
        output.push_str("    int64_t count;\n");
        output.push_str("    int64_t items[256];\n");
        output.push_str("} LinkList;\n\n");

        for def in &self.struct_defs {
            output.push_str(def);
            output.push('\n');
        }
        for def in &self.enum_defs {
            output.push_str(def);
            output.push('\n');
        }

        for g in &self.globals {
            output.push_str(g);
            output.push('\n');
        }
        output.push_str("\n");

        for f in &self.functions {
            output.push_str(f);
            output.push('\n');
        }

        Ok(output)
    }
}

impl CodeGenerator for CBackend {
    fn generate(&mut self, program: &Program) -> Result<String, String> {
        self.generate_program(program)
    }
}

// ===== Python 后端 =====

pub struct PythonBackend {
    indent: usize,
    functions: Vec<String>,
    classes: Vec<String>,
    var_map: HashMap<String, String>,
    struct_map: HashMap<String, Vec<(String, String)>>,
    enum_map: HashMap<String, Vec<(String, usize)>>,
    tmp_counter: usize,
    has_main: bool,
}

impl PythonBackend {
    pub fn new() -> Self {
        Self {
            indent: 0,
            functions: Vec::new(),
            classes: Vec::new(),
            var_map: HashMap::new(),
            struct_map: HashMap::new(),
            enum_map: HashMap::new(),
            tmp_counter: 0,
            has_main: false,
        }
    }

    /// 生成管道运算符代码: `a | b(c)` => `b(a, c)`
    /// 语法上右结合，语义上左结合:
    /// `a | b | c` 解析为 `a | (b | c)`, 语义为 `c(b(a))`
    fn generate_pipe(&mut self, left: &Expr, right: &Expr) -> Result<String, String> {
        let left_str = self.generate_expr(left)?;
        self.apply_pipe(left_str, right)
    }

    /// 将 left_str 作为输入，应用管道右侧 right
    /// `left_str | right` 语义:
    /// - `left_str | f(...)` => `f(left_str, ...)`
    /// - `left_str | f` => `f(left_str)`
    /// - `left_str | (mid | tail)` => `tail(mid(left_str))`
    fn apply_pipe(&mut self, left_str: String, right: &Expr) -> Result<String, String> {
        match right {
            Expr::Call { callee, args } => {
                let mut all_args = vec![left_str];
                for arg in args {
                    all_args.push(self.generate_expr(arg)?);
                }
                Ok(format!("{}({})", callee, all_args.join(", ")))
            }
            Expr::Ident(name) => {
                Ok(format!("{}({})", name, left_str))
            }
            // 嵌套管道: left_str | (mid | tail) => tail(mid(left_str))
            Expr::Binary { op: BinOp::Pipe, left: mid, right: tail } => {
                let mid_result = self.apply_pipe(left_str, mid)?;
                self.apply_pipe(mid_result, tail)
            }
            _ => Err("pipe right-hand side must be a call or function name".to_string()),
        }
    }

    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }

    fn fresh_tmp(&mut self) -> String {
        self.tmp_counter += 1;
        format!("_tmp{}", self.tmp_counter)
    }

    fn py_type(type_ann: &TypeAnnotation) -> String {
        match type_ann {
            TypeAnnotation::I8 | TypeAnnotation::I16 | TypeAnnotation::I32 | TypeAnnotation::I64
            | TypeAnnotation::U8 | TypeAnnotation::U16 | TypeAnnotation::U32 | TypeAnnotation::U64
            | TypeAnnotation::USize => "int".to_string(),
            TypeAnnotation::F32 | TypeAnnotation::F64 => "float".to_string(),
            TypeAnnotation::Bool => "bool".to_string(),
            TypeAnnotation::Str | TypeAnnotation::Void => "str".to_string(),
            TypeAnnotation::Unit => "None".to_string(),
            TypeAnnotation::Named(n) => n.clone(),
            TypeAnnotation::Ptr(_) => "Any".to_string(),
            TypeAnnotation::Stream(_) => "list".to_string(),
            TypeAnnotation::Ref(inner, _) => Self::py_type(inner),
            TypeAnnotation::Generic(base, _) => Self::py_type(base),
            TypeAnnotation::Array(elem, _) => format!("list[{}]", Self::py_type(elem)),
            TypeAnnotation::Tuple(elems) => {
                if elems.is_empty() {
                    "tuple".to_string()
                } else {
                    let parts: Vec<String> = elems.iter()
                        .map(|e| Self::py_type(e))
                        .collect();
                    format!("tuple[{}]", parts.join(", "))
                }
            }
        }
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Int(n) => Ok(n.to_string()),
            Expr::Float(f) => {
                let s = format!("{}", f);
                if s.contains('.') || s.contains('e') || s.contains('E') {
                    Ok(s)
                } else {
                    Ok(format!("{}.0", s))
                }
            }
            Expr::Str(s) => {
                let escaped = s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
                Ok(format!("\"{}\"", escaped))
            }
            Expr::Bool(b) => Ok(if *b { "True" } else { "False" }.to_string()),
            Expr::None => Ok("None".to_string()),
            Expr::Ident(name) => {
                if let Some(py_name) = self.var_map.get(name) {
                    Ok(py_name.clone())
                } else {
                    Ok(name.clone())
                }
            }
            Expr::Binary { op, left, right } => {
                // 管道运算符 `a | b(c)` 等价于 `b(a, c)`（右结合）
                if let BinOp::Pipe = op {
                    return self.generate_pipe(left, right);
                }
                let left_str = self.generate_expr(left)?;
                let right_str = self.generate_expr(right)?;
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Mod => "%",
                    BinOp::Eq => "==",
                    BinOp::Neq => "!=",
                    BinOp::Lt => "<",
                    BinOp::Gt => ">",
                    BinOp::LtEq => "<=",
                    BinOp::GtEq => ">=",
                    BinOp::And => "and",
                    BinOp::Or => "or",
                    BinOp::Pipe => unreachable!(),
                };
                Ok(format!("({} {} {})", left_str, op_str, right_str))
            }
            Expr::Unary { op, operand } => {
                let operand_str = self.generate_expr(operand)?;
                match op {
                    UnaryOp::Neg => Ok(format!("(-{})", operand_str)),
                    UnaryOp::Not => Ok(format!("(not {})", operand_str)),
                }
            }
            Expr::Call { callee, args } => {
                match callee.as_str() {
                    "print" => {
                        let mut arg_strs = Vec::new();
                        for arg in args {
                            arg_strs.push(self.generate_expr(arg)?);
                        }
                        Ok(format!("print({})", arg_strs.join(", ")))
                    }
                    "println" => {
                        if args.is_empty() {
                            Ok("print()".to_string())
                        } else if args.len() >= 2 {
                            if let Expr::Str(fmt) = &args[0] {
                                if fmt.contains("{}") {
                                    let mut parts = Vec::new();
                                    let mut arg_idx = 1;
                                    for ch in fmt.chars() {
                                        if ch == '{' && arg_idx < args.len() {
                                            // peek next char
                                            parts.push("{}".to_string());
                                            arg_idx += 1;
                                        } else if ch == '}' && arg_idx > 1 {
                                            // already consumed
                                        } else {
                                            // push as string literal char
                                            if let Some(last) = parts.last_mut() {
                                                if last.starts_with("'") || last.starts_with("\"") {
                                                    last.pop();
                                                    last.push(ch);
                                                    last.push('"');
                                                } else {
                                                    parts.push(format!("'{}'", ch));
                                                }
                                            } else {
                                                parts.push(format!("'{}'", ch));
                                            }
                                        }
                                    }
                                    // Simplified: use f-string
                                    let mut fstr = fmt.clone();
                                    let mut arg_exprs = Vec::new();
                                    let mut arg_i = 1;
                                    while fstr.contains("{}") && arg_i < args.len() {
                                        let expr_str = self.generate_expr(&args[arg_i])?;
                                        fstr = fstr.replacen("{}", "{}", 1);
                                        arg_exprs.push(expr_str);
                                        arg_i += 1;
                                    }
                                    let escaped_fmt = fstr.replace('\\', "\\\\").replace('"', "\\\"");
                                    if arg_exprs.is_empty() {
                                        Ok(format!("print(\"{}\")", escaped_fmt))
                                    } else {
                                        Ok(format!("print(\"{}\".format({}))", escaped_fmt, arg_exprs.join(", ")))
                                    }
                                } else {
                                    let mut arg_strs = Vec::new();
                                    for arg in args {
                                        arg_strs.push(self.generate_expr(arg)?);
                                    }
                                    Ok(format!("print({})", arg_strs.join(", ")))
                                }
                            } else {
                                let mut arg_strs = Vec::new();
                                for arg in args {
                                    arg_strs.push(self.generate_expr(arg)?);
                                }
                                Ok(format!("print({})", arg_strs.join(", ")))
                            }
                        } else {
                            let arg_str = self.generate_expr(&args[0])?;
                            Ok(format!("print({})", arg_str))
                        }
                    }
                    "len" => {
                        let arg_str = self.generate_expr(&args[0])?;
                        Ok(format!("len({})", arg_str))
                    }
                    "sleep" => {
                        let arg_str = self.generate_expr(&args[0])?;
                        Ok(format!("time.sleep({} / 1000.0)", arg_str))
                    }
                    "abs" => {
                        let arg_str = self.generate_expr(&args[0])?;
                        Ok(format!("abs({})", arg_str))
                    }
                    "min" => {
                        let mut arg_strs = Vec::new();
                        for arg in args { arg_strs.push(self.generate_expr(arg)?); }
                        Ok(format!("min({})", arg_strs.join(", ")))
                    }
                    "max" => {
                        let mut arg_strs = Vec::new();
                        for arg in args { arg_strs.push(self.generate_expr(arg)?); }
                        Ok(format!("max({})", arg_strs.join(", ")))
                    }
                    "sqrt" => {
                        let arg_str = self.generate_expr(&args[0])?;
                        Ok(format!("math.sqrt({})", arg_str))
                    }
                    "str_upper" => {
                        let arg_str = self.generate_expr(&args[0])?;
                        Ok(format!("{}.upper()", arg_str))
                    }
                    "str_lower" => {
                        let arg_str = self.generate_expr(&args[0])?;
                        Ok(format!("{}.lower()", arg_str))
                    }
                    _ => {
                        let mut arg_strs = Vec::new();
                        for arg in args {
                            arg_strs.push(self.generate_expr(arg)?);
                        }
                        Ok(format!("{}({})", callee, arg_strs.join(", ")))
                    }
                }
            }
            Expr::IfExpr { condition, then_value, else_value } => {
                let cond_str = self.generate_expr(condition)?;
                let then_str = self.generate_expr(then_value)?;
                let else_str = self.generate_expr(else_value)?;
                Ok(format!("({} if {} else {})", then_str, cond_str, else_str))
            }
            Expr::List(items) => {
                let mut item_strs = Vec::new();
                for item in items {
                    item_strs.push(self.generate_expr(item)?);
                }
                Ok(format!("[{}]", item_strs.join(", ")))
            }
            Expr::Tuple(elems) => {
                let mut elem_strs = Vec::new();
                for e in elems {
                    elem_strs.push(self.generate_expr(e)?);
                }
                if elem_strs.len() == 1 {
                    Ok(format!("({},)", elem_strs[0]))
                } else {
                    Ok(format!("({})", elem_strs.join(", ")))
                }
            }
            Expr::Index { target, index } => {
                let target_str = self.generate_expr(target)?;
                let index_str = self.generate_expr(index)?;
                Ok(format!("{}[{}]", target_str, index_str))
            }
            Expr::BlockExpr(block) => {
                // Python 不支持块表达式，用立即调用 lambda 模拟
                let _tmp = self.fresh_tmp();
                let saved_indent = self.indent;
                self.indent = 1;
                let mut body_lines = Vec::new();
                for stmt in &block.stmts {
                    let lines = self.generate_stmt(stmt)?;
                    body_lines.extend(lines);
                }
                self.indent = saved_indent;
                // 简化：返回最后一条表达式的值
                if let Some(Stmt::Expr(last_expr)) = block.stmts.last() {
                    let last_str = self.generate_expr(last_expr)?;
                    body_lines.pop();
                    body_lines.push(format!("    return {}", last_str));
                } else {
                    body_lines.push("    return None".to_string());
                }
                Ok(format!("(lambda: (\n{}\n))()", body_lines.join("\n")))
            }
            Expr::StructInit { name, fields } => {
                let mut field_strs = Vec::new();
                for (fname, fval) in fields {
                    let val_str = self.generate_expr(fval)?;
                    field_strs.push(format!("{}={}", fname, val_str));
                }
                Ok(format!("{}({})", name, field_strs.join(", ")))
            }
            Expr::FieldAccess { target, field } => {
                let target_str = self.generate_expr(target)?;
                Ok(format!("{}.{}", target_str, field))
            }
            Expr::Path { base, segment } => {
                // 枚举无参变体: Type::Variant -> Type.Variant
                Ok(format!("{}.{}", base, segment))
            }
            Expr::PathCall { base, segment, args } => {
                // 枚举带参变体: Type::Variant(args) -> Type.Variant(args)
                let mut arg_strs = Vec::new();
                for arg in args {
                    arg_strs.push(self.generate_expr(arg)?);
                }
                Ok(format!("{}.{}({})", base, segment, arg_strs.join(", ")))
            }
            Expr::MatchExpr { scrutinee, arms } => {
                // 生成嵌套的 if-elif-else 表达式
                let scrut_str = self.generate_expr(scrutinee)?;
                let tmp = self.fresh_tmp();
                self.var_map.insert(tmp.clone(), tmp.clone());
                let mut result = format!("(lambda {tmp}: ");
                let mut first = true;
                for arm in arms {
                    let body_str = self.generate_block_return(&arm.body)?;
                    match &arm.pattern {
                        Pattern::Wildcard => {
                            result.push_str(&body_str);
                        }
                        Pattern::Literal(expr) => {
                            let lit_str = self.generate_expr(expr)?;
                            if first {
                                result.push_str(&format!("{} if {tmp} == {} else ", body_str, lit_str));
                            } else {
                                result.push_str(&format!("{} if {tmp} == {} else ", body_str, lit_str));
                            }
                        }
                        Pattern::Bind(name) => {
                            self.var_map.insert(name.clone(), tmp.clone());
                            result.push_str(&body_str);
                        }
                        Pattern::EnumVariant { type_name, variant } => {
                            let var_name = format!("{}.{}", type_name, variant);
                            if first {
                                result.push_str(&format!("{} if {tmp} == {} else ", body_str, var_name));
                            } else {
                                result.push_str(&format!("{} if {tmp} == {} else ", body_str, var_name));
                            }
                        }
                        Pattern::EnumVariantWithPayload { variant, .. } => {
                            result.push_str(&format!("{} if isinstance({tmp}, tuple) and {tmp}[0] == '{}' else ", body_str, variant));
                        }
                    }
                    first = false;
                }
                if first {
                    result.push_str("None");
                }
                result.push_str(&format!(")({})", scrut_str));
                Ok(result)
            }
            Expr::Await(inner) => {
                let inner_str = self.generate_expr(inner)?;
                Ok(format!("await {}", inner_str))
            }
            Expr::Try(inner) => {
                let inner_str = self.generate_expr(inner)?;
                Ok(format!("__link_try({})", inner_str))
            }
            Expr::Lambda { params, body, .. } => {
                let param_str: Vec<String> = params.iter().map(|(n, _)| n.clone()).collect();
                // 简单情况: 函数体只有一条 return 语句 => 使用 Python lambda
                if body.stmts.len() == 1 {
                    if let Stmt::Return(Some(ret_expr)) = &body.stmts[0] {
                        let ret_str = self.generate_expr(ret_expr)?;
                        return Ok(format!("lambda {}: {}", param_str.join(", "), ret_str));
                    }
                }
                // 复杂情况: 函数体有多条语句 => 生成嵌套 def（需作为前置语句）
                // 由于 generate_expr 只能返回表达式字符串，这里使用立即调用方式
                let lambda_name = format!("__lambda_{}", self.tmp_counter);
                self.tmp_counter += 1;
                // 保存当前缩进，生成 def 函数体
                let saved_indent = self.indent;
                self.indent = 1;
                let body_lines = self.generate_block_lines(body)?;
                self.indent = saved_indent;
                let mut body_code = String::new();
                for line in &body_lines {
                    body_code.push_str("    ");
                    body_code.push_str(line);
                    body_code.push('\n');
                }
                if body_lines.is_empty() {
                    body_code.push_str("    pass\n");
                }
                // 使用立即调用表达式: (lambda_name = None; def lambda_name(...): ...; lambda_name) 不能在表达式内
                // 折中方案: 用 exec 模拟，或者提示用户简化 lambda
                Err(format!(
                    "complex lambda body not supported in Python backend (use single return statement):\ndef {}({}):\n{}",
                    lambda_name, param_str.join(", "), body_code
                ))
            }
            Expr::Ref(inner, _) => self.generate_expr(inner),
            Expr::Deref(inner) => self.generate_expr(inner),
            Expr::AsCast(expr, _) => self.generate_expr(expr),
        }
    }

    fn generate_block_return(&mut self, block: &Block) -> Result<String, String> {
        // 生成块的最后一条表达式作为返回值
        if let Some(Stmt::Expr(last_expr)) = block.stmts.last() {
            self.generate_expr(last_expr)
        } else if let Some(Stmt::Return(Some(expr))) = block.stmts.last() {
            self.generate_expr(expr)
        } else {
            Ok("None".to_string())
        }
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<Vec<String>, String> {
        let mut lines = Vec::new();

        match stmt {
            Stmt::LetDecl { name, value, .. } => {
                let py_name = name.clone();
                self.var_map.insert(name.clone(), py_name.clone());
                if let Some(val) = value {
                    let val_str = self.generate_expr(val)?;
                    lines.push(format!("{}{} = {}", self.indent_str(), py_name, val_str));
                } else {
                    lines.push(format!("{}{} = None", self.indent_str(), py_name));
                }
            }
            Stmt::Assign { target, value } => {
                let target_str = self.generate_expr(target)?;
                let val_str = self.generate_expr(value)?;
                lines.push(format!("{}{} = {}", self.indent_str(), target_str, val_str));
            }
            Stmt::Expr(expr) => {
                let expr_str = self.generate_expr(expr)?;
                lines.push(format!("{}{}", self.indent_str(), expr_str));
            }
            Stmt::Return(Some(expr)) => {
                let expr_str = self.generate_expr(expr)?;
                lines.push(format!("{}return {}", self.indent_str(), expr_str));
            }
            Stmt::Return(None) => {
                lines.push(format!("{}return None", self.indent_str()));
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let cond_str = self.generate_expr(condition)?;
                lines.push(format!("{}if {}:", self.indent_str(), cond_str));
                self.indent += 1;
                if then_branch.stmts.is_empty() {
                    lines.push(format!("{}pass", self.indent_str()));
                } else {
                    let then_lines = self.generate_block_lines(then_branch)?;
                    lines.extend(then_lines);
                }
                self.indent -= 1;

                if let Some(else_block) = else_branch {
                    lines.push(format!("{}else:", self.indent_str()));
                    self.indent += 1;
                    if else_block.stmts.is_empty() {
                        lines.push(format!("{}pass", self.indent_str()));
                    } else {
                        let else_lines = self.generate_block_lines(else_block)?;
                        lines.extend(else_lines);
                    }
                    self.indent -= 1;
                }
            }
            Stmt::While { condition, body } => {
                let cond_str = self.generate_expr(condition)?;
                lines.push(format!("{}while {}:", self.indent_str(), cond_str));
                self.indent += 1;
                if body.stmts.is_empty() {
                    lines.push(format!("{}pass", self.indent_str()));
                } else {
                    let body_lines = self.generate_block_lines(body)?;
                    lines.extend(body_lines);
                }
                self.indent -= 1;
            }
            Stmt::For { var_name, start, end, body } => {
                let start_str = self.generate_expr(start)?;
                let end_str = self.generate_expr(end)?;
                let py_name = var_name.clone();
                self.var_map.insert(var_name.clone(), py_name.clone());
                lines.push(format!("{}for {} in range({}, {}):", self.indent_str(), py_name, start_str, end_str));
                self.indent += 1;
                if body.stmts.is_empty() {
                    lines.push(format!("{}pass", self.indent_str()));
                } else {
                    let body_lines = self.generate_block_lines(body)?;
                    lines.extend(body_lines);
                }
                self.indent -= 1;
            }
            Stmt::ForIterable { var_name, iterable, body } => {
                let iter_str = self.generate_expr(iterable)?;
                let py_name = var_name.clone();
                self.var_map.insert(var_name.clone(), py_name.clone());
                lines.push(format!("{}for {} in {}:", self.indent_str(), py_name, iter_str));
                self.indent += 1;
                if body.stmts.is_empty() {
                    lines.push(format!("{}pass", self.indent_str()));
                } else {
                    let body_lines = self.generate_block_lines(body)?;
                    lines.extend(body_lines);
                }
                self.indent -= 1;
            }
            Stmt::Loop(body) => {
                lines.push(format!("{}while True:", self.indent_str()));
                self.indent += 1;
                let body_lines = self.generate_block_lines(body)?;
                lines.extend(body_lines);
                self.indent -= 1;
            }
            Stmt::Break => {
                lines.push(format!("{}break", self.indent_str()));
            }
            Stmt::Continue => {
                lines.push(format!("{}continue", self.indent_str()));
            }
            Stmt::FnDecl { name, params, return_type: _, body, is_async } => {
                if name == "main" {
                    self.has_main = true;
                }
                let mut param_strs = Vec::new();
                let mut fn_var_map = HashMap::new();
                for (pname, _) in params {
                    param_strs.push(pname.clone());
                    fn_var_map.insert(pname.clone(), pname.clone());
                }
                let saved_var_map = std::mem::replace(&mut self.var_map, fn_var_map);
                let saved_indent = self.indent;
                self.indent = 1;

                let body_lines = self.generate_block_lines(body)?;

                self.var_map = saved_var_map;
                self.indent = saved_indent;

                let prefix = if *is_async { "async " } else { "" };
                let mut fn_code = format!("{}def {}({}):\n", prefix, name, param_strs.join(", "));
                if body_lines.is_empty() {
                    fn_code.push_str("    pass\n");
                } else {
                    for line in &body_lines {
                        fn_code.push_str(line);
                        fn_code.push('\n');
                    }
                }
                self.functions.push(fn_code);
            }
            Stmt::StructDecl { name, fields } => {
                let mut field_names = Vec::new();
                for field in fields {
                    field_names.push((field.name.clone(), Self::py_type(&field.type_ann)));
                }
                self.struct_map.insert(name.clone(), field_names.clone());

                let mut class_code = format!("class {}:\n", name);
                class_code.push_str("    def __init__(self");
                for (fname, _) in &field_names {
                    class_code.push_str(&format!(", {}", fname));
                }
                class_code.push_str("):\n");
                if field_names.is_empty() {
                    class_code.push_str("        pass\n");
                } else {
                    for (fname, _) in &field_names {
                        class_code.push_str(&format!("        self.{} = {}\n", fname, fname));
                    }
                }
                class_code.push_str(&format!("    def __repr__(self):\n"));
                if field_names.is_empty() {
                    class_code.push_str(&format!("        return \"{}()\"\n", name));
                } else {
                    let field_repr: Vec<String> = field_names.iter()
                        .map(|(f, _)| format!("\"{}=\" + str(self.{})", f, f))
                        .collect();
                    class_code.push_str(&format!("        return \"{}(\" + {} + \")\"\n", name, field_repr.join(" + \", \" + ")));
                }
                self.classes.push(class_code);
            }
            Stmt::EnumDecl { name, variants } => {
                let mut variant_info = Vec::new();
                for (i, variant) in variants.iter().enumerate() {
                    variant_info.push((variant.name.clone(), i));
                }
                self.enum_map.insert(name.clone(), variant_info.clone());

                let mut class_code = format!("class {}:\n", name);
                for (vname, _) in &variant_info {
                    if variants.iter().find(|v| &v.name == vname).map_or(false, |v| !v.payload.is_empty()) {
                        // 带参数的变体: 返回元组
                        class_code.push_str(&format!("    @staticmethod\n"));
                        class_code.push_str(&format!("    def {}(*args):\n", vname));
                        class_code.push_str(&format!("        return (\"{}\", args)\n", vname));
                    } else {
                        class_code.push_str(&format!("    {} = \"{}\"\n", vname, vname));
                    }
                }
                self.classes.push(class_code);
            }
            Stmt::Match { scrutinee, arms } => {
                let tmp = self.fresh_tmp();
                let scrut_str = self.generate_expr(scrutinee)?;
                self.var_map.insert(tmp.clone(), tmp.clone());
                lines.push(format!("{}{} = {}", self.indent_str(), tmp, scrut_str));

                let mut first = true;
                let mut emitted_else = false;
                for arm in arms {
                    if emitted_else {
                        break;
                    }
                    let cond_str = match &arm.pattern {
                        Pattern::Wildcard => None,
                        Pattern::Literal(expr) => {
                            let lit_str = self.generate_expr(expr)?;
                            Some(format!("{} == {}", tmp, lit_str))
                        }
                        Pattern::Bind(name) => {
                            self.var_map.insert(name.clone(), tmp.clone());
                            None
                        }
                        Pattern::EnumVariant { type_name, variant } => {
                            Some(format!("{} == {}.{}", tmp, type_name, variant))
                        }
                        Pattern::EnumVariantWithPayload { variant, bindings, .. } => {
                            for (i, binding) in bindings.iter().enumerate() {
                                if binding != "_" {
                                    self.var_map.insert(binding.clone(), format!("{}[1][{}]", tmp, i));
                                }
                            }
                            let cond = format!("isinstance({}, tuple) and {}[0] == \"{}\"", tmp, tmp, variant);
                            Some(cond)
                        }
                    };

                    if first {
                        match cond_str {
                            Some(c) => lines.push(format!("{}if {}:", self.indent_str(), c)),
                            None => {
                                lines.push(format!("{}if True:", self.indent_str()));
                                self.indent += 1;
                                let arm_lines = self.generate_block_lines(&arm.body)?;
                                if arm_lines.is_empty() {
                                    lines.push(format!("{}pass", self.indent_str()));
                                } else {
                                    lines.extend(arm_lines);
                                }
                                self.indent -= 1;
                                first = false;
                                emitted_else = true;
                                continue;
                            }
                        }
                        first = false;
                    } else {
                        match cond_str {
                            Some(c) => lines.push(format!("{}elif {}:", self.indent_str(), c)),
                            None => {
                                lines.push(format!("{}else:", self.indent_str()));
                                self.indent += 1;
                                let arm_lines = self.generate_block_lines(&arm.body)?;
                                if arm_lines.is_empty() {
                                    lines.push(format!("{}pass", self.indent_str()));
                                } else {
                                    lines.extend(arm_lines);
                                }
                                self.indent -= 1;
                                emitted_else = true;
                                continue;
                            }
                        }
                    }
                    self.indent += 1;
                    let arm_lines = self.generate_block_lines(&arm.body)?;
                    if arm_lines.is_empty() {
                        lines.push(format!("{}pass", self.indent_str()));
                    } else {
                        lines.extend(arm_lines);
                    }
                    self.indent -= 1;
                }
                if first {
                    lines.push(format!("{}pass", self.indent_str()));
                }
            }
            Stmt::ExternDecl { language: _, module: _, decls } => {
                for sig in decls {
                    let mut param_strs = Vec::new();
                    for (pname, _) in &sig.params {
                        param_strs.push(pname.clone());
                    }
                    let prefix = if sig.is_async { "async " } else { "" };
                    let mut fn_code = format!("{}def {}({}):\n", prefix, sig.name, param_strs.join(", "));
                    let default = match &sig.return_type {
                        Some(rt) => match rt {
                            TypeAnnotation::I8 | TypeAnnotation::I16 | TypeAnnotation::I32 | TypeAnnotation::I64
                            | TypeAnnotation::U8 | TypeAnnotation::U16 | TypeAnnotation::U32 | TypeAnnotation::U64
                            | TypeAnnotation::USize => "    return 0\n".to_string(),
                            TypeAnnotation::F32 | TypeAnnotation::F64 => "    return 0.0\n".to_string(),
                            TypeAnnotation::Bool => "    return False\n".to_string(),
                            TypeAnnotation::Str | TypeAnnotation::Void => "    return \"\"\n".to_string(),
                            TypeAnnotation::Unit => "    return None\n".to_string(),
                            _ => "    return None\n".to_string(),
                        },
                        None => "    return None\n".to_string(),
                    };
                    fn_code.push_str(&default);
                    self.functions.push(fn_code);
                }
            }
            Stmt::ExportDecl { decls, .. } => {
                for sig in decls {
                    let mut param_strs = Vec::new();
                    for (pname, _) in &sig.params {
                        param_strs.push(pname.clone());
                    }
                    let prefix = if sig.is_async { "async " } else { "" };
                    let mut fn_code = format!("{}def {}({}):\n", prefix, sig.name, param_strs.join(", "));
                    fn_code.push_str("    pass\n");
                    self.functions.push(fn_code);
                }
            }
            Stmt::FlowDecl { .. } => {
                return Err("flow not supported in Python backend".to_string());
            }
            Stmt::DomainDecl { .. } => {
                // domain 声明在 Python 中生成字典
            }
            Stmt::ModDecl { .. } | Stmt::UseDecl { .. } => {
                // 模块/导入声明忽略
            }
        }

        Ok(lines)
    }

    fn generate_block_lines(&mut self, block: &Block) -> Result<Vec<String>, String> {
        let mut lines = Vec::new();
        for stmt in &block.stmts {
            let stmt_lines = self.generate_stmt(stmt)?;
            lines.extend(stmt_lines);
        }
        Ok(lines)
    }

    fn generate_program(&mut self, program: &Program) -> Result<String, String> {
        let Program::Block(stmts) = program;

        let mut main_body_lines = Vec::new();
        let mut has_toplevel_code = false;

        for stmt in stmts {
            match stmt {
                Stmt::FnDecl { .. } | Stmt::StructDecl { .. } | Stmt::EnumDecl { .. }
                | Stmt::ExternDecl { .. } | Stmt::ExportDecl { .. }
                | Stmt::FlowDecl { .. } | Stmt::DomainDecl { .. }
                | Stmt::ModDecl { .. } | Stmt::UseDecl { .. } => {
                    self.generate_stmt(stmt)?;
                }
                _ => {
                    has_toplevel_code = true;
                    let saved_indent = self.indent;
                    self.indent = 1;
                    let lines = self.generate_stmt(stmt)?;
                    self.indent = saved_indent;
                    main_body_lines.extend(lines);
                }
            }
        }

        let mut output = String::new();
        output.push_str("#!/usr/bin/env python3\n");
        output.push_str("# Generated by Link compiler - Python backend\n");
        output.push_str("import math\n");
        output.push_str("import time\n");
        output.push_str("\n");

        // 预置 stream 相关函数（Link 语义: map(stream, fn) 参数顺序与 Python 内置相反）
        output.push_str("def stream(iterable):\n");
        output.push_str("    return list(iterable)\n\n");
        output.push_str("def map(source, fn):\n");
        output.push_str("    return [fn(x) for x in source]\n\n");
        output.push_str("def filter(source, fn):\n");
        output.push_str("    return [x for x in source if fn(x)]\n\n");
        output.push_str("def for_each(source, fn):\n");
        output.push_str("    for x in source:\n");
        output.push_str("        fn(x)\n\n");
        output.push_str("def collect(source):\n");
        output.push_str("    return list(source)\n\n");

        for class in &self.classes {
            output.push_str(class);
            output.push('\n');
        }

        for func in &self.functions {
            output.push_str(func);
            output.push('\n');
        }

        if has_toplevel_code && !self.has_main {
            output.push_str("\nif __name__ == \"__main__\":\n");
            for line in &main_body_lines {
                output.push_str(line);
                output.push('\n');
            }
        } else if self.has_main {
            output.push_str("\nif __name__ == \"__main__\":\n");
            output.push_str("    main()\n");
        }

        Ok(output)
    }
}

impl CodeGenerator for PythonBackend {
    fn generate(&mut self, program: &Program) -> Result<String, String> {
        self.generate_program(program)
    }
}

pub fn compile_to_python(program: &Program) -> Result<String, String> {
    let mut backend = PythonBackend::new();
    backend.generate(program)
}

// ============================================================================
// WASM Backend — WebAssembly Text Format (WAT)
// ============================================================================

/// WASM 后端：生成 WebAssembly 文本格式代码
/// 
/// 支持的特性：
/// - 函数导出（可被 JavaScript 调用）
/// - i32/i64/f32/f64 基本类型
/// - 控制流（if/else/while/for/loop/block）
/// - 算术和比较运算
/// - 内存操作（通过线性内存）
/// 
/// 限制：
/// - 不支持字符串（需要外部 JavaScript 辅助）
/// - 不支持结构体和枚举（需要手动内存管理）
/// - 不支持列表（需要外部内存分配器）
pub struct WasmBackend {
    functions: Vec<String>,
    exports: Vec<String>,
    memory_size: usize, // 线性内存大小（页数）
    var_map: HashMap<String, String>,
    var_type_map: HashMap<String, WasmType>,
    fn_params: HashMap<String, Vec<WasmType>>,
    fn_return: HashMap<String, Option<WasmType>>,
    tmp_counter: usize,
    label_counter: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum WasmType {
    I32,
    I64,
    F32,
    F64,
}

impl WasmType {
    fn from_annotation(ann: &TypeAnnotation) -> Self {
        match ann {
            TypeAnnotation::I8 | TypeAnnotation::I16 | TypeAnnotation::I32 => WasmType::I32,
            TypeAnnotation::U8 | TypeAnnotation::U16 | TypeAnnotation::U32 => WasmType::I32,
            TypeAnnotation::I64 | TypeAnnotation::U64 | TypeAnnotation::USize => WasmType::I64,
            TypeAnnotation::F32 => WasmType::F32,
            TypeAnnotation::F64 => WasmType::F64,
            TypeAnnotation::Bool => WasmType::I32,
            _ => WasmType::I32, // 默认
        }
    }

    fn to_wat(&self) -> &'static str {
        match self {
            WasmType::I32 => "i32",
            WasmType::I64 => "i64",
            WasmType::F32 => "f32",
            WasmType::F64 => "f64",
        }
    }
}

impl WasmBackend {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            exports: Vec::new(),
            memory_size: 1, // 默认 1 页（64KB）
            var_map: HashMap::new(),
            var_type_map: HashMap::new(),
            fn_params: HashMap::new(),
            fn_return: HashMap::new(),
            tmp_counter: 0,
            label_counter: 0,
        }
    }

    fn fresh_tmp(&mut self) -> String {
        self.tmp_counter += 1;
        format!("$tmp{}", self.tmp_counter)
    }

    fn fresh_label(&mut self) -> String {
        self.label_counter += 1;
        format!("$L{}", self.label_counter)
    }

    fn generate_program(&mut self, program: &Program) -> Result<String, String> {
        let Program::Block(stmts) = program;

        // 第一遍：收集函数签名
        for stmt in stmts {
            if let Stmt::FnDecl { name, params, return_type, .. } = stmt {
                let param_types: Vec<WasmType> = params.iter()
                    .map(|(_, t)| WasmType::from_annotation(t))
                    .collect();
                let ret_type = return_type.as_ref().map(|t| WasmType::from_annotation(t));
                self.fn_params.insert(name.clone(), param_types);
                self.fn_return.insert(name.clone(), ret_type);
            }
        }

        // 第二遍：生成函数代码
        for stmt in stmts {
            if let Stmt::FnDecl { name, params, body, .. } = stmt {
                let func_code = self.generate_function(name, params, body)?;
                self.functions.push(func_code);
                self.exports.push(name.clone());
            }
        }

        // 组装完整模块
        let mut wat = String::new();
        wat.push_str("(module\n");

        // 导入辅助函数（如打印）
        wat.push_str("  ;; Import console.log for println\n");
        wat.push_str("  (import \"console\" \"log\" (func $println (param i32)))\n");

        // 内存
        wat.push_str(&format!("  (memory (export \"memory\") {})\n", self.memory_size));

        // 函数
        for func in &self.functions {
            wat.push_str(func);
            wat.push('\n');
        }

        // 导出
        for export_name in &self.exports {
            wat.push_str(&format!("  (export \"{}\" (func ${}))\n", export_name, export_name));
        }

        wat.push_str(")\n");
        Ok(wat)
    }

    fn generate_function(&mut self, name: &str, params: &[(String, TypeAnnotation)], body: &Block) -> Result<String, String> {
        let mut code = format!("  (func ${}", name);

        // 参数
        for (pname, ptype) in params {
            let wasm_type = WasmType::from_annotation(ptype);
            code.push_str(&format!(" (param ${} {})", pname, wasm_type.to_wat()));
            self.var_type_map.insert(pname.clone(), wasm_type);
        }

        // 返回值
        if let Some(ret_type) = self.fn_return.get(name).and_then(|r| *r) {
            code.push_str(&format!(" (result {})", ret_type.to_wat()));
        }

        code.push_str("\n");

        // 局部变量（在函数体中遇到 let 声明时添加）
        let saved_var_map = self.var_map.clone();
        let saved_var_type_map = self.var_type_map.clone();

        // 生成函数体
        let body_code = self.generate_block(body)?;

        self.var_map = saved_var_map;
        self.var_type_map = saved_var_type_map;

        code.push_str(&body_code);
        code.push_str("  )\n");

        Ok(code)
    }

    fn generate_block(&mut self, block: &Block) -> Result<String, String> {
        let mut code = String::new();
        for stmt in &block.stmts {
            let stmt_code = self.generate_stmt(stmt)?;
            code.push_str(&stmt_code);
        }
        Ok(code)
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<String, String> {
        match stmt {
            Stmt::LetDecl { name, type_annotation, value } => {
                let ty = type_annotation.as_ref()
                    .map(|t| WasmType::from_annotation(t))
                    .unwrap_or(WasmType::I32);

                let mut code = format!("    (local ${} {})\n", name, ty.to_wat());
                self.var_type_map.insert(name.clone(), ty);

                if let Some(val) = value {
                    let val_code = self.generate_expr(val)?;
                    code.push_str(&val_code);
                    code.push_str(&format!("    (local.set ${})\n", name));
                }
                Ok(code)
            }
            Stmt::Assign { target, value } => {
                let val_code = self.generate_expr(value)?;
                let mut code = val_code;
                match target.as_ref() {
                    Expr::Ident(name) => {
                        code.push_str(&format!("    (local.set ${})\n", name));
                    }
                    _ => return Err(format!("WASM backend only supports simple variable assignment, got {:?}", target)),
                }
                Ok(code)
            }
            Stmt::Expr(expr) => {
                // 处理表达式语句（包括 println 函数调用）
                if let Expr::Call { callee, args } = expr {
                    if callee == "println" {
                        // println 在 WASM 中需要导入的外部函数
                        // 目前只支持打印整数
                        if !args.is_empty() {
                            let arg_code = self.generate_expr(&args[0])?;
                            let mut code = arg_code;
                            // 转换为 i32（如果需要）
                            code.push_str("    (i32.wrap_i64)\n"); // 假设是 i64，转换为 i32
                            code.push_str("    (call $println)\n");
                            return Ok(code);
                        } else {
                            return Ok(String::new());
                        }
                    }
                }
                let expr_code = self.generate_expr(expr)?;
                Ok(expr_code)
            }
            Stmt::Return(Some(expr)) => {
                let expr_code = self.generate_expr(expr)?;
                let mut code = expr_code;
                code.push_str("    (return)\n");
                Ok(code)
            }
            Stmt::Return(None) => {
                Ok("    (return)\n".to_string())
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let cond_code = self.generate_expr(condition)?;
                let then_code = self.generate_block(then_branch)?;
                let mut code = cond_code;
                code.push_str("    (if\n");
                code.push_str("      (then\n");
                code.push_str(&then_code);
                code.push_str("      )\n");
                if let Some(else_block) = else_branch {
                    let else_code = self.generate_block(else_block)?;
                    code.push_str("      (else\n");
                    code.push_str(&else_code);
                    code.push_str("      )\n");
                }
                code.push_str("    )\n");
                Ok(code)
            }
            Stmt::While { condition, body } => {
                let loop_label = self.fresh_label();
                let cond_code = self.generate_expr(condition)?;
                let body_code = self.generate_block(body)?;
                let mut code = format!("    (block ${}_outer\n", loop_label);
                code.push_str(&format!("      (loop ${}_inner\n", loop_label));
                code.push_str(&format!("        (br_if ${}_outer\n", loop_label));
                code.push_str("          (i32.eqz\n");
                code.push_str(&cond_code);
                code.push_str("          )\n");
                code.push_str("        )\n");
                code.push_str(&body_code);
                code.push_str(&format!("        (br ${}_inner)\n", loop_label));
                code.push_str("      )\n");
                code.push_str("    )\n");
                Ok(code)
            }
            Stmt::For { var_name, start, end, body } => {
                let loop_label = self.fresh_label();
                let ty = WasmType::I64; // for 循环变量通常是 i64
                let mut code = format!("    (local ${} {})\n", var_name, ty.to_wat());
                self.var_type_map.insert(var_name.clone(), ty);

                let start_code = self.generate_expr(start)?;
                code.push_str(&start_code);
                code.push_str(&format!("    (local.set ${})\n", var_name));

                code.push_str(&format!("    (block ${}_outer\n", loop_label));
                code.push_str(&format!("      (loop ${}_inner\n", loop_label));
                
                let end_code = self.generate_expr(end)?;
                code.push_str(&format!("        (br_if ${}_outer\n", loop_label));
                code.push_str("          (i64.ge_s\n");
                code.push_str(&format!("            (local.get ${})\n", var_name));
                code.push_str(&end_code);
                code.push_str("          )\n");
                code.push_str("        )\n");

                let body_code = self.generate_block(body)?;
                code.push_str(&body_code);

                code.push_str(&format!("        (local.get ${})\n", var_name));
                code.push_str("        (i64.const 1)\n");
                code.push_str("        (i64.add)\n");
                code.push_str(&format!("        (local.set ${})\n", var_name));

                code.push_str(&format!("        (br ${}_inner)\n", loop_label));
                code.push_str("      )\n");
                code.push_str("    )\n");
                Ok(code)
            }
            Stmt::ForIterable { var_name: _, iterable: _, body: _ } => {
                Err("WASM backend does not support ForIterable yet".to_string())
            }
            Stmt::Loop(body) => {
                let loop_label = self.fresh_label();
                let body_code = self.generate_block(body)?;
                let mut code = format!("    (block ${}_outer\n", loop_label);
                code.push_str(&format!("      (loop ${}_inner\n", loop_label));
                code.push_str(&body_code);
                code.push_str(&format!("        (br ${}_inner)\n", loop_label));
                code.push_str("      )\n");
                code.push_str("    )\n");
                Ok(code)
            }
            Stmt::Break => {
                Ok("    (br 1)\n".to_string())
            }
            Stmt::Continue => {
                Ok("    (br 0)\n".to_string())
            }
            _ => Ok(String::new()),
        }
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Int(n) => {
                Ok(format!("    (i64.const {})\n", n))
            }
            Expr::Float(f) => {
                Ok(format!("    (f64.const {})\n", f))
            }
            Expr::Bool(b) => {
                Ok(format!("    (i32.const {})\n", if *b { 1 } else { 0 }))
            }
            Expr::Ident(name) => {
                if let Some(ty) = self.var_type_map.get(name) {
                    Ok(format!("    (local.get ${})\n", name))
                } else {
                    Err(format!("Undefined variable: {}", name))
                }
            }
            Expr::Binary { op, left, right } => {
                let left_code = self.generate_expr(left)?;
                let right_code = self.generate_expr(right)?;
                let mut code = left_code;
                code.push_str(&right_code);

                // 检测类型：如果任一操作数是 f64，使用 f64 指令
                let is_float = match (left.as_ref(), right.as_ref()) {
                    (Expr::Float(_), _) | (_, Expr::Float(_)) => true,
                    _ => false,
                };

                let op_wat = match op {
                    BinOp::Add => "add",
                    BinOp::Sub => "sub",
                    BinOp::Mul => "mul",
                    BinOp::Div => "div",
                    BinOp::Mod => "rem",
                    BinOp::Eq => "eq",
                    BinOp::Neq => "ne",
                    BinOp::Lt => "lt",
                    BinOp::Gt => "gt",
                    BinOp::LtEq => "le",
                    BinOp::GtEq => "ge",
                    BinOp::And => "and",
                    BinOp::Or => "or",
                    BinOp::Pipe => return Err("pipe operator not supported in WASM backend".to_string()),
                };

                if is_float {
                    code.push_str(&format!("    (f64.{})\n", op_wat));
                } else {
                    code.push_str(&format!("    (i64.{})\n", op_wat));
                }
                Ok(code)
            }
            Expr::Unary { op, operand } => {
                let operand_code = self.generate_expr(operand)?;
                let mut code = operand_code;
                match op {
                    UnaryOp::Neg => code.push_str("    (i64.neg)\n"),
                    UnaryOp::Not => {
                        code.push_str("    (i64.const 1)\n");
                        code.push_str("    (i64.xor)\n");
                    }
                }
                Ok(code)
            }
            Expr::Call { callee, args } => {
                // 用户定义的函数调用
                let mut code = String::new();
                for arg in args {
                    code.push_str(&self.generate_expr(arg)?);
                }
                code.push_str(&format!("    (call ${})\n", callee));
                Ok(code)
            }
            Expr::IfExpr { condition, then_value, else_value } => {
                let cond_code = self.generate_expr(condition)?;
                let then_code = self.generate_expr(then_value)?;
                let else_code = self.generate_expr(else_value)?;
                let mut code = cond_code;
                code.push_str("    (if (result i64)\n");
                code.push_str("      (then\n");
                code.push_str(&then_code);
                code.push_str("      )\n");
                code.push_str("      (else\n");
                code.push_str(&else_code);
                code.push_str("      )\n");
                code.push_str("    )\n");
                Ok(code)
            }
            _ => Ok(String::new()),
        }
    }
}

impl CodeGenerator for WasmBackend {
    fn generate(&mut self, program: &Program) -> Result<String, String> {
        self.generate_program(program)
    }
}

pub fn compile_to_wasm(program: &Program) -> Result<String, String> {
    let mut backend = WasmBackend::new();
    backend.generate(program)
}

pub fn compile_to_c(program: &Program) -> Result<String, String> {
    let mut backend = CBackend::new_with_defaults();
    backend.generate(program)
}

pub fn compile_to_c_with_opts(program: &Program, opt_level: OptLevel, debug_info: bool) -> Result<String, String> {
    let mut backend = CBackend::new(opt_level, debug_info);
    backend.generate(program)
}

pub fn compile_to_native(program: &Program, output_path: &str) -> Result<String, String> {
    compile_to_native_with_opts(program, output_path, OptLevel::O2, false)
}

pub fn compile_to_native_with_opts(
    program: &Program,
    output_path: &str,
    opt_level: OptLevel,
    debug_info: bool,
) -> Result<String, String> {
    let c_code = compile_to_c_with_opts(program, opt_level, debug_info)?;
    let c_path = format!("{}.c", output_path);

    std::fs::write(&c_path, &c_code)
        .map_err(|e| format!("Failed to write C file: {}", e))?;

    // Detect available C compiler: try gcc, clang, cl, cc in order
    let compiler_info = detect_c_compiler()
        .ok_or_else(|| {
            format!(
                "No C compiler found in PATH. Install one of: gcc, clang, cl (MSVC), or cc.\n\
                 Generated C source at: {}\n\
                 You can compile it manually, e.g.:\n  gcc {} -o {}",
                c_path, c_path, output_path
            )
        })?;

    let mut cmd = match compiler_info.kind.as_str() {
        "msvc" => {
            let mut cmd = std::process::Command::new(&compiler_info.path);
            cmd.arg(&c_path);
            cmd.arg(format!("/Fe:{}", output_path));
            cmd.arg(opt_level.as_msvc_flag());
            if debug_info {
                cmd.arg("/Zi");
            }
            cmd
        }
        _ => {
            // gcc / clang / cc style
            let mut cmd = std::process::Command::new(&compiler_info.path);
            cmd.arg("-std=c99");
            cmd.arg(&c_path);
            cmd.arg("-o");
            cmd.arg(output_path);
            cmd.arg(opt_level.as_c_flag());
            if debug_info {
                cmd.arg("-g");
            }
            cmd
        }
    };

    match cmd.status() {
        Ok(s) if s.success() => Ok(output_path.to_string()),
        Ok(s) => Err(format!("C compiler exited with code: {:?}", s.code())),
        Err(e) => Err(format!("Failed to invoke C compiler '{}': {}", compiler_info.path, e)),
    }
}

struct CCompilerInfo {
    kind: String,
    path: String,
}

fn detect_c_compiler() -> Option<CCompilerInfo> {
    // On Windows, prefer cl (MSVC) first; elsewhere prefer gcc/clang first.
    let candidates: &[(&str, &str)] = if cfg!(target_os = "windows") {
        &[
            ("cl", "msvc"),
            ("gcc", "gcc"),
            ("clang", "clang"),
            ("clang-cl", "msvc"),
            ("cc", "gcc"),
        ]
    } else {
        &[
            ("gcc", "gcc"),
            ("clang", "clang"),
            ("cc", "gcc"),
            ("cl", "msvc"),
        ]
    };

    for (name, kind) in candidates {
        // Try `where` on Windows, `which` elsewhere, via `Command::new`
        let resolved = if cfg!(target_os = "windows") {
            std::process::Command::new("where")
                .arg(name)
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        String::from_utf8_lossy(&o.stdout)
                            .lines()
                            .next()
                            .map(|s| s.trim().to_string())
                    } else {
                        None
                    }
                })
        } else {
            std::process::Command::new("which")
                .arg(name)
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                })
        };

        if let Some(path) = resolved {
            if !path.is_empty() {
                return Some(CCompilerInfo {
                    kind: kind.to_string(),
                    path,
                });
            }
        }
    }
    None
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
    fn test_generate_int_literal() {
        let program = parse("42");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("42LL"));
        assert!(code.contains("int main"));
    }

    #[test]
    fn test_generate_binary_add() {
        let program = parse("1 + 2");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("(1LL + 2LL)"));
    }

    #[test]
    fn test_generate_function() {
        let program = parse("fn add(a: i64, b: i64) -> i64 { return a + b; }");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("int64_t add(int64_t a, int64_t b)"));
        assert!(code.contains("return (a + b);"));
    }

    #[test]
    fn test_generate_if_else() {
        let program = parse("fn abs(x: i64) -> i64 { if x < 0 { return -x; } else { return x; } }");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("if ((x < 0LL))"));
        assert!(code.contains("return -(x);"));
    }

    #[test]
    fn test_generate_while() {
        let program = parse("fn sum(n: i64) -> i64 { let i = 0; let s = 0; while i < n { s = s + i; i = i + 1; } return s; }");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("while"));
        assert!(code.contains("(i < n)"));
    }

    #[test]
    fn test_generate_for() {
        let program = parse("fn sum(n: i64) -> i64 { let s = 0; for i in 0..n { s = s + i; } return s; }");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("for"));
    }

    #[test]
    fn test_generate_let_decl() {
        let program = parse("let x: i32 = 42; x");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("int32_t x ="));
    }

    #[test]
    fn test_generate_struct() {
        let program = parse("struct Point { x: i32, y: i32 } let p = Point { x: 1, y: 2 }; p.x");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("typedef struct"));
        assert!(code.contains("Point"));
    }

    #[test]
    fn test_generate_list() {
        let program = parse("let xs = [1, 2, 3]; xs[0]");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("LinkList"));
    }

    #[test]
    fn test_generate_enum() {
        let program = parse("enum Color { Red, Green, Blue } let c = Color::Red; c");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("_v_Red"));
    }

    #[test]
    fn test_generate_match() {
        let program = parse("fn test(x: i64) -> i64 { match x { 1 => { 10 } 2 => { 20 } _ => { 0 } } }");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("if"));
    }

    #[test]
    fn test_opt_level() {
        let program = parse("42");
        let code = compile_to_c_with_opts(&program, OptLevel::O3, true).unwrap();
        assert!(code.contains("#line"));
    }

    // ===== Phase 2.11: Integration tests for string formatting & type inference =====

    #[test]
    fn test_format_string_int() {
        let program = parse("fn main() { println(\"val = {}\", 42); }");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("printf(\"val = %lld\\n\""));
        assert!(code.contains("(long long)(42LL)"));
    }

    #[test]
    fn test_format_string_float() {
        let program = parse("fn main() { println(\"pi = {}\", 3.14); }");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("printf(\"pi = %lf\\n\""));
        assert!(!code.contains("(long long)(3.14"));
    }

    #[test]
    fn test_format_string_bool() {
        let program = parse("fn main() { println(\"flag = {}\", true); }");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("printf(\"flag = %s\\n\""));
        assert!(code.contains("? \"true\" : \"false\""));
    }

    #[test]
    fn test_format_string_str() {
        let program = parse("fn main() { println(\"hello {}\", \"world\"); }");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("printf(\"hello %s\\n\""));
    }

    #[test]
    fn test_format_string_multiple_args() {
        let program = parse("fn main() { println(\"{} + {} = {}\", 1, 2, 3); }");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("printf(\"%lld + %lld = %lld\\n\""));
    }

    #[test]
    fn test_format_string_function_call_double() {
        let program = parse("fn sq(x: f64) -> f64 { return x * x; } fn main() { println(\"sq(2) = {}\", sq(2.0)); }");
        let code = compile_to_c(&program).unwrap();
        // Function returns f64, so printf should use %lf not %lld
        assert!(code.contains("printf(\"sq(2) = %lf\\n\""));
        assert!(!code.contains("(long long)(sq"));
    }

    #[test]
    fn test_format_string_function_call_int() {
        let program = parse("fn dbl(n: i64) -> i64 { return n * 2; } fn main() { println(\"dbl(5) = {}\", dbl(5)); }");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("printf(\"dbl(5) = %lld\\n\""));
        assert!(code.contains("(long long)(dbl(5LL))"));
    }

    #[test]
    fn test_format_string_mixed_types() {
        let program = parse("fn main() { println(\"{} {} {}\", 1, 2.5, \"x\"); }");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("printf(\"%lld %lf %s\\n\""));
    }

    #[test]
    fn test_function_return_type_tracking() {
        // Ensure fn_return_types pre-scan works regardless of definition order
        let program = parse(
            "fn main() { println(\"{}\", helper()); }
             fn helper() -> f64 { return 1.5; }"
        );
        let code = compile_to_c(&program).unwrap();
        // helper() returns f64, so should use %lf
        assert!(code.contains("printf(\"%lf\\n\", helper())"));
    }

    #[test]
    fn test_struct_field_access_in_format() {
        let program = parse(
            "struct P { x: f64, y: f64 }
             fn main() { let p = P { x: 1.0, y: 2.0 }; println(\"x = {}\", p.x); }"
        );
        let code = compile_to_c(&program).unwrap();
        // p.x is f64, so should use %lf
        assert!(code.contains("printf(\"x = %lf\\n\""));
    }

    #[test]
    fn test_enum_with_payload_pathcall() {
        let program = parse(
            "enum Shape { Circle(f64), Square(f64) }
             fn main() { let c = Shape::Circle(5.0); c }"
        );
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("Shape_v_Circle"));
        assert!(code.contains(".data.Circle = { 5.0 }"));
    }

    #[test]
    fn test_nested_function_calls() {
        let program = parse(
            "fn inc(x: i64) -> i64 { return x + 1; }
             fn main() { println(\"{}\", inc(inc(5))); }"
        );
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("inc(inc(5LL))"));
        assert!(code.contains("(long long)(inc(inc(5LL)))"));
    }

    #[test]
    fn test_arithmetic_mixed_types() {
        let program = parse(
            "fn main() { let x: i64 = 10; let y: f64 = 2.5; println(\"{}\", x); }"
        );
        let code = compile_to_c(&program).unwrap();
        // x is i64, so should use %lld
        assert!(code.contains("printf(\"%lld\\n\""));
        assert!(code.contains("(long long)(x)"));
    }

    #[test]
    fn test_format_string_no_placeholders() {
        // Plain string without {} should use %s
        let program = parse("fn main() { println(\"hello world\"); }");
        let code = compile_to_c(&program).unwrap();
        assert!(code.contains("printf(\"%s\\n\", \"hello world\")"));
    }

    #[test]
    fn test_format_string_escaped_chars() {
        let program = parse("fn main() { println(\"a\\\\tb{}\", 1); }");
        let code = compile_to_c(&program).unwrap();
        // Should contain escaped backslash and tab
        assert!(code.contains("printf("));
    }

    #[test]
    fn test_compiler_detection_returns_none_when_no_compiler() {
        // This is a smoke test - we can't guarantee compiler availability,
        // but the function should not panic
        let _ = detect_c_compiler();
    }

    // ===== Python backend tests =====

    #[test]
    fn test_py_int_literal() {
        let program = parse("42");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("42"));
        assert!(code.contains("__main__"));
    }

    #[test]
    fn test_py_binary_add() {
        let program = parse("1 + 2");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("(1 + 2)"));
    }

    #[test]
    fn test_py_function() {
        let program = parse("fn add(a: i64, b: i64) -> i64 { return a + b; }");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("def add(a, b):"));
        assert!(code.contains("return (a + b)"));
    }

    #[test]
    fn test_py_if_else() {
        let program = parse("fn abs(x: i64) -> i64 { if x < 0 { return -x; } else { return x; } }");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("if (x < 0):"));
        assert!(code.contains("else:"));
        assert!(code.contains("return (-x)"));
    }

    #[test]
    fn test_py_while_loop() {
        let program = parse("fn count(n: i64) -> i64 { let i = 0; while i < n { i = i + 1; } return i; }");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("while"));
        assert!(code.contains("(i < n)"));
    }

    #[test]
    fn test_py_for_loop() {
        let program = parse("fn sum(n: i64) -> i64 { let s = 0; for i in 0..n { s = s + i; } return s; }");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("for i in range(0, n):"));
    }

    #[test]
    fn test_py_let_decl() {
        let program = parse("let x = 42; x");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("x = 42"));
    }

    #[test]
    fn test_py_struct() {
        let program = parse("struct Point { x: i32, y: i32 } let p = Point { x: 1, y: 2 }; p.x");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("class Point:"));
        assert!(code.contains("def __init__(self, x, y):"));
        assert!(code.contains("self.x = x"));
    }

    #[test]
    fn test_py_list() {
        let program = parse("let xs = [1, 2, 3]; xs[0]");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("[1, 2, 3]"));
    }

    #[test]
    fn test_py_enum() {
        let program = parse("enum Color { Red, Green, Blue } let c = Color::Red; c");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("class Color:"));
        assert!(code.contains("Color.Red"));
    }

    #[test]
    fn test_py_match() {
        let program = parse("fn test(x: i64) -> i64 { match x { 1 => { return 10; } 2 => { return 20; } _ => { return 0; } } }");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("if") || code.contains("elif"));
    }

    #[test]
    fn test_py_println_format() {
        let program = parse("fn main() { println(\"val = {}\", 42); }");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("print("));
        assert!(code.contains("format"));
    }

    #[test]
    fn test_py_println_plain() {
        let program = parse("fn main() { println(\"hello\"); }");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("print(\"hello\")"));
    }

    #[test]
    fn test_py_main_function() {
        let program = parse("fn main() { println(\"hello world\"); }");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("def main():"));
        assert!(code.contains("__main__"));
        assert!(code.contains("main()"));
    }

    #[test]
    fn test_py_bool_and_none() {
        let program = parse("fn main() { let b = true; let n = none; }");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("True"));
        assert!(code.contains("None"));
    }

    #[test]
    fn test_py_string() {
        let program = parse("fn main() { let s = \"hello\"; println(s); }");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("\"hello\""));
    }

    #[test]
    fn test_py_async_fn() {
        let program = parse("async fn fetch() -> i64 { return 42; }");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("async def fetch():"));
    }

    #[test]
    fn test_py_logical_ops() {
        let program = parse("fn test(a: bool, b: bool) -> bool { return a and b; }");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("and"));
    }

    #[test]
    fn test_py_enum_with_payload() {
        let program = parse("enum Shape { Circle(f64), Square(f64) } fn main() { let c = Shape::Circle(5.0); c }");
        let code = compile_to_python(&program).unwrap();
        assert!(code.contains("Shape.Circle"));
    }

    // ===== WASM backend tests =====

    #[test]
    fn test_wasm_function() {
        let program = parse("fn add(a: i64, b: i64) -> i64 { return a + b; }");
        let code = compile_to_wasm(&program).unwrap();
        assert!(code.contains("(module"));
        assert!(code.contains("(func $add"));
        assert!(code.contains("(param $a i64)"));
        assert!(code.contains("(param $b i64)"));
        assert!(code.contains("(result i64)"));
        assert!(code.contains("(i64.add)"));
        assert!(code.contains("(export \"add\""));
    }

    #[test]
    fn test_wasm_control_flow() {
        let program = parse("fn abs(x: i64) -> i64 { if x < 0 { return -x; } else { return x; } }");
        let code = compile_to_wasm(&program).unwrap();
        assert!(code.contains("(if"));
        assert!(code.contains("(then"));
        assert!(code.contains("(else"));
        assert!(code.contains("(i64.lt"));
        assert!(code.contains("(i64.neg)"));
    }

    #[test]
    fn test_wasm_loop() {
        let program = parse("fn sum(n: i64) -> i64 { let s: i64 = 0; let i: i64 = 0; while i < n { s = s + i; i = i + 1; } return s; }");
        let code = compile_to_wasm(&program).unwrap();
        assert!(code.contains("(block"));
        assert!(code.contains("(loop"));
        assert!(code.contains("(br_if"));
        assert!(code.contains("(i64.add)"));
    }

    #[test]
    fn test_wasm_for_loop() {
        let program = parse("fn sum_range(n: i64) -> i64 { let s: i64 = 0; for i in 0..n { s = s + i; } return s; }");
        let code = compile_to_wasm(&program).unwrap();
        assert!(code.contains("(local $i i64)"));
        assert!(code.contains("(local $s i64)"));
        assert!(code.contains("(i64.const 1)"));
        assert!(code.contains("(i64.add)"));
    }

    #[test]
    fn test_wasm_arithmetic() {
        let program = parse("fn calc(a: i64, b: i64) -> i64 { return (a + b) * 2 - 1; }");
        let code = compile_to_wasm(&program).unwrap();
        assert!(code.contains("(i64.add)"));
        assert!(code.contains("(i64.mul)"));
        assert!(code.contains("(i64.sub)"));
    }

    #[test]
    fn test_wasm_comparison() {
        let program = parse("fn is_positive(x: i64) -> i64 { if x > 0 { return 1; } else { return 0; } }");
        let code = compile_to_wasm(&program).unwrap();
        assert!(code.contains("(i64.gt"));
    }

    #[test]
    fn test_wasm_float() {
        let program = parse("fn double(x: f64) -> f64 { return x * 2.0; }");
        let code = compile_to_wasm(&program).unwrap();
        assert!(code.contains("(param $x f64)"));
        assert!(code.contains("(result f64)"));
        assert!(code.contains("(f64.const 2"));
        assert!(code.contains("(f64.mul)"));
    }

    #[test]
    fn test_wasm_module_structure() {
        let program = parse("fn main() -> i64 { return 42; }");
        let code = compile_to_wasm(&program).unwrap();
        assert!(code.starts_with("(module"));
        assert!(code.contains("(import \"console\" \"log\""));
        assert!(code.contains("(memory (export \"memory\")"));
        assert!(code.trim().ends_with(")"));
    }

    #[test]
    fn test_wasm_multiple_functions() {
        let program = parse("fn a() -> i64 { return 1; } fn b() -> i64 { return 2; } fn c() -> i64 { return a() + b(); }");
        let code = compile_to_wasm(&program).unwrap();
        assert!(code.contains("(func $a"));
        assert!(code.contains("(func $b"));
        assert!(code.contains("(func $c"));
        assert!(code.contains("(call $a)"));
        assert!(code.contains("(call $b)"));
        assert!(code.contains("(export \"a\""));
        assert!(code.contains("(export \"b\""));
        assert!(code.contains("(export \"c\""));
    }
}
