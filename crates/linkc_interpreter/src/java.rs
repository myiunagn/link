use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::path::PathBuf;
use crate::Value;
use linkc_parser::TypeAnnotation;

/// Java FFI 运行时
/// 
/// 通过 stdio 与 Java 子进程通信，Java 端以 JSON 格式收发调用。
/// 这种方式无需依赖 jni，跨平台且易于部署。
#[derive(Default, Debug)]
pub struct JavaRuntime {
    /// 启动 Java 进程的命令（如 "java"）
    java_cmd: String,
    /// 启动 Java 程序需要的 classpath/参数（-cp 等）
    classpath: Option<String>,
    /// 已加载的 Java 类对应的子进程：class_name -> Child stdin handle
    instances: HashMap<String, JavaProcess>,
}

struct JavaProcess {
    #[allow(dead_code)]
    child: std::process::Child,
}

impl std::fmt::Debug for JavaProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JavaProcess").finish_non_exhaustive()
    }
}

impl JavaRuntime {
    pub fn new() -> Self {
        Self {
            java_cmd: std::env::var("JAVA_CMD").unwrap_or_else(|_| "java".to_string()),
            classpath: std::env::var("JAVA_CLASSPATH").ok(),
            instances: HashMap::new(),
        }
    }

    /// 配置 Java 命令
    pub fn with_command(mut self, cmd: &str) -> Self {
        self.java_cmd = cmd.to_string();
        self
    }

    /// 配置 classpath
    pub fn with_classpath(mut self, cp: &str) -> Self {
        self.classpath = Some(cp.to_string());
        self
    }

    /// 启动 Java 桥接进程。
    /// 桥接器（LinkJavaBridge）是一个内置的 Java 程序，
    /// 它从 stdin 接收 JSON 调用，从 stdout 返回 JSON 结果。
    /// 启动参数应包含主类所在目录。
    pub fn start_bridge(&mut self, class_name: &str, class_path: &PathBuf) -> Result<(), String> {
        if self.instances.contains_key(class_name) {
            return Ok(());
        }

        // 启动一个 "持续运行的 Java 桥接程序"
        // 调用形式: java -cp <cp> LinkJavaBridge <className>
        // 桥接器在接收到 JSON 调用后反射执行 className.methodName(args)
        // 失败时直接返回错误即可（不需要持久进程）
        // 这里用 lazy 模式：每次调用启动一次 java 进程（简单稳健）
        // 真正的"持久进程"模式需要 Java 端实现一个循环读取器
        let cp_str = class_path.to_str().unwrap_or(".");

        let mut cmd = Command::new(&self.java_cmd);
        cmd.arg("-cp").arg(cp_str);
        cmd.arg("LinkJavaBridge");
        cmd.arg(class_name);
        cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        
        let child = cmd.spawn()
            .map_err(|e| format!("Failed to start Java bridge: {}", e))?;

        self.instances.insert(class_name.to_string(), JavaProcess { child });
        Ok(())
    }

    /// 调用 Java 类的静态方法
    /// `class_path` 用于 -cp
    /// Java 端的 LinkJavaBridge 程序负责反射调用
    pub fn call_static(
        &mut self,
        class_name: &str,
        class_path: &str,
        method_name: &str,
        args: &[Value],
        ret_type: Option<&TypeAnnotation>,
    ) -> Result<Value, String> {
        // 构造 JSON 请求
        let json_args: Vec<serde_json::Value> = args.iter()
            .map(value_to_json)
            .collect();
        
        let request = serde_json::json!({
            "class": class_name,
            "method": method_name,
            "args": json_args,
        });
        
        let request_str = format!("{}\n", request.to_string());

        // 启动一次性 Java 进程执行
        let cp_with_bridge = if class_path.is_empty() {
            class_path.to_string()
        } else {
            class_path.to_string()
        };
        
        let output = Command::new(&self.java_cmd)
            .arg("-cp").arg(&cp_with_bridge)
            .arg("LinkJavaBridge")
            .arg("call")
            .arg(class_name)
            .arg(method_name)
            .arg(&request_str)
            .output()
            .map_err(|e| format!("Failed to execute Java: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Java execution failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().next()
            .ok_or_else(|| "No output from Java".to_string())?;
        
        let response: serde_json::Value = serde_json::from_str(line)
            .map_err(|e| format!("Invalid JSON from Java: {}", e))?;

        json_to_value(&response, ret_type)
    }
}

fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Int(i) => serde_json::json!(i),
        Value::Float(f) => serde_json::json!(f),
        Value::Bool(b) => serde_json::json!(b),
        Value::Str(s) => serde_json::json!(s),
        Value::None => serde_json::json!(null),
        Value::List(items) => {
            let arr: Vec<serde_json::Value> = items.iter().map(value_to_json).collect();
            serde_json::json!(arr)
        }
        _ => serde_json::json!(null),
    }
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
                Err("Invalid number from Java".to_string())
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
