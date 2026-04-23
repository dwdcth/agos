# Phase 1: Foundation Kernel - Pattern Map

**Mapped:** 2026-04-15
**Scope:** 仅覆盖 Foundation Kernel 所需模式
**Files analyzed:** 10
**Analogs found:** 10 / 10

## Context Summary

- 当前仓库没有 Rust 实现代码，Phase 1 的代码模式主要来自 `reference/mempal`，再结合 `.planning/research/STACK.md`、`.planning/research/ARCHITECTURE.md` 和 `01-CONTEXT.md` 做收敛。
- `reference/mempal` 只能作为结构和风格参考，不能作为领域模型模板。Agent Memos 的 Phase 1 必须服务于 lexical-first 底座，而不是混合检索产品本身。
- 新锁定要求已经生效：配置必须显式建模三种检索模式，而不是 `embedding.enabled` 布尔开关。

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `Cargo.toml` | config | transform | `reference/mempal/Cargo.toml` | exact |
| `src/lib.rs` | provider | transform | `reference/mempal/src/lib.rs` | exact |
| `src/main.rs` | controller | request-response | `reference/mempal/src/main.rs` | role-match |
| `src/core/mod.rs` | provider | transform | `reference/mempal/src/core/mod.rs` | exact |
| `src/core/config.rs` | config | file-I/O | `reference/mempal/src/core/config.rs` | exact |
| `src/core/db.rs` | service | CRUD | `reference/mempal/src/core/db.rs` | exact |
| `src/core/migrations.rs` | migration | batch | `reference/mempal/src/core/db.rs` | partial |
| `src/memory/record.rs` | model | CRUD | `reference/mempal/src/core/types.rs` | partial |
| `src/memory/repository.rs` | service | CRUD | `reference/mempal/src/core/db.rs` | partial |
| `src/interfaces/cli.rs` | controller | request-response | `reference/mempal/src/main.rs` | role-match |

## Pattern Assignments

### `Cargo.toml` (config, transform)

**Analog:** `reference/mempal/Cargo.toml`

**可复用模式**

- 用单 package + `[[bin]]` + `src/lib.rs`/`src/main.rs` 组合，而不是一开始拆 workspace。`mempal` 实际上就是单 crate 单二进制入口，适合当前仓库还没有实现代码的状态。
- 保留 feature 扩展位，但不要让 embedding 相关 feature 成为默认启用路径。

**参考片段**

- `reference/mempal/Cargo.toml:15-23`

```toml
[[bin]]
name = "mempal"
path = "src/main.rs"

[features]
default = ["model2vec"]
rest = ["dep:axum", "dep:tower", "dep:tower-http", "model2vec"]
```

**Phase 1 应如何改写**

- 保留 `[[bin]]` 单二进制入口。
- 依赖以 `clap + rusqlite + serde + toml + anyhow + thiserror + tracing` 为主，和 `.planning/research/STACK.md:13-30` 保持一致。
- 不要照搬 `default = ["model2vec"]`。Phase 1 不应该默认拉起 embedding 依赖，更不应该让向量路径成为默认运行路径。
- `sqlite-vec` 最多作为注释掉的后续 feature 预留，或者完全不出现在 Phase 1 `Cargo.toml`。

---

### `src/lib.rs` (provider, transform)

**Analog:** `reference/mempal/src/lib.rs`

**可复用模式**

- 用 `lib.rs` 暴露稳定模块图，`main.rs` 只做 CLI 启动。
- Phase 1 只暴露最小模块集，避免把后续 `search`、`agent`、`mcp`、`embed` 一次性铺开。

**参考片段**

- `reference/mempal/src/lib.rs:1-11`

```rust
#![warn(clippy::all)]

pub mod core;
pub mod embed;
pub mod ingest;
pub mod mcp;
pub mod search;
```

**Phase 1 应如何改写**

- 保留 `#![warn(clippy::all)]` 这类顶层 lint 姿势。
- Phase 1 推荐模块图：

```rust
pub mod core;
pub mod interfaces;
pub mod memory;
```

- 不要在 Phase 1 暴露 `embed`、`search`、`mcp`、`cowork` 等模块；这些是后续阶段的模块，不是 foundation kernel 的交付面。

---

### `src/main.rs` (controller, request-response)

**Analog:** `reference/mempal/src/main.rs`

**可复用模式**

- 启动顺序应保持非常薄：解析 CLI -> 加载配置 -> 打开数据库 -> 调用具体命令处理器。
- 错误处理在 CLI 边界用 `anyhow::Context` 包裹，最终统一打印错误链。

**参考片段**

- `reference/mempal/src/main.rs:157-173`

```rust
#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;
    let db = Database::open(&expand_home(&config.db_path))?;
```

**Phase 1 应如何改写**

- 可以保留 `run()` 模式，但建议把命令定义与处理迁到 `src/interfaces/cli.rs`，让 `main.rs` 只做启动装配。
- Phase 1 的命令面应收敛为：
  - `init`
  - `status`
  - 可选 `config show` 或 `doctor`
- `main.rs` 不要发展成 `mempal` 那种大型命令分发文件；foundation 阶段先把边界切干净。

---

### `src/core/mod.rs` (provider, transform)

**Analog:** `reference/mempal/src/core/mod.rs`

**可复用模式**

- `core` 只承载基础设施：配置、数据库、迁移、错误、少量 shared helper。
- 领域模型不要直接塞进 `core`。

**参考片段**

- `reference/mempal/src/core/mod.rs:1-7`

```rust
#![warn(clippy::all)]

pub mod config;
pub mod db;
pub mod protocol;
pub mod types;
pub mod utils;
```

**Phase 1 应如何改写**

- 推荐：

```rust
pub mod config;
pub mod db;
pub mod migrations;
```

- 如果需要 shared error，也可以后续加 `error.rs`，但不要把 `memory` record 类型放进 `core/types.rs`。Phase 1 的 typed memory base model 更适合放进 `src/memory/record.rs`。

---

### `src/core/config.rs` (config, file-I/O)

**Analog:** `reference/mempal/src/core/config.rs`

**可复用模式**

- 使用 `serde::Deserialize` + `#[serde(default)]`。
- 配置文件缺失时返回默认配置，而不是报错。
- 保留 `load()` / `load_from()` 双入口。

**参考片段**

- `reference/mempal/src/core/config.rs:9-33`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Config {
    pub db_path: String,
    pub embed: EmbedConfig,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from(&default_config_path())
    }
}
```

**Phase 1 必须采用的新配置模式**

- 不再使用 `embedding.enabled` 布尔开关。
- 推荐把三种模式建成 typed enum：

```rust
pub enum RetrievalMode {
    LexicalOnly,
    EmbeddingOnly,
    Hybrid,
}
```

- 推荐 TOML 形状：

```toml
db_path = "~/.agent-memos/agent-memos.db"

[retrieval]
mode = "lexical_only"

[embedding]
backend = "disabled"
# model = "..."
# api_endpoint = "..."
```

**解释**

- `lexical_only` 对应“没有 embedding model”的状态，仍然允许 ordinary retrieval 以后走 lexical-lightweight 基线。
- `embedding_only` 是合法配置状态，但在 Phase 1 只能被 status/doctor 正确识别和解释，不要求真正执行 embedding 检索。
- `hybrid` 表示“lexical-lightweight + embedding together”，但必须在文案和状态上明确 lexical 仍是主基线，embedding 只是并存的第二通道。
- 这三个值也对应三种使用场景，而不是纯技术开关：
  - `lexical_only` 面向标识符、配置项、错误码、来源过滤和强可解释检索
  - `embedding_only` 面向模糊语义、近义改写、措辞不稳定的召回
  - `hybrid` 面向 mixed corpus 和 agent memory recall，其中 lexical 保底，embedding 补召回或 rerank

**不要复制的 mempal 模式**

- 不要照搬 `DEFAULT_EMBED_BACKEND = "model2vec"` 和 `[embed] backend = "model2vec"` 的默认策略。
- 不要让配置语义退化成“是否开 embedding”；现在锁定要求是三态检索模式。

---

### `src/core/db.rs` (service, CRUD)

**Analog:** `reference/mempal/src/core/db.rs`

**可复用模式**

- `Database` 持有 `rusqlite::Connection` 和物理路径。
- `open()` 内完成目录创建、连接打开、migration 应用。
- `schema_version()`、`path()` 这类状态读取 API 应保留，用于 `status`。

**参考片段**

- `reference/mempal/src/core/db.rs:84-109`

```rust
pub struct Database {
    conn: Connection,
    path: PathBuf,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        apply_migrations(&conn)?;
```

**Phase 1 应如何改写**

- 保留 `Database::open()` 的职责边界。
- `db.rs` 只负责：
  - 连接生命周期
  - migration 执行
  - health/status probe
  - repository 需要的底层 `conn()`
- `db.rs` 不应该变成 mempal 那种把 taxonomy、search、triple、vector 等所有 CRUD 都塞进去的 god object。Phase 1 之后应逐步把 typed CRUD 放进 `src/memory/repository.rs`。

**不要复制的 mempal 模式**

- 不要在 `open()` 里自动执行 `register_sqlite_vec()`；见 `reference/mempal/src/core/db.rs:101` 和 `:799-815`。这会把 Phase 1 绑定到向量扩展生命周期。
- 不要在 Phase 1 创建 `drawer_vectors` 或任何向量表。
- 不要把 FTS 表、FTS trigger、hybrid search 准备逻辑塞进 foundation migration。

---

### `src/core/migrations.rs` (migration, batch)

**Analog:** `reference/mempal/src/core/db.rs`

**可复用模式**

- 用静态 migration 列表 + `PRAGMA user_version` 管理版本。
- 每个 migration 是显式版本号，不用隐式“当前 schema 覆盖一切”的做法。

**参考片段**

- `reference/mempal/src/core/db.rs:706-733`
- `reference/mempal/src/core/db.rs:768-797`

```rust
fn apply_migrations(conn: &Connection) -> Result<(), DbError> {
    let current_version = read_user_version(conn)?;
    for migration in migrations().iter().filter(|m| m.version > current_version) {
        conn.execute_batch(migration.sql)?;
        set_user_version(conn, migration.version)?;
    }
}
```

**Phase 1 应如何改写**

- 推荐把 migration 元数据和 SQL 常量从 `db.rs` 拆到 `src/core/migrations.rs`，这样 planner 后面更容易把 Phase 2 的 lexical index 迁移单独追加。
- Phase 1 migration 只包含：
  - 基础 memory record 表
  - 可能的 provenance 或 source 表
  - 基础索引
  - `user_version`
- 明确不要复制 `reference/mempal/src/core/db.rs:741-766` 的 FTS 触发器 migration。FTS5/`libsimple` 是 Phase 2 检索主线，不是 Phase 1 的 schema critical path。
- 如果 status 需要报告 FTS readiness，Phase 1 只做 capability probe，不做 FTS 实体表落库。

---

### `src/memory/record.rs` (model, CRUD)

**Analog:** `reference/mempal/src/core/types.rs`

**可复用模式**

- 用 typed enum + typed struct，统一 derive `Serialize` / `Deserialize`。
- 把来源、时间、范围等字段放在强类型模型上，而不是 CLI 层的散字段拼装。

**参考片段**

- `reference/mempal/src/core/types.rs:3-23`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    Project,
    Conversation,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Drawer {
    pub id: String,
    pub content: String,
```

**Phase 1 应如何改写**

- 只借用“typed record”的方式，不借用 `Drawer`、`Triple`、`TaxonomyEntry` 这些领域对象。
- 推荐在 `src/memory/record.rs` 放：
  - `MemoryRecord`
  - `MemoryScope`
  - `MemorySource`
  - `RecordKind`
  - `TruthLayer`
  - `ProvenanceRef` 或 `ProvenanceMeta`
- `MemoryRecord` 至少覆盖 `.planning/REQUIREMENTS.md:10-12` 和 `01-CONTEXT.md:39-44` 指定的字段：
  - source
  - timestamp
  - scope
  - record type
  - truth metadata
  - provenance

**不要复制的 mempal 模式**

- 不要把搜索结果结构、知识图谱结构、taxonomy 结构塞到 Phase 1 基础 record 里。
- 不要提前加入 retrieval score、vector dim、RRF rank 等 Phase 2 才需要的字段。

---

### `src/memory/repository.rs` (service, CRUD)

**Analog:** `reference/mempal/src/core/db.rs`

**可复用模式**

- 延续 mempal 的“从 SQLite 行映射回 typed struct”写法，但把 typed memory CRUD 从 `Database` 本体拆出来。
- repository 负责 SQL 和 row mapping；CLI 不接触 SQL。

**参考片段**

- `reference/mempal/src/core/db.rs:120-150`
- `reference/mempal/src/core/db.rs:333-382`

```rust
pub fn insert_drawer(&self, drawer: &Drawer) -> Result<(), DbError> { ... }
pub fn get_drawer(&self, drawer_id: &str) -> Result<Option<Drawer>, DbError> { ... }
```

**Phase 1 应如何改写**

- 新建 `MemoryRepository<'db>` 或 `MemoryRepository`，包装对 `Database`/`Connection` 的访问。
- 初始方法建议：
  - `insert_record`
  - `get_record`
  - `list_records`
  - `count_records`
  - `scope_counts`
- 后续 truth-layer、lexical index、agent search 都通过 repository 或更高层 service 扩展，不要继续往 `Database` 本体堆业务方法。

---

### `src/interfaces/cli.rs` (controller, request-response)

**Analog:** `reference/mempal/src/main.rs`

**可复用模式**

- `clap::{Parser, Subcommand}` 组织 CLI。
- `status` 命令直接调用底层状态 API，输出稳定、无 LLM 依赖的开发者可读结果。

**参考片段**

- `reference/mempal/src/main.rs:30-35`
- `reference/mempal/src/main.rs:828-866`

```rust
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn status_command(db: &Database) -> Result<()> {
    let schema_version = db.schema_version()?;
    println!("schema_version: {schema_version}");
}
```

**Phase 1 必须调整的 status 边界**

- `status` 必须报告三态检索模式，而不是 embedding on/off：
  - `retrieval_mode: lexical_only`
  - `retrieval_mode: embedding_only`
  - `retrieval_mode: hybrid`
- `status` 应拆成几个固定块：
  - `database`: path、是否存在、schema_version、migration_state
  - `retrieval`: mode、lexical_status、embedding_status
  - `dependencies`: SQLite、FTS5 capability、`libsimple`/embedding backend readiness
- Phase 1 下各模式的解释建议：
  - `lexical_only`: 合法；embedding 未配置；ordinary retrieval 基线保留给后续 phase
  - `embedding_only`: 合法；embedding 配置存在，但若后端尚未实现，应标记为 `configured-but-deferred`
  - `hybrid`: 合法；文案必须写清 lexical 为 primary baseline，embedding 为 secondary path

**建议不要照搬**

- 不要在 `main.rs` 里继续膨胀几十个命令。
- 不要让 `status` 只输出数量统计；Phase 1 的关键是“底座 readiness”，不是业务量统计。

## Shared Patterns

### 1. 单二进制优先，先不拆 workspace

**来源**

- `reference/mempal/Cargo.toml:15-23`
- `reference/mempal/src/lib.rs:1-11`
- `.planning/ROADMAP.md:23-36`

**适用范围**

- `Cargo.toml`
- `src/lib.rs`
- `src/main.rs`

**结论**

- Phase 1 优先选择“单 crate + 单 binary + 明确模块边界”，不要为了未来可扩展性提前拆 workspace。
- 理由不是偷懒，而是当前仓库零实现、Phase 1 范围很窄；先把启动、配置、DB、状态跑通，后面再按模块成熟度拆 crate。

### 2. 配置缺省回退 + 三态检索模式

**来源**

- `reference/mempal/src/core/config.rs:9-33`
- `.planning/phases/01-foundation-kernel/01-CONTEXT.md:31-45`

**适用范围**

- `src/core/config.rs`
- `src/interfaces/cli.rs`
- `src/main.rs`

**结论**

- 配置文件缺失时，应回退到默认 TOML 语义。
- 但默认值必须是三态模式中的一种，推荐：
  - `retrieval.mode = "lexical_only"`
  - `embedding.backend = "disabled"`
- 这比 `enabled = false` 更稳定，因为它显式表达了“当前系统处于哪种检索合同”。

### 3. Migration 要显式版本化，但只建基础表

**来源**

- `reference/mempal/src/core/db.rs:706-733`
- `reference/mempal/src/core/db.rs:768-797`
- `.planning/ROADMAP.md:27-30`

**适用范围**

- `src/core/db.rs`
- `src/core/migrations.rs`

**结论**

- 保留 `PRAGMA user_version` 和静态 migration 列表。
- Phase 1 migration 不应提前把 FTS、`libsimple` tokenizer、`sqlite-vec` 表结构写死。
- 后续 Phase 2 再追加 lexical index migration，Phase X 再追加 embedding side-channel migration，避免 foundation 反过来绑架检索实现。

### 4. 领域模型放 `memory/`，不是 `core/`

**来源**

- `reference/mempal/src/core/types.rs:3-74`
- `.planning/research/ARCHITECTURE.md:35-42`
- `.planning/research/ARCHITECTURE.md:100-105`

**适用范围**

- `src/memory/record.rs`
- `src/memory/repository.rs`

**结论**

- `core` 负责 infra，`memory` 负责 typed memory base model。
- Phase 1 如果把 `MemoryRecord` 直接放进 `core/types.rs`，后面很容易把 cognition、truth-layer、search metadata 全塞成一团。

## Mempal Reuse / Do Not Copy

### 应复用

- 单二进制入口和 `lib.rs` 模块图。
- `serde + toml + default fallback` 的本地配置加载模式。
- `Database::open()` 内的目录创建、连接建立、migration 应用顺序。
- `PRAGMA user_version` 驱动的显式 migration 版本管理。
- `status` 命令走稳定状态 API，而不是临时 SQL 拼接。

### 明确不要复制

- `reference/mempal/Cargo.toml:20-23` 的默认 embedding feature 策略。
- `reference/mempal/src/core/config.rs:7` 的默认 embedding backend 思路。
- `reference/mempal/src/core/db.rs:101` 与 `:799-815` 的 `sqlite-vec` 自动注册。
- `reference/mempal/src/core/db.rs:741-766` 的 FTS trigger migration；这属于 Phase 2。
- `reference/mempal/src/core/types.rs` 中 `Drawer` / `Triple` / `TaxonomyEntry` 的领域命名。
- `reference/mempal/src/main.rs` 当前的大型命令面；Phase 1 只需要底座命令。
- `reference/mempal/README_zh.md:12-18` 和 `:113-121` 描述的 hybrid search 产品语义。Agent Memos 当前是 lexical-first foundation，不是 RRF 混合检索成品。

## No Exact Analog Found

| File | Role | Data Flow | Reason |
|---|---|---|---|
| `src/memory/record.rs` | model | CRUD | `mempal` 有 typed storage structs，但没有符合 0415 理论和 FND-02 的基础 memory record 语义 |
| `src/memory/repository.rs` | service | CRUD | `mempal` 把大部分数据访问塞进 `Database`，没有单独的 memory repository 分层 |
| `src/interfaces/cli.rs` | controller | request-response | `mempal` 直接把 CLI 放在 `main.rs`，Phase 1 更适合先把命令边界拆开 |

## Recommendation

Phase 1 的首选模式是：**单 crate 单二进制 + `lib.rs` 模块图 + `core` 只管配置/数据库/migration + `memory` 单独承载 typed base record + `interfaces/cli` 负责 `init/status` 边界**。

配置上首选：**`[retrieval].mode = "lexical_only" | "embedding_only" | "hybrid"` 三态枚举，默认 `lexical_only`；`[embedding]` 只描述后端与参数，不再承担“是否启用”的主语义**。命令和状态上首选：**`status` 明确报告 retrieval mode、lexical readiness、embedding readiness 和 deferred 状态，并在 `hybrid` 模式下明确 lexical 是 primary baseline、embedding 是 secondary path**。
