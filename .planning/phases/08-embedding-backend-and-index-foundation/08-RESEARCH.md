# Phase 8: Embedding Backend And Index Foundation - Research

**Researched:** 2026-04-16  
**Domain:** optional embedding backend, additive embedding persistence, vector sidecar/index foundation  
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
### Retrieval Role And Default Behavior
- **D-01:** Lexical-first remains the default and required retrieval baseline.
- **D-02:** The embedding path is optional in v1.1 and must be able to remain fully disabled without harming lexical-only operation.
- **D-03:** Phase 8 establishes embedding capability and storage only; it does not yet make embedding results participate in final retrieval fusion.

### Backend And Config Shape
- **D-04:** Embedding backend state must remain explicit and typed, not hidden behind a boolean `enabled` flag.
- **D-05:** Config must allow a developer to specify backend + model information while preserving the existing lexical-only defaults.
- **D-06:** Runtime diagnostics must distinguish between at least these states:
  - disabled intentionally
  - configured but not ready
  - ready for use

### Persistence And Index Boundary
- **D-07:** Embedding persistence must be additive side-table state, not a replacement for `memory_records` as the authority store.
- **D-08:** Embeddings should align with the current retrieval/citation grain, so chunk-level or equivalent retrieval-unit persistence is preferred over whole-document-only vectors.
- **D-09:** Vector index/sidecar state must remain optional and local-first, fitting the existing single-machine SQLite deployment model.

### Explainability And Service Boundary
- **D-10:** Phase 8 must preserve the project’s explainability rule: lexical citations and authority records remain the primary explanation source even after embedding capability exists.
- **D-11:** Ordinary retrieval and agent-search should continue to depend on shared internal retrieval services, not on a separate semantic-only bypass path.

### the agent's Discretion
- Exact embedding backend enum variants beyond `disabled`, as long as the config/runtime contract stays typed and inspectable.
- Exact embedding table names, index schema, and migration split.
- Exact backend invocation abstraction (trait/service/module shape) so long as it remains optional and local-first compatible.

### Deferred Ideas (OUT OF SCOPE)
- lexical/embedding fusion and rerank policy
- agent-search-specific semantic orchestration changes
- MCP / HTTP exposure of embedding controls
- richer provider-specific embedding lifecycle management
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| EMB-01 | Developer can configure an optional embedding backend and model without weakening the existing lexical-only baseline. [VERIFIED: `.planning/REQUIREMENTS.md`] | The current config/readiness path already models retrieval/embedding state explicitly; Phase 8 should extend those enums and readiness states rather than replacing them with booleans. [VERIFIED: `src/core/config.rs`; VERIFIED: `src/core/app.rs`; VERIFIED: `src/core/status.rs`] |
| EMB-02 | System can generate and persist embeddings for ingested records or chunks in additive storage structures. [VERIFIED: `.planning/REQUIREMENTS.md`] | Current ingest already yields chunk-aligned authority rows; additive chunk-level embedding side tables fit the project’s schema-evolution pattern and align with citation grain. [VERIFIED: `src/ingest/mod.rs`; VERIFIED: `src/ingest/chunk.rs`; VERIFIED: `tests/foundation_schema.rs`] |
| EMB-03 | System can maintain an optional vector sidecar or index for embedding-backed retrieval without redefining the authority store. [VERIFIED: `.planning/REQUIREMENTS.md`] | Existing migrations and status inspection already treat lexical sidecars as additive local indexes; vector sidecars should mirror that pattern rather than replace `memory_records`. [VERIFIED: `src/core/migrations.rs`; VERIFIED: `src/core/status.rs`; VERIFIED: `.planning/research/STACK.md`] |
</phase_requirements>

## Summary

Phase 8 should be treated as infrastructure work for semantic capability, not as “semantic retrieval shipped.” The repo already has the right architectural precedent: typed config in `src/core/config.rs`, runtime readiness derivation in `src/core/app.rs`, capability-state inspection in `src/core/status.rs`, and additive side-table schema evolution across prior phases. The cleanest foundation is to extend those seams for embeddings while keeping lexical-first intact and optional semantic capability explicit. [VERIFIED: `src/core/config.rs`; VERIFIED: `src/core/app.rs`; VERIFIED: `src/core/status.rs`; VERIFIED: `.planning/phases/08-embedding-backend-and-index-foundation/08-CONTEXT.md`]

The best persistence grain is the same one current retrieval already uses: chunk-aligned retrieval units. `IngestService` already normalizes source content into chunked `MemoryRecord`s and persists chunk metadata/citation anchors. Storing embeddings additively against those retrieval units means future dual-channel fusion can stay consistent with current citation and snippet contracts instead of introducing a document-level semantic layer that no longer lines up with retrieved authority records. [VERIFIED: `src/ingest/mod.rs`; VERIFIED: `src/ingest/chunk.rs`; VERIFIED: `.planning/PROJECT.md`]

**Primary recommendation:** in Phase 8, add typed embedding backend/model config, additive chunk-level embedding tables plus optional vector sidecar/index schema, and truthful readiness reporting for the new capability. Do not yet modify `SearchService` ranking behavior beyond exposing the existence/readiness of the embedding substrate. [ASSUMED]

## Current Code Findings

### 1. The config seam is already prepared for typed extension

- `EmbeddingBackend` exists today with `Disabled` and `Reserved`. [VERIFIED: `src/core/config.rs`]
- `EmbeddingConfig` already carries `backend`, `model`, and `endpoint`. [VERIFIED: `src/core/config.rs`]
- `RuntimeReadiness::from_config(...)` already keeps lexical/embedding intent explicit rather than flattening it into one boolean. [VERIFIED: `src/core/app.rs`]

**Implication:** Phase 8 should extend existing embedding states and readiness notes, not replace the config model.

### 2. Status/doctor already have the capability-state pattern Phase 8 needs

- `StatusReport` separately tracks lexical dependency state, embedding dependency state, and index readiness. [VERIFIED: `src/core/status.rs`]
- `DoctorReport` already turns impossible or reserved combinations into structured failures/warnings by command path. [VERIFIED: `src/core/doctor.rs`]
- Phase 6 proved that operational gating can be made consistent once status facts are truthful. [VERIFIED: `.planning/phases/06-runtime-gate-enforcement/06-VERIFICATION.md`]

**Implication:** embedding readiness should be represented as capability-state + notes first, then leveraged later by phase-specific runtime behavior.

### 3. Ingest already provides chunk-aligned storage and deterministic record IDs

- `chunk_source(...)` produces retrieval-unit drafts with anchors and hashes. [VERIFIED: `src/ingest/chunk.rs`]
- `IngestService::ingest(...)` persists chunk-aligned `MemoryRecord`s with chunk metadata and deterministic IDs. [VERIFIED: `src/ingest/mod.rs`]

**Implication:** Embedding persistence should target chunk-aligned retrieval units (`record_id` or equivalent) so future dual-channel fusion can reuse existing snippets/citations.

## Recommended Foundation Direction

### Config and readiness

Recommended exact direction:

- Extend `EmbeddingBackend` beyond `Disabled` / `Reserved` with one or more implementation-ready backends (for example a local HTTP embedding service and/or a built-in vector path).
- Keep the current default `Disabled`.
- Expand `RuntimeReadiness`, `StatusReport`, and `DoctorReport` notes so operators can tell:
  - embedding disabled intentionally
  - embedding configured but backend unreachable/misconfigured
  - embedding storage/index missing
  - embedding substrate ready

Do not use a plain `embedding_enabled: bool`. [ASSUMED]

### Schema and storage

Recommended additive schema split:

- `record_embeddings`
  - keyed by `record_id`
  - stores backend/model metadata, vector dimensionality, and embedding payload reference
- `record_embedding_index_state` or equivalent
  - captures bootstrap/build state for the optional vector sidecar/index

Keep `memory_records` as the authority store. The embedding tables should reference `memory_records(id)` and never replace it. [ASSUMED]

### Backend abstraction

Recommended exact shape:

- add a narrow embedding service trait/module that can:
  - compute embeddings for chunk text
  - report backend readiness
  - expose model/dimension metadata

This service should be invoked from ingest-adjacent flows or explicit bootstrap flows, not from `SearchService` yet. [ASSUMED]

### Vector sidecar/index

Recommended Phase 8 scope:

- create/bootstrap optional vector sidecar/index schema
- persist enough metadata to know whether it is built and usable
- avoid retrieval fusion/ranking logic until Phase 9

This keeps Phase 8 focused on foundation and diagnostics rather than leaking into dual-channel behavior prematurely. [VERIFIED: `.planning/ROADMAP.md`]

## Testing Direction

### Best regression split

1. **Config/status tests**
   - extend existing config/status CLI coverage for embedding backend states and vector readiness

2. **Schema/persistence tests**
   - extend `tests/foundation_schema.rs` or add a targeted embedding-schema integration test
   - verify additive embedding tables and vector sidecar bootstrap

3. **Ingest/persistence tests**
   - verify chunk-aligned embedding persistence and backend metadata persistence

This matches the repo’s established “typed config + additive schema + integration test” pattern. [VERIFIED: `tests/status_cli.rs`; VERIFIED: `tests/foundation_schema.rs`; VERIFIED: `tests/ingest_pipeline.rs`]

## Anti-Patterns To Avoid

- **Do not make `embedding_only` / `hybrid` silently runnable by accident during foundation work.** Phase 8 should make semantic capability explicit, not magically change runtime semantics before fusion is implemented. [VERIFIED: `.planning/phases/06-runtime-gate-enforcement/06-CONTEXT.md`]
- **Do not persist vectors inside `memory_records` blobs or overload provenance fields.** Additive side tables are the project pattern. [VERIFIED: `.planning/phases/03-truth-layer-governance/03-01-SUMMARY.md`]
- **Do not store document-level embeddings only if retrieval/reporting remains chunk-based.** That would weaken explainability and future alignment. [ASSUMED]
- **Do not let agent-search build a semantic-only side path before ordinary retrieval supports it.** Shared retrieval services remain the contract. [VERIFIED: `.planning/PROJECT.md`; `.planning/phases/08-embedding-backend-and-index-foundation/08-CONTEXT.md`]

## Recommended Plan Shape

Phase 8 cleanly fits three plans:

1. **Plan 08-01:** extend config/runtime readiness/status for real embedding backend states.
2. **Plan 08-02:** persist chunk-level embeddings additively at ingest/storage seams.
3. **Plan 08-03:** add optional vector sidecar/index bootstrap and inspectable readiness state.

This split maps directly to the roadmap and keeps “semantic foundation” separate from “dual-channel retrieval behavior,” which belongs to Phase 9. [VERIFIED: `.planning/ROADMAP.md`]
