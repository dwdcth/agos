# GitHub Actions 自动化构建与发布经验总结

在 Agent Memos 项目的 `v0.1.0` 发布过程中，我们建立了一套基于 GitHub Actions 的全架构自动化构建与 Release 流水线。本文总结了在配置过程中的关键经验与问题解决方法。

## 1. 工作流目标

实现“打标即发布”：
*   **触发机制**：仅在推送以 `v` 开头的标签（如 `v0.1.0`）时触发。
*   **多平台覆盖**：同时构建 Linux (x86, ARM, LoongArch)、macOS (Intel, M1/M2) 和 Windows (x86, ARM) 的 Release 二进制包。
*   **自动 Release**：构建成功后自动创建 GitHub Release 页面，并将所有平台的压缩包作为附件上传。

## 2. 核心架构设计

### 统一构建环境
为了解决不同平台间工具链差异（尤其是 Windows 上的 `patch` 报错），我们采用了**统一宿主机策略**：
*   **宿主机**：统一使用 `ubuntu-latest`。
*   **交叉编译工具**：使用 `zig` + `cargo-zigbuild`。这种方式允许我们在稳定的 Linux 环境下，交叉生成 macOS 和 Windows 的二进制文件。

### 编译矩阵 (Matrix)
利用 `matrix` 策略并行运行 7 个构建任务：
```yaml
strategy:
  matrix:
    platform:
      - { target: x86_64-unknown-linux-gnu, bin: agent-memos }
      - { target: aarch64-unknown-linux-gnu, bin: agent-memos }
      - { target: x86_64-pc-windows-gnu, bin: agent-memos.exe }
      # ... 其他 4 个平台
```

## 3. 踩坑与修复记录

### A. 权限不足 (403 Forbidden)
**现象**：`softprops/action-gh-release` 报错 `Resource not accessible by integration`。
**原因**：GitHub 默认限制了 `GITHUB_TOKEN` 的写入权限。
**修复**：在 workflow 中显式声明 `permissions`：
```yaml
permissions:
  contents: write
```

### B. Node.js 版本过时警告
**现象**：警告 `Node.js 20 actions are deprecated`。
**修复**：通过环境变量强制使用 Node.js 24：
```yaml
env:
  FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true
```

### C. Windows 交叉编译难题
**现象**：默认的 `x86_64-pc-windows-msvc` 目标在 Linux 宿主机上非常难配置。
**修复**：将 Windows 目标切换为 `pc-windows-gnu` 或 `pc-windows-gnullvm`。这些目标能完美配合 Zig 的 MinGW 工具链，实现免 SDK 的 Windows 交叉编译。

### D. C++ 补丁应用失败
**现象**：`libsimple` 的 `build.rs` 在 Windows 宿主机上调用 `patch` 时崩溃。
**修复**：通过将构建任务迁移到 `ubuntu-latest`，利用 Linux 系统自带的健壮 `patch` 工具完成源码修补。

## 4. 最佳实践建议

1.  **标签管理**：推送前务必确认本地标签指向正确的提交。重新构建建议删除远程标签再重新推送：
    ```bash
    git tag -d v0.1.0 && git tag v0.1.0
    git push gh :refs/tags/v0.1.0 && git push gh v0.1.0
    ```
2.  **Artifact 传递**：利用 `actions/upload-artifact` 在 `build` 任务和 `release` 任务之间传递二进制文件，保持流程解耦。
3.  **Fail-fast 设置**：设置 `fail-fast: false`，确保即便某个架构（如 LoongArch）编译失败，其他架构的构建和发布依然能继续。

## 结论

GitHub Actions 配合 Zig 编译器极大地简化了 Rust 项目的跨平台交付。通过将所有架构的构建逻辑收拢到单个 Linux workflow 中，我们不仅规避了环境碎片化带来的 Bug，还实现了极高的构建成功率和分发效率。
