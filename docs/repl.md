# REPL 交互模式

## 启动 REPL

```bash
link repl
```

或直接运行 `link`(无参数也会进入 REPL):

```bash
link
```

启动后看到欢迎信息:

```
Link 0.1.0 REPL
Type 'exit' or Ctrl+C to quit
>
```

## 基本用法

每一行输入一条语句,回车立即求值并打印结果:

```
> let x = 42
> x
42
> x * 2
84
> println("hello")
hello
```

!!! tip "隐式返回值"
    每条表达式的结果会自动打印。`let` / `fn` / `extern` 等声明语句返回 `none`,不打印。

## 多行输入

REPL 目前按行求值,**暂不支持**跨行表达式。需要写多行代码请用文件:

```bash
link run myfile.link
```

## 在 REPL 中声明函数

```
> fn add(a: i32, b: i32) -> i32 { a + b }
> add(3, 4)
7
```

## 在 REPL 中使用 FFI

REPL 中可以声明 `extern` 块,后续行即可调用:

```
> extern "C" { fn abs(n: i32) -> i32; }
> abs(-42)
42
```

```
> extern "python" module "math" { fn sqrt(x: f64) -> f64; }
> sqrt(16.0)
4
```

## 退出 REPL

```
> exit
```

或按 `Ctrl+C`。

## 命令一览

| 命令 | 说明 |
|------|------|
| `exit` / `quit` | 退出 REPL |
| `Ctrl+C` | 强制退出 |
| 任意 Link 代码 | 立即求值 |

## 会话示例

下面是一个完整的 REPL 会话示例,展示主要特性:

```
Link 0.1.0 REPL
Type 'exit' or Ctrl+C to quit
> let name = "Link"
> println("Hello,", name)
Hello, Link
> fn fib(n: i32) -> i32 { if n < 2 { return n; } fib(n-1) + fib(n-2) }
> fib(10)
55
> extern "C" { fn abs(n: i32) -> i32; }
> abs(-100)
100
> let nums = [1, 2, 3, 4, 5]
> len(nums)
5
> nums[2]
3
> exit
```

## 下一步

- [变量与类型](basics/types.md)
- [多语言互联](ffi/overview.md)
