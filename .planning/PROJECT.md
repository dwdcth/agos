# Agent Memos

## What This Is

Agent Memos 是一个用 Rust 实现的本地优先记忆认知系统，目标是把 `doc/` 中的 0415 认知理论落成可运行的软件骨架。它分为两块核心能力：一块是可解释、可追溯的 ordinary retrieval，另一块是基于智能体的搜索系统，用来在检索之上完成路由、验证、工作记忆组装和行动支持。

这个系统面向需要长期上下文的 AI agent 场景，尤其是编码 agent、知识工作 agent 和需要跨 session 连续性的自动化流程。项目会以 `reference/mempal` 为代码风格参考，但核心模型、领域术语和认知分层以本仓库的 0415 文档组为准。

## Core Value

当 agent 需要回忆历史决策、证据、模式或当前认知状态时，系统必须能快速给出带出处、带时间性、带状态约束的正确记忆，而不是只返回“看起来相似”的文本片段。

## Current State

v1.1 已交付并归档。当前系统在 v1.0 的认知底座之上，进一步具备：

- 可配置的 embedding backend/model 与独立的 vector sidecar/index readiness surface
- additive embedding persistence，不替换 authority store
- lexical-only / embedding-only / hybrid 三种 ordinary retrieval mode，共用一个 `SearchService` 合同
- 显式 dual-channel contribution trace，以及 truthful `status` / `doctor` diagnostics
- 继续复用 ordinary retrieval seam 的 `agent-search --mode`，没有 semantic-only bypass path

仓库当前处于“准备定义下一里程碑”的状态，而不是“继续收尾 v1.1”。

## Next Milestone Goals

- 为下一里程碑定义新的 requirements 与 roadmap，而不是继续沿用 v1.1 的临时目标
- 决定下一个重点是接口扩展、embedding lifecycle/tooling，还是治理/质量硬化
- 保持 lexical-first explainability、ordinary retrieval seam、local-first SQLite 架构不被后续扩展稀释

## Requirements

### Validated

- ✓ 普通检索链路：支持 SQLite 本地存储、`libsimple` FTS5 中文/拼音检索，以及用 Rust 实现的 BM25/TF-IDF 风格轻量关键词权重与可解释 rerank。 — v1.0
- ✓ 基于智能体的搜索链路：在检索之上支持 query 路由、多步检索、证据核验、引用输出和工作记忆装配。 — v1.0
- ✓ 将 0415 理论中的 T1/T2/T3、世界模型、自我模型、技能记忆、注意力、工作记忆、价值层、元认知与反刍机制映射为可演化的软件模块。 — v1.0 cognitive kernel
- ✓ 提供本地优先、单机可运行的 Rust 架构，优先支持 CLI/库/API 形态，后续可扩展到 MCP 或 agent 接口。 — v1.0
- ✓ 形成可扩展的模块边界与数据模型，使后续阶段能独立推进 ingest、index、search、agent、rumination、verification 等子系统。 — v1.0
- ✓ Runtime readiness / doctor gating now governs downstream operational commands. — v1.0 gap-closure
- ✓ Agent-search follow-up retrieval now re-enters working memory and decision selection. — v1.0 gap-closure
- ✓ Optional embedding backend/model config, additive chunk embedding persistence, and vector sidecar readiness landed without weakening the lexical-first baseline. — v1.1
- ✓ Ordinary retrieval now supports lexical-only, embedding-only, and hybrid mode selection derived from a shared root `config.toml` contract. — v1.1
- ✓ Search results and operator diagnostics explicitly report dual-channel contribution, active channels, and gated channels. — v1.1
- ✓ Agent-search now reuses the same runtime-configured ordinary retrieval seam under dual-channel mode selection. — v1.1

### Active

- [ ] 为下一里程碑定义新的 requirements / roadmap（接口、embedding lifecycle、治理工作流、质量硬化等）
- [ ] 决定 MCP / HTTP interface 是否进入下一里程碑范围
- [ ] 决定 richer embedding lifecycle / rebuild tooling 是否成为下一里程碑主线

### Out of Scope

- 多租户云服务与复杂账号体系 — 当前目标是先把本地优先的认知底座跑通。
- 视觉化产品界面优先级高于核心能力 — 核心检索、记忆建模和 agent 搜索稳定性优先。
- 与 LLM provider 深度绑定的单一实现 — 通过 `rig` 保持模型与工具层的可替换性。
- 试图在 v1.x 一次性实现“完整 AGI” — 当前只做记忆认知底座，不做全能自主体。

## Context

### Current State

当前仓库已经交付两个连续 milestone：

- lexical-first ordinary retrieval：SQLite + `libsimple` + explainable rerank
- embedding second-channel retrieval：shared runtime config、hybrid fusion、explicit channel trace
- truth governance：T1/T2/T3、promotion review、ontology candidate
- working memory / value / metacognition：typed runtime cognition surface
- bounded agent-search：thin Rig boundary、cited report、follow-up evidence integration
- rumination / write-back：SPQ / LPQ、local adaptation、candidate-first long cycle

当前剩余的是非阻塞 tech debt，而不是 milestone blocker：

- live Rig smoke 仍然是可选验证
- rumination negative-path coverage 仍可继续加深
- richer embedding lifecycle / index rebuild tooling 仍待定义
- MCP / HTTP interface surface 仍待进入正式 milestone

当前仓库没有业务实现代码，主要输入来自三类材料：

- `doc/`：0415 认知理论文档组，定义了记忆系统与认知系统的边界。核心观点包括：记忆不是仓库而是未来行动的准备系统；纯检索系统与认知系统必须分层；认知系统由记忆、世界模型、自我模型、技能记忆、注意力、工作记忆、行动、元认知、价值层构成闭环。
- `doc/0415-真值层.md`、`doc/0415-自我模型.md`、`doc/0415-注意力状态.md`、`doc/0415-工作记忆.md`、`doc/0415-元认知层.md`、`doc/0415-反刍机制.md`：这些文档进一步给出 T1/T2/T3、侧路调制、候选行动、veto、短周期/长周期反刍等核心结构。
- `reference/mempal/`：一个完整的 Rust 参考实现，展示了单二进制、本地 SQLite、MCP、知识图谱与模块拆分方式。它的 `core / embed / ingest / search / mcp / cowork` 分层，以及对检索服务拆分、taxonomy routing 和本地数据库组织的处理，对本项目的软件组织很有参考价值。

项目的实现导向应满足以下现实目标：

- 普通检索的基线不依赖模型文件，而是以 `libsimple` + SQLite FTS5 + Rust 轻量关键词权重为主，兼顾中文检索体验、时间正确性、出处追踪与 recall explainability。
- 检索架构允许轻量方案与嵌入模型并存：lexical-first 负责稳定基线，embedding 通道负责语义补召回或 rerank，但不应破坏 ordinary retrieval 的可解释性。
- 智能体搜索不能只是包装聊天调用，而要与工作记忆、证据核验、候选行动和元认知监督接起来。
- 数据结构既要能支撑检索，也要能支撑后续的认知装配与跨层写回。

<details>
<summary>Archived milestone context</summary>

已归档的 milestone 细节见：

- [v1.0 roadmap archive](/home/tongyuan/project/agent_memos/.planning/milestones/v1.0-ROADMAP.md)
- [v1.0 requirements archive](/home/tongyuan/project/agent_memos/.planning/milestones/v1.0-REQUIREMENTS.md)
- [v1.1 roadmap archive](/home/tongyuan/project/agent_memos/.planning/milestones/v1.1-ROADMAP.md)
- [v1.1 requirements archive](/home/tongyuan/project/agent_memos/.planning/milestones/v1.1-REQUIREMENTS.md)

</details>

## Constraints

- **Tech stack**: 必须使用 Rust 实现核心系统 — 用户明确要求用 Rust 落地。
- **Retrieval baseline**: 必须以 `libsimple = "~0.9"` + SQLite FTS5 + Rust 轻量关键词权重为主，不要求模型文件或嵌入服务。
- **Optional extension**: `sqlite-vec` 可以作为可选语义检索扩展，但不能成为 lexical-first baseline 的前置依赖。
- **Coexistence rule**: 轻量检索与 embedding 检索可以并存，但 embedding 只能作为第二通道、补召回或 rerank，不应替代 lexical-first 基线。
- **Agent framework**: 智能体搜索必须基于 `rig` 设计 — 保持模型与工具层的可扩展性。
- **Architecture style**: 代码框架需要参考 `reference/mempal` — 优先模仿其模块拆分、单二进制组织与本地数据库思路。
- **Local-first**: 系统以单机、本地 SQLite 数据库为核心 — 避免先引入分布式依赖和复杂运维。
- **Explainability**: 检索与 agent 搜索都必须保留引用、来源和时间/状态解释 — 否则无法支撑认知系统的可信使用。
- **Scope control**: 每个 milestone 优先收敛单一主线，避免同时展开 UI、部署平台或多租户能力。

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| 项目按“ordinary retrieval + agent-search”双主线拆分 | 用户已明确要求两块能力同时存在，且理论上 recall 与 cognition 本就应分层 | ✓ Good |
| 检索底座采用 SQLite + `libsimple` + Rust 轻量关键词权重 | 更贴近当前规模和目标，避免把模型文件或语义向量作为前提 | ✓ Good |
| `sqlite-vec` 仅作为可选扩展，而不是必选依赖 | 若后续 lexical-first 路线出现召回瓶颈，可平滑追加语义检索能力 | ✓ Good |
| 轻量检索与 embedding 检索允许并存 | lexical-first 负责稳定与可解释性，embedding 通道负责语义补强与 rerank | ✓ Good |
| embedding second-channel 继续通过 ordinary retrieval seam 暴露 | 保持 `search` 与 `agent-search` contract 一致，避免 semantic-only bypass | ✓ Good |
| 智能体层采用 `rig`，而不是自写一层 provider glue | 后续需要模型、embedding、tool、agent 组合能力，`rig` 更适合作为 orchestration 层 | ✓ Good |
| 软件骨架参考 `reference/mempal` 的模块布局 | `mempal` 提供了合适的 Rust 模块拆分与 SQLite 搜索实践 | ✓ Good |
| 0415 文档组作为领域真源 | 该项目的差异化价值在认知理论，而不是通用检索壳子 | ✓ Good |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `$gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `$gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-17 after v1.1 milestone archive*
