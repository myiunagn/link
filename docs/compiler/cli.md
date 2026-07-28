# 编译器 CLI 使用

`link compile` 命令用于将 Link 源代码编译为原生可执行文件。

## 命令语法

```bash
link compile <input.link> [options]
```

## 选项

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `-o <path>` | 输出路径 | 输入文件名（不含扩展名） |
| `--backend <type>` | 代码生成后端: `c` / `llvm` / `python` / `py` / `wasm` | `c` |
| `--emit-c` | 生成 C 代码（C 后端） | 否 |
| `--emit-ir` | 生成 LLVM IR（LLVM 后端） | 否 |
| `--opt-level <N>` | 优化等级（0-3） | 2 |
| `-g` | 包含调试信息 | 否 |
| `--no-link` | 不链接（仅生成目标文件） | 否 |
| `--no-borrow-check` | 禁用借用检查器 | 否 |

## 使用示例

### 基本编译

```bash
link compile myfile.link
```

### 指定输出路径

```bash
link compile myfile.link -o myprogram
```

### 生成 C 代码

```bash
link compile myfile.link --emit-c -o output.c
```

### 生成 LLVM IR

```bash
link compile myfile.link --backend llvm --emit-ir -o output.ll
```

### 优化等级

```bash
link compile myfile.link --opt-level 3
```

### 调试信息

```bash
link compile myfile.link -g
```

### 不链接（仅生成目标文件）

```bash
link compile myfile.link --no-link
```

### 生成 Python 代码

```bash
link compile myfile.link --backend python -o output.py
```

### 生成 WASM WAT 代码

```bash
link compile myfile.link --backend wasm -o output.wat
```

### 禁用借用检查器

```bash
link compile myfile.link --no-borrow-check
```

## 后端选择

### C 后端（默认）

```bash
link compile myfile.link --backend c
```

特点：
- 跨平台支持好
- 依赖系统 C 编译器
- 支持 struct/enum/list/match 等高级特性

### LLVM 后端

```bash
link compile myfile.link --backend llvm
```

特点：
- 需要安装 LLVM
- 通过 `--features llvm-backend` 启用
- 支持更多优化通道
- 当前功能较少（基础类型、函数、控制流）

### Python 后端

```bash
link compile myfile.link --backend python
# 或简写
link compile myfile.link --backend py
```

特点：
- 生成可读的 Python 代码
- 无需额外依赖，直接用 Python 解释器运行
- 适合快速原型开发和脚本场景
- 支持 struct/enum/match/stream/async 等高级特性

### WASM 后端

```bash
link compile myfile.link --backend wasm
```

特点：
- 生成 WAT（WebAssembly Text Format）格式代码
- 可通过 `wat2wasm` 等工具编译为 `.wasm` 二进制
- 支持导入导出函数
- 适合 Web 端和嵌入式场景

## 优化等级

| 等级 | 说明 |
|------|------|
| O0 | 无优化，编译最快，适合调试 |
| O1 | 基础优化，平衡编译时间和运行性能 |
| O2 | 默认优化，全面优化 |
| O3 | 最高优化，可能增加编译时间 |

## 调试信息

使用 `-g` 选项可以生成调试信息：

- **C 后端**: 添加 `#line` 指令
- **LLVM 后端**: 添加调试元数据

## 环境要求

### C 后端

- **Windows**: 需要 MSVC (`cl`) 在 PATH 中
- **Linux/macOS**: 需要 GCC/Clang (`cc`) 在 PATH 中

### LLVM 后端

- 需要系统安装 LLVM（通过 `--features llvm-backend` 启用）

## 借用检查器

Link v0.2.0 内置了借用检查器，在编译时自动执行，用于保证内存安全：

- **所有权跟踪**：每个值有且仅有一个所有者
- **移动语义**：赋值和传参默认移动所有权
- **Copy 类型**：基础类型默认 Copy，不移动所有权
- **借用规则**：不可变借用可多个，可变借用只能一个
- **use-after-move 检测**：编译时报告使用已移动值的错误

可通过 `--no-borrow-check` 选项禁用借用检查器（不推荐）。

详细说明请参考 [借用检查器文档](borrow-checker.md)。

## 编译优化

Link 编译器支持多种编译优化，在 `--opt-level` 大于等于 1 时启用：

### 常量折叠

编译期对常量表达式进行求值，减少运行时计算：

```link
// 编译期直接计算为 42
let x = 10 + 32;
```

### 死代码消除

移除永远不会执行的代码分支：

```link
if false {
    println("这段代码会被消除");
}
```

## LSP 语言服务器

`link lsp` 命令启动 LSP 语言服务器，为编辑器提供智能支持：

```bash
link lsp
```

支持的功能：

- **诊断**：实时语法和类型错误提示
- **补全**：代码自动补全
- **悬停**：悬停显示类型和文档
- **跳转**：跳转到定义位置
- **符号大纲**：文档符号列表

### VS Code 配置

在 VS Code 中安装 Link 语言扩展后，会自动启动 `link lsp`。

## 游戏后端

`link game` 命令用于运行游戏后端域类型的程序：

```bash
link game mygame.link
```

功能特性：

- **domain 语法**：声明式定义游戏域
- **WebSocket**：内置 WebSocket 服务器
- **房间系统**：玩家匹配与房间管理
- **帧同步**：确定性帧同步机制

## 命令行帮助

```bash
link compile --help
link lsp --help
link game --help
```
