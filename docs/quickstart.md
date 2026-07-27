# 快速开始

## Hello, World

创建 `hello.link`:

```link
println("Hello, World!");
```

运行:

```bash
link run hello.link
```

输出:

```
Hello, World!
```

## 基本语法

```link
// 变量声明
let name = "Link";
let version = 0.1;
let year = 2026;

// 类型注解(可选)
let count: i32 = 42;
let pi: f64 = 3.14159;
let is_ready: bool = true;

// 字符串插值(暂不支持,用逗号拼接)
println("语言:", name, "版本:", version, "年份:", year);
```

## 控制流

```link
// if / else if / else
let score = 85;
if score >= 90 {
    println("优秀");
} else if score >= 60 {
    println("及格");
} else {
    println("不及格");
}

// while 循环
let i = 0;
while i < 5 {
    println("i =", i);
    i = i + 1;
}

// for 循环(范围式)
for j in 0..3 {
    println("j =", j);
}

// loop + break
let k = 0;
loop {
    if k >= 3 { break; }
    println("k =", k);
    k = k + 1;
}
```

## 函数

```link
// 普通函数
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// 递归
fn factorial(n: i32) -> i32 {
    if n <= 1 { return 1; }
    n * factorial(n - 1)
}

// 调用
let sum = add(3, 4);
let fact = factorial(5);
println("add(3, 4) =", sum);
println("factorial(5) =", fact);
```

## 列表

```link
let nums = [1, 2, 3, 4, 5];
let first = nums[0];
let len = len(nums);

println("第一个:", first);
println("长度:", len);

// 嵌套
let matrix = [[1, 2], [3, 4]];
println("matrix[1][0] =", matrix[1][0]);
```

## 内置函数

| 函数 | 说明 | 示例 |
|------|------|------|
| `print(...)` | 打印不换行 | `print("a", "b")` |
| `println(...)` | 打印换行 | `println("hi")` |
| `len(x)` | 字符串/列表长度 | `len([1,2,3])` |

## 多语言互联(Link 的灵魂)

### 调用 C 标准库

```link
extern "C" {
    fn abs(n: i32) -> i32;
    fn sqrt(x: f64) -> f64;
}

println("abs(-42) =", abs(-42));
println("sqrt(16.0) =", sqrt(16.0));
```

### 调用 Python

```link
extern "python" module "math" {
    fn sqrt(x: f64) -> f64;
    fn pow(base: f64, exp: f64) -> f64;
}

extern "python" module "os" {
    fn getcwd() -> str;
}

println("math.sqrt(16.0) =", sqrt(16.0));
println("math.pow(2, 10) =", pow(2.0, 10.0));
println("os.getcwd() =", getcwd());
```

### 调用 C++

```link
extern "C++" module "examples/cpp_demo.dll" {
    fn cpp_add(a: i32, b: i32) -> i32;
    fn cpp_factorial(n: i32) -> i32;
    fn cpp_greet(name: str) -> str;
}

println("cpp_add(3, 4) =", cpp_add(3, 4));
println("cpp_factorial(5) =", cpp_factorial(5));
println("cpp_greet(\"Link\") =", cpp_greet("Link"));
```

!!! info "C++ FFI 原理"
    C++ 函数用 `extern "C"` 导出即可获得 C ABI,Link 通过 `extern "C++"` 加载 DLL/SO 并按 C ABI 调用。

## 完整示例

把所有特性组合起来:

```link
// === 1. 声明外部函数 ===
extern "C" {
    fn abs(n: i32) -> i32;
}

extern "python" module "math" {
    fn sqrt(x: f64) -> f64;
    fn pow(base: f64, exp: f64) -> f64;
}

extern "C++" module "examples/cpp_demo.dll" {
    fn cpp_factorial(n: i32) -> i32;
}

// === 2. Link 自己的函数 ===
fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    sqrt(pow(dx, 2.0) + pow(dy, 2.0))
}

// === 3. 业务逻辑 ===
let d = distance(0.0, 0.0, 3.0, 4.0);
println("距离 =", d);

let f = cpp_factorial(5);
println("5! =", f);

let a = abs(-100);
println("|-100| =", a);

// === 4. 表达式混合使用 ===
let combined = abs(-3) + cpp_factorial(3);
println("combined =", combined);

combined
```

运行:

```bash
link run demo.link
```

输出:

```
距离 = 5
5! = 120
|-100| = 100
combined = 9
```

## 下一步

- [REPL 交互模式](repl.md)
- [语言基础](basics/types.md)
- [多语言互联](ffi/overview.md)
