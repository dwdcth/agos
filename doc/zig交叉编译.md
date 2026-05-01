# Zig 交叉编译经验总结

在 Agent Memos 项目的 v0.1.0 版本发布过程中，我们通过引入 **Zig** 编译器解决了复杂的 C++ 扩展交叉编译难题。本文总结了使用 Zig 配合 `cargo-zigbuild` 实现全架构自动化构建的经验。

## 1. 背景与核心痛点

项目依赖 `libsimple`（一个 C++ 编写的 SQLite 中文分词插件），这给持续集成（CI）带来了三大挑战：
*   **工具链依赖**：`libsimple` 的 `build.rs` 会在编译时下载源码并调用系统 `patch` 命令应用补丁。Windows 环境下的 `patch.exe`（通常来自 Strawberry Perl）极不稳定，经常报 `Assertion failed` 错误。
*   **交叉编译复杂性**：传统的交叉编译（如在 x86 Linux 上编译 ARM64 或 LoongArch64）需要手动安装并配置各种 `gcc-aarch64-linux-gnu` 等工具链。
*   **环境不一致**：原生 macOS 和 Windows 的构建环境差异巨大，难以统一管理构建逻辑。

## 2. 为什么选择 Zig？

Zig 不仅仅是一门编程语言，它还是一个极其强大的 **C/C++ 交叉编译器**。
*   **开箱即用**：Zig 内部集成了 Clang 和各种架构的 libc/标准库头文件。你不需要安装任何外部交叉编译器，只需要一个 `zig` 二进制文件。
*   **glibc 版本控制**：可以通过 `target=x86_64-linux-gnu.2.17` 这种语法精确指定依赖的 glibc 版本，确保编译出的二进制文件能在旧版 Linux 上运行。
*   **统一宿主机**：利用 Zig，我们可以在 **Linux (Ubuntu)** 上通过交叉编译直接生成 **Windows** 和 **macOS** 的二进制文件，从而规避了 Windows 宿主机上 `patch` 工具的 Bug。

## 3. 最终架构方案

我们统一使用了 `ubuntu-latest` 作为 GitHub Actions 的宿主机，通过 `cargo-zigbuild` 驱动 Zig 完成所有平台的构建。

### 编译矩阵 (Matrix)
| 目标系统 | 架构 (Target) | 编译器 | 特点 |
| :--- | :--- | :--- | :--- |
| **Linux** | x86_64, AArch64, LoongArch64 | Zig | 完美支持 C++ 扩展，解决 glibc 兼容性 |
| **Windows** | x86_64 (GNU), ARM64 (GNULLVM) | Zig | 在 Linux 上生成 .exe，绕过 Windows patch Bug |
| **macOS** | x86_64, AArch64 (M1/M2) | Clang/Native | macOS 目标依然推荐在 macOS 宿主机或 Zig 兼容模式下构建 |

## 4. 关键配置示例

### GitHub Action 核心片段
```yaml
- name: Install Zig
  uses: goto-bus-stop/setup-zig@v2

- name: Install cargo-zigbuild
  run: cargo install cargo-zigbuild

- name: Build
  # 开启 chinese-tokenizer 功能，Zig 会自动处理关联的 C++ 源码
  run: cargo zigbuild --release --target ${{ matrix.platform.target }} --features "chinese-tokenizer"
```

### Windows 目标的特殊处理
为了在 Linux 上顺利编译 Windows 版本，我们将目标从 `pc-windows-msvc` 切换到了 `pc-windows-gnullvm` 或 `pc-windows-gnu`。这使得 Zig 能够利用内置的 MinGW-w64 库，而不需要连接庞大的 Windows SDK。

## 5. 踩坑与建议

1.  **权限问题**：GitHub Actions 的 `GITHUB_TOKEN` 默认可能没有 Release 写入权限。必须在 workflow 中显式声明 `permissions: contents: write`。
2.  **Node.js 版本**：随着 GitHub 策略更新，建议在 workflow 环境变量中设置 `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true` 以消除过时警告。
3.  **Patch 工具**：如果你的 Rust 依赖（如 `libsimple`）在 `build.rs` 中调用了系统命令，尽量在 Linux 环境下进行交叉编译。Linux 的命令行工具链（patch, sed, awk）远比 Windows 健壮。
4.  **Zig 版本**：建议使用较新的 Zig 版本（如 0.13.0+），因为它对 LoongArch64 和较新的 macOS 架构有更好的支持。

## 结论

通过 **Zig + cargo-zigbuild** 的组合，我们将原本破碎、多平台的构建流程统一到了一个标准的 Linux 环境中。这不仅大幅提升了 CI 的成功率，还让 Agent Memos 实现了全架构（7 种目标平台）的一键自动化发布。
