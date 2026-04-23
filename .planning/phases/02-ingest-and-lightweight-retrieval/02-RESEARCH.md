# Phase 02: Ingest And Lightweight Retrieval - Research

**Researched:** 2026-04-15
**Domain:** Rust 本地优先 ingest 与 lexical-first ordinary retrieval
**Confidence:** MEDIUM

<user_constraints>
## User Constraints (from CONTEXT.md)

以下内容按 `02-CONTEXT.md` 原文复制。[VERIFIED: codebase grep]

### Locked Decisions
- Phase 2 must implement ordinary retrieval as a lexical-first path with no model files or embedding services required.
- The retrieval baseline is `libsimple` + SQLite FTS5 + Rust lightweight keyword/scoring rules.
- Ordinary retrieval must remain fully usable from CLI or library APIs without invoking Rig or any LLM.
- Ingest must preserve source linkage and chunk provenance as first-class metadata, not reconstruct them later.
- Retrieval results must include source, scope, timestamp or validity data, and enough trace detail to explain why each memory was returned.
- The three retrieval modes from Phase 1 (`lexical_only`, `embedding_only`, `hybrid`) remain part of the config contract, but Phase 2 only implements the lexical-first ordinary retrieval path.
- `embedding_only` and `hybrid` must not force this phase to implement semantic retrieval; they remain reserved extension semantics unless a plan explicitly scopes otherwise.
- If Phase 2 surfaces status/readiness for retrieval capabilities, lexical capability should become real while semantic capability can remain deferred or not built.
- Rust-side scoring should stay lightweight, inspectable, and deterministic: no opaque model-based reranking in this phase.
- The retrieval path should preserve the explainability rule: lexical remains the primary explanation source, and future semantic extensions must not erase that contract.

### Claude's Discretion
- Exact module/file split within `ingest/` and `search/`, as long as it builds cleanly on the current Phase 1 crate structure.
- Exact shape of chunk metadata and source identifiers, as long as provenance, source linkage, and time/scope filters remain explicit.
- Specific CLI command names for ingest and search, as long as they are thin wrappers over internal services.
- Exact scoring breakdown fields, as long as they remain inspectable and support citations/explanations.

### Deferred Ideas (OUT OF SCOPE)
- semantic retrieval execution and merge contracts
- Rig wiring and agent-search orchestration
- truth-layer governance and promotion gates
- working-memory assembly and metacognitive checks
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ING-01 | ingest notes, documents, and conversation-like text into normalized memory units [VERIFIED: `.planning/REQUIREMENTS.md`] | 采用 `detect -> normalize -> chunk -> persist` 管线，先只支持文本型输入与已导出的对话 JSON/JSONL。[VERIFIED: codebase grep; CITED: https://docs.rs/crate/libsimple/latest] |
| ING-02 | preserve source linkage and chunk provenance [VERIFIED: `.planning/REQUIREMENTS.md`] | 每个 chunk 必须带 `source_uri`、`source_kind`、`chunk_index`、`chunk_count` 与 anchor/span 元数据，不依赖事后重建。[VERIFIED: codebase grep; CITED: https://sqlite.org/fts5.html] |
| ING-03 | persist lexical and scoring metadata without embeddings [VERIFIED: `.planning/REQUIREMENTS.md`] | 以 `memory_records` 为权威存储，加 FTS5 sidecar 和可解释评分信号，不引入向量表或模型服务。[VERIFIED: codebase grep; CITED: https://sqlite.org/fts5.html] |
| RET-01 | lexical search over Chinese and PinYin using `libsimple`-backed SQLite FTS [VERIFIED: `.planning/REQUIREMENTS.md`] | 用 `libsimple::enable_auto_extension()` + `set_jieba_dict()` 初始化 SQLite 连接，并用 `tokenize='simple'` 建 FTS5 表。[CITED: https://docs.rs/crate/libsimple/latest] |
| RET-02 | Rust-side BM25/TF-IDF-style keyword weighting and context bonuses [VERIFIED: `.planning/REQUIREMENTS.md`] | 先让 FTS5 提供候选和 lexical 基分，再在 Rust 做小幅 bonus 组合；不自写 tokenizer 或全文索引。[CITED: https://sqlite.org/fts5.html; VERIFIED: codebase grep] |
| RET-03 | compose lexical, keyword, emotion, importance, recency into stable ranking [VERIFIED: `.planning/REQUIREMENTS.md`] | 采用 typed `ScoreBreakdown`，lexical 为主，缺失信号默认 `0.0`，避免 opaque rerank。[ASSUMED] |
| RET-04 | results include source, scope, timestamp/validity, and trace data [VERIFIED: `.planning/REQUIREMENTS.md`] | 结果契约必须显式返回 citation、filter trace、score breakdown 与 validity 字段。[VERIFIED: codebase grep] |
| RET-05 | filter by scope, record type, truth layer, and time validity [VERIFIED: `.planning/REQUIREMENTS.md`] | 过滤应在 SQL 候选阶段完成，避免先召回再在 Rust 大量丢弃。[CITED: https://sqlite.org/fts5.html; ASSUMED] |
| AGT-01 | ordinary retrieval usable without LLM or agent runtime [VERIFIED: `.planning/REQUIREMENTS.md`] | CLI 与 library API 直接调用 ingest/search service，不出现 Rig 依赖。[VERIFIED: codebase grep] |
</phase_requirements>

## Summary

Phase 2 最稳的实现方向，是把 Phase 1 的 `memory_records` 继续作为权威 chunk 存储，在其上追加文本型 ingest 管线、一个 `libsimple` 驱动的 SQLite FTS5 lexical 索引层，以及一个完全在 Rust 内部完成的轻量 `ScoreBreakdown` 组合层。[VERIFIED: codebase grep; CITED: https://docs.rs/crate/libsimple/latest; CITED: https://sqlite.org/fts5.html]

`reference/mempal` 对 Phase 2 最值得复用的是模块拆分思路，不是它的 hybrid/vector 方案；当前项目应复用 `detect/normalize/chunk/search` 这种边界，但把执行路径收敛为 lexical-first、no-LLM、no-embedding 的 ordinary retrieval。[VERIFIED: codebase grep]

**Primary recommendation:** 用 `memory_records` + external-content FTS5 + `libsimple` tokenizer 做召回，用 `ScoreBreakdown` 做 deterministic rerank，并把 citation/trace 作为一等返回字段；不要在 Phase 2 引入 Rig、embedding、RRF 或格式解析大而全。[VERIFIED: codebase grep; CITED: https://docs.rs/crate/libsimple/latest; CITED: https://sqlite.org/fts5.html]

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| CLI / library 请求解析 | Browser / Client | API / Backend | Phase 2 入口应保持为 `clap` CLI 与库 API 边界；解析参数后立即转入 service，不在入口层做检索逻辑。[VERIFIED: codebase grep; ASSUMED] |
| source detection / normalization / chunking | API / Backend | Database / Storage | 这是纯应用服务逻辑，输出稳定的 chunk draft 后再持久化。[VERIFIED: codebase grep] |
| lexical indexing and candidate recall | Database / Storage | API / Backend | FTS5/tokenizer/bm25 属于 SQLite 能力，服务层只负责建表、查询和结果映射。[CITED: https://sqlite.org/fts5.html; CITED: https://docs.rs/crate/libsimple/latest] |
| score composition / explainability / citation assembly | API / Backend | Database / Storage | 这是项目差异化逻辑，应在 Rust 内完成并保留 breakdown，而不是塞回 SQL 黑箱里。[VERIFIED: `.planning/PROJECT.md`; ASSUMED] |
| provenance / validity / truth metadata persistence | Database / Storage | API / Backend | 这些字段是 ordinary retrieval 的解释基础，必须在写入时就落盘。[VERIFIED: codebase grep; VERIFIED: `.planning/REQUIREMENTS.md`] |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `libsimple` | `0.9.0` [CITED: https://docs.rs/crate/libsimple/latest] | 中文/拼音 FTS5 tokenizer、`jieba_query`/`simple_query` 辅助 | Phase 2 的 locked baseline 已固定为 `libsimple` + FTS5，而且官方示例直接展示了 SQLite 连接初始化与 tokenizer 配置。[CITED: https://docs.rs/crate/libsimple/latest] |
| `rusqlite` | keep repo pin `0.37.0` [VERIFIED: codebase grep] | SQLite 连接、migration、参数绑定 | 当前仓库已启用 `bundled` SQLite；`libsimple 0.9.0` 文档声明兼容 `rusqlite >=0.32,<1.0`，所以 Phase 2 无需为研究结论引入额外版本升级风险。[VERIFIED: codebase grep; CITED: https://docs.rs/crate/libsimple/latest] |
| SQLite FTS5 | bundled with current `rusqlite` build [VERIFIED: codebase grep] | lexical candidate recall、`bm25`、`highlight`/`snippet` | FTS5 官方能力已经覆盖 candidate recall、排名函数和解释片段生成，不需要手写全文索引。[CITED: https://sqlite.org/fts5.html] |
| `serde_json` | keep repo pin `1.x` [VERIFIED: codebase grep] | provenance / chunk anchor / trace JSON 序列化 | Phase 1 已用它持久化 provenance，Phase 2 继续扩展 JSON 元数据最小、最稳。[VERIFIED: codebase grep] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `clap` | keep repo pin `4.x` [VERIFIED: codebase grep] | `ingest` / `search` CLI | 保持入口层薄，不另起 CLI 框架。[VERIFIED: codebase grep] |
| `tracing` | keep repo pin `0.1.x` [VERIFIED: codebase grep] | ingest/search trace 与调试日志 | 只记录 query plan、candidate counts、score breakdown 摘要，不记录敏感全文默认值。[VERIFIED: codebase grep; ASSUMED] |
| `regex` | optional [ASSUMED] | ASCII/pinyin token bonus、query 轻量切词 | 只有在 Rust bonus 需要独立 token 规则时再加；不要抢 FTS tokenizer 的职责。[ASSUMED] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `libsimple` lexical baseline | `sqlite-vec` / embedding recall | 这会违反 Phase 2 的 locked scope；语义检索只能保留扩展缝，不应成为本阶段前提。[VERIFIED: `.planning/PROJECT.md`; VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-CONTEXT.md`] |
| external-content FTS5 | duplicated content table | duplicated content 写入更简单，但会增加一致性和 explainability 的双写成本；Phase 2 更适合让 `memory_records` 继续做 authority。[CITED: https://sqlite.org/fts5.html; VERIFIED: codebase grep] |

**Installation:**
```bash
cargo add libsimple@0.9.0
# keep existing rusqlite = { version = "0.37", features = ["bundled"] }
```

**Version verification:** `libsimple` 当前 docs.rs latest 为 `0.9.0`，页面时间为 `2026-04-12`。[CITED: https://docs.rs/crate/libsimple/latest] 当前仓库已锁定 `rusqlite 0.37.0`、`clap 4`、`serde_json 1`、`tracing 0.1`。[VERIFIED: codebase grep]

## Architecture Patterns

### System Architecture Diagram

```text
CLI / library call
    |
    v
IngestRequest / SearchRequest
    |
    +--> ingest::detect --> ingest::normalize --> ingest::chunk
    |                                             |
    |                                             v
    |                                  chunk drafts + source anchors
    |                                             |
    |                                             v
    |                            memory_records (authority store)
    |                                             |
    |                                             +--> memory_records_fts (FTS5 sidecar)
    |
    +--> search::lexical --> FTS5 candidate recall --> search::score --> search::citation
                                                      |                    |
                                                      v                    v
                                                filter trace         citation/result contract
                                                      |
                                                      v
                                                 SearchResponse
```

### Recommended Project Structure

```text
src/
├── ingest/
│   ├── detect.rs        # 输入格式识别：plain text / document-like text / conversation export
│   ├── normalize.rs     # 统一成 NormalizedSource
│   ├── chunk.rs         # 按文本或 turn 生成 ChunkDraft
│   └── mod.rs           # IngestService / IngestRequest / IngestReport
├── search/
│   ├── lexical.rs       # FTS5/libsimple 建表、查询、candidate recall
│   ├── score.rs         # ScoreBreakdown 与 deterministic rerank
│   ├── citation.rs      # Citation / ChunkAnchor / trace rendering
│   └── mod.rs           # SearchService / SearchRequest / SearchResponse
├── memory/
│   ├── record.rs        # 扩展 chunk/source/validity typed metadata
│   └── repository.rs    # 权威持久化边界；不持有 FTS SQL
└── interfaces/
    └── cli.rs           # `ingest` / `search` 子命令，薄封装 service
```

### Pattern 1: Source-Normalized Ingest

**What:** 先把原始输入转成 `NormalizedSource`，再 chunk，而不是边读边直接写表。[VERIFIED: codebase grep]

**When to use:** 所有 `ingest path`、`ingest text`、conversation export 导入路径。[VERIFIED: `.planning/ROADMAP.md`]

**Example:**
```rust
// Source: adapted from reference/mempal ingest split and project Phase 1 models.
pub struct NormalizedSource {
    pub canonical_uri: String,
    pub source_kind: SourceKind,
    pub scope: Scope,
    pub record_type: RecordType,
    pub truth_layer: TruthLayer,
    pub recorded_at: String,
    pub text: String,
    pub anchors: Vec<ChunkAnchorSeed>,
}
```

### Pattern 2: FTS Recall First, Rust Bonus Second

**What:** SQL 只负责候选召回、基础 rank 和 snippet；Rust 再叠加 keyword/importance/recency 等可解释 bonus。[CITED: https://sqlite.org/fts5.html; VERIFIED: `.planning/PROJECT.md`]

**When to use:** 所有 ordinary retrieval 请求。[VERIFIED: `.planning/REQUIREMENTS.md`]

**Example:**
```rust
// Source: adapted from libsimple + SQLite FTS5 docs.
let raw = lexical.search_candidates(&query, &filters, recall_limit)?;
let ranked = raw
    .into_iter()
    .map(|candidate| scorer.score(&query, candidate))
    .collect::<Vec<_>>();
```

### Pattern 3: Citation-As-Data

**What:** citation 不是渲染字符串，而是结构化字段：source、chunk anchor、trace、validity。[VERIFIED: `.planning/PROJECT.md`; VERIFIED: `.planning/REQUIREMENTS.md`]

**When to use:** 所有 CLI JSON 输出和 library API 返回值。[VERIFIED: `.planning/ROADMAP.md`]

**Example:**
```rust
// Source: project requirement synthesis.
pub struct Citation {
    pub record_id: String,
    pub source_uri: String,
    pub source_kind: SourceKind,
    pub source_label: Option<String>,
    pub chunk_index: u32,
    pub chunk_count: u32,
    pub anchor: ChunkAnchor,
}
```

### Anti-Patterns to Avoid

- **把 `embedding_only` / `hybrid` 做成 lexical fallback**：Phase 1 已把三种 mode 锁成配置契约，Phase 2 应只让 lexical capability 变成 ready，不能偷偷改写 mode 语义。[VERIFIED: codebase grep]
- **把 score 全塞进 SQL**：这样会让 explainability、测试和 future semantic coexistence 一起变差。[ASSUMED]
- **把源文件路径当成唯一 provenance**：chunk provenance 还需要 index、count、anchor/span，否则 citation 不可落地。[VERIFIED: `.planning/REQUIREMENTS.md`]

## Recommended Ingest Pipeline

1. `detect_input(path_or_text)`：只识别文本型输入、已导出的 conversation JSON/JSONL、普通 note/doc 文本；Phase 2 不做 PDF/Office/网页抓取器大而全。[ASSUMED]
2. `normalize_source(raw)`：统一换行、剥离容器格式、保留 `canonical_uri`、`source_kind`、`source_label`、`recorded_at`、`scope`、`record_type`、`truth_layer`。[VERIFIED: codebase grep; ASSUMED]
3. `chunk_source(normalized)`：plain/document 文本用稳定窗口 + 软边界；conversation 按 turn 组块，避免把多轮对话打成无来源的大块。[VERIFIED: `reference/mempal/src/ingest/chunk.rs`; ASSUMED]
4. `persist_chunks()`：每个 chunk 作为一条 `memory_records` 权威记录写入，并携带 `chunk_index`、`chunk_count`、`anchor`、`content_hash`、`derived_from`、`valid_from`、`valid_to`。[ASSUMED]
5. `sync_fts()`：通过 trigger 或显式 upsert 维护 `memory_records_fts`；迁移已有数据后要执行一次 `rebuild` 以校准 external-content 索引。[CITED: https://sqlite.org/fts5.html]

## FTS / libsimple Integration Direction

- 在 `Database::open()` 或单独 bootstrap helper 中使用 `OnceLock` 包一层 `libsimple::enable_auto_extension()`，避免重复注册 SQLite auto extension。[CITED: https://docs.rs/crate/libsimple/latest; ASSUMED]
- 每个新连接建立后调用 `libsimple::release_jieba_dict()` 与 `libsimple::set_jieba_dict(&conn, dict_dir)`，让 `tokenize='simple'` 可用。[CITED: https://docs.rs/crate/libsimple/latest]
- 推荐创建 `memory_records_fts` 作为 external-content FTS 表，`content='memory_records'`，`content_rowid='rowid'`，不要把当前 `memory_records.id TEXT PRIMARY KEY` 直接误当作 `content_rowid`。[VERIFIED: codebase grep; CITED: https://sqlite.org/fts5.html]
- recall SQL 用参数化查询，返回 `rowid`、`rank` 或 `bm25(...)`、`snippet(...)`，再 join 回 `memory_records` 取 typed metadata。[CITED: https://sqlite.org/fts5.html]
- mixed query 策略推荐做成显式 helper：中文优先 `jieba_query`，ASCII/pinyin 补 `simple_query`，必要时对两路结果取并集后去重。[CITED: https://docs.rs/crate/libsimple/latest; ASSUMED]

## Scoring And Explainability Model

推荐用稳定 typed breakdown，而不是一个无解释的 `final_score: f32`。[VERIFIED: `.planning/PROJECT.md`]

```rust
pub struct ScoreBreakdown {
    pub lexical_raw: f32,
    pub lexical_norm: f32,
    pub keyword_bonus: f32,
    pub importance_bonus: f32,
    pub recency_bonus: f32,
    pub emotion_bonus: f32,
    pub filter_penalty: f32,
    pub final_score: f32,
}
```

- `lexical_norm` 必须是主导项；bonus 只能做小幅扰动，不能把 lexical 候选次序彻底推翻。[VERIFIED: `.planning/PROJECT.md`; ASSUMED]
- `keyword_bonus` 应基于 query token 与 `content_text`/`source_label` 的显式重叠，不做 opaque 语义分数。[VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-CONTEXT.md`; ASSUMED]
- `importance_bonus` 与 `emotion_bonus` 在缺失元数据时默认为 `0.0`，先把字段和 trace 合同定住，再决定 Phase 3+ 是否引入更丰富信号。[ASSUMED]
- explainability 输出至少包含 `matched_query`, `fts_strategy`, `applied_filters`, `score_breakdown`, `snippet`, `citation`。[VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-CONTEXT.md`]

## Retrieval Result Contract

```rust
pub struct SearchResult {
    pub record_id: String,
    pub content_text: String,
    pub snippet: Option<String>,
    pub source: SourceRef,
    pub scope: Scope,
    pub record_type: RecordType,
    pub truth_layer: TruthLayer,
    pub recorded_at: String,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub citation: Citation,
    pub trace: RetrievalTrace,
}
```

```rust
pub struct RetrievalTrace {
    pub mode: &'static str,
    pub lexical_query: String,
    pub fts_strategy: &'static str,
    pub applied_filters: Vec<String>,
    pub score: ScoreBreakdown,
}
```

最小 citation 字段建议为 `record_id`、`source_uri`、`source_kind`、`source_label`、`chunk_index`、`chunk_count`、`anchor`；其中 `anchor` 建议做成 `LineRange | CharRange | TurnRange` 三选一的 enum，以便 Phase 2 先覆盖文本与对话，不为未来格式扩展锁死结构。[VERIFIED: `.planning/REQUIREMENTS.md`; ASSUMED]

## CLI And Library Guidance

### CLI

- `agent-memos ingest <PATH>`：默认做 source detect；支持 `--source-kind` 覆盖、`--scope`、`--record-type`、`--truth-layer`、`--recorded-at`、`--dry-run`、`--json`。[VERIFIED: codebase grep; ASSUMED]
- `agent-memos search <QUERY>`：支持 `--top-k`、`--scope`、`--record-type`、`--truth-layer`、`--valid-at`、`--from`、`--to`、`--trace`、`--json`。[VERIFIED: `.planning/REQUIREMENTS.md`; ASSUMED]
- CLI 只做参数解析与文本/JSON 渲染，不直接拼 SQL、不直接做打分。[VERIFIED: codebase grep]

### Library API

```rust
pub trait OrdinaryRetrieval {
    fn ingest(&self, request: IngestRequest) -> Result<IngestReport, IngestError>;
    fn search(&self, request: SearchRequest) -> Result<SearchResponse, SearchError>;
}
```

library surface 不应暴露任何 Rig、LLM、embedding backend 类型；ordinary retrieval 与 agent search 的边界应在 type level 就分开。[VERIFIED: `.planning/PROJECT.md`; VERIFIED: `.planning/REQUIREMENTS.md`]

## Don’t Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| 中文 / 拼音切词 | 自写 tokenizer | `libsimple` tokenizer + `jieba_query` / `simple_query` [CITED: https://docs.rs/crate/libsimple/latest] | 这是 Phase 2 的 locked baseline，而且官方示例已覆盖 SQLite 集成。[CITED: https://docs.rs/crate/libsimple/latest] |
| 全文排名 | 自写倒排 / TF matrix | SQLite FTS5 `rank` / `bm25` [CITED: https://sqlite.org/fts5.html] | FTS5 已提供成熟的 candidate recall 与排名函数。[CITED: https://sqlite.org/fts5.html] |
| 解释片段 | 手搓 substring 命中高亮 | FTS5 `snippet()` / `highlight()` [CITED: https://sqlite.org/fts5.html] | 直接复用数据库已有命中片段能力更稳。[CITED: https://sqlite.org/fts5.html] |
| 索引同步 | ad-hoc 双写协议 | external-content FTS + trigger + `rebuild` [CITED: https://sqlite.org/fts5.html] | 可以让 `memory_records` 继续做 authority，同时保持 FTS 可重建。[CITED: https://sqlite.org/fts5.html] |

**Key insight:** Phase 2 的差异化不在 tokenizer 或索引内核，而在 provenance、citation、score breakdown 和 future semantic seam；这些地方才值得手写 Rust 逻辑。[VERIFIED: `.planning/PROJECT.md`; ASSUMED]

## Common Pitfalls

### Pitfall 1: 把 `TEXT PRIMARY KEY` 直接拿去当 FTS `content_rowid`

**What goes wrong:** 当前 `memory_records.id` 是文本主键，不是 SQLite `rowid` 别名；直接当 `content_rowid` 会让 external-content FTS 设计失真。[VERIFIED: codebase grep; CITED: https://sqlite.org/fts5.html]

**How to avoid:** 用隐藏 `rowid` join，或另加整数 docid；不要在 Phase 2 里重写整张 authority 表的主键策略。[VERIFIED: codebase grep; ASSUMED]

### Pitfall 2: 先全量召回，再在 Rust 里做大部分过滤

**What goes wrong:** `scope`、`record_type`、`truth_layer`、时间窗口如果不进 SQL，会让候选数量膨胀，trace 也难解释。[VERIFIED: `.planning/REQUIREMENTS.md`; ASSUMED]

**How to avoid:** filter 先落在 SQL 候选阶段，Rust 只处理 bonus 与最终排序。[ASSUMED]

### Pitfall 3: bonus 反客为主

**What goes wrong:** 如果 keyword/importance/recency 加得太重，结果会从“lexical-first”变成“规则黑箱 first”。[VERIFIED: `.planning/PROJECT.md`; ASSUMED]

**How to avoid:** 让 lexical 基分保持支配地位，并把每一项 bonus 暴露到 trace 里。[ASSUMED]

### Pitfall 4: 因为配置里有 `embedding_only` / `hybrid` 就提前接 semantic

**What goes wrong:** 这会直接突破 Phase 2 scope，并污染 ordinary retrieval 的 explainability contract。[VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-CONTEXT.md`]

**How to avoid:** 只让 lexical capability 变成 ready；其余 mode 继续 truthful deferred。[VERIFIED: codebase grep]

## Code Examples

### libsimple bootstrap on connection

```rust
// Source: adapted from https://docs.rs/crate/libsimple/latest
static LIBSIMPLE_INIT: std::sync::OnceLock<Result<(), String>> = std::sync::OnceLock::new();

fn prepare_connection(conn: &rusqlite::Connection, db_dir: &std::path::Path) -> anyhow::Result<()> {
    LIBSIMPLE_INIT
        .get_or_init(|| libsimple::enable_auto_extension().map_err(|e| e.to_string()))
        .as_ref()
        .map_err(|e| anyhow::anyhow!(e.clone()))?;

    let dict_dir = libsimple::release_jieba_dict(db_dir)?;
    libsimple::set_jieba_dict(conn, &dict_dir)?;
    Ok(())
}
```

### external-content FTS table for `memory_records`

```sql
-- Source: adapted from https://sqlite.org/fts5.html
CREATE VIRTUAL TABLE memory_records_fts USING fts5(
    content_text,
    source_label UNINDEXED,
    content = 'memory_records',
    content_rowid = 'rowid',
    tokenize = 'simple'
);
```

## Testing Focus

- unit: `detect`、`normalize`、`chunk` 的格式识别与 anchor 生成。[VERIFIED: codebase grep]
- integration: migration 后存在 FTS 表/trigger，ingest 写入 `memory_records` 与 FTS 一致，`rebuild` 能重建历史数据。[CITED: https://sqlite.org/fts5.html; VERIFIED: codebase grep]
- retrieval: 中文 query、拼音 query、mixed query、scope/type/truth/time filter、trace 字段、排序稳定性。[VERIFIED: `.planning/REQUIREMENTS.md`; ASSUMED]
- CLI: `ingest` / `search --json` 无需 Rig 或 LLM 即可运行。[VERIFIED: `.planning/ROADMAP.md`]

## Anti-Goals

- 不做 embedding/vector execution、RRF、hybrid merge。[VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-CONTEXT.md`]
- 不引入 Rig、LLM provider、agent runtime 作为 ordinary retrieval 前提。[VERIFIED: `.planning/PROJECT.md`]
- 不承诺 PDF/Office/网页抓取全家桶；Phase 2 只做文本型 ingest 基线。[ASSUMED]
- 不把 truth promotion、working memory、metacognition 偷渡进 retrieval phase。[VERIFIED: `.planning/ROADMAP.md`]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| memory 工具默认先上 embedding/vector [VERIFIED: `reference/mempal/README_zh.md`] | 本项目在 Phase 2 锁 lexical-first baseline，semantic 仅保留扩展缝。[VERIFIED: `.planning/PROJECT.md`; VERIFIED: `.planning/ROADMAP.md`] | 2026-04-15 project decisions [VERIFIED: `.planning/PROJECT.md`] | ordinary retrieval 更可解释，也更适合 no-model-file 约束。[VERIFIED: `.planning/PROJECT.md`] |
| “top-k 文本片段”式返回 [ASSUMED] | 结构化 citation + trace + validity result contract。[VERIFIED: `.planning/REQUIREMENTS.md`] | Phase 2 scope lock [VERIFIED: `.planning/phases/02-ingest-and-lightweight-retrieval/02-CONTEXT.md`] | 让 planner 后续能直接衔接 truth governance 与 agent search。[VERIFIED: `.planning/ROADMAP.md`; ASSUMED] |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | mixed query 应同时尝试 `jieba_query` 与 `simple_query` 再去重 | FTS / libsimple Integration Direction | 中文/拼音混合检索的 recall 可能需要调整 |
| A2 | `importance_bonus` / `emotion_bonus` 在 Phase 2 可以先保留字段并默认 `0.0` | Scoring And Explainability Model | 若用户要求立即提供真实信号来源，需补 schema 或 ingest metadata |
| A3 | Phase 2 只支持文本型 ingest 输入就足够闭合当前范围 | Recommended Ingest Pipeline / Anti-Goals | 若必须立刻 ingest PDF/HTML，计划会低估工作量 |

## Open Questions (RESOLVED)

1. **Phase 2 是否要在 schema 中立刻加 `valid_from` / `valid_to` nullable 列？**
   - Decision: **Yes.** Phase 2 should add explicit nullable `valid_from` and `valid_to` columns alongside `recorded_at`, and treat them as the canonical validity contract for retrieval filtering and explanation.
   - Rationale: `recorded_at` answers "when the record was captured", while `valid_from` / `valid_to` answer "when the memory is valid". `RET-04` and `RET-05` need that distinction, and a nullable contract is better than inventing implicit fallback semantics later.
   - Null semantics: `NULL` means "open-ended or unknown validity bound", not "the feature does not exist".
   - Planning impact: the ingest-foundation plan owns schema/model/repository support for these fields; later retrieval plans consume the same contract for filtering, citations, and traces without reopening the data model decision.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | build / test / CLI [VERIFIED: codebase grep] | ✓ [VERIFIED: local command] | `rustc 1.94.1`, `cargo 1.94.1` [VERIFIED: local command] | — |
| bundled SQLite via `rusqlite` | storage / FTS [VERIFIED: codebase grep] | ✓ [VERIFIED: codebase grep] | `rusqlite 0.37.0` bundled [VERIFIED: codebase grep] | — |
| `sqlite3` CLI | manual DB inspection [ASSUMED] | ✗ [VERIFIED: local command] | — | 通过 `rusqlite` tests 或 DB Browser 替代 |
| `cargo-nextest` | optional faster test loops [ASSUMED] | ✗ [VERIFIED: local command] | — | 用 `cargo test` |
| Node / npx | Context7 CLI fallback only [VERIFIED: execution log] | ✓ with caveat [VERIFIED: local command] | `v22.22.0` [VERIFIED: local command] | 直接查官方 docs；当前 `ctx7` CLI 在本机失败 [VERIFIED: execution log] |

**Missing dependencies with no fallback:**
- None for Phase 2 implementation.[VERIFIED: local command; VERIFIED: codebase grep]

**Missing dependencies with fallback:**
- `sqlite3` CLI missing; not blocking because project already uses bundled SQLite and tests pass。[VERIFIED: local command; VERIFIED: execution log]
- `cargo-nextest` missing; use `cargo test`。[VERIFIED: local command]

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness via `cargo test` [VERIFIED: codebase grep] |
| Config file | `Cargo.toml` [VERIFIED: codebase grep] |
| Quick run command | `cargo test --quiet` [VERIFIED: execution log] |
| Full suite command | `cargo test && cargo clippy --all-targets -- -D warnings` [VERIFIED: `.planning/phases/01-foundation-kernel/01-VALIDATION.md`] |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ING-01 | plain text / conversation export normalize into chunkable units | unit+integration | `cargo test --test ingest_pipeline ingest_normalizes_supported_sources -- --nocapture` | ❌ Wave 0 |
| ING-02 | source linkage and chunk provenance survive ingest round-trip | integration | `cargo test --test ingest_pipeline ingest_preserves_chunk_provenance -- --nocapture` | ❌ Wave 0 |
| ING-03 | ingest persists lexical metadata without embeddings | integration | `cargo test --test lexical_search lexical_candidate_recall_uses_phase_two_fts -- --nocapture` | ❌ Wave 0 |
| RET-01 | Chinese and PinYin lexical recall works | integration | `cargo test --test lexical_search search_supports_chinese_and_pinyin_queries -- --nocapture` | ❌ Wave 0 |
| RET-02 | Rust keyword bonus changes ranking deterministically | integration | `cargo test --test lexical_search keyword_bonus_affects_rank_deterministically -- --nocapture` | ❌ Wave 0 |
| RET-03 | score breakdown is stable and explainable | integration | `cargo test --test retrieval_cli library_search_returns_citations_and_filter_trace -- --nocapture` | ❌ Wave 0 |
| RET-04 | result includes source/scope/validity/trace/citation | integration | `cargo test --test retrieval_cli library_search_returns_citations_and_filter_trace -- --nocapture` | ❌ Wave 0 |
| RET-05 | filters for scope/type/truth/time work at SQL candidate stage | integration | `cargo test --test retrieval_cli filters_apply_before_rerank -- --nocapture` | ❌ Wave 0 |
| AGT-01 | ordinary retrieval runs from CLI/library without Rig or LLM | integration | `cargo test --test retrieval_cli ordinary_search_runs_without_agent_runtime -- --nocapture` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --quiet` [VERIFIED: execution log]
- **Per wave merge:** `cargo test && cargo clippy --all-targets -- -D warnings` [VERIFIED: `.planning/phases/01-foundation-kernel/01-VALIDATION.md`]
- **Phase gate:** Full suite green before `/gsd-verify-work`。[VERIFIED: `.planning/config.json`]

### Wave 0 Gaps

- [ ] `tests/ingest_pipeline.rs` — 覆盖 `ING-01` / `ING-02`。[ASSUMED]
- [ ] `tests/lexical_search.rs` — 覆盖 `ING-03` / `RET-01` / `RET-02`。[ASSUMED]
- [x] `tests/status_cli.rs` — existing canonical status contract extended in Plan `02-02` for lexical readiness semantics.[VERIFIED: codebase grep]
- [ ] `tests/retrieval_cli.rs` — 覆盖 `RET-03` / `RET-04` / `RET-05` / `AGT-01`。[ASSUMED]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no [VERIFIED: `.planning/ROADMAP.md`] | — |
| V3 Session Management | no [VERIFIED: `.planning/ROADMAP.md`] | — |
| V4 Access Control | yes [VERIFIED: `.planning/REQUIREMENTS.md`] | scope / record_type / truth_layer / validity filters must be typed request fields, not free-form SQL fragments。[VERIFIED: codebase grep; ASSUMED] |
| V5 Input Validation | yes [VERIFIED: `.planning/REQUIREMENTS.md`] | `clap` typed args, path canonicalization, bound SQL params, enum parsing。[VERIFIED: codebase grep; ASSUMED] |
| V6 Cryptography | no [VERIFIED: `.planning/ROADMAP.md`] | — |

### Known Threat Patterns for Rust + SQLite lexical retrieval

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| dynamic SQL / FTS injection through query strings | Tampering | MATCH 参数和 filter 参数都走 bound params；不拼接用户原始 query 到 SQL 片段。[CITED: https://sqlite.org/fts5.html; ASSUMED] |
| path traversal or accidental over-ingest | Information Disclosure | CLI 层 canonicalize 输入路径，并在 report 中回显 canonical source URI。[ASSUMED] |
| generic extension loading exposed to untrusted paths | Elevation of Privilege | 只做内部一次性 `libsimple` bootstrap，不暴露通用 `load_extension` 路径；`LoadExtensionGuard` 本身也标记为危险能力。[CITED: https://docs.rs/rusqlite/latest/rusqlite/struct.LoadExtensionGuard.html] |

## Sources

### Primary (HIGH confidence)

- `.planning/PROJECT.md` - project constraints and lexical-first baseline
- `.planning/ROADMAP.md` - Phase 2 goal, success criteria, plan slots
- `.planning/REQUIREMENTS.md` - `ING-*`, `RET-*`, `AGT-01`
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-CONTEXT.md` - locked decisions and discretion
- `src/memory/record.rs` / `src/memory/repository.rs` / `migrations/0001_foundation.sql` - current authority schema and typed metadata
- `reference/mempal/src/ingest/*.rs` / `reference/mempal/src/search/mod.rs` - module split and ingest/search boundaries
- https://docs.rs/crate/libsimple/latest - tokenizer setup, query helpers, compatibility note
- https://sqlite.org/fts5.html - FTS5 design, external-content tables, ranking, snippet/highlight

### Secondary (MEDIUM confidence)

- https://docs.rs/rusqlite/latest/rusqlite/struct.LoadExtensionGuard.html - extension loading safety boundary
- `reference/mempal/README_zh.md` - reference product direction and contrast with current project scope

### Tertiary (LOW confidence)

- None.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - core choices are locked by project docs and verified against official `libsimple` / SQLite docs.
- Architecture: MEDIUM - module boundaries are clear, but exact mixed-query helper and score weighting still require implementation judgment.
- Pitfalls: HIGH - most risks are already visible from current schema, phase scope, and official FTS docs.

**Research date:** 2026-04-15
**Valid until:** 2026-05-15

## RESEARCH COMPLETE
