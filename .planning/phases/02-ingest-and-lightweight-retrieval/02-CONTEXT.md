# Phase 2: Ingest And Lightweight Retrieval - Context

**Gathered:** 2026-04-15
**Status:** Ready for planning
**Source:** Roadmap, requirements, Phase 1 outputs, and project research

<domain>
## Phase Boundary

Phase 2 delivers the ordinary retrieval mainline on top of the Phase 1 foundation. It is responsible for:
- ingesting notes, documents, and conversation-like text into normalized memory units
- preserving source linkage and chunk provenance during ingest
- creating the lexical-first retrieval path using `libsimple` + SQLite FTS5
- adding Rust-side lightweight keyword/scoring logic over recalled candidates
- returning ordinary retrieval results with citations, scope, validity metadata, and explanation-friendly traces
- exposing ordinary retrieval through CLI and/or library surfaces without requiring any LLM or Rig runtime

This phase is not responsible for:
- embedding-based retrieval execution
- hybrid lexical/embedding merge logic beyond preserving extension seams
- Rig-based agent search orchestration
- T1/T2/T3 promotion/governance logic beyond carrying existing truth metadata through retrieval
- working-memory assembly, metacognition, or rumination

</domain>

<decisions>
## Implementation Decisions

### Locked decisions
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

### the agent's Discretion
- Exact module/file split within `ingest/` and `search/`, as long as it builds cleanly on the current Phase 1 crate structure.
- Exact shape of chunk metadata and source identifiers, as long as provenance, source linkage, and time/scope filters remain explicit.
- Specific CLI command names for ingest and search, as long as they are thin wrappers over internal services.
- Exact scoring breakdown fields, as long as they remain inspectable and support citations/explanations.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and requirements
- `.planning/ROADMAP.md` - Phase 2 goal, success criteria, and plan slots
- `.planning/REQUIREMENTS.md` - `ING-01`, `ING-02`, `ING-03`, `RET-01`, `RET-02`, `RET-03`, `RET-04`, `RET-05`, `AGT-01`
- `.planning/PROJECT.md` - lexical-first baseline, coexistence rule, local-first constraints
- `.planning/STATE.md` - current project state and carry-over concerns

### Prior phase outputs
- `.planning/phases/01-foundation-kernel/01-01-SUMMARY.md` - typed config and retrieval-mode contract
- `.planning/phases/01-foundation-kernel/01-02-SUMMARY.md` - current SQLite schema and memory repository
- `.planning/phases/01-foundation-kernel/01-03-SUMMARY.md` - status/doctor/inspect command surface
- `.planning/phases/01-foundation-kernel/01-04-SUMMARY.md` - informational diagnostics and truthful init/status behavior

### Project research
- `.planning/research/STACK.md` - recommended Phase 2 stack around `libsimple`, FTS5, and Rust-side scoring
- `.planning/research/ARCHITECTURE.md` - recommended `ingest/` and `search/` module boundaries
- `.planning/research/SUMMARY.md` - lexical-first implementation rationale and pitfalls

### Domain theory
- `doc/0415-00记忆认知架构.md` - recall vs cognition boundary
- `doc/0415-真值层.md` - truth metadata expectations that retrieval must preserve

### Reference implementation
- `reference/mempal/README_zh.md` - structural inspiration for ingest/search CLI flow
- `reference/mempal/src/ingest/mod.rs` - ingest module reference
- `reference/mempal/src/ingest/detect.rs` - source detection reference
- `reference/mempal/src/ingest/normalize.rs` - normalization reference
- `reference/mempal/src/ingest/chunk.rs` - chunking reference
- `reference/mempal/src/search/mod.rs` - search service layout reference

</canonical_refs>

<specifics>
## Specific Ideas

- Prefer a Phase 2 module expansion like:

```text
src/
├── ingest/
│   ├── detect.rs
│   ├── normalize.rs
│   ├── chunk.rs
│   └── mod.rs
├── search/
│   ├── lexical.rs
│   ├── score.rs
│   ├── rerank.rs
│   ├── citation.rs
│   └── mod.rs
```

- Phase 2 should probably keep `lexical_only` as the only fully ready retrieval execution mode, while `embedding_only` / `hybrid` stay declared but not implemented.
- Retrieval explainability should expose at least:
  - lexical match basis
  - Rust-side bonus/scoring contribution
  - source and provenance anchors
  - scope and validity filters applied
- Ingest should handle at least notes, plain documents, and conversation-like text in a way that can be extended later, without overcommitting to every future format now.

</specifics>

<deferred>
## Deferred Ideas

- semantic retrieval execution and merge contracts
- Rig wiring and agent-search orchestration
- truth-layer governance and promotion gates
- working-memory assembly and metacognitive checks

</deferred>

---

*Phase: 02-ingest-and-lightweight-retrieval*
*Context gathered: 2026-04-15 via roadmap, requirements, research, and Phase 1 outputs*
