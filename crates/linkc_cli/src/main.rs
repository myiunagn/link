use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        run_repl();
        return;
    }

    let command = &args[1];
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
    } else if command == "--version" || command == "-V" {
        println!("linkc 0.1.0");
    } else if command == "--help" || command == "-h" {
        print_help();
    } else {
        eprintln!("Unknown command: {}", command);
        eprintln!("Run `link --help` for usage.");
        process::exit(1);
    }
}

fn print_help() {
    println!("Usage: link <command> [args]");
    println!();
    println!("Commands:");
    println!("  run <file>              Run a Link source file");
    println!("  compile <file> [opts]   Compile a Link source file");
    println!("  repl                    Start interactive REPL");
    println!("  bindgen <args>          Generate bindings from export blocks");
    println!("  --version, -V           Print version");
    println!("  --help, -h              Print this help");
    println!();
    println!("compile usage:");
    println!("  link compile <input.link> [options]");
    println!();
    println!("Options:");
    println!("  -o <path>           Output path (default: input file stem)");
    println!("  --backend <type>    Codegen backend: c (default) | llvm");
    println!("  --emit-c            Emit C code (C backend only)");
    println!("  --emit-ir           Emit LLVM IR (LLVM backend only)");
    println!("  --opt-level <N>     Optimization level: 0-3 (default: 2)");
    println!("  -g                  Include debug information");
    println!("  --no-link           Don't link to native executable");
    println!();
    println!("Examples:");
    println!("  link compile myfile.link");
    println!("  link compile myfile.link --backend llvm --emit-ir");
    println!("  link compile myfile.link --opt-level 3 -g");
    println!("  link compile myfile.link --emit-c -o output.c");
    println!();
    println!("bindgen usage:");
    println!("  link bindgen --lang <lang> <input.link> [-o <output>] [--module <name>]");
    println!("    --lang      Target language: c | python | typescript");
    println!("    -o          Output file (default: stdout)");
    println!("    --module    Module name (default: input file stem)");
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
                    return Err("--backend requires a value (c | llvm)".to_string());
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
        other => {
            return Err(format!("Unknown backend: '{}' (supported: c, llvm)", other));
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

fn run_repl() {
    println!("Link 0.1.0 REPL");
    println!("Type 'exit' or Ctrl+C to quit");
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

fn run_file(filename: &str) -> Result<(), String> {
    let source = fs::read_to_string(filename)
        .map_err(|e| format!("Cannot read file '{}': {}", filename, e))?;

    let tokens = linkc_lexer::lex(&source);
    let mut parser = linkc_parser::Parser::new(tokens);
    let program = parser.parse_program()?;

    let mut env = linkc_interpreter::Environment::new();
    let mut ctx = linkc_interpreter::InterpContext::new();
    let result = linkc_interpreter::eval_program(&program, &mut env, &mut ctx)?;

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
    }
}
