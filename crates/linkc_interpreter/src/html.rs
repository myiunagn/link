use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use crate::Value;
use linkc_parser::TypeAnnotation;

/// HTML/JS FFI 运行时
/// 
/// 通过 HTTP 与本地或远程 Web 服务通信，调用浏览器中运行的 JavaScript 函数。
/// 典型用法：启动一个 Node.js HTTP 服务（也可用浏览器），Link 通过 HTTP POST 调用。
#[derive(Default, Debug)]
pub struct HtmlRuntime {
    /// 服务端点地址（如 http://127.0.0.1:3000）
    endpoint: String,
    /// 调用超时（毫秒）
    timeout_ms: u64,
}

impl HtmlRuntime {
    pub fn new() -> Self {
        Self {
            endpoint: std::env::var("LINK_HTML_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string()),
            timeout_ms: 5000,
        }
    }

    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = endpoint.to_string();
        self
    }

    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// 调用 HTML/JS 端注册的函数
    /// 协议: HTTP POST /<func_name>，请求体为 JSON { args: [...] }，响应为 JSON。
    pub fn call_func(
        &self,
        func_name: &str,
        args: &[Value],
        ret_type: Option<&TypeAnnotation>,
    ) -> Result<Value, String> {
        // 构造 JSON 请求体
        let json_args: Vec<serde_json::Value> = args.iter()
            .map(|v| match v {
                Value::Int(i) => serde_json::json!(i),
                Value::Float(f) => serde_json::json!(f),
                Value::Bool(b) => serde_json::json!(b),
                Value::Str(s) => serde_json::json!(s),
                Value::None => serde_json::json!(null),
                Value::List(items) => {
                    let arr: Vec<serde_json::Value> = items.iter().map(|v| match v {
                        Value::Int(i) => serde_json::json!(i),
                        Value::Float(f) => serde_json::json!(f),
                        Value::Bool(b) => serde_json::json!(b),
                        Value::Str(s) => serde_json::json!(s),
                        _ => serde_json::json!(null),
                    }).collect();
                    serde_json::json!(arr)
                }
                _ => serde_json::json!(null),
            })
            .collect();

        let body = serde_json::json!({ "args": json_args }).to_string();

        // 解析 URL
        let url = format!("{}/{}", self.endpoint.trim_end_matches('/'), func_name);
        let (host, port, path) = parse_url(&url)?;

        // 构造 HTTP 请求
        let request = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            path, host, body.len(), body
        );

        // TCP 连接
        let addr = format!("{}:{}", host, port);
        let mut stream = TcpStream::connect(&addr)
            .map_err(|e| format!("Failed to connect to HTML server {}: {}", addr, e))?;
        
        stream.set_read_timeout(Some(Duration::from_millis(self.timeout_ms)))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;
        stream.set_write_timeout(Some(Duration::from_millis(self.timeout_ms)))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        stream.write_all(request.as_bytes())
            .map_err(|e| format!("Failed to send HTTP request: {}", e))?;

        // 读取响应
        let mut response = String::new();
        stream.read_to_string(&mut response)
            .map_err(|e| format!("Failed to read HTTP response: {}", e))?;

        // 解析 HTTP 响应
        let body_start = response.find("\r\n\r\n")
            .ok_or_else(|| "Invalid HTTP response".to_string())?;
        let response_body = &response[body_start + 4..];
        
        // 解析 JSON
        let json: serde_json::Value = serde_json::from_str(response_body.trim())
            .map_err(|e| format!("Invalid JSON from HTML server: {}", e))?;

        // 期望格式 { "result": <value> } 或直接 <value>
        let result_json = if let Some(obj) = json.as_object() {
            if let Some(r) = obj.get("result") {
                r
            } else if let Some(e) = obj.get("error") {
                return Err(format!("HTML function error: {}", e));
            } else {
                &json
            }
        } else {
            &json
        };

        json_to_value(result_json, ret_type)
    }
}

fn parse_url(url: &str) -> Result<(String, u16, String), String> {
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    let (host_port, path) = match url.find('/') {
        Some(idx) => (&url[..idx], &url[idx..]),
        None => (url, "/"),
    };
    let (host, port) = match host_port.find(':') {
        Some(idx) => {
            let port: u16 = host_port[idx+1..].parse()
                .map_err(|_| format!("Invalid port in URL: {}", url))?;
            (&host_port[..idx], port)
        }
        None => (host_port, 80),
    };
    Ok((host.to_string(), port, path.to_string()))
}

fn json_to_value(json: &serde_json::Value, _expected_type: Option<&TypeAnnotation>) -> Result<Value, String> {
    match json {
        serde_json::Value::Null => Ok(Value::None),
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err("Invalid number from HTML".to_string())
            }
        }
        serde_json::Value::String(s) => Ok(Value::Str(s.clone())),
        serde_json::Value::Array(arr) => {
            let mut items = Vec::new();
            for v in arr {
                items.push(json_to_value(v, None)?);
            }
            Ok(Value::List(items))
        }
        _ => Err(format!("Cannot convert JSON {:?} to Value", json)),
    }
}
