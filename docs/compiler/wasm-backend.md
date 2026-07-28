# WASM 代码生成后端

Link 编译器支持将 Link 源代码编译为 WebAssembly Text 格式（WAT），然后可以通过工具链转换为 WASM 二进制文件在浏览器或 WASM 运行时中执行。

## 特性

| 类别 | 支持内容 |
|------|---------|
| **基本类型** | i32/i64/f32/f64/bool |
| **字面量** | 整数、浮点数、布尔值 |
| **运算** | 算术(+ - * / %)、比较(== != < > <= >=)、逻辑(&& \|\| !) |
| **变量** | let 声明、赋值 |
| **函数** | 声明、递归、参数、返回值 |
| **控制流** | if/else、while、for、loop、break、continue |
| **导入导出** | import/export 函数声明 |
| **内存** | 线性内存操作 |

## 使用方法

### 生成 WAT 代码

```bash
link compile myfile.link --backend wasm -o output.wat
```

### 转换为 WASM 二进制

使用 WABT 工具链中的 `wat2wasm` 将 WAT 转换为 WASM：

```bash
wat2wasm output.wat -o output.wasm
```

### 在浏览器中运行

```html
<script>
WebAssembly.instantiateStreaming(fetch('output.wasm'))
    .then(result => {
        const exports = result.instance.exports;
        console.log(exports.add(2, 3));
    });
</script>
```

### 在 Node.js 中运行

```javascript
const fs = require('fs');
const wasmBuffer = fs.readFileSync('output.wasm');

WebAssembly.instantiate(wasmBuffer)
    .then(result => {
        const exports = result.instance.exports;
        console.log(exports.add(2, 3));
    });
```

## WAT 格式说明

WAT（WebAssembly Text Format）是 WebAssembly 的文本表示形式，使用 S-表达式语法。

### 基本结构

```wat
(module
  ;; 函数定义
  (func $add (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )
  ;; 导出函数
  (export "add" (func $add))
)
```

### 数据类型

| WAT 类型 | 对应 Link 类型 | 说明 |
|----------|---------------|------|
| `i32` | `i32` / `u32` | 32 位整数 |
| `i64` | `i64` / `u64` | 64 位整数 |
| `f32` | `f32` | 32 位浮点数 |
| `f64` | `f64` | 64 位浮点数 |

## 导入导出

### 导出函数

使用 `export` 关键字导出函数，使其可以在宿主环境中调用：

```link
export fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

export fn multiply(a: i32, b: i32) -> i32 {
    return a * b;
}
```

生成的 WAT 代码：

```wat
(module
  (func $add (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )
  (func $multiply (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.mul
  )
  (export "add" (func $add))
  (export "multiply" (func $multiply))
)
```

### 导入函数

使用 `extern "wasm"` 声明导入函数，从宿主环境导入：

```link
extern "wasm" module "env" {
    fn log(x: i32) -> void;
    fn sin(x: f64) -> f64;
}

fn compute(x: f64) -> f64 {
    return sin(x) * 2.0;
}
```

生成的 WAT 代码：

```wat
(module
  (import "env" "log" (func $log (param i32)))
  (import "env" "sin" (func $sin (param f64) (result f64)))
  (func $compute (param $x f64) (result f64)
    local.get $x
    call $sin
    f64.const 2.0
    f64.mul
  )
)
```

## 生成的 WAT 代码示例

### Link 源代码

```link
export fn factorial(n: i64) -> i64 {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

export fn fib(n: i32) -> i32 {
    if n < 2 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}
```

### 生成的 WAT 代码

```wat
(module
  (func $factorial (param $n i64) (result i64)
    (local $temp i64)
    local.get $n
    i64.const 1
    i64.le_s
    if (result i64)
      i64.const 1
    else
      local.get $n
      local.get $n
      i64.const 1
      i64.sub
      call $factorial
      i64.mul
    end
  )
  (func $fib (param $n i32) (result i32)
    (local $temp i32)
    local.get $n
    i32.const 2
    i32.lt_s
    if (result i32)
      local.get $n
    else
      local.get $n
      i32.const 1
      i32.sub
      call $fib
      local.get $n
      i32.const 2
      i32.sub
      call $fib
      i32.add
    end
  )
  (export "factorial" (func $factorial))
  (export "fib" (func $fib))
)
```

## 内存操作

WASM 使用线性内存模型，可以通过导入内存或定义内存来使用：

```link
extern "wasm" module "env" {
    memory: [0];
    fn memset(ptr: i32, value: i32, size: i32) -> void;
}
```

## 适用场景

WASM 后端适合以下场景：

- **Web 前端**：在浏览器中运行高性能计算
- **Serverless**：WASM 作为轻量级 Serverless 运行时
- **嵌入式**：资源受限环境的安全执行
- **插件系统**：为应用提供安全的插件沙箱
- **跨平台**：一次编译，到处运行

## 工具链

常用的 WASM 工具链：

- **WABT**：WebAssembly Binary Toolkit，包含 wat2wasm、wasm2wat 等工具
- **wasmtime**：字节码联盟的 WASM 运行时
- **wasmer**：另一个流行的 WASM 运行时
- **Binaryen**：WASM 优化和代码生成工具

### 安装 WABT

```bash
# Windows (使用 winget)
winget install WebAssembly.wabt

# macOS
brew install wabt

# Linux (Ubuntu)
sudo apt-get install wabt
```

## 限制

当前 WASM 后端暂不支持：
- struct/enum 复合类型（需要手动内存布局）
- 字符串（需要通过线性内存操作）
- 列表/集合类型
- `stream<T>` 数据流
- async/await 异步编程
- 借用检查器
- Garbage Collection（依赖未来 WASM GC 提案）
