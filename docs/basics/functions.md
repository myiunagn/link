# 函数

## 声明函数

用 `fn` 关键字声明函数:

```link
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

组成部分:

- `fn` 关键字
- 函数名(`add`)
- 参数列表 `(a: i32, b: i32)`,每个参数需带类型
- 返回类型 `-> i32`(可选,无返回值可省略或写 `-> ()`)
- 函数体 `{ ... }`

## 调用函数

```link
let result = add(3, 4);
println(result);   // 7

// 表达式中调用
let x = add(1, 2) + add(3, 4);
println(x);        // 10
```

## 返回值

### 隐式返回

函数最后一条表达式的值会自动作为返回值:

```link
fn double(x: i32) -> i32 {
    x * 2
}

fn greet(name: str) -> str {
    "Hello, " + name
}
```

### 显式 return

用 `return` 提前退出:

```link
fn factorial(n: i32) -> i32 {
    if n <= 1 { return 1; }
    n * factorial(n - 1)
}

fn classify(n: i32) -> str {
    if n < 0 { return "负数"; }
    if n == 0 { return "零"; }
    "正数"
}
```

### 无返回值

省略 `->` 或写 `-> ()`,函数返回 `none`:

```link
fn print_welcome() {
    println("欢迎使用 Link!");
}

fn print_sum(a: i32, b: i32) -> () {
    println("和:", a + b);
}
```

## 递归

函数可以调用自身:

```link
fn fib(n: i32) -> i32 {
    if n < 2 { return n; }
    fib(n - 1) + fib(n - 2)
}

println(fib(10));   // 55
```

```link
fn factorial(n: i32) -> i32 {
    if n <= 1 { return 1; }
    n * factorial(n - 1)
}

println(factorial(5));   // 120
```

!!! warning "栈溢出"
    深度递归会消耗栈空间。`fib(40)` 在解释器下较慢,`fib(50)` 可能需要数秒。后续 LLVM 后端会优化。

## 作用域与闭包

### 局部变量

函数体内的 `let` 声明是局部的:

```link
fn foo() -> i32 {
    let x = 10;       // 局部变量
    let y = 20;
    x + y
}

// println(x);   // 错误:x 不在此作用域
```

### 捕获外层变量

函数体内可以读取外层(定义时所在作用域)的变量:

```link
let pi = 3.14159;

fn circle_area(r: f64) -> f64 {
    pi * r * r       // 捕获外层 pi
}

println(circle_area(2.0));   // 12.566...
```

### 变量遮蔽

内层变量可以遮蔽外层同名变量:

```link
let x = 1;

fn outer() -> i32 {
    let x = 10;        // 遮蔽外层 x
    let inner = x + 5; // 15
    inner
}

println(outer());   // 15
println(x);          // 1 (外层 x 未变)
```

## 嵌套函数

v0.1 暂不支持函数内声明函数。需要辅助函数请在顶层声明:

```link
// 辅助函数
fn is_even(n: i32) -> bool {
    n % 2 == 0
}

fn main_logic(nums: list) -> i32 {
    let count = 0;
    for i in 0..len(nums) {
        if is_even(nums[i]) {
            count = count + 1;
        }
    }
    count
}

let nums = [1, 2, 3, 4, 5, 6];
println("偶数个数:", main_logic(nums));   // 3
```

## 高阶函数(v0.1 限制)

v0.1 函数值可作为变量传递,但**不能**作为参数或返回值(类型系统未完善):

```link
fn double(x: i32) -> i32 { x * 2 }
fn triple(x: i32) -> i32 { x * 3 }

// 可赋值给变量
let f = double;
println(f(5));   // 10

// 切换
f = triple;
println(f(5));   // 15
```

!!! note "未来计划"
    Phase 1.3+ 将支持完整的高阶函数、闭包和 `map` / `filter` / `reduce` 等函数式特性。

## 调用约定

- 参数按值传递
- 参数数量必须匹配
- 类型不匹配会在运行时报错(未来会做编译期检查)

```link
fn add(a: i32, b: i32) -> i32 { a + b }

add(1, 2);      // ✓
// add(1);       // ✗ 参数数量不符
// add(1, "x");  // ✗ 类型不符
```

## 内置函数

Link 提供以下内置函数:

| 函数 | 签名 | 说明 |
|------|------|------|
| `print(...)` | `(... args) -> none` | 打印不换行,参数用空格分隔 |
| `println(...)` | `(... args) -> none` | 打印换行,参数用空格分隔 |
| `len(x)` | `(str \| list) -> i32` | 字符串/列表长度 |

```link
println("a", "b", "c");   // a b c
print("x", "y");          // x y (不换行)
print("\n");              // 手动换行

let s = "hello";
let n = len(s);           // 5

let arr = [1, 2, 3];
let m = len(arr);         // 3
```

## 综合示例

### 计算斐波那契数列前 N 项

```link
fn fib(n: i32) -> i32 {
    if n < 2 { return n; }
    fib(n - 1) + fib(n - 2)
}

// 打印前 10 项
for i in 0..10 {
    print(fib(i), " ");
}
println("");
// 输出:0 1 1 2 3 5 8 13 21 34
```

### 求最大公约数

```link
fn gcd(a: i32, b: i32) -> i32 {
    if b == 0 { return a; }
    gcd(b, a % b)
}

println(gcd(48, 36));   // 12
println(gcd(17, 5));    // 1
```

### 检查素数

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

for n in 1..20 {
    if is_prime(n) {
        print(n, " ");
    }
}
println("");
// 输出:2 3 5 7 11 13 17 19
```

## 下一步

- [列表与字符串](collections.md)
- [多语言互联](../ffi/overview.md)
