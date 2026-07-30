# Link

A language for connecting everything — from IoT devices to game servers to multi-language glue.

[![Documentation](https://img.shields.io/badge/docs-myiunagn.github.io-blue)](https://myiunagn.github.io/linkdoc/)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Version](https://img.shields.io/badge/version-v1.0.0-blue)](https://github.com/myiunagn/link/releases)

**Link** 是一门为"互联"而生的语言:从 IoT 设备到游戏后端,再到多语言胶水层。

v1.0.0 已支持基本类型、控制流、函数、`stream<T>` 数据流、struct/enum 复合类型、**全球 12 种编程语言 FFI 互联**、**多后端编译器**(C / LLVM / Python / WASM)、**借用检查器**、**LSP 语言服务器**、**自举编译器**和**标准库**。

## 文档

完整的语言手册、安装指南、教程与示例请访问在线文档:

**https://myiunagn.github.io/linkdoc/**

本地预览文档:

```bash
pip install -r requirements-docs.txt
mkdocs serve
```

## Build

```bash
cargo build
cargo run -p linkc_cli -- run tests/fixtures/fib.link
```

## Self-host bootstrap

Link now has a Stage 1 seed compiler written in Link. Rust is used once as the
Stage 0 bootstrap host; the resulting Stage 1 compiler emits portable C and the
generated program does not link Rust.

```powershell
./bootstrap/verify.ps1
```

The current seed covers the integer/control-flow core. See
[`bootstrap/README.md`](bootstrap/README.md) for the supported surface and the
path to a full fixed-point self-host build.

## Features

### 语言核心

- 基本类型:`int` / `float` / `str` / `bool` / `none` / `list`
- 控制流:`if/else` / `while` / `for` / `loop` / `break` / `continue`
- 函数:声明、递归、闭包作用域、`async fn`
- 复合类型:`struct` / `enum` / `match` 模式匹配(5 种 Pattern)
- `stream<T>` 数据流:创建、`map` / `filter` / `for_each` / `collect`、管道运算符 `|`
- `flow` 声明块:声明式数据流定义(source / sample / pipeline)
- `async` / `await`:异步编程支持

### 多语言互联(全球 12 种语言)

- **C** — 标准库(`libc` / `msvcrt`),C ABI 直接调用
- **C++** — 共享库(通过 `extern "C"` 桥接)
- **Python** — 标准库(`math` / `os` / 任意模块),CPython C API
- **WebAssembly** — 加载 `.wasm` 模块,wasmtime 运行时
- **Java** — 子进程 + JSON-RPC 桥接器
- **HTML / JavaScript** — HTTP POST 调用 Node.js 服务器
- **Go** — 子进程桥接(`go run`)
- **Rust** — 子进程桥接(自动编译 `.rs` → 可执行文件)
- **C# / .NET** — 子进程桥接(`dotnet run`)
- **PHP** — 子进程桥接(`php`)
- **Ruby** — 子进程桥接(`ruby`)
- **Swift** — 子进程桥接(`swift`)
- **Kotlin** — 子进程桥接(自动编译 `.kt` → JAR)

### 编译器(v0.2.0 新增)

- **C 后端**:C 代码生成,支持 struct/enum/list/match/字符串拼接
- **LLVM 后端**:LLVM IR 生成,PassManager 优化(条件编译)
- **Python 后端**:生成 Python 代码,支持完整语言特性
- **WASM 后端**:生成 WebAssembly 文本格式(WAT)代码
- **类型检查器**:54 个测试,支持 match 模式变量绑定和枚举变体返回类型推断
- **借用检查器**:所有权跟踪、use-after-move 检测、Copy 类型特殊处理
- **常量折叠优化**:编译期常量表达式求值
- **死代码消除**:return/break/continue 后不可达代码自动删除
- **优化等级**:O0-O3,调试符号支持

### LSP 语言服务器(v0.2.0 新增)

- 实时诊断(集成 lexer/parser/sema)
- 自动补全(关键字 35+ / 内置函数 35+ / 内置类型 16 / 文档符号)
- 悬停提示(函数签名 / 结构体 / 枚举 / 内置函数文档)
- 跳转定义(函数 / 变量 / 结构体 / 枚举 / 模块)
- 文档符号大纲(Function / Struct / Enum / Variable / Module)

### 游戏后端(v0.2.0 新增)

- `domain` 关键字声明语法
- WebSocket 服务器(tokio-tungstenite)
- 房间系统(多房间隔离、动态创建、join/leave/chat)
- 实体系统(玩家 + 道具,含位置/速度/得分/HP)
- 60 FPS 帧同步循环,每帧广播 JSON 快照
- 圆形碰撞检测

### 绑定生成

- `link bindgen --lang python/ts/c` 自动生成绑定代码

### CLI 命令

- `link run <file>` — 执行 Link 文件
- `link repl` — 交互式 REPL
- `link compile <file>` — 编译(支持 `--backend c/llvm/python/wasm`)
- `link bindgen` — 生成多语言绑定
- `link lsp` — 启动语言服务器
- `link game <file>` — 启动游戏服务器

## Quick Start

```link
// 调用 C 标准库 abs
extern "C" { fn abs(n: i32) -> i32; }

// 调用 Python math.sqrt
extern "python" module "math" { fn sqrt(x: f64) -> f64; }

// 调用 C++ 共享库
extern "C++" module "examples/cpp_demo.dll" {
    fn cpp_factorial(n: i32) -> i32;
}

// 调用 WebAssembly 模块
extern "wasm" module "module.wasm" { fn add(a: i32, b: i32) -> i32; }

println("abs(-42) =", abs(-42));
println("math.sqrt(16.0) =", sqrt(16.0));
println("cpp_factorial(5) =", cpp_factorial(5));

// stream<T> 数据流 + 管道运算符
let result = stream([1, 2, 3, 4, 5])
    | map(fn(x) -> i64 { return x * 2; })
    | filter(fn(x) -> bool { return x > 5; })
    | collect();
println(result);  // [6, 8, 10]
```

## Project Structure

```
link/
├── crates/
│   ├── linkc_lexer/        # 词法分析
│   ├── linkc_parser/       # 语法分析
│   ├── linkc_interpreter/  # 解释器(含 FFI)
│   │   ├── src/
│   │   │   ├── lib.rs      # 解释器核心 + stream<T>
│   │   │   ├── python.rs   # Python FFI
│   │   │   ├── wasm.rs     # WebAssembly FFI
│   │   │   ├── java.rs     # Java FFI
│   │   │   ├── html.rs     # HTML/JS FFI
│   │   │   └── process.rs  # 进程桥接 FFI (Go/Rust/C#/PHP/Ruby/Swift/Kotlin)
│   │   └── tests/          # FFI 集成测试
│   ├── linkc_sema/         # 语义分析(类型检查 + 借用检查 + 常量折叠 + 死代码消除)
│   ├── linkc_codegen/      # 代码生成(C / LLVM / Python / WASM 后端)
│   ├── linkc_bindgen/      # 多语言绑定生成器(C / Python / TypeScript)
│   ├── linkc_lsp/          # LSP 语言服务器
│   ├── linkc_llvm/         # LLVM IR 后端(条件编译)
│   └── linkc_cli/          # 命令行入口
├── docs/                   # MkDocs 文档源
├── editors/vscode/         # VS Code 扩展
├── examples/               # 示例代码
├── tests/fixtures/         # 测试用例 + 多语言桥接脚本
├── mkdocs.yml              # 文档配置
└── requirements-docs.txt   # 文档依赖
```

## Testing

```bash
cargo test
```

300+ 单元/集成测试全部通过。

## License

MIT — Copyright (c) 2024 ctost link
