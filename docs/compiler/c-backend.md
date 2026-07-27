# C 代码生成后端

Link 编译器支持将 Link 源代码编译为 C 代码，然后通过系统 C 编译器生成原生可执行文件。

## 特性

| 类别 | 支持内容 |
|------|---------|
| **基本类型** | i8/i16/i32/i64/u8/u16/u32/u64/usize/f32/f64/bool/str/void |
| **字面量** | 整数、浮点数、字符串、布尔值、none、列表 |
| **运算** | 算术(+ - * / %)、比较(== != < > <= >=)、逻辑(&& \|\| !)、一元(- !) |
| **变量** | let 声明（含类型注解）、赋值 |
| **函数** | 声明、递归、参数、返回值 |
| **控制流** | if/else、while、for、loop、break、continue |
| **复合类型** | struct 定义/初始化/字段访问、enum（带/不带 payload） |
| **模式匹配** | match 语句（通配符、字面量、绑定、枚举变体） |
| **列表** | 字面量、索引访问 |
| **FFI** | extern "C" 声明 |
| **优化** | O0/O1/O2/O3 等级 |
| **调试** | `-g` 生成 `#line` 指令 |

## 使用方法

### 生成 C 代码

```bash
link compile myfile.link --emit-c
```

### 编译为可执行文件

```bash
link compile myfile.link
```

### 优化等级

```bash
link compile myfile.link --opt-level 3
```

### 调试信息

```bash
link compile myfile.link -g
```

## 生成的 C 代码示例

### Link 源代码

```link
fn fib(n: i64) -> i64 {
    if n < 2 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

struct Point { x: i32, y: i32 }

let p = Point { x: 1, y: 2 };
p.x + p.y
```

### 生成的 C 代码

```c
#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    int32_t x;
    int32_t y;
} Point;

int64_t fib(int64_t n);

int64_t fib(int64_t n) {
    if ((n < 2LL)) {
        return n;
    }
    return (fib((n - 1LL)) + fib((n - 2LL)));
}

int main(void) {
    struct Point p = { .x = 1, .y = 2 };
    printf("%lld\n", (long long)((p.x + p.y)));
    return 0;
}
```

## 支持的平台

- **Windows**: 使用 MSVC (`cl`) 编译器
- **Linux/macOS**: 使用 GCC/Clang (`cc`)

## 限制

当前 C 后端暂不支持：
- `stream<T>` 类型（数据流）
- `flow` 声明块
- `async/await` 异步编程
- 管道运算符 `|`
