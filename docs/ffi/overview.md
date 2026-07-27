# 多语言互联概述

!!! abstract "核心定位"
    多语言互联是 Link 的灵魂特性。Link 不是孤立的语言,而是连接其他语言的胶水层。

## 设计哲学

传统多语言协作的痛点:

| 方案 | 问题 |
|------|------|
| RPC / IPC | 序列化开销、网络延迟、错误处理复杂 |
| SWIG / Protobuf | 是工具而非语言,胶水代码散落各处 |
| 嵌入式解释器(如 Lua) | 只能调用宿主,不能反过来 |
| 单语言 | 强迫所有逻辑用同一种语言,不现实 |

Link 的答案:**把"互联"做成一等语言特性**。

```link
extern "C"      module "libc"      { fn abs(n: i32) -> i32; }
extern "python" module "math"      { fn sqrt(x: f64) -> f64; }
extern "wasm"    module "mod.wasm"  { fn add(a: i32, b: i32) -> i32; }
extern "go"      module "bridge.go" { fn greet(name: str) -> str; }
```

- 一门语言,统一类型系统
- 编译期检查所有跨语言调用签名
- 运行时直接走 C ABI / libpython / wasmtime / 子进程桥接
- `stream<T>` 让数据自动跨语言流动

## extern 块语法

```link
extern "<language>" [module "<module_spec>"] {
    fn <name>(<params>) -> <return_type>;
    fn <name>(<params>) -> <return_type>;
    ...
}
```

### language

Link 支持全球 12 种编程语言的 FFI 互操作:

| 语言 | 关键字 | 别名 | 调用方式 | 说明 |
|------|--------|------|----------|------|
| C | `"C"` | `"c"` | C ABI | 调用 C 标准库或自定义 C 库 |
| C++ | `"C++"` | `"cpp"` | C ABI | 调用 C++ 共享库(需 `extern "C"` 导出) |
| Python | `"python"` | `"py"` | CPython C API | 调用 Python 模块 |
| WebAssembly | `"wasm"` | — | wasmtime 运行时 | 加载 `.wasm` 模块 |
| Java | `"java"` | — | 子进程 + JSON-RPC | 调用 Java 静态方法 |
| HTML / JS | `"html"` | `"js"` | HTTP POST | 调用 Node.js 服务器函数 |
| Go | `"go"` | — | 子进程桥接 | `go run bridge.go` |
| Rust | `"rust"` | — | 子进程桥接 | 自动编译 `.rs` → 可执行文件 |
| C# / .NET | `"csharp"` | `"dotnet"` | 子进程桥接 | `dotnet run` |
| PHP | `"php"` | — | 子进程桥接 | `php bridge.php` |
| Ruby | `"ruby"` | — | 子进程桥接 | `ruby bridge.rb` |
| Swift | `"swift"` | — | 子进程桥接 | `swift bridge.swift` |
| Kotlin | `"kotlin"` | — | 子进程桥接 | 自动编译 `.kt` → JAR |

### module

`module` 字段含义因语言而异:

| 语言 | module 含义 | 示例 |
|------|------------|------|
| C | 库名(系统库或 DLL/SO 路径) | `"c"` / `"m"` / `"mylib.dll"` |
| C++ | DLL/SO 文件路径 | `"engine.dll"` / `"libengine.so"` |
| Python | Python 模块名 | `"math"` / `"os"` / `"json"` |
| WebAssembly | `.wasm` 文件路径 | `"module.wasm"` |
| Java | `class_path::class_name` | `"build::com.example.Math"` |
| HTML / JS | HTTP 端点地址 | `"http://127.0.0.1:3000"` |
| Go / Rust / C# 等 | 桥接脚本路径 | `"bridge.go"` / `"bridge.rs"` |

### 函数签名

每个 `fn` 声明遵循 Link 函数语法:

```link
fn <name>(<param_name>: <type>, ...) -> <return_type>;
```

!!! info "签名只是声明"
    `extern` 块中的 `fn` 没有函数体,只是一个**签名声明**,告诉 Link 如何调用外部函数。
    真正的实现在外部库中。

## export 块(规划中)

`export` 是 `extern` 的反向:把 Link 函数导出供其他语言调用。

```link
export "C" {
    fn my_func(n: i32) -> i32;
}

fn my_func(n: i32) -> i32 {
    n * 2
}
```

v0.1 中 `export` 仅作为语法占位,运行时不执行操作。Phase 1.6 会实现 C 头文件生成。

## 支持矩阵

### 类型映射

| Link 类型 | C 类型 | Python 类型 | C++ 类型 | WASM 类型 | 进程桥接(JSON) |
|-----------|--------|-------------|----------|-----------|----------------|
| `i32` | `int32_t` | `int` | `int32_t` | `i32` | JSON number |
| `i64` | `int64_t` | `int` | `int64_t` | `i64` | JSON number |
| `f32` | `float` | `float` | `float` | `f32` | JSON number |
| `f64` | `double` | `float` | `double` | `f64` | JSON number |
| `bool` | `bool` | `bool` | `bool` | `i32` (0/1) | JSON boolean |
| `str` | `const char*` | `str` | `const char*` | `i32` (ptr) | JSON string |
| `none` | `void` / `NULL` | `None` | `void` | — | JSON null |
| `list` | — | `list` | — | — | JSON array |

### FFI 调用方式分类

| 分类 | 语言 | 调用方式 | 性能 |
|------|------|----------|------|
| **零开销** | C / C++ | C ABI 直接调用 | 最快,无序列化 |
| **低开销** | Python | CPython C API | 有 Python 解释器开销 |
| **运行时** | WebAssembly | wasmtime JIT | 接近原生性能 |
| **子进程** | Java | 子进程 + JSON-RPC | 有进程启动开销 |
| **HTTP** | HTML / JS | HTTP POST | 有网络开销 |
| **子进程桥接** | Go / Rust / C# / PHP / Ruby / Swift / Kotlin | 子进程 + JSON stdin/stdout | 有进程启动开销 |

### 支持的签名

v0.1 支持的函数签名(参数数量 × 返回类型):

| 参数数量 | 返回类型 | C/C++ | Python | WASM | 进程桥接 |
|---------|---------|-------|--------|------|----------|
| 0 | `i32` / `i64` / `f64` / `str` / `bool` | :material-check:{.green} | :material-check:{.green} | :material-check:{.green} | :material-check:{.green} |
| 1 | `i32` / `i64` / `f32` / `f64` / `str` / `bool` | :material-check:{.green} | :material-check:{.green} | :material-check:{.green} | :material-check:{.green} |
| 2 | `i32` / `i64` / `f64` | :material-check:{.green} | :material-check:{.green} | :material-check:{.green} | :material-check:{.green} |
| 3 | `i32` / `f64`(int 参数也可) | :material-check:{.green} | :material-check:{.green} | :material-check:{.green} | :material-check:{.green} |
| 任意 | 任意基础类型 | :material-close:{.red} 规划中 | :material-check:{.green} | :material-check:{.green} | :material-check:{.green} |

!!! note "扩展计划"
    后续版本会支持任意参数数量、struct 传参、回调函数等高级特性。

## 性能特点

- **C / C++**:通过 C ABI 直接调用,无运行时开销(零序列化、零拷贝)
- **Python**:通过 CPython C API 调用,有 Python 解释器开销,但比 RPC 快得多
- **WebAssembly**:通过 wasmtime JIT 编译,接近原生性能
- **Java / 进程桥接语言**:有子进程启动开销,适合批量调用而非高频单次调用
- **HTML / JS**:有 HTTP 网络开销,适合 Web 服务场景
- **类型转换**:基础类型(int/float/bool)无开销,str 需 UTF-8 转换

## 进程桥接协议

Go / Rust / C# / PHP / Ruby / Swift / Kotlin 通过统一的 JSON 协议与 Link 通信:

```
Link 进程                    桥接脚本
   │                           │
   ├── JSON 请求 (stdin) ────→ │
   │   {                       │
   │     "module": "...",      │
   │     "function": "add",    │
   │     "args": [1, 2]        │
   │   }                       │
   │                           ├── 执行函数
   │                           │
   │ ←── JSON 响应 (stdout) ──┤
   │   {"result": 3}           │
   │                           │
```

### 桥接脚本示例

每种语言需要提供一个桥接脚本,从 stdin 读取 JSON 请求,执行函数后输出 JSON 响应:

=== "Go"

    ```go
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

=== "Python"

    ```python
    import sys, json

    data = json.loads(sys.stdin.read())
    fn = data.get("function", "")
    args = data.get("args", [])

    if fn == "add":
        result = args[0] + args[1]
    elif fn == "greet":
        result = f"Hello, {args[0]}!"

    print(json.dumps({"result": result}))
    ```

=== "Ruby"

    ```ruby
    require 'json'

    data = JSON.parse(STDIN.read)
    fn = data["function"]
    args = data["args"]

    case fn
    when "add"
      result = args[0] + args[1]
    when "greet"
      result = "Hello, #{args[0]}!"
    end

    puts JSON.generate({"result" => result})
    ```

### 环境变量配置

可以通过环境变量配置桥接脚本路径:

```bash
# 设置 Go 桥接脚本
export LINK_PROCESS_BRIDGE_GO=/path/to/bridge.go

# 设置 Ruby 桥接脚本
export LINK_PROCESS_BRIDGE_RUBY=/path/to/bridge.rb
```

或在 Link 代码中通过 `module` 指定:

```link
extern "go" module "/path/to/bridge.go" {
    fn add(a: i64, b: i64) -> i64;
}
```

## 各语言详细文档

- [C 互连](c.md) — 调用 C 标准库与自定义 C 库
- [Python 互连](python.md) — 调用 Python 标准库与第三方库
- [C++ 互连](cpp.md) — 通过 C ABI 调用 C++ 共享库
- [WebAssembly 互连](wasm.md) — 加载并调用 WASM 模块
- [Java 互连](java.md) — 通过子进程调用 Java 静态方法
- [HTML/JS 互连](html.md) — 通过 HTTP 调用 JavaScript 函数
- [进程桥接互连](process.md) — Go / Rust / C# / PHP / Ruby / Swift / Kotlin

## 综合示例

```link
extern "C" {
    fn abs(n: i32) -> i32;
}

extern "python" module "math" {
    fn sqrt(x: f64) -> f64;
    fn pow(base: f64, exp: f64) -> f64;
}

extern "C++" module "examples/cpp_demo.dll" {
    fn cpp_factorial(n: i32) -> i32;
    fn cpp_greet(name: str) -> str;
}

extern "wasm" module "module.wasm" {
    fn add(a: i32, b: i32) -> i32;
}

extern "go" module "bridge.go" {
    fn greet(name: str) -> str;
}

// 混合使用多种语言
fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = abs(x2 - x1) as f64;
    let dy = abs(y2 - y1) as f64;
    sqrt(pow(dx, 2.0) + pow(dy, 2.0))
}

let d = distance(0.0, 0.0, 3.0, 4.0);
println("距离 =", d);                       // 5

let fact = cpp_factorial(5);
let msg = cpp_greet("Link");
println("5! =", fact);                       // 120
println("问候:", msg);                       // Hello, Link! from C++

let w = add(10, 20);
println("wasm add(10, 20) =", w);            // 30

let g = greet("World");
println("go greet:", g);                     // Hello, World!

// stream<T> + 管道运算符
let result = stream([1, 2, 3, 4, 5])
    | map(fn(x) -> i64 { return x * 2; })
    | filter(fn(x) -> bool { return x > 5; })
    | collect();
println("stream result:", result);           // [6, 8, 10]
```

## 安全性注意

!!! warning "FFI 是 unsafe 的"
    Link FFI 直接调用外部代码,不做内存安全检查。错误的签名或调用约定可能导致:
    
    - 段错误(内存越界)
    - 内存泄漏
    - 调用栈损坏
    
    请确保 `extern` 声明的签名与外部函数实际签名**完全一致**。

### 常见陷阱

1. **C++ 名称修饰**:C++ 函数未用 `extern "C"` 导出时,符号名会被修饰,Link 找不到
2. **架构不匹配**:64 位 Link 加载 32 位 DLL 会失败
3. **Python GIL**:Python FFI 调用会自动获取 GIL,但长时间运行可能阻塞其他 Python 线程
4. **字符串所有权**:C/C++ 返回的 `const char*` 必须指向静态或堆内存,不能是栈上临时变量
5. **进程桥接超时**:子进程调用可能因脚本错误而挂起,确保桥接脚本正确输出 JSON 后退出
6. **路径问题**:桥接脚本路径可以是绝对路径或相对路径,相对路径基于 Link 进程工作目录

## 下一步

- [C 互连](c.md)
- [Python 互连](python.md)
- [C++ 互连](cpp.md)
- [WebAssembly 互连](wasm.md)
- [Java 互连](java.md)
- [HTML/JS 互连](html.md)
- [进程桥接互连](process.md)
