<INSTRUCTIONS>
@/Users/mac/.codex/RTK.md
</INSTRUCTIONS>

<!-- GSD:project-start source:PROJECT.md -->
## Project

**Agent Memos**

Agent Memos 是一个用 Rust 实现的本地优先记忆认知系统，目标是把 `doc/` 中的 0415 认知理论落成可运行的软件骨架。它分为两块核心能力：一块是可解释、可追溯的普通检索系统，另一块是基于智能体的搜索系统，用来在检索之上完成路由、验证、工作记忆组装和行动支持。

这个系统面向需要长期上下文的 AI agent 场景，尤其是编码 agent、知识工作 agent 和需要跨 session 连续性的自动化流程。项目会以 `reference/mempal` 为代码风格参考，但核心模型、领域术语和认知分层以本仓库的 0415 文档组为准，和这个项目没有一点关系。

**Core Value:** 当 agent 需要回忆历史决策、证据、模式或当前认知状态时，系统必须能快速给出带出处、带时间性、带状态约束的正确记忆，而不是只返回“看起来相似”的文本片段。

### Constraints

- **Tech stack**: 必须使用 Rust 实现核心系统 — 用户明确要求用 Rust 落地。
- **Vector retrieval**: 必须使用 `sqlite-vec` — 作为本地优先向量检索底座。
- **Lexical retrieval**: 必须使用 `libsimple = "~0.9"` — 需要支持 SQLite FTS5 的中文/拼音 tokenizer 能力。
- **Agent framework**: 智能体搜索必须基于 `rig` 设计 — 保持模型、工具和向量存储集成的可扩展性。
- **Architecture style**: 代码框架需要参考 `reference/mempal` — 优先模仿其模块拆分、单二进制组织与本地数据库思路。
- **Local-first**: v1 以单机、本地 SQLite 数据库为核心 — 避免先引入分布式依赖和复杂运维。
- **Explainability**: 检索与 agent 搜索都必须保留引用、来源和时间/状态解释 — 否则无法支撑认知系统的可信使用。
- **Scope control**: 第一阶段优先做认知底座与搜索链路，不提前展开 UI、部署平台或多租户能力 — 保持项目收敛。
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## Recommended Stack
### Core Technologies
| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust | 1.85+ | Core implementation language | `libsimple 0.9.0` declares `rust-version = 1.85.0`, and the project needs strong type boundaries for cognitive layers plus a single-binary deployment path. |
| SQLite + rusqlite | `rusqlite 0.37.x` | Primary local data store and query engine | Fits the local-first constraint, composes well with FTS5 and vector extensions, and is already proven by `reference/mempal`. |
| sqlite-vec | `0.1.x` | Vector similarity search inside SQLite | Official project positions it as a small vector-search extension that runs wherever SQLite runs, which matches this project's deployment model. |
| libsimple | `~0.9` | Chinese / PinYin FTS5 tokenizer | Official crate metadata describes it as Rust bindings for a SQLite FTS5 tokenizer with Chinese and PinYin support, which directly addresses Chinese retrieval quality. |
| rig-core | Latest compatible release at implementation time | Agent orchestration, model abstraction, tools, embeddings | Official Rig docs emphasize provider unification, agent workflows, and vector-store integrations, making it the right orchestration layer for agentic search. |
### Supporting Libraries
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio | `1.x` | Async runtime | Needed for rig-based agent workflows, embedding calls, background rumination jobs, and optional MCP/API surfaces. |
| serde / serde_json | `1.x` | Typed persistence and API payloads | Use for cognitive state snapshots, search responses, traces, and schema evolution metadata. |
| thiserror / anyhow | `2.x / 1.x` | Error modeling and propagation | Use `thiserror` for domain errors and `anyhow` at interface boundaries or CLI entrypoints. |
| tracing / tracing-subscriber | `0.1 / 0.3` | Observability | Required to debug retrieval decisions, agent reasoning routes, and rumination jobs. |
| clap | `4.x` | CLI interface | Use for early product surface, parity with `mempal`, and inspection/debug workflows. |
| axum | `0.8.x` | Optional HTTP / MCP-adjacent service surface | Add once search and agent workflows are stable and need remote invocation. |
### Development Tools
| Tool | Purpose | Notes |
|------|---------|-------|
| cargo fmt / clippy | Style and lint gates | Keep module boundaries clean while the model is still changing quickly. |
| cargo nextest or cargo test | Verification | Needed once truth-layer promotion, search fusion, and agent-search orchestration become test-heavy. |
| sqlite3 CLI / DB Browser | Inspect SQLite schema and FTS/vec behavior | Useful during schema and ranking tuning. |
## Installation
# Core
# Required lexical search support
# Agentic search
# Optional service surface
## Alternatives Considered
| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| SQLite + `sqlite-vec` | External vector DB | Only if local-first is abandoned and multi-tenant scale becomes the primary goal. |
| `libsimple` FTS5 | Pure BM25 tokenization without Chinese support | Only if the corpus is guaranteed to be English-only. |
| `rig-core` as orchestration layer | Hand-rolled provider adapters | Only for an extremely narrow single-provider prototype; otherwise the abstraction cost pays for itself. |
| Custom hybrid-search layer on top of SQLite | `rig-sqlite` as the primary data model | Use `rig-sqlite` only as a later adapter if it fits; the core store here needs richer truth layers and lexical fusion than a vector-store-first model. |
## What NOT to Use
| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Vector-only retrieval as the sole search path | Chinese lexical recall, exact-match decisions, and symbolic cues will be lost | Hybrid retrieval with `libsimple` + `sqlite-vec` + explicit reranking |
| Letting Rig own the primary memory schema too early | The project's differentiator is cognitive modeling, not generic RAG plumbing | Keep Rig at the orchestration/tool layer and own the core schema directly |
| Cloud-first infra in v1 | Violates the local-first constraint and adds operational drag before the cognition model is proven | Single-file SQLite-first deployment |
| Blindly copying `reference/mempal` crate-for-crate | `mempal` is a memory product reference, but this project needs extra layers for T1/T2/T3, working memory, metacognition, and rumination | Reuse its module discipline, not its exact domain model |
## Stack Patterns by Variant
- Use `clap` + `rusqlite` + direct service objects
- Because it keeps the feedback loop tight while retrieval and cognition semantics are still moving
- Add `axum` or MCP bindings around the same application services
- Because interface expansion should not rewrite the retrieval and cognition core
- Add a thin `rig` adapter module over the internal search/working-memory services
- Because the core ranking and truth-layer semantics should stay stable even if agent tooling changes
## Version Compatibility
| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `libsimple 0.9.0` | `rusqlite >=0.32,<1.0` | Verified from the crate metadata on docs.rs. |
| `sqlite-vec 0.1.x` | SQLite / rusqlite with extension loading | Needs explicit extension initialization before vector queries. |
| `rig-core` latest compatible | Matching Rig companion crates only when needed | Pin `rig-core` and any `rig-*` companions together during implementation. |
## Sources
- `doc/0415-00记忆认知架构.md` — project-specific domain theory and system boundaries
- `reference/mempal/README_zh.md` and `reference/mempal/Cargo.toml` — proven Rust local-first memory architecture reference
- https://docs.rs/crate/libsimple/latest — confirmed `libsimple 0.9.0`, Rust version, compatibility, and tokenizer scope
- https://github.com/asg017/sqlite-vec — confirmed project positioning, deployment model, and pre-v1 status
- https://github.com/0xPlaygrounds/rig and https://docs.rs/crate/rig-core/latest — confirmed Rig's agent/provider/vector-store abstractions
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->
## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, or `.github/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
