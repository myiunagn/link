# 异步编程(async / await)

Link 支持 `async` / `await` 异步编程语法,用于编写非阻塞的并发代码。

## 基本语法

### async 函数声明

在 `fn` 前加 `async` 关键字声明异步函数:

```link
async fn fetch_data(url: str) -> str {
    sleep(100);  // 模拟 IO 等待
    return "data from " + url;
}

async fn compute(a: i32, b: i32) -> i32 {
    return a + b;
}
```

### await 表达式

用 `await` 调用 async 函数并等待结果:

```link
async fn greet(name: str) -> str {
    return "hello, " + name;
}

let msg = await greet("Link");
println(msg);  // hello, Link
```

`await` 是前缀运算符,优先级与一元运算符(`-` / `!`)相同。

## sleep 异步原语

`sleep(ms)` 是内置的异步原语,阻塞当前线程指定毫秒数:

```link
async fn delayed_greeting(name: str) -> str {
    sleep(100);  // 等待 100ms
    return "hello, " + name;
}

let result = await delayed_greeting("Link");
```

## 链式 await

多个 async 函数可以链式调用:

```link
async fn inc(x: i32) -> i32 { return x + 1; }
async fn double(x: i32) -> i32 { return x * 2; }

let a = await inc(5);      // 6
let b = await double(a);   // 12
println(b);
```

## 在 flow 块中使用

`flow` 声明块的 pipeline 中可以调用 async 函数:

```link
async fn fetch_items() -> list {
    sleep(50);
    return [1, 2, 3, 4, 5];
}

fn double(x: i32) -> i32 { return x * 2; }

flow ProcessItems {
    source: stream(await fetch_items());
    pipeline:
        source | map(double) | collect;
}
```

## v0.1 执行语义

**重要**:v0.1 是树漫游解释器,`async` / `await` 当前是**阻塞语义**:

- `async fn` 标记函数为异步,但执行时仍串行
- `await expr` 等价于直接求值 `expr`(无真正并发)
- `sleep(ms)` 真正阻塞当前线程

这意味着:

```link
async fn task_a() -> i32 { sleep(100); return 1; }
async fn task_b() -> i32 { sleep(100); return 2; }

// v0.1: 串行执行,总耗时 ~200ms
let a = await task_a();
let b = await task_b();
```

**未来 v0.2**:LLVM 后端 + Tokio-like 运行时将实现真正并发,上述代码总耗时将降至 ~100ms。

## 使用场景

async / await 适用于:

- **IO 密集型**:文件读写、网络请求、数据库查询
- **多语言 FFI**:异步调用 Python / Java / HTML 等外部函数
- **数据流水线**:flow 块中异步获取数据源

```link
async fn fetch_env_info() -> str {
    sleep(20);
    return getcwd();  // Python os.getcwd()
}

let env = await fetch_env_info();
println("Working directory:", env);
```

## 完整示例

参见 [examples/async_demo.link](https://github.com/myiunagn/link/blob/main/examples/async_demo.link):

```link
async fn delayed_greeting(name: str) -> str {
    sleep(100);
    return "hello, " + name;
}

async fn compute_sum(a: i32, b: i32) -> i32 {
    sleep(50);
    return a + b;
}

fn main_work() -> i32 {
    let greeting = await delayed_greeting("Link");
    println(greeting);

    let sum = await compute_sum(3, 4);
    println(sum);

    return sum;
}

let result = main_work();
println("done");
```

输出:

```
hello, Link
7
done
```

## 当前限制

- 无真正并发(v0.1 阻塞语义)
- 无 `Promise.all` / `join_all` 等组合子
- 无 `select` / `race` 等选择原语
- `sleep` 是唯一内置异步原语(无 async IO)

这些限制将在 v0.2(LLVM 后端 + 异步运行时)中解决。

## 下一步

- [flow 声明块](flow.md)
- [多语言互联](../ffi/overview.md)
- [多语言 Demo](../examples.md)
