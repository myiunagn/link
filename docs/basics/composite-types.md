# 复合类型: struct / enum / match

Link v0.1.5 引入了对复合类型的支持,包括:

- **struct**: 命名字段集合,用于组织相关数据
- **enum**: 标记联合,表达"多选一"的数据形态
- **match**: 模式匹配,安全地解构 enum 和其他值

## struct 结构体

### 声明

用 `struct` 关键字声明一个结构体,后跟名称和字段列表:

```link
struct Point {
    x: i32,
    y: i32,
}

struct User {
    name: str,
    age: i32,
    active: bool,
}
```

字段类型可以是任何已知的类型注解,包括自定义类型名:

```link
struct Line {
    start: Point,
    end: Point,
}
```

### 实例化

使用 `Name { field: value, ... }` 语法创建实例:

```link
let origin = Point { x: 0, y: 0 };
let p = Point { x: 10, y: 20 };

let u = User {
    name: "Alice",
    age: 30,
    active: true,
};
```

!!! note "字段顺序"
    实例化时字段顺序可以与声明顺序不同,解释器会按名称匹配。

### 字段访问

使用 `.` 运算符访问字段:

```link
let p = Point { x: 10, y: 20 };
println(p.x);   // 10
println(p.y);   // 20

// 在表达式中使用
let sum = p.x + p.y;   // 30
```

### 作为函数参数和返回值

```link
fn make_point(a: i32, b: i32) -> Point {
    return Point { x: a, y: b };
}

fn distance(p: Point) -> i32 {
    return p.x + p.y;
}

let p = make_point(3, 4);
println(distance(p));   // 7
```

### 错误情况

```link
struct Point { x: i32, y: i32 }

// 错误: 未知字段
// let p = Point { x: 1, y: 2, z: 3 };   // Error: Struct Point has no field 'z'

let p = Point { x: 1, y: 2 };
// 错误: 访问不存在的字段
// println(p.z);                          // Error: No such field: z
```

## enum 枚举

### 声明

用 `enum` 关键字声明,变体可以无参数,也可以带参数(称为 payload):

```link
// 简单枚举(无参数)
enum Color {
    Red,
    Green,
    Blue,
}

// 带参数的枚举
enum Shape {
    Circle(f64),
    Rect(i32, i32),
}

// 混合形式
enum Message {
    Quit,                       // 无参数
    Move(i32, i32),             // 带参数
    Write(str),                 // 单参数
    Send(str, str, i32),        // 多参数
}
```

### 构造变体

- **无参数变体**: 使用 `Type::Variant` 路径语法
- **带参数变体**: 使用 `Type::Variant(args...)` 路径调用语法

```link
let red = Color::Red;
let green = Color::Green;

let circ = Shape::Circle(3.14);
let rect = Shape::Rect(10, 20);

let msg = Message::Move(100, 200);
let txt = Message::Write("hello");
```

### 嵌套使用

枚举的 payload 可以是 struct 或其他自定义类型:

```link
struct Point { x: i32, y: i32 }

enum Shape {
    Circle(f64),
    Rect(Point, Point),
    Empty,
}

let p1 = Point { x: 0, y: 0 };
let p2 = Point { x: 10, y: 20 };
let rect = Shape::Rect(p1, p2);
```

### 错误情况

```link
enum Color { Red, Green }
enum Shape { Circle(f64) }

// 错误: 未知变体
// let c = Color::Blue;              // Error: No such variant 'Blue' in enum Color

// 错误: 参数数量不符
// let s = Shape::Circle(1, 2);      // Error: Variant Shape::Circle expects 1 args, got 2
```

## match 模式匹配

`match` 是处理枚举值的主要方式,也可以用于字面量匹配。它既是语句,也是表达式。

### 基本语法

```link
match scrutinee {
    pattern => { body }
    pattern => { body }
    _ => { default }
}
```

### 匹配枚举变体

```link
enum Color { Red, Green, Blue }

let c = Color::Green;
match c {
    Color::Red => { println("红色") }
    Color::Green => { println("绿色") }
    Color::Blue => { println("蓝色") }
}
```

### 解构带参数的变体

使用 `Type::Variant(binding1, binding2, ...)` 解构并绑定 payload:

```link
enum Message {
    Quit,
    Move(i32, i32),
    Write(str),
}

let m = Message::Move(10, 20);
match m {
    Message::Quit => { println("退出") }
    Message::Move(x, y) => {
        println("移动到");
        println(x);
        println(y);
    }
    Message::Write(text) => {
        println(text);
    }
}
```

绑定的变量在 arm 的 body 内可用。使用 `_` 作为绑定名表示忽略该参数:

```link
match m {
    Message::Move(_, y) => { println(y) }   // 只关心 y
    _ => { println("其他") }
}
```

### 通配符 `_`

`_` 匹配任何值且不绑定变量,常作为最后一个 arm 的兜底:

```link
let c = Color::Blue;
match c {
    Color::Red => { 1 }
    _ => { 99 }       // 匹配 Green 和 Blue 等所有其他情况
}
```

### 字面量模式

match 不限于枚举,也可以匹配整型、字符串、布尔等字面量:

```link
let n = 5;
let label = match n {
    1 => { "一" }
    5 => { "五" }
    _ => { "其他" }
};
println(label);
```

```link
let s = "hello";
match s {
    "hi" => { println("打招呼") }
    "hello" => { println("你好") }
    _ => { println("未知") }
}
```

### match 作为表达式

match 可以用在 `let`、函数返回值、参数等任何需要表达式的地方:

```link
// 用作 let 的右值
let n = 5;
let label = match n {
    1 => { 100 }
    5 => { 200 }
    _ => { 999 }
};

// 用作函数返回值
fn classify(c: Color) -> i32 {
    return match c {
        Color::Red => { 1 }
        Color::Green => { 2 }
        _ => { 0 }
    };
}
```

!!! warning "匹配必须完备"
    如果没有任何 arm 匹配,运行时会报错 `No match arm matched value`。
    建议始终用 `_` 作为最后一个 arm 兜底。

### 完整示例

```link
struct Point { x: i32, y: i32 }

enum Shape {
    Circle(f64),
    Rect(Point, Point),
    Empty,
}

fn area(s: Shape) -> f64 {
    return match s {
        Shape::Empty => { 0.0 }
        Shape::Circle(r) => { 3.14159 * r * r }
        Shape::Rect(a, b) => {
            let w = b.x - a.x;
            let h = b.y - a.y;
            return (w * h) as f64;
        }
    };
}

let p1 = Point { x: 0, y: 0 };
let p2 = Point { x: 10, y: 20 };
let rect = Shape::Rect(p1, p2);
let circ = Shape::Circle(5.0);

println(area(rect));    // 200
println(area(circ));    // 78.5...
```

## 设计说明

### 类型注册

struct 和 enum 的定义在执行到声明语句时注册到解释器的类型表中:

- `ctx.struct_defs: HashMap<String, Vec<StructField>>`
- `ctx.enum_defs: HashMap<String, Vec<EnumVariantDecl>>`

实例化和变体构造时会查询这些表进行验证。

### 模式种类

当前支持的 Pattern:

| 模式 | 语法 | 说明 |
|------|------|------|
| 通配符 | `_` | 匹配任何值 |
| 绑定 | `name` | 匹配任何值并绑定到变量 |
| 字面量 | `42` / `"hi"` / `true` / `none` | 按值相等匹配 |
| 枚举变体 | `Color::Red` | 匹配无参数变体 |
| 带参数变体 | `Color::RGB(r, g, b)` | 解构变体并绑定 payload |

### 限制

v0.1.5 的复合类型实现有以下限制,后续版本会改进:

- 不支持方法(`impl` 块),struct/enum 只承载数据
- 不支持泛型(如 `List<T>`)
- 不支持 trait
- struct 字段不可变,无 `mut` 修饰
- match 不支持嵌套模式(如 `Some(Some(x))`)
- match 不支持范围模式(如 `1..=10`)
- match 不支持 OR 模式(如 `1 | 2`)

## 下一步

- [控制流](control-flow.md)
- [函数](functions.md)
- [FFI 概览](../ffi/overview.md)
