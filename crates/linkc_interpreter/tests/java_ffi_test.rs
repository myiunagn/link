// Java FFI 集成测试
// 需要 java 在 PATH 中,以及 MathUtils 和 LinkJavaBridge 已编译

use linkc_interpreter::*;
use linkc_parser::*;

#[test]
fn test_java_runtime_construction() {
    // 验证 JavaRuntime 可以构造
    let rt = JavaRuntime::new()
        .with_command("java")
        .with_classpath(".");
    let _ = rt;
}

#[test]
fn test_java_via_source_parsing() {
    // 验证 extern "java" 语法可以被解析
    let source = r#"
        extern "java" module "build/classes::com.example.MathUtils" {
            fn add(a: i64, b: i64) -> i64;
        }
        1
    "#;
    let tokens = linkc_lexer::lex(source);
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Failed to parse extern java syntax: {:?}", result.err());
}

#[test]
fn test_java_short_form() {
    // 简写形式: 直接写类名
    let source = r#"
        extern "java" module "MathUtils" {
            fn add(a: i64, b: i64) -> i64;
        }
        1
    "#;
    let tokens = linkc_lexer::lex(source);
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Failed to parse extern java syntax: {:?}", result.err());
}
