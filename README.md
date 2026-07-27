# Link

A language for connecting everything — from IoT devices to game servers to multi-language glue.

[![Documentation](https://img.shields.io/badge/docs-myiunagn.github.io-blue)](https://myiunagn.github.io/linkdoc/)
[![License: MIT](https://img.shields.io/badge/license-MIT-green)](LICENSE)

**Link** 是一门为"互联"而生的语言:从 IoT 设备到游戏后端,再到多语言胶水层。
当前 v0.1 已支持基本类型、控制流、函数、列表,以及 **C / Python / C++ 三向 FFI 互联**。

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

## Features

- 基本类型:`int` / `float` / `str` / `bool` / `none` / `list`
- 控制流:`if/else` / `while` / `for` / `loop` / `break` / `continue`
- 函数:声明、递归、闭包作用域
- 内置函数:`print` / `println` / `len`
- 多语言互联:
    - C 标准库(`libc` / `msvcrt`)
    - Python 标准库(`math` / `os` / 任意模块)
    - C++ 共享库(通过 `extern "C"` 桥接)
- 运行模式:`link run <file>` 文件执行,`link repl` 交互式 REPL

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

println("abs(-42) =", abs(-42));
println("math.sqrt(16.0) =", sqrt(16.0));
println("cpp_factorial(5) =", cpp_factorial(5));
```

## Project Structure

```
link/
├── crates/
│   ├── linkc_lexer/        # 词法分析
│   ├── linkc_parser/       # 语法分析
│   ├── linkc_interpreter/  # 解释器(含 FFI)
│   └── linkc_cli/          # 命令行入口
├── docs/                   # MkDocs 文档源
├── examples/               # C++ 示例库等
├── editors/vscode/         # VS Code 扩展
├── tests/fixtures/         # Link 测试用例
├── mkdocs.yml              # 文档配置
└── requirements-docs.txt   # 文档依赖
```

## License

MIT

