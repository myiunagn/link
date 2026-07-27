//! linkc_bindgen 集成测试 —— 从源码解析到生成完整流程

use linkc_bindgen::{generate, TargetLang};
use linkc_lexer::lex;
use linkc_parser::Parser;

fn parse(src: &str) -> linkc_parser::Program {
    let tokens = lex(src);
    let mut p = Parser::new(tokens);
    p.parse_program().expect("parse error")
}

#[test]
fn end_to_end_c_header_generation() {
    let src = r#"
export "C" {
    fn link_init() -> bool;
    fn link_add(a: i32, b: i32) -> i32;
    fn link_greet(name: str) -> str;
}
"#;
    let program = parse(src);
    let out = generate(&program, TargetLang::C, "my_sdk").unwrap();
    assert!(out.contains("#ifndef LINK_MY_SDK_H"));
    assert!(out.contains("#define LINK_MY_SDK_H"));
    assert!(out.contains("extern \"C\""));
    assert!(out.contains("bool link_init(void);"));
    assert!(out.contains("int32_t link_add(int32_t a, int32_t b);"));
    assert!(out.contains("const char* link_greet(const char* name);"));
}

#[test]
fn end_to_end_python_pyi_generation() {
    let src = r#"
export "python" module "my_sdk" {
    fn start_server(port: u16) -> bool;
    async fn fetch(url: str) -> str;
    fn events() -> stream<i32>;
}
"#;
    let program = parse(src);
    let out = generate(&program, TargetLang::Python, "my_sdk").unwrap();
    assert!(out.contains("\"\"\""));
    assert!(out.contains("Source module: my_sdk"));
    assert!(out.contains("from typing import AsyncIterable"));
    assert!(out.contains("def start_server(port: int) -> bool: ..."));
    assert!(out.contains("async def fetch(url: str) -> str: ..."));
    assert!(out.contains("def events() -> AsyncIterable[int]: ..."));
}

#[test]
fn end_to_end_typescript_dts_generation() {
    let src = r#"
export "typescript" module "my-sdk" {
    fn createRoom(maxPlayers: u8) -> u32;
    async fn fetchState(id: u32) -> str;
    fn shutdown();
}
"#;
    let program = parse(src);
    let out = generate(&program, TargetLang::TypeScript, "my_sdk").unwrap();
    assert!(out.contains("declare module \"my_sdk\""));
    assert!(out.contains("Source module: my-sdk"));
    assert!(out.contains("export function createRoom(maxPlayers: number): number;"));
    assert!(out.contains("export function fetchState(id: number): Promise<string>;"));
    assert!(out.contains("export function shutdown(): void;"));
}

#[test]
fn error_when_no_matching_export_block() {
    let src = r#"
export "python" {
    fn foo() -> bool;
}
"#;
    let program = parse(src);
    let result = generate(&program, TargetLang::C, "test");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("No `export \"C\""));
}

#[test]
fn multiple_export_blocks_are_merged() {
    let src = r#"
export "C" {
    fn first() -> bool;
}

export "C" {
    fn second(x: i32) -> i32;
}
"#;
    let program = parse(src);
    let out = generate(&program, TargetLang::C, "multi").unwrap();
    assert!(out.contains("bool first(void);"));
    assert!(out.contains("int32_t second(int32_t x);"));
}

#[test]
fn only_matching_language_is_picked_up() {
    // 同时有三种语言的 export 块,只生成对应语言
    let src = r#"
export "C" { fn c_fn() -> bool; }
export "python" { fn py_fn() -> bool; }
export "typescript" { fn ts_fn() -> bool; }
"#;
    let program = parse(src);

    let c_out = generate(&program, TargetLang::C, "multi").unwrap();
    assert!(c_out.contains("c_fn"));
    assert!(!c_out.contains("py_fn"));
    assert!(!c_out.contains("ts_fn"));

    let py_out = generate(&program, TargetLang::Python, "multi").unwrap();
    assert!(py_out.contains("py_fn"));
    assert!(!py_out.contains("c_fn"));

    let ts_out = generate(&program, TargetLang::TypeScript, "multi").unwrap();
    assert!(ts_out.contains("ts_fn"));
    assert!(!ts_out.contains("c_fn"));
}

#[test]
fn pointer_and_named_types_in_c() {
    let src = r#"
export "C" {
    fn alloc(size: usize) -> void*;
    fn get_handle(id: u32) -> RoomHandle;
}
"#;
    let program = parse(src);
    let out = generate(&program, TargetLang::C, "ptr_test").unwrap();
    assert!(out.contains("void* alloc(size_t size);"));
    assert!(out.contains("RoomHandle get_handle(uint32_t id);"));
}

#[test]
fn target_lang_extension() {
    assert_eq!(TargetLang::C.extension(), "h");
    assert_eq!(TargetLang::Python.extension(), "pyi");
    assert_eq!(TargetLang::TypeScript.extension(), "d.ts");
}

#[test]
fn target_lang_from_str() {
    assert_eq!(TargetLang::from_str("c"), Some(TargetLang::C));
    assert_eq!(TargetLang::from_str("C"), Some(TargetLang::C));
    assert_eq!(TargetLang::from_str("header"), Some(TargetLang::C));
    assert_eq!(TargetLang::from_str("python"), Some(TargetLang::Python));
    assert_eq!(TargetLang::from_str("py"), Some(TargetLang::Python));
    assert_eq!(TargetLang::from_str("typescript"), Some(TargetLang::TypeScript));
    assert_eq!(TargetLang::from_str("ts"), Some(TargetLang::TypeScript));
    assert_eq!(TargetLang::from_str("rust"), None);
}
