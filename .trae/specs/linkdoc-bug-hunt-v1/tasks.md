# Link 文档示例 Bug 狩猎 v1 — 任务分解与执行计划

说明：每一个 md 文档对应一个 Task；每一个 Task 内部有多个代码块（按行号索引），分类 A/B/C/D 处理。每个 Task 独立完成后推进下一个。两轮重复执行。

## [x] Task 1: examples.md — 完整示例清单（A+B+C+D 类最集中，最高优先级）
- **Priority**: high
- **Depends On**: None
- **Description**:
  - 处理 `d:\linkdoc\docs\examples.md` 中的所有 ```link 代码块
  - 涵盖: Hello World, 基本算术, 变量与类型, 斐波那契, 阶乘, gcd, 素数判断, 冒泡排序, 二分查找, C/Python/C++ FFI, 三语言混合, 简易计算器, FizzBuzz, 九九乘法表, 字符串反转, 矩阵运算, 统计计算, 数据过滤, stream 数据流, WebAssembly/Java/HTML/Go 互连, 全语言混合, 等
  - A 类示例: `link run` 或 `link compile --backend py + python` 执行并与注释中期望输出对比
  - FFI 外部依赖类 (C/Python/C++/wasm/java/html/go): 必须 parser+sema 通过 + Python 后端 codegen 通过（生成 .py 不报错）；不强行加载外部 DLL 运行
- **Acceptance Criteria Addressed**: AC-1, AC-2
- **Test Requirements**:
  - `programmatic` TR-1.1: `examples.md` 中每份 A 类源码单文件执行 exit code = 0，且关键输出（注释中的 `// 5`、`// 输出:` 等）匹配；不匹配或崩溃即失败
  - `programmatic` TR-1.2: B/C 类示例 parser + sema 不报错；C 类可被 Python backend 代码生成（生成 .py 不报错）
  - `programmatic` TR-1.3: 每一个被修过的 bug 回归后 `cargo test` 全工作区 0 failed
- **Notes**: 本文件示例最多，预计会发现最多 bug，优先处理

---

## [x] Task 2: quickstart.md — 快速入门文档
- **Priority**: high
- **Depends On**: None（可与 Task1 串行）
- **Description**:
  - 处理 `d:\linkdoc\docs\quickstart.md` 所有 ```link 代码块
  - 涉及 HelloWorld、变量与函数、if/while/for、struct/enum、stream 管道、extern 声明、FFI 示例
- **Acceptance Criteria Addressed**: AC-1, AC-2
- **Test Requirements**:
  - `programmatic` TR-2.1: 所有 A 类示例 exit code 0，输出匹配文档注释期望
  - `programmatic` TR-2.2: B/C 类 parser+sema 通过 + Python backend codegen 通过

---

## [x] Task 3: index.md, introduction.md, installation.md, repl.md — 首页与概览类文档
- **Priority**: medium
- **Depends On**: None
- **Description**:
  - `d:\linkdoc\docs\index.md`: 首页示例（hello, stream, ffi 小片段 等）
  - `d:\linkdoc\docs\introduction.md`: 介绍页示例
  - `d:\linkdoc\docs\installation.md`: 安装验证（可能有 hello 示例）
  - `d:\linkdoc\docs\repl.md`: REPL 交互（如代码片段独立可保存为 .link 则执行）
- **Acceptance Criteria Addressed**: AC-1
- **Test Requirements**:
  - `programmatic` TR-3.1: 所有 ```link 块 parser+sema 通过
  - `programmatic` TR-3.2: 能独立运行的 A 类，执行 exit code=0

---

## [x] Task 4: basics/types.md, basics/operators.md — 基础类型与运算符
- **Priority**: high
- **Depends On**: None
- **Description**:
  - `basics/types.md`: 所有基础类型（i8~u64/f32/f64/bool/str/list）示例、类型标注、别名等
  - `basics/operators.md`: 算术、逻辑、比较、管道 `|` 运算符示例
- **Acceptance Criteria Addressed**: AC-1
- **Test Requirements**:
  - `programmatic` TR-4.1: 所有代码块 parser+sema 通过
  - `programmatic` TR-4.2: A 类示例解释器 / Python 后端执行 exit code=0

---

## [x] Task 5: basics/functions.md, basics/control-flow.md — 函数与控制流
- **Priority**: high
- **Depends On**: None
- **Description**:
  - `basics/functions.md`: fn 声明、多返回、lambda、高阶函数、递归、extern fn 等
  - `basics/control-flow.md`: if/else, match, while, loop, for, break/continue
- **Acceptance Criteria Addressed**: AC-1
- **Test Requirements**:
  - `programmatic` TR-5.1: 所有代码块 parser+sema 通过
  - `programmatic` TR-5.2: A 类示例执行 exit code=0 并匹配文档输出

---

## [x] Task 6: basics/composite-types.md, basics/collections.md — 复合类型与集合
- **Priority**: medium
- **Depends On**: None
- **Description**:
  - `basics/composite-types.md`: struct (字段, 初始化, 访问), enum (变体, payload, match)
  - `basics/collections.md`: list (索引/长度/追加/遍历), 嵌套 list
- **Acceptance Criteria Addressed**: AC-1
- **Test Requirements**:
  - `programmatic` TR-6.1: 所有代码块 parser+sema 通过
  - `programmatic` TR-6.2: A 类示例执行成功

---

## [x] Task 7: basics/flow.md, basics/async.md — flow 声明与 async
- **Priority**: medium
- **Depends On**: None
- **Description**:
  - `basics/flow.md`: flow 块、source/pipeline 属性；示例必须 parser+sema 通过（可能解释器不完全执行 flow 语义）
  - `basics/async.md`: async fn / await
- **Acceptance Criteria Addressed**: AC-1
- **Test Requirements**:
  - `programmatic` TR-7.1: 所有代码块 parser+sema 通过；A 类（如可）执行成功

---

## [ ] Task 8: compiler/cli.md, compiler/c-backend.md, compiler/llvm-backend.md
- **Priority**: medium
- **Depends On**: None
- **Description**:
  - `compiler/cli.md`: `link compile`, `--emit-c/--emit-ir/--backend` 等示例命令（其中 shell 命令忽略，内含的 .link 示例代码块检查）
  - `compiler/c-backend.md`: C 后端示例（用 C 后端编译，生成 C 不报错）
  - `compiler/llvm-backend.md`: LLVM 后端示例（至少 parser+sema 通过，codegen 若没启用 llvm feature 则仅保证不 panic）
- **Acceptance Criteria Addressed**: AC-1
- **Test Requirements**:
  - `programmatic` TR-8.1: 所有 ```link 块 parser+sema 通过
  - `programmatic` TR-8.2: C 后端示例（`--backend c --emit-c`）生成 .c 不报错
  - `programmatic` TR-8.3: Python/WASM backend 对应示例 codegen 不报错

---

## [x] Task 9: compiler/python-backend.md, compiler/wasm-backend.md, compiler/borrow-checker.md
- **Priority**: high
- **Depends On**: None
- **Description**:
  - `compiler/python-backend.md`: Python backend 示例，直接编译运行并验证输出
  - `compiler/wasm-backend.md`: WASM backend 示例，codegen 生成 WAT 文件不报错
  - `compiler/borrow-checker.md`: 借用检查示例，应有"能通过"和"应报错"两类；验证 pass/reject 行为符合文档描述
- **Acceptance Criteria Addressed**: AC-1
- **Test Requirements**:
  - `programmatic` TR-9.1: python-backend.md 的每个 A 类示例 `--backend py` 编译后 `python` 执行 exit 0 + 输出匹配
  - `programmatic` TR-9.2: wasm-backend.md 每个示例 `--backend wasm` 生成 .wat 不报错
  - `programmatic` TR-9.3: borrow-checker.md 中明确"应该报错"的片段，sema 阶段确实报告错误；"合法"的不报错

---

## [ ] Task 10: ffi/overview.md, ffi/bindgen.md, ffi/c.md, ffi/cpp.md
- **Priority**: medium
- **Depends On**: None
- **Description**:
  - `ffi/overview.md`: FFI 总览示例，extern "X" 多语言，声明片段（B/C 类）parser+sema + Python backend codegen
  - `ffi/bindgen.md`: `link bindgen` 相关代码块（其中 shell 命令跳过），.link 声明 parser+sema 通过
  - `ffi/c.md`: extern "C" 声明、struct/enum 通过 FFI、libloading 加载示例（parser+sema + codegen）
  - `ffi/cpp.md`: extern "C++" 声明（parser+sema + codegen）
- **Acceptance Criteria Addressed**: AC-1
- **Test Requirements**:
  - `programmatic` TR-10.1: 所有 ```link 代码块 parser+sema 通过
  - `programmatic` TR-10.2: C/Python 后端生成不报错（C backend 生成 .h/.c，Python backend 生成 .py）

---

## [ ] Task 11: ffi/python.md, ffi/java.md, ffi/html.md, ffi/wasm.md, ffi/process.md
- **Priority**: medium
- **Depends On**: None
- **Description**:
  - `ffi/python.md`: extern "python" 声明（parser+sema + Python backend codegen 通过）
  - `ffi/java.md`: extern "java" 声明（parser+sema）
  - `ffi/html.md`: extern "html" 声明（parser+sema）
  - `ffi/wasm.md`: extern "wasm" 声明 + 示例（parser+sema + WASM backend codegen 生成 .wat 不报错）
  - `ffi/process.md`: extern "go"/"ruby"/"rs"/"kt" 等进程桥接（parser+sema + codegen，不强制运行外部进程）
- **Acceptance Criteria Addressed**: AC-1
- **Test Requirements**:
  - `programmatic` TR-11.1: 所有 ```link 代码块 parser+sema 通过
  - `programmatic` TR-11.2: 对应 backend 的 codegen 生成不报错（有则测，无至少保证 parser+sema）

---

## [ ] Task 12: spec.md — 规格文档（核心语法与语义定义，示例最多最杂）
- **Priority**: high
- **Depends On**: Task 1-11 均通过（集中于语法规格，是最终回归校验）
- **Description**:
  - `d:\linkdoc\docs\spec.md` 中所有 ```link 代码块（类型、表达式、语句、FFI、flow、stream、match、struct/enum 所有规范示例）
  - 每一段示例按 A/B/C/D 分类处理
- **Acceptance Criteria Addressed**: AC-1
- **Test Requirements**:
  - `programmatic` TR-12.1: 所有 ```link 代码块 parser+sema 通过
  - `programmatic` TR-12.2: 所有可独立运行 A 类，执行 exit code=0

---

## [x] Task 13: plans/*.md — 计划历史文档（片段化代码）
- **Priority**: low
- **Depends On**: None
- **Description**:
  - `plans/2026-07-26-link-phase0-skeleton.md`, `plans/2026-07-27-link-phase1-1-c-ffi.md`, `plans/2026-07-27-link-phase1-3-stream.md`
  - 其中 ```link 代码块 parser+sema；```rust 代码块跳过（为文档当时设计稿，非当前实现）
- **Acceptance Criteria Addressed**: AC-1
- **Test Requirements**:
  - `programmatic` TR-13.1: 所有 ```link 代码块 parser+sema 通过（大多是设计阶段片段，允许为 D 类但必须记录）

---

## [ ] Task 14: 第二轮全量回归（重复 Task 1~13 的检查流程）
- **Priority**: high
- **Depends On**: Task 1~13 全部标记 [x]
- **Description**:
  - 不增量，按相同清单与分类完整再跑一遍
  - 任何一处失败，立即修复并回到相应 Task 重新过；修复后本轮需重新完整跑
  - 必须确保 `cargo test` 全 workspace 0 failed
- **Acceptance Criteria Addressed**: AC-3
- **Test Requirements**:
  - `programmatic` TR-14.1: 第一轮 checklist 里所有 A/B/C 在第二轮一致通过，零降级
  - `programmatic` TR-14.2: `cargo test` 0 failed

---

## [x] Task 15: 产物与日志汇总（bug_log, checklist, tasks 状态统一）
- **Priority**: medium
- **Depends On**: Task 14 [x]
- **Description**:
  - 最终检查：checklist.md 全部 [x]；bug_log.md 每条修复有前后对照 + cargo test 通过记录
- **Acceptance Criteria Addressed**: AC-4
- **Test Requirements**:
  - `human-judgement` TR-15.1: Spec 模式下三份文件闭环，所有示例编号可追溯，修复项可溯源
