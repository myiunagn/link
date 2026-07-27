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
extern "C++"    module "engine"    { fn render() -> i32; }
```

- 一门语言,统一类型系统
- 编译期检查所有跨语言调用签名
- 运行时直接走 C ABI / libpython,零序列化开销
- 未来 `stream<T>` 让数据自动跨语言流动

## extern 块语法

```link
extern "<language>" [module "<module_spec>"] {
    fn <name>(<params>) -> <return_type>;
    fn <name>(<params>) -> <return_type>;
    ...
}
```

### language

| 语言 | 关键字 | 别名 | 说明 |
|------|--------|------|------|
| C | `"C"` | `"c"` | 调用 C 标准库或自定义 C 库 |
| C++ | `"C++"` | `"cpp"` | 调用 C++ 共享库(需 `extern "C"` 导出) |
| Python | `"python"` | `"py"` | 调用 Python 模块 |

### module

`module` 字段含义因语言而异:

| 语言 | module 含义 | 示例 |
|------|------------|------|
| C | 库名(系统库或 DLL/SO 路径) | `"c"` / `"m"` / `"mylib.dll"` |
| C++ | DLL/SO 文件路径 | `"engine.dll"` / `"libengine.so"` |
| Python | Python 模块名 | `"math"` / `"os"` / `"json"` |

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

| Link 类型 | C 类型 | Python 类型 | C++ 类型(via C ABI) |
|-----------|--------|-------------|---------------------|
| `i32` | `int32_t` | `int` | `int32_t` |
| `i64` | `int64_t` | `int` | `int64_t` |
| `f32` | `float` | `float` | `float` |
| `f64` | `double` | `float` | `double` |
| `bool` | `bool` | `bool` | `bool` |
| `str` | `const char*` | `str` | `const char*` |
| `none` | `void` / `NULL` | `None` | `void` |

### 支持的签名

v0.1 支持的函数签名(参数数量 × 返回类型):

| 参数数量 | 返回类型 | 支持 |
|---------|---------|------|
| 0 | `i32` / `i64` / `f64` / `str` / `bool` | :material-check:{.green} |
| 1 | `i32` / `i64` / `f32` / `f64` / `str` / `bool` | :material-check:{.green} |
| 2 | `i32` / `i64` / `f64` | :material-check:{.green} |
| 3 | `i32` / `f64`(int 参数也可) | :material-check:{.green} |
| 其他 | - | :material-close:{.red} 规划中 |

!!! note "扩展计划"
    后续版本会支持任意参数数量、struct 传参、回调函数等高级特性。

## 性能特点

- **C / C++**:通过 C ABI 直接调用,无运行时开销(零序列化、零拷贝)
- **Python**:通过 CPython C API 调用,有 Python 解释器开销,但比 RPC 快得多
- **类型转换**:基础类型(int/float/bool)无开销,str 需 UTF-8 转换

## 各语言详细文档

- [C 互连](c.md) — 调用 C 标准库与自定义 C 库
- [Python 互连](python.md) — 调用 Python 标准库与第三方库
- [C++ 互连](cpp.md) — 通过 C ABI 调用 C++ 共享库

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

// 混合使用三种语言
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

## 下一步

- [C 互连](c.md)
- [Python 互连](python.md)
- [C++ 互连](cpp.md)
