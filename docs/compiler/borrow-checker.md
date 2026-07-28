# 借用检查器

Link v0.2.0 内置了借用检查器（Borrow Checker），在编译时自动执行，用于保证内存安全，避免常见的内存错误如 use-after-move、双重释放等。

## 核心概念

### 所有权（Ownership）

Link 中的每个值都有且仅有一个**所有者**（owner），当所有者离开作用域时，值会被自动释放。

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

#### 函数传参的移动

```link
fn take_ownership(s: str) {
    println(s);
}  // s 离开作用域，值被释放

fn main() {
    let s = "hello";
    take_ownership(s);  // s 的所有权移动到函数中
    // println(s);      // 错误: use after move
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

#### 哪些类型是 Copy 的？

| 类别 | 类型 |
|------|------|
| 整数 | `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `usize` |
| 浮点数 | `f32`, `f64` |
| 布尔 | `bool` |
| 其他 | 只包含 Copy 类型的 struct |

!!! note "字符串不是 Copy 类型"
    `str` 字符串类型不是 Copy 类型，因为它是堆分配的，复制开销较大。

### 借用（Borrowing）

借用允许你使用值而不获取其所有权，通过引用（`&T` 和 `&mut T`）实现。

#### 不可变借用（&T）

不可变借用允许读取值，但不能修改：

```link
fn print_length(s: &str) {
    println("length:", len(*s));
}  // s 离开作用域，但不释放值，因为它只是借用

fn main() {
    let s = "hello";
    print_length(&s);    // 不可变借用，s 的所有权仍在 main 中
    println(s);          // 正确: s 仍然拥有值
}
```

#### 可变借用（&mut T）

可变借用允许修改值，但同一时间只能有一个可变借用：

```link
fn append_hello(s: &mut str) {
    *s = *s + " world";
}

fn main() {
    let mut s = "hello";
    append_hello(&mut s);  // 可变借用
    println(s);            // 输出: hello world
}
```

## 借用规则

借用检查器遵循以下核心规则：

### 规则 1：同一时间，要么有一个可变借用，要么有任意数量的不可变借用

```link
fn main() {
    let mut s = "hello";

    // 正确: 多个不可变借用
    let r1 = &s;
    let r2 = &s;
    println(r1, r2);

    // 正确: 一个可变借用
    let r3 = &mut s;
    // let r4 = &mut s;  // 错误: 不能同时有两个可变借用

    // 错误: 不可变借用和可变借用不能同时存在
    // let r5 = &s;
    // let r6 = &mut s;
}
```

### 规则 2：借用的生命周期不能长于所有者

```link
fn dangling() -> &str {
    let s = "hello";
    return &s;  // 错误: s 会在函数结束时释放，返回悬垂引用
}
```

### 规则 3：不能在有借用的情况下移动所有权

```link
fn main() {
    let s = "hello";
    let r = &s;       // 借用 s
    // let s2 = s;    // 错误: 不能在有借用时移动 s
    println(r);       // 借用在这里结束
}
```

## use-after-move 检测

借用检查器会在编译时检测 use-after-move 错误：

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

### 编译错误示例

当你尝试使用已移动的值时，编译器会报错：

```
error: use of moved value: `s`
 --> test.link:5:12
  |
3 |     let s2 = s1;
  |              -- value moved here
4 |
5 |     println(s1);
  |             ^^ value used here after move
  |
  = note: `str` does not implement `Copy`
```

## 函数返回与所有权

函数可以返回值，将所有权转移给调用者：

```link
fn create_string() -> str {
    let s = "hello";
    return s;  // s 的所有权转移给调用者
}  // s 离开作用域，但值已被返回，不会被释放

fn main() {
    let s = create_string();  // 获得所有权
    println(s);               // 正确
}  // s 离开作用域，值被释放
```

## 常见模式

### 1. 函数借用而不是获取所有权

如果函数不需要获取值的所有权，应该使用借用：

```link
// 推荐: 使用借用，不获取所有权
fn length(s: &str) -> i64 {
    return len(*s);
}

// 不推荐: 获取所有权，调用者无法再使用
fn length_own(s: str) -> i64 {
    return len(s);
}
```

### 2. 可变借用修改值

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

### 3. 结构体字段的借用

```link
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let p = Point { x: 10, y: 20 };
    let x_ref = &p.x;    // 借用结构体的单个字段
    println(*x_ref);     // 10
}
```

## 作用域与借用结束

借用的生命周期由作用域决定，借用在最后一次使用后结束：

```link
fn main() {
    let mut s = "hello";

    let r1 = &s;         // 不可变借用开始
    println(r1);         // r1 在这里最后一次使用

    let r2 = &mut s;     // 正确: r1 的借用已经结束
    *r2 = *r2 + " world";
    println(r2);
}
```

## 为什么需要借用检查器？

借用检查器在编译时保证内存安全，避免了以下常见错误：

1. **use-after-move**：使用已经移动所有权的值
2. **双重释放**：同一个值被释放两次
3. **悬垂引用**：引用指向已经释放的内存
4. **数据竞争**：多个可变引用同时修改数据

这些错误在 C/C++ 中是运行时错误，难以调试，但在 Link 中会在编译时被捕获。

## 禁用借用检查器

如果确定不需要借用检查器（例如快速原型开发），可以使用 `--no-borrow-check` 选项禁用：

```bash
link compile myfile.link --no-borrow-check
```

!!! warning "不建议禁用"
    禁用借用检查器会失去编译时内存安全保证，可能导致运行时内存错误。建议仅在调试或快速原型时临时使用。

## 与其他语言的对比

| 特性 | Link | Rust | C++ | Python |
|------|------|------|-----|--------|
| 所有权系统 | ✓ | ✓ | ✗ | ✗ |
| 借用检查器 | ✓ | ✓ | ✗ | ✗ |
| 移动语义 | ✓ 默认 | ✓ 默认 | ✓ 需 `std::move` | ✗ |
| Copy 类型 | ✓ | ✓ | ✓ | ✗ |
| 编译时保证 | ✓ | ✓ | ✗ | ✗ |
| 垃圾回收 | ✗ | ✗ | ✗ | ✓ |
