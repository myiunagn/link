# 进程桥接互连

!!! abstract "概述"
    Link 通过子进程 + JSON 协议与 7 种编程语言互操作:
    **Go / Rust / C# / PHP / Ruby / Swift / Kotlin**。
    每种语言提供一个桥接脚本,从 stdin 读取 JSON 请求,执行函数后输出 JSON 响应。

## 支持的语言

| 语言 | 关键字 | 执行命令 | 说明 |
|------|--------|----------|------|
| Go | `"go"` | `go run bridge.go` | 直接运行 Go 源文件 |
| Rust | `"rust"` | `rustc bridge.rs` → 执行 | 自动编译后执行 |
| C# / .NET | `"csharp"` / `"dotnet"` | `dotnet run` | 运行 .NET 项目 |
| PHP | `"php"` | `php bridge.php` | 运行 PHP 脚本 |
| Ruby | `"ruby"` | `ruby bridge.rb` | 运行 Ruby 脚本 |
| Swift | `"swift"` | `swift bridge.swift` | 运行 Swift 脚本 |
| Kotlin | `"kotlin"` | `kotlinc bridge.kt` → JAR | 自动编译为 JAR 后执行 |

## 基本用法

### module 字段

`module` 字段指定桥接脚本路径:

```link
extern "go" module "bridge.go" {
    fn add(a: i64, b: i64) -> i64;
    fn greet(name: str) -> str;
}

extern "ruby" module "bridge.rb" {
    fn transform(input: i64) -> i64;
}
```

### 调用示例

```link
extern "go" module "bridge.go" {
    fn add(a: i64, b: i64) -> i64;
    fn greet(name: str) -> str;
}

let sum = add(10, 20);        // 30
let msg = greet("World");     // Hello, World!
```

## 类型映射

所有进程桥接语言使用统一的 JSON 协议:

| Link 类型 | JSON 类型 | 说明 |
|-----------|-----------|------|
| `i32` / `i64` | number | JSON 数字 |
| `f32` / `f64` | number | JSON 数字 |
| `bool` | boolean | JSON 布尔 |
| `str` | string | JSON 字符串 |
| `none` | null | JSON null |
| `list` | array | JSON 数组 |

## 通信协议

### 请求格式(Link → 桥接脚本)

Link 通过 stdin 发送 JSON 请求:

```json
{
    "module": "bridge",
    "function": "add",
    "args": [10, 20]
}
```

### 响应格式(桥接脚本 → Link)

桥接脚本通过 stdout 输出 JSON 响应:

```json
{
    "result": 30
}
```

### 错误响应

如果函数执行出错,返回错误信息:

```json
{
    "error": "Division by zero"
}
```

## 桥接脚本示例

### Go

```go
// bridge.go
package main

import (
    "encoding/json"
    "os"
)

func main() {
    var req map[string]interface{}
    json.NewDecoder(os.Stdin).Decode(&req)

    fn := req["function"].(string)
    args := req["args"].([]interface{})

    var result interface{}
    switch fn {
    case "add":
        result = int64(args[0].(float64)) + int64(args[1].(float64))
    case "greet":
        result = "Hello, " + args[0].(string) + "!"
    }

    json.NewEncoder(os.Stdout).Encode(map[string]interface{}{
        "result": result,
    })
}
```

### Rust

```rust
// bridge.rs
use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let req: serde_json::Value = serde_json::from_str(&input).unwrap();
    let fn_name = req["function"].as_str().unwrap();
    let args = req["args"].as_array().unwrap();

    let result = match fn_name {
        "add" => {
            let a = args[0].as_i64().unwrap();
            let b = args[1].as_i64().unwrap();
            a + b
        }
        "greet" => {
            format!("Hello, {}!", args[0].as_str().unwrap())
        }
        _ => panic!("Unknown function"),
    };

    println!("{}", serde_json::json!({"result": result}));
}
```

### PHP

```php
<?php
// bridge.php
$input = file_get_contents('php://stdin');
$req = json_decode($input, true);

$fn = $req['function'];
$args = $req['args'];

switch ($fn) {
    case 'add':
        $result = $args[0] + $args[1];
        break;
    case 'greet':
        $result = "Hello, {$args[0]}!";
        break;
    default:
        echo json_encode(['error' => "Unknown function: $fn"]);
        exit(1);
}

echo json_encode(['result' => $result]);
```

### Ruby

```ruby
# bridge.rb
require 'json'

input = STDIN.read
req = JSON.parse(input)

fn = req['function']
args = req['args']

case fn
when 'add'
  result = args[0] + args[1]
when 'greet'
  result = "Hello, #{args[0]}!"
else
  puts JSON.generate({'error' => "Unknown function: #{fn}"})
  exit(1)
end

puts JSON.generate({'result' => result})
```

### Swift

```swift
// bridge.swift
import Foundation

let input = FileHandle.standardInput.readDataToEndOfFile()
let req = try! JSONSerialization.jsonObject(with: input) as! [String: Any]

let fn = req["function"] as! String
let args = req["args"] as! [Any]

var result: Any
switch fn {
case "add":
    result = (args[0] as! Int) + (args[1] as! Int)
case "greet":
    result = "Hello, \(args[0])!"
default:
    fatalError("Unknown function")
}

let response = ["result": result]
let data = try! JSONSerialization.data(withJSONObject: response)
print(String(data: data, encoding: .utf8)!)
```

### Kotlin

```kotlin
// bridge.kt
import kotlinx.serialization.*
import kotlinx.serialization.json.*

fun main() {
    val input = generateSequence(::readLine).joinToString("\n")
    val req = Json.parseToJsonElement(input).jsonObject
    val fn = req["function"]!!.jsonPrimitive.content
    val args = req["args"]!!.jsonArray

    val result: JsonElement = when (fn) {
        "add" -> JsonPrimitive(args[0].jsonPrimitive.long + args[1].jsonPrimitive.long)
        "greet" -> JsonPrimitive("Hello, ${args[0].jsonPrimitive.content}!")
        else -> throw IllegalArgumentException("Unknown function: $fn")
    }

    println(Json.encodeToString(buildJsonObject { put("result", result) }))
}
```

## 环境变量配置

可通过环境变量配置桥接脚本路径,避免在代码中硬编码:

```bash
# 设置各语言的桥接脚本路径
export LINK_PROCESS_BRIDGE_GO=/path/to/bridge.go
export LINK_PROCESS_BRIDGE_RUST=/path/to/bridge.rs
export LINK_PROCESS_BRIDGE_CSHARP=/path/to/bridge.cs
export LINK_PROCESS_BRIDGE_PHP=/path/to/bridge.php
export LINK_PROCESS_BRIDGE_RUBY=/path/to/bridge.rb
export LINK_PROCESS_BRIDGE_SWIFT=/path/to/bridge.swift
export LINK_PROCESS_BRIDGE_KOTLIN=/path/to/bridge.kt
```

设置后,Link 代码中可省略 `module`:

```link
extern "go" {
    fn add(a: i64, b: i64) -> i64;
}
```

## 自动编译

Rust 和 Kotlin 桥接脚本支持自动编译:

- **Rust (`.rs`)**:Link 自动调用 `rustc` 编译为临时可执行文件,然后执行
- **Kotlin (`.kt`)**:Link 自动调用 `kotlinc` 编译为临时 JAR,然后通过 `java -jar` 执行

其他语言(Go / PHP / Ruby / Swift)直接使用解释器/编译器运行,无需额外编译步骤。

## 完整示例

```link
// 多语言进程桥接示例
extern "go" module "bridge.go" {
    fn add(a: i64, b: i64) -> i64;
    fn greet(name: str) -> str;
}

extern "ruby" module "bridge.rb" {
    fn transform(input: i64) -> i64;
}

extern "php" module "bridge.php" {
    fn process(data: str) -> str;
}

// 组合多语言调用
let go_result = add(10, 20);
let ruby_result = transform(go_result);
let php_result = process("test data");

println("Go add(10, 20) =", go_result);
println("Ruby transform =", ruby_result);
println("PHP process =", php_result);

// 与 stream<T> 结合
let result = stream([1, 2, 3, 4, 5])
    | map(fn(x) -> i64 { return add(x, 100); })
    | collect();
println("stream + Go add:", result);  // [101, 102, 103, 104, 105]
```

## 性能特点

- **子进程开销**:每次调用启动新进程,有启动开销(Go ~100ms, Rust 编译更慢)
- **JSON 序列化**:参数和返回值通过 JSON 传递
- **适合批量调用**:建议将多个操作合并到一次调用中减少进程启动次数

!!! tip "性能优化建议"
    - 对于 Rust/Kotlin,预编译桥接脚本为可执行文件/JAR,直接指定二进制路径
    - 对于高频调用,考虑使用 C FFI 或 WASM FFI 替代
    - 使用环境变量配置桥接路径,便于部署时优化

## 前置条件

| 语言 | 需要安装 | 验证命令 |
|------|---------|----------|
| Go | Go 1.16+ | `go version` |
| Rust | Rust / rustc | `rustc --version` |
| C# | .NET SDK 6.0+ | `dotnet --version` |
| PHP | PHP 8.0+ | `php --version` |
| Ruby | Ruby 3.0+ | `ruby --version` |
| Swift | Swift 5.5+ | `swift --version` |
| Kotlin | Kotlin compiler + JRE | `kotlinc -version` |

## 常见问题

### 进程启动失败

确保对应语言的运行时已安装且在 PATH 中可用。用上表的验证命令检查。

### JSON 解析错误

确保桥接脚本输出的 JSON 格式正确,且只有一行输出(避免多余日志干扰)。

### 桥接脚本未找到

检查 `module` 路径或环境变量是否正确设置。路径可以是绝对路径或相对路径。

## 下一步

- [多语言互联概述](overview.md)
- [WebAssembly 互连](wasm.md)
- [Java 互连](java.md)
