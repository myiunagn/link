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
    当前版本支持:基本类型、控制流、函数、列表,**C / Python / C++ 三向 FFI 互联**。

## 核心特性

<div class="grid cards" markdown>

- :material-link-variant: **连接为一等公民**

    ---

    `stream<T>` / `endpoint` / `group` 是语言内置类型,不是库里的 class。
    数据从源到汇自动调度,无需手写并发。

- :material-language-python: **多语言原生互通**

    ---

    天然是其他语言的胶水层。一行 `extern "python"` 即可调用 Python 标准库,
    一行 `extern "C++"` 即可加载 C++ DLL/SO。

- :material-code-braces: **静态类型 + 声明式**

    ---

    类型严谨如 C++,声明优先如 Terraform。描述"要什么",不描述"怎么做"。

- :material-flash: **原生码编译**

    ---

    Rust + LLVM 工具链,编译为原生可执行文件。无 GC,无运行时开销。

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
    fn cpp_greet(name: str) -> str;
}

let x = abs(-42);
let y = sqrt(16.0);
let f = cpp_factorial(5);
let g = cpp_greet("Link");

println("abs(-42)        =", x);
println("math.sqrt(16.0) =", y);
println("cpp_factorial(5) =", f);
println("cpp_greet(\"Link\") =", g);
```

输出:

```
abs(-42)        = 42
math.sqrt(16.0) = 4
cpp_factorial(5) = 120
cpp_greet("Link") = Hello, Link! from C++
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

    Link 的灵魂特性。看它如何把 C、Python、C++ 拼成一体。

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
| Phase 1.3:stream<T> | :material-clock-outline:{.yellow} 规划中 | 数据流核心类型 |
| Phase 2:LLVM 后端 | :material-clock-outline:{.yellow} 规划中 | 原生码编译 |

## 许可证

MIT
