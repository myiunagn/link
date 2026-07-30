# Link Examples

```
examples/
├── basic/          # 基础语言特性
│   ├── hello.link          最简单的程序
│   ├── fib.link            斐波那契（递归）
│   ├── struct_enum_demo.link   struct + enum + match
│   ├── flow_demo.link      声明式数据流
│   ├── module_demo.link    模块导入
│   └── test_sema.link      语义分析测试
│
├── ffi/            # 多语言互操作（12 种语言）
│   ├── all_languages_ffi.link   全部语言 FFI 示例
│   ├── multilang_demo.link      多语言协同
│   ├── cpp_demo/            C++ 桥接源码和 DLL
│   └── link_sdk/            C / Python / TypeScript 绑定 SDK
│
├── compiler/       # 编译器功能
│   ├── compile_demo.link      编译流程
│   ├── full_compile_demo.link 完整编译 + 优化
│   └── compiler_demo.link     编译器 API
│
├── backend/        # 多后端输出
│   ├── python_backend_demo.link   → Python 代码
│   ├── python_backend_demo.py     生成的 Python
│   └── wasm_backend_demo.link     → WebAssembly
│
├── async/          # 异步编程
│   └── async_demo.link       async / await
│
├── game/           # 游戏后端
│   ├── game_server.link       WebSocket 房间 + 帧同步
│   └── game_client.html       浏览器客户端
│
└── stdlib/         # 标准库
    ├── stdlib_demo.link       内置函数 + 类型
    └── math_utils.link        数学工具
```

## 运行

```bash
# 解释执行
link run examples/basic/hello.link

# 编译执行
link compile examples/basic/fib.link -o fib
./fib

# 自举编译
./bootstrap/build/link-bootstrap examples/basic/hello.link hello.c
cc -std=c99 hello.c -o hello && ./hello
```
