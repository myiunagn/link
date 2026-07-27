# Link Language Support for VS Code

VS Code 扩展,为 Link 编程语言提供语法高亮和语言支持。

## 功能

- **语法高亮** - 关键字、类型、字符串、数字、注释、函数等
- **括号匹配** - `{}`, `[]`, `()` 自动闭合和匹配
- **注释支持** - `//` 行注释,`/* */` 块注释
- **错误诊断** - 实时检测语法和运行时错误(需要 Link CLI)
- **代码折叠** - `#region` / `#endregion` 标记

## 安装

### 从源码安装

1. 安装 Node.js (v16+)
2. 安装依赖并编译:

```bash
cd editors/vscode
npm install
npm run compile
```

3. 按 `F5` 启动扩展开发宿主(Extension Development Host),或使用 `code --install-extension` 打包安装。

### 开发模式

在 VS Code 中打开 `editors/vscode` 目录,按 `F5` 即可在新窗口中测试扩展。

## 配置

| 配置项 | 类型 | 说明 |
|--------|------|------|
| `link.binaryPath` | string | Link CLI 可执行文件路径(留空自动查找) |

在 `.vscode/settings.json` 中设置:

```json
{
  "link.binaryPath": "d:/link/target/debug/link.exe"
}
```

## 语法高亮说明

高亮覆盖以下元素:

- **关键字**: `fn`, `let`, `if`, `else`, `while`, `for`, `in`, `loop`, `break`, `continue`, `return`
- **类型**: `i32`, `i64`, `f64`, `bool`, `str`, `list`, `none`
- **字面量**: 整数、浮点数、字符串、`true`/`false`/`none`
- **内置函数**: `print`, `println`, `len`
- **运算符**: 算术、比较、逻辑、赋值、范围 `..`
- **注释**: 行注释 `//`、块注释 `/* */`

## 项目结构

```
editors/vscode/
├── package.json              # 扩展清单
├── tsconfig.json             # TypeScript 配置
├── language-configuration.json  # 语言配置(括号、注释等)
├── syntaxes/
│   └── link.tmLanguage.json  # TextMate 语法高亮
└── src/
    └── extension.ts          # 扩展主入口(错误诊断)
```

## 已知限制

- 错误诊断需要保存文件后才能检测(基于 CLI 调用)
- 行号定位目前依赖错误输出中的行号信息
- 不支持自动补全、悬停提示等高级 LSP 功能

## License

MIT
