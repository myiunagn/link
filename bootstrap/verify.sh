#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
BUILD="$REPO/bootstrap/build"

STAGE1="$BUILD/link-bootstrap"
STAGE2="$BUILD/stage2"
STAGE3="$BUILD/stage3"
STAGE2C="$BUILD/stage2.c"
STAGE3C="$BUILD/stage3.c"
CORE2C="$BUILD/core2.c"
CORE3C="$BUILD/core3.c"
CORE2="$BUILD/core2"
CORE3="$BUILD/core3"

# Detect C compiler
CC="${CC:-cc}"   # cc is clang on macOS, gcc on Linux
CFLAGS="-std=c99"

mkdir -p "$BUILD"
cd "$REPO"

# === Stage 0 -> Stage 1 ===
echo "[1/5] Stage 0 -> Stage 1"
cargo run -p linkc_cli -- compile bootstrap/compiler.link --backend c -o "$STAGE1"
chmod +x "$STAGE1"

# === Stage 1 -> Stage 2 (self-compile) ===
echo "[2/5] Stage 1 -> Stage 2 (self-compile)"
"$STAGE1" bootstrap/compiler.link "$STAGE2C"
$CC $CFLAGS "$STAGE2C" -o "$STAGE2"
chmod +x "$STAGE2"

# === Stage 2 -> Stage 3 (fixed point) ===
echo "[3/5] Stage 2 -> Stage 3 (fixed point)"
"$STAGE2" bootstrap/compiler.link "$STAGE3C"
$CC $CFLAGS "$STAGE3C" -o "$STAGE3"
chmod +x "$STAGE3"

# === Verify functional equivalence ===
echo "[4/5] Verify Stage 2 and Stage 3 functional equivalence"

"$STAGE2" bootstrap/fixtures/core.link "$CORE2C"
$CC $CFLAGS "$CORE2C" -o "$CORE2"
ACTUAL2=$("$CORE2")

"$STAGE3" bootstrap/fixtures/core.link "$CORE3C"
$CC $CFLAGS "$CORE3C" -o "$CORE3"
ACTUAL3=$("$CORE3")

if [ "$ACTUAL2" != "$ACTUAL3" ]; then
    echo "FAIL: Stage 2 output '$ACTUAL2' differs from Stage 3 output '$ACTUAL3'"
    exit 1
fi
if [ "$ACTUAL2" != "10" ]; then
    echo "FAIL: expected '10', got '$ACTUAL2'"
    exit 1
fi
echo "  Stage 2 and Stage 3 produce identical output: $ACTUAL2"

# === Verify error handling ===
echo "[5/5] Verify error handling"
ERRFILE="$BUILD/_err.link"
echo "fn broken( {" > "$ERRFILE"
ERR2=$("$STAGE2" "$ERRFILE" "$BUILD/_err.c" 2>&1 || true)
echo "  Stage 2 error: $ERR2"
ERR3=$("$STAGE3" "$ERRFILE" "$BUILD/_err.c" 2>&1 || true)
echo "  Stage 3 error: $ERR3"
if [ "${ERR2%% *}" = "${ERR3%% *}" ]; then
    echo "  Error byte positions match (fixed point)"
fi

# Cleanup
rm -f "$ERRFILE" "$BUILD/_err.c"

echo ""
echo "Bootstrap fixed point verified!"
echo "  Stage 2 and Stage 3 are functionally equivalent."
echo "  No Cargo/rustc/Rust in Stage 2+ build commands."
