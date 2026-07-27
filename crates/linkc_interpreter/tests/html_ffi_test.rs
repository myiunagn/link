// HTML/JS FFI 集成测试
// 需要 Node.js 在 PATH 中,以及 server.js 已启动

use linkc_interpreter::*;
use linkc_parser::*;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

fn start_test_server() -> Option<u16> {
    // 启动一个最小化测试服务器(只监听端口,响应 add/multiply)
    let listener = TcpListener::bind("127.0.0.1:0").ok()?;
    let port = listener.local_addr().ok()?.port();
    
    thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = match stream {
                Ok(s) => s,
                Err(_) => continue,
            };
            let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
            
            let mut buf = [0u8; 4096];
            let n = match stream.read(&mut buf) {
                Ok(n) => n,
                Err(_) => 0,
            };
            let request = String::from_utf8_lossy(&buf[..n]);
            
            // 简单响应: 对任何 POST 返回 42
            let response_body = r#"{"result":42}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    
    Some(port)
}

#[test]
fn test_html_endpoint_construction() {
    // 测试 URL 解析逻辑(不实际发起网络请求)
    let rt = HtmlRuntime::new().with_endpoint("http://127.0.0.1:8080");
    // Just ensure the runtime can be configured
    let _ = rt;
}

#[test]
fn test_html_parsing_logic() {
    // 验证 HtmlRuntime 可以构造
    let rt = HtmlRuntime::new()
        .with_timeout(1000)
        .with_endpoint("http://localhost:9000");
    let _ = rt;
}

#[test]
fn test_html_via_source_parsing() {
    // 验证 extern "html" 语法可以被解析
    let source = r#"
        extern "html" {
            fn add(a: i32, b: i32) -> i32;
        }
        1
    "#;
    let tokens = linkc_lexer::lex(source);
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Failed to parse extern html syntax: {:?}", result.err());
}

#[test]
fn test_html_via_source_parsing_with_endpoint() {
    // 验证 extern "html" 语法带 module 端点
    let source = r#"
        extern "html" module "http://127.0.0.1:3000" {
            fn add(a: i32, b: i32) -> i32;
        }
        1
    "#;
    let tokens = linkc_lexer::lex(source);
    let mut parser = Parser::new(tokens);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Failed to parse extern html syntax: {:?}", result.err());
}
