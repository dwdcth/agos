# Phase 3: Truth Layer Governance - Pattern Map

**Mapped:** 2026-04-15
**Files analyzed:** 10
**Analogs found:** 10 / 10

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `migrations/0004_truth_layer_governance.sql` | migration | transform | `migrations/0002_ingest_foundation.sql` | exact |
| `src/core/migrations.rs` | config | batch | `src/core/migrations.rs` | exact |
| `src/memory/record.rs` | model | transform | `src/memory/record.rs` | exact |
| `src/memory/truth.rs` | model | transform | `src/memory/record.rs` | role-match |
| `src/memory/repository.rs` | store | CRUD | `src/memory/repository.rs` | exact |
| `src/memory/governance.rs` | service | request-response | `src/ingest/mod.rs` | role-match |
| `src/search/filter.rs` | model | transform | `src/search/filter.rs` | exact |
| `src/search/lexical.rs` | service | request-response | `src/search/lexical.rs` | exact |
| `tests/foundation_schema.rs` | test | batch | `tests/foundation_schema.rs` | exact |
| `tests/retrieval_cli.rs` / `tests/truth_governance.rs` | test | request-response | `tests/retrieval_cli.rs` | role-match |

## Pattern Assignments

### `migrations/0004_truth_layer_governance.sql`（migration, transform）

**Analog:** `migrations/0002_ingest_foundation.sql:1-12`

**要复制的迁移风格：对 `memory_records` 做加法，不重建 authority store**

```sql
ALTER TABLE memory_records ADD COLUMN chunk_index INTEGER;
ALTER TABLE memory_records ADD COLUMN chunk_count INTEGER;
ALTER TABLE memory_records ADD COLUMN chunk_anchor_json TEXT;
ALTER TABLE memory_records ADD COLUMN content_hash TEXT;
ALTER TABLE memory_records ADD COLUMN valid_from TEXT;
ALTER TABLE memory_records ADD COLUMN valid_to TEXT;

CREATE INDEX IF NOT EXISTS idx_memory_records_source_chunk_order
    ON memory_records(source_uri, chunk_index, recorded_at DESC);
```

**Phase 3 应用方式**

- Truth layer 需要被检索和过滤直接看到的字段，优先继续放在 `memory_records` 上，沿用 Phase 2 的 additive schema 模式。
- 不要重写 `memory_records_fts`，也不要把治理信息塞进 FTS sidecar。
- 适合加在 `memory_records` 上的字段是“检索语义必须直接读到”的字段，例如 T3 的 `confidence`、`revocability`、共享可见性或治理状态摘要。
- 适合拆到新表的是“流程记录”而不是“记录当前态”，例如：
  - `truth_promotion_reviews`
  - `truth_promotion_evidence`
  - `truth_ontology_candidates`

**推荐建表方式**

- `memory_records` 继续保存当前记录态。
- 新表只通过 `record_id` 关联，不复制 `content_text`，避免 authority 分裂。
- 审查细节可保留 `*_json` 字段，但可查询状态字段必须单独成列。

---

### `src/core/migrations.rs`（config, batch）

**Analog:** `src/core/migrations.rs:4-23`

**要复制的注册模式**

```rust
const FOUNDATION_SCHEMA_SQL: &str = include_str!("../../migrations/0001_foundation.sql");
const INGEST_FOUNDATION_SQL: &str = include_str!("../../migrations/0002_ingest_foundation.sql");
const LEXICAL_SIDECAR_SQL: &str = include_str!("../../migrations/0003_lexical_sidecar.sql");

Migrations::new(vec![
    M::up(FOUNDATION_SCHEMA_SQL)
        .foreign_key_check()
        .comment("foundation schema bootstrap"),
    M::up(INGEST_FOUNDATION_SQL)
        .foreign_key_check()
        .comment("ingest authority metadata"),
    M::up(LEXICAL_SIDECAR_SQL)
        .foreign_key_check()
        .comment("lexical fts sidecar"),
])
```

**Phase 3 应用方式**

- Phase 3 migration 继续按独立 `M::up(...)` 注册，不修改前 3 个 migration 的内容。
- comment 名称应清楚标出“truth governance”而不是泛化成 search/agent。
- 这是保证 Phase 2 retrieval 不回归的关键模式。

---

### `src/memory/record.rs`（model, transform）

**Analog:** `src/memory/record.rs:3-15`, `src/memory/record.rs:115-147`, `src/memory/record.rs:174-178`

**要复制的基础模型风格：主记录结构稳定，扩展字段 typed 化**

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub source: SourceRef,
    pub timestamp: RecordTimestamp,
    pub scope: Scope,
    pub record_type: RecordType,
    pub truth_layer: TruthLayer,
    pub provenance: Provenance,
    pub content_text: String,
    pub chunk: Option<ChunkMetadata>,
    pub validity: ValidityWindow,
}
```

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TruthLayer {
    T1,
    T2,
    T3,
}

impl TruthLayer {
    pub fn as_str(self) -> &'static str { ... }
    pub fn parse(value: &str) -> Option<Self> { ... }
}
```

**Phase 3 应用方式**

- 不要把所有治理字段直接塞进 `Provenance` JSON；`MemoryRecord` 仍然是 authority row 的 typed 外观。
- 若治理字段直接跟随记录生命周期并被 query/filter 使用，优先作为明确 typed 字段挂在 `MemoryRecord` 上，再在 repository 边界序列化。
- 推荐把新增治理模型拆到新文件 `src/memory/truth.rs`，但保留 `MemoryRecord` 只承载“当前态”字段，避免 `record.rs` 膨胀成工作流模块。

**优先保留在 `MemoryRecord` 当前态上的内容**

- `truth_layer`
- `t3_confidence`
- `t3_revocability`
- `governance_status_summary` 或等价的可查询摘要

**不建议直接塞进 `MemoryRecord` 主体的大块流程内容**

- 审核证据列表
- 审批事件历史
- T2 -> T1 的结构审查详情

---

### `src/memory/truth.rs`（model, transform）

**Analog:** `src/memory/record.rs:115-147`, `src/memory/record.rs:174-178`

**要复制的枚举/小结构模式**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TruthLayer {
    T1,
    T2,
    T3,
}

impl TruthLayer {
    pub fn as_str(self) -> &'static str { ... }
    pub fn parse(value: &str) -> Option<Self> { ... }
}
```

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidityWindow {
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
}
```

**Phase 3 应用方式**

- 新文件适合放：
  - `T3Metadata`
  - `PromotionReviewStatus`
  - `PromotionGate`
  - `OntologyCandidateStatus`
  - `ReviewOutcome`
- 保持和 `record.rs` 一样的风格：小而清晰的 enum/struct，带 `as_str` / `parse`，而不是 free-form string。
- T3 文档要求的结构可以直接映射成 typed 模型，但不要把 Phase 4 元认知行为提前实现成 service。

**推荐模型拆分**

- `T3Metadata`: `confidence`, `revocability`, `shared_conflict_note`
- `PromotionGate`: `result_trigger`, `evidence_review`, `consensus_check`, `metacog_approval`
- `T2ToT1Candidate`: `stability`, `reproducibility`, `invariance`, `predictive_usefulness`, `structural_review`

---

### `src/memory/repository.rs`（store, CRUD）

**Analog:** `src/memory/repository.rs:39-100`, `src/memory/repository.rs:102-235`, `src/memory/repository.rs:264-290`

**要复制的写入模式：SQL 列明确展开，JSON 只在边界序列化**

```rust
pub fn insert_record(&self, record: &MemoryRecord) -> Result<(), RepositoryError> {
    let provenance_json = serde_json::to_string(&record.provenance)?;
    let chunk_anchor_json = record
        .chunk
        .as_ref()
        .map(|chunk| serde_json::to_string(&chunk.anchor))
        .transpose()?;

    self.conn.execute(
        r#"
        INSERT INTO memory_records (
            id, source_uri, source_kind, source_label, recorded_at,
            scope, record_type, truth_layer, provenance_json, content_text,
            chunk_index, chunk_count, chunk_anchor_json, content_hash,
            valid_from, valid_to, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
        "#,
        params![ ... ],
    )?;

    Ok(())
}
```

**要复制的读取模式：统一 row mapper，严格报错**

```rust
fn map_record_row(row: &rusqlite::Row<'_>) -> Result<MemoryRecord, RepositoryError> {
    let truth_layer = row.get::<_, String>(7)?;
    let provenance_json = row.get::<_, String>(8)?;

    Ok(MemoryRecord {
        truth_layer: parse_truth_layer(&truth_layer)?,
        provenance: serde_json::from_str::<Provenance>(&provenance_json)?,
        ...
    })
}
```

```rust
fn parse_truth_layer(value: &str) -> Result<TruthLayer, RepositoryError> {
    TruthLayer::parse(value).ok_or_else(|| RepositoryError::InvalidEnum {
        field: "truth_layer",
        value: value.to_string(),
    })
}
```

**Phase 3 应用方式**

- repository 继续拥有所有 SQLite 读写和 row mapping，不把 SQL 下沉到 `search` 或 `interfaces`。
- 新增治理表时，优先沿用：
  - `insert_*`
  - `get_*`
  - `list_*`
  - `map_*_row`
  这组对称方法。
- 对 promotion/candidate 这种流程记录，推荐新增专门 repository API，而不是复用 `provenance_json` 做半结构化存储。
- 如果某个状态需要被 lexical search 过滤，必须保证 repository 与 lexical row hydration 能读到同一套 typed enum。

**具体落点建议**

- `MemoryRepository` 继续管理 `memory_records` 当前态。
- 在同文件追加或新建子 repository 结构都可以，但更推荐同模块下新增：
  - `TruthGovernanceRepository`
  让 `MemoryRepository` 不被 promotion/candidate SQL 继续放大。

---

### `src/memory/governance.rs`（service, request-response）

**Analog:** `src/ingest/mod.rs:24-57`, `src/ingest/mod.rs:59-124`

**要复制的 service 结构**

```rust
#[derive(Debug, Error)]
pub enum IngestError {
    #[error(transparent)]
    Normalize(#[from] NormalizeError),
    #[error(transparent)]
    Persist(#[from] RepositoryError),
}

pub struct IngestService<'db> {
    repository: MemoryRepository<'db>,
    chunk_config: ChunkConfig,
}

impl<'db> IngestService<'db> {
    pub fn new(conn: &'db Connection) -> Self { ... }

    pub fn ingest(&self, request: IngestRequest) -> Result<IngestReport, IngestError> {
        ...
        self.repository.insert_record(&record)?;
        ...
    }
}
```

**Phase 3 应用方式**

- Governance service 应该只做“流程编排 + 规则检查”，不要接管底层 SQL。
- 推荐新建 `TruthGovernanceService<'db>`，内部组合：
  - `MemoryRepository`
  - `TruthGovernanceRepository`
- 请求/响应也沿用 typed request/report 风格，例如：
  - `PromoteT3Request`
  - `PromotionReviewReport`
  - `CreateOntologyCandidateRequest`
  - `CandidateReport`

**边界要求**

- 只记录 `metacog_approval` 状态，不实现元认知推理本身。
- 只创建 T2 -> T1 candidate，不直接改写 T1 结构。
- 这是 Phase 3 和 Phase 4/5 的清晰分界。

---

### `src/search/filter.rs`（model, transform）

**Analog:** `src/search/filter.rs:5-29`

**要复制的 typed filter 模式**

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SearchFilters {
    pub scope: Option<Scope>,
    pub record_type: Option<RecordType>,
    pub truth_layer: Option<TruthLayer>,
    pub valid_at: Option<String>,
    pub recorded_from: Option<String>,
    pub recorded_to: Option<String>,
}

impl SearchFilters {
    pub fn truth_layer_value(&self) -> Option<&'static str> {
        self.truth_layer.map(TruthLayer::as_str)
    }
}
```

**Phase 3 应用方式**

- 不要推翻 `truth_layer: Option<TruthLayer>`；在此基础上增加治理感知的 filter。
- 推荐新增的是“治理语义过滤”，不是替换 Phase 2 过滤：
  - `shared_only`
  - `include_revoked`
  - `promotion_status`
  - `candidate_status`
- 对 planner 最稳的选择是保留旧字段兼容，再新增独立字段，而不是重定义 `truth_layer` 的含义。

**推荐语义**

- `truth_layer = T2` 仍表示“只看 T2 记录”。
- “只看共享真值”不要偷偷重写成 `truth_layer != T3`，应该单独有治理语义字段或 helper。
- 这样可以保住 Phase 2 CLI / library 的兼容性。

---

### `src/search/lexical.rs`（service, request-response）

**Analog:** `src/search/lexical.rs:15-49`, `src/search/lexical.rs:51-85`, `src/search/lexical.rs:120-228`, `src/search/lexical.rs:231-317`

**要复制的 SQL 过滤前置模式**

```rust
WHERE memory_records_fts MATCH jieba_query(?1)
  AND (?3 IS NULL OR mr.scope = ?3)
  AND (?4 IS NULL OR mr.record_type = ?4)
  AND (?5 IS NULL OR mr.truth_layer = ?5)
  AND (?6 IS NULL OR mr.valid_from IS NULL OR mr.valid_from <= ?6)
  AND (?6 IS NULL OR mr.valid_to IS NULL OR mr.valid_to >= ?6)
  AND (?7 IS NULL OR mr.recorded_at >= ?7)
  AND (?8 IS NULL OR mr.recorded_at <= ?8)
ORDER BY bm25(memory_records_fts), mr.recorded_at DESC, mr.id ASC
LIMIT ?2
```

**要复制的 recall filter 打包模式**

```rust
#[derive(Debug, Clone, Copy)]
struct RecallFilters<'a> {
    scope: Option<&'a str>,
    record_type: Option<&'a str>,
    truth_layer: Option<&'a str>,
    valid_at: Option<&'a str>,
    recorded_from: Option<&'a str>,
    recorded_to: Option<&'a str>,
}
```

**Phase 3 应用方式**

- 继续坚持“filter-first lexical recall”，治理过滤也优先在 SQL 层完成。
- Phase 3 不要改变 FTS candidate recall 的主流程，只在 `JOIN` / `EXISTS` 上加治理条件。
- 推荐做法：
  - `memory_records` 上的治理当前态字段，直接 `mr.xxx = ?`
  - promotion/candidate side table，用 `EXISTS (SELECT 1 ...)` 约束
- 不要先把所有 lexical results 拉出来再在 Rust 里按 promotion/candidate 状态做二次丢弃，这会破坏 Phase 2 的 auditable filtering 模式。

**保兼容要求**

- `SELECT` 主体仍以 `mr.*` 为中心，确保 `SearchResult.record` 的 hydration 不被打散。
- 不要把治理状态作为 snippet 或 citation 来源；citation 仍只从 persisted chunk/source metadata 构造。

---

### `src/search/mod.rs`（service, request-response）

**Analog:** `src/search/mod.rs:19-49`, `src/search/mod.rs:75-90`

**要复制的 request/service 外观**

```rust
pub struct SearchRequest {
    pub query: String,
    pub limit: usize,
    pub filters: SearchFilters,
}

pub struct SearchService<'db> {
    lexical: lexical::LexicalSearch<'db>,
}

pub fn search(&self, request: &SearchRequest) -> Result<SearchResponse, SearchError> {
    let candidates = self.lexical.recall(request)?;
    let scored = score::score_candidates(request, candidates);
    Ok(rerank::rerank_results(request, scored)?)
}
```

**Phase 3 应用方式**

- Search surface 继续薄；truth governance 不应让 `SearchService` 变成工作流服务。
- 可增加治理语义的 `AppliedFilters` / `ResultTrace`，但不要把 promotion 审批报告混进普通 retrieval result。
- 普通 retrieval 仍是“查记录”，不是“执行晋升”。

---

### `tests/foundation_schema.rs`（test, batch）

**Analog:** `tests/foundation_schema.rs:137-177`, `tests/foundation_schema.rs:179-219`, `tests/foundation_schema.rs:251-304`

**要复制的 schema/test 风格**

```rust
assert_eq!(db.schema_version().expect("schema version"), 3);

assert!(
    columns.contains(&"chunk_index".to_string())
        && columns.contains(&"chunk_count".to_string())
        && columns.contains(&"chunk_anchor_json".to_string()),
    "memory_records should expose additive ingest columns: {columns:?}"
);
```

```rust
let loaded = repo
    .get_record(&record.id)
    .expect("record lookup should succeed")
    .expect("record should exist");

assert_eq!(loaded, record);
```

**Phase 3 应用方式**

- 继续验证“schema additive + old retrieval objects still exist”。
- Phase 3 的 schema test 至少覆盖：
  - `memory_records` 新列存在
  - 新治理表存在
  - `memory_records_fts` 和 3 个 trigger 仍存在
  - 老的 row round-trip 不丢字段
- 这类测试是“Phase 2 不回归”的主防线。

---

### `tests/retrieval_cli.rs` / `tests/truth_governance.rs`（test, request-response）

**Analog:** `tests/retrieval_cli.rs:76-103`, `tests/retrieval_cli.rs:105-229`

**要复制的 library-first integration 风格**

```rust
let request = SearchRequest::new("lexical retrieval citations")
    .with_limit(5)
    .with_filters(SearchFilters {
        scope: Some(Scope::Project),
        record_type: Some(RecordType::Decision),
        truth_layer: Some(TruthLayer::T2),
        valid_at: Some("2026-04-15T12:00:00Z".to_string()),
        recorded_from: Some("2026-04-10T00:00:00Z".to_string()),
        recorded_to: Some("2026-04-16T00:00:00Z".to_string()),
    });

let response = search.search(&request).expect("library retrieval should succeed");
assert_eq!(response.results.len(), 1, "filters should narrow results in SQL");
```

**Phase 3 应用方式**

- Governance tests 也应先走 library API，而不是只测 CLI。
- 重点新增断言：
  - T3 记录在默认 shared-only 语义下不会污染共享查询
  - 指定 `truth_layer = T3` 或 `include_revoked` 时能被显式查出
  - 未通过 gate 的 T3 不会伪装成 T2
  - T2 -> T1 只生成 candidate，不改变 record 的 `truth_layer`

**适合新建的测试文件**

- `tests/truth_governance.rs`
  - repository round-trip
  - promotion gate state transitions
  - search filter semantics with governance-aware filters

---

## Shared Patterns

### 1. Authority Store First

**Sources**

- `migrations/0001_foundation.sql:1-20`
- `migrations/0002_ingest_foundation.sql:1-12`
- `migrations/0003_lexical_sidecar.sql:1-26`
- `tests/foundation_schema.rs:179-219`

**Apply to**

- 所有 Truth Layer Governance 设计

**Concrete pattern**

- `memory_records` 仍是 authority store。
- schema 只做 additive evolution。
- FTS sidecar 只索引 `source_label` / `content_text`，不成为真值治理的 authority。

**Planner guidance**

- 当前态字段靠近 `memory_records`。
- 流程态字段拆 side table。
- 不要引入第二个“主记忆表”。

### 2. Typed Enum Across SQL Boundary

**Sources**

- `src/memory/record.rs:24-147`
- `src/memory/repository.rs:204-290`
- `src/search/filter.rs:17-28`

**Apply to**

- 新的 truth status、promotion status、candidate status

**Concrete pattern**

```rust
pub fn as_str(self) -> &'static str { ... }
pub fn parse(value: &str) -> Option<Self> { ... }
```

```rust
Enum::parse(value).ok_or_else(|| RepositoryError::InvalidEnum { ... })
```

**Planner guidance**

- 所有可查询状态都做 typed enum。
- 不要把 `status = "approved"` 这种字符串散落在 SQL 和 service 层。

### 3. Filter First, Score Second

**Sources**

- `src/search/lexical.rs:15-85`
- `src/search/lexical.rs:135-228`
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-02-SUMMARY.md`
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md`

**Apply to**

- governance-aware retrieval semantics

**Concrete pattern**

- SQL recall 先做 scope/type/truth/time 过滤。
- Rust scorer 只对已通过过滤的 candidates 做排序与解释。

**Planner guidance**

- promotion/candidate/revocable filters 也应在 recall SQL 中表达。
- 不要把治理语义推迟到 rerank 之后。

### 4. Thin Service Over Repository

**Sources**

- `src/ingest/mod.rs:46-124`
- `src/search/mod.rs:67-90`

**Apply to**

- `TruthGovernanceService`

**Concrete pattern**

- Service 负责 request/report 和规则编排。
- Repository 负责 SQL 和 hydration。

**Planner guidance**

- Promotion gate、candidate creation、review listing 放 service。
- SQLite 细节不应进 CLI 或未来 agent 层。

### 5. Compatibility Tests Are Part Of The Pattern

**Sources**

- `tests/foundation_schema.rs:137-249`
- `tests/retrieval_cli.rs:105-229`

**Apply to**

- Phase 3 migration、repository 和 search 变更

**Concrete pattern**

- 每个 phase 都保留前一 phase 的 contract，并通过测试显式断言。

**Planner guidance**

- Phase 3 测试必须同时证明：
  - 新治理字段可用
  - Phase 2 lexical retrieval/citation/filter contract 仍然成立

## Pattern Decisions For Phase 3

### Truth metadata 放置位置

- **首选：** 把“查询必须直接解释或过滤”的 truth metadata 放在 `memory_records`。
- **原因：** Phase 2 的 lexical SQL 已直接 `JOIN memory_records AS mr` 并按 `mr.truth_layer`、`valid_from`、`valid_to` 过滤，Phase 3 继续沿这条路径最稳。
- **不建议：** 把 `confidence`、`revocability` 仅埋在 `provenance_json`，否则 search/repository/filter 会退化成 JSON-aware 特判。

### Governance logic 放置位置

- **首选：** `memory/` 内小幅拆分，而不是新建一个抢权的 `truth/` 顶层模块。
- **推荐结构：**
  - `src/memory/record.rs` 继续放 base record
  - `src/memory/truth.rs` 放治理 enum/struct
  - `src/memory/repository.rs` 或 `src/memory/governance_repository.rs` 放持久化
  - `src/memory/governance.rs` 放 promotion/candidate service
- **原因：** 当前 repo 里 truth layer 本来就是 memory domain 的一部分，search 只是消费这些 typed 语义。

### Promotion / candidate records 建模

- **首选：** dedicated side tables + typed records。
- **推荐拆法：**
  - `truth_promotion_reviews`
    - `record_id`
    - `status`
    - `result_triggered_at`
    - `evidence_status`
    - `consensus_status`
    - `metacog_status`
    - `review_payload_json`
  - `truth_ontology_candidates`
    - `record_id`
    - `status`
    - `stability_status`
    - `reproducibility_status`
    - `invariance_status`
    - `usefulness_status`
    - `structural_review_status`
    - `notes_json`
- **原因：** 这些是流程记录，不是 record 当前态；拆表更符合 queryability 和 auditability。

### Lexical retrieval 兼容性保持方式

- **首选：** 保持 `memory_records` + `memory_records_fts` 结构不变，只在 recall SQL 上增加治理过滤条件。
- **原因：** Phase 2 已明确采用 “authority-store lexical indexing” 和 “filter-first lexical recall”。
- **不建议：**
  - 修改 FTS content source
  - 把治理信息当 snippet/citation 来源
  - 新建第二套可检索主表

## No Exact Analog

| File / Concern | Role | Data Flow | Reason | Fallback Pattern |
|---|---|---|---|---|
| `src/memory/governance.rs` 的 promotion/candidate 业务规则 | service | request-response | 当前仓库还没有治理工作流 service | 复制 `src/ingest/mod.rs` 的 request/report/error/service 骨架 |
| `truth_promotion_reviews` / `truth_ontology_candidates` 的工作流表 | migration/store | CRUD | 当前仓库只有 authority rows，没有 review workflow 表 | 复制 `0002` 的 additive migration + `repository.rs` 的 typed row mapping |
| governance-aware 检索语义（shared/private/pending-review） | model/service | request-response | 目前只有 `truth_layer` 单字段过滤 | 复制 `SearchFilters` + `LexicalSearch` 的 typed filter + SQL recall 模式 |

## Metadata

**Analog search scope:** `src/core`, `src/memory`, `src/ingest`, `src/search`, `migrations`, `tests`, `.planning/phases/01-*`, `.planning/phases/02-*`, `doc/0415-真值层.md`

**Files scanned:** 20+

**Pattern extraction date:** 2026-04-15

## 推荐方案

1. 首选“`memory_records` 当前态 + governance side tables 流程态”的混合模式；不要重建 authority store，也不要把治理状态全塞进 JSON。
2. 首选把 truth governance 继续放在 `memory/` 域内，新增 `src/memory/truth.rs` 和 `src/memory/governance.rs`，由 `search` 只消费 typed filter 和 query semantics。
3. 首选在 `src/search/filter.rs` / `src/search/lexical.rs` 上做治理感知扩展，保持 Phase 2 的 lexical-first、SQL 先过滤、citation 不变这三条基线不动。
