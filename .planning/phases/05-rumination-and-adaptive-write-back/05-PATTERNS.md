# Phase 5: Rumination And Adaptive Write-back - Pattern Map

**Mapped:** 2026-04-16
**Files analyzed:** 10
**Analogs found:** 10 / 10

## Revision Notes

- Phase 5 应继续以当前仓库的 Phase 3/4 真实实现为 baseline，不要回退到 `reference/mempal` 的领域模型。
- 本 phase 的关键不是“让系统自动改写真值”，而是补一个有预算、可审计、候选优先的 learning control plane。
- 最重要的 locked rule：短周期只允许本地自适应写回；共享真值相关变化仍然只能走 candidate/proposal 路径。

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `migrations/0005_rumination_writeback.sql` | migration | transform | `migrations/0004_truth_layer_governance.sql` | exact |
| `src/core/migrations.rs` | config | batch | `src/core/migrations.rs` | exact |
| `src/cognition/mod.rs` | config | transform | `src/cognition/mod.rs` | exact |
| `src/cognition/assembly.rs` | service | transform | `src/cognition/assembly.rs` | exact |
| `src/cognition/rumination.rs` | service | event-driven | `src/agent/orchestration.rs` + `src/memory/governance.rs` | composite |
| `src/memory/repository.rs` | store | CRUD | `src/memory/repository.rs` | exact |
| `tests/foundation_schema.rs` | test | batch | `tests/foundation_schema.rs` | exact |
| `tests/rumination_queue.rs` | test | event-driven | `tests/truth_governance.rs` + `tests/agent_search.rs` | composite |
| `tests/rumination_writeback.rs` | test | event-driven | `tests/agent_search.rs` + `tests/truth_governance.rs` | composite |
| `tests/rumination_governance_integration.rs` | test | event-driven | `tests/truth_governance.rs` | exact |

## Pattern Assignments

### `migrations/0005_rumination_writeback.sql`（migration, transform）

**Analog:** `migrations/0004_truth_layer_governance.sql:1-93`, `migrations/0002_ingest_foundation.sql:1-12`

**要复制的迁移风格：additive side-table，不改 authority backbone**

```sql
CREATE TABLE IF NOT EXISTS truth_promotion_reviews (
    review_id TEXT PRIMARY KEY,
    source_record_id TEXT NOT NULL REFERENCES memory_records(id) ON DELETE CASCADE,
    ...
    decision_state TEXT NOT NULL CHECK (decision_state IN ('pending', 'approved', 'rejected', 'cancelled')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_truth_promotion_reviews_decision
    ON truth_promotion_reviews(decision_state, updated_at DESC);
```

```sql
ALTER TABLE memory_records ADD COLUMN chunk_index INTEGER;
ALTER TABLE memory_records ADD COLUMN chunk_count INTEGER;

CREATE INDEX IF NOT EXISTS idx_memory_records_validity_window
    ON memory_records(valid_from, valid_to);
```

**Phase 5 应用方式**

- 继续沿用 Phase 3 的 side-table 思路，不要改 `memory_records`、`memory_records_fts`，也不要把 queue 状态塞进 retrieval 主表。
- `SPQ` 和 `LPQ` 建议做成两张显式表，列名保持镜像一致，而不是一个总表加 `queue_kind`。这样最符合 D-01 的“explicit and distinct”。
- 两张 queue 表都应该有显式调度列，而不是只存一个 JSON blob。最少保留：
  - `item_id`
  - `trigger_kind`
  - `status`
  - `dedupe_key`
  - `priority`
  - `attempt_count`
  - `budget_cost`
  - `cooldown_until`
  - `next_eligible_at`
  - `payload_json`
  - `created_at`
  - `updated_at`
  - `processed_at`
- 本地自适应状态建议单独 side-table 保存，例如：
  - `adaptive_self_state`
  - `adaptive_risk_boundary`
- 共享真值相关的长周期输出不要落到本地 adaptive 表里。要么落 `lpq_candidate_outbox`，要么由 service 调现有 governance seam 创建 review/candidate，但都不能直接生成 T1/T2 authority row。

**优先选择**

- Queue state 用小 enum + `CHECK`，不要 free-form string。
- 索引优先放在 `status, next_eligible_at, updated_at DESC`，服务层才有稳定的“取到期 item”语义。
- 如果需要统一审计流，新增 `rumination_writeback_events`，不要把审计混入普通日志。

---

### `src/core/migrations.rs`（config, batch）

**Analog:** `src/core/migrations.rs:4-28`

**要复制的注册模式**

```rust
const TRUTH_LAYER_GOVERNANCE_SQL: &str =
    include_str!("../../migrations/0004_truth_layer_governance.sql");

Migrations::new(vec![
    M::up(FOUNDATION_SCHEMA_SQL)
        .foreign_key_check()
        .comment("foundation schema bootstrap"),
    ...
    M::up(TRUTH_LAYER_GOVERNANCE_SQL)
        .foreign_key_check()
        .comment("truth governance"),
])
```

**Phase 5 应用方式**

- Phase 5 migration 独立注册为 `0005`，不要重写前 4 个 migration。
- comment 直接写 `rumination and adaptive write-back`，不要模糊成 search 或 cognition。
- schema version 应从 4 升到 5，对应更新 `tests/foundation_schema.rs`。

---

### `src/cognition/mod.rs`（config, transform）

**Analog:** `src/cognition/mod.rs:1-6`

**要复制的模块导出风格**

```rust
pub mod action;
pub mod assembly;
pub mod metacog;
pub mod report;
pub mod value;
pub mod working_memory;
```

**Phase 5 应用方式**

- 只追加 `pub mod rumination;`，不要把实现塞进 `mod.rs`。
- Rumination 属于 `cognition`，不是 `agent`、`search`、`memory` 的附属函数。
- 如果后面确实需要拆成 `adaptive.rs` / `learning.rs`，也先从 `cognition` 下面继续加子模块，不要把本地 adaptive state 混回 `working_memory.rs`。

---

### `src/cognition/assembly.rs`（service, transform）

**Analog:** `src/cognition/assembly.rs:1-220`

**要复制的 provider composition + runtime-only snapshot 风格**

```rust
pub trait SelfStateProvider {
    fn snapshot(&self, request: &WorkingMemoryRequest, truths: &[TruthRecord]) -> SelfStateSnapshot;
}

pub struct SelfStateSnapshot {
    pub task_context: Option<String>,
    pub capability_flags: Vec<String>,
    pub readiness_flags: Vec<String>,
    pub facts: Vec<SelfStateFact>,
}
```

**Phase 5 应用方式**

- Phase 5 对 `assembly.rs` 的改动只应是“base provider + local adaptation overlay provider”的组合扩展，不要把 rumination 写回逻辑塞进 assembler。
- `WorkingMemory` 仍保持 runtime-only；overlay 只是在装配时读取 `local_adaptation_entries`，不是把整份 working memory 持久化或做可变缓存。
- `self_state`、`risk_boundary` 和本地 T3-adjacent 适配项继续通过现有 `SelfStateProvider` seam 进入快照，避免发明第二套自我模型装配协议。

---

### `src/cognition/rumination.rs`（service, event-driven）

**Primary Analog 1:** `src/agent/orchestration.rs:152-293`, `src/agent/orchestration.rs:420-437`

**要复制的 bounded orchestration + typed report 模式**

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AgentSearchReport {
    pub working_memory: WorkingMemory,
    pub decision: DecisionReport,
    pub retrieval_steps: Vec<RetrievalStepReport>,
    pub citations: Vec<Citation>,
    pub executed_steps: usize,
    pub step_limit: usize,
}

fn execute(&self, request: &AgentSearchRequest) -> Result<AgentSearchReport, AgentSearchError> {
    let retrieval_steps = ...;
    let working_memory = self.assembler.assemble(&request.working_memory)?;
    let scored_branches = self.scorer.score(&working_memory, &request.branch_values)?;
    let decision = self.gate.evaluate(&working_memory, scored_branches)?;

    Ok(AgentSearchReport {
        citations: collect_unique_citations(&retrieval_steps),
        executed_steps: retrieval_steps.len(),
        retrieval_steps,
        step_limit: request.step_limit,
        working_memory,
        decision,
    })
}
```

**Primary Analog 2:** `src/memory/governance.rs:310-374`

**要复制的 candidate-first service seam**

```rust
pub fn create_ontology_candidate(
    &self,
    request: CreateOntologyCandidateRequest,
) -> Result<OntologyCandidate, TruthGovernanceError> { ... }

pub fn list_pending_reviews(&self) -> Result<Vec<PromotionReview>, TruthGovernanceError> { ... }

pub fn list_pending_candidates(&self) -> Result<Vec<OntologyCandidate>, TruthGovernanceError> { ... }
```

**Primary Analog 3:** `src/cognition/report.rs:10-44`, `src/cognition/metacog.rs:107-147`, `src/cognition/metacog.rs:181-214`

**要复制的 typed outcome 输入面**

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecisionReport {
    pub scored_branches: Vec<ScoredBranchReport>,
    pub selected_branch: Option<ScoredBranchReport>,
    pub gate: GateReport,
    pub active_risks: Vec<String>,
    pub metacog_flags: Vec<MetacognitiveFlag>,
}
```

```rust
if self.has_risk_marker(&top_branch.branch, &self.soft_veto_risk_markers) {
    ...
    return DecisionReport {
        ...
        gate: GateReport {
            decision: GateDecision::SoftVeto,
            ...
        },
        active_risks,
        metacog_flags,
    };
}
```

**Phase 5 应用方式**

- 把 `rumination.rs` 做成单入口 learning control plane，像 `AgentSearchOrchestrator` 一样显式串联步骤，而不是让 queue/scheduler/write-back 分散在多处 helper 里。
- 推荐结构：
  - `RuminationTrigger`
  - `RuminationQueueKind`
  - `RuminationItemStatus`
  - `RuminationItem`
  - `ShortCycleWriteBackReport`
  - `LongCycleCandidateReport`
  - `RuminationService`
- `RuminationService` 的主方法建议保持分离：
  - `capture_short_cycle(...)`
  - `capture_long_cycle(...)`
  - `run_spq(...)`
  - `run_lpq(...)`
- 触发输入不要直接读日志或原始 SQL。优先消费现有 typed 输入：
  - `DecisionReport.gate.decision`
  - `DecisionReport.active_risks`
  - `DecisionReport.metacog_flags`
  - `AgentSearchReport.citations`
  - `AgentSearchReport.working_memory.present.world_fragments`
- `SPQ` 触发直接映射现有 metacog 结果：
  - `SoftVeto`
  - `HardVeto`
  - `Escalate`
  - 外部 `user_correction`
  - 外部 `action_failure`
- `LPQ` 触发保持显式输入，不要为了 session boundary/idle window 去反向污染 agent-search 主链。可以让 Phase 5 的 API 明确接收：
  - `session_closed_at`
  - `idle_since`
  - `evidence_accumulated`
  - `pattern_counter`
- 输出也必须是 typed report，不要返回“已经 rumination 完成”的字符串。

**本 phase 的关键边界**

- 短周期自动写回只落本地 adaptive 表。
- 长周期只产出：
  - `skill_template`
  - `promotion_candidate`
  - `value_adjustment_candidate`
- `promotion_candidate` 最好先保存在 LPQ outbox 或 equivalent proposal struct，再由显式 governance adapter 消费；不要在短周期里直接触碰 shared truth。
- 不要修改 `WorkingMemory` / `DecisionReport` / `AgentSearchReport` 既有合同，只把它们当输入。

---

### `src/memory/repository.rs`（store, CRUD）

**Analog 1:** `src/memory/repository.rs:241-374`, `src/memory/repository.rs:470-530`

**要复制的 insert/get/list_pending 模式**

```rust
pub fn insert_promotion_review(
    &self,
    review: &PromotionReview,
) -> Result<(), RepositoryError> { ... }

pub fn list_pending_promotion_reviews(&self) -> Result<Vec<PromotionReview>, RepositoryError> {
    let mut statement = self.conn.prepare(
        r#"
        SELECT ...
        FROM truth_promotion_reviews
        WHERE decision_state = ?1
        ORDER BY updated_at DESC, review_id DESC
        "#,
    )?;
    ...
}
```

```rust
pub fn list_pending_ontology_candidates(
    &self,
) -> Result<Vec<OntologyCandidate>, RepositoryError> {
    let mut statement = self.conn.prepare(
        r#"
        SELECT ...
        FROM truth_ontology_candidates
        WHERE candidate_state = ?1
        ORDER BY updated_at DESC, candidate_id DESC
        "#,
    )?;
    ...
}
```

**Analog 2:** `src/memory/repository.rs:578-600`, `src/cognition/working_memory.rs:47-82`

**要复制的 typed projection 模式**

```rust
pub fn get_truth_record(&self, id: &str) -> Result<Option<TruthRecord>, RepositoryError> {
    let Some(base) = self.get_record(id)? else {
        return Ok(None);
    };

    let truth_record = match base.truth_layer {
        TruthLayer::T1 => TruthRecord::T1 { base },
        TruthLayer::T2 => TruthRecord::T2 {
            open_candidates: self.list_ontology_candidates(id)?,
            base,
        },
        TruthLayer::T3 => TruthRecord::T3 {
            t3_state: Some(self.get_t3_state(id)?.ok_or_else(...)?),
            open_reviews: self.list_promotion_reviews(id)?,
            base,
        },
    };

    Ok(Some(truth_record))
}
```

**Phase 5 应用方式**

- Queue SQL 继续只放在 repository。RuminationService 负责策略和 guard，不负责写 SQL。
- 推荐新增 repository API：
  - `insert_spq_item`
  - `insert_lpq_item`
  - `find_pending_spq_by_dedupe_key`
  - `find_pending_lpq_by_dedupe_key`
  - `list_due_spq_items`
  - `list_due_lpq_items`
  - `update_spq_item_status`
  - `update_lpq_item_status`
  - `upsert_adaptive_self_state`
  - `upsert_adaptive_risk_boundary`
  - `insert_lpq_candidate_outbox`
  - `list_pending_lpq_candidate_outbox`
- pending 队列的 membership 必须像 Phase 3 一样依赖显式状态列，不要靠“最近一次日志里出现了什么”来推断。
- `adaptive_self_state` / `adaptive_risk_boundary` 需要独立 repository API，不要复用 `truth_ontology_candidates` 或 `memory_records.provenance_json`。
- 长周期生成 shared-truth-facing 提案时，repository 最多保存 outbox/proposal row；真正进入 `TruthGovernanceService` 的动作放在 service 层。

**推荐排序语义**

- `SPQ`: `ORDER BY priority DESC, next_eligible_at ASC, updated_at ASC`
- `LPQ`: `ORDER BY next_eligible_at ASC, updated_at ASC`
- 两者都显式支持：
  - `pending`
  - `leased`
  - `applied` / `emitted`
  - `skipped`
  - `cancelled`

---

### `tests/foundation_schema.rs`（test, batch）

**Analog:** `tests/foundation_schema.rs:137-220`

**要复制的 schema bootstrap 回归模式**

```rust
#[test]
fn foundation_migration_bootstraps_clean_db() {
    let db = Database::open(&path).expect("fresh database should bootstrap");
    assert_eq!(db.schema_version().expect("schema version"), 4);

    let names = table_names(&path);
    assert!(
        names.contains(&"truth_t3_state".to_string())
            && names.contains(&"truth_promotion_reviews".to_string())
            && names.contains(&"truth_promotion_evidence".to_string())
            && names.contains(&"truth_ontology_candidates".to_string()),
        "truth-governance side tables should exist: {names:?}"
    );
}
```

**Phase 5 应用方式**

- schema version 改成 5。
- 断言新增 rumination/adaptive side tables 和索引存在。
- 继续保留“不能出现 vec/rig 表”的负向断言思路；Phase 5 也不应引入 UI 或 semantic retrieval 表。
- 建议新增断言：普通 authority 表名不变，证明 Phase 5 没有重写 retrieval backbone。

---

### `tests/rumination_queue.rs`（test, event-driven）

**Analog 1:** `tests/truth_governance.rs:1-220`

**要复制的 repository-backed queue/state assertions**

```rust
let pending_reviews = service.list_pending_reviews()?;
assert_eq!(pending_reviews.len(), 1);
assert_eq!(pending_reviews[0].decision_state, DecisionState::Pending);
```

**Analog 2:** `tests/agent_search.rs:150-320`

**要复制的 scripted input + structured report test style**

```rust
let report = orchestrator.execute(&request).expect("agent search");
assert_eq!(report.executed_steps, 2);
assert!(!report.citations.is_empty());
```

**Phase 5 应用方式**

- `tests/rumination_queue.rs` 负责覆盖 Plan `05-01` 的双队列控制面，不要和短周期写回或治理桥接混在一个文件里。
- 断言重点应放在：
  - `action_failure` / `user_correction` / metacognitive veto 进入 `SPQ`
  - `session_boundary` / `evidence_accumulation` / `idle_window` / `abnormal_pattern_accumulation` 进入 `LPQ`
  - `route -> dedupe -> cooldown -> budget -> enqueue` 的 durable 节流顺序
  - `SPQ` claim 顺序永远先于 ready `LPQ`
- 如果没有完全等价 analog，也不要退回 enum parse 小测试；这里必须延续现有 integration-style repository + service 联测模式。

---

### `tests/rumination_writeback.rs`（test, event-driven）

**Analog 1:** `tests/agent_search.rs:150-226`, `tests/agent_search.rs:229-320`

**要复制的 scripted orchestration 测试骨架**

```rust
fn sample_agent_search_report() -> AgentSearchReport { ... }

#[derive(Clone)]
struct ScriptedRetriever { ... }

impl RetrievalPort for ScriptedRetriever {
    fn search(&self, request: &SearchRequest) -> anyhow::Result<SearchResponse> { ... }
}
```

**Analog 2:** `tests/value_metacog.rs:143-300`

**要复制的 gate-trigger 覆盖面**

```rust
assert_eq!(soft_report.gate.decision, GateDecision::SoftVeto);
assert_eq!(hard_report.gate.decision, GateDecision::HardVeto);
assert_eq!(escalate_report.gate.decision, GateDecision::Escalate);
assert!(escalate_report.gate.autonomy_paused);
```

**Analog 3:** `tests/truth_governance.rs:529-705`

**要复制的 queue separation + non-mutation 断言**

```rust
let pending_reviews = service.list_pending_reviews()?;
let pending_candidates = service.list_pending_candidates()?;

assert_eq!(pending_reviews.len(), 1);
assert_eq!(pending_candidates.len(), 1);
```

**Phase 5 应用方式**

- 新测试文件优先做 integration 风格，不要只测 enum parse。
- 必测场景：
  - `SPQ` 会因 `SoftVeto` / `HardVeto` / `Escalate` / user correction 入队
  - dedupe/cooldown/budget 能阻止重复 rumination
  - `SPQ` 只更新本地 adaptive state，不新增或改写 T1/T2 authority row
  - `LPQ` 只产出 `skill_template` / `promotion_candidate` / `value_adjustment_candidate`
  - governance-facing candidate 进入 pending proposal/outbox 或 governance queue，但不会自动 approved
  - ordinary retrieval / working memory / agent-search 的原测试继续通过
- 测试断言风格延续 Phase 3/4：
  - 断言具体状态枚举
  - 断言 queue 数量
  - 断言 source truth 仍保持原 truth layer
  - 断言 structured report 里保留 citations / flags / diagnostics

---

### `tests/rumination_governance_integration.rs`（test, event-driven）

**Analog:** `tests/truth_governance.rs:529-705`

**要复制的 pending queue visibility + non-approval 验证风格**

```rust
let pending_reviews = service.list_pending_reviews()?;
let pending_candidates = service.list_pending_candidates()?;

assert_eq!(pending_reviews.len(), 1);
assert_eq!(pending_candidates.len(), 1);
```

**Phase 5 应用方式**

- 该测试文件只负责 Plan `05-03` 的长周期 candidate 与 Phase 3 governance 桥接，不承担短周期 overlay 断言。
- 重点验证：
  - `LPQ` 只产生 `skill_template`、`promotion_candidate`、`value_adjustment_candidate`
  - `promotion_candidate` materialize 后能在 Phase 3 pending review / candidate 队列中可见
  - `governance_ref_id` 被回写到统一 rumination candidate 记录
  - 不存在 approved / accepted 的自动状态推进
- 当前仓库没有一个完全等价的 “rumination + governance bridge” 现成测试，但 `tests/truth_governance.rs` 已提供最接近的 canonical queue-visibility analog；执行时必须沿用那种 repository/service integration 断言粒度，而不是退化成 mock-only 单元测试。

## Shared Patterns

### 触发输入：只消费现有结构化报告

**Source:** `src/cognition/report.rs:10-44`, `src/agent/orchestration.rs:160-168`

```rust
pub struct DecisionReport {
    pub scored_branches: Vec<ScoredBranchReport>,
    pub selected_branch: Option<ScoredBranchReport>,
    pub gate: GateReport,
    pub active_risks: Vec<String>,
    pub metacog_flags: Vec<MetacognitiveFlag>,
}
```

```rust
pub struct AgentSearchReport {
    pub working_memory: WorkingMemory,
    pub decision: DecisionReport,
    pub retrieval_steps: Vec<RetrievalStepReport>,
    pub citations: Vec<Citation>,
    pub executed_steps: usize,
    pub step_limit: usize,
}
```

**Apply to:** 所有 SPQ/LPQ trigger capture 和 write-back report 入口

- Rumination 输入直接吃这些 DTO，不要回去重建第二套 search/decision model。
- `DecisionReport.gate` 是最稳的 SPQ 触发面。
- `AgentSearchReport.citations` 和 `working_memory.present.world_fragments` 是长周期证据引用面。

### 共享真值路径：candidate/proposal-first

**Source:** `src/memory/governance.rs:310-374`, `src/memory/repository.rs:500-530`, `tests/truth_governance.rs:529-705`

```rust
pub fn list_pending_candidates(&self) -> Result<Vec<OntologyCandidate>, TruthGovernanceError> {
    self.repository
        .list_pending_ontology_candidates()
        .map_err(Into::into)
}
```

**Apply to:** 所有 `promotion_candidate` 相关 Phase 5 输出

- LPQ 产生 shared-truth-facing 结果时，沿用 pending queue 思路。
- 不要让 SPQ/LPQ 直接改 `memory_records.truth_layer`。
- 能进入 shared-truth 路径的，必须是 review/candidate/proposal，而不是本地 adaptive patch。

### 本地自适应路径：与 shared truth 分仓

**Source:** `src/cognition/working_memory.rs:20-26`, `src/cognition/assembly.rs:139-155`, `src/cognition/metacog.rs:123-146`

```rust
pub struct SelfStateSnapshot {
    pub task_context: Option<String>,
    pub capability_flags: Vec<String>,
    pub readiness_flags: Vec<String>,
    pub facts: Vec<SelfStateFact>,
}
```

**Apply to:** `adaptive_self_state`, `adaptive_risk_boundary`, `ShortCycleWriteBackReport`

- 现有 `SelfStateSnapshot` 是 runtime snapshot，不是 durable store。
- Phase 5 应新增 durable local adaptive 表，但保持字段风格和 `SelfStateSnapshot` 一致，避免再造第三种 self model 语言。
- `active_risks`、`metacog_flags` 可以成为短周期修正来源，但它们的持久化去向必须是本地 adaptive state，不是 truth governance 表。

### 保持主链不变：rumination 仍在 Rig 边界之外

**Source:** `src/agent/rig_adapter.rs:17-25`, `src/agent/rig_adapter.rs:46-50`

```rust
impl Default for RigBoundary {
    fn default() -> Self {
        Self {
            tool_name: "internal_agent_search",
            default_max_turns: 4,
            allows_truth_write: false,
            allows_semantic_retrieval: false,
            allows_rumination: false,
        }
    }
}
```

**Apply to:** 所有 Phase 5 agent integration 设计

- 不要把 rumination 作为 Rig tool 权限默认打开。
- Phase 5 优先在 `AgentSearchReport` 返回之后、agent boundary 之外接 capture/process。
- 这样能保住现有 retrieve -> assemble -> score -> gate 合同和 AGT-04 的 no-bypass 约束。

## No Exact Analog Found

| File / Concern | Role | Data Flow | Reason |
|---|---|---|---|
| `src/cognition/rumination.rs` 的双队列调度本体 | service | event-driven | 当前仓库还没有既做 queue scheduling、又做 local write-back、又保持 candidate-first shared-truth 边界的单一 service；应组合 `src/agent/orchestration.rs` 的 bounded orchestration 与 `src/memory/governance.rs` 的 candidate-first seam。 |
| `value_adjustment_candidate` 的 typed payload | model | transform | 当前只有 ontology candidate，没有 value-layer candidate；建议复用 `src/memory/truth.rs` 的 typed enum/`as_str`/`parse` 风格，但先放在 `cognition/rumination.rs` 或其子模块，不要塞进 truth-layer 模型。 |

## 推荐选择

- 优先采用 `src/cognition/rumination.rs` 作为单入口 bounded learning control plane，内部再分 `capture` / `schedule` / `apply` / `emit`，不要把写回逻辑散落到 agent、search、interfaces。
- 优先采用两张显式 queue 表 `SPQ` / `LPQ` 加两张本地 adaptive state 表；本地短周期写回与 shared-truth proposal/outbox 分仓，避免边界塌缩。
- 优先让 LPQ 先产出 `promotion_candidate` outbox，再显式接入 `TruthGovernanceService`；不要让长周期直接批准 shared truth。
- 优先保持 `DecisionReport` / `AgentSearchReport` / `WorkingMemory` 只读输入合同不变，Phase 5 通过消费这些结构化报告增量接入学习控制，而不是回写主链。
