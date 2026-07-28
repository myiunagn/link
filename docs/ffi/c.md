# C 互连

## 概述

通过 `extern "C"` 调用 C 标准库或自定义 C 共享库。Link 会动态加载 libc / msvcrt,按 C ABI 直接调用函数,无运行时开销。

## 基本用法

### 调用 C 标准库

```link
extern "C" {
    fn abs(n: i32) -> i32;
    fn sqrt(x: f64) -> f64;
    fn pow(base: f64, exp: f64) -> f64;
}

println("abs(-42) =", abs(-42));         // 42
println("sqrt(16.0) =", sqrt(16.0));     // 4
println("pow(2, 10) =", pow(2.0, 10.0)); // 1024
```

### 默认库

省略 `module` 时,Link 会加载默认 C 库:

| 平台 | 默认库 |
|------|--------|
| Linux | `libc.so.6` |
| macOS | `libc.dylib` |
| Windows | `msvcrt.dll` 或 `ucrtbase.dll` |

```link
// 等价于 extern "C" module "c"
extern "C" {
    fn abs(n: i32) -> i32;
}
```

### 指定库

用 `module` 显式指定库名:

```link
// 加载 libm(数学库,Linux/macOS)
extern "C" module "m" {
    fn sqrt(x: f64) -> f64;
    fn sin(x: f64) -> f64;
    fn cos(x: f64) -> f64;
}
```

## 类型映射

| Link 类型 | C 类型 | 说明 |
|-----------|--------|------|
| `i32` | `int32_t` / `int` | 32 位有符号整数 |
| `i64` | `int64_t` / `long long` | 64 位有符号整数 |
| `f32` | `float` | 单精度浮点 |
| `f64` | `double` | 双精度浮点 |
| `bool` | `bool` / `int` | 布尔值 |
| `str` | `const char*` | 以 null 结尾的字符串 |
| `none` | `void` | 无返回值 |

## 支持的函数签名

### 0 参数

```link
extern "C" {
    fn rand() -> i32;
}

let r = rand();
println("随机数:", r);
```

### 1 参数

```link
extern "C" {
    fn abs(n: i32) -> i32;        // int -> int
    fn sqrt(x: f64) -> f64;       // double -> double
    fn toupper(c: i32) -> i32;    // int -> int
}
```

### 2 参数

```link
extern "C" {
    fn pow(base: f64, exp: f64) -> f64;
    fn max(a: i32, b: i32) -> i32;
}
```

### 3 参数

```link
extern "C" {
    fn clamp(a: i32, b: i32, c: i32) -> i32;
}
```

### 字符串返回

C 函数返回 `const char*` 时,Link 自动转换为 `str`:

```link
extern "C" {
    fn getenv(name: str) -> str;
}

let home = getenv("HOME");
println("HOME =", home);
```

!!! warning "字符串所有权"
    C 函数返回的字符串必须指向静态或堆内存。返回栈上临时变量会导致未定义行为。

## 调用自定义 C 库

### 1. 编写 C 库

`mylib.c`:

```c
#include <stdint.h>

int32_t add(int32_t a, int32_t b) {
    return a + b;
}

int32_t factorial(int32_t n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}

const char* version() {
    return "mylib v1.0";
}
```

### 2. 编译为共享库

=== "Linux"

    ```bash
    gcc -shared -fPIC -o libmylib.so mylib.c
    ```

=== "macOS"

    ```bash
    gcc -shared -fPIC -o libmylib.dylib mylib.c
    ```

=== "Windows (MSVC)"

    ```powershell
    cl /LD mylib.c /link /OUT:mylib.dll
    ```

=== "Windows (MinGW)"

    ```powershell
    gcc -shared -o mylib.dll mylib.c
    ```

### 3. 在 Link 中调用

```link
extern "C" module "mylib.dll" {
    fn add(a: i32, b: i32) -> i32;
    fn factorial(n: i32) -> i32;
    fn version() -> str;
}

println("add(3, 4) =", add(3, 4));           // 7
println("factorial(5) =", factorial(5));     // 120
println("version =", version());             // mylib v1.0
```

!!! tip "库路径"
    - 相对路径基于运行 `link` 命令的当前目录
    - 也可用绝对路径:`module "C:/libs/mylib.dll"`
    - 系统库目录中的库可直接用名字:`module "mylib"`(无需扩展名)

## 完整示例

### 数学计算

```link
extern "C" module "m" {
    fn sqrt(x: f64) -> f64;
    fn pow(base: f64, exp: f64) -> f64;
    fn sin(x: f64) -> f64;
    fn cos(x: f64) -> f64;
}

fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    sqrt(pow(dx, 2.0) + pow(dy, 2.0))
}

fn angle(x: f64, y: f64) -> f64 {
    // atan2 暂未声明,这里用近似
    let r = sqrt(x * x + y * y);
    if r == 0.0 { return 0.0; }
    sin(y / r)   // 简化示例
}

let d = distance(0.0, 0.0, 3.0, 4.0);
println("距离 =", d);   // 5
```

### 字符串处理

```link
extern "C" {
    fn strlen(s: str) -> i32;
    fn atoi(s: str) -> i32;
}

let s = "hello";
let n = strlen(s);
println("长度:", n);     // 5

let num = atoi("42");
println("数字:", num);    // 42
```

## 故障排查

### "Symbol 'xxx' not found"

C 函数符号未找到。可能原因:

1. **Windows 上函数未导出**:MSVC 编译时需加 `__declspec(dllexport)`
2. **C++ 名称修饰**:C++ 文件需用 `extern "C"` 包裹函数声明
3. **函数名拼写错误**:检查大小写

### "Failed to load library"

库加载失败。可能原因:

1. **库文件不存在**:检查路径
2. **架构不匹配**:64 位 Link 加载 32 位 DLL
3. **依赖缺失**:DLL 依赖的其他 DLL 不在 PATH 中

### 调用后段错误

通常是签名不匹配导致:

1. 参数数量不对
2. 参数类型不对(如 C 函数期望 `int`,Link 传了 `f64`)
3. 返回类型不对(如 C 函数返回 `int`,Link 声明为 `str`)

## 下一步

- [Python 互连](python.md)
- [C++ 互连](cpp.md)
