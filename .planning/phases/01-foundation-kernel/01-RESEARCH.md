# Phase 1: Foundation Kernel - Research

**Researched:** 2026-04-15  
**Domain:** Rust 本地优先基础内核、SQLite 启动底座、TOML 三态检索配置  
**Confidence:** HIGH

<user_constraints>
## User Constraints

### Locked Decisions
[VERIFIED: .planning/phases/01-foundation-kernel/01-CONTEXT.md]
- Configuration must use TOML as the primary local config format.
- Phase 1 should define a typed config surface early, so later phases can extend it without format churn.
- Config must express retrieval strategy as three explicit modes rather than a single boolean:
  - `lexical_only`, meaning no embedding model
  - `embedding_only`
  - `hybrid`, meaning lexical-lightweight plus embedding together
- Phase 1 should model this as a stable typed enum/string mode in config, even if only part of the behavior is exercised immediately.
- Embedding must remain optional in foundation work, because ordinary retrieval v1 does not require model files or embedding services.
- Startup and status checks must treat all three configured modes as valid states and explain what is currently active, unavailable, or deferred rather than collapsing them into a generic enabled/disabled flag.
- The system should preserve a clean extension path for later optional embedding backends without forcing vector infrastructure into the Phase 1 critical path.
- The config and service boundaries should leave room for later coexistence between lexical-first retrieval and embedding-based retrieval under one search surface, with embedding remaining a secondary path for expansion or rerank rather than replacing the lexical baseline.
- The three modes should be documented as different retrieval intents, not only different implementation toggles:
  - `lexical_only` for precise, source-aware, strongly explainable retrieval
  - `embedding_only` for semantic-intent-heavy recall where wording is unstable
  - `hybrid` for mixed corpora or mixed query styles where lexical remains the baseline explanation source
- The data model created in Phase 1 must already preserve source, timestamp, scope, record type, and truth metadata required by FND-02.
- The schema and service boundaries should make later T1/T2/T3 specialization additive rather than forcing a storage rewrite.

### Claude's Discretion
[VERIFIED: .planning/phases/01-foundation-kernel/01-CONTEXT.md]
- Exact TOML section names and field naming, as long as they are clear and stable.
- Exact enum values for the three retrieval modes, as long as they clearly distinguish `lexical_only`, `embedding_only`, and `hybrid`.
- How much of the future coexistence contract is reflected in Phase 1 config, as long as Phase 1 only reserves extension seams and does not prematurely implement semantic retrieval.
- Whether the application is a single crate or a small workspace, as long as it preserves a mempal-like modular separation and a single binary entrypoint.
- Specific migration tooling choice and CLI command naming.

### Deferred Ideas
[VERIFIED: .planning/phases/01-foundation-kernel/01-CONTEXT.md]
- Concrete embedding backend integration
- lexical + semantic coexistence merge logic
- `sqlite-vec` schema and reindex flows
- lexical recall and reranking implementation
- Rig wiring and agent-search orchestration
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FND-01 | Developer can initialize a local-first Rust application with a SQLite database, schema migrations, and deterministic startup checks for retrieval dependencies. [VERIFIED: .planning/REQUIREMENTS.md] | Use `rusqlite` + `rusqlite_migration`, deterministic startup probe order, and CLI `status`/`doctor` split. [VERIFIED: cargo info rusqlite --registry crates-io][VERIFIED: cargo info rusqlite_migration --registry crates-io] |
| FND-02 | System can persist typed memory records with source, timestamp, scope, record type, truth-layer metadata, and provenance fields. [VERIFIED: .planning/REQUIREMENTS.md] | Keep a minimal base `memory_records` table with typed enums/newtypes and additive truth/provenance columns from day one. [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md][VERIFIED: .planning/phases/01-foundation-kernel/01-CONTEXT.md] |
| FND-03 | Developer can inspect system health and index status from a CLI surface without requiring an LLM. [VERIFIED: .planning/REQUIREMENTS.md] | Expose human-readable and JSON status output for DB path, schema version, migration state, retrieval mode, and embedding capability state. [CITED: /home/tongyuan/project/agent_memos/reference/mempal/README_zh.md][VERIFIED: .planning/phases/01-foundation-kernel/01-CONTEXT.md] |
</phase_requirements>

## Summary

Phase 1 应该实现一个单二进制、单 crate 的 Rust 骨架，核心只做 `config -> startup checks -> sqlite bootstrap -> cli status` 这条启动主链，并把基础 memory entity、schema version 和 future retrieval seam 锁住；不要把 `libsimple`、`sqlite-vec`、Rig 或任何 embedding 依赖拉进关键路径。这个方向同时满足本项目的本地优先约束、`mempal` 的模块纪律参考、以及 0415 文档里“检索系统负责 recall、认知系统负责 cognition”的边界。 [VERIFIED: .planning/PROJECT.md][CITED: /home/tongyuan/project/agent_memos/reference/mempal/README_zh.md][CITED: /home/tongyuan/project/agent_memos/doc/0415-00记忆认知架构.md]

新的锁定要求意味着 Phase 1 的配置面不能再用单一 `embedding.enabled` 布尔值表达未来能力，而要稳定表达三种模式：`lexical_only`、`embedding_only`、`hybrid`。推荐把“搜索组合模式”和“embedding 后端配置”拆成两个相邻但不同职责的配置块：`[retrieval] mode = ...` 负责搜索路径组合，`[embedding] backend = ...` 负责后端声明。这样 Phase 1 即使不实现 embedding，也能准确表达“禁用是正常态、embedding-only/hybrid 是保留态”，并为后续 lexical-first + embedding side-channel 共存留下清晰接口。 [VERIFIED: user request 2026-04-15][CITED: https://serde.rs/enum-representations.html][CITED: /home/tongyuan/project/agent_memos/reference/mempal/src/core/config.rs]

**Primary recommendation:** Phase 1 用 `serde + toml + directories + clap + rusqlite + rusqlite_migration` 建一个单 crate 单二进制基础内核，配置采用“三态 retrieval mode + embedding backend block”，startup/status 明确区分 `disabled`、`reserved-but-unavailable`、`ready`，embedding 只建模不实现。 [VERIFIED: cargo info serde --registry crates-io][VERIFIED: cargo info toml --registry crates-io][VERIFIED: cargo info directories --registry crates-io][VERIFIED: cargo info clap --registry crates-io][VERIFIED: cargo info rusqlite --registry crates-io][VERIFIED: cargo info rusqlite_migration --registry crates-io]

## Retrieval Mode Fit

三种模式不只是未来实现路径，也代表三种不同的检索问题类型。Phase 1 虽然还不实现真正的检索执行，但应该把这种语义直接写进 config 说明、status 文案和后续 plan 假设里。 [VERIFIED: user request 2026-04-15][VERIFIED: .planning/PROJECT.md]

| Mode | Best-fit scenarios | Why it fits | Caveats |
|------|--------------------|-------------|---------|
| `lexical_only` | 标识符、文件名、配置键、错误码、日期、来源过滤、需要强解释的 ordinary retrieval | 词法命中、范围过滤、时间/来源约束最稳定，也最容易说明“为什么被召回”。 [VERIFIED: .planning/PROJECT.md][VERIFIED: .planning/REQUIREMENTS.md] | 对近义改写、模糊语义、长文本隐式相关性的补召回较弱。 [ASSUMED] |
| `embedding_only` | 用户问题表达模糊、语料措辞不统一、更多依赖语义相似而不是词面匹配 | 适合解决“词没对上但意思接近”的召回问题。 [VERIFIED: .planning/PROJECT.md][ASSUMED] | 精确过滤、debug、可解释性和 source-sensitive precision 通常更弱，不应默默取代 lexical baseline。 [VERIFIED: .planning/PROJECT.md][ASSUMED] |
| `hybrid` | 代码/配置/日志 与 笔记/讨论/经验 同时存在；agent 跨 session 回忆；同一查询里既有精确词又有模糊意图 | lexical 保底，embedding 负责补召回或 rerank，最适合混合语料。 [VERIFIED: .planning/PROJECT.md][VERIFIED: .planning/STATE.md] | 必须提前定义合并契约，且 lexical 应继续承担主解释来源。 [VERIFIED: .planning/PROJECT.md][VERIFIED: .planning/research/STACK.md] |

**Default recommendation:** ordinary retrieval 默认应落在 `lexical_only`；`embedding_only` 是特殊意图模式；`hybrid` 用于混合语料和 agent memory recall，但要明确 lexical 仍是 primary explanation channel。 [VERIFIED: .planning/PROJECT.md][VERIFIED: .planning/STATE.md]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| TOML config loading | Core bootstrap | CLI entrypoint | Config 决定启动参数和未来扩展缝，CLI 只消费解析后的强类型配置。 [CITED: /home/tongyuan/project/agent_memos/reference/mempal/src/core/config.rs][CITED: https://serde.rs/attr-default.html] |
| SQLite bootstrap and migrations | Core bootstrap | SQLite storage | 迁移归应用启动控制，SQLite 只提供持久化与 `user_version` 状态。 [VERIFIED: cargo info rusqlite_migration --registry crates-io][CITED: /home/tongyuan/project/agent_memos/reference/mempal/src/core/db.rs] |
| Typed base memory entities | Domain model | SQLite storage | 0415 真值层要求 source/time/scope/truth/provenance 先成为一等结构，再映射到表。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md][VERIFIED: .planning/REQUIREMENTS.md] |
| Startup checks | Core bootstrap | SQLite storage | Phase 1 需要 deterministic startup checks，但不应把未来 Phase 2/4 功能绑成硬依赖。 [VERIFIED: .planning/ROADMAP.md][VERIFIED: .planning/phases/01-foundation-kernel/01-CONTEXT.md] |
| CLI status surface | CLI entrypoint | Core bootstrap | `status` 是开发者检查面，不应该拥有 schema 或 config 逻辑。 [CITED: /home/tongyuan/project/agent_memos/reference/mempal/src/main.rs] |
| Future retrieval coexistence seam | Config model | Search services | Phase 1 只锁配置/状态契约，真正的 lexical/embedding 执行留到后续 phase。 [VERIFIED: user request 2026-04-15][VERIFIED: .planning/REQUIREMENTS.md] |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard | Source |
|---------|---------|---------|--------------|--------|
| `rusqlite` | `0.39.0` | SQLite 连接、事务、查询。 | 本地优先单文件 DB 的直接标准接口；与 Phase 1 的 deterministic bootstrap 对齐。 | [VERIFIED: cargo info rusqlite --registry crates-io] |
| `rusqlite_migration` | `2.5.0` | 用 `user_version` 管理 SQLite schema migrations。 | 比手写 migration bookkeeping 更稳，且不额外引入 migration 表。 | [VERIFIED: cargo info rusqlite_migration --registry crates-io] |
| `serde` | `1.0.228` | 强类型配置与实体序列化。 | Phase 1 需要可扩展 TOML config 和 typed entities；Serde 是 Rust 主流基线。 | [VERIFIED: cargo info serde --registry crates-io] |
| `toml` | `1.1.2+spec-1.1.0` | TOML 解析/写回。 | 用户已锁定 TOML 配置格式；直接用官方 TOML crate 即可。 | [VERIFIED: cargo info toml --registry crates-io] |
| `clap` | `4.6.0` | CLI 命令和参数解析。 | `mempal` 参考和 Phase 1 `status`/`init` 需求都适合 `clap derive` 路线。 | [VERIFIED: cargo info clap --registry crates-io][CITED: /home/tongyuan/project/agent_memos/reference/mempal/src/main.rs] |
| `thiserror` | `2.0.18` | 领域错误建模。 | 启动检查、配置校验、迁移失败都需要稳定错误类型。 | [VERIFIED: cargo info thiserror --registry crates-io] |
| `anyhow` | `1.0.102` | CLI 边界错误聚合。 | `main` / subcommand 边界适合 `anyhow`，内部继续用 typed errors。 | [VERIFIED: cargo info anyhow --registry crates-io][CITED: /home/tongyuan/project/agent_memos/reference/mempal/src/main.rs] |

### Supporting

| Library | Version | Purpose | When to Use | Source |
|---------|---------|---------|-------------|--------|
| `directories` | `6.0.0` | 平台标准 config/data path 解析。 | 比手写 `HOME` 拼路径更稳，适合 `~/.agent-memos/` 默认目录。 | [VERIFIED: cargo info directories --registry crates-io] |
| `tracing-subscriber` | `0.3.23` | 启动日志与开发态诊断。 | `status`/`doctor` 之外仍需结构化日志帮助调试启动链。 | [VERIFIED: cargo info tracing-subscriber --registry crates-io] |
| `uuid` | `1.23.0` | 记录 ID 生成。 | 若 Phase 1 就要落存储对象，UUID v7 比裸字符串更稳。 | [VERIFIED: cargo info uuid --registry crates-io] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `rusqlite_migration` | 手写 `PRAGMA user_version` 迁移器 | 可行，但会把 migration 状态机和错误处理重新造一遍。 [VERIFIED: cargo info rusqlite_migration --registry crates-io] |
| `directories` | 直接仿 `mempal` 用 `HOME` 拼路径 | 更少依赖，但跨平台路径细节更脆。 [CITED: /home/tongyuan/project/agent_memos/reference/mempal/src/core/config.rs][VERIFIED: cargo info directories --registry crates-io] |
| 单 crate | 小 workspace | workspace 更适合多 interface/feature 成熟后；Phase 1 只做 foundation，单 crate 更省样板。 [CITED: /home/tongyuan/project/agent_memos/reference/mempal/README_zh.md][ASSUMED] |

**Installation:**
```bash
cargo add rusqlite@0.39 rusqlite_migration@2.5 serde@1 --features derive toml@1 clap@4 --features derive anyhow@1 thiserror@2 directories@6 tracing-subscriber@0.3 uuid@1 --features v7,serde
```

**Version verification:** Above versions were verified on 2026-04-15 with `cargo search` / `cargo info`; crates.io publish dates were not directly retrievable from this environment because the public API returned HTTP 403 to direct requests, so only versions are locked here. [VERIFIED: cargo search rusqlite --registry crates-io --limit 1][VERIFIED: cargo search clap --registry crates-io --limit 1][VERIFIED: cargo search toml --registry crates-io --limit 1][VERIFIED: cargo search serde --registry crates-io --limit 1][VERIFIED: cargo search anyhow --registry crates-io --limit 1][VERIFIED: cargo search thiserror --registry crates-io --limit 1]

## Architecture Patterns

### System Architecture Diagram

```text
CLI (`init` / `status` / `doctor`)
    ->
Config Loader (TOML -> typed config)
    ->
Startup Validator
    -> config coherence check
    -> db path resolution
    -> migration runner
    -> capability probe
    ->
SQLite Store
    -> schema version
    -> base memory tables
    ->
Status Snapshot
    -> human output
    -> JSON output

Future path reserved only:
Status Snapshot -> retrieval mode contract -> Phase 2 lexical path / future embedding path
```

### Recommended Project Structure

```text
src/
├── main.rs                  # CLI entry
├── lib.rs                   # module graph
├── core/
│   ├── config.rs            # TOML load + defaults + mode validation
│   ├── paths.rs             # config/data path resolution
│   ├── startup.rs           # bootstrap and status snapshot assembly
│   ├── error.rs             # typed domain errors
│   └── mod.rs
├── db/
│   ├── mod.rs
│   ├── connection.rs        # open connection, pragmas
│   ├── migrations.rs        # migration registration
│   └── schema.rs            # table names and helpers
├── memory/
│   ├── mod.rs
│   ├── entity.rs            # MemoryRecord + enums/newtypes
│   ├── repository.rs        # inserts/lookups for base entities
│   └── types.rs             # scope/record/truth/source enums
└── cli/
    ├── mod.rs
    ├── status.rs            # human + JSON render
    └── init.rs              # db init/bootstrap commands
```

### Pattern 1: Typed TOML With Additive Defaults
**What:** 用 `#[serde(default)]` 加强类型配置，缺失字段回落到稳定默认值，而不是散落在 CLI 或 env 里。 [CITED: https://serde.rs/attr-default.html][CITED: /home/tongyuan/project/agent_memos/reference/mempal/src/core/config.rs]
**When to use:** 从 Phase 1 起就这样做，避免后续格式迁移。 [VERIFIED: .planning/phases/01-foundation-kernel/01-CONTEXT.md]
**Example:**
```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub db_path: String,
    pub retrieval: RetrievalConfig,
    pub embedding: EmbeddingConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct RetrievalConfig {
    pub mode: RetrievalMode,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalMode {
    LexicalOnly,
    EmbeddingOnly,
    Hybrid,
}
```
// Source: https://serde.rs/attr-default.html and https://serde.rs/enum-representations.html

### Pattern 2: Retrieval Composition And Backend Declaration Are Separate
**What:** `retrieval.mode` 表示搜索组合模式，`embedding.backend` 表示 embedding 后端或禁用状态；两者组合后再做一致性校验。 [VERIFIED: user request 2026-04-15][CITED: /home/tongyuan/project/agent_memos/reference/mempal/README_zh.md]
**When to use:** Phase 1 即锁死这个形状；真正执行逻辑留给后续 phase。 [VERIFIED: .planning/phases/01-foundation-kernel/01-CONTEXT.md]
**Example:**
```toml
db_path = "~/.agent-memos/agent-memos.db"

[retrieval]
mode = "lexical_only" # lexical_only | embedding_only | hybrid

[embedding]
backend = "disabled"  # disabled | api | sqlite_vec | model2vec | ...
model = ""
endpoint = ""
```
// Source: user constraint + mempal config pattern

### Pattern 3: Additive Schema, Not Predictive Overfitting
**What:** Phase 1 只存 base entity 必需字段，不预埋 Phase 2 score 列或 Phase 4 cognition blobs。 [VERIFIED: .planning/phases/01-foundation-kernel/01-CONTEXT.md][CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md]
**When to use:** 所有初始 migration。 [VERIFIED: .planning/ROADMAP.md]
**Example:**
```sql
CREATE TABLE memory_records (
    id TEXT PRIMARY KEY,
    source_uri TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    recorded_at TEXT NOT NULL,
    scope TEXT NOT NULL,
    record_type TEXT NOT NULL,
    truth_layer TEXT NOT NULL,
    provenance_json TEXT NOT NULL,
    content_text TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```
-- [VERIFIED: .planning/phases/01-foundation-kernel/01-CONTEXT.md][CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md]

### Anti-Patterns to Avoid
- **把 `embedding.enabled` 当唯一控制面：** 新锁定要求已经把配置面升级为三态模式；单布尔值无法表达 `embedding_only` 与 `hybrid` 的差别。 [VERIFIED: user request 2026-04-15]
- **把 Phase 1 status 做成“disabled 就报错”：** 用户明确要求 embedding disabled 是正常状态。 [VERIFIED: .planning/phases/01-foundation-kernel/01-CONTEXT.md]
- **提前把 FTS5/libsimple、sqlite-vec、Rig 放进 foundation critical path：** 这违反当前 phase scope，也会把后续可选扩展变成启动硬依赖。 [VERIFIED: .planning/ROADMAP.md][VERIFIED: .planning/PROJECT.md]
- **把 truth/provenance 字段留到 Phase 3 再补：** FND-02 已要求这些是 first-class storage structures。 [VERIFIED: .planning/REQUIREMENTS.md]

## Phase 1 Embedding Switch Handling

推荐把旧的“embedding 开关”重定义为“retrieval 组合模式 + embedding 后端状态”的两段式模型。默认值应是 `retrieval.mode = "lexical_only"` 且 `embedding.backend = "disabled"`；这是健康状态，不是降级状态。 [VERIFIED: user request 2026-04-15][VERIFIED: .planning/phases/01-foundation-kernel/01-CONTEXT.md]

Phase 1 只需要保证以下行为：

| Case | Config meaning | Phase 1 startup behavior | `status` behavior |
|------|----------------|--------------------------|-------------------|
| `lexical_only` + `backend=disabled` | 无 embedding 模型；后续 lexical-lightweight baseline 的默认准备态。 | 启动成功。 [VERIFIED: user request 2026-04-15] | 报告 `embedding=disabled(normal)`。 [VERIFIED: .planning/phases/01-foundation-kernel/01-CONTEXT.md] |
| `embedding_only` + concrete backend | 预留未来纯 embedding 路径。 | 配置解析成功；`status` 始终成功并报告 `reserved/unimplemented` 与 `ready=false`，`doctor`/`init` 仅在命令路径要求可用 embedding runtime 时返回非零；不要静默回退到 lexical。 [VERIFIED: user request 2026-04-15][VERIFIED: .planning/phases/01-foundation-kernel/01-03-PLAN.md] | 报告 `mode=embedding_only`, `capability=reserved`, `ready=false`。 [VERIFIED: .planning/phases/01-foundation-kernel/01-03-PLAN.md] |
| `hybrid` + concrete backend | 预留 lexical-first + embedding side-channel 共存。 | 配置解析成功；`status` 始终成功并报告 lexical baseline + embedding secondary 的 `reserved/unimplemented` 状态，`doctor`/`init` 仅在命令路径要求完整 hybrid runtime 时返回非零；Phase 1 不实现融合逻辑。 [VERIFIED: user request 2026-04-15][VERIFIED: .planning/phases/01-foundation-kernel/01-03-PLAN.md] | 报告 `mode=hybrid`, `lexical_role=baseline`, `embedding_role=secondary`, `ready=false`。 [VERIFIED: .planning/PROJECT.md][VERIFIED: .planning/phases/01-foundation-kernel/01-03-PLAN.md] |

关键点不是“现在就支持 embedding”，而是“现在就把未来组合语义建对”。建议在状态结构里同时保留 `configured_mode`、`effective_mode`、`embedding_backend`、`capability_state` 四个字段；这能避免未来把配置语义、编译能力和运行时 readiness 混成一个布尔值。 [VERIFIED: user request 2026-04-15][ASSUMED]

如果 planner 要进一步预留配置扩展位，最值得保留的是 `hybrid` 相关契约，而不是过早引入具体向量实现：
- `retrieval.hybrid_strategy` - 例如 `expansion` 或 `rerank`
- `retrieval.lexical_weight` / `retrieval.embedding_weight` - 仅作为未来字段预留，不要求 Phase 1 真正生效
- `status.lexical_role` / `status.embedding_role` - 明确谁是 baseline，谁是 secondary path
[VERIFIED: .planning/PROJECT.md][VERIFIED: .planning/research/STACK.md][ASSUMED]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SQLite migration bookkeeping | 自写 migration version 状态机 | `rusqlite_migration` | 该 crate 已明确围绕 `user_version` 提供 rusqlite migration 支持。 [VERIFIED: cargo info rusqlite_migration --registry crates-io] |
| CLI parsing | 手写 `std::env::args()` 分发 | `clap derive` | `mempal` 已证明这条路径适合单二进制工具。 [CITED: /home/tongyuan/project/agent_memos/reference/mempal/src/main.rs][VERIFIED: cargo info clap --registry crates-io] |
| TOML decoding | 自己 parse 字符串或 map | `serde` + `toml` | 用户已锁定 TOML；Serde/TOML 是 Rust 标准组合。 [VERIFIED: cargo info serde --registry crates-io][VERIFIED: cargo info toml --registry crates-io] |
| 平台路径选择 | 手拼 `HOME`/`AppData` 分支 | `directories` | 该 crate 已封装 Linux/macOS/Windows 标准目录逻辑。 [VERIFIED: cargo info directories --registry crates-io] |

**Key insight:** Phase 1 最容易浪费时间的地方不是业务逻辑，而是把成熟的启动/配置/迁移基础设施重新造一遍。 [CITED: /home/tongyuan/project/agent_memos/reference/mempal/README_zh.md][VERIFIED: cargo info rusqlite_migration --registry crates-io]

## Common Pitfalls

### Pitfall 1: 把“禁用 embedding”当异常
**What goes wrong:** 默认配置无法健康启动，status 永远带 error。 [VERIFIED: .planning/phases/01-foundation-kernel/01-CONTEXT.md]  
**Why it happens:** 设计者把 embedding 看成默认能力，而不是可选 side-channel。 [VERIFIED: .planning/PROJECT.md]  
**How to avoid:** 默认模式固定为 `lexical_only + disabled`，并把它标记为 `normal`。 [VERIFIED: user request 2026-04-15]  
**Warning signs:** 日志或状态输出出现 “embedding missing” 但当前模式其实不需要 embedding。 [ASSUMED]

### Pitfall 2: 用一个布尔值压扁三种检索组合
**What goes wrong:** 后续无法清晰区分 `embedding_only` 和 `hybrid`。 [VERIFIED: user request 2026-04-15]  
**Why it happens:** 过早把“后端存在”误当成“搜索组合语义”。 [ASSUMED]  
**How to avoid:** 单独建 `retrieval.mode`，不要只留 `embedding.enabled`。 [VERIFIED: user request 2026-04-15]  
**Warning signs:** config 校验里出现 `if embedding.enabled { ... } else { ... }` 两分法。 [ASSUMED]

### Pitfall 3: 在基础表里提前塞未来 ranking/cognition 字段
**What goes wrong:** schema 很快变得臃肿，后续 phase 还得迁移清理。 [VERIFIED: .planning/phases/01-foundation-kernel/01-CONTEXT.md]  
**Why it happens:** 试图一次性预测 Phase 2-5 的所有字段。 [ASSUMED]  
**How to avoid:** 只保留 source/time/scope/record type/truth/provenance/content 这些 FND-02 必需字段。 [VERIFIED: .planning/REQUIREMENTS.md]  
**Warning signs:** foundation migration 已出现 score、embedding vector、working memory blob 等列。 [VERIFIED: .planning/ROADMAP.md]

### Pitfall 4: 对保留模式做静默回退
**What goes wrong:** 用户把 `embedding_only` 或 `hybrid` 写进配置，却被程序悄悄当成 `lexical_only`。 [VERIFIED: user request 2026-04-15]  
**Why it happens:** 试图在未实现阶段“看起来能跑”。 [ASSUMED]  
**How to avoid:** 解析成功后明确返回 `reserved/unimplemented` 状态。 [ASSUMED]  
**Warning signs:** status 不显示 `configured_mode` 和 `effective_mode` 的差别。 [ASSUMED]

## Code Examples

Verified patterns from official sources. [CITED: https://serde.rs/enum-representations.html][CITED: https://serde.rs/attr-default.html][CITED: /home/tongyuan/project/agent_memos/reference/mempal/README_zh.md]

### 推荐 TOML 形状
```toml
db_path = "~/.agent-memos/agent-memos.db"

[retrieval]
mode = "lexical_only"

[embedding]
backend = "disabled"
model = ""
endpoint = ""
```
// [VERIFIED: user request 2026-04-15][CITED: /home/tongyuan/project/agent_memos/reference/mempal/README_zh.md][CITED: https://serde.rs/enum-representations.html]

### 推荐状态结构
```rust
#[derive(Debug, Clone, Serialize)]
pub struct StartupStatus {
    pub db_path: String,
    pub schema_version: u32,
    pub migrations_clean: bool,
    pub configured_mode: RetrievalMode,
    pub effective_mode: EffectiveMode,
    pub embedding_backend: String,
    pub capability_state: CapabilityState,
}
```
// [VERIFIED: .planning/REQUIREMENTS.md][VERIFIED: user request 2026-04-15]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| 单一 `embedding.enabled` 布尔值 | `retrieval.mode = lexical_only | embedding_only | hybrid` 加独立 `embedding.backend` | 2026-04-15 user lock update | 允许 Phase 1 就把未来 coexistence contract 建对。 [VERIFIED: user request 2026-04-15] |
| 在 foundation 里预装向量依赖 | Phase 1 只保留配置/状态 seam，不把 embedding 变成 prerequisite | 项目初始化阶段已锁定 | 避免 foundation critical path 被未来可选能力绑死。 [VERIFIED: .planning/PROJECT.md][VERIFIED: .planning/ROADMAP.md] |

**Deprecated/outdated:**
- 单布尔 embedding 配置面：已不满足新的三态稳定要求。 [VERIFIED: user request 2026-04-15]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A2 | 单 crate 比小 workspace 更适合当前 Phase 1 范围。 [ASSUMED] | `## Standard Stack` | 如果后续马上要并行开发多个 crate，初始骨架可能需要再拆。 |

## Open Questions (RESOLVED)

1. **`embedding_only` / `hybrid` 在 Phase 1 的退出语义已经锁定。**
   - Resolution: `status` 是信息型命令，对 `lexical_only`、`embedding_only`、`hybrid` 三种模式都返回退出码 `0`，并显式报告 `configured_mode`、`effective_mode`、`embedding_backend`、`ready` 和解释性 notes。 [VERIFIED: .planning/phases/01-foundation-kernel/01-03-PLAN.md]
   - Resolution: `embedding_only` 和 embedding-backed `hybrid` 在 Phase 1 仍然是 `reserved/unimplemented` 的非-ready 状态，但必须被原样展示，不能静默降级成 `lexical_only`。 [VERIFIED: .planning/phases/01-foundation-kernel/01-03-PLAN.md]
   - Resolution: `doctor` 是阻塞型诊断命令，`init` 仅在命令路径请求了无效或不可能的 runtime 组合时返回非零；普通 `status`/检查路径不因 reserved mode 而失败。 [VERIFIED: .planning/phases/01-foundation-kernel/01-03-PLAN.md]
   - Traceability: This matches the Phase 1 embedding-handling matrix above and Plan `01-03` Task 1 / Task 2.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `rustc` | 编译 Phase 1 Rust skeleton | ✓ | `1.94.1` | — [VERIFIED: `rustc --version`] |
| `cargo` | 依赖管理、测试、运行 | ✓ | `1.94.1` | — [VERIFIED: `cargo --version`] |
| `sqlite3` CLI | 手工排查数据库 | ✗ | — | 用 `rusqlite` + `bundled` feature 完成 code-path 验证。 [VERIFIED: `command -v sqlite3`][CITED: /home/tongyuan/project/agent_memos/reference/mempal/Cargo.toml] |

**Missing dependencies with no fallback:**
- None. [VERIFIED: `rustc --version`][VERIFIED: `cargo --version`]

**Missing dependencies with fallback:**
- `sqlite3` CLI 缺失，但不阻塞 Phase 1，因为应用内 SQLite bootstrap 就是主路径。 [VERIFIED: `command -v sqlite3`][VERIFIED: cargo info rusqlite --registry crates-io]

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | None yet; root repo currently has no `Cargo.toml`, `src/`, or `tests/` files. [VERIFIED: `rg --files -g 'Cargo.toml' -g 'src/**' -g 'tests/**'`] |
| Config file | none — see Wave 0. [VERIFIED: `rg --files -g 'Cargo.toml' -g 'src/**' -g 'tests/**'`] |
| Quick run command | `cargo test -q` after Phase 1 creates the crate. [ASSUMED] |
| Full suite command | `cargo test` after Phase 1 creates the crate. [ASSUMED] |

### Phase Requirements → Test Map
All mappings below come from `FND-01` to `FND-03` plus the current absence of Rust test files in repo root. [VERIFIED: .planning/REQUIREMENTS.md][VERIFIED: `rg --files -g 'Cargo.toml' -g 'src/**' -g 'tests/**'`]

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FND-01 | config + DB bootstrap + migration idempotence | integration | `cargo test db_bootstrap_is_idempotent -q` | ❌ Wave 0 |
| FND-02 | base memory entity round-trip persistence | integration | `cargo test memory_record_roundtrip -q` | ❌ Wave 0 |
| FND-03 | `status` 输出正确反映 disabled / reserved / ready 状态 | integration | `cargo test status_reports_mode_matrix -q` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -q` [ASSUMED]
- **Per wave merge:** `cargo test` [ASSUMED]
- **Phase gate:** `cargo test && cargo clippy --all-targets -- -D warnings` [ASSUMED]

### Wave 0 Gaps
- [ ] root `Cargo.toml` — Phase 1 crate bootstrap still absent. [VERIFIED: `rg --files -g 'Cargo.toml' -g 'src/**' -g 'tests/**'`] 
- [ ] `src/main.rs` and `src/lib.rs` — needed for CLI and library split. [VERIFIED: `rg --files -g 'Cargo.toml' -g 'src/**' -g 'tests/**'`] 
- [ ] `tests/bootstrap.rs` — cover migration idempotence and startup checks. [ASSUMED]
- [ ] `tests/status.rs` — cover mode matrix and CLI JSON output. [ASSUMED]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | No auth surface in Phase 1 CLI-only scope. [VERIFIED: .planning/ROADMAP.md] |
| V3 Session Management | no | No session surface in Phase 1. [VERIFIED: .planning/ROADMAP.md] |
| V4 Access Control | no | Local CLI only; no multi-user service boundary yet. [VERIFIED: .planning/PROJECT.md] |
| V5 Input Validation | yes | Validate config enum combinations and use typed clap arguments. [VERIFIED: cargo info clap --registry crates-io][CITED: https://serde.rs/enum-representations.html] |
| V6 Cryptography | no | Phase 1 does not introduce crypto or secret storage. [VERIFIED: .planning/ROADMAP.md] |

### Known Threat Patterns for Phase 1 Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Config path confusion / wrong file selection | Tampering | Resolve config/data paths in one module and print resolved path in `status`. [VERIFIED: cargo info directories --registry crates-io][CITED: /home/tongyuan/project/agent_memos/reference/mempal/src/core/config.rs] |
| Silent config downgrade from `hybrid` to default mode | Tampering | Fail or mark non-ready on unsupported mode/backend combos; never silently coerce. [VERIFIED: user request 2026-04-15][ASSUMED] |
| SQL injection via future ad-hoc queries | Tampering | Keep inserts/selects parameterized through `rusqlite`. [CITED: /home/tongyuan/project/agent_memos/reference/mempal/src/core/db.rs][VERIFIED: cargo info rusqlite --registry crates-io] |

## Sources

### Primary (HIGH confidence)
- `.planning/phases/01-foundation-kernel/01-CONTEXT.md` - locked scope, TOML rule, disabled-is-normal rule, deferred ideas.
- `.planning/PROJECT.md` - project constraints, local-first and lexical-first coexistence rule.
- `.planning/ROADMAP.md` - Phase 1 goal, success criteria, plan slots.
- `.planning/REQUIREMENTS.md` - `FND-01` to `FND-03`.
- [`doc/0415-00记忆认知架构.md`](/home/tongyuan/project/agent_memos/doc/0415-00记忆认知架构.md) - retrieval vs cognition boundary.
- [`doc/0415-真值层.md`](/home/tongyuan/project/agent_memos/doc/0415-真值层.md) - T1/T2/T3 layering and provenance expectations.
- [`reference/mempal/README_zh.md`](/home/tongyuan/project/agent_memos/reference/mempal/README_zh.md) - single binary, TOML config pattern, status command precedent.
- [`reference/mempal/src/core/config.rs`](/home/tongyuan/project/agent_memos/reference/mempal/src/core/config.rs) - `serde(default)` TOML load pattern.
- [`reference/mempal/src/core/db.rs`](/home/tongyuan/project/agent_memos/reference/mempal/src/core/db.rs) - DB bootstrap and migration entry precedent.
- [`reference/mempal/src/main.rs`](/home/tongyuan/project/agent_memos/reference/mempal/src/main.rs) - CLI command organization precedent.
- `cargo info rusqlite --registry crates-io`
- `cargo info rusqlite_migration --registry crates-io`
- `cargo info serde --registry crates-io`
- `cargo info toml --registry crates-io`
- `cargo info clap --registry crates-io`
- `cargo info anyhow --registry crates-io`
- `cargo info thiserror --registry crates-io`
- `cargo info directories --registry crates-io`
- `cargo info tracing-subscriber --registry crates-io`
- `cargo info uuid --registry crates-io`

### Secondary (MEDIUM confidence)
- https://serde.rs/enum-representations.html - enum representation options for typed config modes.
- https://serde.rs/attr-default.html - default-field handling for additive config evolution.

### Tertiary (LOW confidence)
- None.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all recommended crates were verified against current cargo registry metadata, and each maps directly to Phase 1 scope. [VERIFIED: cargo info rusqlite --registry crates-io][VERIFIED: cargo info rusqlite_migration --registry crates-io][VERIFIED: cargo info serde --registry crates-io][VERIFIED: cargo info toml --registry crates-io][VERIFIED: cargo info clap --registry crates-io]
- Architecture: HIGH - driven by locked scope docs plus `mempal` reference module boundaries. [VERIFIED: .planning/ROADMAP.md][CITED: /home/tongyuan/project/agent_memos/reference/mempal/README_zh.md]
- Pitfalls: MEDIUM - most come from explicit project constraints, but some Phase 1 unsupported-mode handling still needs a plan-time decision. [VERIFIED: user request 2026-04-15][ASSUMED]

**Research date:** 2026-04-15  
**Valid until:** 2026-05-15 for stack/version checks; revisit sooner if the user changes retrieval-mode requirements again. [VERIFIED: user request 2026-04-15]

## RESEARCH COMPLETE
