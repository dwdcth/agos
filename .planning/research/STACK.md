# Stack Research

**Domain:** Rust local-first memory cognition system for AI agents
**Researched:** 2026-04-15
**Confidence:** MEDIUM

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust | 1.85+ | Core implementation language | `libsimple 0.9.0` declares `rust-version = 1.85.0`, and the project needs strong type boundaries for cognitive layers plus a single-binary deployment path. |
| SQLite + rusqlite | `rusqlite 0.37.x` | Primary local data store and query engine | Fits the local-first constraint, ships well as a single-machine dependency, and composes naturally with FTS5. |
| SQLite FTS5 + libsimple | `libsimple ~0.9` | Chinese / PinYin lexical retrieval and BM25 base ranking | This gives a strong lexical baseline without model files, which matches the new lightweight-first retrieval strategy. |
| Rust lightweight scorer | std + optional small utility crates | BM25/TF-IDF-style weights, context bonus rules, emotion/importance/recency rerank | Direct Rust scoring is the right translation of the Python prototype: simple, inspectable, and cheap at small corpus sizes. |
| rig-core | Latest compatible release at implementation time | Agent orchestration, model abstraction, tools | Rig remains the right orchestration layer for agentic search even when the retrieval baseline is lexical-first. |
| sqlite-vec (optional) | `0.1.x` | Future semantic-retrieval extension | Keep as an opt-in extension path if lexical-first retrieval later shows recall gaps. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio | `1.x` | Async runtime | Needed for rig-based agent workflows, background rumination jobs, and optional MCP/API surfaces. |
| serde / serde_json | `1.x` | Typed persistence and API payloads | Use for cognitive state snapshots, search responses, traces, and schema evolution metadata. |
| thiserror / anyhow | `2.x / 1.x` | Error modeling and propagation | Use `thiserror` for domain errors and `anyhow` at interface boundaries or CLI entrypoints. |
| tracing / tracing-subscriber | `0.1 / 0.3` | Observability | Required to debug retrieval decisions, agent reasoning routes, and rumination jobs. |
| clap | `4.x` | CLI interface | Use for early product surface, parity with `mempal`, and inspection/debug workflows. |
| regex | `1.x` | Optional lightweight token normalization | Use only if Rust-side bonus scoring needs extra token extraction beyond what FTS5 already handles. |
| axum | `0.8.x` | Optional HTTP / MCP-adjacent service surface | Add once search and agent workflows are stable and need remote invocation. |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| cargo fmt / clippy | Style and lint gates | Keep module boundaries clean while the model is still changing quickly. |
| cargo nextest or cargo test | Verification | Needed once truth-layer promotion, search fusion, and agent-search orchestration become test-heavy. |
| sqlite3 CLI / DB Browser | Inspect SQLite schema and FTS/vec behavior | Useful during schema and ranking tuning. |

## Installation

```bash
# Core
cargo add rusqlite@0.37 tokio@1 serde serde_json anyhow thiserror tracing tracing-subscriber clap@4

# Required lexical search support
cargo add libsimple@~0.9

# Optional lightweight text helpers
cargo add regex@1

# Agentic search
cargo add rig-core

# Optional service surface
cargo add axum@0.8

# Optional semantic extension
cargo add sqlite-vec@0.1
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| FTS5 + `libsimple` + Rust lightweight scorer | `sqlite-vec` semantic retrieval | Use `sqlite-vec` only if lexical-first retrieval later proves insufficient for recall quality. |
| `libsimple` FTS5 | Pure BM25 tokenization without Chinese support | Only if the corpus is guaranteed to be English-only. |
| `rig-core` as orchestration layer | Hand-rolled provider adapters | Only for an extremely narrow single-provider prototype; otherwise the abstraction cost pays for itself. |
| Custom lexical-first search layer on top of SQLite | `rig-sqlite` as the primary data model | Use `rig-sqlite` only as a later adapter if it fits; the core store here needs richer truth layers and retrieval governance than a vector-store-first model. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Making vector search a v1 prerequisite | It adds model/extension complexity before the lexical baseline is proven | Start with `libsimple` + FTS5 + Rust lightweight rerank |
| Letting Rig own the primary memory schema too early | The project's differentiator is cognitive modeling, not generic RAG plumbing | Keep Rig at the orchestration/tool layer and own the core schema directly |
| Cloud-first infra in v1 | Violates the local-first constraint and adds operational drag before the cognition model is proven | Single-file SQLite-first deployment |
| Blindly copying `reference/mempal` crate-for-crate | `mempal` is a memory product reference, but this project needs extra layers for T1/T2/T3, working memory, metacognition, and rumination | Reuse its module discipline, not its exact domain model |

## Stack Patterns by Variant

**If the first release is CLI-first:**
- Use `clap` + `rusqlite` + direct service objects
- Because it keeps the feedback loop tight while retrieval and cognition semantics are still moving

**If the first release wants zero model files:**
- Keep ranking to FTS5 BM25 plus Rust-side keyword, emotion, importance, and recency bonuses
- Because this directly mirrors the lightweight Python idea while staying easy to test in Rust

**If Phase 4 introduces remote or tool access:**
- Add `axum` or MCP bindings around the same application services
- Because interface expansion should not rewrite the retrieval and cognition core

**If lexical-first retrieval later needs semantic help:**
- Add a thin `sqlite-vec` adapter behind the same retrieval interface
- Because semantic retrieval should be an extension, not a schema-defining prerequisite

**If agent search needs Rig-native retrieval adapters later:**
- Add a thin `rig` adapter module over the internal search/working-memory services
- Because the core ranking and truth-layer semantics should stay stable even if agent tooling changes

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `libsimple 0.9.0` | `rusqlite >=0.32,<1.0` | Verified from the crate metadata on docs.rs. |
| SQLite FTS5 BM25 | SQLite with FTS5 enabled | Forms the default retrieval baseline with no model files. |
| `sqlite-vec 0.1.x` | SQLite / rusqlite with extension loading | Keep behind an optional feature gate if semantic retrieval is added later. |
| `rig-core` latest compatible | Matching Rig companion crates only when needed | Pin `rig-core` and any `rig-*` companions together during implementation. |

## Sources

- `doc/0415-00记忆认知架构.md` — project-specific domain theory and system boundaries
- `reference/mempal/README_zh.md` and `reference/mempal/Cargo.toml` — proven Rust local-first memory architecture reference
- https://docs.rs/crate/libsimple/latest — confirmed `libsimple 0.9.0`, Rust version, compatibility, and tokenizer scope
- https://github.com/asg017/sqlite-vec — confirmed project positioning, deployment model, and pre-v1 status
- https://github.com/0xPlaygrounds/rig and https://docs.rs/crate/rig-core/latest — confirmed Rig's agent/provider/vector-store abstractions

---
*Stack research for: Rust local-first memory cognition system*
*Researched: 2026-04-15*
