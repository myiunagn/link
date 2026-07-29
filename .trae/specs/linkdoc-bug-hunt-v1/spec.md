# Link 语言 v0.2 文档示例全量 Bug 狩猎 — Product Requirement Document

## Overview
- **Summary**: 系统性遍历 linkdoc 文档站中所有 Markdown 文档内的 ` ```link ` 代码块，对每一个可独立运行或可编译的 Link 源码示例执行"解析 → 语义检查 → （解释器运行 / 编译到 Python+C+WASM）"流程，捕获所有语法错误、类型错误、运行时错误和后端代码生成 Bug；发现 bug 后立即修复并回归测试，完成一轮后再完整跑第二轮，直到两轮均无 bug 为止。
- **Purpose**: 保证官方文档中的每一个示例都真实可运行，避免用户复制粘贴示例后因语言实现缺陷或文档与实现不一致而无法执行的体验问题，同时暴露编译器各层级隐藏 bug。
- **Target Users**: Link 语言的文档阅读者、初学者、v0.2 发行版验证者

## Goals
- 第一轮：扫描 `d:\linkdoc\docs` 下所有 .md 文档，提取全部 ```link 代码块（排除纯粹展示 Rust 项目结构的 Rust 代码块以及明显是片段的伪代码），为每一份源码做：(a) `cargo run -- parse` / parser + sema 不报错；(b) 不依赖外部库的示例通过解释器执行 (`link run`) 或 Python 编译执行 (`link compile --backend py` 并 `python`) 得到文档注释/代码片段中期望的输出；(c) 触发到 parser/sema/codegen 任一层报错的，判定为 bug，定位根因并修复；修复后回归本示例并对 cargo test 全量测试保持 0 regress。
- 第二轮：第一轮全部通过后，完整再跑一次相同清单，确保修复引入的回归被清除。
- 最终要求：两轮均 0 新增失败示例，`cargo test` 工作空间全部用例仍然通过。

## Non-Goals (Out of Scope)
- **不修复文档文案层面的错字**（除非文案描述和实现语义冲突并导致示例不可运行）
- **不引入 v0.3 的新语法/新特性**，只做 bug 修复 + 与文档一致的最小修补
- **不要求必须真正运行 FFI 外部依赖**（C DLL / Python 模块 / C++ 共享库 / Java / WASM / HTML / Go 桥接）：FFI 类示例只需要 parser+sema 通过 + codegen（对应后端）能生成目标代码，不强制外部第三方库可加载；除非是内置可实现的（如 `extern "C"` 标准库 `abs/sqrt/pow` 在解释器层面已经有 stub）
- **不重写 MkDocs 部署站点文件**，只读不修改 linkdoc\docs\ 下的 md 源码（除非确实文档写错了，此时以最小改动修正）

## Background & Context
- Link 代码结构：`d:\link\` workspace，crate 结构：linkc_lexer / linkc_parser / linkc_sema / linkc_interpreter / linkc_codegen (C + Python + WASM) / linkc_cli / linkc_lsp / linkc_llvm
- 最近一次修复：解析器内 `Expr::Lambda` 分支完整化；PythonBackend 增加 pipe 运算符支持与 stream/map/filter/collect 预置函数；`Error: Unexpected token: fn` 已清零
- 解释器命令：`d:\link\target\release\link.exe run <file.link>`；Python 编译：`link compile <file.link> --backend py -o <file.py>` 然后 `python <file.py>`
- 文档位置：`d:\linkdoc\docs\`，内含 basics / compiler / ffi / plans 等子目录，共 37 份 md

## Functional Requirements
- **FR-1**: 自动扫描并穷举 `d:\linkdoc\docs` 下所有 md 文件中的 ```link fenced code block，生成"文件路径:起始行号"级别的唯一索引
- **FR-2**: 对每一份代码块，按可执行性分类：
  - A. 纯 Link 独立示例：能单文件直接 `link run` 或 `link compile --backend py` + `python`
  - B. 声明片段：只有 `extern {}` / `struct` / `enum` 声明或片段（无法独立执行但必须 parser+sema 通过）
  - C. 外部依赖示例：引用 FFI（DLL/module/wasm/java/html/go/process），仅要求 parser + sema + 不依赖外部的 codegen 层通过
  - D. 伪代码/不可执行片段：含明显省略号 `...` 或明确示意代码，人工标记跳过
- **FR-3**: 对 A 类示例执行"编译/运行—对比期望输出"，期望输出从代码注释（如 `// 5`、`// 输出: ...`）或代码末尾 println 推断
- **FR-4**: 一旦任一示例报错（解析/sema/运行时/codegen/断言差异），立即最小化根因定位并修复；修复后 (a) 回归本示例 (b) 运行 `cargo test` 确保不引入回归
- **FR-5**: 第一轮完成后，完整执行第二轮（同样的 A/B/C 分类清单重复执行一次）
- **FR-6**: 失败示例要按统一日志格式写入 `bug_log.md`（编号、文档位置、代码、错误信息、修复 diff、回归结果）并可追溯

## Non-Functional Requirements
- **NFR-1 (正确性)**: 修复不得破坏已有 `cargo test` 全量用例
- **NFR-2 (完整度)**: 所有 A/B/C 类示例必须至少被执行/检查一次，第二轮与第一轮清单完全一致，禁止跳号、禁止因"看似难"而不处理
- **NFR-3 (可追溯)**: 每一个被处理的示例必须在 checklist 中由 `[ ] → [x]` 明确标记，含实际输出 vs 期望输出；每个被修复的 bug 必须在 bug_log 有编号条目

## Constraints
- **Technical**: 仅修改 `d:\link\crates\**` 下的 Rust 源码；不修改 linkdoc 仓库文档内容（除非发现文档确实写错且与实现矛盾时，修改仅作为最后手段且最小化）
- **Business**: 不限总耗时，但是必须完整执行两轮
- **Dependencies**: 使用已编译好的 `d:\link\target\release\link.exe`；python 运行时要求已在 PATH（3.10+）；不要求在本机安装 LLVM/Java/GCC 来跑 FFI

## Assumptions
- 本地已具备 Rust stable toolchain（`cargo test` 能通过）
- Python 已安装并在 PATH，可执行 `python` 
- linkdoc 仓库在 `d:\linkdoc\` 且 docs 目录下的 md 文件与线上文档一致
- A 类示例没有故意随机输出（如时间戳），所有输出可与注释中的期望做确定性或范围性比对

## Acceptance Criteria

### AC-1: 第一轮全量示例可枚举、分类并通过检查
- **Given**: `d:\linkdoc\docs` 所有 md 内容可读，release build 生成完毕
- **When**: 执行第一轮枚举与 A/B/C 分类，并按分类执行对应检查
- **Then**: A 类全部 `link run` 或 `link compile --backend py + python` 无错误且输出与期望匹配；B 类 parser + sema 无错误；C 类 parser + sema + 不依赖外部的 codegen 层（Python/C/WASM 目标）生成无报错；D 类明确标注 skip 且有理由
- **Verification**: `programmatic`（自动化脚本 + checklist 勾选）
- **Notes**: A 类示例优先用解释器 `link run`；如解释器缺 stream 等语义，降级为 Python backend 编译运行

### AC-2: 每个被命中的 bug 都被修复且零回归
- **Given**: 任一示例在 AC-1 流程中报错
- **When**: 完成修复并 `cargo build --release` 重新生成可执行文件
- **Then**: 失败示例在同条件下重新执行通过；`cargo test` 全工作区 0 失败
- **Verification**: `programmatic`（脚本 + test harness）

### AC-3: 第二轮完整执行零新增失败
- **Given**: 第一轮已全部通过且 bug_log 中所有条目均标"回归通过"
- **When**: 以相同的清单和分类完整再跑第二轮
- **Then**: 第一轮通过的所有示例在第二轮中保持通过，无新增失败
- **Verification**: `programmatic`

### AC-4: 产物与日志完整
- **Given**: 两轮都完成
- **When**: 审查 `d:\link\.trae\specs\linkdoc-bug-hunt-v1\` 下的文件
- **Then**: checklist.md 每一个条目都标 `[x]` 并有实际输出/通过说明；bug_log.md（如有）列出修复项、根因与验证；tasks.md 显示所有任务项已闭环
- **Verification**: `human-judgment`

## Open Questions
- [ ] 无。所有不确定项在执行中按"严格、不放过"原则处理：宁可人工标记并说明原因，也不跳过
