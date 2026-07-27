# LLVM IR 后端

Link 编译器支持将 Link 源代码编译为 LLVM IR，然后通过 LLVM 工具链生成原生可执行文件。

## 特性

| 类别 | 支持内容 |
|------|---------|
| **基本类型** | i64、f64、bool、指针 |
| **运算** | 算术(+ - * / %)、比较、逻辑、一元 |
| **函数** | 声明、递归、参数、返回值 |
| **控制流** | if/else、while、for、loop |
| **变量** | let 声明、赋值 |
| **优化** | LLVM PassManager 优化通道 |

## 使用方法

### 启用 LLVM 后端

LLVM 后端使用条件编译，需要通过 feature 启用：

```bash
cargo build --features llvm-backend --release
```

### 生成 LLVM IR

```bash
link compile myfile.link --backend llvm --emit-ir
```

### 编译为可执行文件

```bash
link compile myfile.link --backend llvm
```

## 生成的 LLVM IR 示例

### Link 源代码

```link
fn add(a: i64, b: i64) -> i64 {
    return a + b;
}

add(2, 3)
```

### 生成的 LLVM IR

```llvm
; ModuleID = 'link_module'
source_filename = "link_module"

define i64 @add(i64 %0, i64 %1) {
entry:
  %a = alloca i64, align 8
  %b = alloca i64, align 8
  store i64 %0, ptr %a, align 8
  store i64 %1, ptr %b, align 8
  %2 = load i64, ptr %a, align 8
  %3 = load i64, ptr %b, align 8
  %4 = add nsw i64 %2, %3
  ret i64 %4
}

define i64 @main() {
entry:
  %0 = call i64 @add(i64 2, i64 3)
  ret i64 0
}
```

## 优化

LLVM 后端使用 LLVM PassManager 进行优化，默认使用 Aggressive 优化等级。

## 依赖

LLVM 后端需要系统安装 LLVM：

### Windows

```powershell
winget install LLVM.LLVM
```

### Linux (Ubuntu)

```bash
sudo apt-get install llvm
```

### macOS

```bash
brew install llvm
```

## 限制

当前 LLVM 后端暂不支持：
- struct/enum 复合类型
- 列表
- `stream<T>` 类型
- `flow` 声明块
- `async/await` 异步编程
- match 模式匹配
