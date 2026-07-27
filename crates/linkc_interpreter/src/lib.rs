use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use linkc_parser::{Program, Stmt, Expr, Block, BinOp, UnaryOp, TypeAnnotation, FnSignature};
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

fn eval_block(stmts: &[Stmt], env: &mut Environment, ctx: &mut InterpContext) -> Result<Value, String> {
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
            // 用特殊错误类型传递 return 值
            Err(format!("__return__|{}", value_to_string(&val)))
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
    }
}

fn eval_binary_op(op: &BinOp, left: &Value, right: &Value) -> Result<Value, String> {
    match op {
        BinOp::Add => {
            if left.type_name() == "int" && right.type_name() == "int" {
                Ok(Value::Int(left.as_int()? + right.as_int()?))
            } else {
                Ok(Value::Float(left.as_float()? + right.as_float()?))
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
                Err(e) if e.starts_with("__return__|") => {
                    let val_str = &e["__return__|".len()..];
                    Ok(parse_value_from_string(val_str))
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
}

fn builtin_print(args: &[Value]) -> Result<Value, String> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 { print!(" "); }
        print_value(arg);
    }
    Ok(Value::None)
}

fn builtin_println(args: &[Value]) -> Result<Value, String> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 { print!(" "); }
        print_value(arg);
    }
    println!();
    Ok(Value::None)
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

fn parse_value_from_string(s: &str) -> Value {
    if let Ok(n) = s.parse::<i64>() {
        Value::Int(n)
    } else if let Ok(f) = s.parse::<f64>() {
        Value::Float(f)
    } else if s == "true" {
        Value::Bool(true)
    } else if s == "false" {
        Value::Bool(false)
    } else if s == "none" {
        Value::None
    } else {
        Value::Str(s.to_string())
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
}
