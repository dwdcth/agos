# Phase 4: Working Memory And Agent Search - Pattern Map

**Mapped:** 2026-04-16
**Files analyzed:** 9
**Analogs found:** 8 / 9

## Revision Notes

- Phase `04-03` canonically uses `src/agent/orchestration.rs`, `src/agent/rig_adapter.rs`, and `tests/agent_search.rs`.
- This pattern map intentionally treats those filenames as the Phase 4 analog set; do not rename them to older `agent_search.rs` / `rig.rs` / `working_memory_agent_search.rs` variants unless the plans are revised again.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/cognition/mod.rs` | config | transform | `src/memory/mod.rs` | exact |
| `src/cognition/working_memory.rs` | service | request-response | `src/search/mod.rs` | role-match |
| `src/cognition/value.rs` | utility | transform | `src/search/score.rs` | exact |
| `src/cognition/metacog.rs` | service | request-response | `src/memory/governance.rs` | exact |
| `src/agent/mod.rs` | config | transform | `src/memory/mod.rs` | exact |
| `src/agent/orchestration.rs` | service | request-response | `src/memory/governance.rs` | role-match |
| `src/agent/rig_adapter.rs` | provider | request-response | `src/interfaces/cli.rs` | partial |
| `src/lib.rs` | config | transform | `src/lib.rs` | exact |
| `tests/agent_search.rs` | test | request-response | `tests/retrieval_cli.rs` + `tests/truth_governance.rs` | role-match |

## Pattern Assignments

### `src/cognition/mod.rs`（config, transform）

**Analog:** `src/memory/mod.rs:1-4`, `src/lib.rs:1-7`

**要复制的模块导出风格**

```rust
pub mod governance;
pub mod record;
pub mod repository;
pub mod truth;
```

```rust
#![warn(clippy::all)]

pub mod core;
pub mod ingest;
pub mod interfaces;
pub mod memory;
pub mod search;
```

**Phase 4 应用方式**

- 新增 `cognition` 模块时，先在 `src/cognition/mod.rs` 只导出子模块，不在这里塞实现逻辑。
- 再在 `src/lib.rs` 追加 `pub mod cognition;` 和 `pub mod agent;`，沿用当前单 crate、按域分模块的扩展方式。
- 不要把 working memory、value、metacognition 直接塞回 `src/search` 或 `src/memory`；Phase 4 需要新边界，而不是让已有模块膨胀成 god module。

---

### `src/cognition/working_memory.rs`（service, request-response）

**Analog 1:** `src/search/mod.rs:19-90`

**要复制的 typed request/response + 单入口 service 模式**

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchRequest {
    pub query: String,
    pub limit: usize,
    pub filters: SearchFilters,
}

impl SearchRequest {
    pub fn new(query: impl Into<String>) -> Self { ... }
    pub fn with_limit(mut self, limit: usize) -> Self { ... }
    pub fn with_filters(mut self, filters: SearchFilters) -> Self { ... }
}

pub struct SearchService<'db> {
    lexical: lexical::LexicalSearch<'db>,
}

impl<'db> SearchService<'db> {
    pub fn search(&self, request: &SearchRequest) -> Result<SearchResponse, SearchError> {
        let candidates = self.lexical.recall(request)?;
        let scored = score::score_candidates(request, candidates);
        Ok(rerank::rerank_results(request, scored)?)
    }
}
```

**Analog 2:** `src/search/rerank.rs:15-49`

**要复制的“先组 typed 结果，再统一排序/塑形”模式**

```rust
pub fn rerank_results(
    request: &SearchRequest,
    scored: Vec<ScoredCandidate>,
) -> Result<SearchResponse, CitationError> {
    let applied_filters = request.filters.clone();
    let mut results = scored
        .into_iter()
        .map(|candidate| {
            Ok(SearchResult {
                citation: Citation::from_record(&candidate.record)?,
                record: candidate.record,
                snippet: candidate.snippet,
                score: candidate.score,
                trace: ResultTrace {
                    matched_query: request.query.clone(),
                    query_strategies: candidate.query_strategies,
                    applied_filters: applied_filters.clone(),
                },
            })
        })
        .collect::<Result<Vec<_>, CitationError>>()?;

    results.sort_by(|left, right| { ... });
    Ok(SearchResponse { applied_filters, results })
}
```

**Phase 4 应用方式**

- `WorkingMemory` 文件优先同时承载：
  - `WorkingMemoryRequest`
  - `PresentFrame`
  - `ActionBranch`
  - `WorkingMemory`
  - `WorkingMemoryAssembler`
- `WorkingMemoryAssembler` 应该像 `SearchService` 一样是薄编排入口，内部顺序推荐是：
  - 调 ordinary retrieval，拿 `SearchResponse`
  - 调 truth/governance 读模型，拿 `TruthRecord`
  - 组 `PresentFrame`
  - 产出初始 `branches`
  - 返回 typed `WorkingMemory`
- 保持 builder-style assembly，但 builder 只服务组装过程，不要暴露成松散 JSON 拼装接口。
- 组装输入必须消费 `SearchResponse` / `TruthRecord`，不要重新查 SQL 行，也不要把 top-k 结果直接改名叫 working memory。
- `WorkingMemory` 里建议直接保留 evidence 引用到 `SearchResult` 或其等价 typed 子集，这样 citations、trace、truth-layer 解释不会在 Phase 4 丢失。

**推荐形态**

- `WorkingMemory { present: PresentFrame, branches: Vec<ActionBranch> }`
- `PresentFrame` 内部再放：
  - `world_fragments`
  - `self_state`
  - `active_goal`
  - `active_risks`
  - `metacog_flags`
- `ActionBranch` 不要只存字符串动作；至少保留：
  - `kind`
  - `summary`
  - `supporting_evidence`
  - `risk_markers`
  - `value_breakdown` 或可延后填充字段

---

### `src/cognition/value.rs`（utility, transform）

**Analog:** `src/search/score.rs:8-70`

**要复制的“typed breakdown + 纯函数打分”模式**

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScoreBreakdown {
    pub lexical_raw: f32,
    pub lexical_base: f32,
    pub keyword_bonus: f32,
    pub importance_bonus: f32,
    pub recency_bonus: f32,
    pub emotion_bonus: f32,
    pub final_score: f32,
}

pub fn score_candidates(
    request: &SearchRequest,
    candidates: Vec<LexicalCandidate>,
) -> Vec<ScoredCandidate> {
    if candidates.is_empty() {
        return Vec::new();
    }

    candidates
        .into_iter()
        .map(|candidate| {
            let lexical_base = 1.0 / (1.0 + candidate.lexical_raw.abs());
            ...
            let final_score =
                lexical_base + keyword_bonus + importance_bonus + recency_bonus + emotion_bonus;

            ScoredCandidate {
                ...
                score: ScoreBreakdown { ... },
            }
        })
        .collect::<Vec<_>>()
}
```

**Phase 4 应用方式**

- `ValueConfig`、`ValueBreakdown`、`ScoredActionBranch` 应都放在 `src/cognition/value.rs`，不要散落到 `agent` 或 `interfaces`。
- Phase 4 的 value scoring 应保持纯 transform：
  - 输入 `WorkingMemory` 或 `Vec<ActionBranch>`
  - 输出 `Vec<ScoredActionBranch>`
  - 不直接做 DB / Rig / CLI IO
- 五维 value 直接仿照 `ScoreBreakdown` 明确成字段：
  - `goal_progress`
  - `information_gain`
  - `risk_avoidance`
  - `resource_efficiency`
  - `agent_robustness`
  - `final_score`
- 线性加权聚合放进单独 helper，例如 `aggregate_value(&ValueConfig, &ValueBreakdown) -> f32`。
- 这能给后续 multiplicative 升级留下稳定合同：以后只换聚合函数，不必改 `ValueBreakdown` 和 `ScoredActionBranch` 类型。

**锁定建议**

- `ValueConfig` 用 typed struct，不要 `HashMap<String, f32>`。
- 默认权重可以 `impl Default`，但真正执行时通过 request 注入或 assembler/agent service 传入，保持动态可配。
- 排序和裁剪留在调用方，不要在 `value.rs` 里偷偷决定“最终动作”；它只负责比较分数。

---

### `src/cognition/metacog.rs`（service, request-response）

**Analog 1:** `src/memory/truth.rs:70-141`, `src/memory/truth.rs:179-279`

**要复制的枚举/typed outcome 风格**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewGateState {
    Pending,
    Passed,
    Rejected,
}

impl ReviewGateState {
    pub fn as_str(self) -> &'static str { ... }
    pub fn parse(value: &str) -> Option<Self> { ... }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TruthRecord {
    T1 { base: MemoryRecord },
    T2 { base: MemoryRecord, open_candidates: Vec<OntologyCandidate> },
    T3 { base: MemoryRecord, t3_state: Option<T3State>, open_reviews: Vec<PromotionReview> },
}
```

**Analog 2:** `src/memory/governance.rs:98-141`, `src/memory/governance.rs:154-308`, `src/memory/governance.rs:376-470`

**要复制的 error + guard + report 结构**

```rust
#[derive(Debug, Error)]
pub enum TruthGovernanceError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error("source record {record_id} was not found")]
    SourceRecordNotFound { record_id: String },
    #[error("promotion review {review_id} has no attached evidence")]
    MissingEvidence { review_id: String },
    #[error("promotion review {review_id} has not passed {gate:?}")]
    GateNotPassed { review_id: String, gate: PromotionGate },
}

pub fn approve_promotion(
    &self,
    request: ApprovePromotionRequest,
) -> Result<PromotionApprovalReport, TruthGovernanceError> {
    let mut review = self.require_open_review(&request.review_id)?;
    let source_record = self.require_active_t3_source(&review.source_record_id)?;
    let evidence = self.repository.list_promotion_evidence(&review.review_id)?;
    if evidence.is_empty() {
        return Err(TruthGovernanceError::MissingEvidence { ... });
    }
    for gate in [ ... ] {
        if !gate_is_passed(&review, gate) {
            return Err(TruthGovernanceError::GateNotPassed { ... });
        }
    }
    ...
}
```

**Phase 4 应用方式**

- `MetacognitionService` 应该沿用 `TruthGovernanceService` 的风格：
  - request struct 入参
  - typed report 出参
  - `thiserror` 错误枚举
  - 私有 `require_*` / `gate_*` helper
- warning / veto / escalate 都必须是结构化输出，而不是日志副作用。
- 推荐类型拆分：
  - `MetacogWarning`
  - `VetoKind`：`hard` / `soft`
  - `EscalationDecision`
  - `MetacogReport`
- `soft veto` 的实现位置应该在 metacognition 层：它返回“需插入 regulative branch”的指令，由上游 orchestrator 把该 branch 注入 working memory。
- `hard veto` 则返回预定义 safe response 所需的 typed 信息，不要在 Rig adapter 里临时拼字符串。
- metacognition 不应直接调用检索；它只看 working memory、scored branches、truth/governance flags。

**锁定建议**

- 不要把 `warning`、`veto`、`escalate` 混成一个布尔值。
- 不要把 metacog 结果塞回 `TruthGovernanceService`；Phase 3 的 `metacog_approval_state` 是治理 gate 痕迹，Phase 4 的 metacognition 是运行时控制逻辑。

---

### `src/agent/mod.rs`（config, transform）

**Analog:** `src/memory/mod.rs:1-4`

**要复制的最小导出模式**

```rust
pub mod governance;
pub mod record;
pub mod repository;
pub mod truth;
```

**Phase 4 应用方式**

- `src/agent/mod.rs` 只导出 `orchestration` 和 `rig_adapter`。
- 不要把 cognition 类型重新定义一遍；agent 层只引用 `crate::cognition::*` 和既有 `crate::search::*` / `crate::memory::*`。

---

### `src/agent/orchestration.rs`（service, request-response）

**Analog 1:** `src/memory/governance.rs:143-374`

**要复制的“内部多步编排，但不越权下沉 SQL”的模式**

```rust
pub struct TruthGovernanceService<'db> {
    repository: MemoryRepository<'db>,
}

impl<'db> TruthGovernanceService<'db> {
    pub fn new(conn: &'db Connection) -> Self {
        Self {
            repository: MemoryRepository::new(conn),
        }
    }

    pub fn create_promotion_review(
        &self,
        request: CreatePromotionReviewRequest,
    ) -> Result<PromotionReviewReport, TruthGovernanceError> {
        ...
        self.require_active_t3_source(&request.source_record_id)?;
        ...
        self.repository.insert_promotion_review(&review)?;
        ...
    }
}
```

**Analog 2:** `src/interfaces/cli.rs:268-282`

**要复制的“构造 request 后调用内部 service”模式**

```rust
fn search_command(app: &AppContext, command: SearchCommand) -> Result<ExitCode> {
    let db = Database::open(app.db_path())?;
    let service = SearchService::new(db.conn());
    let response = service.search(
        &SearchRequest::new(command.query)
            .with_limit(command.top_k)
            .with_filters(SearchFilters { ... }),
    )?;
    ...
}
```

**Phase 4 应用方式**

- `AgentSearchOrchestrator` / `AgentSearchService` 应该是 Phase 4 的内部 orchestration 核心，不是 Rig 本身。
- 其依赖顺序推荐是：
  - `SearchService`
  - `TruthGovernanceService` 或 `MemoryRepository` 的只读 truth seam
  - `WorkingMemoryAssembler`
  - `ValueScorer`
  - `MetacognitionService`
- 推荐返回 `AgentSearchReport`，其中显式包含：
  - `working_memory`
  - `selected_branch`
  - `all_branches`
  - `metacog_report`
  - `citations`
  - `trace`
- 这层可以做多步 retrieval，但每一步都必须调用既有 `SearchService.search`，而不是自己拼 FTS SQL。
- 这层不得直接写入 shared truth 或 governance 表；Phase 4 只做 orchestration 和 decision support。

**锁定建议**

- 候选动作生成、value scoring、veto 语义都留在内部 service；`orchestration.rs` 只负责编排，不把认知逻辑交给 Rig。
- 如果需要对外提供“普通回答/安全回答/升级人工”三类终态，也要先形成 typed decision，再在更外层做渲染。

---

### `src/agent/rig_adapter.rs`（provider, request-response）

**Analog:** `src/interfaces/cli.rs:93-158`, `src/interfaces/cli.rs:234-285`

**要复制的薄适配层模式**

```rust
pub fn run(cli: Cli, config: Config) -> Result<ExitCode> {
    let app = AppContext::load(config)?;

    match cli.command {
        Commands::Search { ... } => search_command(&app, SearchCommand { ... }),
        ...
    }
}

fn search_command(app: &AppContext, command: SearchCommand) -> Result<ExitCode> {
    let db = Database::open(app.db_path())?;
    let service = SearchService::new(db.conn());
    let response = service.search(&SearchRequest::new(command.query) ... )?;
    ...
}
```

**Phase 4 应用方式**

- `src/agent/rig_adapter.rs` 应只负责把 Rig tool / agent callback 映射到 `AgentSearchOrchestrator` / `AgentSearchService`。
- 它的职责上限应类似 CLI：
  - 解析外部输入
  - 构造内部 request
  - 调内部 service
  - 把 typed report 转为 Rig 需要的输出
- 不要在这里放：
  - candidate generation
  - working-memory assembly
  - metacognitive veto 规则
  - truth write-back
- 当前代码库没有 Rig 直接类比文件，所以这里是 **partial** 匹配：实现时主要复制 CLI 的薄包装边界，而不是复制具体业务。

---

### `tests/agent_search.rs`（test, request-response）

**Analog 1:** `tests/retrieval_cli.rs:105-232`

**要复制的“真实 SQLite + 真实 service + 结构化断言”模式**

```rust
#[test]
fn library_search_returns_citations_and_filter_trace() {
    let path = fresh_db_path("library-shape");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    ...
    let search = SearchService::new(db.conn());
    let request = SearchRequest::new("lexical retrieval citations")
        .with_limit(5)
        .with_filters(SearchFilters { ... });

    let response = search.search(&request).expect("library retrieval should succeed");
    assert_eq!(response.results.len(), 1);
    assert_eq!(result.trace.applied_filters, response.applied_filters);
}
```

**Analog 2:** `tests/truth_governance.rs:154-283`, `tests/truth_governance.rs:286-340`

**要复制的“多 gate 流程 + 失败分支也要测”模式**

```rust
let created = service.create_promotion_review(...).expect("review should create");
assert_eq!(created.review.result_trigger_state, ReviewGateState::Pending);

let first_error = service
    .approve_promotion(...)
    .expect_err("approval should fail before gates pass");
assert!(matches!(
    first_error,
    TruthGovernanceError::GateNotPassed { ... }
));
```

**Phase 4 应用方式**

- 工作记忆和 agent search 测试优先用集成测试，不要只写纯单元测试。
- 最少覆盖三类断言：
  - `working_memory` 组装后保留 citations / truth-layer / risk flags
  - `value` 能把 epistemic / instrumental / regulative branch 放进同一比较空间
  - `metacog` 能产生 warning、soft veto、hard veto、escalate 的可观测结果
- Rig 相关测试优先验证“是否仍通过内部 service 边界”，不要在首版就绑定具体模型输出。

## Shared Patterns

### 1. 保留 ordinary retrieval 契约，不改成 agent 私有返回

**Source:** `src/search/mod.rs:52-90`, `src/search/rerank.rs:15-49`, `02-03-SUMMARY.md`

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchResponse {
    pub applied_filters: AppliedFilters,
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchResult {
    pub record: MemoryRecord,
    pub snippet: String,
    pub citation: Citation,
    pub score: ScoreBreakdown,
    pub trace: ResultTrace,
}
```

**Apply to:** `src/cognition/working_memory.rs`, `src/agent/orchestration.rs`, `src/agent/rig_adapter.rs`

- Phase 4 只能消费这个合同，不能把它降级成 `Vec<String>` 或 LLM prompt blob。
- working memory 的 evidence 槽位应保留对 `SearchResult` 的可追溯引用。
- 如果需要摘要字段，新增 summary 字段即可，不要覆盖 citation/trace。

### 2. 认知控制逻辑走 service 层，SQLite 继续留在 repository seam 后面

**Source:** `src/memory/governance.rs:143-374`, `src/memory/repository.rs:34-413`, `03-02-SUMMARY.md`, `03-03-SUMMARY.md`

```rust
pub struct TruthGovernanceService<'db> {
    repository: MemoryRepository<'db>,
}
```

**Apply to:** `src/cognition/metacog.rs`, `src/agent/orchestration.rs`

- Phase 4 新服务可以协调多个已有 service，但不要新增直接写 SQL 的旁路。
- 读 truth-layer 状态时，优先复用 `TruthRecord` 和 governance queue 接口，而不是另造 raw row DTO。
- 这也是 `AGT-04` 的关键约束。

### 3. 新的打分与 gate 结果都必须是 typed、可序列化、可测试的

**Source:** `src/search/score.rs:8-70`, `src/memory/truth.rs:127-279`

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScoreBreakdown { ... }
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionReview { ... }
```

**Apply to:** `src/cognition/value.rs`, `src/cognition/metacog.rs`, `src/agent/orchestration.rs`

- warning、veto、escalate、value breakdown 都要是 typed struct / enum。
- 不要把控制层结果只放到 `tracing` 或字符串 explanation。

### 4. 外部适配层必须保持薄，Rig 只能编排不能夺权

**Source:** `src/interfaces/cli.rs:93-158`, `src/interfaces/cli.rs:234-285`, `04-CONTEXT.md`

**Apply to:** `src/agent/rig_adapter.rs`

- Rig adapter 负责接入，不负责认知核心。
- 内部真正的业务入口应该是 `AgentSearchService`，Rig 只是调用者之一。
- 这样以后 CLI / MCP / HTTP 都能共享同一个 agent-search 核心。

## No Exact Analog Found

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `src/agent/rig_adapter.rs` | provider | request-response | 仓库里还没有任何 Rig 或 LLM 适配代码；只能复制 `src/interfaces/cli.rs` 的薄边界模式，不能从现有代码里找到更贴近的实现。 |

## Preferred Phase 4 Choices

- `WorkingMemory` 放在 `src/cognition/working_memory.rs`，采用 `SearchRequest/SearchResponse/SearchService` 那种 typed contract + 单入口 assembler 模式；builder 只用于内部组装，不对外暴露松散 JSON。
- `ValueConfig` 和五维 value breakdown 全放 `src/cognition/value.rs`，照 `src/search/score.rs` 做纯 transform；聚合函数单独抽出来，便于后续替换线性组合而不破坏类型合同。
- `MetacognitionService` 放在 `src/cognition/metacog.rs`，照 `TruthGovernanceService` 做 request/report/error/guard 分层；warning、soft veto、hard veto、escalate 全都结构化。
- `AgentSearchOrchestrator` / `AgentSearchService` 放在 `src/agent/orchestration.rs` 做内部编排核心；`src/agent/rig_adapter.rs` 只做薄 Rig adapter。这样最符合当前仓库已经形成的“ordinary retrieval 保持稳定，认知逻辑内部化，外部接口只包一层”的模式。

## Metadata

**Analog search scope:** `src/search/*.rs`, `src/memory/*.rs`, `src/interfaces/cli.rs`, `tests/*.rs`, Phase 2/3 summaries and prior pattern maps
**Files scanned:** 19
**Pattern extraction date:** 2026-04-15
