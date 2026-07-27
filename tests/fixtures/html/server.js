// Link HTML/JS FFI 服务器
// 用法: node server.js
// Link 通过 extern "html" module "http://127.0.0.1:3000" 注册函数
// 协议: HTTP POST /<func_name> 请求体 {"args":[...]} -> {"result":<value>}

const http = require('http');

// 注册的 JS 函数
const functions = {
  add: (a, b) => a + b,
  multiply: (a, b) => a * b,
  greet: (name) => `Hello from JavaScript, ${name}!`,
  isEven: (n) => n % 2 === 0,
  square: (x) => x * x,
  reverse: (s) => s.split('').reverse().join(''),
  uppercase: (s) => s.toUpperCase(),
};

const server = http.createServer((req, res) => {
  // 简单的 CORS
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');

  if (req.method === 'OPTIONS') {
    res.writeHead(200);
    res.end();
    return;
  }

  // URL 形如 /add /multiply
  const funcName = req.url.replace(/^\//, '').split('?')[0];

  if (req.method !== 'POST') {
    res.writeHead(405, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: 'Method not allowed, use POST' }));
    return;
  }

  if (!functions[funcName]) {
    res.writeHead(404, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: `Function '${funcName}' not found` }));
    return;
  }

  let body = '';
  req.on('data', chunk => body += chunk);
  req.on('end', () => {
    try {
      const data = JSON.parse(body || '{}');
      const args = data.args || [];
      const result = functions[funcName](...args);
      res.writeHead(200, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ result }));
    } catch (e) {
      res.writeHead(400, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ error: e.message }));
    }
  });
});

const port = process.env.PORT || 3000;
server.listen(port, () => {
  console.log(`Link HTML FFI server running on http://127.0.0.1:${port}`);
  console.log(`Registered functions: ${Object.keys(functions).join(', ')}`);
});
