# flow 声明块

`flow` 是 Link 的声明式数据流定义语法,用于把"数据源 → 变换 → 消费"组织成一条可读的管道。

## 基本语法

```link
flow FlowName "可选描述" {
    source: <expr>;
    sample: every 1s;          // 可选,v0.1 解析但忽略
    pipeline:
        <pipeline expr>;
}
```

- `flow` 后跟名称(必填)
- 名称后可选字符串字面量作为描述
- `source:` 字段可选,其表达式求值后绑定到子作用域中的 `source` 变量
- `sample:` 字段可选,v0.1 仅解析不执行(时间调度规划在 Phase 1.8)
- `pipeline:` 字段必填,通常是一条 `|` 管道链

## 最简示例

```link
fn double(x: i32) -> i32 { return x * 2; }

flow DoubleAll {
    source: stream([1, 2, 3]);
    pipeline:
        source | map(double) | collect;
}
```

执行后返回 `[2, 4, 6]`。

## 字段说明

### source

`source:` 字段为 flow 提供数据源,其值会绑定到子作用域中的 `source` 变量,供 pipeline 引用:

```link
flow GreetAll {
    source: stream(["alice", "bob"]);
    pipeline:
        source | for_each(print_name);
}
```

source 字段可省略,此时 pipeline 内可以直接内联 stream 表达式:

```link
flow Inline {
    pipeline:
        stream([1, 2, 3]) | map(double) | collect;
}
```

### sample

`sample:` 字段用于声明采样策略(如 `every 1s`)。v0.1 解释器暂未实现时间调度,该字段会被解析但不影响执行:

```link
flow Sampled {
    source: stream([100, 200, 300]);
    sample: every 1s;
    pipeline:
        source | for_each(print_val);
}
```

时间调度与异步执行规划在 Phase 1.8(异步运行时)中实现。

### pipeline

`pipeline:` 是 flow 的核心,定义数据如何变换和消费。它通常是一条由 `|` 连接的管道链:

```link
flow SquareEvens "平方偶数" {
    source: stream([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    pipeline:
        source | filter(is_even) | map(square) | collect;
}
```

可用算子见 [stream 文档](../basics/types.md#列表) 与 [复合类型](../basics/composite-types.md)。常用算子:

- `stream(list)` —— 从列表创建流
- `map(fn)` —— 元素变换
- `filter(fn)` —— 过滤
- `for_each(fn)` —— 遍历消费(返回 none)
- `collect` —— 收集为列表

## 执行语义

v0.1 是树漫游解释器,没有真正的并发调度。`flow` 块的当前执行语义:

1. 遇到 `flow` 声明时**立即执行**(不是声明后注册等待调用)
2. 为 flow 创建独立子作用域,`source` 变量不会泄露到外部
3. 若有 `source:` 字段,求值后绑定到 `source`
4. 求值 `pipeline:` 表达式,其返回值作为整个 flow 块的返回值
5. 多个 flow 按源码出现顺序串行执行

```link
flow First {
    source: stream([1, 2]);
    pipeline: source | map(double) | collect;   // [2, 4]
}
flow Second {
    source: stream([1, 2]);
    pipeline: source | map(triple) | collect;   // [3, 6]
}
// 最终返回 Second 的结果 [3, 6]
```

未来 Phase 1.8 引入异步运行时后,多个 flow 将自动并行调度。

## 作用域隔离

`source` 变量只在 flow 块内部可见,不会污染外部作用域:

```link
flow F {
    source: stream([1, 2, 3]);
    pipeline: source | collect;
}
// 此处引用 source 会报错:Undefined variable: source
println(source);   // Error
```

## 完整示例

```link
fn double(x: i32) -> i32 { return x * 2; }
fn is_even(x: i32) -> bool { return x % 2 == 0; }
fn square(x: i32) -> i32 { return x * x; }
fn greet(name: str) { println("hello, " + name); }
fn print_val(v: i32) { println(v); }

// for_each 消费流
flow HelloFlow "问候流" {
    source: stream(["alice", "bob", "charlie"]);
    pipeline:
        source | for_each(greet);
}

// map + filter + collect 链式处理
flow SquareEvens "平方偶数" {
    source: stream([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    pipeline:
        source | filter(is_even) | map(square) | collect;
}

// 无 source,pipeline 内联 stream
flow InlineFlow {
    pipeline:
        stream([1, 2, 3]) | map(double) | collect;
}
```

输出:

```
hello, alice
hello, bob
hello, charlie
[2, 4, 6]
```

## 当前限制

v0.1 的 flow 实现有以下限制,后续版本会改进:

- 不支持 `sample: every 1s` 时间调度(Phase 1.8)
- 不支持跨 flow 引用(如 `flow B { pipeline: A.output | ... }`)
- 不支持 `window` / `aggregate` / `sink` 等高级算子
- 不支持闭包字面量 `filter(x) { x > 0 }`,需先声明具名函数
- 多个 flow 串行执行,无并行调度

## 下一步

- [stream 类型](../basics/types.md#列表)
- [复合类型](../basics/composite-types.md)
- [设计规格](../spec.md)
