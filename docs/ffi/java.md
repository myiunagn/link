# Java 互连

!!! abstract "概述"
    Link 通过子进程 + JSON-RPC 桥接器调用 Java 静态方法。
    需要本机安装 JDK,并提供一个 Java 桥接器类。

## 基本用法

### module 字段格式

Java FFI 的 `module` 字段格式为 `"<class_path>::<class_name>"`:

```link
extern "java" module "build/classes::com.example.MathUtils" {
    fn add(a: i64, b: i64) -> i64;
    fn multiply(a: i64, b: i64) -> i64;
}
```

- `class_path`:Java classpath 路径(编译后的 `.class` 文件所在目录)
- `class_name`:包含静态方法的 Java 类全限定名

### 调用示例

```link
extern "java" module "build/classes::com.example.MathUtils" {
    fn add(a: i64, b: i64) -> i64;
    fn factorial(n: i64) -> i64;
    fn greet(name: str) -> str;
}

let sum = add(10, 20);        // 30
let fact = factorial(5);      // 120
let msg = greet("Link");      // Hello, Link!
```

## 类型映射

| Link 类型 | Java 类型 | JSON 传输类型 |
|-----------|-----------|---------------|
| `i32` | `int` / `Integer` | JSON number |
| `i64` | `long` / `Long` | JSON number |
| `f32` | `float` / `Float` | JSON number |
| `f64` | `double` / `Double` | JSON number |
| `bool` | `boolean` / `Boolean` | JSON boolean |
| `str` | `String` | JSON string |

## 创建 Java 桥接器

### Java 桥接器类

Link 会通过 `java` 命令运行桥接器类,桥接器从 stdin 读取 JSON 请求,调用对应的静态方法,输出 JSON 响应:

```java
// src/com/example/MathUtils.java
package com.example;

import java.util.Scanner;

public class MathUtils {
    public static long add(long a, long b) {
        return a + b;
    }

    public static long factorial(long n) {
        if (n <= 1) return 1;
        return n * factorial(n - 1);
    }

    public static String greet(String name) {
        return "Hello, " + name + "!";
    }

    // 桥接器入口
    public static void main(String[] args) {
        Scanner scanner = new Scanner(System.in);
        String input = scanner.nextLine();

        // 解析 JSON 请求并调用对应方法
        // 输出 JSON 响应: {"result": <value>}
        // ...
    }
}
```

### 编译 Java 类

```bash
mkdir -p build/classes
javac -d build/classes src/com/example/MathUtils.java
```

## 完整示例

```link
extern "java" module "build/classes::com.example.MathUtils" {
    fn add(a: i64, b: i64) -> i64;
    fn factorial(n: i64) -> i64;
    fn greet(name: str) -> str;
}

// 组合 Java 函数
fn sum_range(start: i64, end: i64) -> i64 {
    let total = 0;
    for i in start..end {
        total = add(total, i);
    }
    total
}

println("1+2+...+10 =", sum_range(1, 11));  // 55
println("5! =", factorial(5));               // 120
println("问候:", greet("Link"));             // Hello, Link!
```

## 性能特点

- **子进程开销**:每次调用都会启动新的 JVM 进程,有启动开销
- **适合批量调用**:如果需要多次调用,考虑在 Java 侧缓存结果
- **JSON 序列化**:参数和返回值通过 JSON 传递,有序列化开销

!!! tip "性能优化建议"
    Java FFI 适合调用计算密集型任务(如大数据处理、复杂算法),不适合高频小调用。

## 前置条件

- 本机已安装 JDK (建议 JDK 11+)
- `java` 命令在 PATH 中可用
- Java 桥接器类已编译

## 常见问题

### 找不到类

确保 `class_path` 指向包含 `.class` 文件的目录,`class_name` 使用全限定名(含包名)。

### JVM 启动失败

检查 JDK 是否正确安装,`java -version` 能正常输出。

## 下一步

- [多语言互联概述](overview.md)
- [进程桥接互连](process.md)
