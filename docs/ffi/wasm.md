# WebAssembly 互连

!!! abstract "概述"
    Link 通过 `wasmtime` 运行时加载并调用 WebAssembly 模块的导出函数。
    WASM 模块可以用 C / C++ / Rust / AssemblyScript 等语言编译生成。

## 基本用法

### 加载 WASM 模块

```link
extern "wasm" module "math_utils.wasm" {
    fn add(a: i32, b: i32) -> i32;
    fn multiply(a: i32, b: i32) -> i32;
    fn square(a: i32) -> i32;
}

let sum = add(10, 20);        // 30
let product = multiply(3, 4); // 12
let sq = square(5);           // 25
```

### module 字段

`module` 字段指定 `.wasm` 文件的路径:

```link
extern "wasm" module "path/to/module.wasm" {
    fn function_name(...) -> ...;
}
```

- 路径可以是相对路径(基于 Link 进程工作目录)或绝对路径
- 文件必须是有效的 WebAssembly 二进制格式(`.wasm`)

## 类型映射

| Link 类型 | WASM 类型 | 说明 |
|-----------|-----------|------|
| `i32` | `i32` | 32 位有符号整数 |
| `i64` | `i64` | 64 位有符号整数 |
| `f32` | `f32` | 32 位浮点数 |
| `f64` | `f64` | 64 位浮点数 |
| `bool` | `i32` | 0 = false, 非 0 = true |

!!! note "WASM 类型限制"
    WebAssembly 原生只支持数值类型。字符串、数组等复杂类型需要通过内存指针传递,目前 Link WASM FFI 主要支持数值类型。

## 创建 WASM 模块

### 用 Rust 创建 WASM 模块

1. 创建 Rust 项目:

```bash
cargo new --lib math_utils
cd math_utils
```

2. 修改 `Cargo.toml`:

```toml
[package]
name = "math_utils"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
```

3. 编写代码:

```rust
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

#[no_mangle]
pub extern "C" fn square(a: i32) -> i32 {
    a * a
}
```

4. 编译为 WASM:

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
```

编译后的文件在 `target/wasm32-unknown-unknown/release/math_utils.wasm`。

### 用 C 创建 WASM 模块

使用 Emscripten 工具链:

```c
// math_utils.c
int add(int a, int b) {
    return a + b;
}

int multiply(int a, int b) {
    return a * b;
}
```

```bash
emcc math_utils.c -o math_utils.wasm \
    -s EXPORTED_FUNCTIONS='["_add", "_multiply"]' \
    -s EXPORTED_RUNTIME_METHODS='["ccall", "cwrap"]'
```

## 完整示例

```link
// 加载 WASM 模块
extern "wasm" module "math_utils.wasm" {
    fn add(a: i32, b: i32) -> i32;
    fn multiply(a: i32, b: i32) -> i32;
    fn square(a: i32) -> i32;
}

// 在 Link 中组合 WASM 函数
fn compute(a: i32, b: i32) -> i32 {
    let sum = add(a, b);
    let product = multiply(a, b);
    square(sum) + square(product)
}

println("compute(2, 3) =", compute(2, 3));
// add(2,3)=5, multiply(2,3)=6, 5²+6² = 25+36 = 61
```

## 性能特点

- **JIT 编译**:wasmtime 会在首次调用时 JIT 编译 WASM 模块,后续调用接近原生性能
- **沙箱隔离**:WASM 在沙箱中运行,不会直接访问宿主内存
- **零启动开销(首次后)**:模块加载后常驻内存,无需重复加载

## 常见问题

### 模块加载失败

确保 `.wasm` 文件路径正确,且文件是有效的 WebAssembly 二进制。

### 函数未找到

WASM 模块必须使用 `#[no_mangle]` (Rust) 或 `EXPORTED_FUNCTIONS` (C/Emscripten) 导出函数,函数名必须与 `extern` 声明完全一致。

## 下一步

- [多语言互联概述](overview.md)
- [进程桥接互连](process.md)
