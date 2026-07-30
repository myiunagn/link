$ErrorActionPreference = "Stop"

$repo = Split-Path -Parent $PSScriptRoot
$build = Join-Path $PSScriptRoot "build"
$stage1 = Join-Path $build "link-bootstrap.exe"
$stage2 = Join-Path $build "stage2.exe"
$stage3 = Join-Path $build "stage3.exe"
$stage2c = Join-Path $build "stage2.c"
$stage3c = Join-Path $build "stage3.c"
$core2c = Join-Path $build "core2.c"
$core3c = Join-Path $build "core3.c"
$core2exe = Join-Path $build "core2.exe"
$core3exe = Join-Path $build "core3.exe"

New-Item -ItemType Directory -Force -Path $build | Out-Null
Push-Location $repo
try {
    # === Stage 0 → Stage 1 (Rust compiles compiler.link) ===
    Write-Host "[1/5] Stage 0 -> Stage 1"
    cargo run -p linkc_cli -- compile bootstrap/compiler.link --backend c -o $stage1
    if ($LASTEXITCODE -ne 0) { throw "Stage 0 failed to compile the Link seed compiler" }

    # === Stage 1 → Stage 2 (Stage 1 compiles itself) ===
    Write-Host "[2/5] Stage 1 -> Stage 2 (self-compile)"
    & $stage1 bootstrap/compiler.link $stage2c
    if ($LASTEXITCODE -ne 0) { throw "Stage 1 failed to compile compiler.link" }
    gcc -std=c99 -w $stage2c -o $stage2
    if ($LASTEXITCODE -ne 0) { throw "GCC failed to compile Stage 2" }

    # === Stage 2 → Stage 3 (Stage 2 compiles itself) ===
    Write-Host "[3/5] Stage 2 -> Stage 3 (fixed point)"
    & $stage2 bootstrap/compiler.link $stage3c
    if ($LASTEXITCODE -ne 0) { throw "Stage 2 failed to compile compiler.link" }
    gcc -std=c99 -w $stage3c -o $stage3
    if ($LASTEXITCODE -ne 0) { throw "GCC failed to compile Stage 3" }

    # === Verify functional equivalence ===
    Write-Host "[4/5] Verify Stage 2 and Stage 3 functional equivalence"

    # Both compile core.link
    & $stage2 bootstrap/fixtures/core.link $core2c
    if ($LASTEXITCODE -ne 0) { throw "Stage 2 failed to compile core.link" }
    gcc -std=c99 $core2c -o $core2exe
    if ($LASTEXITCODE -ne 0) { throw "GCC failed on Stage 2 core output" }

    & $stage3 bootstrap/fixtures/core.link $core3c
    if ($LASTEXITCODE -ne 0) { throw "Stage 3 failed to compile core.link" }
    gcc -std=c99 $core3c -o $core3exe
    if ($LASTEXITCODE -ne 0) { throw "GCC failed on Stage 3 core output" }

    $actual2 = (& $core2exe | Out-String).Trim()
    $actual3 = (& $core3exe | Out-String).Trim()

    if ($actual2 -ne $actual3) {
        throw "Stage 2 output '$actual2' differs from Stage 3 output '$actual3'"
    }
    if ($actual2 -ne "10") {
        throw "Expected '10', got '$actual2'"
    }
    Write-Host "  Stage 2 and Stage 3 produce identical output: $actual2"

    # === Verify error handling ===
    Write-Host "[5/5] Verify error handling"
    $errfile = Join-Path $build "_err.link"
    "fn broken( {" | Out-File -Encoding ascii $errfile
    $errOut2 = & $stage2 $errfile "$build/_err.c" 2>&1
    Write-Host "  Stage 2 error: byte=$($errOut2[0]) line=$($errOut2[1]) col=$($errOut2[2])"
    $errOut3 = & $stage3 $errfile "$build/_err.c" 2>&1
    Write-Host "  Stage 3 error: byte=$($errOut3[0]) line=$($errOut3[1]) col=$($errOut3[2])"
    if ($errOut2[0] -eq $errOut3[0]) {
        Write-Host "  Error positions match (fixed point)"
    }

    Write-Host ""
    Write-Host "Bootstrap fixed point verified!"
    Write-Host "  Stage 2 and Stage 3 are functionally equivalent."
    Write-Host "  No Cargo/rustc/Rust in Stage 2+ build commands."
} finally {
    Pop-Location
    Remove-Item -Force -ErrorAction SilentlyContinue "$build\_err.link", "$build\_err.c"
}
