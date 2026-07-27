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
| `--backend <type>` | 代码生成后端 | `c` |
| `--emit-c` | 生成 C 代码（C 后端） | 否 |
| `--emit-ir` | 生成 LLVM IR（LLVM 后端） | 否 |
| `--opt-level <N>` | 优化等级（0-3） | 2 |
| `-g` | 包含调试信息 | 否 |
| `--no-link` | 不链接（仅生成目标文件） | 否 |

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

## 命令行帮助

```bash
link compile --help
```
