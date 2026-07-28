# Python 互连

## 概述

通过 `extern "python"` 调用 Python 标准库或第三方库。Link 会动态加载 libpython,通过 CPython C API 调用 Python 函数。

!!! info "工作原理"
    Link 内部维护一个 `PythonRuntime`,首次使用 `extern "python"` 时自动初始化:
    
    1. 加载 Python 共享库(`python3.dll` / `libpython3.so`)
    2. 调用 `Py_Initialize()` 初始化解释器
    3. 通过 `PyImport_ImportModule` 导入指定模块
    4. 通过 `PyObject_GetAttrString` 获取函数
    5. 用 `PyObject_CallFunction` 调用,自动转换 Link 与 Python 类型

## 基本用法

### 调用 math 模块

```link
extern "python" module "math" {
    fn sqrt(x: f64) -> f64;
    fn pow(base: f64, exp: f64) -> f64;
    fn log(x: f64) -> f64;
    fn log2(x: f64) -> f64;
    fn floor(x: f64) -> f64;
    fn ceil(x: f64) -> f64;
}

println("sqrt(16.0) =", sqrt(16.0));       // 4
println("pow(2, 10) =", pow(2.0, 10.0));   // 1024
println("log(e) =", log(2.71828));         // 1.0
println("floor(3.7) =", floor(3.7));       // 3
```

### 调用 os 模块

```link
extern "python" module "os" {
    fn getcwd() -> str;
    fn getpid() -> i32;
}

println("当前目录:", getcwd());
println("进程 ID:", getpid());
```

### 调用 json 模块

```link
extern "python" module "json" {
    fn dumps(obj: str) -> str;
}

// 注意:v0.1 暂不支持直接传递 dict,只能传字符串
let s = dumps("{\"name\": \"Link\", \"version\": 0.1}");
println(s);
```

## 类型映射

| Link 类型 | Python 类型 | 转换方式 |
|-----------|-------------|----------|
| `i32` / `i64` | `int` | `PyLong_FromLongLong` |
| `f32` / `f64` | `float` | `PyFloat_FromDouble` |
| `bool` | `bool` | `PyBool_FromLong` |
| `str` | `str` | `PyUnicode_DecodeUTF8` |
| `none` | `None` | `Py_BuildValue("z", NULL)` |
| `list` | `list` | (规划中) |

## 支持的函数签名

Python FFI 支持 0~2 个参数,返回类型可以是基本类型:

```link
extern "python" module "math" {
    fn pi() -> f64;                      // 0 参数
    fn sqrt(x: f64) -> f64;              // 1 参数
    fn pow(base: f64, exp: f64) -> f64;  // 2 参数
}

extern "python" module "os" {
    fn getcwd() -> str;
    fn getpid() -> i32;
}
```

## 找到 Python 库

Link 会自动按以下顺序查找 Python 共享库:

### Windows

1. `python3.dll`(Python 3.x 主 DLL,通常在 Python 安装目录)
2. `python311.dll` / `python310.dll` / `python39.dll`(具体版本)
3. PATH 中的所有 `python3*.dll`

### Linux

1. `libpython3.so`
2. `libpython3.11.so.1.0` / `libpython3.10.so.1.0`(具体版本)
3. `libpython3.so` (系统中 `python3 -c "import sys; print(sys.version)"`)

### macOS

1. Python Framework 内嵌的 libpython
2. Homebrew 安装的 `libpython3.dylib`

### 手动指定

如果自动查找失败,可以手动指定 DLL 路径(规划中):

```link
// 未来支持:
extern "python" module "C:/Python311/python311.dll" {
    fn sqrt(x: f64) -> f64;
}
```

## 完整示例

### 数学计算

```link
extern "python" module "math" {
    fn sqrt(x: f64) -> f64;
    fn pow(base: f64, exp: f64) -> f64;
    fn sin(x: f64) -> f64;
    fn cos(x: f64) -> f64;
    fn radians(deg: f64) -> f64;
}

fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    sqrt(pow(dx, 2.0) + pow(dy, 2.0))
}

let d = distance(0.0, 0.0, 3.0, 4.0);
println("距离 =", d);                       // 5

let r = radians(90.0);
let s = sin(r);
println("sin(90°) =", s);                   // 1.0
```

### 系统信息

```link
extern "python" module "os" {
    fn getcwd() -> str;
    fn getpid() -> i32;
    fn uname() -> str;   // 注意:实际返回 tuple,这里简化
}

extern "python" module "time" {
    fn time() -> f64;
}

println("当前目录:", getcwd());
println("进程 ID:", getpid());
println("时间戳:", time());
```

### 混合使用多种语言

```link
// C 标准库
extern "C" {
    fn abs(n: i32) -> i32;
}

// Python 标准库
extern "python" module "math" {
    fn sqrt(x: f64) -> f64;
    fn pow(base: f64, exp: f64) -> f64;
}

// C++ 共享库
extern "C++" module "examples/cpp_demo.dll" {
    fn cpp_factorial(n: i32) -> i32;
}

// 在一个函数中混合调用三种语言
fn complex_calc(x: f64) -> f64 {
    let a = abs(-3);              // C
    let b = sqrt(x);              // Python
    let c = cpp_factorial(3);     // C++
    b * a + c                     // 4 * 3 + 6 = 18 (x=16)
}

println("结果:", complex_calc(16.0));
```

## 性能注意事项

!!! tip "Python 调用开销"
    每次 Python 函数调用有约 1-5 微秒开销(类型转换 + GIL 获取)。对于热路径代码:
    
    - 简单数学运算优先用 C / C++ FFI
    - Python 适合调用其丰富的标准库(如 `json` / `re` / `datetime`)
    - 批量处理优于循环单次调用

## 故障排查

### "Failed to load Python library"

Python 未安装或未在 PATH 中。解决方法:

1. **Windows**:确认安装时勾选 "Add Python to PATH"
2. **Linux**:安装 `python3-dev`(Ubuntu)或 `python3-devel`(CentOS)
3. **macOS**:`brew install python`

### "Symbol 'Py_Initialize' not found"

加载到了错误的 Python 库(如 embedded Python)。确保 `python3.dll` 是完整版而非 stub。

### Python 异常

Python 函数抛出异常时,Link 会捕获并转为错误信息:

```
Error: Python exception: ValueError: math domain error
```

检查输入参数是否合法。

### "Failed to import module 'xxx'"

Python 模块不存在或不在 `sys.path` 中。检查:

1. 模块名拼写
2. 是否安装了第三方库(`pip install xxx`)
3. 当前 Python 环境是否正确(虚拟环境可能未激活)

## 下一步

- [C++ 互连](cpp.md)
- [示例集](../examples.md)
