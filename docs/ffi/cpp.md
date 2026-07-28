# C++ 互连

## 概述

通过 `extern "C++"` 调用 C++ 共享库。Link 利用 C++ 函数的 `extern "C"` 导出,通过 C ABI 直接调用,既保留了 C++ 的强大能力,又避免了名称修饰问题。

!!! abstract "核心原理"
    C++ 函数用 `extern "C"` 声明后,会获得 C ABI 兼容的符号名(无修饰),Link 通过 `dlopen` / `LoadLibrary` 加载 DLL/SO,按 C ABI 调用。这相当于把 C++ 当作"扩展的 C"来用,但内部可以自由使用 STL、模板、RAII 等所有 C++ 特性。

## 基本用法

### 1. 编写 C++ 库

`mylib.cpp`:

```cpp
#include <cstdint>
#include <string>
#include <vector>
#include <algorithm>

// 关键:必须用 extern "C" 导出
extern "C" __declspec(dllexport) int32_t add(int32_t a, int32_t b) {
    return a + b;
}

extern "C" __declspec(dllexport) int32_t factorial(int32_t n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}

extern "C" __declspec(dllexport) const char* version() {
    return "mylib v1.0 (C++17)";
}
```

!!! warning "extern \"C\" 是必需的"
    不加 `extern "C"`,C++ 编译器会进行**名称修饰**(name mangling),如 `add` 被编译为 `?add@@YAHHH@Z`(MSVC)或 `_Z3addii`(GCC),Link 找不到符号。

### 2. 编译为共享库

=== "Windows (MSVC)"

    ```powershell
    cl /LD /EHsc /std:c++17 mylib.cpp /link /OUT:mylib.dll
    ```
    
    参数说明:
    - `/LD`:编译为 DLL
    - `/EHsc`:C++ 异常处理
    - `/std:c++17`:使用 C++17 标准

=== "Windows (MinGW)"

    ```powershell
    g++ -shared -std=c++17 -o mylib.dll mylib.cpp
    ```

=== "Linux"

    ```bash
    g++ -shared -fPIC -std=c++17 -o libmylib.so mylib.cpp
    ```

=== "macOS"

    ```bash
    clang++ -shared -std=c++17 -o libmylib.dylib mylib.cpp
    ```

### 3. 在 Link 中调用

```link
extern "C++" module "mylib.dll" {
    fn add(a: i32, b: i32) -> i32;
    fn factorial(n: i32) -> i32;
    fn version() -> str;
}

println("add(3, 4) =", add(3, 4));           // 7
println("factorial(5) =", factorial(5));     // 120
println("version =", version());             // mylib v1.0 (C++17)
```

## 类型映射

C++ 与 Link 共享 C ABI,类型映射与 [C 互连](c.md) 完全一致:

| Link 类型 | C++ 类型 | 说明 |
|-----------|----------|------|
| `i32` | `int32_t` | 32 位整数 |
| `i64` | `int64_t` | 64 位整数 |
| `f32` | `float` | 单精度浮点 |
| `f64` | `double` | 双精度浮点 |
| `bool` | `bool` | 布尔 |
| `str` | `const char*` | C 字符串 |
| `none` | `void` | 无返回值 |

## 导出宏

为了跨平台编译,通常用宏统一导出声明:

```cpp
// mylib_export.h
#ifdef _WIN32
    #ifdef MYLIB_EXPORTS
        #define MYLIB_API __declspec(dllexport)
    #else
        #define MYLIB_API __declspec(dllimport)
    #endif
#else
    #define MYLIB_API __attribute__((visibility("default")))
#endif

extern "C" {
    MYLIB_API int32_t add(int32_t a, int32_t b);
    MYLIB_API const char* version();
}
```

编译 DLL 时定义 `MYLIB_EXPORTS`,使用方则不定义。

## 支持的函数签名

v0.1 支持 0~3 个参数的函数:

```link
extern "C++" module "mylib.dll" {
    // 0 参数
    fn version() -> str;
    fn counter() -> i32;
    
    // 1 参数
    fn factorial(n: i32) -> i32;
    fn sqrt(x: f64) -> f64;
    fn greet(name: str) -> str;
    
    // 2 参数
    fn add(a: i32, b: i32) -> i32;
    fn pow(base: f64, exp: f64) -> f64;
    
    // 3 参数
    fn max3(a: i32, b: i32, c: i32) -> i32;
    fn average(a: i32, b: i32, c: i32) -> f64;   // int 参数 + f64 返回
}
```

## 内部使用 C++ 特性

C++ 函数内部可以自由使用任何 C++ 特性,**对外只暴露 C ABI**:

```cpp
#include <vector>
#include <algorithm>
#include <string>
#include <memory>

// 内部使用 STL 容器
extern "C" __declspec(dllexport) int32_t max3(int32_t a, int32_t b, int32_t c) {
    std::vector<int32_t> v = {a, b, c};                    // STL 容器
    return *std::max_element(v.begin(), v.end());          // STL 算法
}

// 内部使用 std::string 管理内存,对外返回 const char*
extern "C" __declspec(dllexport) const char* greet(const char* name) {
    static thread_local std::string buffer;                 // C++ 静态局部变量
    buffer = "Hello, ";
    buffer += name;
    buffer += "! from C++";
    return buffer.c_str();                                  // 返回 C 字符串
}

// 内部使用 RAII 与智能指针
extern "C" __declspec(dllexport) double process(int32_t n) {
    auto data = std::make_unique<std::vector<double>>(n);   // 智能指针
    for (int i = 0; i < n; ++i) {
        (*data)[i] = i * 2.0;
    }
    double sum = 0;
    for (auto x : *data) sum += x;
    return sum / n;                                          // 自动释放内存
}
```

!!! tip "字符串所有权"
    返回 `const char*` 时,字符串必须指向**静态或堆内存**。推荐用 `static thread_local std::string` 模式,避免内存泄漏。

## 完整示例

### 数学库

`math_lib.cpp`:

```cpp
#include <cmath>
#include <cstdint>

extern "C" __declspec(dllexport) double cpp_sqrt(double x) {
    return std::sqrt(x);
}

extern "C" __declspec(dllexport) double cpp_pow(double base, double exp) {
    return std::pow(base, exp);
}

extern "C" __declspec(dllexport) int32_t cpp_factorial(int32_t n) {
    if (n <= 1) return 1;
    return n * cpp_factorial(n - 1);
}

extern "C" __declspec(dllexport) int32_t cpp_fib(int32_t n) {
    if (n < 2) return n;
    return cpp_fib(n - 1) + cpp_fib(n - 2);
}
```

`use_math.link`:

```link
extern "C++" module "math_lib.dll" {
    fn cpp_sqrt(x: f64) -> f64;
    fn cpp_pow(base: f64, exp: f64) -> f64;
    fn cpp_factorial(n: i32) -> i32;
    fn cpp_fib(n: i32) -> i32;
}

println("sqrt(2) =", cpp_sqrt(2.0));
println("pow(2, 10) =", cpp_pow(2.0, 10.0));
println("factorial(5) =", cpp_factorial(5));
println("fib(10) =", cpp_fib(10));
```

### 字符串处理

`string_lib.cpp`:

```cpp
#include <string>
#include <algorithm>
#include <cctype>

extern "C" __declspec(dllexport) const char* to_upper(const char* s) {
    static thread_local std::string buffer;
    buffer = s;
    std::transform(buffer.begin(), buffer.end(), buffer.begin(), ::toupper);
    return buffer.c_str();
}

extern "C" __declspec(dllexport) const char* reverse(const char* s) {
    static thread_local std::string buffer;
    buffer = s;
    std::reverse(buffer.begin(), buffer.end());
    return buffer.c_str();
}

extern "C" __declspec(dllexport) int32_t count_words(const char* s) {
    std::string str = s;
    int count = 0;
    bool in_word = false;
    for (char c : str) {
        if (std::isspace(c)) {
            in_word = false;
        } else if (!in_word) {
            in_word = true;
            ++count;
        }
    }
    return count;
}
```

`use_string.link`:

```link
extern "C++" module "string_lib.dll" {
    fn to_upper(s: str) -> str;
    fn reverse(s: str) -> str;
    fn count_words(s: str) -> i32;
}

println(to_upper("hello world"));        // HELLO WORLD
println(reverse("Link"));                // kniL
println(count_words("the quick brown fox"));  // 4
```

## 内置示例:cpp_demo.dll

仓库自带一个完整的 C++ 示例库 `cpp_demo.cpp`,包含 12 个函数:

| 函数 | 签名 | 说明 |
|------|------|------|
| `cpp_add` / `cpp_sub` / `cpp_mul` | `(i32, i32) -> i32` | 算术 |
| `cpp_sqrt` / `cpp_pow` | `(f64) -> f64` / `(f64, f64) -> f64` | 数学 |
| `cpp_factorial` / `cpp_fib` | `(i32) -> i32` | 递归 |
| `cpp_version` | `() -> str` | 版本信息 |
| `cpp_greet` | `(str) -> str` | 字符串 |
| `cpp_is_even` | `(i32) -> bool` | 布尔 |
| `cpp_max3` | `(i32, i32, i32) -> i32` | 三参数 |
| `cpp_average` | `(i32, i32, i32) -> f64` | 混合签名 |

测试文件 `cpp_extern.link`。

## 故障排查

### "Symbol 'xxx' not found"

C++ 符号未找到。最常见原因:

1. **未用 `extern "C"` 导出**:
   ```cpp
   // 错误:会被名称修饰
   int32_t add(int32_t a, int32_t b) { return a + b; }
   
   // 正确:无修饰
   extern "C" __declspec(dllexport) int32_t add(int32_t a, int32_t b) { return a + b; }
   ```

2. **Windows 上未加 `__declspec(dllexport)`**:
   ```cpp
   // Linux/macOS:加 visibility 属性
   extern "C" __attribute__((visibility("default"))) int32_t add(...);
   
   // Windows:加 dllexport
   extern "C" __declspec(dllexport) int32_t add(...);
   ```

3. **函数名拼写错误**:Link 声明与 C++ 实现必须完全一致

### 验证符号导出

用工具检查 DLL/SO 中的符号:

=== "Windows"

    ```powershell
    dumpbin /exports mylib.dll
    ```

=== "Linux"

    ```bash
    nm -D libmylib.so | grep " T "
    ```

=== "macOS"

    ```bash
    nm -gU libmylib.dylib
    ```

符号列表中应能看到 `add` / `factorial` 等纯名字。如果看到 `?add@@YA...` 或 `_Z3addii`,说明没正确用 `extern "C"`。

### 调用后崩溃

1. **字符串所有权**:返回 `const char*` 必须指向静态/堆内存,不能用栈上局部变量
2. **架构不匹配**:64 位 Link 加载 32 位 DLL
3. **签名不一致**:Link 声明的参数类型与 C++ 实现不一致

### C++ 异常

C++ 异常**不能**跨 FFI 边界传播。在导出函数中必须捕获所有异常:

```cpp
extern "C" __declspec(dllexport) int32_t safe_div(int32_t a, int32_t b) {
    try {
        if (b == 0) throw std::runtime_error("除零");
        return a / b;
    } catch (const std::exception& e) {
        // 返回错误码或写入全局错误信息
        return 0;
    }
}
```

!!! danger "未捕获异常的后果"
    未捕获的 C++ 异常跨过 FFI 边界会导致进程崩溃(Windows 上是 SEH 异常,Linux 上是 SIGABRT)。

## 下一步

- [示例集](../examples.md)
- [多语言互联概述](overview.md)
