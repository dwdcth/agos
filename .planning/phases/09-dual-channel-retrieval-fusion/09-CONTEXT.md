# Phase 9: Dual-Channel Retrieval Fusion - Context

**Gathered:** 2026-04-16
**Status:** Ready for planning
**Source:** v1.1 milestone definition, Phase 8 embedding foundation outputs, and zero-friction discuss defaults derived from the current retrieval codebase

<domain>
## Phase Boundary

Phase 9 is where embedding becomes behavior, not just substrate. It is responsible for:
- adding embedding second-channel recall on top of the existing lexical-first search contract
- implementing lexical-first fusion, dedupe, and rerank across lexical and embedding candidates
- extending the search result trace so final ranking can explain how lexical and embedding channels contributed

This phase is not responsible for:
- changing lexical-first into embedding-first
- reworking ingest/storage foundation already completed in Phase 8
- changing agent-search orchestration boundaries beyond what is required for ordinary retrieval compatibility
- introducing new interface surfaces such as MCP or HTTP

</domain>

<decisions>
## Implementation Decisions

### Retrieval Role And Fusion Policy
- **D-01:** Lexical-first remains the primary retrieval/explanation channel even when embedding recall is enabled.
- **D-02:** Embedding acts as a second channel for recall expansion and/or rerank signal, not as a replacement for lexical recall.
- **D-03:** Dual-channel retrieval must still return one stable `SearchResponse` / `SearchResult` contract rather than splitting lexical and semantic results into separate product surfaces.

### Candidate Merge And Deduplication
- **D-04:** Lexical and embedding candidates must be merged into one candidate set before final ranking.
- **D-05:** Deduplication should happen on the same retrieval-unit identity the project already uses for citations and authority rows (record/chunk identity), not on lossy text matching.
- **D-06:** If lexical and embedding both surface the same record, the result trace must preserve that both channels contributed.

### Explainability Contract
- **D-07:** Final result traces must be able to say whether a result came from lexical recall only, embedding recall only, or both.
- **D-08:** Lexical citation and authority-row provenance remain the primary explanation source even when embedding contributes recall or rerank signal.
- **D-09:** Phase 9 should not hide channel behavior behind one opaque score; the trace must stay operator-readable.

### Service Boundary
- **D-10:** Dual-channel fusion belongs inside ordinary retrieval services, not in a separate semantic-only bypass path.
- **D-11:** Agent-search should continue to reuse ordinary retrieval once this phase lands, rather than building its own fusion stack.

### Config And Test Matrix
- **D-12:** Phase 9 must add config parsing for the existing repo-root `config.toml` retrieval-related fields that are needed by dual-channel retrieval work.
- **D-13:** The codebase should be able to derive multiple retrieval runtime variants from the existing config contract rather than hand-maintaining unrelated ad hoc test fixtures.
- **D-14:** Phase 9 must include automated coverage for three explicit retrieval modes:
  - lexical-only / lightweight-weighted retrieval
  - embedding-only / vector retrieval
  - hybrid lexical + vector retrieval
- **D-15:** These mode-specific tests should run against generated configs derived from the same config schema, so the test matrix also validates config parsing behavior.

### the agent's Discretion
- Exact fusion math (e.g. rank-based vs weighted-score merge) as long as lexical remains the primary explanation channel.
- Exact trace field names for per-channel contribution.
- Exact module split between search recall, fusion, and rerank helpers.

</decisions>

<specifics>
## Specific Ideas

- Treat this phase as “one retrieval service, two channels” rather than “bolt semantic retrieval on the side.”
- Prefer additive trace fields over replacing the current `ScoreBreakdown` / result trace outright.
- Keep the operator story simple: a developer should be able to look at a result and tell how lexical and embedding each contributed.
- Phase 9 should feel like a careful extension of Phase 2, not like a new retrieval product line.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone scope
- `.planning/PROJECT.md` — current v1.1 milestone goal and active requirements
- `.planning/ROADMAP.md` — Phase 9 goal, success criteria, and plan slots
- `.planning/REQUIREMENTS.md` — `DCR-01`, `DCR-02`, `DCR-03`, `DCR-04`
- `.planning/STATE.md` — current milestone state

### Prior phase outputs
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-02-SUMMARY.md` — lexical recall and score foundation
- `.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md` — current explainable result contract
- `.planning/phases/08-embedding-backend-and-index-foundation/08-CONTEXT.md` — locked embedding-foundation constraints
- `.planning/phases/08-embedding-backend-and-index-foundation/08-01-SUMMARY.md` — backend/readiness foundation
- `.planning/phases/08-embedding-backend-and-index-foundation/08-02-SUMMARY.md` — additive embedding persistence and sidecar schema
- `.planning/phases/08-embedding-backend-and-index-foundation/08-03-SUMMARY.md` — operator-visible embedding substrate diagnostics
- `.planning/phases/08-embedding-backend-and-index-foundation/08-VERIFICATION.md` — foundation-phase proof that lexical-first remained intact

### Runtime/code seams
- `src/search/mod.rs` — ordinary retrieval service boundary
- `src/search/lexical.rs` — lexical recall path
- `src/search/score.rs` — current lexical scoring logic
- `src/search/rerank.rs` — final result shaping and trace
- `src/memory/repository.rs` — additive embedding side-table read path
- `src/core/status.rs` — embedding substrate readiness context
- `config.toml` — existing richer runtime config example that should inform the retrieval-mode test matrix and parsing work

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/search/mod.rs` already has a clean `SearchService::search(...) -> SearchResponse` seam that Phase 9 should extend rather than replace.
- `src/search/lexical.rs` already returns typed lexical candidates with query-strategy trace; embedding recall should mirror that additive-candidate pattern.
- `src/search/score.rs` and `src/search/rerank.rs` already separate candidate scoring from final result shaping, which is the natural insertion point for fusion.
- Phase 8 already created chunk-aligned `record_embeddings` and `record_embedding_index_state`, so Phase 9 can consume semantic substrate without reopening schema design.

### Established Patterns
- additive side-table evolution over the authority store
- lexical-first explainability preserved even when optional capability exists
- one ordinary-retrieval service boundary reused by higher layers
- typed traces preferred over opaque ranking heuristics

### Integration Points
- embedding recall should plug into the candidate-generation path before final rerank
- fusion should dedupe by authority record identity so citations remain stable
- result trace extensions must remain compatible with current `SearchResult` consumers
- config parsing and generated test variants should exercise the same retrieval-mode semantics the service code uses, rather than duplicating hardcoded one-off fixtures

</code_context>

<deferred>
## Deferred Ideas

- human-tunable fusion policy packs
- agent-search-specific semantic prompting or query rewriting
- interface-surface expansion for dual-channel controls
- background re-embedding / model lifecycle automation

</deferred>

---

*Phase: 09-dual-channel-retrieval-fusion*
*Context gathered: 2026-04-16*
