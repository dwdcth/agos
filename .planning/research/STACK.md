# Stack Research

**Domain:** Rust local-first memory cognition system for AI agents
**Researched:** 2026-04-15
**Confidence:** MEDIUM

## Recommended Stack

### Core Technologies

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust | 1.85+ | Core implementation language | `libsimple 0.9.0` declares `rust-version = 1.85.0`, and the project needs strong type boundaries for cognitive layers plus a single-binary deployment path. |
| SQLite + rusqlite | `rusqlite 0.37.x` | Primary local data store and query engine | Fits the local-first constraint, composes well with FTS5 and vector extensions, and is already proven by `reference/mempal`. |
| sqlite-vec | `0.1.x` | Vector similarity search inside SQLite | Official project positions it as a small vector-search extension that runs wherever SQLite runs, which matches this project's deployment model. |
| libsimple | `~0.9` | Chinese / PinYin FTS5 tokenizer | Official crate metadata describes it as Rust bindings for a SQLite FTS5 tokenizer with Chinese and PinYin support, which directly addresses Chinese retrieval quality. |
| rig-core | Latest compatible release at implementation time | Agent orchestration, model abstraction, tools, embeddings | Official Rig docs emphasize provider unification, agent workflows, and vector-store integrations, making it the right orchestration layer for agentic search. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio | `1.x` | Async runtime | Needed for rig-based agent workflows, embedding calls, background rumination jobs, and optional MCP/API surfaces. |
| serde / serde_json | `1.x` | Typed persistence and API payloads | Use for cognitive state snapshots, search responses, traces, and schema evolution metadata. |
| thiserror / anyhow | `2.x / 1.x` | Error modeling and propagation | Use `thiserror` for domain errors and `anyhow` at interface boundaries or CLI entrypoints. |
| tracing / tracing-subscriber | `0.1 / 0.3` | Observability | Required to debug retrieval decisions, agent reasoning routes, and rumination jobs. |
| clap | `4.x` | CLI interface | Use for early product surface, parity with `mempal`, and inspection/debug workflows. |
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
cargo add rusqlite@0.37 sqlite-vec@0.1 tokio@1 serde serde_json anyhow thiserror tracing tracing-subscriber clap@4

# Required lexical search support
cargo add libsimple@~0.9

# Agentic search
cargo add rig-core

# Optional service surface
cargo add axum@0.8
```

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| SQLite + `sqlite-vec` | External vector DB | Only if local-first is abandoned and multi-tenant scale becomes the primary goal. |
| `libsimple` FTS5 | Pure BM25 tokenization without Chinese support | Only if the corpus is guaranteed to be English-only. |
| `rig-core` as orchestration layer | Hand-rolled provider adapters | Only for an extremely narrow single-provider prototype; otherwise the abstraction cost pays for itself. |
| Custom hybrid-search layer on top of SQLite | `rig-sqlite` as the primary data model | Use `rig-sqlite` only as a later adapter if it fits; the core store here needs richer truth layers and lexical fusion than a vector-store-first model. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Vector-only retrieval as the sole search path | Chinese lexical recall, exact-match decisions, and symbolic cues will be lost | Hybrid retrieval with `libsimple` + `sqlite-vec` + explicit reranking |
| Letting Rig own the primary memory schema too early | The project's differentiator is cognitive modeling, not generic RAG plumbing | Keep Rig at the orchestration/tool layer and own the core schema directly |
| Cloud-first infra in v1 | Violates the local-first constraint and adds operational drag before the cognition model is proven | Single-file SQLite-first deployment |
| Blindly copying `reference/mempal` crate-for-crate | `mempal` is a memory product reference, but this project needs extra layers for T1/T2/T3, working memory, metacognition, and rumination | Reuse its module discipline, not its exact domain model |

## Stack Patterns by Variant

**If the first release is CLI-first:**
- Use `clap` + `rusqlite` + direct service objects
- Because it keeps the feedback loop tight while retrieval and cognition semantics are still moving

**If Phase 4 introduces remote or tool access:**
- Add `axum` or MCP bindings around the same application services
- Because interface expansion should not rewrite the retrieval and cognition core

**If agent search needs Rig-native retrieval adapters later:**
- Add a thin `rig` adapter module over the internal search/working-memory services
- Because the core ranking and truth-layer semantics should stay stable even if agent tooling changes

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `libsimple 0.9.0` | `rusqlite >=0.32,<1.0` | Verified from the crate metadata on docs.rs. |
| `sqlite-vec 0.1.x` | SQLite / rusqlite with extension loading | Needs explicit extension initialization before vector queries. |
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
