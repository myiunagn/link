use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use crate::Value;
use linkc_parser::TypeAnnotation;

/// 通用进程桥接 Runtime
/// 
/// 支持通过子进程调用任意语言的脚本/程序。
/// 桥接协议:
///   输入: JSON 请求写入 stdin
///   输出: 从 stdout 读取一行 JSON 响应
///
/// 支持的语言: go, rust, csharp, php, ruby, swift, kotlin
/// 
/// 每种语言需要一个桥接脚本/程序,负责:
///   1. 从 stdin 读取 JSON 参数
///   2. 解析 JSON
///   3. 调用目标函数
///   4. 输出 JSON 结果到 stdout
#[derive(Default, Debug)]
pub struct ProcessRuntime {
    /// 语言命令映射: language -> command
    commands: HashMap<String, String>,
    /// 桥接脚本路径: language -> bridge_path
    bridges: HashMap<String, String>,
}

impl ProcessRuntime {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            bridges: HashMap::new(),
        }
    }

    /// 设置语言的执行命令
    pub fn set_command(&mut self, language: &str, command: &str) {
        self.commands.insert(language.to_string(), command.to_string());
    }

    /// 设置桥接脚本路径
    pub fn set_bridge(&mut self, language: &str, bridge_path: &str) {
        self.bridges.insert(language.to_string(), bridge_path.to_string());
    }

    /// 从环境变量自动配置桥接路径
    /// 查找 LINK_PROCESS_BRIDGE_<LANG> 格式的环境变量
    pub fn from_env() -> Self {
        let mut rt = Self::new();
        for lang in &["go", "rust", "csharp", "php", "ruby", "swift", "kotlin"] {
            let env_var = format!("LINK_PROCESS_BRIDGE_{}", lang.to_uppercase());
            if let Ok(path) = std::env::var(&env_var) {
                rt.set_bridge(lang, &path);
            }
        }
        rt
    }

    /// 调用外部函数
    /// 
    /// # 参数
    /// - `language`: 语言标识 (go/rust/csharp/php/ruby/swift/kotlin)
    /// - `module`: 模块/类名
    /// - `func_name`: 函数名
    /// - `args`: 参数列表
    /// - `ret_type`: 期望返回类型
    pub fn call_func(
        &self,
        language: &str,
        module: &str,
        func_name: &str,
        args: &[Value],
        ret_type: Option<&TypeAnnotation>,
    ) -> Result<Value, String> {
        let bridge = self.bridges.get(language)
            .ok_or_else(|| format!("No bridge script for language: '{}'. Set it with set_bridge() or env LINK_PROCESS_BRIDGE_{}", language, language.to_uppercase()))?;

        // 构造 JSON 请求
        let json_args: Vec<serde_json::Value> = args.iter()
            .map(|v| value_to_json(v))
            .collect();
        
        let request = serde_json::json!({
            "module": module,
            "function": func_name,
            "args": json_args,
        });

        // 执行命令(通过 stdin 传递 JSON)
        let output = self.execute(language, bridge, &request)?;

        // 解析响应
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("{} execution failed: {}", language, stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().next()
            .ok_or_else(|| format!("No output from {}", language))?;
        
        let response: serde_json::Value = serde_json::from_str(line)
            .map_err(|e| format!("Invalid JSON from {}: {}", language, e))?;

        // 检查是否有错误
        if let Some(obj) = response.as_object() {
            if let Some(error) = obj.get("error") {
                return Err(format!("{} function error: {}", language, error.as_str().unwrap_or("unknown")));
            }
            if let Some(result) = obj.get("result") {
                return json_to_value(result, ret_type);
            }
        }

        json_to_value(&response, ret_type)
    }

    fn execute(
        &self,
        language: &str,
        bridge: &str,
        request: &serde_json::Value,
    ) -> Result<std::process::Output, String> {
        let req_str = request.to_string();

        let mut cmd = self.build_command(language, bridge)?;

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn {} process: {}", language, e))?;

        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(req_str.as_bytes())
                .map_err(|e| format!("Failed to write to {} stdin: {}", language, e))?;
        }

        child.wait_with_output()
            .map_err(|e| format!("Failed to wait for {} process: {}", language, e))
    }

    fn build_command(&self, language: &str, bridge: &str) -> Result<Command, String> {
        let ext = std::path::Path::new(bridge)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // 如果 bridge 文件的扩展名与 language 不匹配,优先使用扩展名推断
        let lang_matches_ext = match ext.as_str() {
            "go" => language == "go",
            "rs" => language == "rust",
            "cs" => language == "csharp" || language == "dotnet",
            "php" => language == "php",
            "rb" => language == "ruby",
            "swift" => language == "swift",
            "kt" => language == "kotlin",
            "py" => language == "python",
            _ => true,
        };

        // Rust .rs 文件:先编译再执行
        if ext == "rs" {
            let output_dir = std::env::temp_dir();
            let bin_path = output_dir.join(format!("link_bridge_{}", std::process::id()));
            let mut c = Command::new("rustc");
            c.arg(bridge).arg("-o").arg(&bin_path);
            let compile_output = c.output()
                .map_err(|e| format!("Failed to compile rust bridge: {}", e))?;
            if !compile_output.status.success() {
                let stderr = String::from_utf8_lossy(&compile_output.stderr);
                return Err(format!("Rust bridge compilation failed: {}", stderr));
            }
            return Ok(Command::new(&bin_path));
        }

        // Kotlin .kt 文件:先编译为 jar 再执行
        if ext == "kt" {
            let output_dir = std::env::temp_dir();
            let jar_path = output_dir.join(format!("link_kotlin_bridge_{}.jar", std::process::id()));
            let mut c = Command::new("kotlinc");
            c.arg(bridge).arg("-include-runtime").arg("-d").arg(&jar_path);
            let compile_output = c.output()
                .map_err(|e| format!("Failed to compile kotlin bridge: {}", e))?;
            if !compile_output.status.success() {
                let stderr = String::from_utf8_lossy(&compile_output.stderr);
                return Err(format!("Kotlin bridge compilation failed: {}", stderr));
            }
            let mut cmd = Command::new("java");
            cmd.arg("-jar").arg(&jar_path);
            return Ok(cmd);
        }

        // 根据文件扩展名确定 (program, args) — 优先使用扩展名推断
        let (program, args): (&str, Vec<&str>) = match ext.as_str() {
            "py" => ("python", vec![bridge]),
            "rb" => ("ruby", vec![bridge]),
            "php" => ("php", vec![bridge]),
            "go" => ("go", vec!["run", bridge]),
            "rs" => ("rustc", vec![bridge]),
            "swift" => ("swift", vec![bridge]),
            "kt" => ("kotlinc", vec![bridge]),
            "cs" => {
                if language == "csharp" || language == "dotnet" {
                    ("dotnet", vec!["run", "--project", bridge, "--"])
                } else {
                    ("dotnet", vec!["script", bridge])
                }
            }
            // 如果扩展名与 language 匹配,使用 language 的默认命令
            _ if lang_matches_ext => {
                match language {
                    "go" => ("go", vec!["run", bridge]),
                    "rust" => ("rustc", vec![bridge]),
                    "csharp" | "dotnet" => ("dotnet", vec!["run", "--project", bridge, "--"]),
                    "php" => ("php", vec![bridge]),
                    "ruby" => ("ruby", vec![bridge]),
                    "swift" => ("swift", vec![bridge]),
                    "kotlin" => ("java", vec!["-jar", bridge]),
                    "python" => ("python", vec![bridge]),
                    _ => return Ok(Command::new(bridge)),
                }
            }
            // 默认:直接执行(假设 bridge 是已编译的可执行文件)
            _ => return Ok(Command::new(bridge)),
        };

        // 检查是否有自定义命令覆盖(仅在用户显式设置时使用)
        if let Some(cmd_override) = self.commands.get(language) {
            let mut cmd = Command::new(cmd_override);
            for arg in &args {
                cmd.arg(arg);
            }
            return Ok(cmd);
        }

        let mut cmd = Command::new(program);
        for arg in &args {
            cmd.arg(arg);
        }
        Ok(cmd)
    }
}

pub fn value_to_json(value: &Value) -> serde_json::Value {
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

pub fn json_to_value(json: &serde_json::Value, _expected_type: Option<&TypeAnnotation>) -> Result<Value, String> {
    match json {
        serde_json::Value::Null => Ok(Value::None),
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err("Invalid number from process".to_string())
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