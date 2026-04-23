# Phase 1: Foundation Kernel - Context

**Gathered:** 2026-04-15
**Status:** Ready for planning
**Source:** Project initialization + explicit user guidance

<domain>
## Phase Boundary

Phase 1 delivers the local-first Rust foundation that later ingest, lexical retrieval, truth-layer governance, and agent search will build on.

This phase is responsible for:
- a runnable single-binary Rust application skeleton
- SQLite database bootstrap and migration entrypoints
- typed base memory structures that preserve source, time, scope, record type, and truth metadata
- startup and status inspection commands for developers
- configuration loading for core runtime options

This phase is not responsible for:
- full ingest pipelines
- `libsimple` lexical retrieval behavior
- semantic/vector retrieval implementation
- Rig agent workflows
- rumination, promotion gates, or full truth-layer governance logic

</domain>

<decisions>
## Implementation Decisions

### Locked decisions
- Configuration must use TOML as the primary local config format.
- Phase 1 should define a typed config surface early, so later phases can extend it without format churn.
- Config must express retrieval strategy as three explicit modes rather than a single boolean:
  - lexical-only, meaning no embedding model
  - embedding-only
  - lexical-lightweight plus embedding together
- Phase 1 should model this as a stable typed enum/string mode in config, even if only part of the behavior is exercised immediately.
- Embedding must remain optional in foundation work, because ordinary retrieval v1 does not require model files or embedding services.
- Startup and status checks must treat all three configured modes as valid states and explain what is currently active, unavailable, or deferred rather than collapsing them into a generic enabled/disabled flag.
- The system should preserve a clean extension path for later optional embedding backends without forcing vector infrastructure into the Phase 1 critical path.
- The config and service boundaries should leave room for later coexistence between lexical-first retrieval and embedding-based retrieval under one search surface, with embedding remaining a secondary path for expansion or rerank rather than replacing the lexical baseline when hybrid mode is used.
- The three modes are not only implementation toggles; they represent different retrieval intents and should be documented accordingly:
  - `lexical_only` for precise keyword, identifier, source-aware, and strongly explainable retrieval
  - `embedding_only` for semantic-intent-heavy recall where wording is unstable but exact explainability is weaker
  - `hybrid` for mixed corpora or mixed query styles, where lexical remains the baseline explanation source and embedding acts as a secondary expansion or rerank path
- The data model created in Phase 1 must already preserve source, timestamp, scope, record type, and truth metadata required by FND-02.
- The schema and service boundaries should make later T1/T2/T3 specialization additive rather than forcing a storage rewrite.

### the agent's Discretion
- Exact TOML section names and field naming, as long as they are clear and stable.
- Exact enum values for the three retrieval modes, as long as they clearly distinguish no-embedding, embedding-only, and hybrid operation.
- How much of the future coexistence contract is reflected in Phase 1 config, as long as Phase 1 only reserves extension seams and does not prematurely implement semantic retrieval.
- Whether the application is a single crate or a small workspace, as long as it preserves a mempal-like modular separation and a single binary entrypoint.
- Specific migration tooling choice and CLI command naming.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and requirements
- `.planning/ROADMAP.md` - Phase 1 goal, success criteria, and plan slots
- `.planning/REQUIREMENTS.md` - `FND-01`, `FND-02`, `FND-03`
- `.planning/PROJECT.md` - project constraints and architectural direction
- `.planning/STATE.md` - current workflow state and existing concerns

### Domain theory
- `doc/0415-00记忆认知架构.md` - retrieval vs cognition boundary and memory as structured future-oriented state
- `doc/0415-真值层.md` - T1/T2/T3 layering and provenance expectations
- `doc/0415-理论吸收清单.md` - config surface as an intentional engineering carrier, including TOML-based parameter families

### Reference implementation
- `reference/mempal/README_zh.md` - local-first single-binary Rust reference, TOML config pattern, optional embedding backends
- `reference/mempal/Cargo.toml` - crate/features layout and dependency discipline
- `reference/mempal/src/core/config.rs` - TOML config loading pattern
- `AGENTS.md` - updated coexistence rule for lexical-first and embedding retrieval

</canonical_refs>

<specifics>
## Specific Ideas

- Favor a minimal Phase 1 config shape like:

```toml
db_path = "~/.agent-memos/agent-memos.db"

[retrieval]
mode = "lexical_only"

[embedding]
backend = "disabled"
```

- Or, if the planner prefers a more explicit three-mode surface:

```toml
[retrieval]
mode = "lexical_only"   # no embedding model
# mode = "embedding_only" # embedding-only
# mode = "hybrid"         # lexical-lightweight + embedding
```

- The TOML shape should make it easy to evolve later into a lexical baseline plus optional semantic side-channel without changing the config format again.

- Retrieval-mode documentation should state the intended fit:
  - `lexical_only` is the default and best fit for identifiers, config keys, error text, dates, file paths, and source-sensitive retrieval
  - `embedding_only` fits fuzzy semantic recall and unstable wording, but should be treated as weaker on exact filters and explainability
  - `hybrid` fits agent memory recall and mixed corpora, but lexical should still anchor citations and primary explanations

- Status output should expose enough detail to answer:
  - database path / existence
  - schema version
  - migration readiness
  - FTS dependency readiness when relevant
  - retrieval mode (`lexical_only`, `embedding_only`, `hybrid`, or the final chosen names)
  - embedding backend status (`disabled`, configured, missing dependency, deferred)

- Base memory entities should not hardcode Phase 2 retrieval scoring fields yet, but should leave room for later indexing and truth-governance extensions.

</specifics>

<deferred>
## Deferred Ideas

- Concrete embedding backend integration
- lexical + semantic coexistence merge logic
- `sqlite-vec` schema and reindex flows
- lexical recall and reranking implementation
- Rig wiring and agent-search orchestration

</deferred>

---

*Phase: 01-foundation-kernel*
*Context gathered: 2026-04-15 via project docs and user guidance*
