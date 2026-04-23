# Phase 8: Embedding Backend And Index Foundation - Context

**Gathered:** 2026-04-16
**Status:** Ready for planning
**Source:** v1.1 milestone definition, existing retrieval/runtime constraints, and zero-friction discuss defaults derived from the current codebase

<domain>
## Phase Boundary

Phase 8 establishes the optional embedding second-channel foundation without changing the project’s lexical-first identity. It is responsible for:
- adding configurable embedding backend/model contracts
- adding additive embedding persistence for ingested records or chunks
- introducing an optional vector sidecar/index bootstrap path that fits the existing local-first SQLite deployment model
- extending readiness/state reporting so the embedding foundation can be inspected truthfully

This phase is not responsible for:
- making embedding the primary retrieval path
- implementing dual-channel fusion/rerank logic across lexical and embedding channels
- redesigning agent-search orchestration or truth governance
- introducing new MCP/HTTP interface surfaces

</domain>

<decisions>
## Implementation Decisions

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

</decisions>

<specifics>
## Specific Ideas

- Treat Phase 8 as “make semantic capability real”, not “turn semantic retrieval on by default”.
- Prefer chunk-aligned embedding storage so future fusion can stay compatible with current citation/report contracts.
- Keep the operator story explicit: if a backend or vector sidecar is missing, diagnostics should say so directly.
- Phase 8 should feel like the semantic equivalent of Phase 2/6 groundwork, not like a product-surface expansion.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone scope
- `.planning/PROJECT.md` — current v1.1 milestone goal, active requirements, and constraints
- `.planning/ROADMAP.md` — Phase 8 goal, success criteria, and plan slots
- `.planning/REQUIREMENTS.md` — `EMB-01`, `EMB-02`, and `EMB-03`
- `.planning/STATE.md` — current milestone state

### Prior retrieval/runtime outputs
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-01-SUMMARY.md` — ingest normalization/chunk persistence baseline
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-02-SUMMARY.md` — lexical retrieval/readiness foundation
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md` — search result contract, citations, and filters
- `.planning/phases/06-runtime-gate-enforcement/06-CONTEXT.md` — runtime gate and diagnostic boundary that embedding readiness must fit into
- `.planning/phases/06-runtime-gate-enforcement/06-VERIFICATION.md` — operational diagnostics and readiness enforcement baseline

### Stack and constraint references
- `.planning/research/STACK.md` — optional `sqlite-vec` path, Rig positioning, local-first constraints
- `.planning/milestones/v1.0-ROADMAP.md` — completed milestone baseline for retrieval/cognition contracts

### Runtime/code seams
- `src/core/config.rs` — current typed retrieval/backend config surface
- `src/core/app.rs` — runtime readiness derivation
- `src/core/status.rs` — readiness snapshot / capability-state reporting
- `src/core/doctor.rs` — command-path gating policy
- `src/ingest/mod.rs` and `src/ingest/chunk.rs` — ingest grain and chunk lifecycle
- `src/search/mod.rs` and `src/search/lexical.rs` — current ordinary retrieval service boundary
- `src/interfaces/cli.rs` — current diagnostic and operational surfaces

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/core/config.rs` already has typed retrieval/embedding config structures and is the natural seam for extending backend/model configuration.
- `src/core/status.rs` and `src/core/doctor.rs` already encode capability-state and readiness-reporting patterns that embedding readiness should reuse.
- `src/ingest/*` already chunks and persists retrieval units, which is the natural grain for additive embedding storage.

### Established Patterns
- additive side-table evolution over the authority store
- typed config and capability-state diagnostics
- lexical-first baseline preserved even when optional future capabilities exist
- thin CLI wrappers over internal service seams

### Integration Points
- embedding persistence should hang off existing ingest completion, not bypass it
- vector sidecar/index bootstrap should fit the same migration/bootstrap pipeline used for prior additive schema phases
- future dual-channel retrieval (Phase 9) should be able to reuse the embedding storage introduced here without changing the authority record contract

</code_context>

<deferred>
## Deferred Ideas

- lexical/embedding fusion and rerank policy
- agent-search-specific semantic orchestration changes
- MCP / HTTP exposure of embedding controls
- richer provider-specific embedding lifecycle management

</deferred>

---

*Phase: 08-embedding-backend-and-index-foundation*
*Context gathered: 2026-04-16*
