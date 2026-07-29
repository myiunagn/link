# Link Doc Bug Hunt v1 - Bug Log

Date: 2026-07-29, R3 regression

---

## BUG-20260729-126

- **Title**: Python backend `Stmt::Match` codegen generates multiple `else:` blocks for consecutive unconditional patterns → Python SyntaxError (`else:` without matching `if`)
- **Severity**: High (breaks Python backend compilation for any match with 2+ Bind/Wildcard arms)
- **Trigger block**: CT011 (basics/composite-types.md:178) — match syntax template:
  ```link
  match scrutinee {
      pattern => { body }
      pattern => { body }
      _ => { default }
  }
  ```
- **Root cause**: `crates/linkc_codegen/src/lib.rs` `Stmt::Match` handler. Every arm with `cond_str is None` (Wildcard or Bind patterns) emits a new else-like block:
  - 1st unconditional arm → `if True:` (set first=false, continue)
  - 2nd unconditional arm → `else:` (still allowed by code, duplicated)
  - 3rd unconditional arm → another `else:` (Python SyntaxError: duplicate else)
- **Expected**: After emitting the first catch-all arm (`if True:` / `else:`), all subsequent arms are dead code and must not generate code (break the loop).
- **Fix**: Add `emitted_else` bool flag in `Stmt::Match`. Set `true` whenever we emit a catch-all arm. If `emitted_else` at top of loop → `break`.
- **Fix files**:
  - `d:\link\crates\linkc_codegen\src\lib.rs:1623-1707` (lines 1629-1701)
- **Verification**:
  - Reproduce: `link compile --backend py -o CT011.py CT011.link` → `CT011.py` has `if True: body` followed by 2× `else:` blocks; `py_compile.compile()` raises `SyntaxError: invalid syntax` at `else:`.
  - Post-fix: Same command produces valid Python; `py_compile.compile()` succeeds; only first arm is kept (correct, later arms are unreachable).
  - `cargo test`: 54 passed; 0 failed.
  - `cargo build --release`: success.
- **Discovered in**: R3 regression run, block CT011

---

## BUG-20260729-127

- **Title**: Sema over-strictly requires bool for `if`/`while` conditions — conflicts with documented truthy semantics (str/list/none/number truth values)
- **Severity**: Medium (breaks Python backend for documented truthy if examples; interpreter/link run already supported truthy correctly)
- **Trigger block**: O011 (basics/operators.md:164) — truth table demo:
  ```link
  if "" { println("不会执行"); }
  if [] { println("不会执行"); }
  if none { println("不会执行"); }
  if 0 { println("会执行"); }
  if "hello" { println("会执行"); }
  ```
- **Observed**:
  - `link run O011.link` — exit=0, correctly executes truthy semantics ("会执行", "会执行")
  - `link compile --backend py O011.link` — sema emits: `if condition must be bool, got str/unit/i64 ... Type checking failed with 5 error(s)`, rc=1
- **Root cause**: 3 locations in `crates/linkc_sema/src/lib.rs` hard-enforced bool-only conditions without matching interpreter behavior:
  - `Stmt::If` (line ~400, function `infer_block_return_type`)
  - `Stmt::While` (line ~414)
  - `Expr::IfExpr` (line ~633, function `infer_expr`)
  - And 3 analogous locations in type-annotation variant checker (lines ~861/883)
- **Docs contract**: basics/operators.md lines 155-162 explicitly document truthy table (none/""/[]=false, 0/non-empty str=true). Interpreter already honors this; only sema/py-backend blocked.
- **Fix**: Remove all 5 `cond_type.is_bool()` checks; keep inferring condition type but skip bool-only error. Update test:
  - `test_check_bad_if_condition` (old assertion `must be bool`) → replaced by `test_check_truthy_if_condition` + `test_check_truthy_nonbool_conditions` (both assert empty errors)
- **Fix files**:
  - `d:\link\crates\linkc_sema\src\lib.rs:400-422` (Stmt::If / Stmt::While in infer_block_return_type)
  - `d:\link\crates\linkc_sema\src\lib.rs:627-631` (Expr::IfExpr in infer_expr)
  - `d:\link\crates\linkc_sema\src\lib.rs:852-876` (Stmt::If / Stmt::While in type-annotation checker)
  - `d:\link\crates\linkc_sema\src\lib.rs:2117-2129` (tests replaced)
- **Verification**:
  - Post-fix: `link compile --backend py O011.link` — rc=0, valid Python generated
  - `link run O011.link` — rc=0, same truthy output as before
  - `cargo test`: 257 total across all crates; 0 failed (linkc_sema: 56/56, linkc_interpreter: 101/101, linkc_parser: 55/55)
  - `cargo build --release`: success
- **Discovered in**: R4 regression run, block O011
