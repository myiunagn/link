# Phase 1.1: C FFI 基础 实施计划

> **目标**:Link 能通过 `extern "C"` 调用 C 标准库函数,通过 `export "C"` 导出 Link 函数给 C 调用。

**架构**:在解释器中集成 `libloading`,运行时动态加载共享库,通过 C ABI 调用外部函数。

**Tech Stack**:Rust + libloading + C ABI

---

## 文件结构

| 文件 | 职责 |
|------|------|
| `crates/linkc_lexer/src/lib.rs` | 添加 `Extern`/`Export` 关键字 token |
| `crates/linkc_parser/src/lib.rs` | 添加 `ExternBlock`/`ExportBlock` AST + 解析 |
| `crates/linkc_interpreter/src/lib.rs` | 通过 libloading 调用 C 函数 |
| `crates/linkc_interpreter/Cargo.toml` | 添加 libloading 依赖 |
| `crates/linkc_cli/src/main.rs` | 注册 C 库路径配置 |
| `tests/fixtures/c_extern.link` | C FFI 测试示例 |

---

## Task 1: Lexer 添加 extern/export 关键字

**Files:**
- Modify: `crates/linkc_lexer/src/lib.rs`

- [ ] **Step 1**: 在 `Token` 枚举添加 `Extern`、`Export` 变体
- [ ] **Step 2**: 在 `lex_keyword` 添加 `extern` → `Token::Extern`、`export` → `Token::Export`
- [ ] **Step 3**: 添加测试 `test_lexer_extern_keyword`、`test_lexer_export_keyword`

---

## Task 2: Parser 添加 extern/export 块 AST 和解析

**Files:**
- Modify: `crates/linkc_parser/src/lib.rs`

- [ ] **Step 1**: 添加 `ExternDecl`/`ExportDecl` 到 `Stmt` 枚举:
  ```rust
  ExternDecl {
      language: String,      // "C", "python", "js"
      module: Option<String>, // 模块名(如 "numpy")
      decls: Vec<FnSignature>,
  },
  ExportDecl {
      language: String,
      module: Option<String>,
      decls: Vec<FnSignature>,
  },
  ```
  其中 `FnSignature`:
  ```rust
  struct FnSignature {
      name: String,
      params: Vec<(String, TypeAnnotation)>,
      return_type: Option<TypeAnnotation>,
      is_async: bool,
  }
  ```

- [ ] **Step 2**: 添加 `parse_extern_block` 方法,解析语法:
  ```
  extern "C" {
      fn abs(n: i32) -> i32;
      fn sqrt(x: f64) -> f64;
  }
  ```

- [ ] **Step 3**: 添加 `parse_export_block` 方法

- [ ] **Step 4**: 添加测试

---

## Task 3: Interpreter 集成 libloading 实现 C 调用

**Files:**
- Modify: `crates/linkc_interpreter/Cargo.toml`
- Modify: `crates/linkc_interpreter/src/lib.rs`

- [ ] **Step 1**: 添加 `libloading` 依赖
- [ ] **Step 2**: 在 `Environment` 添加 `loaded_libs: HashMap<String, Library>` 字段
- [ ] **Step 3**: 实现 `eval_extern_decl`:
  - 对 `extern "C"` 块,解析函数签名
  - 注册为 `NativeFunction`,调用时通过 libloading 查找符号
- [ ] **Step 4**: 实现类型转换:
  - Link i32 → C int32_t
  - Link f64 → C double
  - Link str → C const char*
  - Link bool → C bool

- [ ] **Step 5**: 添加测试:调用 C `abs(-42)` 返回 42

---

## Task 4: 测试和示例

**Files:**
- Create: `tests/fixtures/c_extern.link`

- [ ] **Step 1**: 创建测试文件调用 C 标准库:
  ```link
  extern "C" {
      fn abs(n: i32) -> i32;
      fn sqrt(x: f64) -> f64;
  }

  abs(-42)
  ```

- [ ] **Step 2**: 运行验证 `abs(-42)` = 42

- [ ] **Step 3**: 全量测试回归

---

## 验收标准

- [ ] `extern "C" { fn abs(n: i32) -> i32; }` 语法可解析
- [ ] 调用 `abs(-42)` 返回 42
- [ ] 调用 `sqrt(16.0)` 返回 4.0
- [ ] 所有原有测试仍然通过
- [ ] 零编译警告
