# 安装

## 前置依赖

Link 用 Rust 编写,需要 Rust 工具链才能从源码构建。

### Rust 工具链

=== "Windows"

    使用 [rustup](https://rustup.rs/) 安装:
    
    ```powershell
    # 下载 rustup-init.exe 并运行
    # 或使用 winget:
    winget install Rustlang.Rustup
    ```
    
    **MSVC 工具链**(推荐):
    
    需安装 Visual Studio Build Tools,包含:
    - MSVC v143 - VS 2022 C++ x64/x86 build tools
    - Windows 11 SDK
    
    下载:[vs_BuildTools.exe](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
    
    **GNU 工具链**(备选):
    
    ```powershell
    rustup default stable-x86_64-pc-windows-gnu
    ```

=== "Linux"

    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env
    ```

=== "macOS"

    ```bash
    # 通过 Homebrew
    brew install rustup
    rustup-init
    ```

验证安装:

```bash
cargo --version
rustc --version
```

### Python(可选,用于 Python FFI)

如需使用 `extern "python"` 调用 Python 函数,需安装 Python 3.8+:

=== "Windows"

    ```powershell
    winget install Python.Python.3.11
    ```
    
    或从 [python.org](https://www.python.org/downloads/) 下载。
    安装时勾选 "Add Python to PATH"。

=== "Linux"

    ```bash
    # Ubuntu / Debian
    sudo apt install python3 python3-dev

    # CentOS / RHEL
    sudo dnf install python3 python3-devel
    ```

=== "macOS"

    ```bash
    brew install python
    ```

### C++ 编译器(可选,用于 C++ FFI)

如需编译自己的 C++ 共享库供 Link 调用:

=== "Windows"

    随 Visual Studio Build Tools 安装 MSVC,或安装 MinGW:
    
    ```powershell
    winget install MartinStorsjo.LLVM-MinGW.UCRT
    ```

=== "Linux"

    ```bash
    # GCC
    sudo apt install g++

    # 或 Clang
    sudo apt install clang
    ```

=== "macOS"

    ```bash
    xcode-select --install
    ```

## 从源码构建

```bash
git clone https://github.com/myiunagn/link.git
cd link
cargo build --release
```

构建产物位于 `target/release/link`(Windows 上为 `link.exe`)。

## 安装到 PATH

=== "Linux / macOS"

    ```bash
    # 创建符号链接到 ~/.local/bin
    mkdir -p ~/.local/bin
    ln -s "$(pwd)/target/release/link" ~/.local/bin/link

    # 确保 ~/.local/bin 在 PATH 中
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
    source ~/.bashrc

    # 验证
    link --version
    ```

=== "Windows"

    ```powershell
    # 创建符号链接到用户 bin 目录
    $binDir = "$env:USERPROFILE\bin"
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null
    Copy-Item ".\target\release\link.exe" "$binDir\link.exe"

    # 将 bin 目录加入 PATH(永久)
    [Environment]::SetEnvironmentVariable(
        "Path",
        "$binDir;" + [Environment]::GetEnvironmentVariable("Path", "User"),
        "User"
    )

    # 刷新当前会话
    $env:Path = "$binDir;$env:Path"

    # 验证
    link --version
    ```

## 验证安装

```bash
link --version
# 输出:linkc 0.1.0
```

运行测试套件确认环境正常:

```bash
cargo test
```

预期输出:

```
test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## VS Code 扩展(可选)

仓库内置 VS Code 语法高亮扩展,位于 `editors/vscode/`:

```bash
cd editors/vscode
npm install
npm run compile
```

按 `F5` 启动扩展开发宿主窗口,打开 `.link` 文件即可看到语法高亮。

## 故障排查

### `cargo build` 报错 "link.exe not found"

Windows 上未安装 MSVC Build Tools。解决方法:

1. 安装 Visual Studio Build Tools(见前置依赖)
2. 或切换到 GNU 工具链:`rustup default stable-x86_64-pc-windows-gnu`

### Python FFI 报错 "Failed to load python3.dll"

Python 未在 PATH 中。解决方法:

- Windows:确认安装时勾选了 "Add Python to PATH"
- Linux:确认安装了 `python3-dev`(Debian/Ubuntu)或 `python3-devel`(RHEL/CentOS)
- 也可手动指定 DLL 路径:`extern "python" module "C:/Python311/python311.dll"`

### C++ FFI 报错 "Failed to load library"

C++ 共享库路径错误或架构不匹配。检查:

1. DLL/SO 路径是否正确(相对路径基于运行目录)
2. 架构是否匹配(64 位 Link 需要 64 位 DLL)
3. C++ 函数是否用 `extern "C"` 导出(否则会被名称修饰)

## 下一步

- [快速开始](quickstart.md)
- [REPL 交互模式](repl.md)
