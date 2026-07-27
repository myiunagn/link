# 变量与类型

## 基本类型

Link v0.1 提供以下基本类型:

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

Link 是静态类型语言,但 v0.1 解释器对类型检查较宽松,主要在以下时机报错:

- 索引越界
- 除零
- 调用函数时参数数量不符
- 对错误类型调用方法(如对 `int` 调 `len()`)

未来 LLVM 后端会引入完整类型检查。

## 下一步

- [运算符](operators.md)
- [控制流](control-flow.md)
- [复合类型: struct / enum / match](composite-types.md)
