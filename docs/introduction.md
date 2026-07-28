# 介绍

## Link 是什么

**Link 是一门专用于"互联"的静态类型声明式数据流语言。**

- 以多语言互联为第一切入点
- 延展到游戏后端与 IoT 设备互联
- 多后端编译: C / LLVM / Python / WASM
- 当前 v0.2.0 版本,支持完整编译、借用检查、LSP 语言服务器

## 一句话定位

> SQL 之于数据库,Link 之于互联。

| 维度 | 类比 |
|------|------|
| 领域定位 | SQL 之于数据库,Link 之于互联 |
| 范式 | Terraform 的声明式 + 流式计算 + C++ 的类型严谨 |
| 编译 | Rust 的 LLVM 路径 |
| 语言互联 | 类似 SWIG / Protobuf,但作为一等语言而非工具 |

## 为什么需要 Link

### 问题:多语言协作的痛点

现代工程普遍存在多语言协作场景:

- **后端**:Go/Rust/Java 写业务,C/C++ 写性能模块,Python 写算法/ML
- **游戏**:C++ 引擎 + Lua 脚本 + Python 工具链 + Go 后端
- **IoT**:C 固件 + Python 上位机 + JS 配网

现有方案的问题:

1. SWIG / Protobuf 是**工具**而非语言,胶水代码散落各处
2. 不同语言间通信要起 RPC / IPC,有序列化开销
3. 类型系统割裂,跨语言重构困难

### Link 的答案

把"互联"做成**一等语言特性**:

```link
extern "C"      { fn abs(n: i32) -> i32; }
extern "python" module "math" { fn sqrt(x: f64) -> f64; }
extern "C++"    module "engine.dll" { fn render() -> i32; }
```

- 一门语言,统一类型系统
- 编译期检查所有跨语言调用签名
- 运行时直接走 C ABI / libpython,零序列化开销
- 未来 `stream<T>` 让数据自动跨语言流动

## 核心抽象

| 概念 | 说明 | 状态 |
|------|------|------|
| `stream<T>` | 数据流,Link 的灵魂 | :material-check-circle:{.green} 已实现 |
| `endpoint` | 连接端点(设备/玩家/服务) | :material-clock-outline:{.yellow} 规划中 |
| `group` | 群组(房间/网关/集群) | :material-clock-outline:{.yellow} 规划中 |
| `extern` / `export` | 多语言互操作 | :material-check-circle:{.green} 已实现 |
| `domain` | 游戏后端域类型 | :material-check-circle:{.green} 已实现 |
| 借用检查器 | 所有权与内存安全 | :material-check-circle:{.green} 已实现 |
| LSP 语言服务器 | 编辑器智能支持 | :material-check-circle:{.green} 已实现 |

## 当前版本能做什么

v0.2.0 已实现:

- 基本类型:`i8`/`i16`/`i32`/`i64`/`u8`/`u16`/`u32`/`u64`/`usize` / `f32`/`f64` / `str` / `bool` / `none` / `list<T>`
- 运算符:算术、比较、逻辑、管道运算符 `|`
- 控制流:`if/else` / `while` / `for` / `loop` / `break` / `continue`
- 函数:声明、递归、闭包作用域
- 内置函数:`print` / `println` / `len`
- 列表:字面量、索引、嵌套
- **复合类型**:struct / enum + match 模式匹配
- **数据流**: `stream<T>` + 管道运算符
- **声明式**: `flow` 声明块
- **异步编程**: async/await
- **FFI**:
    - 全球 12 种编程语言: C / C++ / Python / WASM / Java / JS / Go / Rust / C# / PHP / Ruby / Swift / Kotlin
    - bindgen: C/Python/TypeScript 绑定生成
- **编译后端**:
    - C 后端:完整 C 代码生成,优化等级 O0-O3
    - LLVM 后端: LLVM IR 生成(条件编译)
    - Python 后端: 生成 Python 代码
    - WASM 后端: 生成 WAT 格式代码
- **优化**: 常量折叠、死代码消除
- **借用检查器**: 所有权跟踪、移动语义、Copy 类型、use-after-move 检测
- **LSP 语言服务器**: 诊断、补全、悬停、跳转、符号大纲
- **游戏后端域类型**: domain 语法、WebSocket、房间系统、帧同步
- 运行模式:`link run <file>` 文件执行,`link repl` 交互式 REPL,`link compile` 编译

## 设计哲学

1. **连接为一等公民** —— `stream<T>` / `endpoint` / `group` 是语言内置类型,不是库里的 class
2. **流是默认执行模型** —— 数据从源到汇自动调度,无需手写并发
3. **声明优先,无副作用** —— 描述"要什么",不描述"怎么做"
4. **多语言原生互通** —— Link 不孤立存在,天然是其他语言的胶水层

## 路线图

| 阶段 | 内容 | 状态 |
|------|------|------|
| Phase 0 | 骨架:Lexer + Parser + Interpreter | :material-check-circle:{.green} 完成 |
| **Phase 1** | **多语言互联 + 语言特性** | :material-check-circle:{.green} **全部完成** |
| Phase 1.1 | C FFI 基础 | :material-check-circle:{.green} 完成 |
| Phase 1.2 | Python / C++ FFI | :material-check-circle:{.green} 完成 |
| Phase 1.3 | `stream<T>` 数据流核心类型 + 管道运算符 | :material-check-circle:{.green} 完成 |
| Phase 1.4 | 多语言 FFI (12 种语言) | :material-check-circle:{.green} 完成 |
| Phase 1.5 | struct/enum + match 模式匹配 | :material-check-circle:{.green} 完成 |
| Phase 1.6 | bindgen: C/Python/TypeScript 绑定生成 | :material-check-circle:{.green} 完成 |
| Phase 1.7 | flow 声明块 | :material-check-circle:{.green} 完成 |
| Phase 1.8 | async/await 异步编程 | :material-check-circle:{.green} 完成 |
| **Phase 2** | **编译器 + 工具链** | :material-check-circle:{.green} **全部完成** |
| Phase 2.1 | C 后端:完整 C 代码生成 | :material-check-circle:{.green} 完成 |
| Phase 2.2 | LLVM 后端: LLVM IR 生成 | :material-check-circle:{.green} 完成 |
| Phase 2.3 | Python 后端: 生成 Python 代码 | :material-check-circle:{.green} 完成 |
| Phase 2.4 | WASM 后端: 生成 WAT 格式代码 | :material-check-circle:{.green} 完成 |
| Phase 2.5 | 类型检查器 (54 个测试) | :material-check-circle:{.green} 完成 |
| Phase 2.6 | 常量折叠优化 | :material-check-circle:{.green} 完成 |
| Phase 2.7 | 死代码消除 | :material-check-circle:{.green} 完成 |
| Phase 2.8 | 借用检查器: 所有权跟踪、use-after-move 检测 | :material-check-circle:{.green} 完成 |
| Phase 2.9 | LSP 语言服务器: 诊断、补全、悬停、跳转、符号大纲 | :material-check-circle:{.green} 完成 |
| Phase 2.10 | 游戏后端域类型: domain 语法、WebSocket、房间系统、帧同步 | :material-check-circle:{.green} 完成 |
| Phase 3 | `endpoint` / `group` IoT 抽象 | :material-clock-outline:{.yellow} 规划中 |

## 下一步

- [安装 Link](installation.md)
- [快速开始](quickstart.md)
- [多语言互联概述](ffi/overview.md)
