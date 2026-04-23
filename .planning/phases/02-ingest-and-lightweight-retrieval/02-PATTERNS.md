# Phase 2: Ingest And Lightweight Retrieval - Pattern Map

**Mapped:** 2026-04-15
**Files analyzed:** 18
**Analogs found:** 16 / 18

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `migrations/0002_ingest_foundation.sql` | migration | batch | `migrations/0001_foundation.sql` | role-match |
| `migrations/0003_lexical_sidecar.sql` | migration | batch | `migrations/0001_foundation.sql` | role-match |
| `src/core/migrations.rs` | config | batch | `src/core/migrations.rs` | exact |
| `src/lib.rs` | config | transform | `src/lib.rs` | exact |
| `src/memory/repository.rs` | store | CRUD | `src/memory/repository.rs` | exact |
| `src/ingest/mod.rs` | service | file-I/O | `reference/mempal/src/ingest/mod.rs` | structure-ref |
| `src/ingest/detect.rs` | utility | transform | `reference/mempal/src/ingest/detect.rs` | structure-ref |
| `src/ingest/normalize.rs` | utility | transform | `reference/mempal/src/ingest/normalize.rs` | structure-ref |
| `src/ingest/chunk.rs` | utility | transform | `reference/mempal/src/ingest/chunk.rs` | structure-ref |
| `src/search/mod.rs` | service | request-response | `src/interfaces/cli.rs` | role-match |
| `src/search/lexical.rs` | store | request-response | `src/memory/repository.rs` | role-match |
| `src/search/score.rs` | utility | transform | `reference/mempal/src/search/mod.rs` | partial |
| `src/search/rerank.rs` | utility | transform | `reference/mempal/src/search/mod.rs` | partial |
| `src/search/citation.rs` | utility | transform | `src/memory/record.rs` | role-match |
| `src/search/filter.rs` | utility | transform | `src/core/status.rs` | partial |
| `src/interfaces/cli.rs` | controller | request-response | `src/interfaces/cli.rs` | exact |
| `tests/ingest_pipeline.rs` | test | request-response | `tests/foundation_schema.rs` | role-match |
| `tests/lexical_search.rs` | test | request-response | `tests/status_cli.rs` | role-match |
| `tests/retrieval_cli.rs` | test | request-response | `tests/status_cli.rs` | exact |

## Pattern Assignments

### `migrations/0002_ingest_foundation.sql` + `migrations/0003_lexical_sidecar.sql` + `src/core/migrations.rs`

**主类比:** `migrations/0001_foundation.sql`, `src/core/migrations.rs`

**应复用的 migration 注册方式**
- `src/core/migrations.rs:4-15`

```rust
const FOUNDATION_SCHEMA_SQL: &str = include_str!("../../migrations/0001_foundation.sql");

pub fn apply_migrations(conn: &mut Connection) -> Result<(), rusqlite_migration::Error> {
    migrations().to_latest(conn)
}

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(FOUNDATION_SCHEMA_SQL)
            .foreign_key_check()
            .comment("foundation schema bootstrap"),
    ])
}
```

**应复用的 SQL 风格**
- `migrations/0001_foundation.sql:1-20`

```sql
CREATE TABLE IF NOT EXISTS memory_records (
    id TEXT PRIMARY KEY,
    source_uri TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    source_label TEXT,
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

**Phase 2 具体套法**
- 继续 additive migration，不回写或重构 `0001`。
- `0002_ingest_foundation.sql` 只负责 ingest authority-store 扩展，例如 chunk/source/validity 元数据，不抢 lexical sidecar 的职责。
- `0003_lexical_sidecar.sql` 再单独引入 FTS5 sidecar、相关 trigger / rebuild helper 和 lexical-only 检索元数据。
- 新的 ingest / lexical 表应围绕 `memory_records.id` 建索引与外键语义，不要把检索行为和 authority schema 混成一次大迁移。

---

### `src/lib.rs`

**主类比:** `src/lib.rs`

**应复用的模块导出模式**
- `src/lib.rs:1-5`

```rust
#![warn(clippy::all)]

pub mod core;
pub mod interfaces;
pub mod memory;
```

**Phase 2 具体套法**
- 继续单 crate 扩展，直接加 `pub mod ingest;` 和 `pub mod search;`。
- 不要为了 Phase 2 把仓库改成 mempal 的多 crate/workspace；当前项目的 locked baseline 是 Phase 1 现有单 crate。

---

### `src/memory/repository.rs`

**主类比:** `src/memory/repository.rs`

**应复用的 repository 构造与错误边界**
- `src/memory/repository.rs:14-34`

```rust
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("invalid {field} stored in database: {value}")]
    InvalidEnum {
        field: &'static str,
        value: String,
    },
}

pub struct MemoryRepository<'db> {
    conn: &'db Connection,
}

impl<'db> MemoryRepository<'db> {
    pub fn new(conn: &'db Connection) -> Self {
        Self { conn }
    }
```

**应复用的 SQL 写入边界**
- `src/memory/repository.rs:36-74`

```rust
pub fn insert_record(&self, record: &MemoryRecord) -> Result<(), RepositoryError> {
    let provenance_json = serde_json::to_string(&record.provenance)?;

    self.conn.execute(
        r#"
        INSERT INTO memory_records (
            id,
            source_uri,
            source_kind,
            source_label,
            recorded_at,
            scope,
            record_type,
            truth_layer,
            provenance_json,
            content_text,
            created_at,
            updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        "#,
        params![ ... ],
    )?;

    Ok(())
}
```

**应复用的 row mapping 风格**
- `src/memory/repository.rs:166-219`

```rust
fn map_record_row(row: &rusqlite::Row<'_>) -> Result<MemoryRecord, RepositoryError> {
    let source_kind = row.get::<_, String>(2)?;
    let scope = row.get::<_, String>(5)?;
    let record_type = row.get::<_, String>(6)?;
    let truth_layer = row.get::<_, String>(7)?;
    let provenance_json = row.get::<_, String>(8)?;

    Ok(MemoryRecord {
        // ...
        provenance: serde_json::from_str::<Provenance>(&provenance_json)?,
        content_text: row.get(9)?,
    })
}
```

**Phase 2 具体套法**
- `memory` 仍然只拥有 typed persistence 和 hydration，不拥有 detect/normalize/chunk/score 逻辑。
- 可以在这里新增 ingest/search 所需的低层读写方法，例如插入 chunk 元数据、按过滤条件加载记录、按候选 id 批量 hydrate。
- 不要把 CLI 参数解析、检索排序、解释文本拼装塞进 repository。

---

### `src/ingest/mod.rs`

**主类比:** `reference/mempal/src/ingest/mod.rs`，但边界基线以本仓库 `src/memory/repository.rs` 为准

**可借的 orchestrator 结构**
- `reference/mempal/src/ingest/mod.rs:3-25`
- `reference/mempal/src/ingest/mod.rs:121-178`

```rust
pub mod chunk;
pub mod detect;
pub mod normalize;

use crate::ingest::{
    chunk::{chunk_conversation, chunk_text},
    detect::{Format, detect_format},
    normalize::{NormalizeError, normalize_content},
};

pub async fn ingest_file_with_options<...>(...) -> Result<IngestStats> {
    let bytes = tokio::fs::read(path).await?;
    let content = String::from_utf8_lossy(&bytes).to_string();
    let format = detect_format(&content);
    let normalized = normalize_content(&content, format)?;
    let chunks = match format {
        Format::... => chunk_conversation(&normalized),
        Format::PlainText => chunk_text(&normalized, CHUNK_WINDOW, CHUNK_OVERLAP),
    };
    // ...
}
```

**Phase 2 应改写成的本仓库模式**
- 保持 `mod.rs` 只做 ingest orchestration: `read -> detect -> normalize -> chunk -> map typed chunk -> repository persist`。
- 优先同步实现，除非 Phase 2 明确需要 `tokio`；本仓库 Phase 1 CLI 和 DB 都是同步边界。
- 输入输出类型应围绕本仓库的 `MemoryRecord`、`SourceRef`、`Provenance` 和未来 chunk metadata，而不是 mempal 的 `Drawer`。

**不要复制的 mempal 前提**
- `reference/mempal/src/ingest/mod.rs:100-119` 的 `Embedder` 泛型与 async embed 流程。
- `reference/mempal/src/ingest/mod.rs:205-239` 的 vector 插入、drawer 去重、taxonomy 路由。
- Phase 2 不应引入向量生成、房间路由、wing/room 语义依赖。

---

### `src/ingest/detect.rs`

**主类比:** `reference/mempal/src/ingest/detect.rs`

**可借的 pure function 拆分**
- `reference/mempal/src/ingest/detect.rs:3-30`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    ClaudeJsonl,
    ChatGptJson,
    CodexJsonl,
    SlackJson,
    PlainText,
}

pub fn detect_format(content: &str) -> Format {
    if is_claude_jsonl(content) { ... }
    if is_codex_jsonl(content) { ... }
    if is_slack_json(content) { ... }
    if is_chatgpt_json(content) { ... }
    Format::PlainText
}
```

**Phase 2 具体套法**
- 维持 `detect.rs` 为纯文本判别，不做 DB 调用、不做 chunking。
- 可以直接借 `Format` + `detect_format()` 这种 shape。
- 仅保留当前 Phase 2 确认支持的格式；不要为了“未来可能支持”提前接过多外部格式分支。

---

### `src/ingest/normalize.rs`

**主类比:** `reference/mempal/src/ingest/normalize.rs`

**可借的错误与入口模式**
- `reference/mempal/src/ingest/normalize.rs:6-24`

```rust
pub type Result<T> = std::result::Result<T, NormalizeError>;

#[derive(Debug, Error)]
pub enum NormalizeError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("unsupported ChatGPT JSON shape")]
    UnsupportedChatGptShape,
}

pub fn normalize_content(content: &str, format: Format) -> Result<String> {
    match format {
        Format::PlainText => Ok(content.trim().to_string()),
        // ...
    }
}
```

**可借的 transcript 归一化思路**
- `reference/mempal/src/ingest/normalize.rs:196-212`

```rust
fn render_transcript(items: impl IntoIterator<Item = (String, String)>) -> String {
    let mut lines = Vec::new();

    for (role, content) in items {
        if content.trim().is_empty() {
            continue;
        }

        if matches!(role.as_str(), "user" | "human") {
            lines.push(format!("> {}", content.trim()));
        } else {
            lines.push(content.trim().to_string());
        }
    }

    lines.join("\n")
}
```

**Phase 2 具体套法**
- `normalize.rs` 输出“适合 chunk 和 lexical indexing 的统一文本”，不要直接产出 DB 行。
- 对 conversation-like 文本，推荐继续用 `> ` 标识 user turn，这与后续 `chunk_conversation` 的边界自然衔接。
- 不要把 truth-layer、scope、record_type 推断塞进 normalize；这些属于 ingest service 的更高层决策。

---

### `src/ingest/chunk.rs`

**主类比:** `reference/mempal/src/ingest/chunk.rs`

**可借的纯 chunk 工具模式**
- `reference/mempal/src/ingest/chunk.rs:1-47`
- `reference/mempal/src/ingest/chunk.rs:49-70`

```rust
pub fn chunk_text(text: &str, window: usize, overlap: usize) -> Vec<String> { ... }

pub fn chunk_conversation(transcript: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = Vec::new();

    for line in transcript.lines() {
        let is_user_turn = line.starts_with("> ");
        if is_user_turn && !current.is_empty() {
            chunks.push(current.join("\n"));
            current.clear();
        }
        // ...
    }
}
```

**Phase 2 具体套法**
- `chunk.rs` 保持无 IO、无 SQL、无 config 读取。
- 除返回 chunk 文本外，Phase 2 还应返回或可推导 `chunk_index`、`char_range`/`line_range` 这类 provenance 锚点。
- 不要照搬 mempal 的固定 window 常量；常量可以放在 `ingest/mod.rs` 或 typed options 中。

---

### `src/search/mod.rs`

**主类比:** 本仓库 `src/interfaces/cli.rs` 的 thin boundary + `reference/mempal/src/search/mod.rs` 的模块组织

**本仓库应继承的 service 入口风格**
- `src/interfaces/cli.rs:42-50`

```rust
pub fn run(cli: Cli, config: Config) -> Result<ExitCode> {
    let app = AppContext::load(config)?;

    match cli.command {
        Commands::Init => init_command(&app),
        Commands::Status => status_command(&app),
        Commands::Doctor => doctor_command(&app),
        Commands::Inspect { command } => inspect_command(&app, command),
    }
}
```

**可借的 search 模块拆分意识，但不要借语义前提**
- `reference/mempal/src/search/mod.rs:11-17`

```rust
use crate::search::filter::build_filter_clause;

pub mod filter;
pub mod rerank;
pub mod route;
```

**Phase 2 具体套法**
- `search/mod.rs` 应只做 ordinary lexical retrieval orchestration: `filter build -> lexical recall -> score -> rerank -> citation assemble`。
- 入口建议保持同步、纯库调用友好，例如 `pub fn ordinary_search(...) -> Result<Vec<SearchResult>, SearchError>`。
- 不要把 config mode 分支扩散到每个子模块；Phase 2 可由上层决定只走 lexical path。

---

### `src/search/lexical.rs`

**主类比:** `src/memory/repository.rs`

**应复用的 SQL 访问边界**
- `src/memory/repository.rs:31-34`
- `src/memory/repository.rs:104-132`

```rust
impl<'db> MemoryRepository<'db> {
    pub fn new(conn: &'db Connection) -> Self {
        Self { conn }
    }

    pub fn list_records(&self) -> Result<Vec<MemoryRecord>, RepositoryError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT ...
            FROM memory_records
            ORDER BY recorded_at ASC, id ASC
            "#,
        )?;
        // ...
    }
}
```

**Phase 2 具体套法**
- `lexical.rs` 只负责候选召回和最低限度的 SQL-level filter，不负责 bonus 计算和最终说明文本。
- 风格上应像 repository：小的 `prepare/query/map` 方法、typed error、显式 SQL。
- 推荐返回 `LexicalCandidate` 之类的中间类型，带 raw lexical score、matched terms、source ids，而不是直接返回 CLI 文本。

**本仓库当前缺失的模式**
- 当前 repo 没有 FTS5/libsimple 查询基线；这里属于“无本地精确类比”，只能继承 repository 的 SQL 边界，不要直接照搬 mempal 的向量查询。

---

### `src/search/score.rs` + `src/search/rerank.rs`

**主类比:** `reference/mempal/src/search/mod.rs` 里的“召回后再合并排序”思想，但实现必须改成 lexical-first

**可借的阶段拆分，不可借的排序内容**
- `reference/mempal/src/search/mod.rs:82-99`
- `reference/mempal/src/search/mod.rs:131-195`

```rust
// Hybrid search: vector + BM25, merged via RRF
let vector_results = search_by_vector(...)?;
let fts_ids = db.search_fts(...)?;
let mut results = if fts_ids.is_empty() {
    vector_results
} else {
    rrf_merge(vector_results, &fts_ids, &route, db, top_k)
};
```

**Phase 2 具体套法**
- 借“候选召回”和“最终排序”分层，不借 vector + RRF。
- `score.rs` 负责 deterministic score breakdown:
  - lexical raw score
  - keyword bonus
  - importance bonus
  - recency bonus
  - 可能的 emotion/context bonus
- `rerank.rs` 只接收候选和 score breakdown，输出稳定排序结果与 trace。
- 不要在 Phase 2 预埋 hybrid merge 逻辑；保留接口 seam 即可。

---

### `src/search/citation.rs`

**主类比:** `src/memory/record.rs`

**应复用的 metadata 来源**
- `src/memory/record.rs:3-20`
- `src/memory/record.rs:52-57`
- `src/memory/record.rs:140-145`

```rust
pub struct MemoryRecord {
    pub id: String,
    pub source: SourceRef,
    pub timestamp: RecordTimestamp,
    pub scope: Scope,
    pub record_type: RecordType,
    pub truth_layer: TruthLayer,
    pub provenance: Provenance,
    pub content_text: String,
}
```

**Phase 2 具体套法**
- citation/result assembly 必须基于现有 typed metadata 组装，不要在 search 层再发明一套来源字段。
- 返回结果至少应复用:
  - `source.uri`
  - `source.kind`
  - `source.label`
  - `timestamp.recorded_at`
  - `scope`
  - `truth_layer`
  - `provenance`
- chunk 级定位信息应从 Phase 2 新增 metadata 读出后合并进 citation，而不是覆盖现有 provenance。

---

### `src/search/filter.rs`

**主类比:** `src/core/status.rs`

**应复用的显式 capability/filter 显示哲学**
- `src/core/status.rs:139-188`

```rust
pub fn render_text(&self) -> String {
    let mut lines = vec![
        "database:".to_string(),
        format!("  path: {}", self.db_path.display()),
        // ...
        "dependencies:".to_string(),
        format!("  lexical_dependency_state: {}", self.lexical_dependency_state),
        format!("  embedding_dependency_state: {}", self.embedding_dependency_state),
        format!("  index_readiness: {}", self.index_readiness),
    ];
    // ...
}
```

**Phase 2 具体套法**
- `filter.rs` 应把 `scope`、`record_type`、`truth_layer`、时间有效性过滤显式建模成 typed query，而不是散落成字符串 if/else。
- 与 `status.rs` 一样，过滤/未命中过滤条件应可解释，便于结果 trace。
- 这里没有本地精确 analog；优先复用 `CapabilityState` 那种“不要隐式吞掉状态”的表达方式。

---

### `src/interfaces/cli.rs`

**主类比:** `src/interfaces/cli.rs`

**应复用的 thin command boundary**
- `src/interfaces/cli.rs:16-35`
- `src/interfaces/cli.rs:79-119`

```rust
#[derive(Debug, Clone, Parser)]
#[command(name = "agent-memos", about = "Local-first memory kernel for agents")]
pub struct Cli {
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

fn status_command(app: &AppContext) -> Result<ExitCode> {
    let report = StatusReport::collect(app)?;
    println!("{}", report.render_text());
    Ok(ExitCode::SUCCESS)
}
```

**应复用的 init 先诊断、后执行、再重取快照模式**
- `src/interfaces/cli.rs:53-77`

```rust
fn init_command(app: &AppContext) -> Result<ExitCode> {
    let preflight_status = StatusReport::collect(app)?;
    let doctor = DoctorReport::evaluate(&preflight_status, CommandPath::Init);
    // ...
    let db = Database::open(app.db_path())?;
    let post_init_status = StatusReport::collect(app)?;
    let post_init_doctor = DoctorReport::evaluate(&post_init_status, CommandPath::Init);
    // ...
}
```

**Phase 2 具体套法**
- CLI 只新增 `ingest`/`search`/必要的 `inspect` 子命令，不直接实现 normalize、FTS 查询、rerank。
- 命令函数保持“加载 app -> 调 service -> 打印结果 -> 返回 ExitCode”。
- 普通检索命令应能在 library 层之外单独运行，但其业务逻辑必须来自 `src/search/*`。

---

### `tests/retrieval_cli.rs`

**主类比:** `tests/status_cli.rs`

**应复用的 CLI 集成测试模式**
- `tests/status_cli.rs:20-48`
- `tests/status_cli.rs:58-99`

```rust
fn write_config(path: &Path, db_path: &Path, mode: &str, backend: &str) {
    // 写最小 config fixture
}

fn run_cli(config_path: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_agent-memos"))
        .arg("--config")
        .arg(config_path)
        .args(args)
        .output()
        .expect("binary should run")
}
```

**Phase 2 具体套法**
- Phase 2 CLI 测试继续走真实二进制，不绕过 CLI 直接测内部函数。
- 重点覆盖:
  - `init -> ingest -> search` 主链路
  - `lexical_only` 成功检索
  - `embedding_only` / `hybrid` 仍保持诚实状态，不暗中走语义检索
  - 结果里有 citation / scope / timestamp / trace 字段

## Shared Patterns

### 1. 继续复用 Phase 1 的单 crate 扩展方式

**来源**
- `src/lib.rs:1-5`
- `src/main.rs:24-30`
- `src/interfaces/mod.rs:1-3`

```rust
fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    let config = match cli.config.as_deref() {
        Some(path) => Config::load_from(path)?,
        None => Config::load()?,
    };
    interfaces::run(cli, config)
}
```

**应用到**
- `src/ingest/*`
- `src/search/*`
- `src/interfaces/cli.rs`

**结论**
- Phase 2 是在 Phase 1 crate 上继续长，不是切 crate、不是切 workspace。

### 2. 继续复用 typed config contract，不改三模式形状

**来源**
- `src/core/config.rs:12-49`
- `src/core/app.rs:39-79`

```rust
pub enum RetrievalMode {
    LexicalOnly,
    EmbeddingOnly,
    Hybrid,
}

pub struct RetrievalConfig {
    pub mode: RetrievalMode,
}
```

**应用到**
- `src/search/mod.rs`
- `src/interfaces/cli.rs`
- `src/core/status.rs`

**结论**
- 继续解析 `lexical_only` / `embedding_only` / `hybrid`。
- 但 Phase 2 只实现 lexical-first ordinary retrieval，不在实现层伪造 semantic path。

### 3. repository/service 分层继续沿用

**来源**
- `src/core/db.rs:40-76`
- `src/memory/repository.rs:31-34`

```rust
pub fn open(path: &Path) -> Result<Self, DbError> { ... }

pub struct MemoryRepository<'db> {
    conn: &'db Connection,
}
```

**应用到**
- `ingest` service 读文件、调纯函数、调用 repository
- `search` service 调 lexical store + hydrate + rerank

**结论**
- `core/db.rs` 只管连接和 migration。
- `memory/repository.rs` 只管 typed persistence/query。
- `ingest` / `search` 是 application service，不应反向侵入 `core`。

### 4. 状态与说明必须保持“诚实输出”

**来源**
- `src/core/status.rs:67-137`
- `src/core/doctor.rs:20-77`

```rust
let lexical_dependency_state = match app.config.retrieval.mode {
    RetrievalMode::LexicalOnly | RetrievalMode::Hybrid => CapabilityState::NotBuiltInPhase1,
    RetrievalMode::EmbeddingOnly => CapabilityState::NotApplicable,
};
```

**应用到**
- Phase 2 完成后把 lexical capability 从 `NotBuiltInPhase1` 升成真实状态。
- semantic dependency/index 仍可保持 `Deferred` / `Missing` / `NotApplicable`。

**结论**
- 不要因为实现了 lexical path 就把 `embedding_only` / `hybrid` 伪装成“已完成”。

## 明确复用与明确不复用

### 应明确复用当前代码库的内容

- 复用 `src/core/config.rs` 的 typed retrieval-mode contract，不新增自由字符串模式。
- 复用 `src/core/migrations.rs` 的 additive migration 注册方式。
- 复用 `src/memory/record.rs` 作为 retrieval result metadata 的唯一基础来源。
- 复用 `src/memory/repository.rs` 的 typed SQL boundary 和 error shape。
- 复用 `src/interfaces/cli.rs` 的 thin command dispatch。
- 复用 `tests/status_cli.rs` 的真实二进制 CLI 测试方式。

### 不应从 mempal 复制的内容

- 不复制 `reference/mempal/src/ingest/mod.rs:100-119` 的 `Embedder` 泛型和 async embedding 前提。
- 不复制 `reference/mempal/src/ingest/mod.rs:205-239` 的 vector 插入流程。
- 不复制 `reference/mempal/src/search/mod.rs:45-99` 的“先 embed query 再查”的主路径。
- 不复制 `reference/mempal/src/search/mod.rs:131-195` 的 hybrid RRF merge 作为 Phase 2 排序逻辑。
- 不复制 mempal 的 `Drawer`/`wing`/`room` 域模型；本项目当前基线是 `MemoryRecord` + `SourceRef` + `Scope` + `TruthLayer`。

## No Analog Found

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `src/search/score.rs` | utility | transform | 当前 repo 没有任何本地轻量评分器；只能复用 Phase 1 的纯函数/typed 边界风格，不能直接从 mempal 借向量或 RRF 公式。 |
| `src/search/filter.rs` | utility | transform | 当前 repo 只有状态枚举展示，没有现成的 typed retrieval filter builder；应按 `Scope` / `RecordType` / `TruthLayer` 的 typed enum 方式新建。 |

## Recommendation

- 首选模式一：继续单 crate 扩展，在 `src/ingest/` 和 `src/search/` 下加模块，不改工程形态。
- 首选模式二：`ingest/mod.rs` 做同步 orchestrator，`detect/normalize/chunk` 保持纯函数；文件读取和 repository 写入都留在 `mod.rs`。
- 首选模式三：`search` 明确拆成 `lexical -> score -> rerank -> citation` 四层，`lexical.rs` 只召回，`score.rs` 只算 breakdown，`rerank.rs` 只做稳定排序。
- 首选模式四：普通检索 CLI 继续做 thin wrapper，真正可复用的 library API 放在 `src/search/mod.rs`，并显式保持 “Phase 2 只实现 lexical-first，三模式 config 仍保留” 的诚实语义。

## Metadata

**Analog search scope:** `src/`, `migrations/`, `.planning/phases/01-foundation-kernel/`, `reference/mempal/src/`
**Files scanned:** 22
**Pattern extraction date:** 2026-04-15
