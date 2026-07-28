# HTML / JavaScript 互连

!!! abstract "概述"
    Link 通过 HTTP POST 请求调用 Node.js 服务器上的 JavaScript 函数。
    适用于 Web 服务集成、前端数据处理、Node.js 生态调用等场景。

## 基本用法

### module 字段

HTML/JS FFI 的 `module` 字段指定 HTTP 端点地址:

```link
extern "html" module "http://127.0.0.1:3000" {
    fn add(a: i32, b: i32) -> i32;
    fn render_html(content: str) -> str;
    fn fetch_data(url: str) -> str;
}
```

如果不指定 `module`,默认使用 `http://127.0.0.1:3000`。

### 调用示例

```link
extern "html" module "http://127.0.0.1:3000" {
    fn add(a: i32, b: i32) -> i32;
    fn format_date(timestamp: i64) -> str;
    fn render_html(content: str) -> str;
}

let sum = add(10, 20);                    // 30
let date = format_date(1690000000);       // "2023-07-22"
let html = render_html("<h1>Hello</h1>"); // "<html><body><h1>Hello</h1></body></html>"
```

## 类型映射

| Link 类型 | JavaScript 类型 | JSON 传输类型 |
|-----------|----------------|---------------|
| `i32` | `number` | JSON number |
| `i64` | `number` | JSON number |
| `f32` | `number` | JSON number |
| `f64` | `number` | JSON number |
| `bool` | `boolean` | JSON boolean |
| `str` | `string` | JSON string |

## 创建 Node.js 服务器

Link 通过 HTTP POST 请求调用服务器,请求体为 JSON:

### 请求格式

```json
{
    "module": "...",
    "function": "add",
    "args": [10, 20]
}
```

### 响应格式

```json
{
    "result": 30
}
```

### Node.js 服务器示例

```javascript
// server.js
const http = require('http');

const functions = {
    add: (a, b) => a + b,
    format_date: (timestamp) => {
        return new Date(timestamp * 1000).toISOString().split('T')[0];
    },
    render_html: (content) => {
        return `<html><body>${content}</body></html>`;
    }
};

const server = http.createServer((req, res) => {
    if (req.method !== 'POST') {
        res.writeHead(405);
        res.end('Method Not Allowed');
        return;
    }

    let body = '';
    req.on('data', chunk => { body += chunk; });
    req.on('end', () => {
        try {
            const { function: fnName, args } = JSON.parse(body);
            const fn = functions[fnName];
            if (!fn) {
                res.writeHead(404);
                res.end(JSON.stringify({ error: `Unknown function: ${fnName}` }));
                return;
            }
            const result = fn(...args);
            res.writeHead(200, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify({ result }));
        } catch (e) {
            res.writeHead(500);
            res.end(JSON.stringify({ error: e.message }));
        }
    });
});

server.listen(3000, () => {
    console.log('Link HTML/JS bridge server running on http://127.0.0.1:3000');
});
```

### 启动服务器

```bash
node server.js
```

## 完整示例

```link
extern "html" module "http://127.0.0.1:3000" {
    fn add(a: i32, b: i32) -> i32;
    fn format_date(timestamp: i64) -> str;
    fn render_html(content: str) -> str;
}

// 调用 JS 函数
let sum = add(100, 200);
println("sum =", sum);

// 格式化日期
let date = format_date(1690000000);
println("date =", date);

// 渲染 HTML
let html = render_html("<h1>Hello from Link!</h1>");
println("html =", html);
```

## 性能特点

- **HTTP 开销**:每次调用都有 HTTP 请求/响应开销,适合低频调用
- **网络延迟**:如果是远程服务器,会有网络延迟
- **超时设置**:默认超时 5000ms,可在运行时配置

!!! tip "适用场景"
    HTML/JS FFI 适合:
    - 调用 Node.js 生态库(npm 包)
    - Web 服务集成
    - 前端数据处理逻辑复用
    - 需要浏览器环境的代码执行

## 前置条件

- 本机或远程已运行 Node.js 服务器
- 服务器监听 HTTP POST 请求并返回 JSON 响应
- 网络连通(防火墙未阻止指定端口)

## 常见问题

### 连接超时

确保服务器正在运行,端口号正确,防火墙未阻止连接。

### 函数未找到

服务器端必须注册所有可调用的函数,函数名与 `extern` 声明一致。

### 响应格式错误

服务器必须返回 `{"result": <value>}` 格式的 JSON,否则 Link 无法解析结果。

## 下一步

- [多语言互联概述](overview.md)
- [进程桥接互连](process.md)
