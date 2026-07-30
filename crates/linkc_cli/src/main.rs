use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

mod game;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        run_repl();
        return;
    }

    // Handle -h / -V anywhere (like `python -V script.py`)
    for arg in &args[1..] {
        if arg == "--help" || arg == "-h" || arg == "help" {
            print_help();
            return;
        }
        if arg == "--version" || arg == "-V" || arg == "version" {
            print_version();
            return;
        }
    }

    let first = &args[1];

    // If the first argument is a .link file, run it directly
    // like `python script.py`
    if first.ends_with(".link") {
        if let Err(e) = run_file(first) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
        return;
    }

    let command = first;
    if command == "run" {
        if args.len() < 3 {
            eprintln!("Usage: link run <file.link>");
            process::exit(1);
        }
        let filename = &args[2];
        if let Err(e) = run_file(filename) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    } else if command == "compile" {
        if let Err(e) = run_compile(&args[2..]) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    } else if command == "bindgen" {
        if let Err(e) = run_bindgen(&args[2..]) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    } else if command == "repl" {
        run_repl();
    } else if command == "lsp" {
        run_lsp();
    } else if command == "game" {
        if args.len() < 3 {
            eprintln!("Usage: link game <file.link> [domain_name]");
            process::exit(1);
        }
        let filename = &args[2];
        let domain_name = args.get(3).cloned();
        if let Err(e) = run_game(filename, domain_name) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    } else {
        eprintln!("Unknown command: {}", command);
        eprintln!("Run `link --help` for usage.");
        process::exit(1);
    }
}

fn print_version() {
    println!("Link 1.0.0");
    println!("Copyright (c) 2024 ctost link");
    println!("License: MIT");
}

fn print_help() {
    println!("Link 1.0.0 — a language for connecting everything.");
    println!("Copyright (c) 2024 ctost link  License: MIT");
    println!();
    println!("Usage:");
    println!("  link [options] <file.link> [args]       Run a Link program");
    println!("  link [options]                           Start interactive REPL");
    println!("  link <command> [args]                    Run a subcommand");
    println!();
    println!("Commands:");
    println!("  run    <file.link>          Run a Link source file");
    println!("  compile <file.link> [opts]  Compile to native executable");
    println!("  repl                        Start interactive REPL (default)");
    println!("  bindgen --lang <L> <file>   Generate C/Python/TypeScript bindings");
    println!("  lsp                         Start Language Server Protocol on stdio");
    println!("  game   <file.link>          Start game backend server");
    println!("  help                        Show this help");
    println!("  version                     Show version information");
    println!();
    println!("Options (for compile):");
    println!("  -o <path>           Output file path");
    println!("  --backend <type>    Target: c (default) | llvm | python | wasm");
    println!("  --emit-c            Emit C source instead of binary");
    println!("  --emit-ir           Emit LLVM IR instead of binary");
    println!("  --opt-level <0-3>   Optimization level (default: 2)");
    println!("  -g                  Include debug symbols");
    println!("  --no-link           Stop after codegen, don't link");
    println!();
    println!("Options (for bindgen):");
    println!("  --lang <L>          Target: c | python | typescript");
    println!("  -o <path>           Output file (default: stdout)");
    println!("  --module <name>     Module name (default: input stem)");
    println!();
    println!("General options:");
    println!("  -h, --help          Print this help and exit");
    println!("  -V, --version       Print version and exit");
    println!();
    println!("Examples:");
    println!("  link hello.link");
    println!("  link run hello.link");
    println!("  link compile app.link -o app");
    println!("  link compile app.link --backend python -o app.py");
    println!("  link compile app.link --backend wasm -o app.wat");
    println!("  link bindgen --lang c mylib.link -o mylib.h");
    println!("  link lsp");
    println!();
    println!("Environment:");
    println!("  CC                  C compiler (default: cc / gcc)");
    println!("  LINKPATH            Additional module search paths");
    println!();
    println!("Project: https://github.com/myiunagn/link");
    println!("Docs:    https://myiunagn.github.io/linkdoc/");
}

fn run_compile(args: &[String]) -> Result<(), String> {
    let mut input: Option<&str> = None;
    let mut output: Option<&str> = None;
    let mut backend = "c".to_string();
    let mut emit_c = false;
    let mut emit_ir = false;
    let mut opt_level = linkc_codegen::OptLevel::O2;
    let mut debug_info = false;
    let mut no_link = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    return Err("-o requires a path".to_string());
                }
                output = Some(args[i].as_str());
            }
            "--backend" => {
                i += 1;
                if i >= args.len() {
                    return Err("--backend requires a value (c | llvm | python | wasm)".to_string());
                }
                backend = args[i].clone();
            }
            "--emit-c" => emit_c = true,
            "--emit-ir" => emit_ir = true,
            "--opt-level" => {
                i += 1;
                if i >= args.len() {
                    return Err("--opt-level requires a value (0-3)".to_string());
                }
                opt_level = linkc_codegen::OptLevel::from_str(&args[i])?;
            }
            "-g" => debug_info = true,
            "--no-link" => no_link = true,
            s if s.starts_with("--") => {
                return Err(format!("Unknown compile option: {}", s));
            }
            _ => {
                if input.is_some() {
                    return Err(format!("Unexpected extra argument: {}", args[i]));
                }
                input = Some(args[i].as_str());
            }
        }
        i += 1;
    }

    let input_path = input.ok_or("Missing input file (e.g. my_file.link)")?;

    let source = fs::read_to_string(input_path)
        .map_err(|e| format!("Cannot read file '{}': {}", input_path, e))?;

    let tokens = linkc_lexer::lex(&source);
    let mut parser = linkc_parser::Parser::new(tokens);
    let program = parser.parse_program()?;

    // 模块系统: 处理 use 声明,加载并合并依赖模块的 AST
    let program = load_modules(program, input_path)?;

    let errors = linkc_sema::check_program(&program);
    if !errors.is_empty() {
        for err in &errors {
            eprintln!("{}", err);
        }
        return Err(format!("Type checking failed with {} error(s)", errors.len()));
    }

    let borrow_errors = linkc_sema::check_borrow(&program);
    if !borrow_errors.is_empty() {
        for err in &borrow_errors {
            eprintln!("{}", err);
        }
        return Err(format!("Borrow checking failed with {} error(s)", borrow_errors.len()));
    }

    let program = linkc_sema::const_fold(&program);
    let program = linkc_sema::eliminate_dead_code(&program);

    let stem = std::path::Path::new(input_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("a")
        .to_string();

    let output_path = match output {
        Some(p) => p.to_string(),
        None => {
            if emit_c { format!("{}.c", stem) }
            else if emit_ir { format!("{}.ll", stem) }
            else if backend == "python" { format!("{}.py", stem) }
            else if backend == "wasm" { format!("{}.wat", stem) }
            else { stem.clone() }
        }
    };

    match backend.as_str() {
        "c" => {
            if emit_c {
                let c_code = linkc_codegen::compile_to_c_with_opts(&program, opt_level, debug_info)?;
                fs::write(&output_path, c_code)
                    .map_err(|e| format!("Cannot write to '{}': {}", output_path, e))?;
                println!("Generated C code: {}", output_path);
            } else if no_link {
                let obj_path = format!("{}.o", output_path);
                linkc_codegen::compile_to_native_with_opts(&program, &obj_path, opt_level, debug_info)?;
                println!("Generated object file: {}", obj_path);
            } else {
                let result = linkc_codegen::compile_to_native_with_opts(&program, &output_path, opt_level, debug_info)?;
                println!("Compiled (C backend): {}", result);
            }
        }
        "llvm" => {
            let mut llvm = linkc_llvm::LlvmBackend::new();
            if emit_ir {
                let ir = llvm.compile_to_ir(&program)?;
                fs::write(&output_path, ir)
                    .map_err(|e| format!("Cannot write to '{}': {}", output_path, e))?;
                println!("Generated LLVM IR: {}", output_path);
            } else {
                let result = llvm.compile_to_native(&program, &output_path)?;
                println!("Compiled (LLVM backend): {}", result);
            }
        }
        "python" | "py" => {
            let py_code = linkc_codegen::compile_to_python(&program)?;
            fs::write(&output_path, py_code)
                .map_err(|e| format!("Cannot write to '{}': {}", output_path, e))?;
            println!("Generated Python code: {}", output_path);
        }
        "wasm" => {
            let wat_code = linkc_codegen::compile_to_wasm(&program)?;
            fs::write(&output_path, wat_code)
                .map_err(|e| format!("Cannot write to '{}': {}", output_path, e))?;
            println!("Generated WebAssembly (WAT) code: {}", output_path);
        }
        other => {
            return Err(format!("Unknown backend: '{}' (supported: c, llvm, python, wasm)", other));
        }
    }

    Ok(())
}

fn run_bindgen(args: &[String]) -> Result<(), String> {
    let mut lang: Option<&str> = None;
    let mut input: Option<&str> = None;
    let mut output: Option<&str> = None;
    let mut module: Option<&str> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--lang" | "-l" => {
                i += 1;
                if i >= args.len() {
                    return Err("--lang requires a value (c | python | typescript)".to_string());
                }
                lang = Some(args[i].as_str());
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    return Err("-o requires a file path".to_string());
                }
                output = Some(args[i].as_str());
            }
            "--module" => {
                i += 1;
                if i >= args.len() {
                    return Err("--module requires a name".to_string());
                }
                module = Some(args[i].as_str());
            }
            s if s.starts_with("--") => {
                return Err(format!("Unknown bindgen option: {}", s));
            }
            s if s.starts_with('-') && s.len() > 1 => {
                return Err(format!("Unknown bindgen option: {}", s));
            }
            _ => {
                if input.is_some() {
                    return Err(format!("Unexpected extra argument: {}", args[i]));
                }
                input = Some(args[i].as_str());
            }
        }
        i += 1;
    }

    let lang_str = lang.ok_or("Missing --lang (c | python | typescript)")?;
    let target = linkc_bindgen::TargetLang::from_str(lang_str)
        .ok_or_else(|| format!("Unknown language: '{}' (supported: c, python, typescript)", lang_str))?;
    let input_path = input.ok_or("Missing input file (e.g. my_module.link)")?;

    let source = fs::read_to_string(input_path)
        .map_err(|e| format!("Cannot read file '{}': {}", input_path, e))?;

    let tokens = linkc_lexer::lex(&source);
    let mut parser = linkc_parser::Parser::new(tokens);
    let program = parser.parse_program()?;

    // 模块名默认从输入文件名推导
    let module_name = match module {
        Some(m) => m.to_string(),
        None => {
            let stem = std::path::Path::new(input_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("link_module")
                .to_string();
            stem
        }
    };

    let code = linkc_bindgen::generate(&program, target, &module_name)?;

    match output {
        Some(path) => {
            fs::write(path, code)
                .map_err(|e| format!("Cannot write to '{}': {}", path, e))?;
            println!("Generated: {}", path);
        }
        None => {
            print!("{}", code);
        }
    }
    Ok(())
}

fn run_lsp() {
    // The LSP server reads JSON-RPC from stdin and writes to stdout.
    // Logging goes to stderr so it doesn't corrupt the protocol stream.
    let mut server = linkc_lsp::LanguageServer::new();
    if let Err(e) = server.run() {
        eprintln!("link lsp error: {}", e);
        process::exit(1);
    }
}

fn run_game(filename: &str, domain_name: Option<String>) -> Result<(), String> {
    let source = fs::read_to_string(filename)
        .map_err(|e| format!("Cannot read '{}': {}", filename, e))?;
    let tokens = linkc_lexer::lex(&source);
    let mut parser = linkc_parser::Parser::new(tokens);
    let program = parser.parse_program()
        .map_err(|e| format!("Parse error: {}", e))?;

    // 解释执行，创建 domain 配置对象
    let mut env = linkc_interpreter::Environment::new();
    let mut ctx = linkc_interpreter::InterpContext::new();
    linkc_interpreter::eval_program(&program, &mut env, &mut ctx)
        .map_err(|e| format!("Runtime error: {}", e))?;

    // 查找 domain 配置
    let domain_var = domain_name.as_deref().unwrap_or("GameServer");
    let domain_val = env.get(domain_var)
        .map_err(|_| format!("Domain '{}' not found in program. Define it with `domain {} {{ ... }}`", domain_var, domain_var))?;

    let cfg = game::config_from_link(&domain_val)?;

    println!("Starting game server from domain '{}' in '{}'", domain_var, filename);

    // 启动 tokio 运行时和游戏服务器
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;
    rt.block_on(async {
        if let Err(e) = game::run_server(cfg).await {
            eprintln!("Game server error: {}", e);
        }
    });

    Ok(())
}

fn run_repl() {
    println!("Link 1.0.0 (ctost link) [C backend]");
    println!("Type \"help\" for more information, \"exit\" to quit.");
    let mut env = linkc_interpreter::Environment::new();
    let mut ctx = linkc_interpreter::InterpContext::new();
    let mut input = String::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        input.clear();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {
                let line = input.trim();
                if line.is_empty() { continue; }
                if line == "exit" || line == "quit" { break; }
                if line == "help" {
                    println!("Type \"exit\" to quit, \"copyright\" for copyright, \"version\" for version.");
                    println!("Enter any Link expression or statement to evaluate it.");
                    println!("Example:  1 + 2");
                    println!("         let x = 42;");
                    println!("         println(x);");
                    continue;
                }
                if line == "copyright" {
                    println!("Copyright (c) 2024 ctost link  License: MIT");
                    continue;
                }
                if line == "version" {
                    println!("Link 1.0.0 (ctost link)");
                    continue;
                }
                match run_line(line, &mut env, &mut ctx) {
                    Ok(val) => print_result(&val),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(_) => break,
        }
    }
}

fn run_line(source: &str, env: &mut linkc_interpreter::Environment, ctx: &mut linkc_interpreter::InterpContext) -> Result<linkc_interpreter::Value, String> {
    let tokens = linkc_lexer::lex(source);
    let mut parser = linkc_parser::Parser::new(tokens);
    let program = parser.parse_program()?;
    linkc_interpreter::eval_program(&program, env, ctx)
}

fn print_result(val: &linkc_interpreter::Value) {
    match val {
        linkc_interpreter::Value::None => {}
        _ => {
            print_value(val);
            println!();
        }
    }
}

/// 模块系统: 处理 use 声明,加载并合并依赖模块的 AST
///
/// 模块解析规则(v0.1):
/// - `use foo::bar;` 在入口文件所在目录查找 `foo/bar.link` 或 `foo/bar/mod.link`
/// - 被导入模块的顶层 fn/struct/enum 声明被合并到主程序的顶层
/// - 被导入模块自身的 `use` 也会被递归加载(深度优先,检测循环)
/// - `use foo::bar as baz;` 暂不重命名(仅记录元数据),baz 作为别名留待后续
fn load_modules(program: linkc_parser::Program, entry_path: &str) -> Result<linkc_parser::Program, String> {
    use linkc_parser::{Program as P, Stmt};
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    let entry_dir = Path::new(entry_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    fn find_module(base: &Path, path: &[String]) -> Option<PathBuf> {
        // Search in base, plus standard library locations
        let mut search_bases = vec![base.to_path_buf()];
        // Also search lib/ relative to the project (for std library)
        if let Some(lib_dir) = find_lib_dir(base) {
            search_bases.push(lib_dir);
        }

        let last = path.last().unwrap();
        let rel_dir: PathBuf = path[..path.len() - 1].iter().collect();

        for dir in &search_bases {
            let p = dir.join(&rel_dir);
            let f1 = p.join(format!("{}.link", last));
            if f1.exists() { return Some(f1); }
            let f2 = p.join(last).join("mod.link");
            if f2.exists() { return Some(f2); }
            let f3 = p.join(last);
            if f3.exists() && f3.is_file() { return Some(f3); }
        }
        None
    }

    fn find_lib_dir(entry_dir: &Path) -> Option<PathBuf> {
        // Walk up from entry_dir looking for lib/std/
        let mut current = entry_dir.to_path_buf();
        for _ in 0..5 {
            let lib = current.join("lib/std");
            if lib.exists() { return Some(current.join("lib")); }
            if !current.pop() { break; }
        }
        None
    }

    fn load_recursive(
        path: &Path,
        loaded: &mut HashSet<PathBuf>,
        entry_dir: &Path,
    ) -> Result<Vec<Stmt>, String> {
        let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if loaded.contains(&canon) {
            return Ok(Vec::new()); // 已加载,避免循环
        }
        loaded.insert(canon.clone());

        let source = fs::read_to_string(path)
            .map_err(|e| format!("Cannot read module '{}': {}", path.display(), e))?;
        let tokens = linkc_lexer::lex(&source);
        let mut parser = linkc_parser::Parser::new(tokens);
        let sub_program = parser.parse_program()?;

        let P::Block(mut all_stmts) = sub_program;
        let mut merged = Vec::new();

        // 模块自身的 use 声明先递归加载
        let mut deferred = Vec::new();
        for stmt in all_stmts.drain(..) {
            match &stmt {
                Stmt::UseDecl { path: p, .. } => {
                    if let Some(module_file) = find_module(entry_dir, p) {
                        let sub_stmts = load_recursive(&module_file, loaded, entry_dir)?;
                        merged.extend(sub_stmts);
                    } else {
                        eprintln!("warning: module '{}' not found, skipped", p.join("::"));
                    }
                }
                _ => deferred.push(stmt),
            }
        }
        merged.extend(deferred);

        // Resolve module::function calls to plain function calls
        Ok(merged)
    }

    // Replace module::func() with func() for imported modules
    fn resolve_module_paths(stmts: Vec<Stmt>, module_names: &[String]) -> Vec<Stmt> {
        stmts.into_iter().map(|stmt| resolve_stmt_paths(stmt, module_names)).collect()
    }

    fn resolve_stmt_paths(stmt: linkc_parser::Stmt, module_names: &[String]) -> linkc_parser::Stmt {
        use linkc_parser::{Expr, Stmt};
        match stmt {
            Stmt::Expr(expr) => Stmt::Expr(resolve_expr_paths(expr, module_names)),
            Stmt::LetDecl { name, type_annotation, value } => {
                Stmt::LetDecl {
                    name,
                    type_annotation,
                    value: value.map(|e| resolve_expr_paths(e, module_names)),
                }
            }
            Stmt::Return(Some(expr)) => Stmt::Return(Some(resolve_expr_paths(expr, module_names))),
            Stmt::If { condition, then_branch, else_branch } => Stmt::If {
                condition: resolve_expr_paths(condition, module_names),
                then_branch: resolve_block_paths(then_branch, module_names),
                else_branch: else_branch.map(|b| resolve_block_paths(b, module_names)),
            },
            Stmt::While { condition, body } => Stmt::While {
                condition: resolve_expr_paths(condition, module_names),
                body: resolve_block_paths(body, module_names),
            },
            Stmt::Assign { target, value } => Stmt::Assign {
                target: Box::new(resolve_expr_paths(*target, module_names)),
                value: Box::new(resolve_expr_paths(*value, module_names)),
            },
            Stmt::Match { scrutinee, arms } => Stmt::Match {
                scrutinee: resolve_expr_paths(scrutinee, module_names),
                arms: arms.into_iter().map(|arm| linkc_parser::MatchArm {
                    pattern: arm.pattern,
                    body: resolve_block_paths(arm.body, module_names),
                }).collect(),
            },
            Stmt::FnDecl { name, params, return_type, body, is_async } => Stmt::FnDecl {
                name,
                params,
                return_type,
                body: resolve_block_paths(body, module_names),
                is_async,
            },
            other => other,
        }
    }

    fn resolve_block_paths(block: linkc_parser::Block, module_names: &[String]) -> linkc_parser::Block {
        linkc_parser::Block {
            stmts: block.stmts.into_iter().map(|s| resolve_stmt_paths(s, module_names)).collect(),
        }
    }

    fn resolve_expr_paths(expr: linkc_parser::Expr, module_names: &[String]) -> linkc_parser::Expr {
        use linkc_parser::Expr;
        match expr {
            Expr::PathCall { base, segment, args } => {
                if module_names.contains(&base) {
                    Expr::Call { callee: segment, args: args.into_iter().map(|e| resolve_expr_paths(e, module_names)).collect() }
                } else {
                    Expr::PathCall {
                        base,
                        segment,
                        args: args.into_iter().map(|e| resolve_expr_paths(e, module_names)).collect(),
                    }
                }
            }
            Expr::Path { base, segment } => {
                if module_names.contains(&base) {
                    Expr::Ident(segment)
                } else {
                    Expr::Path { base, segment }
                }
            }
            Expr::Call { callee, args } => Expr::Call {
                callee,
                args: args.into_iter().map(|e| resolve_expr_paths(e, module_names)).collect(),
            },
            Expr::Binary { op, left, right } => Expr::Binary {
                op,
                left: Box::new(resolve_expr_paths(*left, module_names)),
                right: Box::new(resolve_expr_paths(*right, module_names)),
            },
            Expr::Unary { op, operand } => Expr::Unary {
                op,
                operand: Box::new(resolve_expr_paths(*operand, module_names)),
            },
            Expr::FieldAccess { target, field } => Expr::FieldAccess {
                target: Box::new(resolve_expr_paths(*target, module_names)),
                field,
            },
            Expr::StructInit { name, fields } => Expr::StructInit {
                name,
                fields: fields.into_iter().map(|(k, v)| (k, resolve_expr_paths(v, module_names))).collect(),
            },
            Expr::IfExpr { condition, then_value, else_value } => Expr::IfExpr {
                condition: Box::new(resolve_expr_paths(*condition, module_names)),
                then_value: Box::new(resolve_expr_paths(*then_value, module_names)),
                else_value: Box::new(resolve_expr_paths(*else_value, module_names)),
            },
            Expr::Index { target, index } => Expr::Index {
                target: Box::new(resolve_expr_paths(*target, module_names)),
                index: Box::new(resolve_expr_paths(*index, module_names)),
            },
            Expr::MatchExpr { scrutinee, arms } => Expr::MatchExpr {
                scrutinee: Box::new(resolve_expr_paths(*scrutinee, module_names)),
                arms: arms.into_iter().map(|arm| linkc_parser::MatchArm {
                    pattern: arm.pattern,
                    body: resolve_block_paths(arm.body, module_names),
                }).collect(),
            },
            other => other,
        }
    }

    let P::Block(mut stmts) = program;
    let mut merged = Vec::new();
    let mut loaded: HashSet<PathBuf> = HashSet::new();

    // 入口文件标记为已加载,避免重复
    let entry_canon = Path::new(entry_path)
        .canonicalize()
        .unwrap_or_else(|_| Path::new(entry_path).to_path_buf());
    loaded.insert(entry_canon);

    // Collect module names from use declarations
    let module_names: Vec<String> = stmts.iter().filter_map(|stmt| {
        if let Stmt::UseDecl { path, .. } = stmt { path.last().cloned() } else { None }
    }).collect();

    // 第一遍: 收集 use 声明并加载模块,延迟其他语句
    let mut deferred = Vec::new();
    for stmt in stmts.drain(..) {
        match &stmt {
            Stmt::UseDecl { path: p, .. } => {
                if let Some(module_file) = find_module(&entry_dir, p) {
                    let sub_stmts = load_recursive(&module_file, &mut loaded, &entry_dir)?;
                    merged.extend(sub_stmts);
                } else {
                    eprintln!("warning: module '{}' not found, skipped", p.join("::"));
                }
                // use 声明本身保留(作为元数据,代码生成器会忽略)
                merged.push(stmt);
            }
            _ => deferred.push(stmt),
        }
    }
    merged.extend(deferred);

    // Resolve module::function → function for known modules
    let merged = resolve_module_paths(merged, &module_names);

    Ok(P::Block(merged))
}

fn run_file(filename: &str) -> Result<(), String> {
    let source = fs::read_to_string(filename)
        .map_err(|e| format!("Cannot read file '{}': {}", filename, e))?;

    let tokens = linkc_lexer::lex(&source);
    let mut parser = linkc_parser::Parser::new(tokens);
    let program = parser.parse_program()?;

    // 模块系统: 处理 use 声明
    let program = load_modules(program, filename)?;

    let mut env = linkc_interpreter::Environment::new();
    let mut ctx = linkc_interpreter::InterpContext::new();
    let program_result = linkc_interpreter::eval_program(&program, &mut env, &mut ctx)?;

    // 自动调用 main 函数(如果存在)
    let result = match env.get("main") {
        Ok(linkc_interpreter::Value::Function { body, params, .. }) if params.is_empty() => {
            let mut main_env = linkc_interpreter::Environment::extend(env.clone());
            linkc_interpreter::eval_block(&body.stmts, &mut main_env, &mut ctx)?
        }
        Ok(other) => other,
        Err(_) => program_result,
    };

    match result {
        linkc_interpreter::Value::Int(n) => println!("{}", n),
        linkc_interpreter::Value::Float(f) => println!("{}", f),
        linkc_interpreter::Value::Str(s) => println!("{}", s),
        linkc_interpreter::Value::Bool(b) => println!("{}", b),
        linkc_interpreter::Value::None => println!("none"),
        linkc_interpreter::Value::List(_) => { print_value(&result); println!(); }
        linkc_interpreter::Value::Stream(_) => { print_value(&result); println!(); }
        linkc_interpreter::Value::StructInstance { .. } => { print_value(&result); println!(); }
        linkc_interpreter::Value::EnumValue { .. } => { print_value(&result); println!(); }
        linkc_interpreter::Value::Function { name, .. } => println!("<fn {}>", name),
        linkc_interpreter::Value::NativeFunction { name, .. } => println!("<native fn {}>", name),
        linkc_interpreter::Value::ExternFunction { name, .. } => println!("<extern fn {}>", name),
        linkc_interpreter::Value::PythonFunction { name, .. } => println!("<python fn {}>", name),
        linkc_interpreter::Value::WasmFunction { name, .. } => println!("<wasm fn {}>", name),
        linkc_interpreter::Value::JavaFunction { name, .. } => println!("<java fn {}>", name),
        linkc_interpreter::Value::HtmlFunction { name, .. } => println!("<html fn {}>", name),
        linkc_interpreter::Value::ProcessFunction { name, language, .. } => println!("<{} fn {}>", language, name),
        linkc_interpreter::Value::Tuple(_) => { print_value(&result); println!(); }
    }
    Ok(())
}

fn print_value(val: &linkc_interpreter::Value) {
    match val {
        linkc_interpreter::Value::Int(n) => print!("{}", n),
        linkc_interpreter::Value::Float(f) => print!("{}", f),
        linkc_interpreter::Value::Str(s) => print!("{}", s),
        linkc_interpreter::Value::Bool(b) => print!("{}", b),
        linkc_interpreter::Value::None => print!("none"),
        linkc_interpreter::Value::List(items) => {
            print!("[");
            for (i, item) in items.iter().enumerate() {
                if i > 0 { print!(", "); }
                print_value(item);
            }
            print!("]");
        }
        linkc_interpreter::Value::Function { name, .. } => print!("<fn {}>", name),
        linkc_interpreter::Value::NativeFunction { name, .. } => print!("<native fn {}>", name),
        linkc_interpreter::Value::ExternFunction { name, .. } => print!("<extern fn {}>", name),
        linkc_interpreter::Value::PythonFunction { name, .. } => print!("<python fn {}>", name),
        linkc_interpreter::Value::WasmFunction { name, .. } => print!("<wasm fn {}>", name),
        linkc_interpreter::Value::JavaFunction { name, .. } => print!("<java fn {}>", name),
        linkc_interpreter::Value::HtmlFunction { name, .. } => print!("<html fn {}>", name),
        linkc_interpreter::Value::ProcessFunction { name, language, .. } => print!("<{} fn {}>", language, name),
        linkc_interpreter::Value::Stream(items) => {
            print!("stream[");
            for (i, item) in items.iter().enumerate() {
                if i > 0 { print!(", "); }
                print_value(item);
            }
            print!("]");
        }
        linkc_interpreter::Value::StructInstance { type_name, fields } => {
            print!("{} {{ ", type_name);
            for (i, (k, v)) in fields.iter().enumerate() {
                if i > 0 { print!(", "); }
                print!("{}: ", k);
                print_value(v);
            }
            print!(" }}");
        }
        linkc_interpreter::Value::EnumValue { type_name, variant, payload } => {
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
        linkc_interpreter::Value::Tuple(elems) => {
            print!("(");
            for (i, e) in elems.iter().enumerate() {
                if i > 0 { print!(", "); }
                print_value(e);
            }
            print!(")");
        }
    }
}
