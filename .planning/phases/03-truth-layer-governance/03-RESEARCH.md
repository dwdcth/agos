# Phase 3: Truth Layer Governance - Research

**Researched:** 2026-04-15  
**Domain:** 真值层治理、SQLite 增量 schema、检索兼容的治理建模  
**Confidence:** MEDIUM

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Phase 3 must distinguish T1, T2, and T3 in storage and service APIs instead of treating truth-layer as one undifferentiated tag.
- T3 is a private working-hypothesis layer and must preserve explicit provenance, confidence, and revocability fields.
- T3 must never directly overwrite shared truth; promotion toward T2 requires a governed gate with evidence-review and approval state.
- T2 -> T1 must be represented as proposal/candidate handling, not automatic ontology mutation.
- Query semantics must allow callers to filter and interpret records differently by truth layer.
- Existing Phase 2 retrieval and citation behavior must continue to work on top of the new truth metadata, not be broken by this refactor.
- The lexical-first retrieval baseline remains unchanged; Phase 3 should reuse it rather than re-implement it.
- Phase 3 should prepare governance seams that later metacognition/rumination can call, but should not prematurely implement Phase 4/5 behavior.

### the agent's Discretion
- Exact module split for truth-layer metadata, repositories, and promotion-gate services.
- Whether truth governance lives primarily under `memory/`, a new `truth/` module, or a small hybrid split, as long as the boundaries stay clear.
- Exact shape of proposal/candidate records for T2 -> T1, as long as they remain explicit non-automatic proposals.
- Whether promotion reviews are modeled as structured JSON payloads or dedicated typed records, as long as evidence and approval state remain queryable and auditable.

### Deferred Ideas (OUT OF SCOPE)
- Rig-based agent search orchestration
- working-memory assembly and metacognitive checks
- automated rumination or write-back loops
- semantic retrieval execution
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TRU-01 | System distinguishes T1, T2, and T3 records in storage and service APIs instead of treating all memory as one undifferentiated blob. | 保留 `memory_records` authority row，同时增加 layer-specific side tables 与 `TruthRecord`/`TruthService` 投影，做到存储与 API 双重区分。 |
| TRU-02 | T3 records carry explicit provenance, confidence, and revocability markers so private working hypotheses remain auditable. | 为 T3 增加专门的 `truth_t3_state` 表，保存 `confidence`、`revocation_state`、`revoked_at`、`revocation_reason`、`shared_conflict_note`。 |
| TRU-03 | System can promote a T3 structure toward T2 only through an explicit gate that records evidence review and metacognitive approval state. | 用 `truth_promotion_reviews` + `truth_promotion_evidence` 建模四段 gate，不允许 repository 直接把 T3 改写成 T2。 |
| TRU-04 | System can create T2-to-T1 ontology candidates without automatically rewriting the shared ontology layer. | 用 `truth_ontology_candidates` 表示 proposal/candidate，只记录候选和结构化审查状态，不做 T1 自动落库。 |
</phase_requirements>

## Summary

当前代码已经把 `truth_layer` 作为 `memory_records` 的基础列，并且 Phase 2 的 FTS sidecar、SQL 过滤、citation 组装、CLI `search` 都直接依赖这张 authority table；因此 Phase 3 最稳的方向不是拆出三张主内容表，而是保留 `memory_records` / `memory_records_fts` 主干不变，围绕它新增治理 side tables 和 layer-aware service。 [VERIFIED: codebase] [CITED: /home/tongyuan/project/agent_memos/.planning/phases/01-foundation-kernel/01-02-SUMMARY.md] [CITED: /home/tongyuan/project/agent_memos/.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md]

`doc/0415-真值层.md` 明确要求 T3 是“私人工作假设层”，必须可追溯、可撤销，并且 T3 -> T2 要经过“结果触发、证据复核、共识校验、元认知放行”四步；同时 T2 -> T1 只能是候选，不应直接改写本体层。Phase 3 应把这两个跃迁建成显式状态机和审查记录，而不是布尔字段或隐式 update。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md] [CITED: /home/tongyuan/project/agent_memos/doc/0415-元认知层.md]

**Primary recommendation:** 保持 `memory_records` 为唯一内容 authority，新增 `truth/` 治理模块与 3 到 4 张治理表，所有 T3 -> T2 / T2 -> T1 跃迁都通过显式 service gate 执行，默认不改变 Phase 2 的检索排序、citation 和 truth-layer filter 语义。 [VERIFIED: codebase] [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| T1/T2/T3 内容持久化 | Database / Storage | API / Backend | 记录本体仍由 SQLite authority row 持久化，服务层只做 typed 投影与校验。 [VERIFIED: codebase] |
| T3 revocability / provenance / confidence | Database / Storage | API / Backend | 审计字段必须先被持久化，服务层才能暴露可查询的治理语义。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md] |
| T3 -> T2 promotion gate | API / Backend | Database / Storage | gate 需要状态机、前置校验和副作用编排，不能下沉成数据库触发器。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md] |
| T2 -> T1 candidate / proposal queue | API / Backend | Database / Storage | 候选生成与结构审查属于治理服务；数据库负责保留候选记录与依据。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md] |
| Ordinary retrieval compatibility | API / Backend | Database / Storage | 现有 recall/rerank/citation 管线在服务层组装，但依赖 `memory_records` 与 FTS sidecar 不变。 [VERIFIED: codebase] [CITED: https://www.sqlite.org/fts5.html] |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| SQLite + `rusqlite` | `0.37.x` project pin; `0.37.0` latest docs.rs page verified | 保持 authority store、查询和 migration 执行面。 [VERIFIED: Cargo.toml] [VERIFIED: https://docs.rs/crate/rusqlite/latest] | 当前所有 record/query/citation 路径都已经建立在 `Connection` + 参数化 SQL 上，本阶段不应改库。 [VERIFIED: codebase] |
| `rusqlite_migration` | `2.3.x` project pin; `2.5.0` latest docs.rs page verified | 继续做 additive migration。 [VERIFIED: Cargo.toml] [VERIFIED: https://docs.rs/crate/rusqlite_migration/latest] | 现有 schema 已通过 `Migrations::new(...).to_latest()` 串联，Phase 3 适合追加 migration，而不是重写 bootstrap。 [VERIFIED: codebase] |
| `serde` / `serde_json` | `1.x`; `serde_json 1.0.149` latest docs.rs page verified | 持久化 evidence summary、proposal payload、审查摘要。 [VERIFIED: Cargo.toml] [VERIFIED: https://docs.rs/crate/serde_json/latest] | 当前 provenance 已以 JSON 文本持久化，治理摘要沿用同一路径最小。 [VERIFIED: codebase] |
| Existing FTS5 + `libsimple` sidecar | `libsimple ~0.9` project pin | 保持 ordinary retrieval baseline。 [VERIFIED: Cargo.toml] [VERIFIED: https://docs.rs/crate/libsimple/latest] | Phase 2 已把 `memory_records_fts` 作为 external-content sidecar 建好，本阶段不要改 tokenizer、排名或 sidecar 结构。 [VERIFIED: codebase] [CITED: https://www.sqlite.org/fts5.html] |
| `thiserror` / `anyhow` | `2.x / 1.x` | 定义 repository/service 边界错误。 [VERIFIED: Cargo.toml] | 当前 `db`、`repository`、`search` 都已经使用 typed error，本阶段应继续沿用。 [VERIFIED: codebase] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tracing` | `0.1.x` project pin | 记录 promotion decision、review transition、candidate lifecycle。 [VERIFIED: Cargo.toml] | 在 service 层写治理事件 trace，不把日志逻辑塞进 repository。 |
| `clap` | `4.x` project pin | 如需 Phase 3 inspection CLI，可复用现有命令面。 [VERIFIED: Cargo.toml] | 只用于 inspection/status，不要扩成人工审批工作流 UI。 [VERIFIED: codebase] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `memory_records` authority row + side tables | 为 T1/T2/T3 各建一张主内容表 | 三表会直接冲击 Phase 2 的 FTS sidecar、citation 和 `SearchService`，迁移成本高且回归面大。 [VERIFIED: codebase] |
| dedicated governance tables | 单个 `governance_json` 大字段 | JSON blob 写起来快，但 evidence/review/candidate state 不可索引、不可审计、难以写 SQL 队列查询。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md] |
| explicit service gate | repository 里直接 `UPDATE truth_layer='t2'` | 这样无法表达四段 gate，也会把治理副作用藏进 CRUD。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md] [VERIFIED: codebase] |

**Installation:**  
Phase 3 不建议新增 crate；优先复用当前依赖并新增 migration / domain module。 [VERIFIED: Cargo.toml]

**Version verification:** 项目当前依赖 pin 已在 `Cargo.toml` 验证；`rusqlite`、`rusqlite_migration`、`serde_json`、`libsimple` 的 latest docs.rs crate pages 已在本次 research 中核对。 [VERIFIED: Cargo.toml] [VERIFIED: https://docs.rs/crate/rusqlite/latest] [VERIFIED: https://docs.rs/crate/rusqlite_migration/latest] [VERIFIED: https://docs.rs/crate/serde_json/latest] [VERIFIED: https://docs.rs/crate/libsimple/latest]

## Architecture Patterns

### System Architecture Diagram

```text
CLI / Library API
    ↓
SearchService --------------------------┐
    ↓                                   │
memory_records_fts → memory_records     │
    ↓                                   │
Citation / SearchResponse               │
                                        │
TruthGovernanceService -----------------┤
    ├─ load base record via MemoryRepository
    ├─ load layer-specific state via TruthRepository
    ├─ validate promotion / candidate gate
    └─ emit new T2 record or T1 candidate only through explicit commands
                                        │
SQLite
    ├─ memory_records
    ├─ memory_records_fts
    ├─ truth_t3_state
    ├─ truth_promotion_reviews
    ├─ truth_promotion_evidence
    └─ truth_ontology_candidates
```

图里最重要的边界是：ordinary retrieval 继续只依赖 `memory_records` + FTS sidecar，truth governance 通过额外表和 service 协调，不反向污染 recall pipeline。 [VERIFIED: codebase] [CITED: https://www.sqlite.org/fts5.html]

### Recommended Project Structure

```text
src/
├── memory/
│   ├── record.rs          # 基础 authority record，不承载 gate 状态机
│   ├── repository.rs      # 基础 CRUD / row mapping
│   └── mod.rs
├── truth/
│   ├── model.rs           # T3 state, promotion review, ontology candidate
│   ├── repository.rs      # truth_* tables
│   ├── service.rs         # gate rules and typed layer-aware APIs
│   ├── promotion.rs       # T3 -> T2 command / validation
│   ├── candidate.rs       # T2 -> T1 candidate command / validation
│   └── mod.rs
├── search/
│   ├── lexical.rs         # 保持 recall SQL 不变，必要时只加可选 governance filter
│   ├── rerank.rs
│   ├── citation.rs
│   └── mod.rs
└── interfaces/
    └── cli.rs             # 可选新增 inspect/governance 命令
```

推荐 hybrid split：`memory/` 保持 authority row，`truth/` 独立承载治理状态和 gate 服务。这样能避免 `memory/repository.rs` 演化成 god object，同时不迫使 Phase 2 的 `search` 改成跨模块大重写。 [VERIFIED: codebase] [CITED: /home/tongyuan/project/agent_memos/.planning/research/ARCHITECTURE.md]

### Pattern 1: Authority Row + Governance Side Tables
**What:** `memory_records` 继续保存内容、来源、时间、truth_layer；layer-specific 治理状态进入 side tables。 [VERIFIED: codebase]  
**When to use:** 需要在不破坏 Phase 2 FTS/citation 的前提下引入 T1/T2/T3 语义时。  
**Example:**

```rust
// Source: /home/tongyuan/project/agent_memos/src/memory/record.rs
pub struct MemoryRecord {
    pub id: String,
    pub truth_layer: TruthLayer,
    pub provenance: Provenance,
    pub content_text: String,
    pub chunk: Option<ChunkMetadata>,
    pub validity: ValidityWindow,
}
```

### Pattern 2: Promotion Gate As Typed State Machine
**What:** T3 -> T2 用显式 review row 表达四段状态，而不是 `approved: bool`。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md]  
**When to use:** 任何需要审查 evidence、consensus、metacog approval 的共享晋升。  
**Example:**

```rust
// Recommended Phase 3 shape derived from /doc/0415-真值层.md
enum ReviewState { Pending, Passed, Rejected }

struct PromotionReview {
    source_record_id: String,
    result_trigger: ReviewState,
    evidence_review: ReviewState,
    consensus_check: ReviewState,
    metacog_approval: ReviewState,
    decision_state: PromotionDecisionState,
}
```

### Pattern 3: Candidate-First T2 -> T1
**What:** T2 只能生成 ontology candidate，不直接改写 T1。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md]  
**When to use:** 任何“共享事实可能变成世界骨架”的场景。  
**Example:**

```rust
// Recommended Phase 3 shape derived from /doc/0415-真值层.md
struct OntologyCandidate {
    candidate_id: String,
    basis_record_ids: Vec<String>,
    structural_review_state: ReviewState,
    candidate_state: CandidateState,
}
```

### Anti-Patterns to Avoid
- **三张主内容表重写检索主线：** 会迫使 FTS sidecar、citation、row mapping 和 Phase 2 tests 同时重写，回归面过大。 [VERIFIED: codebase]
- **把 gate 压成布尔字段：** 无法表达“结果触发 / 证据复核 / 共识校验 / 元认知放行”四步。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md]
- **用删除代替 T3 撤销：** 删除会破坏 citation 和审计链；应保留 record 并标记 revocation state。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md] [VERIFIED: codebase]
- **让 `search::lexical` 提前 join 所有治理表：** 这会把 ordinary retrieval 和治理查询耦死，并改变排名风险面。 [VERIFIED: codebase]

## Recommended Schema / Model Direction

### Base Rule

保留 `memory_records` 作为唯一内容 authority，并继续使用已有 `truth_layer` 枚举列表示“当前记录属于 T1/T2/T3 哪一层”；Phase 3 不引入第二张内容 authority table。 [VERIFIED: codebase]

### Table 1: `truth_t3_state`

只服务 T3 记录，按 `record_id` 一对一挂载。建议字段：`record_id`、`confidence_score`、`revocation_state`、`revoked_at`、`revocation_reason`、`shared_conflict_note`、`last_reviewed_at`。这样 T3 的“confidence / provenance / revocability”就从标签升级为可查询状态。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md]

### Table 2: `truth_promotion_reviews`

表示一次 T3 -> T2 promotion attempt，而不是最终共享事实本身。建议字段：`review_id`、`source_record_id`、`target_layer='t2'`、`result_trigger_state`、`evidence_review_state`、`consensus_check_state`、`metacog_approval_state`、`decision_state`、`review_notes_json`、`created_at`、`updated_at`、`approved_at`。四段状态都通过 `CHECK` + Rust enum 限制在小集合中。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md] [VERIFIED: codebase]

### Table 3: `truth_promotion_evidence`

用来把 promotion review 和支持证据解耦。建议字段：`review_id`、`evidence_record_id`、`evidence_role`、`evidence_note_json`。如果后续要接外部证据，可加 `external_ref_json`，但本 phase 先以内部 `memory_records.id` 为主。 [VERIFIED: codebase] [ASSUMED]

### Table 4: `truth_ontology_candidates`

表示 T2 -> T1 候选，不表示 T1 已经变更。建议字段：`candidate_id`、`basis_record_ids_json`、`proposed_structure_json`、`time_stability_state`、`agent_reproducibility_state`、`context_invariance_state`、`predictive_utility_state`、`structural_review_state`、`candidate_state`、`created_at`、`updated_at`、`decided_at`。这直接对应文档中的五条铁律。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md]

### Migration Strategy

Phase 3 migration 应保持 additive：新增表、索引和外键，不改动 `memory_records_fts` 定义，不批量改写现有 `memory_records` 文本列。对既有 T3 记录，缺省 `truth_t3_state` 可在 migration 中按 `truth_layer='t3'` 批量插入默认行，或在首次治理读写时 lazy create；前者更利于审计和 SQL 队列查询。 [VERIFIED: codebase] [CITED: https://www.sqlite.org/fts5.html]

## Repository / Service Split

| Boundary | Owns | Must Not Own |
|---------|------|---------------|
| `MemoryRepository` | `memory_records` CRUD、基础 row mapping、authority row 查询。 [VERIFIED: codebase] | promotion gate、candidate state machine、审批规则。 |
| `TruthRepository` | `truth_t3_state`、`truth_promotion_reviews`、`truth_promotion_evidence`、`truth_ontology_candidates`。 | FTS recall、citation 组装、CLI 文本渲染。 |
| `TruthGovernanceService` | layer-aware 读模型、promotion validation、candidate creation、derived T2 record creation。 | 低层 SQL string 细节、tokenizer/score 算法。 |
| `SearchService` | ordinary retrieval、score breakdown、citation、现有 filters。 [VERIFIED: codebase] | 直接审批 promotion 或创建 ontology candidate。 |

**Recommended read model:**  
- `TruthRecord::T1 { base }`  
- `TruthRecord::T2 { base, open_candidates }`  
- `TruthRecord::T3 { base, t3_state, open_reviews }`  
这个读模型让 service API 真正区分三层，但不迫使底层拆三张主表。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md] [ASSUMED]

## Promotion / Candidate Modeling

### T3 -> T2 Promotion

推荐把 promotion 建模为“review 驱动的新共享记录生成”，而不是“源记录 truth_layer 原地改写”。原因有三点：一是 T3 文档要求可撤销和可追溯；二是原地改写会丢掉“曾经是私有假设”的历史；三是 Phase 2 citation 直接引用 `record.id`，新建 T2 row 更容易保留来源链。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md] [VERIFIED: codebase]

**Recommended flow:**
1. `create_promotion_review(source_t3_record_id)` 创建 review 草稿。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md]
2. `attach_evidence(review_id, evidence_record_id)` 追加证据引用。 [VERIFIED: codebase] [ASSUMED]
3. `record_review_state(...)` 分别更新四个 gate 状态。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md]
4. `approve_promotion(review_id)` 仅在四项都 `passed` 时创建新的 T2 `memory_record`，并在其 provenance 中写入 `derived_from=[source_t3_record_id, ...evidence ids]`。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md] [VERIFIED: codebase]

### T2 -> T1 Candidate

推荐把 T2 -> T1 做成“候选描述 + 审查状态”，而不是 Phase 3 就创建单独 ontology schema。T1 在文档中是“世界骨架层”，变更成本高、需要人工显式化决策；现阶段只要能稳定地产出候选队列，后续阶段再决定是否引入专门 ontology tables。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md] [ASSUMED]

## Query Semantics And Retrieval Compatibility

### Compatibility Rule

Phase 2 当前的检索 contract 是：`SearchFilters` 支持 `scope`、`record_type`、`truth_layer`、`valid_at`、`recorded_at` 过滤，`SearchService` 返回 `SearchResponse`，每条结果带 citation 和 filter trace。Phase 3 默认必须保留这条 contract。 [VERIFIED: codebase] [CITED: /home/tongyuan/project/agent_memos/.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md]

### Recommended Additive Filters

只新增治理感知 filters，不修改现有字段含义：  
- `revocation_state: Option<RevocationState>`  
- `promotion_state: Option<PromotionDecisionState>`  
- `candidate_state: Option<CandidateState>`  
- `include_governance_summary: bool`  
这些都应是 additive，而不是替换 `truth_layer`。 [VERIFIED: codebase] [ASSUMED]

### Ranking Rule

ordinary retrieval 的 recall SQL 仍应以 `memory_records_fts JOIN memory_records` 为主，治理表只在 caller 明确指定治理过滤时才参与附加查询；不要把 promotion/candidate 状态 join 进 FTS 主查询并改变 candidate set。SQLite FTS5 external-content table 本来就假定内容表与 FTS sidecar 通过触发器或 rebuild 同步，Phase 3 若不改内容表文本列，就没有理由重写这部分主查询。 [VERIFIED: codebase] [CITED: https://www.sqlite.org/fts5.html]

### Citation Rule

`Citation::from_record` 目前完全由 persisted chunk metadata 构造 citation；只要 Phase 3 不删除或覆盖原始 `memory_record`，citation 逻辑就可以原样复用。 [VERIFIED: codebase]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Promotion gating | `approved: bool` 之类的单字段状态 | 四段 gate + 总决策状态的 typed state machine | 文档明确要求四步审查，单布尔丢失审计和拒绝原因。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md] |
| Retrieval rewrite | 新的治理专用全文索引 | 继续用已有 `memory_records_fts` sidecar | Phase 2 已稳定，重写 recall 不值当且会破坏 explainability。 [VERIFIED: codebase] |
| Citation rebuilding | 从 snippet 或 query 临时猜 citation | 继续用 `Citation::from_record` 和 persisted chunk metadata | 现有 citation 已明确要求由持久化元数据生成。 [VERIFIED: codebase] |
| Governance storage | 一个不可索引的大 JSON blob | typed table + 小 JSON summary 字段 | 队列、审核、revocation 都需要 SQL 查询与索引。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md] |

**Key insight:** 这个 phase 不需要“更聪明的检索”，需要的是“更硬的治理边界”。检索层复用，治理层显式化。 [VERIFIED: codebase] [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md]

## Runtime State Inventory

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | 现有 SQLite schema version 3 已包含 `memory_records` 与 `memory_records_fts`，并且 ingest/search/test 都会写入不同 `truth_layer` 的 authority rows。 [VERIFIED: codebase] | 新 migration 需要为已有 T3 rows 回填默认治理状态，或在首次读取时 lazy-create；推荐 migration 回填。 |
| Live service config | None — 仓库当前只有本地 CLI / library surface，没有外部治理服务配置。 [VERIFIED: codebase] [CITED: /home/tongyuan/project/agent_memos/.planning/PROJECT.md] | 无。 |
| OS-registered state | None — 未发现 systemd/launchd/pm2/task-scheduler 之类注册状态。 [VERIFIED: codebase] | 无。 |
| Secrets/env vars | None — 当前项目配置来自 TOML 文件与 CLI 参数，未发现 Phase 3 相关 secret/env gate。 [VERIFIED: codebase] | 无。 |
| Build artifacts | None — 编译产物不承载 truth governance 状态；真正需要迁移的是用户 SQLite 数据文件。 [VERIFIED: codebase] | 无；只需保证应用启动会运行 migration。 |

## Common Pitfalls

### Pitfall 1: 用“原地升层”代替派生新记录
**What goes wrong:** 直接把 T3 row 的 `truth_layer` 改成 `t2`。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md]  
**Why it happens:** 这样看起来最省表、最省代码。  
**How to avoid:** 把 promotion 视为审查通过后生成新的 T2 row，并在 provenance / review 表里回链。  
**Warning signs:** 出现 `UPDATE memory_records SET truth_layer='t2' ...` 之类逻辑。

### Pitfall 2: 把治理状态塞进 `memory_records`
**What goes wrong:** `memory_records` 出现大量只对 T3 或 candidate 有意义的 nullable 列。  
**Why it happens:** 想少建表。  
**How to avoid:** 基础 authority row 只保留跨层通用字段，T3 / review / candidate 放 side tables。  
**Warning signs:** 新增列开始包含 `metacog_*`、`candidate_*`、`revoked_*` 等十几个 nullable 字段。 [VERIFIED: codebase]

### Pitfall 3: 检索主查询过早 join 治理表
**What goes wrong:** recall 性能和结果稳定性都受影响，Phase 2 tests 会回归。  
**Why it happens:** 想“一次 SQL 全查完”。  
**How to avoid:** 先保持 candidate recall 不变，再按需要补充治理摘要或附加过滤。  
**Warning signs:** `src/search/lexical.rs` 的 FTS SQL 开始 join 多张 `truth_*` 表。 [VERIFIED: codebase]

### Pitfall 4: 只记录“最终批准”，不记录拒绝和待审
**What goes wrong:** 审计链缺口，后续 metacognition / rumination 无法复用。  
**Why it happens:** 早期实现只盯 happy path。  
**How to avoid:** 所有 gate 都必须显式支持 `pending / passed / rejected`。  
**Warning signs:** review 表只有 `approved_at`，没有 per-step state。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md]

### Pitfall 5: Row mapping 重复扩散
**What goes wrong:** 目前 `memory::repository` 与 `search::lexical` 都各自维护一套 `map_record_row`；Phase 3 若再在两个地方手工加治理字段，维护成本会翻倍。 [VERIFIED: codebase]  
**Why it happens:** 现阶段代码量还小，重复容易被忽略。  
**How to avoid:** 在 Phase 3 顺手抽一个共享 row mapper 或 `MemoryRecordRow` adapter。  
**Warning signs:** 同一列序号和 enum parse 逻辑在多个模块一起修改。 [VERIFIED: codebase]

## Code Examples

Verified patterns from current codebase:

### Additive migration registration
```rust
// Source: /home/tongyuan/project/agent_memos/src/core/migrations.rs
Migrations::new(vec![
    M::up(FOUNDATION_SCHEMA_SQL).foreign_key_check(),
    M::up(INGEST_FOUNDATION_SQL).foreign_key_check(),
    M::up(LEXICAL_SIDECAR_SQL).foreign_key_check(),
])
```

### Citation derived only from persisted record metadata
```rust
// Source: /home/tongyuan/project/agent_memos/src/search/citation.rs
pub fn from_record(record: &MemoryRecord) -> Result<Self, CitationError> {
    let chunk = record.chunk.as_ref().ok_or_else(|| CitationError::MissingChunkMetadata {
        record_id: record.id.clone(),
    })?;
    Ok(Self {
        record_id: record.id.clone(),
        source_uri: record.source.uri.clone(),
        recorded_at: record.timestamp.recorded_at.clone(),
        validity: record.validity.clone(),
        anchor: CitationAnchor {
            chunk_index: chunk.chunk_index,
            chunk_count: chunk.chunk_count,
            anchor: chunk.anchor.clone(),
        },
    })
}
```

### Truth-layer filter already lives in the search contract
```rust
// Source: /home/tongyuan/project/agent_memos/src/search/filter.rs
pub struct SearchFilters {
    pub scope: Option<Scope>,
    pub record_type: Option<RecordType>,
    pub truth_layer: Option<TruthLayer>,
    pub valid_at: Option<String>,
    pub recorded_from: Option<String>,
    pub recorded_to: Option<String>,
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `truth_layer` 只是 authority row 上的标签 | `truth_layer` 保留在 base row，同时由 `truth_*` 表和 service 赋予治理语义 | 建议在 Phase 3 落地 | 兼容检索主线，同时让三层变成可操作状态。 |
| promotion 可以被实现成一次 update | promotion 是 review 驱动的新共享记录生成 | 建议在 Phase 3 落地 | 保留 T3 可撤销历史和完整审计链。 |
| T2 -> T1 可能被误做成 direct mutation | T2 -> T1 只生成 candidate/proposal | 文档已明确，Phase 3 应落实 | 为后续人工和 metacog 审查保留结构空间。 |

**Deprecated/outdated:**
- 直接把 T3 变成 T2 的 in-place update。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md]
- 把 governance 设计成“以后再加”的注释而不是实际 schema seam。 [CITED: /home/tongyuan/project/agent_memos/.planning/ROADMAP.md]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `truth_promotion_evidence` 可先以内部 `memory_records.id` 为主，外部证据用 `external_ref_json` 占位。 | Recommended Schema / Model Direction | 如果后续必须一开始就支持外部证据对象，review schema 需要再拆一层。 |
| A2 | `TruthRecord::T1/T2/T3` 读模型适合当前阶段，且无需在 Phase 3 就落独立 T1 ontology tables。 | Repository / Service Split | 如果 planner 决定 Phase 3 同时实现专门 ontology store，这个读模型需要扩展。 |
| A3 | `attach_evidence(review_id, evidence_record_id)` 作为 service API 足够覆盖本 phase 证据挂载需求。 | Promotion / Candidate Modeling | 如果 evidence 需要更复杂的批处理或版本控制，service API 需要扩展。 |
| A4 | T2 -> T1 在 Phase 3 只做 candidate queue，不需要同步落专门 ontology schema。 | Promotion / Candidate Modeling | 如果项目要求本 phase 就实现真实 ontology store，Phase 3 范围会扩大。 |
| A5 | additive governance filters 应保持默认关闭，以维持 Phase 2 默认检索行为。 | Query Semantics And Retrieval Compatibility | 如果产品决定 revoked T3 默认不可见，则需要明确一次兼容性变更。 |
| A6 | Phase 3 暂不引入“外部但未 ingest 的证据对象”作为一等数据模型。 | Open Questions (RESOLVED) | 如果需要外部证据对象，schema 需要新增引用实体。 |
| A7 | revoked T3 在 ordinary retrieval 默认保持可见，治理调用方再显式过滤。 | Open Questions (RESOLVED) / Security Domain | 如果产品要求默认隐藏，search defaults 和回归测试都要改。 |

## Open Questions (RESOLVED)

1. **promotion evidence 是否只接受内部记录引用**
   - Resolution: Phase 3 将内部 `memory_records.id` 作为 promotion evidence 的一等引用对象；若调用方需要附带尚未 ingest 的外部线索，只允许写入 `external_ref_json` 之类的补充摘要字段，不在本 phase 引入新的 external evidence entity 或独立 ingest 流程。
   - Why this is resolved: 现有 provenance、citation、search result 全都围绕 `memory_records.id` 与 `source_uri` 工作，保持内部记录引用优先可以直接复用 Phase 2 lexical retrieval 与 citation 合同，并避免把 Phase 4/5 范围提前拉进来。 [VERIFIED: codebase]
   - Planning impact: repository 和 governance service 只需要支持内部 record-based evidence 挂载，schema 可保留可选 JSON 摘要位而不扩成新子系统。

2. **revoked T3 的 ordinary retrieval 默认可见性**
   - Resolution: revoked T3 在 ordinary retrieval 中默认保持可见；治理调用方通过 additive governance-aware filters 显式排除 revoked records，而不是改写 Phase 2 的默认搜索语义。
   - Why this is resolved: Phase 2 当前只有 `truth_layer` 过滤而没有 revocation 默认策略，直接改成“默认隐藏”会改变 lexical retrieval 的既有行为与回归基线，也会削弱 audit trail 的可见性。 [VERIFIED: codebase]
   - Planning impact: Phase 3 plans必须保留 Phase 2 lexical retrieval compatibility，把 revocation 视为治理过滤能力而不是默认检索行为变更。

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | 编译、测试、clippy | ✓ | `rustc 1.94.1` [VERIFIED: rustc --version] | — |
| Cargo | build/test/clippy | ✓ | `cargo 1.94.1` [VERIFIED: cargo --version] | — |
| `sqlite3` CLI | 手工 schema 检查 | ✗ | — | 用 `rusqlite` tests 与 `inspect schema` 命令替代。 [VERIFIED: codebase] |

**Missing dependencies with no fallback:**  
None.

**Missing dependencies with fallback:**  
- `sqlite3` CLI 缺失；但项目已经用 bundled SQLite 的 `rusqlite` 跑迁移和测试，不阻塞 Phase 3。 [VERIFIED: Cargo.toml] [VERIFIED: codebase]

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` / libtest。 [VERIFIED: rustc --version] |
| Config file | `Cargo.toml`。 [VERIFIED: Cargo.toml] |
| Quick run command | `cargo test --test truth_governance -- --nocapture` |
| Full suite command | `cargo test --tests` |

当前基线：`cargo test --tests` 已通过，25 tests / 7 suites / 3.34s。 [VERIFIED: cargo test --tests]  
当前验证缺口：`cargo clippy --all-targets -- -D warnings` 仍被 `tests/ingest_pipeline.rs:12` 的既有 unused import 阻塞。 [VERIFIED: cargo clippy --all-targets -- -D warnings]

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TRU-01 | T1/T2/T3 在 storage / API 中区分为 layer-aware records | integration | `cargo test --test truth_governance distinguish_truth_records_by_layer -- --nocapture` | ❌ Wave 0 |
| TRU-02 | T3 保留 provenance / confidence / revocability | integration | `cargo test --test truth_governance t3_records_preserve_revocable_metadata -- --nocapture` | ❌ Wave 0 |
| TRU-03 | T3 -> T2 只有四段 gate 全通过才能晋升 | integration | `cargo test --test truth_governance promotion_requires_all_review_states -- --nocapture` | ❌ Wave 0 |
| TRU-04 | T2 -> T1 只创建 candidate，不直接改写 T1 | integration | `cargo test --test truth_governance t2_to_t1_creates_candidate_only -- --nocapture` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --test truth_governance -- --nocapture`
- **Per wave merge:** `cargo test --tests`
- **Phase gate:** `cargo test --tests` 绿，且新增 truth-governance tests 覆盖四个 TRU requirement

### Wave 0 Gaps

- [ ] `tests/truth_governance.rs` — 覆盖 TRU-01 ~ TRU-04 的 repository/service integration path。
- [ ] migration regression test — 断言 schema version 4/5 新表存在且不破坏 `memory_records_fts`。 
- [ ] retrieval compatibility regression — 断言 Phase 3 schema 下旧 `SearchService` 结果、citation、truth-layer filter 仍成立。
- [ ] 处理现有 `tests/ingest_pipeline.rs:12` clippy blocker，避免 Phase 3 收尾时验证被历史债务卡住。 [VERIFIED: cargo clippy --all-targets -- -D warnings]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | 当前仓库没有认证面。 [VERIFIED: codebase] |
| V3 Session Management | no | 当前仓库没有 session 面。 [VERIFIED: codebase] |
| V4 Access Control | no | Phase 3 是本地治理状态机，不引入用户/角色访问控制。 [VERIFIED: codebase] |
| V5 Input Validation | yes | 继续使用 `clap` 参数解析、typed enums、参数化 SQL。 [VERIFIED: codebase] |
| V6 Cryptography | no | 本 phase 不引入加密原语。 [VERIFIED: codebase] |

### Known Threat Patterns for This Stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| SQL injection through governance filters | Tampering | 延续 repository / search 中的参数化 SQL，禁止字符串拼接 filter。 [VERIFIED: codebase] |
| Unauthorized shared-truth promotion | Tampering / Elevation | 只有 `TruthGovernanceService` 能执行 promotion，且必须检查四段 gate。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md] |
| Provenance erasure by in-place mutation | Repudiation / Tampering | 不覆盖原 T3 row；promotion 生成新 T2 row 并保留 review / evidence chain。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md] |
| Revoked hypothesis resurfacing without context | Integrity | 给 search 增加可选 revocation filter，并在 governance read model 中显式暴露 revocation state。 [ASSUMED] |

## Explicit Anti-Goals

- 不实现 Rig、working memory、semantic retrieval 或 rumination。 [CITED: /home/tongyuan/project/agent_memos/.planning/phases/03-truth-layer-governance/03-CONTEXT.md]
- 不重写 `memory_records_fts`、不调整 lexical scoring、不过早改 Phase 2 ordinary retrieval 主线。 [VERIFIED: codebase]
- 不做自动 T3 -> T2 或自动 T2 -> T1。 [CITED: /home/tongyuan/project/agent_memos/doc/0415-真值层.md]
- 不在本 phase 引入复杂的人类审批 UI、远程 service 或角色系统。 [CITED: /home/tongyuan/project/agent_memos/.planning/PROJECT.md]

## Sources

### Primary (HIGH confidence)
- `/home/tongyuan/project/agent_memos/doc/0415-真值层.md` - T1/T2/T3 定义、T3 tuple、T3 -> T2 四段 gate、T2 -> T1 五条铁律
- `/home/tongyuan/project/agent_memos/doc/0415-00记忆认知架构.md` - ordinary retrieval 与 cognition 边界
- `/home/tongyuan/project/agent_memos/doc/0415-元认知层.md` - metacognitive approval 只是 gate seam，不是本 phase 逻辑主体
- `/home/tongyuan/project/agent_memos/src/memory/record.rs` - authority row shape 与 truth_layer 基础类型
- `/home/tongyuan/project/agent_memos/src/memory/repository.rs` - current base repository boundary
- `/home/tongyuan/project/agent_memos/src/search/filter.rs` - current retrieval filter contract
- `/home/tongyuan/project/agent_memos/src/search/lexical.rs` - current FTS recall path and SQL filters
- `/home/tongyuan/project/agent_memos/src/search/citation.rs` - citation source of truth
- `/home/tongyuan/project/agent_memos/src/core/migrations.rs` - additive migration path
- `https://www.sqlite.org/fts5.html` - FTS5 external-content tables, triggers, rebuild behavior

### Secondary (MEDIUM confidence)
- `/home/tongyuan/project/agent_memos/.planning/phases/01-foundation-kernel/01-02-SUMMARY.md`
- `/home/tongyuan/project/agent_memos/.planning/phases/02-ingest-and-lightweight-retrieval/02-01-SUMMARY.md`
- `/home/tongyuan/project/agent_memos/.planning/phases/02-ingest-and-lightweight-retrieval/02-02-SUMMARY.md`
- `/home/tongyuan/project/agent_memos/.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md`
- `/home/tongyuan/project/agent_memos/.planning/research/ARCHITECTURE.md`
- `https://docs.rs/crate/rusqlite/latest`
- `https://docs.rs/crate/rusqlite_migration/latest`
- `https://docs.rs/crate/serde_json/latest`
- `https://docs.rs/crate/libsimple/latest`
- `cargo test --tests`
- `cargo clippy --all-targets -- -D warnings`

### Tertiary (LOW confidence)
- None.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - 本 phase 基本复用现有依赖，且 crate pages 已核对。  
- Architecture: MEDIUM - 推荐的 `truth/` module 和 side-table 方案是基于当前代码与理论文档做出的实现推断。  
- Pitfalls: HIGH - 主要风险直接来自当前代码边界和 `0415-真值层` 的硬约束。

**Research date:** 2026-04-15  
**Valid until:** 2026-05-15

## RESEARCH COMPLETE
