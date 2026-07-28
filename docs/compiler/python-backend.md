# Python 代码生成后端

Link 编译器支持将 Link 源代码编译为 Python 代码，然后通过 Python 解释器直接运行。

## 特性

| 类别 | 支持内容 |
|------|---------|
| **基本类型** | i8/i16/i32/i64/u8/u16/u32/u64/usize/f32/f64/bool/str/none |
| **字面量** | 整数、浮点数、字符串、布尔值、none、列表 |
| **运算** | 算术(+ - * / %)、比较(== != < > <= >=)、逻辑(&& \|\| !)、一元(- !) |
| **变量** | let 声明（含类型注解）、赋值 |
| **函数** | 声明、递归、参数、返回值、匿名函数 |
| **控制流** | if/else、while、for、loop、break、continue |
| **复合类型** | struct 定义/初始化/字段访问、enum（带/不带 payload） |
| **模式匹配** | match 语句（通配符、字面量、绑定、枚举变体） |
| **列表** | 字面量、索引访问、迭代 |
| **数据流** | `stream<T>` + 管道运算符 `\|` |
| **异步编程** | async/await |
| **声明式** | flow 声明块 |
| **FFI** | extern "python" 声明 |

## 使用方法

### 生成 Python 代码

```bash
link compile myfile.link --backend python -o output.py
```

也可以使用简写：

```bash
link compile myfile.link --backend py -o output.py
```

### 直接运行

生成的 Python 代码可以直接用 Python 解释器运行：

```bash
python output.py
```

## 生成的 Python 代码示例

### Link 源代码

```link
fn fib(n: i64) -> i64 {
    if n < 2 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

struct Point {
    x: i32,
    y: i32,
}

let p = Point { x: 1, y: 2 };
println("p.x + p.y =", p.x + p.y);
println("fib(10) =", fib(10));

// stream<T> + 管道运算符
let result = stream([1, 2, 3, 4, 5])
    | map(fn(x: i64) -> i64 { return x * 2; })
    | filter(fn(x: i64) -> bool { return x > 5; })
    | collect();
println("stream result =", result);
```

### 生成的 Python 代码

```python
def fib(n):
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)

class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

p = Point(x=1, y=2)
print("p.x + p.y =", p.x + p.y)
print("fib(10) =", fib(10))

# stream<T> + 管道运算符
def _link_stream_map(stream, func):
    return [func(x) for x in stream]

def _link_stream_filter(stream, func):
    return [x for x in stream if func(x)]

def _link_stream_collect(stream):
    return stream

result = _link_stream_collect(
    _link_stream_filter(
        _link_stream_map(
            [1, 2, 3, 4, 5],
            lambda x: x * 2
        ),
        lambda x: x > 5
    )
)
print("stream result =", result)
```

## Python FFI 互操作

Python 后端支持直接调用 Python 模块和函数：

```link
extern "python" module "math" {
    fn sqrt(x: f64) -> f64;
    fn sin(x: f64) -> f64;
}

extern "python" module "os.path" {
    fn exists(path: str) -> bool;
}

let x = sqrt(16.0);
let y = sin(3.14159 / 2.0);
println("sqrt(16) =", x);
println("sin(pi/2) =", y);
```

生成的 Python 代码会自动导入相应的模块：

```python
import math
import os.path

x = math.sqrt(16.0)
y = math.sin(3.14159 / 2.0)
print("sqrt(16) =", x)
print("sin(pi/2) =", y)
```

## 适用场景

Python 后端适合以下场景：

- **快速原型开发**：利用 Python 丰富的生态系统快速验证想法
- **脚本编写**：将 Link 作为更安全的 Python 替代品编写脚本
- **机器学习/数据科学**：结合 NumPy、Pandas、TensorFlow 等库
- **教学与学习**：生成的 Python 代码可读性高，便于理解 Link 的语义
- **跨语言胶水层**：作为连接多种语言的中间层

## 限制

当前 Python 后端暂不支持：
- 借用检查器（Python 本身是 GC 语言）
- 原生性能优化（依赖 Python 解释器性能）
- 低级内存操作（指针、手动内存管理等）
