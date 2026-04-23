# Phase 5: Rumination And Adaptive Write-back - Research

**Researched:** 2026-04-16  
**Domain:** 双队列反刍、受控写回、治理候选集成  
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
### Queue Model And Triggering
- **D-01:** Phase 5 uses a dual-queue learning model: `SPQ` for short-cycle synchronous rumination and `LPQ` for long-cycle asynchronous rumination.
- **D-02:** `SPQ` is triggered by action failure, user correction, and metacognitive veto events.
- **D-03:** `LPQ` is triggered by session boundaries, evidence accumulation, idle windows, and abnormal-pattern accumulation.
- **D-04:** Queue triggering must support deduplication, minimum interval / cooldown, and bounded budget so rumination does not loop uncontrollably.
- **D-05:** `SPQ` always has higher priority than `LPQ`.

### Short-Cycle Write-back Boundaries
- **D-06:** Short-cycle write-back may update `self_state`, `risk_boundary`, and local/private T3-adjacent adaptation state.
- **D-07:** Short-cycle write-back must not directly mutate shared T2/T1 truth.
- **D-08:** Short-cycle write-back is primarily corrective and should optimize for immediate next-step safety and self-model correction.

### Long-Cycle Output Shapes
- **D-09:** Long-cycle rumination outputs must share a unified queue-item schema.
- **D-10:** The initial long-cycle output classes are:
  - `skill_template`
  - `promotion_candidate`
  - `value_adjustment_candidate`
- **D-11:** Long-cycle processing is allowed to synthesize candidates and proposals, but not to auto-apply them into shared truth.

### Write-back Safety And Approval
- **D-12:** Default write-back policy is candidate/proposal-first, not direct mutation.
- **D-13:** Shared-truth-facing changes remain proposal-driven and require explicit governance handling rather than automatic approval.
- **D-14:** The system should distinguish between:
  - local adaptive updates that are allowed automatically
  - governance candidates that must be reviewed or consumed by later services

### the agent's Discretion
- Exact queue item struct names and storage split, as long as `SPQ`/`LPQ` remain explicit and distinct.
- Exact cooldown / dedupe / budget field names and default values.
- Exact decomposition of `self_state` and `risk_boundary` write-back targets.
- Exact long-cycle candidate payload shape, as long as the three output classes above stay first-class.

### Deferred Ideas (OUT OF SCOPE)
- fully autonomous shared-truth mutation
- semantic retrieval driven rumination
- UI-driven review consoles
- cross-process distributed learning queues
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| LRN-01 | System can route write-back work into short-cycle and long-cycle queues instead of treating all learning as one batch process. [VERIFIED: `.planning/REQUIREMENTS.md`; VERIFIED: `.planning/ROADMAP.md`] | 用显式 `SPQ`/`LPQ` 路由、统一 `rumination_queue_items` schema、`trigger_state` 节流账本和 `SPQ > LPQ` 调度规则实现，不把短长周期塞进同一优先级链。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`; CITED: `doc/0415-反刍机制.md`] |
| LRN-02 | Short-cycle write-back can update self-model or risk-boundary state from action outcomes and user correction without directly mutating shared truth. [VERIFIED: `.planning/REQUIREMENTS.md`] | 短周期只写本地自适应侧表或追加式 ledger，由 `SelfStateProvider` 读取 overlay；禁止直接改 `memory_records` 的共享 T2/T1 真值或绕过 Phase 3 治理服务。 [VERIFIED: `src/cognition/assembly.rs`; VERIFIED: `src/cognition/working_memory.rs`; VERIFIED: `src/memory/governance.rs`; CITED: `doc/0415-自我模型.md`; CITED: `doc/0415-反刍机制.md`] |
| LRN-03 | Long-cycle write-back can produce skill templates, shared-fact promotion candidates, or value-adjustment candidates from accumulated evidence. [VERIFIED: `.planning/REQUIREMENTS.md`] | 长周期输出统一落成 `rumination_candidates`，其中 `promotion_candidate` 直接桥接到 Phase 3 的 `TruthGovernanceService` 提案/候选对象，`skill_template` 与 `value_adjustment_candidate` 保持 candidate-first。 [VERIFIED: `src/memory/governance.rs`; VERIFIED: `src/memory/repository.rs`; VERIFIED: `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`; CITED: `doc/0415-反刍机制.md`; CITED: `doc/0415-真值层.md`; CITED: `doc/0415-价值层.md`] |
</phase_requirements>

## Summary

Phase 5 不应实现成“后台神秘自学习”，而应实现成一个有审计、有预算、有边界的学习控制面。当前代码已经有 Phase 3 的治理对象与 pending queue、Phase 4 的 `WorkingMemory` / `DecisionReport` / `AgentSearchReport`，因此最稳的方向不是重做检索、工作记忆、评分或 Rig，而是在它们之上新增一个 `rumination` 协调层，负责触发归一化、双队列调度、短周期局部写回和长周期候选产出。 [VERIFIED: `src/memory/governance.rs`; VERIFIED: `src/cognition/report.rs`; VERIFIED: `src/agent/orchestration.rs`; VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`; CITED: `doc/0415-反刍机制.md`]

推荐采用“单一队列项表 + 触发节流表 + 本地自适应写回 ledger + 长周期候选表”的 SQLite side-table 方案。这样既延续了当前仓库 Phase 3 的 additive side-table 演进模式，也能保持 `SPQ`/`LPQ` 显式可查、可重放、可测试；其中 `SPQ` 走同步 drain，`LPQ` 走机会式异步或显式 sweep，而不是引入常驻后台 daemon。 [VERIFIED: `migrations/0004_truth_layer_governance.sql`; VERIFIED: `src/memory/repository.rs`; VERIFIED: `.planning/PROJECT.md`; CITED: `doc/0415-反刍机制.md`]

**Primary recommendation:** 以 `src/cognition/rumination.rs` 作为编排层，复用现有 `DecisionReport` / `AgentSearchReport` 作为输入，新增 SQLite 队列与候选 side tables，短周期只写本地 overlay，长周期只产出 candidate/proposal 并桥接 Phase 3 治理服务。 [VERIFIED: `src/cognition/report.rs`; VERIFIED: `src/agent/orchestration.rs`; VERIFIED: `src/memory/governance.rs`; CITED: `doc/0415-反刍机制.md`]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|--------------|----------------|-----------|
| 触发归一化与仲裁 | API / Backend [ASSUMED] | — | 触发来自行动结果、用户纠正、元认知 veto 和 session/idle 信号，属于服务层控制逻辑，不属于存储层。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`; VERIFIED: `src/cognition/metacog.rs`; CITED: `doc/0415-反刍机制.md`; CITED: `doc/0415-元认知层.md`] |
| `SPQ` / `LPQ` 持久化与 claim/schedule | Database / Storage [ASSUMED] | API / Backend [ASSUMED] | 队列顺序、去重、冷却和预算需要 durable 状态；调度策略由服务层读取这些状态执行。 [VERIFIED: `src/memory/repository.rs`; VERIFIED: `migrations/0004_truth_layer_governance.sql`; CITED: `doc/0415-反刍机制.md`] |
| 短周期本地写回 | API / Backend [ASSUMED] | Database / Storage [ASSUMED] | 写回规则是认知边界，落地形态应是本地 overlay/ledger，而不是直接改共享 truth row。 [VERIFIED: `src/cognition/assembly.rs`; VERIFIED: `src/cognition/working_memory.rs`; CITED: `doc/0415-自我模型.md`; CITED: `doc/0415-反刍机制.md`] |
| 长周期候选生成 | API / Backend [ASSUMED] | Database / Storage [ASSUMED] | 候选生成依赖证据聚合与 report 解释，但结果需要 durable candidate rows 供后续治理或人工审阅。 [VERIFIED: `src/agent/orchestration.rs`; VERIFIED: `src/memory/governance.rs`; CITED: `doc/0415-反刍机制.md`; CITED: `doc/0415-真值层.md`] |
| 共享真值治理消费 | API / Backend [ASSUMED] | Database / Storage [ASSUMED] | 共享 truth 相关变更已经由 `TruthGovernanceService` 负责，Phase 5 只应调用该服务创建 review/candidate，不应旁路写表。 [VERIFIED: `src/memory/governance.rs`; VERIFIED: `.planning/phases/03-truth-layer-governance/03-02-SUMMARY.md`; VERIFIED: `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`] |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rusqlite` | `0.37.0` [VERIFIED: `Cargo.lock`; VERIFIED: `Cargo.toml`] | 队列表、候选表和本地写回 side tables | 当前仓库所有 durable state 都经由 SQLite + repository 层管理，Phase 5 延续该模式最小化迁移风险。 [VERIFIED: `src/core/db.rs`; VERIFIED: `src/memory/repository.rs`; VERIFIED: `.planning/PROJECT.md`] |
| `serde_json` | `1.0.149` [VERIFIED: `Cargo.lock`; VERIFIED: `Cargo.toml`] | queue payload、candidate payload、审计 notes | 当前治理与 provenance 已大量使用 JSON payload，适合承载统一候选 schema。 [VERIFIED: `src/memory/governance.rs`; VERIFIED: `src/memory/truth.rs`; VERIFIED: `src/memory/repository.rs`] |
| `tokio` | `1.52.0` [VERIFIED: `Cargo.lock`; VERIFIED: `Cargo.toml`] | `LPQ` 的机会式异步 drain 边界 | `tokio` 已在 Phase 4 引入，但不应把核心 rumination 逻辑异步化；只在 CLI/Rig 外层需要后台 sweep 时使用。 [VERIFIED: `src/agent/rig_adapter.rs`; VERIFIED: `src/interfaces/cli.rs`; VERIFIED: `Cargo.toml`] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `anyhow` / `thiserror` | `1.0.102` / `2.0.18` [VERIFIED: `Cargo.lock`] | Phase 5 domain errors 与接口包装 | 继续遵循“domain error typed, interface boundary wraps”的现有模式。 [VERIFIED: `src/memory/governance.rs`; VERIFIED: `src/cognition/assembly.rs`] |
| 现有 `TruthGovernanceService` | repo-local [VERIFIED: `src/memory/governance.rs`] | `promotion_candidate` 下游治理消费 | Phase 5 不应发明第二套 truth governance storage。 [VERIFIED: `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`] |
| 现有 `DecisionReport` / `AgentSearchReport` | repo-local [VERIFIED: `src/cognition/report.rs`; VERIFIED: `src/agent/orchestration.rs`] | SPQ/LPQ 输入 envelope | 这两个 DTO 已经保留风险、flag、citations 和 step reports，足以作为 rumination 输入面。 [VERIFIED: `src/cognition/report.rs`; VERIFIED: `src/agent/orchestration.rs`] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| SQLite side-table queue [VERIFIED: current repo pattern] | 纯内存队列 [ASSUMED] | 纯内存实现简单，但会丢审计、跨命令连续性和去重/冷却状态，不符合本地优先长期学习目标。 [VERIFIED: `.planning/PROJECT.md`; CITED: `doc/0415-反刍机制.md`] |
| 机会式 `LPQ` sweep [VERIFIED: current CLI/local-first shape] | 常驻 daemon [ASSUMED] | 常驻 daemon 会在 Phase 5 过早引入生命周期管理、并发和退出一致性问题；当前项目仍是单机 CLI/库优先。 [VERIFIED: `.planning/PROJECT.md`; VERIFIED: `src/interfaces/cli.rs`] |
| 复用 Phase 3 governance APIs [VERIFIED: `src/memory/governance.rs`] | 自建第二套 promotion queue [ASSUMED] | 第二套 proposal store 会复制 pending lifecycle，增加状态分叉和迁移成本。 [VERIFIED: `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`] |

**Installation:**
```bash
# Phase 5 初始实现不推荐新增 crate。
# 直接复用当前 manifest 中的 rusqlite / serde_json / tokio / anyhow / thiserror。
```

## Architecture Patterns

### System Architecture Diagram

```text
ActionOutcome / UserCorrection / MetacogVeto / SessionBoundary / IdleSignal
    ↓
RuminationTriggerNormalizer
    ↓
Dedup + Cooldown + BudgetGate
    ├── SPQ item → sync claim → short-cycle processor → local adaptation ledger
    └── LPQ item → async/explicit sweep → candidate generator
                                      ├── skill_template → rumination_candidates
                                      ├── promotion_candidate → TruthGovernanceService
                                      └── value_adjustment_candidate → rumination_candidates
    ↓
Audit trail / completion state / retry state
```

这个流向保持了“触发检测、队列调度、候选生成、实际写回”四段分离，符合 Phase 5 context 明确要求。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`; CITED: `doc/0415-反刍机制.md`]

### Recommended Project Structure

```text
src/
├── cognition/
│   ├── rumination.rs          # trigger normalize + queue orchestration + processors
│   └── mod.rs                 # export rumination seam
├── memory/
│   ├── repository.rs          # queue SQL, candidate SQL, local adaptation SQL
│   ├── governance.rs          # reused, not replaced
│   └── truth.rs               # reused truth/candidate DTOs
└── interfaces/
    └── cli.rs                 # optional queue sweep / inspect commands later

tests/
├── rumination_queue.rs
├── rumination_writeback.rs
└── rumination_governance_integration.rs
```

推荐把 Phase 5 新逻辑主要放在 `cognition` 编排层和 `memory::repository` 的 side-table SQL 中，保持与当前“typed service + repository”分层一致。 [VERIFIED: `src/cognition/assembly.rs`; VERIFIED: `src/memory/repository.rs`; VERIFIED: `.planning/research/ARCHITECTURE.md`]

### Pattern 1: 统一队列项 + 独立触发节流账本

**What:** 使用一张 `rumination_queue_items` 持久化所有 `SPQ`/`LPQ` work item，再用一张 `rumination_trigger_state` 保存 `dedupe_key`、`cooldown_until`、预算窗口和最近一次入队结果。 [VERIFIED: current side-table pattern in `migrations/0004_truth_layer_governance.sql`; CITED: `doc/0415-反刍机制.md`]

**When to use:** 所有触发都先归一化成 `RuminationTriggerEvent`，再统一走 `dedupe -> cooldown -> budget -> enqueue`。 [CITED: `doc/0415-反刍机制.md`]

**Example:**
```rust
// Source: recommended Phase 5 contract synthesized from locked context + current DTO seams. [ASSUMED]
enum QueueTier { Spq, Lpq }

enum RuminationOutputKind {
    ShortCycleWriteback,
    SkillTemplate,
    PromotionCandidate,
    ValueAdjustmentCandidate,
}
```

### Pattern 2: `SPQ` 同步 drain，`LPQ` 机会式 sweep

**What:** `SPQ` 入队后立刻 claim 并在当前调用链完成处理；`LPQ` 只落库，随后由显式 `run_ready_lpq(now, budget)` 或 CLI/Rig 边界的机会式 sweep 处理。 [VERIFIED: `.planning/PROJECT.md`; VERIFIED: `src/interfaces/cli.rs`; CITED: `doc/0415-反刍机制.md`]

**When to use:** 任何会影响“下一步还能不能继续做”的事件都走 `SPQ`；任何需要跨时间聚合的任务都走 `LPQ`。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`; CITED: `doc/0415-反刍机制.md`]

**Example:**
```rust
// Source: locked trigger split from Phase 5 context. [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`]
match trigger.kind {
    TriggerKind::ActionFailure
    | TriggerKind::UserCorrection
    | TriggerKind::MetacogVeto => QueueTier::Spq,
    TriggerKind::SessionBoundary
    | TriggerKind::EvidenceAccumulation
    | TriggerKind::IdleWindow
    | TriggerKind::AbnormalPatternAccumulation => QueueTier::Lpq,
}
```

### Pattern 3: 长周期 candidate-first，治理对象复用 Phase 3

**What:** `LPQ` 不直接改共享 truth；它只产出 `skill_template`、`promotion_candidate`、`value_adjustment_candidate`。其中 `promotion_candidate` 通过 `TruthGovernanceService` materialize 成 Phase 3 已存在的 review/candidate 对象。 [VERIFIED: `src/memory/governance.rs`; VERIFIED: `.planning/phases/03-truth-layer-governance/03-02-SUMMARY.md`; VERIFIED: `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`; CITED: `doc/0415-反刍机制.md`; CITED: `doc/0415-真值层.md`]

**When to use:** 任何触及共享层的写回，都必须先变成 governance object，而不是 Phase 5 自己批准。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`; CITED: `doc/0415-真值层.md`]

### Anti-Patterns to Avoid

- **把 `SPQ` / `LPQ` 做成同一优先级链：** 这会把即时止血和长期消化混在一起，直接违背 0415 双时间尺度设计。 [CITED: `doc/0415-反刍机制.md`]
- **短周期直接写共享 T2/T1：** 这会绕过 Phase 3 治理与提案边界。 [VERIFIED: `src/memory/governance.rs`; CITED: `doc/0415-反刍机制.md`; CITED: `doc/0415-真值层.md`]
- **为了 Phase 5 新建第二套 truth proposal 存储：** 当前 repo 已有 `truth_promotion_reviews` 和 `truth_ontology_candidates`。 [VERIFIED: `migrations/0004_truth_layer_governance.sql`; VERIFIED: `src/memory/repository.rs`] 
- **把完整 `WorkingMemory` / 全量检索结果序列化进 queue payload：** rumination 需要引用和摘要，不需要复制整个前台控制场。 [VERIFIED: `src/cognition/working_memory.rs`; VERIFIED: `src/agent/orchestration.rs`; ASSUMED] 
- **实现常驻后台学习守护进程：** 当前项目仍是单机、本地、CLI/库优先，Phase 5 不需要引入 daemon 生命周期管理。 [VERIFIED: `.planning/PROJECT.md`; VERIFIED: `src/interfaces/cli.rs`] 

## Recommended Queue / Schema Direction

### Queue storage

- 推荐使用一张 `rumination_queue_items` 表承载 `SPQ` 和 `LPQ`，通过 `queue_tier` 枚举、`priority`、`not_before`、`status` 和 `(queue_tier, status, priority, not_before, created_at)` 索引保持双队列显式化与可调度性。 [VERIFIED: current side-table pattern in `migrations/0004_truth_layer_governance.sql`; CITED: `doc/0415-反刍机制.md`]  
- 队列项应至少包含：`item_id`、`queue_tier`、`trigger_kind`、`output_kind`、`subject_ref`、`dedupe_key`、`cooldown_key`、`budget_bucket`、`priority`、`status`、`attempt_count`、`payload_json`、`evidence_refs_json`、`source_report_json`、`created_at`、`updated_at`、`claimed_at`、`completed_at`、`last_error`。这些字段足以覆盖锁定触发、节流和统一输出 contract。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`; VERIFIED: `src/cognition/report.rs`; VERIFIED: `src/agent/orchestration.rs`; CITED: `doc/0415-反刍机制.md`]  
- 推荐再加一张 `rumination_trigger_state` 表，按 `(queue_tier, dedupe_key)` 或 `(cooldown_key)` 持久化最近入队时间、冷却截止、窗口预算消耗和最近结果，避免每次通过历史扫表判重。 [CITED: `doc/0415-反刍机制.md`; VERIFIED: current repository pattern in `src/memory/repository.rs`]  

### Trigger model and throttling

- 推荐先定义统一 `RuminationTriggerEvent`，把 `action_failure`、`user_correction`、`metacog_veto`、`session_boundary`、`evidence_accumulation`、`idle_window`、`abnormal_pattern_accumulation` 归一到一个 typed enum，并附 `subject_ref`、`severity`、`source_report_ref`、`occurred_at`。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`; VERIFIED: `src/cognition/metacog.rs`; VERIFIED: `src/agent/orchestration.rs`; CITED: `doc/0415-反刍机制.md`]  
- 触发处理顺序应固定为 `route -> dedupe -> cooldown -> budget -> enqueue`，不要把 budget 检查放在最前，否则同一事件会因为预算耗尽而跳过去重账本，造成后续短时间重复入队。 [CITED: `doc/0415-反刍机制.md`; ASSUMED]  
- `SPQ` 内部优先级建议对齐 0415：系统级安全阻断 > 用户显式纠正 > 高风险元认知警报 > 严重行动失败 > 普通行动结果。 [CITED: `doc/0415-反刍机制.md`]  
- `LPQ` 内部优先级建议对齐 0415：T3→T2 晋升 > 技能抽取 > 失败模式分析 > 价值偏好更新 > T1 候选提案。 [CITED: `doc/0415-反刍机制.md`]  

### Write-back boundary

- 短周期写回应落到新的本地自适应 ledger，例如 `local_adaptation_entries`，而不是直接改 `memory_records` 或 `truth_*` 表。该 ledger 至少区分 `self_state`、`risk_boundary`、`local_t3_adaptation` 三类 target，并记录 `source_queue_item_id` 与证据引用。 [VERIFIED: `src/cognition/assembly.rs`; VERIFIED: `src/cognition/working_memory.rs`; VERIFIED: `src/memory/repository.rs`; CITED: `doc/0415-自我模型.md`; CITED: `doc/0415-反刍机制.md`]  
- `SelfStateProvider` 目前是最小 request-local seam，因此 Phase 5 最稳的改法是引入“base provider + adaptive overlay provider”组合，而不是把 `SelfStateSnapshot` 直接持久化成新的 authority store。 [VERIFIED: `src/cognition/assembly.rs`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-01-SUMMARY.md`]  
- `risk_boundary` 推荐建成本地 override 或 failure-signature ledger，而不是直接修改价值层硬约束；价值硬边界在 0415 中明确不应被自动学习。 [VERIFIED: `src/cognition/value.rs`; CITED: `doc/0415-价值层.md`; CITED: `doc/0415-自我模型.md`]  

### Unified candidate schema direction

- 推荐新增 `rumination_candidates` 表，统一承载 `skill_template`、`promotion_candidate`、`value_adjustment_candidate`，字段至少包含：`candidate_id`、`candidate_kind`、`source_queue_item_id`、`subject_ref`、`evidence_refs_json`、`proposal_json`、`governance_ref_id`、`candidate_state`、`created_at`、`updated_at`。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`; VERIFIED: `src/memory/truth.rs`; VERIFIED: `src/memory/governance.rs`; CITED: `doc/0415-反刍机制.md`]  
- `promotion_candidate` 的 `proposal_json` 应进一步区分 `t3_to_t2` 与 `t2_to_t1` 两种治理去向，但外层 `candidate_kind` 仍保持单一类目，避免在 Phase 5 再复制一层 truth taxonomy。 [VERIFIED: `src/memory/governance.rs`; VERIFIED: `src/memory/truth.rs`; CITED: `doc/0415-真值层.md`]  
- `skill_template` 在 Phase 5 应保持 candidate-first，不要顺手实现完整技能执行/检索子系统；当前目标只是把成功轨迹压缩成可审计模板。 [CITED: `doc/0415-反刍机制.md`; VERIFIED: `.planning/ROADMAP.md`]  
- `value_adjustment_candidate` 只提议软偏好权重或风险厌恶调节，不触碰固定规则基底。 [CITED: `doc/0415-价值层.md`; CITED: `doc/0415-反刍机制.md`]  

## Service Boundaries

- `cognition::rumination` 应拥有四个入口：`record_action_outcome`、`record_user_correction`、`record_metacog_veto`、`sweep_lpq_ready`，其中前三个负责触发归一化与 `SPQ`/`LPQ` 入队，最后一个负责机会式处理长周期积压。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`; VERIFIED: `src/cognition/metacog.rs`; VERIFIED: `src/agent/orchestration.rs`; ASSUMED]  
- `memory::repository` 只负责 queue/candidate/local-ledger SQL，不在 repository 层塞 trigger policy。这个边界与当前 Phase 3 “policy 在 service，SQL 在 repository” 模式一致。 [VERIFIED: `src/memory/governance.rs`; VERIFIED: `src/memory/repository.rs`; VERIFIED: `.planning/phases/03-truth-layer-governance/03-02-SUMMARY.md`]  
- `TruthGovernanceService` 继续是所有 shared-truth-facing proposal 的唯一消费入口；Phase 5 可以调用它创建 `PromotionReview` 或 `OntologyCandidate`，但不能自己批准它们。 [VERIFIED: `src/memory/governance.rs`; VERIFIED: `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`]  
- `agent` 模块和 Rig adapter 只提供 report 输入，不拥有 rumination authority；当前 `RigBoundary` 已明确 `allows_truth_write = false` 且 `allows_rumination = false`，Phase 5 不应反向突破这个边界。 [VERIFIED: `src/agent/rig_adapter.rs`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-03-SUMMARY.md`]  

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| 共享 truth proposal 生命周期 | 第二套 promotion/candidate 状态机 | `TruthGovernanceService` + 现有 `truth_promotion_reviews` / `truth_ontology_candidates` [VERIFIED: `src/memory/governance.rs`; VERIFIED: `migrations/0004_truth_layer_governance.sql`] | 已有 pending queue、gate state 和审计字段，重复实现只会制造状态分叉。 [VERIFIED: `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`] |
| 前台证据重组 | 新检索或新工作记忆装配器 | 复用 `AgentSearchReport` / `DecisionReport` / `WorkingMemory` [VERIFIED: `src/agent/orchestration.rs`; VERIFIED: `src/cognition/report.rs`; VERIFIED: `src/cognition/working_memory.rs`] | 这些 DTO 已经保留 citation、risk、flag 和 branch evidence；Phase 5 应消费它们，而不是重跑上游。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`] |
| 后台学习进程 | 常驻 daemon / 分布式 worker | 显式 sweep + 机会式 async 调用 [VERIFIED: `.planning/PROJECT.md`; VERIFIED: `src/interfaces/cli.rs`] | 当前项目还是单机本地优先，daemon 会过早引入生命周期与并发复杂度。 [VERIFIED: `.planning/PROJECT.md`] |

**Key insight:** Phase 5 的难点不是“怎么多写点表”，而是“怎么在不破坏 Phase 3/4 边界的前提下，把学习变成可审计的控制面”。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`; VERIFIED: `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`; VERIFIED: `.planning/phases/04-working-memory-and-agent-search/04-03-SUMMARY.md`] 

## Common Pitfalls

### Pitfall 1: 让 `SPQ` 直接复用 `LPQ` 的后台调度

**What goes wrong:** 短周期止血被长周期技能抽取或统计聚合拖住，下一步行动仍带着旧错误继续前进。 [CITED: `doc/0415-反刍机制.md`]  
**Why it happens:** 把所有学习都视为统一 batch job。 [CITED: `doc/0415-反刍机制.md`]  
**How to avoid:** `SPQ` 必须同步 drain，`LPQ` 必须可延迟。 [CITED: `doc/0415-反刍机制.md`]  
**Warning signs:** 同一失败签名在相邻两步重复出现，或用户纠正后短时间内仍命中相同风险。 [ASSUMED]  

### Pitfall 2: 用短周期写回去“修 truth”

**What goes wrong:** 本地失败校正直接污染共享 T2/T1，绕开审计与治理 gate。 [CITED: `doc/0415-反刍机制.md`; CITED: `doc/0415-真值层.md`]  
**Why it happens:** 误把“纠偏快”理解成“可以直接写共享层”。 [CITED: `doc/0415-反刍机制.md`]  
**How to avoid:** 短周期只写 `self_state` / `risk_boundary` / 本地 T3-adjacent overlay；共享层只产 review/candidate。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`; VERIFIED: `src/memory/governance.rs`]  
**Warning signs:** Phase 5 代码开始直接更新 `memory_records.truth_layer`、`truth_promotion_reviews.decision_state` 或任何 T1/T2 authority row。 [VERIFIED: `src/memory/repository.rs`; VERIFIED: `src/memory/governance.rs`]  

### Pitfall 3: 把去重、冷却、预算写成临时内存变量

**What goes wrong:** CLI 多次调用或 session 切换后，系统忘记自己刚处理过什么，导致 trigger storm。 [CITED: `doc/0415-反刍机制.md`; VERIFIED: `.planning/PROJECT.md`]  
**Why it happens:** 低估本地优先系统的跨命令连续性。 [VERIFIED: `.planning/PROJECT.md`]  
**How to avoid:** 节流状态 durable 化到 SQLite。 [VERIFIED: current repo durable-state pattern in `src/memory/repository.rs`; CITED: `doc/0415-反刍机制.md`]  
**Warning signs:** 相同 `subject_ref` 在数秒内反复入队，且无对应 cooldown ledger。 [ASSUMED]  

### Pitfall 4: 为 `promotion_candidate` 保留平行 schema

**What goes wrong:** `LPQ` candidate 和 Phase 3 governance object 各自 pending，后续没人知道哪个才是 canonical。 [VERIFIED: `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`; ASSUMED]  
**Why it happens:** 想先“临时存一下”，但没有定义 canonical owner。 [ASSUMED]  
**How to avoid:** `promotion_candidate` 一旦 materialize，就写回 Phase 3 canonical governance rows，并在 Phase 5 candidate 上保存 `governance_ref_id`。 [VERIFIED: `src/memory/governance.rs`; VERIFIED: `src/memory/truth.rs`]  
**Warning signs:** 同一 source record 同时出现在 rumination candidate 表和 truth governance 表，但没有关联 ID。 [ASSUMED]  

## Testing Focus

- 队列仓储测试必须覆盖：`SPQ > LPQ` 抢占、`dedupe_key` 判重、`cooldown_until` 生效、预算窗口封顶、claim/retry/completed 状态迁移。 [CITED: `doc/0415-反刍机制.md`; VERIFIED: current repo integration-test style in `tests/truth_governance.rs`]  
- 服务测试必须覆盖：`action_failure` / `user_correction` / `metacog_veto` 正确进入 `SPQ`，session/evidence/idle/abnormal-pattern 正确进入 `LPQ`，且 `SPQ` 写回不会触碰 shared truth。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`; VERIFIED: `src/cognition/metacog.rs`; CITED: `doc/0415-反刍机制.md`]  
- 治理集成测试必须覆盖：`promotion_candidate` 创建后，Phase 3 pending review/candidate queue 可见；Phase 5 不自动把 review/candidate 推到 approved/accepted。 [VERIFIED: `src/memory/governance.rs`; VERIFIED: `tests/truth_governance.rs`]  
- 回归测试必须覆盖：Phase 2/3/4 搜索与 agent-search 行为不因 rumination 侧表新增而改变。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`; VERIFIED: current integration suites in `tests/retrieval_cli.rs`, `tests/agent_search.rs`]  

## Anti-Goals

- 不重写 retrieval、working memory、value scoring 或 Rig orchestration。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`]  
- 不实现自动共享 truth 批准。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`; CITED: `doc/0415-真值层.md`]  
- 不实现完整 skill memory 子系统，只实现 `skill_template` 候选产出。 [VERIFIED: `.planning/ROADMAP.md`; CITED: `doc/0415-反刍机制.md`]  
- 不为 Phase 5 引入分布式队列、常驻 worker 或 UI review console。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`]  

## Migration Risks

- `MinimalSelfStateProvider` 当前只读取 request-local flags 和 truth projections；一旦加本地 overlay，如果直接替换而不是组合，Phase 4 的 working-memory 测试会退化。 [VERIFIED: `src/cognition/assembly.rs`; VERIFIED: `tests/working_memory_assembly.rs`]  
- `promotion_candidate` 如果先落 Phase 5 candidate 表、后又复制到 Phase 3 governance 表，最容易出现双 canonical 风险；必须从第一版就定义 canonical owner。 [VERIFIED: `src/memory/governance.rs`; VERIFIED: `src/memory/repository.rs`]  
- `LPQ` 如果被做成自动并发后台执行，会把当前同步 repository/service 栈拉入额外生命周期复杂度；第一版更适合显式 sweep。 [VERIFIED: `src/interfaces/cli.rs`; VERIFIED: `src/core/db.rs`; VERIFIED: `.planning/PROJECT.md`]  

## Code Examples

### 输入 envelope 方向

以下结构与现有 `DecisionReport` / `AgentSearchReport` seam 对齐，避免 Phase 5 自己再重组一次前台报告。 [VERIFIED: `src/cognition/report.rs`; VERIFIED: `src/agent/orchestration.rs`]

```rust
// Source: recommended Phase 5 envelope synthesized from existing report DTOs. [ASSUMED]
struct RuminationInput {
    trigger: RuminationTriggerEvent,
    decision: Option<DecisionReport>,
    agent_report: Option<AgentSearchReport>,
    outcome_summary: Option<String>,
}
```

### 长周期治理桥接方向

`promotion_candidate` 应桥接到 Phase 3 canonical service，而不是直接更新 shared truth。 [VERIFIED: `src/memory/governance.rs`; VERIFIED: `src/memory/truth.rs`]

```rust
// Source: recommended bridge derived from existing governance service API. [ASSUMED]
match candidate.target {
    PromotionTarget::T3ToT2 => governance.create_promotion_review(request)?,
    PromotionTarget::T2ToT1 => governance.create_ontology_candidate(request)?,
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| 单批次 learning job [CITED: `doc/0415-反刍机制.md`] | `SPQ` / `LPQ` 双时间尺度反刍 [CITED: `doc/0415-反刍机制.md`] | 0415 理论定稿 [CITED: `doc/0415-反刍机制.md`] | 队列、预算、写回边界必须分开设计。 [CITED: `doc/0415-反刍机制.md`] |
| 直接把结果写回 shared truth [CITED: `doc/0415-真值层.md`] | candidate/proposal-first + governance handling [VERIFIED: `src/memory/governance.rs`; CITED: `doc/0415-真值层.md`] | Phase 3 完成后 [VERIFIED: `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md`] | Phase 5 只能复用治理 seam，不能旁路。 [VERIFIED: `src/memory/governance.rs`] |
| request-local `self_state` 最小快照 [VERIFIED: `src/cognition/assembly.rs`] | request-local base + adaptive overlay [ASSUMED] | Phase 5 建议方向 [ASSUMED] | 短周期写回可以落地，但不需要持久化整份 working memory。 [VERIFIED: `src/cognition/working_memory.rs`; CITED: `doc/0415-自我模型.md`] |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | “触发归一化与仲裁” 的主责 tier 是 API / Backend。 | Architectural Responsibility Map | 如果后续调用面下沉到别的 tier，planner 的任务分配会偏。 |
| A2 | “`SPQ` / `LPQ` 持久化与 claim/schedule” 的主责 tier 是 Database / Storage，辅责 tier 是 API / Backend。 | Architectural Responsibility Map | 如果 tier 划分不对，可能把调度 policy 和 durable 状态耦合错层。 |
| A3 | “短周期本地写回” 的主责 tier 是 API / Backend，辅责 tier 是 Database / Storage。 | Architectural Responsibility Map | 如果 tier 划分不对，可能把写回规则塞进 repository 或直接写 authority rows。 |
| A4 | “长周期候选生成” 的主责 tier 是 API / Backend，辅责 tier 是 Database / Storage。 | Architectural Responsibility Map | 如果 tier 划分不对，candidate synthesis 可能退化成 SQL-side heuristic。 |
| A5 | “共享真值治理消费” 的主责 tier 是 API / Backend，辅责 tier 是 Database / Storage。 | Architectural Responsibility Map | 如果 tier 划分不对，Phase 5 可能绕过治理 service 直写 truth 表。 |
| A6 | 纯内存队列是一个可比替代方案，但不推荐。 | Alternatives Considered | 如果实际上需要纯内存极简模式，当前 research 会高估 durable queue 的必要性。 |
| A7 | 常驻 daemon 是一个可比替代方案，但不推荐。 | Alternatives Considered | 如果后续必须实时后台处理，planner 需要补 worker lifecycle 设计。 |
| A8 | 自建第二套 promotion queue 是一个可比替代方案，但不推荐。 | Alternatives Considered | 如果 Phase 3 governance API 不足，planner 需要追加 seam 扩展而不是直接复用。 |
| A9 | `QueueTier` / `RuminationOutputKind` 示例 enum 代表合适的第一版 contract。 | Code Examples | 如果最终 contract 需要不同拆分，示例代码应同步调整。 |
| A10 | “重复失败签名再次出现” 是 `SPQ`/短周期调度失效的早期 warning sign。 | Common Pitfalls | 如果告警特征不准，验证步骤可能漏掉真实失效模式。 |
| A11 | “相同 `subject_ref` 短时间反复入队” 是节流状态未 durable 化的 warning sign。 | Common Pitfalls | 如果告警特征不准，验证步骤可能误判队列风暴来源。 |
| A12 | “想先临时存一下” 是平行 governance schema 常见成因。 | Common Pitfalls | 如果成因判断不准，review checklist 会偏离真实根因。 |
| A13 | “rumination candidate 与 truth governance 同时存在但无关联 ID” 是状态分叉的 warning sign。 | Common Pitfalls | 如果告警特征不准，验收时可能错过 canonical owner 分裂。 |
| A14 | `RuminationInput` 以现有 `DecisionReport` / `AgentSearchReport` 为主输入 envelope 是合适方向。 | Code Examples | 如果调用方拿不到这些 DTO，Phase 5 需要额外 adapter 或 outcome schema。 |
| A15 | `promotion_candidate` 桥接到 `create_promotion_review` / `create_ontology_candidate` 是合适的第一版桥接方式。 | Code Examples | 如果桥接层需要更多状态预处理，当前 planner 需要补 adapter 任务。 |
| A16 | `request-local base + adaptive overlay` 是 `self_state` 的合适第一版演化路径。 | State of the Art | 如果后续自我模型独立成 durable subsystem，provider seam 会重构。 |
| A17 | `rtk cargo test --test rumination_queue -- --nocapture` 应作为 Phase 5 的 quick run / per-task sampling 命令。 | Validation Architecture | 如果测试拆分方式不同，planner 需要改 quick-run 命令。 |
| A18 | Session boundary / idle window 第一版应通过显式 service/API 调用与可选 CLI sweep 提供，而不是 daemon。 | Open Questions (resolved) | 如果后续必须实时后台触发，planner 需要补充 worker lifecycle 任务。 |
| A19 | `skill_template` 第一版只应持久化为 candidate-first durable row，不应直接扩展为执行/检索子系统。 | Open Questions (resolved) | 如果后续马上需要技能执行或检索，planner 需要追加 Phase 5 范围或拆出后续 phase。 |

## Open Questions (RESOLVED)

1. **Session boundary 与 idle window 的第一版触发面从哪里来？** [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`]
   What we know: 0415 和 Context 都把它们列为 `LPQ` 触发源。 [VERIFIED: `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md`; CITED: `doc/0415-反刍机制.md`]
   What was unclear: 当前 CLI/库接口尚无显式 session lifecycle hook。 [VERIFIED: `src/interfaces/cli.rs`]
   Resolved answer: 第一版通过 `RuminationService` 的显式 service/API 入口捕获 `session_boundary` 和 `idle_window`，并允许在 CLI 边界做可选 sweep；Phase 5 不引入 daemon、worker lifecycle、或后台常驻调度。 [VERIFIED: `.planning/PROJECT.md`; VERIFIED: `src/interfaces/cli.rs`; ASSUMED]

2. **`skill_template` 先落哪里才不会过早扩 scope？** [VERIFIED: `.planning/ROADMAP.md`]
   What we know: Phase 5 需要产出 `skill_template`，但当前仓库还没有技能记忆子系统。 [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `src/` module layout]
   What was unclear: 模板是否需要立即可执行/可检索。 [VERIFIED: current absence of skill module via `src/` tree inspection]
   Resolved answer: Phase 5 仅把 `skill_template` 持久化为 `rumination_candidates` 中的 candidate-first durable row，保留证据、payload 和状态，不引入技能执行器、专用检索接口或新子系统。 [CITED: `doc/0415-反刍机制.md`; VERIFIED: `.planning/ROADMAP.md`; ASSUMED]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `rustc` | 编译与测试 | ✓ [VERIFIED: local command] | `1.94.1` [VERIFIED: local command] | — |
| `cargo` | 编译、测试、clippy | ✓ [VERIFIED: local command] | `1.94.1` [VERIFIED: local command] | — |
| `sqlite3` CLI | 手工 inspect queue/candidate 表 | ✗ [VERIFIED: local command] | — | 用现有 Rust integration tests 和 repository probes 替代。 [VERIFIED: `tests/foundation_schema.rs`; VERIFIED: `src/memory/repository.rs`] |

**Missing dependencies with no fallback:**
- None. [VERIFIED: local environment probe]

**Missing dependencies with fallback:**
- `sqlite3` CLI 缺失，但不阻塞实现；可用 Rust tests 和 repository reads 替代。 [VERIFIED: local environment probe; VERIFIED: `tests/foundation_schema.rs`] 

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` integration/unit tests [VERIFIED: repo layout; VERIFIED: `cargo test --tests --no-run`] |
| Config file | none [VERIFIED: repo root inspection] |
| Quick run command | `rtk cargo test --test rumination_queue -- --nocapture` [ASSUMED] |
| Full suite command | `rtk cargo test --tests && rtk cargo clippy --all-targets -- -D warnings` [VERIFIED: existing phase summaries; VERIFIED: local compile probe] |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LRN-01 | 双队列路由、判重、冷却、预算、优先级 | integration | `rtk cargo test --test rumination_queue -- --nocapture` | ❌ Wave 0 |
| LRN-02 | 短周期只写本地 overlay，不改 shared truth | integration | `rtk cargo test --test rumination_writeback -- --nocapture` | ❌ Wave 0 |
| LRN-03 | 长周期产出三类 candidate，且 `promotion_candidate` 可桥接 Phase 3 治理 | integration | `rtk cargo test --test rumination_governance_integration -- --nocapture` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `rtk cargo test --test rumination_queue -- --nocapture` [ASSUMED]
- **Per wave merge:** `rtk cargo test --tests` [VERIFIED: current repo practice in prior phase summaries]
- **Phase gate:** `rtk cargo test --tests && rtk cargo clippy --all-targets -- -D warnings` [VERIFIED: current repo practice in prior phase summaries]

### Wave 0 Gaps

- [ ] `tests/rumination_queue.rs` — covers `LRN-01`
- [ ] `tests/rumination_writeback.rs` — covers `LRN-02`
- [ ] `tests/rumination_governance_integration.rs` — covers `LRN-03`
- [ ] repository helpers for queue/candidate/local-ledger inspection — support deterministic assertions [VERIFIED: current repository-centered testing pattern in `tests/truth_governance.rs`]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no [VERIFIED: local CLI/library scope in `.planning/PROJECT.md`] | — |
| V3 Session Management | no [VERIFIED: no auth/session subsystem in current repo] | — |
| V4 Access Control | no [VERIFIED: single-user local-first scope in `.planning/PROJECT.md`] | — |
| V5 Input Validation | yes [VERIFIED: queue/candidate payloads will be externalized from reports/triggers] | typed enums + repository validation + explicit target-layer checks in governance services. [VERIFIED: `src/memory/governance.rs`; VERIFIED: `src/memory/repository.rs`] |
| V6 Cryptography | no [VERIFIED: no crypto scope in Phase 5 materials] | — |

### Known Threat Patterns for this phase

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Trigger storm / replay enqueue | Denial of Service | durable dedupe + cooldown + bounded budget before enqueue. [CITED: `doc/0415-反刍机制.md`] |
| Short-cycle direct truth mutation | Tampering | enforce local-only write targets in SPQ and route shared changes through `TruthGovernanceService`. [VERIFIED: `src/memory/governance.rs`; CITED: `doc/0415-真值层.md`] |
| LPQ stale overwrite of local adaptations | Tampering | separate SPQ and LPQ write targets, optimistic claim + task rollback, append-only audit. [CITED: `doc/0415-反刍机制.md`; ASSUMED] |
| Opaque learning artifact with no provenance | Repudiation | queue item audit fields + candidate source refs + governance IDs. [VERIFIED: current provenance/governance pattern in `src/memory/repository.rs`; VERIFIED: `src/memory/truth.rs`] |

## Sources

### Primary (HIGH confidence)
- `.planning/phases/05-rumination-and-adaptive-write-back/05-CONTEXT.md` - locked decisions, discretion, anti-scope
- `.planning/REQUIREMENTS.md` - `LRN-01`, `LRN-02`, `LRN-03`
- `.planning/ROADMAP.md` - Phase 5 goal and plan split
- `.planning/PROJECT.md` - local-first, Rust, explainability, scope constraints
- `.planning/phases/03-truth-layer-governance/03-02-SUMMARY.md` - governed T3→T2 promotion seam
- `.planning/phases/03-truth-layer-governance/03-03-SUMMARY.md` - candidate-only T2→T1 seam
- `.planning/phases/04-working-memory-and-agent-search/04-01-SUMMARY.md` - runtime-only `WorkingMemory` and `SelfStateProvider`
- `.planning/phases/04-working-memory-and-agent-search/04-02-SUMMARY.md` - metacognitive gate outputs
- `.planning/phases/04-working-memory-and-agent-search/04-03-SUMMARY.md` - `AgentSearchReport` and Rig boundary
- `doc/0415-反刍机制.md` - dual queues, triggers, suppression, write-back protocol
- `doc/0415-元认知层.md` - veto as short-cycle trigger
- `doc/0415-真值层.md` - governance boundary and candidate rules
- `doc/0415-自我模型.md` - short-cycle self-model correction targets
- `doc/0415-价值层.md` - soft-preference-only long-cycle value adjustment
- `src/memory/governance.rs` - canonical governance service APIs
- `src/memory/repository.rs` - additive repository pattern
- `src/cognition/assembly.rs` - `SelfStateProvider` seam
- `src/cognition/report.rs` - `DecisionReport`
- `src/agent/orchestration.rs` - `AgentSearchReport`
- `src/agent/rig_adapter.rs` - explicit no-rumination/no-truth-write Rig boundary

### Secondary (MEDIUM confidence)
- `Cargo.toml` / `Cargo.lock` - current crate versions for Phase 5 standard stack
- local environment probes on 2026-04-16 - `rustc --version`, `cargo --version`, `cargo test --tests --no-run`, `command -v sqlite3`

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Phase 5 初始实现不需要引入新依赖，现有 manifest 与 lockfile 已足够。 [VERIFIED: `Cargo.toml`; VERIFIED: `Cargo.lock`]
- Architecture: HIGH - 双队列、写回边界、治理复用都被 0415 文档与已实现 Phase 3/4 seams 明确约束。 [VERIFIED: `src/memory/governance.rs`; VERIFIED: `src/agent/orchestration.rs`; CITED: `doc/0415-反刍机制.md`]
- Pitfalls: HIGH - 主要风险直接来自双时间尺度冲突、shared truth 污染和状态分叉，文档与现有代码都给了强边界。 [CITED: `doc/0415-反刍机制.md`; CITED: `doc/0415-真值层.md`; VERIFIED: `src/memory/governance.rs`]

**Research date:** 2026-04-16  
**Valid until:** 2026-05-16

## RESEARCH COMPLETE
