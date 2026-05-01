# Agent Memos (认知记忆内核)

Agent Memos 是一个基于 Rust 开发的、本地优先（Local-first）的 Agent 认知记忆系统。它不仅是一个存储事实的数据库，更是一个模仿人类认知闭环的逻辑引擎。

## 🧠 系统设计架构

本系统的设计核心在于将“存储”升华为“认知”，分为以下四个关键层级：

### 1. 记 (Ingest Layer) - 感知与编码
将原始输入（文档、对话、网页等）转化为机器可理解的结构化知识。
*   **语义切片**：自动进行智能文本分段。
*   **DSL 提取**：利用 LLM 提取领域（Domain）、主题（Topic）和断言（Claim）。
*   **向量化**：通过模型（如 BGE-M3）生成高维语义索引。
*   **[详细设计: 记层协议](doc/0415-00记忆认知架构.md#1-记层设计)**

### 2. 存 (Memory Layer) - 持久化存储
基于 SQLite 和其向量扩展（sqlite-vec）实现的本地可靠存储。
*   **真值层 (Truth Layer)**：管理知识的可信度与演化。
*   **反刍机制 (Rumination)**：后台自动优化记忆关联。
*   **[详细设计: 真值层管理](doc/0415-真值层.md)** | **[反刍机制](doc/0415-反刍机制.md)**

### 3. 忆 (Search Layer) - 双通道召回
结合了传统关键词搜索与现代语义检索的混合引擎。
*   **词法通道 (Lexical)**：精准匹配关键词。
*   **语义通道 (Embedding)**：理解意图，跨术语召回相关知识。
*   **[详细设计: 认知索引](doc/0415-认知索引.md)**

### 4. 识 (Cognition Layer) - 决策与推理
系统的“大脑”，将找回的记忆转化为可执行的决策支持。
*   **工作记忆 (Working Memory)**：动态构建当前任务的上下文。
*   **元认知 (Metacognition)**：由 LLM 评估信息的风险、价值与冲突。
*   **[详细设计: 工作记忆](doc/0415-工作记忆.md)** | **[元认知层](doc/0415-元认知层.md)**

---

## 🛠️ 如何使用

### 1. 准备配置
复制 `config/agent-memos.toml.example` 为 `config.toml`，并填入你的 SiliconFlow 或 OpenAI API Key。

```toml
[llm]
model = "deepseek-ai/DeepSeek-V3.2"
api_base = "https://api.siliconflow.cn/v1"
api_key = "sk-..."

[embedding]
model = "BAAI/bge-m3"
api_base = "https://api.siliconflow.cn/v1"
api_key = "sk-..."
```

### 2. 初始化环境
```bash
cargo run -- --config config.toml init
cargo run -- --config config.toml doctor  # 检查后端服务是否就绪
```

### 3. 知识注入 (记)
```bash
cargo run -- --config config.toml ingest \
  --source-uri "memo://mars/safety" \
  --recorded-at "2026-05-01T10:00:00Z" \
  --content "在火星温室中，严禁用水灭火，因为会造成高压设备短路。"
```

### 4. 智能搜索 (忆 & 识)
```bash
# 使用混合模式搜索
cargo run -- --config config.toml agent-search "如何处理植物区的火情？" --mode hybrid --json
```

---

## 📦 支持架构

通过 GitHub Actions 自动构建，支持以下平台：
*   **Linux**: x86_64, AArch64 (ARM64), LoongArch64
*   **macOS**: x86_64 (Intel), AArch64 (Apple Silicon)
*   **Windows**: x86_64, AArch64

## 📖 更多文档
*   [概念对照表](doc/0415-概念对照表.md)
*   [理论与实现对应审计](doc/理论实现对应.md)
*   [世界模型设计](doc/0415-世界模型.md)
