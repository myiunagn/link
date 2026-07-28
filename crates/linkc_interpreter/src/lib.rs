use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use linkc_parser::{Program, Stmt, Expr, Block, BinOp, UnaryOp, TypeAnnotation, FnSignature, StructField, EnumVariantDecl, Pattern};
use libloading::{Library, Symbol};

pub mod python;
pub mod wasm;
pub mod java;
pub mod html;
pub mod process;
pub use python::PythonRuntime;
pub use wasm::WasmRuntime;
pub use java::JavaRuntime;
pub use html::HtmlRuntime;
pub use process::ProcessRuntime;
pub use process::{value_to_json, json_to_value};

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    None,
    List(Vec<Value>),
    Function {
        name: String,
        params: Vec<(String, TypeAnnotation)>,
        body: Block,
        closure: Environment,
    },
    NativeFunction {
        name: String,
        arity: Option<usize>,
        func: fn(&[Value]) -> Result<Value, String>,
    },
    ExternFunction {
        name: String,
        lib_key: String,
        signature: FnSignature,
    },
    PythonFunction {
        name: String,
        module: String,
        signature: FnSignature,
    },
    WasmFunction {
        name: String,
        module_path: String,
        signature: FnSignature,
    },
    Stream(Vec<Value>),
    JavaFunction {
        name: String,
        class_name: String,
        class_path: String,
        signature: FnSignature,
    },
    HtmlFunction {
        name: String,
        signature: FnSignature,
    },
    ProcessFunction {
        name: String,
        language: String,
        module: String,
        bridge_path: String,
        signature: FnSignature,
    },
    /// 结构体实例
    StructInstance {
        type_name: String,
        fields: HashMap<String, Value>,
    },
    /// 枚举值
    EnumValue {
        type_name: String,
        variant: String,
        payload: Vec<Value>,
    },
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::None, Value::None) => true,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Function { name: n1, .. }, Value::Function { name: n2, .. }) => n1 == n2,
            (Value::NativeFunction { name: n1, .. }, Value::NativeFunction { name: n2, .. }) => n1 == n2,
            (Value::ExternFunction { name: n1, .. }, Value::ExternFunction { name: n2, .. }) => n1 == n2,
            (Value::PythonFunction { name: n1, .. }, Value::PythonFunction { name: n2, .. }) => n1 == n2,
            (Value::WasmFunction { name: n1, .. }, Value::WasmFunction { name: n2, .. }) => n1 == n2,
            (Value::JavaFunction { name: n1, .. }, Value::JavaFunction { name: n2, .. }) => n1 == n2,
            (Value::HtmlFunction { name: n1, .. }, Value::HtmlFunction { name: n2, .. }) => n1 == n2,
            (Value::ProcessFunction { name: n1, .. }, Value::ProcessFunction { name: n2, .. }) => n1 == n2,
            (Value::Stream(a), Value::Stream(b)) => a == b,
            (Value::StructInstance { type_name: t1, fields: f1 },
             Value::StructInstance { type_name: t2, fields: f2 }) => t1 == t2 && f1 == f2,
            (Value::EnumValue { type_name: t1, variant: v1, payload: p1 },
             Value::EnumValue { type_name: t2, variant: v2, payload: p2 }) => {
                t1 == t2 && v1 == v2 && p1 == p2
            }
            _ => false,
        }
    }
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Str(_) => "str",
            Value::Bool(_) => "bool",
            Value::None => "none",
            Value::List(_) => "list",
            Value::Function { .. } => "function",
            Value::NativeFunction { .. } => "native_function",
            Value::ExternFunction { .. } => "extern_function",
            Value::PythonFunction { .. } => "python_function",
            Value::WasmFunction { .. } => "wasm_function",
            Value::JavaFunction { .. } => "java_function",
            Value::HtmlFunction { .. } => "html_function",
            Value::ProcessFunction { .. } => "process_function",
            Value::Stream(_) => "stream",
            Value::StructInstance { type_name, .. } => type_name,
            Value::EnumValue { type_name, .. } => type_name,
        }
    }

    fn as_int(&self) -> Result<i64, String> {
        match self {
            Value::Int(n) => Ok(*n),
            _ => Err(format!("Expected int, got {}", self.type_name())),
        }
    }

    fn as_float(&self) -> Result<f64, String> {
        match self {
            Value::Float(n) => Ok(*n),
            Value::Int(n) => Ok(*n as f64),
            _ => Err(format!("Expected float, got {}", self.type_name())),
        }
    }

    fn as_bool(&self) -> Result<bool, String> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(format!("Expected bool, got {}", self.type_name())),
        }
    }

    fn as_string(&self) -> Result<String, String> {
        match self {
            Value::Str(s) => Ok(s.clone()),
            _ => Err(format!("Expected str, got {}", self.type_name())),
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::None => false,
            Value::List(items) => !items.is_empty(),
            Value::Str(s) => !s.is_empty(),
            _ => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    pub variables: HashMap<String, Value>,
    pub enclosing: Option<Box<Environment>>,
}

#[derive(Debug, Default)]
pub struct InterpContext {
    pub libs: HashMap<String, Library>,
    pub python: Option<PythonRuntime>,
    pub wasm: Option<WasmRuntime>,
    pub java: Option<JavaRuntime>,
    pub html: Option<HtmlRuntime>,
    pub process: Option<ProcessRuntime>,
    /// 已注册的 struct 类型定义: name -> fields
    pub struct_defs: HashMap<String, Vec<StructField>>,
    /// 已注册的 enum 类型定义: name -> variants
    pub enum_defs: HashMap<String, Vec<EnumVariantDecl>>,
    /// 当前函数的 return 值（用于跨语句块传递返回值）
    pub return_value: Option<Value>,
}

impl InterpContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取或初始化 Python 运行时
    pub fn python(&mut self) -> Result<&PythonRuntime, String> {
        if self.python.is_none() {
            self.python = Some(PythonRuntime::new()?);
        }
        Ok(self.python.as_ref().unwrap())
    }

    /// 获取或初始化 WASM 运行时
    pub fn wasm(&mut self) -> &mut WasmRuntime {
        if self.wasm.is_none() {
            self.wasm = Some(WasmRuntime::new());
        }
        self.wasm.as_mut().unwrap()
    }

    /// 获取或初始化 Java 运行时
    pub fn java(&mut self) -> &mut JavaRuntime {
        if self.java.is_none() {
            self.java = Some(JavaRuntime::new());
        }
        self.java.as_mut().unwrap()
    }

    /// 获取或初始化 HTML 运行时
    pub fn html(&mut self) -> &mut HtmlRuntime {
        if self.html.is_none() {
            self.html = Some(HtmlRuntime::new());
        }
        self.html.as_mut().unwrap()
    }

    /// 获取或初始化 Process 运行时
    pub fn process(&mut self) -> &mut ProcessRuntime {
        if self.process.is_none() {
            self.process = Some(ProcessRuntime::new());
        }
        self.process.as_mut().unwrap()
    }
}

impl Environment {
    pub fn new() -> Self {
        let mut env = Self { variables: HashMap::new(), enclosing: None };
        register_builtins(&mut env);
        env
    }

    pub fn extend(enclosing: Environment) -> Self {
        Self {
            variables: HashMap::new(),
            enclosing: Some(Box::new(enclosing)),
        }
    }

    pub fn get(&self, name: &str) -> Result<Value, String> {
        if let Some(val) = self.variables.get(name) {
            return Ok(val.clone());
        }
        if let Some(ref enclosing) = self.enclosing {
            return enclosing.get(name);
        }
        Err(format!("Undefined variable: {}", name))
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), String> {
        if self.variables.contains_key(&name) {
            self.variables.insert(name, value);
            return Ok(());
        }
        if let Some(ref mut enclosing) = self.enclosing {
            return enclosing.assign(name, value);
        }
        Err(format!("Undefined variable: {}", name))
    }
}

pub fn eval_program(program: &Program, env: &mut Environment, ctx: &mut InterpContext) -> Result<Value, String> {
    match program {
        Program::Block(stmts) => eval_block(stmts, env, ctx),
    }
}

pub fn eval_block(stmts: &[Stmt], env: &mut Environment, ctx: &mut InterpContext) -> Result<Value, String> {
    let mut last = Value::None;
    for stmt in stmts {
        last = eval_stmt(stmt, env, ctx)?;
    }
    Ok(last)
}

fn eval_stmt(stmt: &Stmt, env: &mut Environment, ctx: &mut InterpContext) -> Result<Value, String> {
    match stmt {
        Stmt::Expr(expr) => eval_expr(expr, env, ctx),
        Stmt::LetDecl { name, value, .. } => {
            let val = match value {
                Some(expr) => eval_expr(expr, env, ctx)?,
                None => Value::None,
            };
            env.set(name.clone(), val);
            Ok(Value::None)
        }
        Stmt::Assign { target, value } => {
            let val = eval_expr(value, env, ctx)?;
            env.assign(target.clone(), val)?;
            Ok(Value::None)
        }
        Stmt::FnDecl { name, params, body, .. } => {
            let func = Value::Function {
                name: name.clone(),
                params: params.clone(),
                body: body.clone(),
                closure: env.clone(),
            };
            env.set(name.clone(), func);
            Ok(Value::None)
        }
        Stmt::Return(value) => {
            let val = match value {
                Some(expr) => eval_expr(expr, env, ctx)?,
                None => Value::None,
            };
            // 用 InterpContext 存储 return 值，避免序列化复杂类型
            ctx.return_value = Some(val);
            Err("__return__".to_string())
        }
        Stmt::If { condition, then_branch, else_branch } => {
            let cond = eval_expr(condition, env, ctx)?;
            if cond.is_truthy() {
                eval_block(&then_branch.stmts, env, ctx)
            } else if let Some(else_blk) = else_branch {
                eval_block(&else_blk.stmts, env, ctx)
            } else {
                Ok(Value::None)
            }
        }
        Stmt::While { condition, body } => {
            loop {
                let cond = eval_expr(condition, env, ctx)?;
                if !cond.is_truthy() { break; }
                match eval_block(&body.stmts, env, ctx) {
                    Ok(_) => continue,
                    Err(e) if e.starts_with("__return__") => return Err(e),
                    Err(e) => return Err(e),
                }
            }
            Ok(Value::None)
        }
        Stmt::For { var_name, start, end, body } => {
            let start_val = eval_expr(start, env, ctx)?.as_int()?;
            let end_val = eval_expr(end, env, ctx)?.as_int()?;
            for i in start_val..end_val {
                env.set(var_name.clone(), Value::Int(i));
                match eval_block(&body.stmts, env, ctx) {
                    Ok(_) => continue,
                    Err(e) if e.starts_with("__return__") => return Err(e),
                    Err(e) => return Err(e),
                }
            }
            Ok(Value::None)
        }
        Stmt::Loop(body) => {
            loop {
                match eval_block(&body.stmts, env, ctx) {
                    Ok(_) => continue,
                    Err(e) if e == "__break__" => break,
                    Err(e) if e.starts_with("__return__") => return Err(e),
                    Err(e) => return Err(e),
                }
            }
            Ok(Value::None)
        }
        Stmt::Break => Err("__break__".to_string()),
        Stmt::Continue => Ok(Value::None),
        Stmt::ExternDecl { language, module, decls } => {
            eval_extern_decl(language, module.as_deref(), decls, env, ctx)
        }
        Stmt::ExportDecl { language: _, module: _, decls: _ } => {
            // export 块暂时只在 AST 中存在,运行时不执行操作
            // (后续会生成 C 头文件 / Python 模块等)
            Ok(Value::None)
        }
        Stmt::StructDecl { name, fields } => {
            ctx.struct_defs.insert(name.clone(), fields.clone());
            Ok(Value::None)
        }
        Stmt::EnumDecl { name, variants } => {
            ctx.enum_defs.insert(name.clone(), variants.clone());
            Ok(Value::None)
        }
        Stmt::Match { scrutinee, arms } => {
            let val = eval_expr(scrutinee, env, ctx)?;
            eval_match(&val, arms, env, ctx)
        }
        Stmt::FlowDecl { name, description, source, pipeline } => {
            eval_flow(name, description.as_deref(), source.as_ref(), pipeline, env, ctx)
        }
        Stmt::ModDecl { name: _ } => {
            // 模块声明是元数据,解释器无需执行
            Ok(Value::None)
        }
        Stmt::UseDecl { path: _, alias: _ } => {
            // 导入声明由 CLI 编译前处理(加载并合并 AST),解释器无需执行
            Ok(Value::None)
        }
    }
}

/// 执行 match 语句: 按顺序尝试匹配每个 arm 的 pattern
fn eval_match(val: &Value, arms: &[linkc_parser::MatchArm], env: &mut Environment, ctx: &mut InterpContext) -> Result<Value, String> {
    for arm in arms {
        // 为每个 arm 创建一个子作用域,这样 pattern 绑定的变量不会泄露到外部
        let mut arm_env = Environment::extend(env.clone());
        if try_match_pattern(&arm.pattern, val, &mut arm_env)? {
            return match eval_block(&arm.body.stmts, &mut arm_env, ctx) {
                Ok(v) => Ok(v),
                Err(e) if e == "__return__" => Err(e),
                Err(e) => Err(e),
            };
        }
    }
    Err(format!("No match arm matched value: {}", value_to_string(val)))
}

/// 执行 flow 声明块
///
/// v0.1 语义:flow 是声明式数据流定义,但在树漫游解释器中"自动调度"
/// 等价于"立即在子作用域中求值 pipeline"。
///
/// - 若有 `source:`,先求值 source 表达式并绑定到变量 `source`(在子作用域中)
/// - 然后求值 `pipeline:` 表达式(其中可引用 `source`)
/// - pipeline 通常以 `| for_each(...)` 或 `| collect` 结尾,求值时即触发执行
///
/// 多个 flow 块按源码出现顺序串行执行。返回 pipeline 的求值结果。
fn eval_flow(
    name: &str,
    description: Option<&str>,
    source: Option<&linkc_parser::Expr>,
    pipeline: &linkc_parser::Expr,
    env: &mut Environment,
    ctx: &mut InterpContext,
) -> Result<Value, String> {
    // 为 flow 创建独立子作用域,source 变量不会污染外层
    let mut flow_env = Environment::extend(env.clone());

    // 若有 source 字段,求值并绑定到 `source` 变量
    if let Some(src_expr) = source {
        let src_val = eval_expr(src_expr, &mut flow_env, ctx)?;
        flow_env.set("source".to_string(), src_val);
    }

    // 求值 pipeline
    let result = eval_expr(pipeline, &mut flow_env, ctx)?;

    // flow 名称与描述当前仅作为元数据(未来可用于调度器注册)
    let _ = (name, description);

    Ok(result)
}

/// 尝试用 pattern 匹配 value,匹配成功则将绑定写入 env 并返回 true
fn try_match_pattern(pattern: &Pattern, val: &Value, env: &mut Environment) -> Result<bool, String> {
    match pattern {
        Pattern::Wildcard => Ok(true),
        Pattern::Bind(name) => {
            if name == "_" {
                return Ok(true);
            }
            env.set(name.clone(), val.clone());
            Ok(true)
        }
        Pattern::Literal(expr) => {
            // 字面量模式仅支持常量表达式
            let lit_val = match expr {
                Expr::Int(n) => Value::Int(*n),
                Expr::Float(n) => Value::Float(*n),
                Expr::Str(s) => Value::Str(s.clone()),
                Expr::Bool(b) => Value::Bool(*b),
                Expr::None => Value::None,
                _ => return Err(format!("Unsupported literal pattern")),
            };
            Ok(val == &lit_val)
        }
        Pattern::EnumVariant { type_name, variant } => {
            match val {
                Value::EnumValue { type_name: tn, variant: v, payload } => {
                    if payload.is_empty() {
                        Ok(tn == type_name && v == variant)
                    } else {
                        Ok(false)
                    }
                }
                _ => Ok(false),
            }
        }
        Pattern::EnumVariantWithPayload { type_name, variant, bindings } => {
            match val {
                Value::EnumValue { type_name: tn, variant: v, payload } => {
                    if tn == type_name && v == variant && payload.len() == bindings.len() {
                        for (binding, value) in bindings.iter().zip(payload.iter()) {
                            if binding != "_" {
                                env.set(binding.clone(), value.clone());
                            }
                        }
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
                _ => Ok(false),
            }
        }
    }
}

fn value_to_string(val: &Value) -> String {
    match val {
        Value::Int(n) => n.to_string(),
        Value::Float(n) => n.to_string(),
        Value::Str(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::None => "none".to_string(),
        Value::List(items) => {
            let parts: Vec<String> = items.iter().map(value_to_string).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Function { name, .. } => format!("<fn {}>", name),
        Value::NativeFunction { name, .. } => format!("<native fn {}>", name),
        Value::ExternFunction { name, .. } => format!("<extern fn {}>", name),
        Value::PythonFunction { name, .. } => format!("<python fn {}>", name),
        Value::WasmFunction { name, .. } => format!("<wasm fn {}>", name),
        Value::JavaFunction { name, .. } => format!("<java fn {}>", name),
        Value::HtmlFunction { name, .. } => format!("<html fn {}>", name),
        Value::ProcessFunction { name, language, .. } => format!("<{} fn {}>", language, name),
        Value::Stream(items) => {
            let parts: Vec<String> = items.iter().map(value_to_string).collect();
            format!("stream[{}]", parts.join(", "))
        }
        Value::StructInstance { type_name, fields } => {
            let parts: Vec<String> = fields.iter()
                .map(|(k, v)| format!("{}: {}", k, value_to_string(v)))
                .collect();
            format!("{} {{ {} }}", type_name, parts.join(", "))
        }
        Value::EnumValue { type_name, variant, payload } => {
            if payload.is_empty() {
                format!("{}::{}", type_name, variant)
            } else {
                let parts: Vec<String> = payload.iter().map(value_to_string).collect();
                format!("{}::{}({})", type_name, variant, parts.join(", "))
            }
        }
    }
}

fn eval_expr(expr: &Expr, env: &mut Environment, ctx: &mut InterpContext) -> Result<Value, String> {
    match expr {
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Float(n) => Ok(Value::Float(*n)),
        Expr::Str(s) => Ok(Value::Str(s.clone())),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::None => Ok(Value::None),
        Expr::Ident(name) => env.get(name),
        Expr::List(items) => {
            let mut values = Vec::new();
            for item in items {
                values.push(eval_expr(item, env, ctx)?);
            }
            Ok(Value::List(values))
        }
        Expr::Index { target, index } => {
            let target_val = eval_expr(target, env, ctx)?;
            let index_val = eval_expr(index, env, ctx)?;
            match (target_val, index_val) {
                (Value::List(items), Value::Int(i)) => {
                    if i < 0 || i >= items.len() as i64 {
                        return Err(format!("Index {} out of bounds for list of length {}", i, items.len()));
                    }
                    Ok(items[i as usize].clone())
                }
                (Value::Str(s), Value::Int(i)) => {
                    if i < 0 || i >= s.len() as i64 {
                        return Err(format!("Index {} out of bounds for string of length {}", i, s.len()));
                    }
                    let c = s.chars().nth(i as usize).unwrap();
                    Ok(Value::Str(c.to_string()))
                }
                (v, _) => Err(format!("Cannot index into type {}", v.type_name())),
            }
        }
        Expr::Binary { op, left, right } => {
            if *op == BinOp::Pipe {
                eval_pipe_expr(left, right, env, ctx)
            } else {
                let left_val = eval_expr(left, env, ctx)?;
                let right_val = eval_expr(right, env, ctx)?;
                eval_binary_op(op, &left_val, &right_val)
            }
        }
        Expr::Unary { op, operand } => {
            let val = eval_expr(operand, env, ctx)?;
            eval_unary_op(op, &val)
        }
        Expr::Call { callee, args } => {
            let func = env.get(callee)?;
            let arg_vals: Vec<Value> = args.iter()
                .map(|a| eval_expr(a, env, ctx))
                .collect::<Result<Vec<_>, _>>()?;
            call_function(&func, &arg_vals, ctx)
        }
        Expr::IfExpr { condition, then_value, else_value } => {
            let cond = eval_expr(condition, env, ctx)?;
            if cond.is_truthy() {
                eval_expr(then_value, env, ctx)
            } else {
                eval_expr(else_value, env, ctx)
            }
        }
        Expr::BlockExpr(block) => eval_block(&block.stmts, env, ctx),
        Expr::FieldAccess { target, field } => {
            let target_val = eval_expr(target, env, ctx)?;
            match target_val {
                Value::StructInstance { fields, .. } => {
                    fields.get(field).cloned().ok_or_else(|| format!("No such field: {}", field))
                }
                _ => Err(format!("Cannot access field '{}' on type {}", field, target_val.type_name())),
            }
        }
        Expr::Path { base, segment } => {
            // 检查是否是已注册的 enum 类型
            if ctx.enum_defs.contains_key(base) {
                // 验证变体存在且无参数
                let variants = ctx.enum_defs.get(base).unwrap();
                let found = variants.iter().find(|v| v.name == *segment && v.payload.is_empty());
                if found.is_none() {
                    return Err(format!("No such variant '{}' in enum {} (or it requires payload)", segment, base));
                }
                Ok(Value::EnumValue {
                    type_name: base.clone(),
                    variant: segment.clone(),
                    payload: Vec::new(),
                })
            } else {
                Err(format!("Unknown type or path: {}::{}", base, segment))
            }
        }
        Expr::StructInit { name, fields } => {
            // 检查是否是已注册的 struct 类型
            let struct_def = ctx.struct_defs.get(name)
                .ok_or_else(|| format!("Unknown struct type: {}", name))?
                .clone();
            // 验证字段
            let mut field_values = HashMap::new();
            // 初始化所有字段为 None
            for sf in &struct_def {
                field_values.insert(sf.name.clone(), Value::None);
            }
            // 应用用户提供的字段值
            for (fname, fexpr) in fields {
                if !struct_def.iter().any(|sf| sf.name == *fname) {
                    return Err(format!("Struct {} has no field '{}'", name, fname));
                }
                let fval = eval_expr(fexpr, env, ctx)?;
                field_values.insert(fname.clone(), fval);
            }
            Ok(Value::StructInstance {
                type_name: name.clone(),
                fields: field_values,
            })
        }
        Expr::PathCall { base, segment, args } => {
            // 带参数的枚举变体构造
            if ctx.enum_defs.contains_key(base) {
                let variants = ctx.enum_defs.get(base).unwrap();
                let found = variants.iter().find(|v| v.name == *segment);
                if found.is_none() {
                    return Err(format!("No such variant '{}' in enum {}", segment, base));
                }
                let variant_def = found.unwrap();
                if variant_def.payload.len() != args.len() {
                    return Err(format!(
                        "Variant {}::{} expects {} args, got {}",
                        base, segment, variant_def.payload.len(), args.len()
                    ));
                }
                let mut payload = Vec::with_capacity(args.len());
                for arg in args {
                    payload.push(eval_expr(arg, env, ctx)?);
                }
                Ok(Value::EnumValue {
                    type_name: base.clone(),
                    variant: segment.clone(),
                    payload,
                })
            } else {
                Err(format!("Unknown type or path: {}::{}", base, segment))
            }
        }
        Expr::MatchExpr { scrutinee, arms } => {
            let val = eval_expr(scrutinee, env, ctx)?;
            eval_match(&val, arms, env, ctx)
        }
        Expr::Await(inner) => {
            // v0.1 树漫游解释器:await 直接求值内部表达式(阻塞语义)
            // 真正的 async/await 并发调度留待 v0.2 LLVM 后端 + Tokio-like 运行时
            eval_expr(inner, env, ctx)
        }
    }
}

fn eval_binary_op(op: &BinOp, left: &Value, right: &Value) -> Result<Value, String> {
    match op {
        BinOp::Add => {
            match (left, right) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{}{}", a, b))),
                (Value::List(a), Value::List(b)) => {
                    let mut combined = a.clone();
                    combined.extend(b.iter().cloned());
                    Ok(Value::List(combined))
                }
                _ => Ok(Value::Float(left.as_float()? + right.as_float()?)),
            }
        }
        BinOp::Sub => {
            if left.type_name() == "int" && right.type_name() == "int" {
                Ok(Value::Int(left.as_int()? - right.as_int()?))
            } else {
                Ok(Value::Float(left.as_float()? - right.as_float()?))
            }
        }
        BinOp::Mul => {
            if left.type_name() == "int" && right.type_name() == "int" {
                Ok(Value::Int(left.as_int()? * right.as_int()?))
            } else {
                Ok(Value::Float(left.as_float()? * right.as_float()?))
            }
        }
        BinOp::Div => {
            if left.type_name() == "int" && right.type_name() == "int" {
                let r = right.as_int()?;
                if r == 0 { return Err("Division by zero".to_string()); }
                Ok(Value::Int(left.as_int()? / r))
            } else {
                let r = right.as_float()?;
                if r == 0.0 { return Err("Division by zero".to_string()); }
                Ok(Value::Float(left.as_float()? / r))
            }
        }
        BinOp::Mod => {
            let r = right.as_int()?;
            if r == 0 { return Err("Modulo by zero".to_string()); }
            Ok(Value::Int(left.as_int()? % r))
        }
        BinOp::Eq => Ok(Value::Bool(left == right)),
        BinOp::Neq => Ok(Value::Bool(left != right)),
        BinOp::Lt => Ok(Value::Bool(cmp_values(left, right)? < 0)),
        BinOp::Gt => Ok(Value::Bool(cmp_values(left, right)? > 0)),
        BinOp::LtEq => Ok(Value::Bool(cmp_values(left, right)? <= 0)),
        BinOp::GtEq => Ok(Value::Bool(cmp_values(left, right)? >= 0)),
        BinOp::And => Ok(Value::Bool(left.as_bool()? && right.as_bool()?)),
        BinOp::Or => Ok(Value::Bool(left.as_bool()? || right.as_bool()?)),
        BinOp::Pipe => unreachable!("Pipe should be handled in eval_expr"),
    }
}

fn eval_pipe_expr(left: &Expr, right: &Expr, env: &mut Environment, ctx: &mut InterpContext) -> Result<Value, String> {
    let left_val = eval_expr(left, env, ctx)?;
    apply_pipe_value(left_val, right, env, ctx)
}

fn apply_pipe_value(left_val: Value, right: &Expr, env: &mut Environment, ctx: &mut InterpContext) -> Result<Value, String> {
    match right {
        Expr::Ident(callee) => {
            let func = env.get(callee)?;
            call_function(&func, &[left_val], ctx)
        }
        Expr::Call { callee, args } => {
            let func = env.get(callee)?;
            let mut arg_vals = vec![left_val];
            for arg in args {
                arg_vals.push(eval_expr(arg, env, ctx)?);
            }
            call_function(&func, &arg_vals, ctx)
        }
        Expr::Binary { op: BinOp::Pipe, left: mid, right: tail } => {
            let mid_result = apply_pipe_value(left_val, mid, env, ctx)?;
            apply_pipe_value(mid_result, tail, env, ctx)
        }
        _ => Err("Invalid pipe target: must be a function name or call".to_string()),
    }
}

fn eval_unary_op(op: &UnaryOp, operand: &Value) -> Result<Value, String> {
    match op {
        UnaryOp::Neg => {
            if operand.type_name() == "int" {
                Ok(Value::Int(-operand.as_int()?))
            } else {
                Ok(Value::Float(-operand.as_float()?))
            }
        }
        UnaryOp::Not => Ok(Value::Bool(!operand.as_bool()?)),
    }
}

fn cmp_values(left: &Value, right: &Value) -> Result<i32, String> {
    match (left, right) {
        (Value::Int(l), Value::Int(r)) => {
            if l < r { Ok(-1) } else if l > r { Ok(1) } else { Ok(0) }
        }
        (Value::Float(l), Value::Float(r)) => {
            if l < r { Ok(-1) } else if l > r { Ok(1) } else { Ok(0) }
        }
        (Value::Int(l), Value::Float(r)) => {
            let l = *l as f64;
            if l < *r { Ok(-1) } else if l > *r { Ok(1) } else { Ok(0) }
        }
        (Value::Float(l), Value::Int(r)) => {
            let r = *r as f64;
            if *l < r { Ok(-1) } else if *l > r { Ok(1) } else { Ok(0) }
        }
        _ => Err(format!("Cannot compare {} and {}", left.type_name(), right.type_name())),
    }
}

pub fn call_function(func: &Value, args: &[Value], ctx: &mut InterpContext) -> Result<Value, String> {
    match func {
        Value::Function { name, params, body, closure } => {
            if args.len() != params.len() {
                return Err(format!("Function {} expects {} args, got {}", name, params.len(), args.len()));
            }
            let mut local_env = Environment::extend(closure.clone());
            // 支持递归:将函数自身注入局部环境,使其在函数体内可被名字访问
            if !name.is_empty() {
                local_env.set(name.clone(), func.clone());
            }
            for (i, (param_name, _)) in params.iter().enumerate() {
                local_env.set(param_name.clone(), args[i].clone());
            }
            match eval_block(&body.stmts, &mut local_env, ctx) {
                Ok(val) => Ok(val),
                Err(e) if e == "__return__" => {
                    // 从 ctx 取出 return 值
                    Ok(ctx.return_value.take().unwrap_or(Value::None))
                }
                Err(e) => Err(e),
            }
        }
        Value::NativeFunction { name, arity, func } => {
            if let Some(n) = arity {
                if args.len() != *n {
                    return Err(format!("Function {} expects {} args, got {}", name, n, args.len()));
                }
            }
            func(args)
        }
        Value::ExternFunction { name, lib_key, signature } => {
            call_extern_function(name, lib_key, signature, args, ctx)
        }
        Value::PythonFunction { name, module, signature } => {
            call_python_function(name, module, signature, args, ctx)
        }
        Value::WasmFunction { name, module_path, signature } => {
            call_wasm_function(name, module_path, signature, args, ctx)
        }
        Value::JavaFunction { name, class_name, class_path, signature } => {
            call_java_function(name, class_name, class_path, signature, args, ctx)
        }
        Value::HtmlFunction { name, signature } => {
            call_html_function(name, signature, args, ctx)
        }
        Value::ProcessFunction { name, language, module, bridge_path, signature } => {
            call_process_function(name, language, module, bridge_path, signature, args, ctx)
        }
        _ => Err(format!("Cannot call {}", func.type_name())),
    }
}

fn call_python_function(
    name: &str,
    module: &str,
    signature: &FnSignature,
    args: &[Value],
    ctx: &mut InterpContext,
) -> Result<Value, String> {
    if args.len() != signature.params.len() {
        return Err(format!("Python function {} expects {} args, got {}", name, signature.params.len(), args.len()));
    }
    let ret_type = signature.return_type.clone().unwrap_or(TypeAnnotation::Unit);
    // 先借用 Python 运行时,取出需要的引用后释放借用
    let py = ctx.python()?;
    py.call_module_func(module, name, args, &ret_type)
}

fn call_wasm_function(
    name: &str,
    module_path: &str,
    signature: &FnSignature,
    args: &[Value],
    ctx: &mut InterpContext,
) -> Result<Value, String> {
    if args.len() != signature.params.len() {
        return Err(format!("WASM function {} expects {} args, got {}", name, signature.params.len(), args.len()));
    }
    let ret_type = signature.return_type.as_ref();
    let wasm = ctx.wasm();
    wasm.call_func(module_path, name, args, ret_type)
}

fn call_java_function(
    name: &str,
    class_name: &str,
    class_path: &str,
    signature: &FnSignature,
    args: &[Value],
    ctx: &mut InterpContext,
) -> Result<Value, String> {
    if args.len() != signature.params.len() {
        return Err(format!("Java function {} expects {} args, got {}", name, signature.params.len(), args.len()));
    }
    let ret_type = signature.return_type.as_ref();
    let java = ctx.java();
    java.call_static(class_name, class_path, name, args, ret_type)
}

fn call_html_function(
    name: &str,
    signature: &FnSignature,
    args: &[Value],
    ctx: &mut InterpContext,
) -> Result<Value, String> {
    if args.len() != signature.params.len() {
        return Err(format!("HTML function {} expects {} args, got {}", name, signature.params.len(), args.len()));
    }
    let ret_type = signature.return_type.as_ref();
    let html = ctx.html();
    html.call_func(name, args, ret_type)
}

fn call_process_function(
    name: &str,
    language: &str,
    module: &str,
    bridge_path: &str,
    signature: &FnSignature,
    args: &[Value],
    ctx: &mut InterpContext,
) -> Result<Value, String> {
    if args.len() != signature.params.len() {
        return Err(format!("{} function {} expects {} args, got {}", language, name, signature.params.len(), args.len()));
    }
    let ret_type = signature.return_type.as_ref();
    let process = ctx.process();
    if !bridge_path.is_empty() {
        process.set_bridge(language, bridge_path);
    }
    process.call_func(language, module, name, args, ret_type)
}

pub fn eval_extern_decl(
    language: &str,
    module: Option<&str>,
    decls: &[FnSignature],
    env: &mut Environment,
    ctx: &mut InterpContext,
) -> Result<Value, String> {
    let lang_lower = language.to_lowercase();

    match lang_lower.as_str() {
        // C 和 C++ 共用 C ABI 调用路径
        // C++ 函数需用 extern "C" 导出,即可被 Link 直接调用
        "c" | "c++" | "cpp" => {
            let key = module.unwrap_or("c").to_string();
            if !ctx.libs.contains_key(&key) {
                let lib = load_native_library(&key)?;
                ctx.libs.insert(key.clone(), lib);
            }
            for sig in decls {
                let ext_func = Value::ExternFunction {
                    name: sig.name.clone(),
                    lib_key: key.clone(),
                    signature: sig.clone(),
                };
                env.set(sig.name.clone(), ext_func);
            }
            Ok(Value::None)
        }
        "python" | "py" => {
            // 确保模块名存在
            let module_name = module.ok_or_else(|| {
                "extern \"python\" requires a module name, e.g. extern \"python\" module \"math\"".to_string()
            })?;
            // 注册 Python 函数(运行时惰性加载 Python)
            for sig in decls {
                let py_func = Value::PythonFunction {
                    name: sig.name.clone(),
                    module: module_name.to_string(),
                    signature: sig.clone(),
                };
                env.set(sig.name.clone(), py_func);
            }
            Ok(Value::None)
        }
        "wasm" => {
            let module_path = module.ok_or_else(|| {
                "extern \"wasm\" requires a module path, e.g. extern \"wasm\" module \"path/to/file.wasm\"".to_string()
            })?;
            let wasm = ctx.wasm();
            wasm.load_module(module_path, module_path)?;
            for sig in decls {
                let wasm_func = Value::WasmFunction {
                    name: sig.name.clone(),
                    module_path: module_path.to_string(),
                    signature: sig.clone(),
                };
                env.set(sig.name.clone(), wasm_func);
            }
            Ok(Value::None)
        }
        "java" => {
            // module 格式: "<class_path>::<class_name>"
            // 例如: "build/classes::com.example.MathUtils"
            let combined = module.ok_or_else(|| {
                "extern \"java\" requires a module spec, e.g. extern \"java\" module \"build::com.example.Math\"".to_string()
            })?;
            let parts: Vec<&str> = combined.splitn(2, "::").collect();
            let (class_path, class_name) = if parts.len() == 2 {
                (parts[0], parts[1])
            } else {
                (".", combined)
            };
            for sig in decls {
                let java_func = Value::JavaFunction {
                    name: sig.name.clone(),
                    class_name: class_name.to_string(),
                    class_path: class_path.to_string(),
                    signature: sig.clone(),
                };
                env.set(sig.name.clone(), java_func);
            }
            Ok(Value::None)
        }
        "html" | "js" => {
            // HTML/JS FFI 通过 HTTP 调用远程端点
            // module 是端点地址（可选，默认使用环境变量或 http://127.0.0.1:3000）
            // 如果指定了 module，会创建一个独立的 HtmlRuntime（暂时只支持全局端点）
            for sig in decls {
                let html_func = Value::HtmlFunction {
                    name: sig.name.clone(),
                    signature: sig.clone(),
                };
                env.set(sig.name.clone(), html_func);
            }
            Ok(Value::None)
        }
        "go" | "rust" | "csharp" | "dotnet" | "php" | "ruby" | "swift" | "kotlin" => {
            // 通用进程桥接:通过子进程调用其他语言的脚本/程序
            // module 是桥接脚本路径(可选,可通过 Runtime.set_bridge() 后续设置)
            let bridge_path = module.unwrap_or("").to_string();
            for sig in decls {
                let process_func = Value::ProcessFunction {
                    name: sig.name.clone(),
                    language: lang_lower.clone(),
                    module: bridge_path.clone(),
                    bridge_path: bridge_path.clone(),
                    signature: sig.clone(),
                };
                env.set(sig.name.clone(), process_func);
            }
            Ok(Value::None)
        }
        _ => Err(format!(
            "Unsupported extern language: '{}' (supported: 'C', 'C++'/'cpp', 'python', 'wasm', 'java', 'html'/'js', 'go', 'rust', 'csharp'/'dotnet', 'php', 'ruby', 'swift', 'kotlin')",
            language
        )),
    }
}

/// 从字节流加载 WASM 模块并注册函数（用于测试或内嵌 WASM）
pub fn eval_wasm_module_from_bytes(
    module_name: &str,
    wasm_bytes: &[u8],
    decls: &[FnSignature],
    env: &mut Environment,
    ctx: &mut InterpContext,
) -> Result<Value, String> {
    let wasm = ctx.wasm();
    wasm.load_module_from_bytes(module_name, wasm_bytes)?;
    for sig in decls {
        let wasm_func = Value::WasmFunction {
            name: sig.name.clone(),
            module_path: module_name.to_string(),
            signature: sig.clone(),
        };
        env.set(sig.name.clone(), wasm_func);
    }
    Ok(Value::None)
}

/// 根据 lib_key 加载本地共享库
/// - 若 key 包含路径分隔符或已知扩展名,视为自定义库文件路径(C++ DLL/SO 等)
/// - 否则视为系统库名(如 "c" -> libc/msvcrt,"m" -> libm)
fn load_native_library(key: &str) -> Result<Library, String> {
    unsafe {
        // 判断是否是文件路径:包含分隔符,或以已知动态库扩展名结尾
        let is_path = key.contains('/') || key.contains('\\')
            || key.ends_with(".dll") || key.ends_with(".so") || key.ends_with(".dylib");

        if is_path {
            return Library::new(key)
                .map_err(|e| format!("Failed to load library '{}': {}", key, e));
        }

        // 系统库回退
        match key {
            "c" | "C" => {
                #[cfg(unix)]
                { Library::new("libc.so.6").or_else(|_| Library::new("libc.dylib")) }
                #[cfg(windows)]
                { Library::new("msvcrt.dll").or_else(|_| Library::new("ucrtbase.dll")) }
            }
            "m" => {
                #[cfg(unix)]
                { Library::new("libm.so.6").or_else(|_| Library::new("libm.dylib")) }
                #[cfg(windows)]
                { Library::new("msvcrt.dll") }
            }
            other => {
                // 尝试当作短名加载(平台相关)
                #[cfg(unix)]
                { Library::new(format!("lib{}.so", other))
                    .or_else(|_| Library::new(format!("lib{}.so.6", other)))
                    .or_else(|_| Library::new(format!("lib{}.dylib", other))) }
                #[cfg(windows)]
                { Library::new(format!("{}.dll", other)) }
            }
        }.map_err(|e| format!("Failed to load library '{}': {}", key, e))
    }
}

fn call_extern_function(
    name: &str,
    lib_key: &str,
    signature: &FnSignature,
    args: &[Value],
    ctx: &mut InterpContext,
) -> Result<Value, String> {
    if args.len() != signature.params.len() {
        return Err(format!("Function {} expects {} args, got {}", name, signature.params.len(), args.len()));
    }

    let lib = ctx.libs.get(lib_key)
        .ok_or_else(|| format!("Library '{}' not loaded", lib_key))?;

    // 根据参数和返回类型分发调用
    let ret_type = signature.return_type.clone().unwrap_or(TypeAnnotation::Unit);

    unsafe {
        match (signature.params.len(), &ret_type) {
            // fn() -> T  无参数函数(常见于 C++ 导出)
            (0, TypeAnnotation::I32) => {
                let sym: Symbol<extern "C" fn() -> i32> = lib.get(name.as_bytes())
                    .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                Ok(Value::Int(sym() as i64))
            }
            (0, TypeAnnotation::I64) => {
                let sym: Symbol<extern "C" fn() -> i64> = lib.get(name.as_bytes())
                    .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                Ok(Value::Int(sym()))
            }
            (0, TypeAnnotation::F64) => {
                let sym: Symbol<extern "C" fn() -> f64> = lib.get(name.as_bytes())
                    .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                Ok(Value::Float(sym()))
            }
            (0, TypeAnnotation::Str) => {
                // const char* fn()
                let sym: Symbol<extern "C" fn() -> *const c_char> = lib.get(name.as_bytes())
                    .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                let ptr = sym();
                if ptr.is_null() {
                    Ok(Value::Str(String::new()))
                } else {
                    Ok(Value::Str(CStr::from_ptr(ptr).to_string_lossy().to_string()))
                }
            }
            (0, TypeAnnotation::Bool) => {
                let sym: Symbol<extern "C" fn() -> bool> = lib.get(name.as_bytes())
                    .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                Ok(Value::Bool(sym()))
            }
            // fn(T) -> T  常见: abs(i32) -> i32, sqrt(f64) -> f64
            (1, TypeAnnotation::I32) => {
                let sym: Symbol<extern "C" fn(i32) -> i32> = lib.get(name.as_bytes())
                    .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                let arg = args[0].as_int()? as i32;
                Ok(Value::Int(sym(arg) as i64))
            }
            (1, TypeAnnotation::I64) => {
                let sym: Symbol<extern "C" fn(i64) -> i64> = lib.get(name.as_bytes())
                    .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                let arg = args[0].as_int()?;
                Ok(Value::Int(sym(arg)))
            }
            (1, TypeAnnotation::F64) => {
                let sym: Symbol<extern "C" fn(f64) -> f64> = lib.get(name.as_bytes())
                    .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                let arg = args[0].as_float()?;
                Ok(Value::Float(sym(arg)))
            }
            (1, TypeAnnotation::F32) => {
                let sym: Symbol<extern "C" fn(f32) -> f32> = lib.get(name.as_bytes())
                    .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                let arg = args[0].as_float()? as f32;
                Ok(Value::Float(sym(arg) as f64))
            }
            (1, TypeAnnotation::Str) => {
                // const char* fn(T)
                // 根据参数类型分发
                match &signature.params[0].1 {
                    TypeAnnotation::Str => {
                        // const char* fn(const char*)
                        let sym: Symbol<extern "C" fn(*const c_char) -> *const c_char> = lib.get(name.as_bytes())
                            .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                        let s = CString::new(args[0].as_string()?)
                            .map_err(|e| format!("Invalid string argument: {}", e))?;
                        let ptr = sym(s.as_ptr());
                        if ptr.is_null() { Ok(Value::Str(String::new())) }
                        else { Ok(Value::Str(CStr::from_ptr(ptr).to_string_lossy().to_string())) }
                    }
                    TypeAnnotation::I32 | TypeAnnotation::U32 => {
                        let sym: Symbol<extern "C" fn(i32) -> *const c_char> = lib.get(name.as_bytes())
                            .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                        let a = args[0].as_int()? as i32;
                        let ptr = sym(a);
                        if ptr.is_null() { Ok(Value::Str(String::new())) }
                        else { Ok(Value::Str(CStr::from_ptr(ptr).to_string_lossy().to_string())) }
                    }
                    TypeAnnotation::I64 | TypeAnnotation::U64 => {
                        let sym: Symbol<extern "C" fn(i64) -> *const c_char> = lib.get(name.as_bytes())
                            .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                        let a = args[0].as_int()?;
                        let ptr = sym(a);
                        if ptr.is_null() { Ok(Value::Str(String::new())) }
                        else { Ok(Value::Str(CStr::from_ptr(ptr).to_string_lossy().to_string())) }
                    }
                    TypeAnnotation::F64 | TypeAnnotation::F32 => {
                        let sym: Symbol<extern "C" fn(f64) -> *const c_char> = lib.get(name.as_bytes())
                            .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                        let a = args[0].as_float()?;
                        let ptr = sym(a);
                        if ptr.is_null() { Ok(Value::Str(String::new())) }
                        else { Ok(Value::Str(CStr::from_ptr(ptr).to_string_lossy().to_string())) }
                    }
                    _ => Err(format!("Unsupported parameter type for string-returning extern {}", name)),
                }
            }
            (1, TypeAnnotation::Bool) => {
                let sym: Symbol<extern "C" fn(i32) -> bool> = lib.get(name.as_bytes())
                    .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                let arg = args[0].as_int()? as i32;
                Ok(Value::Bool(sym(arg)))
            }
            (2, TypeAnnotation::I32) => {
                let sym: Symbol<extern "C" fn(i32, i32) -> i32> = lib.get(name.as_bytes())
                    .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                let a = args[0].as_int()? as i32;
                let b = args[1].as_int()? as i32;
                Ok(Value::Int(sym(a, b) as i64))
            }
            (2, TypeAnnotation::I64) => {
                let sym: Symbol<extern "C" fn(i64, i64) -> i64> = lib.get(name.as_bytes())
                    .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                let a = args[0].as_int()?;
                let b = args[1].as_int()?;
                Ok(Value::Int(sym(a, b)))
            }
            (2, TypeAnnotation::F64) => {
                let sym: Symbol<extern "C" fn(f64, f64) -> f64> = lib.get(name.as_bytes())
                    .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                let a = args[0].as_float()?;
                let b = args[1].as_float()?;
                Ok(Value::Float(sym(a, b)))
            }
            (3, TypeAnnotation::I32) => {
                let sym: Symbol<extern "C" fn(i32, i32, i32) -> i32> = lib.get(name.as_bytes())
                    .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                let a = args[0].as_int()? as i32;
                let b = args[1].as_int()? as i32;
                let c = args[2].as_int()? as i32;
                Ok(Value::Int(sym(a, b, c) as i64))
            }
            (3, TypeAnnotation::F64) => {
                // 根据参数类型分发:int 参数 vs float 参数
                let all_int = signature.params.iter().all(|(_, t)|
                    matches!(t, TypeAnnotation::I32 | TypeAnnotation::I64 | TypeAnnotation::U32 | TypeAnnotation::U64)
                );
                if all_int {
                    // fn(i32, i32, i32) -> f64  例如 cpp_average
                    let sym: Symbol<extern "C" fn(i32, i32, i32) -> f64> = lib.get(name.as_bytes())
                        .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                    let a = args[0].as_int()? as i32;
                    let b = args[1].as_int()? as i32;
                    let c = args[2].as_int()? as i32;
                    Ok(Value::Float(sym(a, b, c)))
                } else {
                    // fn(f64, f64, f64) -> f64
                    let sym: Symbol<extern "C" fn(f64, f64, f64) -> f64> = lib.get(name.as_bytes())
                        .map_err(|e| format!("Symbol '{}' not found: {}", name, e))?;
                    let a = args[0].as_float()?;
                    let b = args[1].as_float()?;
                    let c = args[2].as_float()?;
                    Ok(Value::Float(sym(a, b, c)))
                }
            }
            (_, TypeAnnotation::Unit) | (_, TypeAnnotation::Void) => {
                Err(format!("void-returning extern functions not yet supported for {}", name))
            }
            _ => Err(format!(
                "Unsupported extern signature for {}: {} args -> {:?}",
                name, signature.params.len(), ret_type
            )),
        }
    }
}

fn register_builtins(env: &mut Environment) {
    // print(...) - 打印值,不换行
    env.set("print".to_string(), Value::NativeFunction {
        name: "print".to_string(),
        arity: None,
        func: builtin_print,
    });

    // println(...) - 打印值并换行
    env.set("println".to_string(), Value::NativeFunction {
        name: "println".to_string(),
        arity: None,
        func: builtin_println,
    });

    // len(x) - 返回字符串或列表的长度
    env.set("len".to_string(), Value::NativeFunction {
        name: "len".to_string(),
        arity: Some(1),
        func: builtin_len,
    });

    // stream(list) - 从列表创建流
    env.set("stream".to_string(), Value::NativeFunction {
        name: "stream".to_string(),
        arity: Some(1),
        func: builtin_stream,
    });

    // map(stream, fn) - 对每个元素应用函数
    env.set("map".to_string(), Value::NativeFunction {
        name: "map".to_string(),
        arity: Some(2),
        func: builtin_map,
    });

    // filter(stream, fn) - 过滤元素
    env.set("filter".to_string(), Value::NativeFunction {
        name: "filter".to_string(),
        arity: Some(2),
        func: builtin_filter,
    });

    // for_each(stream, fn) - 遍历消费
    env.set("for_each".to_string(), Value::NativeFunction {
        name: "for_each".to_string(),
        arity: Some(2),
        func: builtin_for_each,
    });

    // collect(stream) - 流转回列表
    env.set("collect".to_string(), Value::NativeFunction {
        name: "collect".to_string(),
        arity: Some(1),
        func: builtin_collect,
    });
    env.set("sleep".to_string(), Value::NativeFunction {
        name: "sleep".to_string(),
        arity: Some(1),
        func: builtin_sleep,
    });

    // ===== 标准库: 数学函数 =====
    env.set("abs".to_string(), Value::NativeFunction { name: "abs".to_string(), arity: Some(1), func: builtin_abs });
    env.set("min".to_string(), Value::NativeFunction { name: "min".to_string(), arity: Some(2), func: builtin_min });
    env.set("max".to_string(), Value::NativeFunction { name: "max".to_string(), arity: Some(2), func: builtin_max });
    env.set("sqrt".to_string(), Value::NativeFunction { name: "sqrt".to_string(), arity: Some(1), func: builtin_sqrt });
    env.set("pow".to_string(), Value::NativeFunction { name: "pow".to_string(), arity: Some(2), func: builtin_pow });
    env.set("floor".to_string(), Value::NativeFunction { name: "floor".to_string(), arity: Some(1), func: builtin_floor });
    env.set("ceil".to_string(), Value::NativeFunction { name: "ceil".to_string(), arity: Some(1), func: builtin_ceil });
    env.set("round".to_string(), Value::NativeFunction { name: "round".to_string(), arity: Some(1), func: builtin_round });

    // ===== 标准库: 字符串函数 =====
    env.set("to_string".to_string(), Value::NativeFunction { name: "to_string".to_string(), arity: Some(1), func: builtin_to_string });
    env.set("str_concat".to_string(), Value::NativeFunction { name: "str_concat".to_string(), arity: Some(2), func: builtin_str_concat });
    env.set("str_len".to_string(), Value::NativeFunction { name: "str_len".to_string(), arity: Some(1), func: builtin_str_len });
    env.set("str_upper".to_string(), Value::NativeFunction { name: "str_upper".to_string(), arity: Some(1), func: builtin_str_upper });
    env.set("str_lower".to_string(), Value::NativeFunction { name: "str_lower".to_string(), arity: Some(1), func: builtin_str_lower });
    env.set("str_contains".to_string(), Value::NativeFunction { name: "str_contains".to_string(), arity: Some(2), func: builtin_str_contains });
    env.set("str_starts_with".to_string(), Value::NativeFunction { name: "str_starts_with".to_string(), arity: Some(2), func: builtin_str_starts_with });
    env.set("str_ends_with".to_string(), Value::NativeFunction { name: "str_ends_with".to_string(), arity: Some(2), func: builtin_str_ends_with });
    env.set("str_substring".to_string(), Value::NativeFunction { name: "str_substring".to_string(), arity: Some(3), func: builtin_str_substring });
    env.set("str_split".to_string(), Value::NativeFunction { name: "str_split".to_string(), arity: Some(2), func: builtin_str_split });
    env.set("str_trim".to_string(), Value::NativeFunction { name: "str_trim".to_string(), arity: Some(1), func: builtin_str_trim });

    // ===== 标准库: 文件 IO =====
    env.set("file_read".to_string(), Value::NativeFunction { name: "file_read".to_string(), arity: Some(1), func: builtin_file_read });
    env.set("file_write".to_string(), Value::NativeFunction { name: "file_write".to_string(), arity: Some(2), func: builtin_file_write });
    env.set("file_append".to_string(), Value::NativeFunction { name: "file_append".to_string(), arity: Some(2), func: builtin_file_append });
    env.set("file_exists".to_string(), Value::NativeFunction { name: "file_exists".to_string(), arity: Some(1), func: builtin_file_exists });

    // ===== 标准库: 时间函数 =====
    env.set("time_now".to_string(), Value::NativeFunction { name: "time_now".to_string(), arity: Some(0), func: builtin_time_now });
    env.set("time_now_ms".to_string(), Value::NativeFunction { name: "time_now_ms".to_string(), arity: Some(0), func: builtin_time_now_ms });

    // ===== 标准库: 类型转换 =====
    env.set("int".to_string(), Value::NativeFunction { name: "int".to_string(), arity: Some(1), func: builtin_int });
    env.set("float".to_string(), Value::NativeFunction { name: "float".to_string(), arity: Some(1), func: builtin_float });
    env.set("bool".to_string(), Value::NativeFunction { name: "bool".to_string(), arity: Some(1), func: builtin_bool });
    env.set("str".to_string(), Value::NativeFunction { name: "str".to_string(), arity: Some(1), func: builtin_str });

    // ===== 标准库: 列表函数 =====
    env.set("list_push".to_string(), Value::NativeFunction { name: "list_push".to_string(), arity: Some(2), func: builtin_list_push });
    env.set("list_pop".to_string(), Value::NativeFunction { name: "list_pop".to_string(), arity: Some(1), func: builtin_list_pop });
    env.set("list_get".to_string(), Value::NativeFunction { name: "list_get".to_string(), arity: Some(2), func: builtin_list_get });
    env.set("list_set".to_string(), Value::NativeFunction { name: "list_set".to_string(), arity: Some(3), func: builtin_list_set });
    env.set("list_contains".to_string(), Value::NativeFunction { name: "list_contains".to_string(), arity: Some(2), func: builtin_list_contains });
    env.set("list_reverse".to_string(), Value::NativeFunction { name: "list_reverse".to_string(), arity: Some(1), func: builtin_list_reverse });
    env.set("list_sort".to_string(), Value::NativeFunction { name: "list_sort".to_string(), arity: Some(1), func: builtin_list_sort });
}

/// sleep(ms) —— 异步原语:阻塞当前线程 ms 毫秒
/// v0.1 中作为 async 函数的占位原语,用于演示异步编程模型
fn builtin_sleep(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(format!("sleep expects 1 arg, got {}", args.len()));
    }
    let ms = args[0].as_int()?;
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    Ok(Value::None)
}

// ===== 标准库: 数学函数实现 =====

fn builtin_abs(args: &[Value]) -> Result<Value, String> {
    let n = args.get(0).ok_or("abs expects 1 arg")?;
    match n {
        Value::Int(x) => Ok(Value::Int(x.abs())),
        Value::Float(x) => Ok(Value::Float(x.abs())),
        _ => Err("abs expects int or float".to_string()),
    }
}

fn builtin_min(args: &[Value]) -> Result<Value, String> {
    let a = args.get(0).ok_or("min expects 2 args")?;
    let b = args.get(1).ok_or("min expects 2 args")?;
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(std::cmp::min(*x, *y))),
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x.min(*y))),
        _ => Err("min expects same-type numeric args".to_string()),
    }
}

fn builtin_max(args: &[Value]) -> Result<Value, String> {
    let a = args.get(0).ok_or("max expects 2 args")?;
    let b = args.get(1).ok_or("max expects 2 args")?;
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(Value::Int(std::cmp::max(*x, *y))),
        (Value::Float(x), Value::Float(y)) => Ok(Value::Float(x.max(*y))),
        _ => Err("max expects same-type numeric args".to_string()),
    }
}

fn builtin_sqrt(args: &[Value]) -> Result<Value, String> {
    let x = args.get(0).ok_or("sqrt expects 1 arg")?;
    let v = match x {
        Value::Int(n) => *n as f64,
        Value::Float(f) => *f,
        _ => return Err("sqrt expects numeric arg".to_string()),
    };
    Ok(Value::Float(v.sqrt()))
}

fn builtin_pow(args: &[Value]) -> Result<Value, String> {
    let base = args.get(0).ok_or("pow expects 2 args")?;
    let exp = args.get(1).ok_or("pow expects 2 args")?;
    let b = match base { Value::Int(n) => *n as f64, Value::Float(f) => *f, _ => return Err("pow expects numeric base".to_string()) };
    let e = match exp { Value::Int(n) => *n as f64, Value::Float(f) => *f, _ => return Err("pow expects numeric exp".to_string()) };
    Ok(Value::Float(b.powf(e)))
}

fn builtin_floor(args: &[Value]) -> Result<Value, String> {
    let x = args.get(0).ok_or("floor expects 1 arg")?;
    let v = match x { Value::Int(n) => *n as f64, Value::Float(f) => *f, _ => return Err("floor expects numeric arg".to_string()) };
    Ok(Value::Float(v.floor()))
}

fn builtin_ceil(args: &[Value]) -> Result<Value, String> {
    let x = args.get(0).ok_or("ceil expects 1 arg")?;
    let v = match x { Value::Int(n) => *n as f64, Value::Float(f) => *f, _ => return Err("ceil expects numeric arg".to_string()) };
    Ok(Value::Float(v.ceil()))
}

fn builtin_round(args: &[Value]) -> Result<Value, String> {
    let x = args.get(0).ok_or("round expects 1 arg")?;
    let v = match x { Value::Int(n) => *n as f64, Value::Float(f) => *f, _ => return Err("round expects numeric arg".to_string()) };
    Ok(Value::Int(v.round() as i64))
}

// ===== 标准库: 字符串函数实现 =====

fn builtin_to_string(args: &[Value]) -> Result<Value, String> {
    let v = args.get(0).ok_or("to_string expects 1 arg")?;
    Ok(Value::Str(value_to_string(v)))
}

fn builtin_str_concat(args: &[Value]) -> Result<Value, String> {
    let a = args.get(0).ok_or("str_concat expects 2 args")?;
    let b = args.get(1).ok_or("str_concat expects 2 args")?;
    Ok(Value::Str(format!("{}{}", value_to_string(a), value_to_string(b))))
}

fn builtin_str_len(args: &[Value]) -> Result<Value, String> {
    let s = match args.get(0) {
        Some(Value::Str(s)) => s,
        _ => return Err("str_len expects a string".to_string()),
    };
    Ok(Value::Int(s.chars().count() as i64))
}

fn builtin_str_upper(args: &[Value]) -> Result<Value, String> {
    let s = match args.get(0) {
        Some(Value::Str(s)) => s,
        _ => return Err("str_upper expects a string".to_string()),
    };
    Ok(Value::Str(s.to_uppercase()))
}

fn builtin_str_lower(args: &[Value]) -> Result<Value, String> {
    let s = match args.get(0) {
        Some(Value::Str(s)) => s,
        _ => return Err("str_lower expects a string".to_string()),
    };
    Ok(Value::Str(s.to_lowercase()))
}

fn builtin_str_contains(args: &[Value]) -> Result<Value, String> {
    let s = match args.get(0) { Some(Value::Str(s)) => s, _ => return Err("str_contains expects string arg 1".to_string()) };
    let sub = match args.get(1) { Some(Value::Str(s)) => s, _ => return Err("str_contains expects string arg 2".to_string()) };
    Ok(Value::Bool(s.contains(sub.as_str())))
}

fn builtin_str_starts_with(args: &[Value]) -> Result<Value, String> {
    let s = match args.get(0) { Some(Value::Str(s)) => s, _ => return Err("str_starts_with expects string arg 1".to_string()) };
    let prefix = match args.get(1) { Some(Value::Str(s)) => s, _ => return Err("str_starts_with expects string arg 2".to_string()) };
    Ok(Value::Bool(s.starts_with(prefix.as_str())))
}

fn builtin_str_ends_with(args: &[Value]) -> Result<Value, String> {
    let s = match args.get(0) { Some(Value::Str(s)) => s, _ => return Err("str_ends_with expects string arg 1".to_string()) };
    let suffix = match args.get(1) { Some(Value::Str(s)) => s, _ => return Err("str_ends_with expects string arg 2".to_string()) };
    Ok(Value::Bool(s.ends_with(suffix.as_str())))
}

fn builtin_str_substring(args: &[Value]) -> Result<Value, String> {
    let s = match args.get(0) { Some(Value::Str(s)) => s, _ => return Err("str_substring expects string arg 1".to_string()) };
    let start = args.get(1).ok_or("str_substring expects start index")?.as_int()? as usize;
    let end = args.get(2).ok_or("str_substring expects end index")?.as_int()? as usize;
    let chars: Vec<char> = s.chars().collect();
    let start = start.min(chars.len());
    let end = end.min(chars.len());
    if start > end {
        return Err("str_substring: start > end".to_string());
    }
    Ok(Value::Str(chars[start..end].iter().collect()))
}

fn builtin_str_split(args: &[Value]) -> Result<Value, String> {
    let s = match args.get(0) { Some(Value::Str(s)) => s, _ => return Err("str_split expects string arg 1".to_string()) };
    let sep = match args.get(1) { Some(Value::Str(s)) => s, _ => return Err("str_split expects string arg 2".to_string()) };
    let parts: Vec<Value> = s.split(sep.as_str()).map(|p| Value::Str(p.to_string())).collect();
    Ok(Value::List(parts))
}

fn builtin_str_trim(args: &[Value]) -> Result<Value, String> {
    let s = match args.get(0) { Some(Value::Str(s)) => s, _ => return Err("str_trim expects a string".to_string()) };
    Ok(Value::Str(s.trim().to_string()))
}

// ===== 标准库: 文件 IO 实现 =====

fn builtin_file_read(args: &[Value]) -> Result<Value, String> {
    let path = match args.get(0) { Some(Value::Str(s)) => s, _ => return Err("file_read expects a string path".to_string()) };
    let content = std::fs::read_to_string(path).map_err(|e| format!("file_read: {}", e))?;
    Ok(Value::Str(content))
}

fn builtin_file_write(args: &[Value]) -> Result<Value, String> {
    let path = match args.get(0) { Some(Value::Str(s)) => s, _ => return Err("file_write expects a string path".to_string()) };
    let content = match args.get(1) { Some(Value::Str(s)) => s, _ => return Err("file_write expects string content".to_string()) };
    std::fs::write(path, content).map_err(|e| format!("file_write: {}", e))?;
    Ok(Value::None)
}

fn builtin_file_append(args: &[Value]) -> Result<Value, String> {
    use std::io::Write;
    let path = match args.get(0) { Some(Value::Str(s)) => s, _ => return Err("file_append expects a string path".to_string()) };
    let content = match args.get(1) { Some(Value::Str(s)) => s, _ => return Err("file_append expects string content".to_string()) };
    let mut f = std::fs::OpenOptions::new().append(true).create(true).open(path).map_err(|e| format!("file_append: {}", e))?;
    f.write_all(content.as_bytes()).map_err(|e| format!("file_append: {}", e))?;
    Ok(Value::None)
}

fn builtin_file_exists(args: &[Value]) -> Result<Value, String> {
    let path = match args.get(0) { Some(Value::Str(s)) => s, _ => return Err("file_exists expects a string path".to_string()) };
    Ok(Value::Bool(std::path::Path::new(path).exists()))
}

// ===== 标准库: 时间函数实现 =====

fn builtin_time_now(_args: &[Value]) -> Result<Value, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("time_now: {}", e))?;
    Ok(Value::Int(now.as_secs() as i64))
}

fn builtin_time_now_ms(_args: &[Value]) -> Result<Value, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("time_now_ms: {}", e))?;
    Ok(Value::Int(now.as_millis() as i64))
}

// ===== 标准库: 类型转换实现 =====

fn builtin_int(args: &[Value]) -> Result<Value, String> {
    let v = args.get(0).ok_or("int expects 1 arg")?;
    match v {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => Ok(Value::Int(*f as i64)),
        Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
        Value::Str(s) => s.parse::<i64>().map(Value::Int).map_err(|e| format!("int: {}", e)),
        _ => Err("int: cannot convert".to_string()),
    }
}

fn builtin_float(args: &[Value]) -> Result<Value, String> {
    let v = args.get(0).ok_or("float expects 1 arg")?;
    match v {
        Value::Int(n) => Ok(Value::Float(*n as f64)),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::Str(s) => s.parse::<f64>().map(Value::Float).map_err(|e| format!("float: {}", e)),
        _ => Err("float: cannot convert".to_string()),
    }
}

fn builtin_bool(args: &[Value]) -> Result<Value, String> {
    let v = args.get(0).ok_or("bool expects 1 arg")?;
    match v {
        Value::Int(n) => Ok(Value::Bool(*n != 0)),
        Value::Float(f) => Ok(Value::Bool(*f != 0.0)),
        Value::Bool(b) => Ok(Value::Bool(*b)),
        Value::Str(s) => Ok(Value::Bool(!s.is_empty())),
        Value::None => Ok(Value::Bool(false)),
        _ => Err("bool: cannot convert".to_string()),
    }
}

fn builtin_str(args: &[Value]) -> Result<Value, String> {
    let v = args.get(0).ok_or("str expects 1 arg")?;
    Ok(Value::Str(value_to_string(v)))
}

// ===== 标准库: 列表函数实现 =====

fn builtin_list_push(args: &[Value]) -> Result<Value, String> {
    let list = match args.get(0) {
        Some(Value::List(items)) => items.clone(),
        _ => return Err("list_push expects a list".to_string()),
    };
    let item = args.get(1).ok_or("list_push expects an item")?.clone();
    let mut new_list = list;
    new_list.push(item);
    Ok(Value::List(new_list))
}

fn builtin_list_pop(args: &[Value]) -> Result<Value, String> {
    let list = match args.get(0) {
        Some(Value::List(items)) => items.clone(),
        _ => return Err("list_pop expects a list".to_string()),
    };
    if list.is_empty() {
        return Err("list_pop: list is empty".to_string());
    }
    let mut new_list = list;
    let last = new_list.pop().unwrap();
    // 返回弹出的元素(列表以引用方式传递时原地修改;v0.1 返回新列表与弹出的元素组成的元组简化为返回弹出元素)
    let _ = new_list;
    Ok(last)
}

fn builtin_list_get(args: &[Value]) -> Result<Value, String> {
    let list = match args.get(0) {
        Some(Value::List(items)) => items,
        _ => return Err("list_get expects a list".to_string()),
    };
    let idx = args.get(1).ok_or("list_get expects an index")?.as_int()? as usize;
    list.get(idx).cloned().ok_or_else(|| format!("list_get: index {} out of bounds", idx))
}

fn builtin_list_set(args: &[Value]) -> Result<Value, String> {
    let list = match args.get(0) {
        Some(Value::List(items)) => items.clone(),
        _ => return Err("list_set expects a list".to_string()),
    };
    let idx = args.get(1).ok_or("list_set expects an index")?.as_int()? as usize;
    let val = args.get(2).ok_or("list_set expects a value")?.clone();
    let mut new_list = list;
    if idx >= new_list.len() {
        return Err(format!("list_set: index {} out of bounds", idx));
    }
    new_list[idx] = val;
    Ok(Value::List(new_list))
}

fn builtin_list_contains(args: &[Value]) -> Result<Value, String> {
    let list = match args.get(0) {
        Some(Value::List(items)) => items,
        _ => return Err("list_contains expects a list".to_string()),
    };
    let target = args.get(1).ok_or("list_contains expects a target")?;
    Ok(Value::Bool(list.iter().any(|v| v == target)))
}

fn builtin_list_reverse(args: &[Value]) -> Result<Value, String> {
    let list = match args.get(0) {
        Some(Value::List(items)) => items.clone(),
        _ => return Err("list_reverse expects a list".to_string()),
    };
    let mut new_list = list;
    new_list.reverse();
    Ok(Value::List(new_list))
}

fn builtin_list_sort(args: &[Value]) -> Result<Value, String> {
    let list = match args.get(0) {
        Some(Value::List(items)) => items.clone(),
        _ => return Err("list_sort expects a list".to_string()),
    };
    let mut new_list = list;
    new_list.sort_by(|a, b| {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => x.cmp(y),
            (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
            (Value::Str(x), Value::Str(y)) => x.cmp(y),
            (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
            _ => std::cmp::Ordering::Equal,
        }
    });
    Ok(Value::List(new_list))
}

fn builtin_print(args: &[Value]) -> Result<Value, String> {
    print_args(args);
    Ok(Value::None)
}

fn builtin_println(args: &[Value]) -> Result<Value, String> {
    print_args(args);
    println!();
    Ok(Value::None)
}

/// 统一打印逻辑:支持 `{}` 格式化占位符
fn print_args(args: &[Value]) {
    if args.is_empty() {
        return;
    }
    // 若第一个参数是字符串且包含 `{}`,则按格式化字符串处理
    if let Value::Str(fmt) = &args[0] {
        if fmt.contains("{}") && args.len() > 1 {
            let mut arg_idx = 1;
            let chars: Vec<char> = fmt.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '}' {
                    if arg_idx < args.len() {
                        print_value(&args[arg_idx]);
                        arg_idx += 1;
                    } else {
                        print!("{{}}");
                    }
                    i += 2;
                } else {
                    print!("{}", chars[i]);
                    i += 1;
                }
            }
            return;
        }
    }
    // 回退: 空格分隔
    for (i, arg) in args.iter().enumerate() {
        if i > 0 { print!(" "); }
        print_value(arg);
    }
}

fn print_value(val: &Value) {
    match val {
        Value::Int(n) => print!("{}", n),
        Value::Float(f) => print!("{}", f),
        Value::Str(s) => print!("{}", s),
        Value::Bool(b) => print!("{}", b),
        Value::None => print!("none"),
        Value::List(items) => {
            print!("[");
            for (i, item) in items.iter().enumerate() {
                if i > 0 { print!(", "); }
                print_value(item);
            }
            print!("]");
        }
        Value::Function { name, .. } => print!("<fn {}>", name),
        Value::NativeFunction { name, .. } => print!("<native fn {}>", name),
        Value::ExternFunction { name, .. } => print!("<extern fn {}>", name),
        Value::PythonFunction { name, .. } => print!("<python fn {}>", name),
        Value::WasmFunction { name, .. } => print!("<wasm fn {}>", name),
        Value::JavaFunction { name, .. } => print!("<java fn {}>", name),
        Value::HtmlFunction { name, .. } => print!("<html fn {}>", name),
        Value::ProcessFunction { name, language, .. } => print!("<{} fn {}>", language, name),
        Value::Stream(items) => {
            print!("stream[");
            for (i, item) in items.iter().enumerate() {
                if i > 0 { print!(", "); }
                print_value(item);
            }
            print!("]");
        }
        Value::StructInstance { type_name, fields } => {
            print!("{} {{ ", type_name);
            for (i, (k, v)) in fields.iter().enumerate() {
                if i > 0 { print!(", "); }
                print!("{}: ", k);
                print_value(v);
            }
            print!(" }}");
        }
        Value::EnumValue { type_name, variant, payload } => {
            print!("{}::{}", type_name, variant);
            if !payload.is_empty() {
                print!("(");
                for (i, v) in payload.iter().enumerate() {
                    if i > 0 { print!(", "); }
                    print_value(v);
                }
                print!(")");
            }
        }
    }
}

fn builtin_len(args: &[Value]) -> Result<Value, String> {
    match &args[0] {
        Value::Str(s) => Ok(Value::Int(s.len() as i64)),
        Value::List(items) => Ok(Value::Int(items.len() as i64)),
        v => Err(format!("len() not supported for type {}", v.type_name())),
    }
}

fn builtin_stream(args: &[Value]) -> Result<Value, String> {
    match &args[0] {
        Value::List(items) => Ok(Value::Stream(items.clone())),
        v => Err(format!("stream() expects a list, got {}", v.type_name())),
    }
}

fn builtin_map(args: &[Value]) -> Result<Value, String> {
    match &args[0] {
        Value::Stream(items) => {
            let func = &args[1];
            let mut result = Vec::new();
            for item in items {
                result.push(call_function(func, &[item.clone()], &mut InterpContext::new())?);
            }
            Ok(Value::Stream(result))
        }
        v => Err(format!("map() expects a stream, got {}", v.type_name())),
    }
}

fn builtin_filter(args: &[Value]) -> Result<Value, String> {
    match &args[0] {
        Value::Stream(items) => {
            let func = &args[1];
            let mut result = Vec::new();
            for item in items {
                let cond = call_function(func, &[item.clone()], &mut InterpContext::new())?;
                if cond.is_truthy() {
                    result.push(item.clone());
                }
            }
            Ok(Value::Stream(result))
        }
        v => Err(format!("filter() expects a stream, got {}", v.type_name())),
    }
}

fn builtin_for_each(args: &[Value]) -> Result<Value, String> {
    match &args[0] {
        Value::Stream(items) => {
            let func = &args[1];
            for item in items {
                call_function(func, &[item.clone()], &mut InterpContext::new())?;
            }
            Ok(Value::None)
        }
        v => Err(format!("for_each() expects a stream, got {}", v.type_name())),
    }
}

fn builtin_collect(args: &[Value]) -> Result<Value, String> {
    match &args[0] {
        Value::Stream(items) => Ok(Value::List(items.clone())),
        v => Err(format!("collect() expects a stream, got {}", v.type_name())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use linkc_lexer::lex;
    use linkc_parser::Parser;

    fn run(source: &str) -> Result<Value, String> {
        let tokens = lex(source);
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program()?;
        let mut env = Environment::new();
        let mut ctx = InterpContext::new();
        eval_program(&program, &mut env, &mut ctx)
    }

    #[test]
    fn test_eval_integer() {
        assert_eq!(run("42").unwrap(), Value::Int(42));
    }

    #[test]
    fn test_eval_float() {
        let v = run("3.14").unwrap();
        if let Value::Float(f) = v {
            assert!((f - 3.14).abs() < 0.001);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_eval_bool() {
        assert_eq!(run("true").unwrap(), Value::Bool(true));
        assert_eq!(run("false").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_eval_string() {
        assert_eq!(run("\"hello\"").unwrap(), Value::Str("hello".to_string()));
    }

    #[test]
    fn test_eval_none() {
        assert_eq!(run("none").unwrap(), Value::None);
    }

    #[test]
    fn test_eval_binary_add() {
        assert_eq!(run("1 + 2").unwrap(), Value::Int(3));
    }

    #[test]
    fn test_eval_binary_complex() {
        assert_eq!(run("2 + 3 * 4").unwrap(), Value::Int(14));
    }

    #[test]
    fn test_eval_comparison() {
        assert_eq!(run("1 < 2").unwrap(), Value::Bool(true));
        assert_eq!(run("3 > 2").unwrap(), Value::Bool(true));
        assert_eq!(run("1 == 2").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_eval_logical() {
        assert_eq!(run("true && false").unwrap(), Value::Bool(false));
        assert_eq!(run("true || false").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_unary_neg() {
        assert_eq!(run("-5").unwrap(), Value::Int(-5));
    }

    #[test]
    fn test_eval_let_and_ident() {
        assert_eq!(run("let x = 42; x").unwrap(), Value::Int(42));
    }

    #[test]
    fn test_eval_if_true_branch() {
        assert_eq!(run("if true { 1 } else { 2 }").unwrap(), Value::Int(1));
    }

    #[test]
    fn test_eval_if_false_branch() {
        assert_eq!(run("if false { 1 } else { 2 }").unwrap(), Value::Int(2));
    }

    #[test]
    fn test_eval_function_call() {
        assert_eq!(
            run("fn add(a: i32, b: i32) -> i32 { return a + b; } add(2, 3)").unwrap(),
            Value::Int(5)
        );
    }

    #[test]
    fn test_eval_recursive_fib() {
        assert_eq!(
            run("fn fib(n: i32) -> i32 { if n <= 1 { return n; } return fib(n - 1) + fib(n - 2); } fib(10)").unwrap(),
            Value::Int(55)
        );
    }

    #[test]
    fn test_eval_list_literal() {
        assert_eq!(
            run("[1, 2, 3]").unwrap(),
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );
    }

    #[test]
    fn test_eval_empty_list() {
        assert_eq!(run("[]").unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_eval_list_index() {
        assert_eq!(run("let a = [10, 20, 30]; a[1]").unwrap(), Value::Int(20));
        assert_eq!(run("let a = [10, 20, 30]; a[0]").unwrap(), Value::Int(10));
        assert_eq!(run("let a = [10, 20, 30]; a[2]").unwrap(), Value::Int(30));
    }

    #[test]
    fn test_eval_list_index_out_of_bounds() {
        assert!(run("let a = [1, 2, 3]; a[5]").is_err());
        assert!(run("let a = [1, 2, 3]; a[-1]").is_err());
    }

    #[test]
    fn test_eval_string_index() {
        assert_eq!(run("\"hello\"[0]").unwrap(), Value::Str("h".to_string()));
        assert_eq!(run("\"hello\"[4]").unwrap(), Value::Str("o".to_string()));
    }

    #[test]
    fn test_eval_builtin_len_string() {
        assert_eq!(run("len(\"hello\")").unwrap(), Value::Int(5));
        assert_eq!(run("len(\"\")").unwrap(), Value::Int(0));
    }

    #[test]
    fn test_eval_builtin_len_list() {
        assert_eq!(run("len([1, 2, 3])").unwrap(), Value::Int(3));
        assert_eq!(run("len([])").unwrap(), Value::Int(0));
    }

    #[test]
    fn test_eval_while_loop() {
        assert_eq!(
            run("let i = 0; while i < 5 { i = i + 1; } i").unwrap(),
            Value::Int(5)
        );
    }

    #[test]
    fn test_eval_for_loop() {
        assert_eq!(
            run("let sum = 0; for i in 0..10 { sum = sum + i; } sum").unwrap(),
            Value::Int(45)
        );
    }

    #[test]
    fn test_eval_nested_list() {
        assert_eq!(
            run("let a = [[1, 2], [3, 4]]; a[1][0]").unwrap(),
            Value::Int(3)
        );
    }

    #[test]
    fn test_eval_list_truthy() {
        assert_eq!(run("if [1] { 1 } else { 2 }").unwrap(), Value::Int(1));
        assert_eq!(run("if [] { 1 } else { 2 }").unwrap(), Value::Int(2));
    }

    #[test]
    fn test_eval_break_loop() {
        assert_eq!(
            run("let i = 0; loop { if i >= 5 { break; } i = i + 1; } i").unwrap(),
            Value::Int(5)
        );
    }

    #[test]
    fn test_eval_else_if() {
        assert_eq!(
            run("if false { 1 } else if true { 2 } else { 3 }").unwrap(),
            Value::Int(2)
        );
    }

    #[test]
    fn test_extern_c_abs() {
        let result = run(r#"
            extern "C" {
                fn abs(n: i32) -> i32;
            }
            abs(-42)
        "#);
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    #[test]
    fn test_extern_c_sqrt() {
        let result = run(r#"
            extern "C" {
                fn sqrt(x: f64) -> f64;
            }
            sqrt(16.0)
        "#);
        let v = result.unwrap();
        if let Value::Float(f) = v {
            assert!((f - 4.0).abs() < 0.001);
        } else {
            panic!("Expected float, got {:?}", v);
        }
    }

    #[test]
    fn test_extern_c_in_expression() {
        let result = run(r#"
            extern "C" {
                fn abs(n: i32) -> i32;
            }
            abs(-10) + 5
        "#);
        assert_eq!(result.unwrap(), Value::Int(15));
    }

    #[test]
    fn test_extern_c_multiple_decls() {
        let result = run(r#"
            extern "C" {
                fn abs(n: i32) -> i32;
                fn sqrt(x: f64) -> f64;
            }
            abs(-3) + 7
        "#);
        assert_eq!(result.unwrap(), Value::Int(10));
    }

    #[test]
    fn test_extern_export_block_no_error() {
        // export 块暂时不执行操作,但应该能解析和运行
        let result = run(r#"
            export "C" {
                fn my_func(n: i32) -> i32;
            }
            42
        "#);
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    // ---- C++ FFI 测试 ----

    /// 辅助:C++ DLL 的绝对路径(基于 CARGO_MANIFEST_DIR)
    fn cpp_dll_path() -> String {
        // crates/linkc_interpreter -> ../../examples/cpp_demo.dll
        let manifest = env!("CARGO_MANIFEST_DIR");
        let path = std::path::Path::new(manifest);
        // parent() 两次:crates/linkc_interpreter -> crates -> workspace root
        let workspace_root = path.parent()
            .and_then(|p| p.parent())
            .unwrap_or(path);
        workspace_root.join("examples").join("cpp_demo.dll")
            .to_string_lossy()
            .replace('\\', "/")
    }

    /// 辅助:C++ DLL 是否存在
    fn cpp_dll_exists() -> bool {
        std::path::Path::new(&cpp_dll_path()).exists()
    }

    #[test]
    fn test_extern_cpp_alias_parses() {
        // extern "C++" 应该和 extern "C" 一样能解析
        // 用系统库 "c" 加载,避免依赖自定义 DLL 路径
        let result = run(r#"
            extern "C++" module "c" {
                fn abs(n: i32) -> i32;
            }
            42
        "#);
        assert_eq!(result.unwrap(), Value::Int(42));
    }

    #[test]
    fn test_extern_cpp_short_alias_parses() {
        // extern "cpp" 也应该作为别名被接受
        let result = run(r#"
            extern "cpp" module "c" {
                fn abs(n: i32) -> i32;
            }
            7
        "#);
        assert_eq!(result.unwrap(), Value::Int(7));
    }

    #[test]
    fn test_unsupported_extern_language() {
        // 不支持的语言应该返回错误
        let result = run(r#"
            extern "erlang" {
                fn foo() -> i32;
            }
            1
        "#);
        let err = result.unwrap_err();
        assert!(err.contains("Unsupported extern language"), "got: {}", err);
        assert!(err.contains("erlang"), "got: {}", err);
    }

    #[test]
    fn test_extern_cpp_calls_real_dll() {
        // 集成测试:调用真实的 C++ DLL
        // 如果 DLL 不存在(如 CI 环境),跳过而非失败
        if !cpp_dll_exists() {
            eprintln!("Skipping test_extern_cpp_calls_real_dll: DLL not found at {}", cpp_dll_path());
            return;
        }
        let dll = cpp_dll_path();
        let code = format!(r#"
            extern "C++" module "{}" {{
                fn cpp_add(a: i32, b: i32) -> i32;
            }}
            cpp_add(3, 4)
        "#, dll);
        let result = run(&code);
        assert_eq!(result.unwrap(), Value::Int(7));
    }

    #[test]
    fn test_extern_cpp_factorial_via_dll() {
        if !cpp_dll_exists() {
            eprintln!("Skipping test_extern_cpp_factorial_via_dll: DLL not found");
            return;
        }
        let dll = cpp_dll_path();
        let code = format!(r#"
            extern "C++" module "{}" {{
                fn cpp_factorial(n: i32) -> i32;
            }}
            cpp_factorial(5)
        "#, dll);
        let result = run(&code);
        assert_eq!(result.unwrap(), Value::Int(120));
    }

    #[test]
    fn test_extern_cpp_string_return_via_dll() {
        if !cpp_dll_exists() {
            eprintln!("Skipping test_extern_cpp_string_return_via_dll: DLL not found");
            return;
        }
        let dll = cpp_dll_path();
        let code = format!(r#"
            extern "C++" module "{}" {{
                fn cpp_version() -> str;
            }}
            cpp_version()
        "#, dll);
        let v = run(&code).unwrap();
        match v {
            Value::Str(s) => assert!(s.contains("Link-C++ Bridge"), "got: {}", s),
            other => panic!("Expected str, got {:?}", other),
        }
    }

    #[test]
    fn test_extern_cpp_bool_return_via_dll() {
        if !cpp_dll_exists() {
            eprintln!("Skipping test_extern_cpp_bool_return_via_dll: DLL not found");
            return;
        }
        let dll = cpp_dll_path();
        let code = format!(r#"
            extern "C++" module "{}" {{
                fn cpp_is_even(n: i32) -> bool;
            }}
            cpp_is_even(42)
        "#, dll);
        assert_eq!(run(&code).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_extern_cpp_average_int_params_f64_return() {
        // 测试混合签名:int 参数 + f64 返回值
        if !cpp_dll_exists() {
            eprintln!("Skipping test_extern_cpp_average_int_params_f64_return: DLL not found");
            return;
        }
        let dll = cpp_dll_path();
        let code = format!(r#"
            extern "C++" module "{}" {{
                fn cpp_average(a: i32, b: i32, c: i32) -> f64;
            }}
            cpp_average(10, 20, 30)
        "#, dll);
        let v = run(&code).unwrap();
        match v {
            Value::Float(f) => assert!((f - 20.0).abs() < 0.001, "got: {}", f),
            other => panic!("Expected float, got {:?}", other),
        }
    }

    // ---- Stream 测试 ----

    #[test]
    fn test_stream_create() {
        let result = run("stream([1, 2, 3])").unwrap();
        assert_eq!(result, Value::Stream(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
    }

    #[test]
    fn test_stream_map() {
        let result = run(r#"
            fn double(x: i32) -> i32 { return x * 2; }
            collect(map(stream([1, 2, 3]), double))
        "#).unwrap();
        assert_eq!(result, Value::List(vec![Value::Int(2), Value::Int(4), Value::Int(6)]));
    }

    #[test]
    fn test_stream_filter() {
        let result = run(r#"
            fn is_even(x: i32) -> bool { return x % 2 == 0; }
            collect(filter(stream([1, 2, 3, 4, 5]), is_even))
        "#).unwrap();
        assert_eq!(result, Value::List(vec![Value::Int(2), Value::Int(4)]));
    }

    #[test]
    fn test_stream_for_each() {
        let result = run(r#"
            fn times_two(x: i32) -> i32 { return x * 2; }
            for_each(stream([1, 2, 3]), times_two)
        "#).unwrap();
        assert_eq!(result, Value::None);
    }

    #[test]
    fn test_stream_pipe_basic() {
        let result = run(r#"
            fn double(x: i32) -> i32 { return x * 2; }
            [1, 2, 3] | stream | map(double) | collect
        "#).unwrap();
        assert_eq!(result, Value::List(vec![Value::Int(2), Value::Int(4), Value::Int(6)]));
    }

    #[test]
    fn test_stream_pipe_filter() {
        let result = run(r#"
            fn is_even(x: i32) -> bool { return x % 2 == 0; }
            [1, 2, 3, 4, 5] | stream | filter(is_even) | collect
        "#).unwrap();
        assert_eq!(result, Value::List(vec![Value::Int(2), Value::Int(4)]));
    }

    #[test]
    fn test_stream_empty() {
        let result = run("collect(stream([]))").unwrap();
        assert_eq!(result, Value::List(vec![]));
    }

    // ---- Struct 测试 ----

    #[test]
    fn test_struct_decl_and_init() {
        let result = run(r#"
            struct Point { x: i32, y: i32 }
            let p = Point { x: 1, y: 2 };
            p
        "#).unwrap();
        let mut fields = std::collections::HashMap::new();
        fields.insert("x".to_string(), Value::Int(1));
        fields.insert("y".to_string(), Value::Int(2));
        assert_eq!(result, Value::StructInstance {
            type_name: "Point".to_string(),
            fields,
        });
    }

    #[test]
    fn test_struct_field_access() {
        let result = run(r#"
            struct Point { x: i32, y: i32 }
            let p = Point { x: 10, y: 20 };
            p.x
        "#).unwrap();
        assert_eq!(result, Value::Int(10));
    }

    #[test]
    fn test_struct_field_access_second() {
        let result = run(r#"
            struct Point { x: i32, y: i32 }
            let p = Point { x: 10, y: 20 };
            p.y
        "#).unwrap();
        assert_eq!(result, Value::Int(20));
    }

    #[test]
    fn test_struct_unknown_field_error() {
        let result = run(r#"
            struct Point { x: i32, y: i32 }
            let p = Point { x: 1, y: 2 };
            p.z
        "#);
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_init_unknown_field_error() {
        let result = run(r#"
            struct Point { x: i32, y: i32 }
            let p = Point { x: 1, y: 2, z: 3 };
            p
        "#);
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_in_function() {
        let result = run(r#"
            struct Point { x: i32, y: i32 }
            fn make_point(a: i32, b: i32) -> Point {
                return Point { x: a, y: b };
            }
            let p = make_point(5, 6);
            p.x + p.y
        "#).unwrap();
        assert_eq!(result, Value::Int(11));
    }

    // ---- Enum 测试 ----

    #[test]
    fn test_enum_decl_and_unit_variant() {
        let result = run(r#"
            enum Color { Red, Green, Blue }
            Color::Red
        "#).unwrap();
        assert_eq!(result, Value::EnumValue {
            type_name: "Color".to_string(),
            variant: "Red".to_string(),
            payload: vec![],
        });
    }

    #[test]
    fn test_enum_with_payload() {
        let result = run(r#"
            enum Color { Red, RGB(i32, i32, i32) }
            Color::RGB(255, 0, 0)
        "#).unwrap();
        assert_eq!(result, Value::EnumValue {
            type_name: "Color".to_string(),
            variant: "RGB".to_string(),
            payload: vec![Value::Int(255), Value::Int(0), Value::Int(0)],
        });
    }

    #[test]
    fn test_enum_unknown_variant_error() {
        let result = run(r#"
            enum Color { Red, Green }
            Color::Blue
        "#);
        assert!(result.is_err());
    }

    #[test]
    fn test_enum_payload_arity_error() {
        let result = run(r#"
            enum Color { RGB(i32, i32, i32) }
            Color::RGB(1, 2)
        "#);
        assert!(result.is_err());
    }

    // ---- Match 测试 ----

    #[test]
    fn test_match_unit_variant() {
        let result = run(r#"
            enum Color { Red, Green, Blue }
            let c = Color::Green;
            match c {
                Color::Red => { 1 }
                Color::Green => { 2 }
                Color::Blue => { 3 }
            }
        "#).unwrap();
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_match_with_payload_bindings() {
        let result = run(r#"
            enum Msg { Quit, Move(i32, i32), Write(str) }
            let m = Msg::Move(10, 20);
            match m {
                Msg::Quit => { 0 }
                Msg::Move(x, y) => { x + y }
                Msg::Write(s) => { 0 }
            }
        "#).unwrap();
        assert_eq!(result, Value::Int(30));
    }

    #[test]
    fn test_match_wildcard() {
        let result = run(r#"
            enum Color { Red, Green, Blue }
            let c = Color::Blue;
            match c {
                Color::Red => { 1 }
                _ => { 99 }
            }
        "#).unwrap();
        assert_eq!(result, Value::Int(99));
    }

    #[test]
    fn test_match_literal_int() {
        let result = run(r#"
            let x = 5;
            match x {
                1 => { 100 }
                5 => { 200 }
                _ => { 300 }
            }
        "#).unwrap();
        assert_eq!(result, Value::Int(200));
    }

    #[test]
    fn test_match_no_arm_error() {
        let result = run(r#"
            enum Color { Red, Green }
            let c = Color::Red;
            match c {
                Color::Green => { 1 }
            }
        "#);
        assert!(result.is_err());
    }

    #[test]
    fn test_match_string_literal() {
        let result = run(r#"
            let s = "hello";
            match s {
                "hi" => { 1 }
                "hello" => { 2 }
                _ => { 3 }
            }
        "#).unwrap();
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_struct_and_enum_combined() {
        let result = run(r#"
            struct Point { x: i32, y: i32 }
            enum Shape { Circle(f64), Rect(Point, Point) }
            let p1 = Point { x: 0, y: 0 };
            let p2 = Point { x: 10, y: 20 };
            let s = Shape::Rect(p1, p2);
            match s {
                Shape::Circle(r) => { 0 }
                Shape::Rect(a, b) => { b.x + b.y }
            }
        "#).unwrap();
        assert_eq!(result, Value::Int(30));
    }

    // ---- Flow 声明块测试 ----

    #[test]
    fn test_flow_basic_pipeline() {
        let result = run(r#"
            fn double(x: i32) -> i32 { return x * 2; }
            flow DoubleAll {
                source: stream([1, 2, 3]);
                pipeline:
                    source | map(double) | collect;
            }
        "#).unwrap();
        assert_eq!(result, Value::List(vec![Value::Int(2), Value::Int(4), Value::Int(6)]));
    }

    #[test]
    fn test_flow_without_source() {
        let result = run(r#"
            fn inc(x: i32) -> i32 { return x + 1; }
            flow Inline {
                pipeline:
                    stream([10, 20, 30]) | map(inc) | collect;
            }
        "#).unwrap();
        assert_eq!(result, Value::List(vec![Value::Int(11), Value::Int(21), Value::Int(31)]));
    }

    #[test]
    fn test_flow_with_description() {
        let result = run(r#"
            fn is_even(x: i32) -> bool { return x % 2 == 0; }
            flow Evens "过滤偶数" {
                source: stream([1, 2, 3, 4, 5, 6]);
                pipeline:
                    source | filter(is_even) | collect;
            }
        "#).unwrap();
        assert_eq!(result, Value::List(vec![Value::Int(2), Value::Int(4), Value::Int(6)]));
    }

    #[test]
    fn test_flow_sample_field_ignored() {
        // sample 字段应被解析但忽略,不报错
        let result = run(r#"
            fn id(x: i32) -> i32 { return x; }
            flow Sampled "采样流" {
                source: stream([100, 200]);
                sample: every 1s;
                pipeline:
                    source | map(id) | collect;
            }
        "#).unwrap();
        assert_eq!(result, Value::List(vec![Value::Int(100), Value::Int(200)]));
    }

    #[test]
    fn test_flow_multiple_executed_in_order() {
        // 多个 flow 按源码顺序执行,返回最后一个的值
        let result = run(r#"
            fn double(x: i32) -> i32 { return x * 2; }
            fn triple(x: i32) -> i32 { return x * 3; }
            flow First {
                source: stream([1, 2]);
                pipeline: source | map(double) | collect;
            }
            flow Second {
                source: stream([1, 2]);
                pipeline: source | map(triple) | collect;
            }
        "#).unwrap();
        assert_eq!(result, Value::List(vec![Value::Int(3), Value::Int(6)]));
    }

    #[test]
    fn test_flow_source_variable_isolated() {
        // source 变量不应泄露到 flow 块外部
        let result = run(r#"
            fn double(x: i32) -> i32 { return x * 2; }
            flow F {
                source: stream([1, 2, 3]);
                pipeline: source | map(double) | collect;
            }
            // 此处 source 不应可见,应报错
            source
        "#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined variable: source"));
    }

    #[test]
    fn test_flow_missing_pipeline_errors() {
        let result = run(r#"
            flow Bad {
                source: stream([1, 2, 3]);
            }
        "#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("missing 'pipeline:'"));
    }

    #[test]
    fn test_string_concatenation() {
        // 顺便验证 str + str 字符串拼接(此前是 bug)
        let result = run(r#""hello, " + "world""#).unwrap();
        assert_eq!(result, Value::Str("hello, world".to_string()));
    }

    // ---- async / await / sleep 测试 ----

    #[test]
    fn test_async_fn_declaration() {
        // async fn 可以正常声明和调用(v0.1 阻塞语义)
        let result = run(r#"
            async fn compute(a: i32, b: i32) -> i32 {
                return a + b;
            }
            compute(3, 4)
        "#).unwrap();
        assert_eq!(result, Value::Int(7));
    }

    #[test]
    fn test_await_expression() {
        // await 调用 async 函数(v0.1 等价于直接调用)
        let result = run(r#"
            async fn double(x: i32) -> i32 {
                return x * 2;
            }
            await double(21)
        "#).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_await_in_let_binding() {
        let result = run(r#"
            async fn greet(name: str) -> str {
                return "hello, " + name;
            }
            let msg = await greet("Link");
            msg
        "#).unwrap();
        assert_eq!(result, Value::Str("hello, Link".to_string()));
    }

    #[test]
    fn test_async_await_chained() {
        // 多个 await 链式调用
        let result = run(r#"
            async fn inc(x: i32) -> i32 { return x + 1; }
            async fn double(x: i32) -> i32 { return x * 2; }
            let a = await inc(5);
            let b = await double(a);
            b
        "#).unwrap();
        assert_eq!(result, Value::Int(12));
    }

    #[test]
    fn test_list_concatenation() {
        // 验证 list + list 拼接(此前是 bug)
        let result = run(r#"[1, 2, 3] + [4, 5]"#).unwrap();
        assert_eq!(result, Value::List(vec![
            Value::Int(1), Value::Int(2), Value::Int(3),
            Value::Int(4), Value::Int(5),
        ]));
    }

    // ===== Phase 2.13: 标准库测试 =====

    #[test]
    fn test_stdlib_math_abs() {
        assert_eq!(run("abs(-42)").unwrap(), Value::Int(42));
        assert_eq!(run("abs(-3.14)").unwrap(), Value::Float(3.14));
        assert_eq!(run("abs(5)").unwrap(), Value::Int(5));
    }

    #[test]
    fn test_stdlib_math_min_max() {
        assert_eq!(run("min(3, 7)").unwrap(), Value::Int(3));
        assert_eq!(run("max(3, 7)").unwrap(), Value::Int(7));
        assert_eq!(run("min(2.5, 1.5)").unwrap(), Value::Float(1.5));
    }

    #[test]
    fn test_stdlib_math_sqrt_pow() {
        assert_eq!(run("sqrt(144)").unwrap(), Value::Float(12.0));
        assert_eq!(run("sqrt(2)").unwrap(), Value::Float(1.4142135623730951));
        assert_eq!(run("pow(2, 10)").unwrap(), Value::Float(1024.0));
    }

    #[test]
    fn test_stdlib_math_floor_ceil_round() {
        assert_eq!(run("floor(3.7)").unwrap(), Value::Float(3.0));
        assert_eq!(run("ceil(3.2)").unwrap(), Value::Float(4.0));
        assert_eq!(run("round(3.5)").unwrap(), Value::Int(4));
        assert_eq!(run("round(3.4)").unwrap(), Value::Int(3));
    }

    #[test]
    fn test_stdlib_str_upper_lower() {
        assert_eq!(run("str_upper(\"hello\")").unwrap(), Value::Str("HELLO".to_string()));
        assert_eq!(run("str_lower(\"WORLD\")").unwrap(), Value::Str("world".to_string()));
    }

    #[test]
    fn test_stdlib_str_len_trim() {
        assert_eq!(run("str_len(\"hello\")").unwrap(), Value::Int(5));
        assert_eq!(run("str_trim(\"  hi  \")").unwrap(), Value::Str("hi".to_string()));
    }

    #[test]
    fn test_stdlib_str_contains_starts_ends() {
        assert_eq!(run("str_contains(\"hello world\", \"world\")").unwrap(), Value::Bool(true));
        assert_eq!(run("str_contains(\"hello\", \"xyz\")").unwrap(), Value::Bool(false));
        assert_eq!(run("str_starts_with(\"hello\", \"he\")").unwrap(), Value::Bool(true));
        assert_eq!(run("str_ends_with(\"hello\", \"lo\")").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_stdlib_str_concat() {
        assert_eq!(run("str_concat(\"foo\", \"bar\")").unwrap(), Value::Str("foobar".to_string()));
    }

    #[test]
    fn test_stdlib_str_substring() {
        assert_eq!(run("str_substring(\"hello world\", 0, 5)").unwrap(), Value::Str("hello".to_string()));
        assert_eq!(run("str_substring(\"hello\", 1, 3)").unwrap(), Value::Str("el".to_string()));
    }

    #[test]
    fn test_stdlib_str_split() {
        let result = run("str_split(\"a,b,c\", \",\")").unwrap();
        assert_eq!(result, Value::List(vec![
            Value::Str("a".to_string()),
            Value::Str("b".to_string()),
            Value::Str("c".to_string()),
        ]));
    }

    #[test]
    fn test_stdlib_type_int() {
        assert_eq!(run("int(3.14)").unwrap(), Value::Int(3));
        assert_eq!(run("int(\"123\")").unwrap(), Value::Int(123));
        assert_eq!(run("int(true)").unwrap(), Value::Int(1));
    }

    #[test]
    fn test_stdlib_type_float() {
        assert_eq!(run("float(42)").unwrap(), Value::Float(42.0));
        assert_eq!(run("float(\"3.5\")").unwrap(), Value::Float(3.5));
    }

    #[test]
    fn test_stdlib_type_bool() {
        assert_eq!(run("bool(0)").unwrap(), Value::Bool(false));
        assert_eq!(run("bool(1)").unwrap(), Value::Bool(true));
        assert_eq!(run("bool(\"\")").unwrap(), Value::Bool(false));
        assert_eq!(run("bool(\"x\")").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_stdlib_type_str() {
        assert_eq!(run("str(42)").unwrap(), Value::Str("42".to_string()));
        assert_eq!(run("str(3.14)").unwrap(), Value::Str("3.14".to_string()));
        assert_eq!(run("str(true)").unwrap(), Value::Str("true".to_string()));
    }

    #[test]
    fn test_stdlib_list_push() {
        let result = run("list_push([1, 2, 3], 4)").unwrap();
        assert_eq!(result, Value::List(vec![
            Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4),
        ]));
    }

    #[test]
    fn test_stdlib_list_get() {
        assert_eq!(run("list_get([10, 20, 30], 1)").unwrap(), Value::Int(20));
    }

    #[test]
    fn test_stdlib_list_contains() {
        assert_eq!(run("list_contains([1, 2, 3], 2)").unwrap(), Value::Bool(true));
        assert_eq!(run("list_contains([1, 2, 3], 5)").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_stdlib_list_reverse() {
        let result = run("list_reverse([1, 2, 3])").unwrap();
        assert_eq!(result, Value::List(vec![Value::Int(3), Value::Int(2), Value::Int(1)]));
    }

    #[test]
    fn test_stdlib_list_sort() {
        let result = run("list_sort([3, 1, 4, 1, 5, 9, 2, 6])").unwrap();
        assert_eq!(result, Value::List(vec![
            Value::Int(1), Value::Int(1), Value::Int(2), Value::Int(3),
            Value::Int(4), Value::Int(5), Value::Int(6), Value::Int(9),
        ]));
    }

    #[test]
    fn test_stdlib_list_pop() {
        assert_eq!(run("list_pop([1, 2, 3])").unwrap(), Value::Int(3));
    }

    #[test]
    fn test_stdlib_file_io() {
        // 先写入,再读取
        let _ = run("file_write(\"test_stdlib_tmp.txt\", \"hello\")").unwrap();
        let exists = run("file_exists(\"test_stdlib_tmp.txt\")").unwrap();
        assert_eq!(exists, Value::Bool(true));
        let content = run("file_read(\"test_stdlib_tmp.txt\")").unwrap();
        assert_eq!(content, Value::Str("hello".to_string()));
        // 追加
        let _ = run("file_append(\"test_stdlib_tmp.txt\", \" world\")").unwrap();
        let content2 = run("file_read(\"test_stdlib_tmp.txt\")").unwrap();
        assert_eq!(content2, Value::Str("hello world".to_string()));
        // 清理
        let _ = std::fs::remove_file("test_stdlib_tmp.txt");
    }

    #[test]
    fn test_stdlib_time_now() {
        let result = run("time_now()").unwrap();
        // 应该是一个合理的时间戳(> 2020 年)
        if let Value::Int(t) = result {
            assert!(t > 1577836800, "time_now should be after 2020");
        } else {
            panic!("time_now should return Int");
        }
    }

    #[test]
    fn test_stdlib_to_string() {
        assert_eq!(run("to_string(42)").unwrap(), Value::Str("42".to_string()));
        assert_eq!(run("to_string(true)").unwrap(), Value::Str("true".to_string()));
        assert_eq!(run("to_string([1, 2])").unwrap(), Value::Str("[1, 2]".to_string()));
    }
}
