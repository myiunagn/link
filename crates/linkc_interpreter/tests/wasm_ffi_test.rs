// WASM FFI 集成测试

use linkc_interpreter::*;
use linkc_parser::*;

fn get_wasm_bytes() -> &'static [u8] {
    // 使用 include_bytes! 加载 WASM 文件
    // 路径相对于 crate root
    include_bytes!("../../../tests/fixtures/wasm_test_crate/target/wasm32-unknown-unknown/release/wasm_test_crate.wasm")
}

#[test]
fn test_wasm_add_function() {
    let mut ctx = InterpContext::new();
    let mut env = Environment::new();

    let decls = vec![
        FnSignature {
            name: "add".to_string(),
            params: vec![
                ("a".to_string(), TypeAnnotation::I32),
                ("b".to_string(), TypeAnnotation::I32),
            ],
            return_type: Some(TypeAnnotation::I32),
            is_async: false,
        },
    ];

    eval_wasm_module_from_bytes("test_module", get_wasm_bytes(), &decls, &mut env, &mut ctx).unwrap();

    let func = env.get("add").unwrap();
    let result = call_function(&func, &[Value::Int(3), Value::Int(4)], &mut ctx).unwrap();
    assert_eq!(result, Value::Int(7));
}

#[test]
fn test_wasm_multiple_functions() {
    let mut ctx = InterpContext::new();
    let mut env = Environment::new();

    let decls = vec![
        FnSignature {
            name: "add".to_string(),
            params: vec![
                ("a".to_string(), TypeAnnotation::I32),
                ("b".to_string(), TypeAnnotation::I32),
            ],
            return_type: Some(TypeAnnotation::I32),
            is_async: false,
        },
        FnSignature {
            name: "subtract".to_string(),
            params: vec![
                ("a".to_string(), TypeAnnotation::I32),
                ("b".to_string(), TypeAnnotation::I32),
            ],
            return_type: Some(TypeAnnotation::I32),
            is_async: false,
        },
        FnSignature {
            name: "square".to_string(),
            params: vec![
                ("a".to_string(), TypeAnnotation::I32),
            ],
            return_type: Some(TypeAnnotation::I32),
            is_async: false,
        },
    ];

    eval_wasm_module_from_bytes("test_module", get_wasm_bytes(), &decls, &mut env, &mut ctx).unwrap();

    // Test add
    let func = env.get("add").unwrap();
    let result = call_function(&func, &[Value::Int(10), Value::Int(20)], &mut ctx).unwrap();
    assert_eq!(result, Value::Int(30));

    // Test subtract
    let func = env.get("subtract").unwrap();
    let result = call_function(&func, &[Value::Int(10), Value::Int(3)], &mut ctx).unwrap();
    assert_eq!(result, Value::Int(7));

    // Test square
    let func = env.get("square").unwrap();
    let result = call_function(&func, &[Value::Int(5)], &mut ctx).unwrap();
    assert_eq!(result, Value::Int(25));
}

#[test]
fn test_wasm_return_bool() {
    let mut ctx = InterpContext::new();
    let mut env = Environment::new();

    let decls = vec![
        FnSignature {
            name: "is_positive".to_string(),
            params: vec![
                ("a".to_string(), TypeAnnotation::I32),
            ],
            return_type: Some(TypeAnnotation::Bool),
            is_async: false,
        },
    ];

    eval_wasm_module_from_bytes("test_module", get_wasm_bytes(), &decls, &mut env, &mut ctx).unwrap();

    let func = env.get("is_positive").unwrap();
    
    let result = call_function(&func, &[Value::Int(5)], &mut ctx).unwrap();
    assert_eq!(result, Value::Bool(true));

    let result = call_function(&func, &[Value::Int(-1)], &mut ctx).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_wasm_float_function() {
    let mut ctx = InterpContext::new();
    let mut env = Environment::new();

    let decls = vec![
        FnSignature {
            name: "float_add".to_string(),
            params: vec![
                ("a".to_string(), TypeAnnotation::F64),
                ("b".to_string(), TypeAnnotation::F64),
            ],
            return_type: Some(TypeAnnotation::F64),
            is_async: false,
        },
    ];

    eval_wasm_module_from_bytes("test_module", get_wasm_bytes(), &decls, &mut env, &mut ctx).unwrap();

    let func = env.get("float_add").unwrap();
    let result = call_function(&func, &[Value::Float(1.5), Value::Float(2.5)], &mut ctx).unwrap();
    assert_eq!(result, Value::Float(4.0));
}

#[test]
fn test_wasm_multiply_function() {
    let mut ctx = InterpContext::new();
    let mut env = Environment::new();

    let decls = vec![
        FnSignature {
            name: "multiply".to_string(),
            params: vec![
                ("a".to_string(), TypeAnnotation::I32),
                ("b".to_string(), TypeAnnotation::I32),
            ],
            return_type: Some(TypeAnnotation::I32),
            is_async: false,
        },
    ];

    eval_wasm_module_from_bytes("test_module", get_wasm_bytes(), &decls, &mut env, &mut ctx).unwrap();

    let func = env.get("multiply").unwrap();
    let result = call_function(&func, &[Value::Int(6), Value::Int(7)], &mut ctx).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_wasm_via_source_parsing() {
    // Test the full pipeline using source code with extern "wasm"
    let source = r#"
        extern "wasm" module "dummy.wasm" {
            fn add(a: i32, b: i32) -> i32;
        }
        add(100, 200)
    "#;
    
    // This test verifies that the parser can handle extern "wasm" syntax
    // The actual execution requires a valid .wasm file, so we test parsing only
    let tokens = linkc_lexer::lex(source);
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Failed to parse extern wasm syntax: {:?}", result.err());
}
