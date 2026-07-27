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
    struct_map: HashMap<String, Vec<(String, String)>>,
    enum_map: HashMap<String, Vec<(String, usize, Vec<String>)>>,
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
            struct_map: HashMap::new(),
            enum_map: HashMap::new(),
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
            TypeAnnotation::Named(name) => Ok(format!("struct {}", name)),
            TypeAnnotation::Stream(_) => {
                Ok("struct LinkStream*".to_string())
            }
        }
    }

    fn default_type() -> String {
        "int64_t".to_string()
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
            Expr::Float(f) => Ok(format!("{}", f)),
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
                let mut arg_strs = Vec::new();
                for arg in args {
                    arg_strs.push(self.generate_expr(arg)?);
                }
                Ok(format!("{}({})", callee, arg_strs.join(", ")))
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
                Ok(format!("((LinkList){{ .count = {}, .items = (int64_t[]){{ {} }} }})", count, init_exprs.join(", ")))
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
                Ok(format!("((struct {}){{ {} }})", name, field_inits.join(", ")))
            }
            Expr::FieldAccess { target, field } => {
                let target_str = self.generate_expr(target)?;
                Ok(format!("{}.{}", target_str, field))
            }
            Expr::Path { base, segment } => {
                let _discriminant = self.enum_map.get(base);
                Ok(format!("((struct {base}){{ .discriminant = {variant_name}_v_{variant} }})",
                    base = base, variant_name = base, variant = segment))
            }
            Expr::PathCall { base, segment, args } => {
                let _discriminant = self.enum_map.get(base);
                let mut arg_strs = Vec::new();
                for arg in args {
                    arg_strs.push(self.generate_expr(arg)?);
                }
                Ok(format!("((struct {base}){{ .discriminant = {variant_name}_v_{variant}, .data.{variant} = {{ {fields} }} }})",
                    base = base, variant_name = base, variant = segment, fields = arg_strs.join(", ")))
            }
            Expr::MatchExpr { .. } => {
                Err("match expressions not supported in C backend yet".to_string())
            }
            Expr::Await(_) => {
                Err("await not supported in C backend (use async runtime)".to_string())
            }
        }
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<Vec<String>, String> {
        let mut lines = Vec::new();

        match stmt {
            Stmt::LetDecl { name, type_annotation, value } => {
                let c_type = if let Some(ta) = type_annotation {
                    Self::map_type(ta)?
                } else {
                    Self::default_type()
                };
                let c_name = name.clone();
                self.var_map.insert(name.clone(), c_name.clone());

                if let Some(val) = value {
                    let val_str = self.generate_expr(val)?;
                    lines.push(format!("{}{} {} = {};", self.indent_str(), c_type, c_name, val_str));
                } else {
                    lines.push(format!("{}{} {};", self.indent_str(), c_type, c_name));
                }
            }
            Stmt::Assign { target, value } => {
                let val_str = self.generate_expr(value)?;
                let c_name = self.var_map.get(target)
                    .cloned()
                    .unwrap_or_else(|| target.clone());
                lines.push(format!("{}{} = {};", self.indent_str(), c_name, val_str));
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
                for (pname, ptype) in params {
                    let c_type = Self::map_type(ptype)?;
                    param_strs.push(format!("{} {}", c_type, pname));
                    fn_var_map.insert(pname.clone(), pname.clone());
                }

                let param_list = if param_strs.is_empty() {
                    "void".to_string()
                } else {
                    param_strs.join(", ")
                };

                let saved_var_map = std::mem::replace(&mut self.var_map, fn_var_map);
                let saved_indent = self.indent;
                self.indent = 1;

                let body_lines = self.generate_block_lines(body)?;

                self.var_map = saved_var_map;
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
                self.var_map.insert(tmp_scrutinee.clone(), tmp_scrutinee.clone());
                lines.push(format!("{}int64_t {} = {};", self.indent_str(), tmp_scrutinee, scrutinee_str));

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
                | Stmt::StructDecl { .. } | Stmt::EnumDecl { .. } | Stmt::FlowDecl { .. } => {
                    self.generate_stmt(stmt)?;
                }
                Stmt::LetDecl { .. } | Stmt::Assign { .. } | Stmt::If { .. }
                | Stmt::While { .. } | Stmt::For { .. } | Stmt::Loop(_)
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

    let mut cmd = if cfg!(target_os = "windows") {
        let mut cmd = std::process::Command::new("cl");
        cmd.arg(&c_path);
        cmd.arg(format!("/Fe:{}", output_path));
        cmd.arg(opt_level.as_msvc_flag());
        if debug_info {
            cmd.arg("/Zi");
        }
        cmd
    } else {
        let mut cmd = std::process::Command::new("cc");
        cmd.arg(&c_path);
        cmd.arg("-o");
        cmd.arg(output_path);
        cmd.arg(opt_level.as_c_flag());
        if debug_info {
            cmd.arg("-g");
        }
        cmd
    };

    match cmd.status() {
        Ok(s) if s.success() => Ok(output_path.to_string()),
        Ok(s) => Err(format!("C compiler exited with code: {:?}", s.code())),
        Err(e) => Err(format!("Failed to invoke C compiler: {}. Is 'cl' or 'cc' available in PATH?", e)),
    }
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
        assert!(code.contains("struct Point"));
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
}
