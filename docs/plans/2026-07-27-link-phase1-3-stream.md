# Phase 1.3: stream<T> 核心类型 实施计划

> **目标**: 实现 Link 的灵魂类型 `stream<T>`，支持创建、变换（map/filter）、消费（for_each/collect）和管道运算符 `|`。

**架构**: 解释器层面扩展 `Value::Stream`，内置高阶函数实现流操作。

---

## 设计决策

### MVP 范围
- `stream(list)` — 从列表创建流
- `map(stream, fn)` — 对每个元素应用函数
- `filter(stream, fn)` — 过滤元素
- `for_each(stream, fn)` — 遍历消费
- `collect(stream)` — 流转回列表
- `|` 管道运算符 — 语法糖，`a | b` 等价于 `b(a)`

### 不做的（后续版本）
- 方法调用语法（`s.map(fn)`）
- 惰性求值（当前立即求值）
- 异步流
- window/aggregate/join 等高级算子

---

## Task 1: Lexer — 添加 Stream 关键字

**Files:** `crates/linkc_lexer/src/lib.rs`

- [x] `Token` 枚举添加 `Stream`
- [x] `Display` 添加 `stream`
- [x] `lex_ident_or_keyword` 添加 `"stream" => Token::Stream`
- [x] 测试

---

## Task 2: Parser — stream 类型 + 管道运算符

**Files:** `crates/linkc_parser/src/lib.rs`

- [x] `TypeAnnotation` 添加 `Stream(Box<TypeAnnotation>)`
- [x] `BinOp` 添加 `Pipe`
- [x] 表达式优先级链底部添加 `parse_pipe`
- [x] `|` 右结合，`a | b | c` = `a | (b | c)`
- [x] 测试

---

## Task 3: Interpreter — Stream Value + 内置函数

**Files:** `crates/linkc_interpreter/src/lib.rs`

- [x] `Value` 添加 `Stream(Vec<Value>)`
- [x] `PartialEq`、`type_name`、`value_to_string` 扩展
- [x] 内置函数:
  - `stream(list) -> Stream` — 从列表创建
  - `map(stream, fn) -> Stream` — 映射
  - `filter(stream, fn) -> Stream` — 过滤
  - `for_each(stream, fn) -> none` — 遍历
  - `collect(stream) -> list` — 收集
- [x] 管道运算符 `|` 求值: 左操作数作为右操作数的第一个参数
  - `s | map(double)` → `map(s, double)`
  - 支持 `a | b(c)` → `b(a, c)`
- [x] 测试

---

## Task 4: CLI + 文档

- [x] CLI `print_value` 添加 Stream 输出
- [x] 添加测试用例
- [x] 全量回归测试

---

## 验收标准

- [x] `stream([1,2,3])` 可创建流
- [x] `map(stream([1,2,3]), double)` 返回 `[2,4,6]`
- [x] `filter(stream([1,2,3,4,5]), is_even)` 返回 `[2,4]`
- [x] `collect(map(stream([1,2,3]), double))` 返回列表
- [x] `[1,2,3] | stream | map(double) | collect` 管道链可用
- [x] 所有原有测试通过
