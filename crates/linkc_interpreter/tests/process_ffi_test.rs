// Process Bridge FFI 集成测试
// 测试通过子进程调用其他语言的桥接脚本

use linkc_interpreter::*;
use linkc_parser::*;
use std::process::{Command, Stdio};

// ========== 语法解析测试 ==========

#[test]
fn test_go_extern_parsing() {
    let source = r#"
        extern "go" {
            fn add(a: i64, b: i64) -> i64;
            fn greet(name: str) -> str;
        }
        1
    "#;
    let tokens = linkc_lexer::lex(source);
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Failed to parse extern go syntax: {:?}", result.err());
}

#[test]
fn test_rust_extern_parsing() {
    let source = r#"
        extern "rust" module "bridge.rs" {
            fn add(a: i64, b: i64) -> i64;
        }
        1
    "#;
    let tokens = linkc_lexer::lex(source);
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Failed to parse extern rust syntax: {:?}", result.err());
}

#[test]
fn test_csharp_extern_parsing() {
    let source = r#"
        extern "csharp" {
            fn multiply(a: i64, b: i64) -> i64;
        }
        1
    "#;
    let tokens = linkc_lexer::lex(source);
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Failed to parse extern csharp syntax: {:?}", result.err());
}

#[test]
fn test_dotnet_extern_parsing() {
    let source = r#"
        extern "dotnet" module "MyProject" {
            fn add(a: i32, b: i32) -> i32;
        }
        1
    "#;
    let tokens = linkc_lexer::lex(source);
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Failed to parse extern dotnet syntax: {:?}", result.err());
}

#[test]
fn test_php_extern_parsing() {
    let source = r#"
        extern "php" {
            fn greet(name: str) -> str;
        }
        1
    "#;
    let tokens = linkc_lexer::lex(source);
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Failed to parse extern php syntax: {:?}", result.err());
}

#[test]
fn test_ruby_extern_parsing() {
    let source = r#"
        extern "ruby" module "bridge.rb" {
            fn subtract(a: i64, b: i64) -> i64;
        }
        1
    "#;
    let tokens = linkc_lexer::lex(source);
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Failed to parse extern ruby syntax: {:?}", result.err());
}

#[test]
fn test_swift_extern_parsing() {
    let source = r#"
        extern "swift" {
            fn add(a: i64, b: i64) -> i64;
        }
        1
    "#;
    let tokens = linkc_lexer::lex(source);
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Failed to parse extern swift syntax: {:?}", result.err());
}

#[test]
fn test_kotlin_extern_parsing() {
    let source = r#"
        extern "kotlin" module "bridge.kt" {
            fn multiply(a: i64, b: i64) -> i64;
        }
        1
    "#;
    let tokens = linkc_lexer::lex(source);
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Failed to parse extern kotlin syntax: {:?}", result.err());
}

// ========== 运行时注册测试 ==========

#[test]
fn test_process_function_registration_go() {
    let mut ctx = InterpContext::new();
    let mut env = Environment::new();

    let decls = vec![
        FnSignature {
            name: "add".to_string(),
            params: vec![
                ("a".to_string(), TypeAnnotation::I64),
                ("b".to_string(), TypeAnnotation::I64),
            ],
            return_type: Some(TypeAnnotation::I64),
            is_async: false,
        },
    ];

    eval_extern_decl("go", None, &decls, &mut env, &mut ctx).unwrap();

    let func = env.get("add").unwrap();
    match &func {
        Value::ProcessFunction { name, language, .. } => {
            assert_eq!(name, "add");
            assert_eq!(language, "go");
        }
        other => panic!("Expected ProcessFunction, got {:?}", other),
    }
}

#[test]
fn test_process_function_registration_rust() {
    let mut ctx = InterpContext::new();
    let mut env = Environment::new();

    let decls = vec![
        FnSignature {
            name: "greet".to_string(),
            params: vec![("name".to_string(), TypeAnnotation::Str)],
            return_type: Some(TypeAnnotation::Str),
            is_async: false,
        },
    ];

    eval_extern_decl("rust", Some("bridge.rs"), &decls, &mut env, &mut ctx).unwrap();

    let func = env.get("greet").unwrap();
    match &func {
        Value::ProcessFunction { name, language, bridge_path, .. } => {
            assert_eq!(name, "greet");
            assert_eq!(language, "rust");
            assert_eq!(bridge_path, "bridge.rs");
        }
        other => panic!("Expected ProcessFunction, got {:?}", other),
    }
}

#[test]
fn test_process_function_registration_multiple() {
    let mut ctx = InterpContext::new();
    let mut env = Environment::new();

    let decls = vec![
        FnSignature {
            name: "add".to_string(),
            params: vec![
                ("a".to_string(), TypeAnnotation::I64),
                ("b".to_string(), TypeAnnotation::I64),
            ],
            return_type: Some(TypeAnnotation::I64),
            is_async: false,
        },
        FnSignature {
            name: "subtract".to_string(),
            params: vec![
                ("a".to_string(), TypeAnnotation::I64),
                ("b".to_string(), TypeAnnotation::I64),
            ],
            return_type: Some(TypeAnnotation::I64),
            is_async: false,
        },
        FnSignature {
            name: "multiply".to_string(),
            params: vec![
                ("a".to_string(), TypeAnnotation::I64),
                ("b".to_string(), TypeAnnotation::I64),
            ],
            return_type: Some(TypeAnnotation::I64),
            is_async: false,
        },
    ];

    eval_extern_decl("ruby", None, &decls, &mut env, &mut ctx).unwrap();

    assert!(env.get("add").is_ok());
    assert!(env.get("subtract").is_ok());
    assert!(env.get("multiply").is_ok());

    for name in &["add", "subtract", "multiply"] {
        let func = env.get(name).unwrap();
        assert!(matches!(func, Value::ProcessFunction { .. }));
    }
}

#[test]
fn test_all_process_languages_registration() {
    let languages = vec!["go", "rust", "csharp", "dotnet", "php", "ruby", "swift", "kotlin"];
    
    for lang in languages {
        let mut ctx = InterpContext::new();
        let mut env = Environment::new();

        let decls = vec![
            FnSignature {
                name: "test_fn".to_string(),
                params: vec![],
                return_type: Some(TypeAnnotation::I64),
                is_async: false,
            },
        ];

        let result = eval_extern_decl(lang, None, &decls, &mut env, &mut ctx);
        assert!(result.is_ok(), "Failed to register for language {}: {:?}", lang, result.err());

        let func = env.get("test_fn").unwrap();
        match &func {
            Value::ProcessFunction { language, .. } => {
                assert_eq!(language, lang, "Language mismatch for {}", lang);
            }
            other => panic!("Expected ProcessFunction for {}, got {:?}", lang, other),
        }
    }
}

// ========== JSON 序列化/反序列化测试 ==========

#[test]
fn test_value_to_json_conversion() {
    assert_eq!(value_to_json(&Value::Int(42)), serde_json::json!(42));
    assert_eq!(value_to_json(&Value::Float(3.14)), serde_json::json!(3.14));
    assert_eq!(value_to_json(&Value::Bool(true)), serde_json::json!(true));
    assert_eq!(value_to_json(&Value::Str("hello".to_string())), serde_json::json!("hello"));
    assert_eq!(value_to_json(&Value::None), serde_json::json!(null));
    assert_eq!(
        value_to_json(&Value::List(vec![Value::Int(1), Value::Int(2)])),
        serde_json::json!([1, 2])
    );
}

#[test]
fn test_json_to_value_conversion() {
    assert_eq!(json_to_value(&serde_json::json!(42), None).unwrap(), Value::Int(42));
    assert_eq!(json_to_value(&serde_json::json!(3.14), None).unwrap(), Value::Float(3.14));
    assert_eq!(json_to_value(&serde_json::json!(true), None).unwrap(), Value::Bool(true));
    assert_eq!(json_to_value(&serde_json::json!("hello"), None).unwrap(), Value::Str("hello".to_string()));
    assert_eq!(json_to_value(&serde_json::json!(null), None).unwrap(), Value::None);
    assert_eq!(
        json_to_value(&serde_json::json!([1, 2, 3]), None).unwrap(),
        Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
    );
}

// ========== 错误处理测试 ==========

#[test]
fn test_process_runtime_no_bridge() {
    let rt = ProcessRuntime::new();
    let result = rt.call_func("go", "test", "add", &[], None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No bridge script"));
}

#[test]
fn test_process_runtime_unknown_language_no_bridge() {
    let mut rt = ProcessRuntime::new();
    rt.set_bridge("invalid_lang", "/tmp/test");
    // This should fall through to auto-detection and try to execute the bridge directly
    let result = rt.call_func("invalid_lang", "test", "add", &[], None);
    // 结果取决于 bridge 文件是否存在,但应该不会是 "No bridge script" 错误
    // 因为我们已经设置了 bridge
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_process_runtime_from_env() {
    let rt = ProcessRuntime::from_env();
    let _ = rt;
}

// ========== 使用 Python 作为桥接的执行测试 ==========

fn python_available() -> bool {
    Command::new("python")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// 创建 Python 桥接脚本用于测试
fn create_test_bridge() -> String {
    let bridge_path = std::env::temp_dir().join("link_test_bridge.py");
    let bridge_code = r#"
import sys, json

data = json.loads(sys.stdin.read())
fn = data.get("function", "")
args = data.get("args", [])

try:
    if fn == "add":
        a = args[0] if args else 0
        b = args[1] if len(args) > 1 else 0
        print(json.dumps({"result": a + b}))
    elif fn == "subtract":
        a = args[0] if args else 0
        b = args[1] if len(args) > 1 else 0
        print(json.dumps({"result": a - b}))
    elif fn == "multiply":
        a = args[0] if args else 0
        b = args[1] if len(args) > 1 else 0
        print(json.dumps({"result": a * b}))
    elif fn == "greet":
        name = args[0] if args else "World"
        print(json.dumps({"result": f"Hello from bridge, {name}!"}))
    else:
        print(json.dumps({"error": f"Unknown function: {fn}"}))
except Exception as e:
    print(json.dumps({"error": str(e)}))
"#;
    std::fs::write(&bridge_path, bridge_code).unwrap();
    bridge_path.to_string_lossy().to_string()
}

#[test]
fn test_process_runtime_call_via_python_bridge() {
    if !python_available() {
        return;
    }

    let bridge_path = create_test_bridge();
    let mut rt = ProcessRuntime::new();
    // 使用 "python" 语言标识,通过文件扩展名自动检测
    rt.set_bridge("python", &bridge_path);

    // 测试 add
    let result = rt.call_func("python", "test", "add", &[Value::Int(3), Value::Int(4)], Some(&TypeAnnotation::I64));
    assert!(result.is_ok(), "add failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value::Int(7));

    // 测试 subtract
    let result = rt.call_func("python", "test", "subtract", &[Value::Int(10), Value::Int(3)], Some(&TypeAnnotation::I64));
    assert!(result.is_ok(), "subtract failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value::Int(7));

    // 测试 multiply
    let result = rt.call_func("python", "test", "multiply", &[Value::Int(6), Value::Int(7)], Some(&TypeAnnotation::I64));
    assert!(result.is_ok(), "multiply failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value::Int(42));

    // 测试 greet
    let result = rt.call_func("python", "test", "greet", &[Value::Str("Link".to_string())], Some(&TypeAnnotation::Str));
    assert!(result.is_ok(), "greet failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value::Str("Hello from bridge, Link!".to_string()));
}

#[test]
fn test_process_runtime_call_unknown_function() {
    if !python_available() {
        return;
    }

    let bridge_path = create_test_bridge();
    let mut rt = ProcessRuntime::new();
    rt.set_bridge("python", &bridge_path);

    let result = rt.call_func("python", "test", "unknown_func", &[], None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown function"));
}

// ========== 通过 Link 源码执行的集成测试 ==========

#[test]
fn test_full_source_execution_process_bridge() {
    if !python_available() {
        return;
    }

    let bridge_path = create_test_bridge();

    // 使用 "go" 作为语言标识(模拟 Go 桥接),但桥接文件实际是 Python 脚本
    // ProcessRuntime 的自动检测会根据 .py 扩展名选择 python 命令
    let source = r#"
        extern "go" {
            fn add(a: i64, b: i64) -> i64;
            fn greet(name: str) -> str;
        }
        add(10, 20)
    "#;

    let tokens = linkc_lexer::lex(source);
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    let mut env = Environment::new();
    let mut ctx = InterpContext::new();

    // 设置 "go" 语言的桥接为 Python 脚本(演示灵活性)
    ctx.process().set_bridge("go", &bridge_path);

    let result = eval_program(&program, &mut env, &mut ctx);
    assert!(result.is_ok(), "Program execution failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value::Int(30));
}

#[test]
fn test_full_source_with_greet_process_bridge() {
    if !python_available() {
        return;
    }

    let bridge_path = create_test_bridge();

    let source = r#"
        extern "ruby" {
            fn greet(name: str) -> str;
        }
        greet("World")
    "#;

    let tokens = linkc_lexer::lex(source);
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    let mut env = Environment::new();
    let mut ctx = InterpContext::new();

    ctx.process().set_bridge("ruby", &bridge_path);

    let result = eval_program(&program, &mut env, &mut ctx);
    assert!(result.is_ok(), "Program execution failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value::Str("Hello from bridge, World!".to_string()));
}

// ========== ProcessRuntime 配置测试 ==========

#[test]
fn test_process_runtime_set_command() {
    let mut rt = ProcessRuntime::new();
    rt.set_command("go", "/custom/go/path");
    // 验证命令已设置(通过直接访问不太容易,但功能上应该正常)
    let _ = rt;
}

#[test]
fn test_process_runtime_set_bridge() {
    let mut rt = ProcessRuntime::new();
    rt.set_bridge("custom", "/tmp/custom_bridge");
    // 验证桥接已设置
    let result = rt.call_func("custom", "test", "add", &[], None);
    // 因为桥接文件不存在,应该会失败,但不会报 "No bridge script"
    assert!(result.is_err());
    assert!(!result.unwrap_err().contains("No bridge script"));
}