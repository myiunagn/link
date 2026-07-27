# 介绍

## Link 是什么

**Link 是一门专用于"互联"的静态类型声明式数据流语言。**

- 以多语言互联为第一切入点
- 延展到游戏后端与 IoT 设备互联
- 编译为原生码(LLVM 路线图进行中)
- 当前 v0.1 为解释器版本,用于验证语言设计与 FFI 设计

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
| `stream<T>` | 数据流,Link 的灵魂 | :material-clock-outline:{.yellow} 规划中 |
| `endpoint` | 连接端点(设备/玩家/服务) | :material-clock-outline:{.yellow} 规划中 |
| `group` | 群组(房间/网关/集群) | :material-clock-outline:{.yellow} 规划中 |
| `extern` / `export` | 多语言互操作 | :material-check-circle:{.green} 已实现 |

## 当前版本能做什么

v0.1 已实现:

- 基本类型:`int` / `float` / `str` / `bool` / `none` / `list`
- 运算符:算术、比较、逻辑
- 控制流:`if/else` / `while` / `for` / `loop` / `break` / `continue`
- 函数:声明、递归、闭包作用域
- 内置函数:`print` / `println` / `len`
- 列表:字面量、索引、嵌套
- **FFI**:
    - C 标准库(`libc.so.6` / `msvcrt.dll`)
    - Python 标准库(`math` / `os` / 任意模块)
    - C++ 共享库(通过 `extern "C"` 桥接)
- 运行模式:`link run <file>` 文件执行,`link repl` 交互式 REPL

## 设计哲学

1. **连接为一等公民** —— `stream<T>` / `endpoint` / `group` 是语言内置类型,不是库里的 class
2. **流是默认执行模型** —— 数据从源到汇自动调度,无需手写并发
3. **声明优先,无副作用** —— 描述"要什么",不描述"怎么做"
4. **多语言原生互通** —— Link 不孤立存在,天然是其他语言的胶水层

## 路线图

| 阶段 | 内容 | 状态 |
|------|------|------|
| Phase 0 | 骨架:Lexer + Parser + Interpreter | :material-check-circle:{.green} 完成 |
| Phase 1.1 | C FFI 基础 | :material-check-circle:{.green} 完成 |
| Phase 1.2 | Python / C++ FFI | :material-check-circle:{.green} 完成 |
| Phase 1.3 | `stream<T>` 数据流核心类型 | :material-clock-outline:{.yellow} 进行中 |
| Phase 1.6 | `export "C"` 头文件生成 | :material-clock-outline:{.yellow} 规划中 |
| Phase 2 | LLVM 后端,原生码编译 | :material-clock-outline:{.yellow} 规划中 |
| Phase 3 | `endpoint` / `group` IoT 抽象 | :material-clock-outline:{.yellow} 规划中 |

## 下一步

- [安装 Link](installation.md)
- [快速开始](quickstart.md)
- [多语言互联概述](ffi/overview.md)
