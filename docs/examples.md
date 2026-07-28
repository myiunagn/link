# 示例

## 入门示例

### Hello, World

```link
println("Hello, World!");
```

### 基本算术

```link
let a = 10;
let b = 3;

println("a + b =", a + b);   // 13
println("a - b =", a - b);   // 7
println("a * b =", a * b);   // 30
println("a / b =", a / b);   // 3 (整数除法)
println("a % b =", a % b);   // 1
```

### 变量与类型

```link
let name: str = "Link";
let version: f64 = 0.1;
let year: i32 = 2026;
let is_open_source: bool = true;

println("语言:", name);
println("版本:", version);
println("年份:", year);
println("开源:", is_open_source);
```

## 算法示例

### 斐波那契数列

```link
fn fib(n: i32) -> i32 {
    if n < 2 { return n; }
    fib(n - 1) + fib(n - 2)
}

// 打印前 15 项
for i in 0..15 {
    print(fib(i), " ");
}
println("");
// 输出:0 1 1 2 3 5 8 13 21 34 55 89 144 233 377
```

### 阶乘

```link
fn factorial(n: i32) -> i32 {
    if n <= 1 { return 1; }
    n * factorial(n - 1)
}

for i in 1..11 {
    println(i, "! =", factorial(i));
}
```

### 最大公约数(辗转相除)

```link
fn gcd(a: i32, b: i32) -> i32 {
    if b == 0 { return a; }
    gcd(b, a % b)
}

println("gcd(48, 36) =", gcd(48, 36));   // 12
println("gcd(17, 5) =", gcd(17, 5));     // 1
println("gcd(100, 75) =", gcd(100, 75)); // 25
```

### 素数判断

```link
fn is_prime(n: i32) -> bool {
    if n < 2 { return false; }
    if n == 2 { return true; }
    if n % 2 == 0 { return false; }
    
    let i = 3;
    while i * i <= n {
        if n % i == 0 { return false; }
        i = i + 2;
    }
    true
}

// 找出 100 以内的所有素数
let count = 0;
for n in 2..100 {
    if is_prime(n) {
        print(n, " ");
        count = count + 1;
    }
}
println("");
println("共", count, "个素数");
```

### 冒泡排序

```link
fn bubble_sort(nums: list) -> list {
    let n = len(nums);
    for i in 0..n {
        for j in 0..(n - i - 1) {
            if nums[j] > nums[j + 1] {
                let temp = nums[j];
                nums[j] = nums[j + 1];
                nums[j + 1] = temp;
            }
        }
    }
    nums
}

let arr = [64, 34, 25, 12, 22, 11, 90];
let sorted = bubble_sort(arr);
println("排序后:", sorted);
// 输出:[11, 12, 22, 25, 34, 64, 90]
```

### 二分查找

```link
fn binary_search(nums: list, target: i32) -> i32 {
    let lo = 0;
    let hi = len(nums);
    
    while lo < hi {
        let mid = (lo + hi) / 2;
        if nums[mid] == target {
            return mid;
        } else if nums[mid] < target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    -1
}

let arr = [1, 3, 5, 7, 9, 11, 13, 15, 17, 19];
for target in [5, 13, 8] {
    let idx = binary_search(arr, target);
    if idx >= 0 {
        println("找到", target, "在索引", idx);
    } else {
        println(target, "不存在");
    }
}
```

## FFI 示例

### C 标准库

```link
extern "C" {
    fn abs(n: i32) -> i32;
    fn sqrt(x: f64) -> f64;
    fn pow(base: f64, exp: f64) -> f64;
}

// 计算两点距离
fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    sqrt(pow(dx, 2.0) + pow(dy, 2.0))
}

println("距离 =", distance(0.0, 0.0, 3.0, 4.0));   // 5
println("|-42| =", abs(-42));                        // 42
```

### Python 标准库

```link
extern "python" module "math" {
    fn sqrt(x: f64) -> f64;
    fn pow(base: f64, exp: f64) -> f64;
    fn log(x: f64) -> f64;
    fn sin(x: f64) -> f64;
    fn cos(x: f64) -> f64;
    fn radians(deg: f64) -> f64;
}

extern "python" module "os" {
    fn getcwd() -> str;
    fn getpid() -> i32;
}

// 数学计算
println("sqrt(2) =", sqrt(2.0));
println("log(e) =", log(2.718281828));
println("sin(90°) =", sin(radians(90.0)));

// 系统信息
println("当前目录:", getcwd());
println("进程 ID:", getpid());
```

### C++ 共享库

```link
extern "C++" module "examples/cpp_demo.dll" {
    fn cpp_add(a: i32, b: i32) -> i32;
    fn cpp_factorial(n: i32) -> i32;
    fn cpp_fib(n: i32) -> i32;
    fn cpp_version() -> str;
    fn cpp_greet(name: str) -> str;
    fn cpp_is_even(n: i32) -> bool;
    fn cpp_max3(a: i32, b: i32, c: i32) -> i32;
}

println("C++ 库版本:", cpp_version());
println("3 + 4 =", cpp_add(3, 4));
println("5! =", cpp_factorial(5));
println("fib(10) =", cpp_fib(10));
println("问候:", cpp_greet("Link"));
println("42 是偶数:", cpp_is_even(42));
println("max(3, 9, 6) =", cpp_max3(3, 9, 6));
```

### 三语言混合

```link
// === 1. C 标准库 ===
extern "C" {
    fn abs(n: i32) -> i32;
}

// === 2. Python 标准库 ===
extern "python" module "math" {
    fn sqrt(x: f64) -> f64;
    fn pow(base: f64, exp: f64) -> f64;
}

// === 3. C++ 共享库 ===
extern "C++" module "examples/cpp_demo.dll" {
    fn cpp_factorial(n: i32) -> i32;
}

// === 4. Link 自己的函数 ===
fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = abs(x2 - x1);       // 调用 C
    let dy = abs(y2 - y1);       // 调用 C
    sqrt(pow(dx, 2.0) + pow(dy, 2.0))  // 调用 Python
}

// === 5. 业务逻辑 ===
let d = distance(0.0, 0.0, 3.0, 4.0);
let f = cpp_factorial(5);         // 调用 C++
let combined = abs(-3) + cpp_factorial(3);  // C + C++

println("距离 =", d);
println("5! =", f);
println("组合 =", combined);
```

## 实用工具

### 简易计算器

```link
fn add(a: f64, b: f64) -> f64 { a + b }
fn sub(a: f64, b: f64) -> f64 { a - b }
fn mul(a: f64, b: f64) -> f64 { a * b }
fn div(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        println("错误:除零");
        return 0.0;
    }
    a / b
}

let x = 10.0;
let y = 3.0;

println(x, "+", y, "=", add(x, y));
println(x, "-", y, "=", sub(x, y));
println(x, "*", y, "=", mul(x, y));
println(x, "/", y, "=", div(x, y));
```

### FizzBuzz

```link
for n in 1..31 {
    if n % 15 == 0 {
        println("FizzBuzz");
    } else if n % 3 == 0 {
        println("Fizz");
    } else if n % 5 == 0 {
        println("Buzz");
    } else {
        println(n);
    }
}
```

### 九九乘法表

```link
for i in 1..10 {
    for j in 1..i + 1 {
        let product = i * j;
        print(j, "*", i, "=", product, "  ");
    }
    println("");
}
```

输出:

```text
1*1=1
1*2=2  2*2=4
1*3=3  2*3=6  3*3=9
(中间省略)
1*9=9  2*9=18  3*9=27  4*9=36  5*9=45  6*9=54  7*9=63  8*9=72  9*9=81
```

### 字符串反转

```link
fn reverse(s: str) -> str {
    let n = len(s);
    let result = "";
    let i = n - 1;
    while i >= 0 {
        result = result + s[i];
        i = i - 1;
    }
    result
}

println(reverse("hello"));    // olleh
println(reverse("Link"));     // kniL
println(reverse("你好世界"));   // 界世好你
```

### 矩阵运算

```link
fn matrix_print(m: list) {
    for i in 0..len(m) {
        let row = m[i];
        for j in 0..len(row) {
            print(row[j], " ");
        }
        println("");
    }
}

fn matrix_add(a: list, b: list) -> list {
    let rows = len(a);
    let cols = len(a[0]);
    let result = [];
    for i in 0..rows {
        let row = [];
        for j in 0..cols {
            row = row + [a[i][j] + b[i][j]];
        }
        result = result + [row];
    }
    result
}

let a = [[1, 2], [3, 4]];
let b = [[5, 6], [7, 8]];
let c = matrix_add(a, b);
matrix_print(c);
// 输出:
// 6 8
// 10 12
```

## 数据处理

### 统计计算

```link
fn sum(nums: list) -> f64 {
    let total = 0.0;
    for i in 0..len(nums) {
        total = total + nums[i];
    }
    total
}

fn mean(nums: list) -> f64 {
    sum(nums) / len(nums)
}

fn max_of(nums: list) -> f64 {
    let m = nums[0];
    for i in 1..len(nums) {
        if nums[i] > m {
            m = nums[i];
        }
    }
    m
}

fn min_of(nums: list) -> f64 {
    let m = nums[0];
    for i in 1..len(nums) {
        if nums[i] < m {
            m = nums[i];
        }
    }
    m
}

let data = [12.5, 23.4, 8.9, 34.2, 18.7, 25.1, 15.6];

println("数据:", data);
println("总和:", sum(data));
println("平均:", mean(data));
println("最大:", max_of(data));
println("最小:", min_of(data));
```

### 数据过滤

```link
fn filter_positive(nums: list) -> list {
    let result = [];
    for i in 0..len(nums) {
        if nums[i] > 0 {
            result = result + [nums[i]];
        }
    }
    result
}

let nums = [-3, 5, -1, 8, 2, -7, 4, 0, 6];
let positives = filter_positive(nums);
println("正数:", positives);   // [5, 8, 2, 4, 6]
```

## 下一步

- [多语言互联概述](ffi/overview.md)
- [设计规格](spec.md)

## stream<T> 数据流

### 基本用法

```link
// 从列表创建 stream
let s = stream([1, 2, 3, 4, 5]);

// map:变换每个元素
let doubled = stream([1, 2, 3])
    | map(fn(x) -> i64 { return x * 2; })
    | collect();
println(doubled);  // [2, 4, 6]

// filter:过滤元素
let evens = stream([1, 2, 3, 4, 5, 6])
    | filter(fn(x) -> bool { return x % 2 == 0; })
    | collect();
println(evens);  // [2, 4, 6]

// for_each:遍历元素
stream([1, 2, 3])
    | for_each(fn(x) { print(x, " "); });
// 输出:1 2 3
```

### 管道链式操作

```link
// 多步管道操作
let result = stream([1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
    | map(fn(x) -> i64 { return x * x; })           // 平方
    | filter(fn(x) -> bool { return x > 50; })      // 保留大于 50 的
    | collect();

println(result);  // [64, 81, 100]
```

## 多语言 FFI 示例

### WebAssembly 互连

```link
extern "wasm" module "math_utils.wasm" {
    fn add(a: i32, b: i32) -> i32;
    fn multiply(a: i32, b: i32) -> i32;
}

println("wasm add(3, 4) =", add(3, 4));          // 7
println("wasm multiply(5, 6) =", multiply(5, 6)); // 30
```

### Java 互连

```link
extern "java" module "build/classes::com.example.MathUtils" {
    fn factorial(n: i64) -> i64;
    fn greet(name: str) -> str;
}

println("5! =", factorial(5));        // 120
println("问候:", greet("Link"));      // Hello, Link!
```

### HTML / JS 互连

```link
extern "html" module "http://127.0.0.1:3000" {
    fn add(a: i32, b: i32) -> i32;
    fn render_html(content: str) -> str;
}

println("js add(10, 20) =", add(10, 20));
println("html:", render_html("<h1>Hello</h1>"));
```

### 进程桥接互连

```link
// Go 桥接
extern "go" module "bridge.go" {
    fn add(a: i64, b: i64) -> i64;
    fn greet(name: str) -> str;
}

// Ruby 桥接
extern "ruby" module "bridge.rb" {
    fn transform(input: i64) -> i64;
}

let go_sum = add(10, 20);
let ruby_result = transform(go_sum);

println("Go add(10, 20) =", go_sum);
println("Ruby transform =", ruby_result);
```

### 全语言混合

```link
extern "C" { fn abs(n: i32) -> i32; }
extern "python" module "math" { fn sqrt(x: f64) -> f64; }
extern "C++" module "cpp_demo.dll" { fn cpp_factorial(n: i32) -> i32; }
extern "wasm" module "mod.wasm" { fn add(a: i32, b: i32) -> i32; }
extern "go" module "bridge.go" { fn greet(name: str) -> str; }

// 混合使用 5 种语言
let c_abs = abs(-42);
let py_sqrt = sqrt(16.0);
let cpp_fact = cpp_factorial(5);
let wasm_add = add(10, 20);
let go_greet = greet("Link");

println("C abs(-42) =", c_abs);
println("Python sqrt(16) =", py_sqrt);
println("C++ factorial(5) =", cpp_fact);
println("WASM add(10, 20) =", wasm_add);
println("Go greet:", go_greet);

// stream + 多语言
let result = stream([1, 2, 3, 4, 5])
    | map(fn(x) -> i64 { return add(x, x); })  // 调用 WASM
    | filter(fn(x) -> bool { return x > 4; })
    | collect();
println("stream result:", result);  // [6, 8, 10]
```
