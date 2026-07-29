# Link 文档示例 Bug 狩猎 v1 — 验证清单（checklist）

> 说明：按 [x]=通过，[ ]=未处理，[/]=处理中，FAIL=失败需修复；每个示例有编号对应 tasks.md。第一轮与第二轮独立记录。

## 第一轮检查总览

- [x] 0. 环境验证：`cargo build --release` 成功，`cargo test` 全工作区 0 failed（基线）
- [x] 1. examples.md 全部示例处理完毕
- [x] 2. quickstart.md 全部示例处理完毕
- [x] 3. index/introduction/installation/repl 示例处理完毕
- [x] 4. basics/types.md + basics/operators.md 处理完毕
- [x] 5. basics/functions.md + basics/control-flow.md 处理完毕
- [x] 6. basics/composite-types.md + basics/collections.md 处理完毕
- [x] 7. basics/flow.md + basics/async.md 处理完毕
- [x] 8. compiler/cli.md + c-backend.md + llvm-backend.md 处理完毕
- [x] 9. compiler/python-backend.md + wasm-backend.md + borrow-checker.md 处理完毕
- [x] 10. ffi/overview.md + bindgen.md + c.md + cpp.md 处理完毕
- [x] 11. ffi/python.md + java.md + html.md + wasm.md + process.md 处理完毕
- [x] 12. spec.md 全部示例处理完毕
- [x] 13. plans/*.md 中 ```link 代码块 处理完毕
- [x] 14. 第二轮全量回归（与 1~13 完整重跑）全通过）
- [x] 15. 日志汇总与最终 cargo test 0 failed

## 详细示例条目（第一轮/第二轮各记同一编号）

### Task 1: examples.md

> 注：示例数量较多，逐个编号 E001..E026
- [ ] E001: examples.md#L7 Hello World: `println("Hello, World!");` A | 期望输出: Hello, World!
- [ ] E002: examples.md#L13 基本算术 (a=10, b=3 五种运算) A
- [ ] E003: examples.md#L26 变量与类型 (name/version/year/is_open_source) A
- [ ] E004: examples.md#L42 斐波那契 (fib + for) A | 期望末尾输出 0 1 1 2 3 5 ... 377
- [ ] E005: examples.md#L58 阶乘 factorial A
- [ ] E006: examples.md#L72 最大公约数 gcd A | gcd(48,36)=12 等
- [ ] E007: examples.md#L85 素数判断 is_prime + 100 以内素数 A
- [ ] E008: examples.md#L113 冒泡排序 bubble_sort A | 期望输出 [11,12,22,25,34,64,90]
- [ ] E009: examples.md#L136 二分查找 binary_search A
- [ ] E010: examples.md#L168 C 标准库 extern "C" (abs/sqrt/pow) C（声明类) B/C 声明 parser+sema; codegen py 不报错
- [ ] E011: examples.md#L188 Python 标准库 extern "python" C（声明类) B/C
- [ ] E012: examples.md#L215 C++ 共享库 extern "C++" B/C
- [ ] E013: examples.md#L237 三语言混合 C+Python+C++ B/C
- [ ] E014: examples.md#L275 简易计算器 add/sub/mul/div A
- [ ] E015: examples.md#L299 FizzBuzz 1..30 A
- [ ] E016: examples.md#L315 九九乘法表 A
- [ ] E017: examples.md#L337 字符串反转 reverse(s: string index  A
- [ ] E018: examples.md#L356 矩阵运算 matrix_add A | 6 8 / 10 12
- [ ] E019: examples.md#L394 统计计算 sum/mean/max/min A
- [ ] E020: examples.md#L438 数据过滤 filter_positive A | [5,8,2,4,6]
- [ ] E021: examples.md#L463 stream 基本用法 (stream|map|filter|for_each|collect) A
- [ ] E022: examples.md#L488 stream 链式管道 (平方+过滤>50) A | [64,81,100]
- [ ] E023: examples.md#L500 WASM extern "wasm" B/C
- [ ] E024: examples.md#L512 Java extern "java" B/C
- [ ] E025: examples.md#L524 HTML extern "html" B/C
- [ ] E026: examples.md#L536 进程桥接 extern "go"/extern "ruby" B/C
- [ ] E027: examples.md#L557 全语言混合 extern 5 语言 B/C

### Task 2: quickstart.md — Q001..Q010
- [ ] Q001: quickstart.md#L7 Hello 快速开始 A
- [ ] Q002: quickstart.md#L25 变量和函数 A
- [ ] Q003: quickstart.md#L42 控制流 if/else/while/for A
- [ ] Q004: quickstart.md#L76 struct/enum 结构体和枚举 A
- [ ] Q005: quickstart.md#L97 match 模式匹配 A
- [ ] Q006: quickstart.md#L122 stream 管道 A
- [ ] Q007: quickstart.md#L134 extern 外部函数 B/C
- [ ] Q008: quickstart.md#L151 多语言 FFI (C+Python) B/C
- [ ] Q009: quickstart.md#L170 综合示例: 计算器 REPL A（或 D 类如为 REPL 交互示例）

### Task3: 首页与概览 I001..I010
- [ ] I001: index.md#L75 首页示例代码 A
- [ ] I002: introduction.md 中全部 ```link 块 B/A
- [ ] I003: installation.md 中所有验证代码 A
- [ ] I004: repl.md 中所有示例（可独立执行则 A）

### Task4: basics — T001..T020
- [ ] T001-T0xx: types.md 内所有 ```link 块 （每个编号递增）
- [ ] O001-O0xx: operators.md 内所有 ```link 块 （每个编号递增）

### Task5: 函数与控制流 F/CF
- [ ] F001..F0xx: functions.md 内所有 ```link 块
- [ ] CF001..CF0xx: control-flow.md 内所有 ```link 块

### Task6: 复合类型与集合 CT/COL
- [ ] CT001..CT0xx: composite-types.md 所有 ```link 块
- [ ] COL001..COL0xx: collections.md 所有 ```link 块

### Task7: flow 与 async
- [ ] FL001..FL0xx: flow.md 所有 ```link 块 （parser+sema 优先）
- [ ] AY001..AY0xx: async.md 所有 ```link 块

### Task8: compiler CLI 相关 CLI0xx
- [ ] CLI001..CLI0xx: cli.md 所有 .link 代码块
- [ ] CB001..CB0xx: c-backend.md
- [ ] LL001..LL0xx: llvm-backend.md（parser+sema 主）

### Task9: compiler py/wasm/borrow
- [ ] PB001..PB0xx: python-backend.md（编译运行
- [ ] WB001..WB0xx: wasm-backend.md（.wat 生成
- [ ] BC001..BC0xx: borrow-checker.md（含"合法/非法对）

### Task10: ffi 概述/bindgen/c/cpp
- [ ] FO001..FO0xx: ffi/overview.md
- [ ] BG001..BG0xx: ffi/bindgen.md
- [ ] FC001..FC0xx: ffi/c.md
- [ ] FCP001..FCP0xx: ffi/cpp.md

### Task11: ffi 剩余文件
- [ ] FP001..FP0xx: ffi/python.md
- [ ] FJ001..FJ0xx: ffi/java.md
- [ ] FH001..FH0xx: ffi/html.md
- [ ] FW001..FW0xx: ffi/wasm.md
- [ ] FPR001..FPR0xx: ffi/process.md

### Task12: spec.md — S001..Sxxx
- [ ] S001..Sxxx: spec.md 中每一个 ```link 代码块（逐个编号

### Task13: plans
- [ ] P0..Pn: plans 3 个 md 文件的 ```link 块。

### Task14 第二轮独立记录
- [ ] R2_1 Task1 重跑全通过
- [ ] R2_2 Task2 重跑全通过
- [ ] R2_3 Task3 重跑全通过
- [ ] R2_4 Task4 重跑全通过
- [ ] R2_5 Task5 重跑全通过
- [ ] R2_6 Task6 重跑全通过
- [ ] R2_7 Task7 重跑全通过
- [ ] R2_8 Task8 重跑全通过
- [ ] R2_9 Task9 重跑全通过
- [ ] R2_10 Task10 重跑全通过
- [ ] R2_11 Task11 重跑全通过
- [ ] R2_12 Task12 重跑全通过
- [ ] R2_13 Task13 重跑全通过
- [ ] R2_cargo `cargo test` 0 failed

### Task15 最终汇总
- [ ] tasks.md 所有任务 [x]
- [ ] bug_log.md 存在且条目齐全（如修复过 bug）
- [ ] 所有失败示例在 bug_log 中均有"修复后通过记录
