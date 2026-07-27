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
    } else if command == "repl" {
        run_repl();
    } else if command == "--version" || command == "-V" {
        println!("linkc 0.1.0");
    } else if command == "--help" || command == "-h" {
        println!("Usage: link <command> [args]");
        println!("Commands:");
        println!("  run <file>  Run a Link source file");
        println!("  repl        Start interactive REPL");
        println!("  --version   Print version");
        println!("  --help      Print this help");
    } else {
        eprintln!("Unknown command: {}", command);
        process::exit(1);
    }
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
        linkc_interpreter::Value::Function { name, .. } => println!("<fn {}>", name),
        linkc_interpreter::Value::NativeFunction { name, .. } => println!("<native fn {}>", name),
        linkc_interpreter::Value::ExternFunction { name, .. } => println!("<extern fn {}>", name),
        linkc_interpreter::Value::PythonFunction { name, .. } => println!("<python fn {}>", name),
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
        linkc_interpreter::Value::Stream(items) => {
            print!("stream[");
            for (i, item) in items.iter().enumerate() {
                if i > 0 { print!(", "); }
                print_value(item);
            }
            print!("]");
        }
    }
}
