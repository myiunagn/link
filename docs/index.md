---
hide:
  - navigation
  - toc
---

# Link 语言

<p align="center">
  <strong>一门为"互联"而生的语言</strong><br/>
  <sub>从 IoT 设备到游戏后端,再到多语言胶水层</sub>
</p>

---

!!! tip "v0.1 已发布"
    当前版本支持:基本类型、控制流、函数、列表、`stream<T>` 数据流、struct/enum 复合类型、async/await、flow 声明块、**全球 12 种编程语言 FFI 互联**、**C/LLVM 编译器后端**。

## 核心特性

<div class="grid cards" markdown>

- :material-link-variant: **连接为一等公民**

    ---

    `stream<T>` / `endpoint` / `group` 是语言内置类型,不是库里的 class。
    数据从源到汇自动调度,无需手写并发。

- :material-language-python: **多语言原生互通**

    ---

    天然是其他语言的胶水层。一行 `extern` 即可调用全球 12 种编程语言:
    C / C++ / Python / WASM / Java / JS / Go / Rust / C# / PHP / Ruby / Swift / Kotlin。

- :material-code-braces: **静态类型 + 声明式**

    ---

    类型严谨如 C++,声明优先如 Terraform。描述"要什么",不描述"怎么做"。

- :material-flash: **原生码编译**

    ---

    支持 C 代码生成和 LLVM IR 后端。O0-O3 优化等级,调试符号支持。无 GC,无运行时开销。

</div>

## 30 秒体验

```link
// 调用 C 标准库 abs
extern "C" {
    fn abs(n: i32) -> i32;
}

// 调用 Python 标准库 math.sqrt
extern "python" module "math" {
    fn sqrt(x: f64) -> f64;
}

// 调用 C++ 共享库 cpp_demo.dll
extern "C++" module "cpp_demo.dll" {
    fn cpp_factorial(n: i32) -> i32;
}

// 调用 WebAssembly 模块
extern "wasm" module "module.wasm" {
    fn add(a: i32, b: i32) -> i32;
}

// 调用 Go 桥接脚本
extern "go" module "bridge.go" {
    fn greet(name: str) -> str;
}

// 调用 Java 静态方法
extern "java" module "build/classes::com.example.Math" {
    fn factorial(n: i64) -> i64;
}

let x = abs(-42);
let y = sqrt(16.0);
let f = cpp_factorial(5);
let w = add(3, 4);
let g = greet("Link");
let j = factorial(5);

println("C abs(-42)           =", x);
println("Python sqrt(16)      =", y);
println("C++ factorial(5)     =", f);
println("WASM add(3, 4)       =", w);
println("Go greet(\"Link\")     =", g);
println("Java factorial(5)    =", j);

// stream<T> 数据流 + 管道运算符
let result = stream([1, 2, 3, 4, 5])
    | map(fn(x) -> i64 { return x * 2; })
    | filter(fn(x) -> bool { return x > 5; })
    | collect();
println("stream result        =", result);  // [6, 8, 10]
```

输出:

```
C abs(-42)           = 42
Python sqrt(16)      = 4
C++ factorial(5)     = 120
WASM add(3, 4)       = 7
Go greet("Link")     = Hello, Link!
Java factorial(5)    = 120
stream result        = [6, 8, 10]
```

## 立即开始

<div class="grid cards" markdown>

- :material-download: **[安装](installation.md)**

    ---

    一行 `cargo build` 即可从源码构建。

- :material-rocket-launch: **[快速开始](quickstart.md)**

    ---

    5 分钟写出第一个 Link 程序。

- :material-console: **[REPL](repl.md)**

    ---

    交互式探索语法,无需写文件。

- :material-translate: **[多语言互联](ffi/overview.md)**

    ---

    Link 的灵魂特性。一行 `extern` 调用全球 12 种编程语言: C / C++ / Python / WASM / Java / JS / Go / Rust / C# / PHP / Ruby / Swift / Kotlin。

</div>

## 设计哲学

1. **连接为一等公民** —— `stream<T>` / `endpoint` / `group` 是语言内置类型,不是库里的 class
2. **流是默认执行模型** —— 数据从源到汇自动调度,无需手写并发
3. **声明优先,无副作用** —— 描述"要什么",不描述"怎么做"
4. **多语言原生互通** —— Link 不孤立存在,天然是其他语言的胶水层

## 项目状态

| 里程碑 | 状态 | 说明 |
|--------|------|------|
| Phase 0:骨架 | :material-check-circle:{.green} 完成 | Lexer / Parser / Interpreter |
| Phase 1.1:C FFI | :material-check-circle:{.green} 完成 | 动态加载 libc/msvcrt,调用 `abs` / `sqrt` |
| Phase 1.2:Python/C++ FFI | :material-check-circle:{.green} 完成 | libpython 动态加载,C++ via C ABI |
| Phase 1.3:stream<T> | :material-check-circle:{.green} 完成 | 数据流核心类型 + 管道运算符 `\|` |
| Phase 1.4:多语言 FFI | :material-check-circle:{.green} 完成 | WASM / Java / HTML/JS / Go / Rust / C# / PHP / Ruby / Swift / Kotlin |
| Phase 1.5:struct/enum | :material-check-circle:{.green} 完成 | 复合类型与模式匹配 |
| Phase 1.6:bindgen | :material-check-circle:{.green} 完成 | C/Python/TypeScript 绑定生成 |
| Phase 1.7:flow 块 | :material-check-circle:{.green} 完成 | 声明式数据流处理 |
| Phase 1.8:async/await | :material-check-circle:{.green} 完成 | 异步编程与 sleep 原语 |
| Phase 2.1:C 后端 | :material-check-circle:{.green} 完成 | C 代码生成,优化等级 O0-O3 |
| Phase 2.2:LLVM 后端 | :material-check-circle:{.green} 完成 | LLVM IR 生成,优化通道 |

## 许可证

MIT
