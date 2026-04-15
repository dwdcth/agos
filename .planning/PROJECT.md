# Agent Memos

## What This Is

Agent Memos 是一个用 Rust 实现的本地优先记忆认知系统，目标是把 `doc/` 中的 0415 认知理论落成可运行的软件骨架。它分为两块核心能力：一块是可解释、可追溯的普通检索系统，另一块是基于智能体的搜索系统，用来在检索之上完成路由、验证、工作记忆组装和行动支持。

这个系统面向需要长期上下文的 AI agent 场景，尤其是编码 agent、知识工作 agent 和需要跨 session 连续性的自动化流程。项目会以 `reference/mempal` 为代码风格参考，但核心模型、领域术语和认知分层以本仓库的 0415 文档组为准，和这个项目没有一点关系。

## Core Value

当 agent 需要回忆历史决策、证据、模式或当前认知状态时，系统必须能快速给出带出处、带时间性、带状态约束的正确记忆，而不是只返回“看起来相似”的文本片段。

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] 实现普通检索链路：支持 SQLite 本地存储、`libsimple` FTS5 中文/拼音检索，以及用 Rust 实现的 BM25/TF-IDF 风格轻量关键词权重与可解释 rerank。
- [ ] 实现基于智能体的搜索链路：在检索之上支持 query 路由、多步检索、证据核验、引用输出和工作记忆装配。
- [ ] 将 0415 理论中的 T1/T2/T3、世界模型、自我模型、技能记忆、注意力、工作记忆、价值层、元认知与反刍机制映射为可演化的软件模块。
- [ ] 提供本地优先、单机可运行的 Rust 架构，优先支持 CLI/库/API 形态，后续可扩展到 MCP 或 agent 接口。
- [ ] 形成可扩展的模块边界与数据模型，使后续阶段能独立推进 ingest、index、search、agent、rumination、verification 等子系统。

### Out of Scope

- 多租户云服务与复杂账号体系 — 当前目标是先把本地优先的认知底座跑通。
- 视觉化产品界面优先级高于核心能力 — v1 先保证检索、记忆建模和 agent 搜索可用。
- 与 LLM provider 深度绑定的单一实现 — 通过 `rig` 保持模型与工具层的可替换性。
- 试图在 v1 一次性实现“完整 AGI” — 当前只做记忆认知底座，不做全能自主体。

## Context

当前仓库没有业务实现代码，主要输入来自三类材料：

- `doc/`：0415 认知理论文档组，定义了记忆系统与认知系统的边界。核心观点包括：记忆不是仓库而是未来行动的准备系统；纯检索系统与认知系统必须分层；认知系统由记忆、世界模型、自我模型、技能记忆、注意力、工作记忆、行动、元认知、价值层构成闭环。
- `doc/0415-真值层.md`、`doc/0415-自我模型.md`、`doc/0415-注意力状态.md`、`doc/0415-工作记忆.md`、`doc/0415-元认知层.md`、`doc/0415-反刍机制.md`：这些文档进一步给出 T1/T2/T3、侧路调制、候选行动、veto、短周期/长周期反刍等核心结构。
- `reference/mempal/`：一个完整的 Rust 参考实现，展示了单二进制、本地 SQLite、MCP、知识图谱与模块拆分方式。它的 `core / embed / ingest / search / mcp / cowork` 分层，以及对检索服务拆分、taxonomy routing 和本地数据库组织的处理，对本项目的软件组织很有参考价值。

项目的实现导向应满足以下现实目标：

- 普通检索的 v1 基线不依赖模型文件，而是以 `libsimple` + SQLite FTS5 + Rust 轻量关键词权重为主，兼顾中文检索体验、时间正确性、出处追踪与 recall explainability。
- 智能体搜索不能只是包装聊天调用，而要与工作记忆、证据核验、候选行动和元认知监督接起来。
- 数据结构既要能支撑检索，也要能支撑后续的认知装配与跨层写回。

## Constraints

- **Tech stack**: 必须使用 Rust 实现核心系统 — 用户明确要求用 Rust 落地。
- **Retrieval baseline**: v1 必须以 `libsimple = "~0.9"` + SQLite FTS5 + Rust 轻量关键词权重为主，不要求模型文件或嵌入服务。
- **Optional extension**: `sqlite-vec` 可以作为后续可选语义检索扩展，但不是 v1 必选前提。
- **Agent framework**: 智能体搜索必须基于 `rig` 设计 — 保持模型与工具层的可扩展性。
- **Architecture style**: 代码框架需要参考 `reference/mempal` — 优先模仿其模块拆分、单二进制组织与本地数据库思路。
- **Local-first**: v1 以单机、本地 SQLite 数据库为核心 — 避免先引入分布式依赖和复杂运维。
- **Explainability**: 检索与 agent 搜索都必须保留引用、来源和时间/状态解释 — 否则无法支撑认知系统的可信使用。
- **Scope control**: 第一阶段优先做认知底座与搜索链路，不提前展开 UI、部署平台或多租户能力 — 保持项目收敛。

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| 项目按“普通检索 + 智能体搜索”双主线拆分 | 用户已明确要求两块能力同时存在，且理论上 recall 与 cognition 本就应分层 | — Pending |
| 检索底座采用 SQLite + `libsimple` + Rust 轻量关键词权重 | 更贴近当前规模和目标，避免把模型文件或语义向量作为 v1 前提 | — Pending |
| `sqlite-vec` 仅作为可选扩展，而不是必选依赖 | 若后续 lexical-first 路线出现召回瓶颈，可平滑追加语义检索能力 | — Pending |
| 智能体层采用 `rig`，而不是自写一层 provider glue | 后续需要模型、embedding、tool、agent 组合能力，`rig` 更适合作为 agent orchestration 层 | — Pending |
| 软件骨架参考 `reference/mempal` 的模块布局 | 当前仓库缺少现成代码，`mempal` 提供了合适的 Rust 模块拆分与 SQLite 搜索实践 | — Pending |
| 0415 文档组作为领域真源 | 该项目的差异化价值在认知理论，而不是通用检索壳子 | — Pending |

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
*Last updated: 2026-04-15 after initialization*
