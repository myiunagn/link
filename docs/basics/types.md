# 变量与类型

## 基本类型

Link v0.2.0 提供以下基本类型:

| 类型 | 关键字 | 示例 | 说明 |
|------|--------|------|------|
| 整数 | `i32` / `i64` / `u32` / `u64` / `i8` / `i16` / `u8` / `u16` / `usize` | `42` | 默认 `i64` |
| 浮点 | `f32` / `f64` | `3.14` | 默认 `f64` |
| 字符串 | `str` | `"hello"` | UTF-8 |
| 布尔 | `bool` | `true` / `false` | |
| 空 | `none` | `none` | 类似 null / unit |
| 列表 | `list<T>` | `[1, 2, 3]` | 动态长度 |

## 变量声明

用 `let` 声明变量,类型注解可选:

```link
// 类型推导
let x = 42;             // i64
let pi = 3.14;          // f64
let name = "Link";      // str
let flag = true;        // bool
let nums = [1, 2, 3];   // list<i64>

// 显式类型注解
let count: i32 = 100;
let ratio: f64 = 0.5;
let label: str = "demo";
let ready: bool = false;
```

## 整数字面量

```link
let decimal = 42;
let negative = -17;
let zero = 0;
```

!!! note "整数字面量"
    v0.1 暂不支持十六进制 `0x..` / 二进制 `0b..` / 下划线分隔。后续版本会加入。

## 浮点字面量

```link
let pi = 3.14159;
let e = 2.71828;
let half = 0.5;
let big = 1.5e3;        // 1500.0
```

## 字符串字面量

```link
let greeting = "Hello, World!";
let empty = "";
let path = "C:\\Users\\name";     // 转义反斜杠
let quote = "She said \"hi\"";    // 转义引号
let newline = "line1\nline2";    // 换行
let tab = "a\tb";                // 制表符
```

### 转义序列

| 转义 | 含义 |
|------|------|
| `\n` | 换行 |
| `\t` | 制表符 |
| `\\` | 反斜杠 |
| `\"` | 双引号 |
| `\0` | 空字符 |

## 布尔值

```link
let is_true = true;
let is_false = false;

// 逻辑运算
let a = true && false;   // false
let b = true || false;   // true
let c = !true;           // false
```

## none 值

`none` 表示"没有值",类似其他语言的 `null` / `nil` / `unit`:

```link
let x = none;
let y: i32 = 0;   // 显式赋值

if x == none {
    println("x 是空");
}
```

!!! warning "类型安全"
    `none` 不能与有类型的变量直接运算,会报错。但可以用于比较和判断。

## 列表

```link
let nums = [1, 2, 3, 4, 5];
let strings = ["a", "b", "c"];
let mixed = [1, "two", 3.0, true];   // 异构(动态类型)
let nested = [[1, 2], [3, 4]];       // 嵌套
let empty = [];
```

### 列表操作

```link
let nums = [10, 20, 30];

// 索引(从 0 开始)
let first = nums[0];     // 10
let last = nums[2];      // 30

// 长度
let n = len(nums);       // 3

// 索引越界会报错
// let bad = nums[10];   // Error: Index 10 out of bounds for list of length 3
```

详见 [列表与字符串](collections.md)。

## 类型转换

v0.1 暂未提供显式类型转换操作符。运算时会自动按以下规则处理:

- `int + int` → `int`
- `int + float` / `float + int` → `float`
- `int == float` → 比较(自动转换)

```link
let i = 5;
let f = 2.5;
let sum = i + f;         // 7.5 (float)
let div = i / 2;         // 2 (int / int = int)
let fdiv = i as f64 / 2.0;  // 2.5 (`as` 转换 — 规划中)
```

## 类型注解的位置

类型注解出现在以下位置:

```link
// 1. let 声明
let x: i32 = 42;

// 2. 函数参数
fn add(a: i32, b: i32) -> i32 { a + b }

// 3. 函数返回值
fn pi() -> f64 { 3.14159 }

// 4. extern 声明
extern "C" {
    fn abs(n: i32) -> i32;
}
```

## 类型检查

Link 是静态类型语言，v0.2.0 内置了完整的类型检查器（54 个测试用例），在编译时进行类型检查：

- 索引越界
- 除零
- 调用函数时参数数量不符
- 类型不匹配
- 对错误类型调用方法(如对 `int` 调 `len()`)

## 所有权与借用

Link v0.2.0 引入了所有权系统和借用检查器，在编译时保证内存安全。

!!! note "详细文档"
    更详细的说明请参考 [借用检查器文档](../compiler/borrow-checker.md)。

### 所有权（Ownership）

Link 中的每个值都有且仅有一个**所有者**，当所有者离开作用域时，值会被自动释放。

```link
fn main() {
    let s = "hello";  // s 是字符串的所有者
    println(s);       // 使用 s
}  // s 离开作用域，字符串被释放
```

### 移动语义（Move Semantics）

默认情况下，赋值和函数传参会**移动**所有权，原变量不再可用：

```link
fn main() {
    let s1 = "hello";
    let s2 = s1;      // s1 的所有权移动给 s2
    // println(s1);   // 错误: use after move - s1 已不再拥有值
    println(s2);      // 正确: s2 现在拥有值
}
```

### Copy 类型

基础类型（整数、浮点数、布尔值）是 **Copy 类型**，它们在赋值时会复制一份，而不是移动所有权：

```link
fn main() {
    let x: i32 = 42;
    let y = x;      // x 是 Copy 类型，复制一份给 y
    println(x);     // 正确: x 仍然可用
    println(y);     // 正确: y 是独立的副本
}
```

#### Copy 类型列表

| 类别 | 类型 |
|------|------|
| 整数 | `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `usize` |
| 浮点数 | `f32`, `f64` |
| 布尔 | `bool` |

非 Copy 类型（如 `str`、`list<T>`、自定义 struct 等）在赋值和传参时会移动所有权。

### 借用（Borrowing）

借用允许你使用值而不获取其所有权，通过引用实现。

#### 不可变借用（&T）

不可变借用允许读取值，但不能修改：

```link
fn print_length(s: &str) {
    println("length:", len(*s));
}

fn main() {
    let s = "hello";
    print_length(&s);    // 不可变借用，s 的所有权仍在 main 中
    println(s);          // 正确: s 仍然拥有值
}
```

#### 可变借用（&mut T）

可变借用允许修改值：

```link
fn increment(x: &mut i32) {
    *x = *x + 1;
}

fn main() {
    let mut count: i32 = 0;
    increment(&mut count);
    println(count);  // 1
}
```

### 借用检查器基本规则

借用检查器在编译时执行，遵循以下规则：

1. **同一时间，要么有一个可变借用，要么有任意数量的不可变借用**
2. **借用的生命周期不能长于所有者**
3. **不能在有借用的情况下移动所有权**

#### 规则 1 示例：不可变借用可以多个

```link
fn main() {
    let s = "hello";
    let r1 = &s;  // 第一个不可变借用
    let r2 = &s;  // 第二个不可变借用 - 正确
    println(r1, r2);
}
```

#### 规则 1 示例：可变借用只能一个

```link
fn main() {
    let mut s = "hello";
    let r1 = &mut s;
    // let r2 = &mut s;  // 错误: 不能同时有两个可变借用
    println(r1);
}
```

#### 规则 1 示例：不可变和可变借用不能共存

```link
fn main() {
    let mut s = "hello";
    let r1 = &s;
    // let r2 = &mut s;  // 错误: 不可变借用和可变借用不能同时存在
    println(r1);
}
```

### use-after-move 检测

借用检查器会在编译时检测 use-after-move 错误，避免运行时内存问题：

```link
struct Person {
    name: str,
    age: i32,
}

fn main() {
    let p1 = Person { name: "Alice", age: 30 };
    let p2 = p1;              // p1 的所有权移动给 p2
    // println(p1.name);      // 错误: use after move
    println(p2.name);         // 正确
}
```

### 为什么需要所有权和借用？

所有权系统和借用检查器在编译时保证内存安全，无需垃圾回收（GC），也避免了常见的内存错误：

- use-after-move / use-after-free
- 双重释放
- 悬垂引用
- 数据竞争

这使得 Link 既能拥有 C++ 级别的性能，又能拥有内存安全保证。

## 下一步

- [运算符](operators.md)
- [控制流](control-flow.md)
- [复合类型: struct / enum / match](composite-types.md)
