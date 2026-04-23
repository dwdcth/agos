# Phase 9: Dual-Channel Retrieval Fusion - Research

**Researched:** 2026-04-16  
**Domain:** lexical-first + vector second-channel retrieval fusion, config-derived test matrix, explainable dual-channel traces  
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
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

### Deferred Ideas (OUT OF SCOPE)
- human-tunable fusion policy packs
- agent-search-specific semantic prompting or query rewriting
- interface-surface expansion for dual-channel controls
- background re-embedding / model lifecycle automation
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DCR-01 | Ordinary retrieval can run a lexical-first search path plus an embedding second channel within one request flow. [VERIFIED: `.planning/REQUIREMENTS.md`] | Phase 8 created real embedding substrate/state; Phase 9 should now introduce an embedding recall path alongside the existing lexical recall path within the same `SearchService::search(...)` flow. [VERIFIED: `src/search/mod.rs`; VERIFIED: `.planning/phases/08-embedding-backend-and-index-foundation/08-VERIFICATION.md`] |
| DCR-02 | Lexical retrieval remains the primary explanation source even when embedding recall contributes candidates or rerank signal. [VERIFIED: `.planning/REQUIREMENTS.md`] | Existing `SearchResult` already centers citations/snippets around authority rows. Embedding contribution should be additive trace/signal data, not a replacement explanation source. [VERIFIED: `src/search/mod.rs`; VERIFIED: `src/search/rerank.rs`; VERIFIED: `.planning/PROJECT.md`] |
| DCR-03 | Retrieval fusion can dedupe and rank results from lexical and embedding channels into one stable result contract. [VERIFIED: `.planning/REQUIREMENTS.md`] | Current lexical candidate → score → rerank split provides a clean seam for additive candidate merge/fusion before final result shaping. [VERIFIED: `src/search/lexical.rs`; VERIFIED: `src/search/score.rs`; VERIFIED: `src/search/rerank.rs`] |
| DCR-04 | Search results expose enough trace data to tell whether lexical recall, embedding recall, or both contributed to the final ranking. [VERIFIED: `.planning/REQUIREMENTS.md`] | Current result traces already preserve matched query / strategies / filters; Phase 9 should extend that trace model rather than replace it. [VERIFIED: `src/search/rerank.rs`; VERIFIED: `src/search/mod.rs`] |
</phase_requirements>

## Summary

Phase 9 is best understood as a careful extension of the existing lexical retrieval pipeline, not as a new semantic pipeline. The current `SearchService::search(...)` path is already cleanly layered as `lexical.recall -> score_candidates -> rerank_results`. That means the least risky implementation is to add an embedding recall path that produces lexical-like candidate objects, merge those candidates with lexical ones on authority identity, then let scoring/rerank shape the final `SearchResponse`. [VERIFIED: `src/search/mod.rs`; VERIFIED: `src/search/lexical.rs`; VERIFIED: `src/search/score.rs`; VERIFIED: `src/search/rerank.rs`]

The user’s new requirement changes Phase 9 in one important way: config parsing and tests are now first-class scope. The repo-root `config.toml` already describes a richer retrieval runtime with `[embedding]` and `[vector]` sections than the current runtime parser supports. Phase 9 should therefore introduce a small config adapter/parser for retrieval-related fields from that config shape and use it to derive three concrete retrieval variants for tests: lexical-only, embedding-only, and hybrid. This keeps the test matrix anchored to a real user-facing config contract instead of ad hoc fixture duplication. [VERIFIED: `config.toml`; VERIFIED: `src/core/config.rs`; VERIFIED: `.planning/phases/09-dual-channel-retrieval-fusion/09-CONTEXT.md`]

**Primary recommendation:** implement Phase 9 in three slices: (1) retrieval-mode config parsing / generated config matrix, (2) embedding recall + dual-channel candidate merge/fusion inside ordinary retrieval, and (3) result trace/rerank extensions that explain channel contribution while preserving lexical citations as the primary explanation surface. [ASSUMED]

## Current Code Findings

### 1. Search pipeline is already split at the right seams

- `SearchService::search(...)` delegates to lexical recall, scoring, and rerank. [VERIFIED: `src/search/mod.rs`]
- `LexicalSearch::recall(...)` yields typed candidates with `query_strategies`. [VERIFIED: `src/search/lexical.rs`]
- `score_candidates(...)` and `rerank_results(...)` already operate after candidate generation, making them the natural insertion point for dual-channel merge/fusion. [VERIFIED: `src/search/score.rs`; VERIFIED: `src/search/rerank.rs`]

**Implication:** No new top-level retrieval service is needed. Phase 9 should extend the candidate-generation path.

### 2. Phase 8 already made the semantic substrate real

- `record_embeddings` and `record_embedding_index_state` exist. [VERIFIED: `migrations/0006_embedding_foundation.sql`; VERIFIED: `tests/foundation_schema.rs`]
- Ingest can already persist chunk-aligned embedding sidecars keyed to authority records. [VERIFIED: `src/ingest/mod.rs`; VERIFIED: `src/memory/repository.rs`; VERIFIED: `tests/ingest_pipeline.rs`]
- `status` can already report backend and sidecar readiness. [VERIFIED: `src/core/status.rs`; VERIFIED: `tests/status_cli.rs`]

**Implication:** Phase 9 does not need to reopen embedding persistence or readiness — it can focus on consumption and explainable fusion.

### 3. Current config parser is narrower than the real runtime config example

- `src/core/config.rs` currently parses only `[retrieval]` and `[embedding]` in a minimal shape. [VERIFIED: `src/core/config.rs`]
- The repo-root `config.toml` includes `[general]`, `[store]`, `[llm]`, `[embedding]`, and `[vector]`, with provider/model/base URL details. [VERIFIED: `config.toml`]

**Implication:** A Phase 9 test/config adapter should parse the retrieval-relevant subset of this real config shape rather than inventing a second unrelated fixture contract.

## Recommended Implementation Direction

### Config-derived test matrix

Recommended exact approach:

- Introduce a retrieval-test config fixture/parser module that can read the retrieval-relevant subset of `config.toml`:
  - `[embedding]`
  - `[vector]`
  - any retrieval-mode override field added for Phase 9
- Generate three runtime variants from one parsed config base:
  - lexical-only
  - embedding-only
  - hybrid

This allows one shared source config to drive mode-specific tests. [ASSUMED]

### Embedding recall path

Recommended exact approach:

- Add an embedding recall path that reads persisted `record_embeddings` rows and produces candidates keyed by `record_id`.
- Keep candidate identity aligned with authority records/chunk IDs so dedupe is lossless.
- Avoid changing citation ownership: citations still come from `MemoryRecord` / chunk metadata, not from vector rows.

### Fusion and ranking

Recommended exact approach:

- Merge lexical and embedding candidates before final ranking.
- Preserve lexical-first explanation by either:
  - preferring lexical snippet/citation surfaces when both channels hit the same record
  - or keeping lexical score and embedding contribution in distinct trace fields
- Use stable dedupe on `record_id`.

### Trace model

Recommended exact target state:

- Each result can explain:
  - lexical only
  - embedding only
  - both
- The final score breakdown should expose channel contribution without obscuring the current lexical score semantics.

## Testing Direction

### Best regression split

1. **Config / matrix tests**
   - parse current `config.toml`
   - derive lexical-only / embedding-only / hybrid variants

2. **Search integration tests**
   - verify lexical-only behavior remains stable
   - verify embedding-only recall path works over sidecar data
   - verify hybrid dedupe/fusion behavior

3. **Trace/rerank tests**
   - verify final `SearchResponse` says which channel contributed

This maps naturally to existing retrieval test files like `tests/retrieval_cli.rs` and `tests/lexical_search.rs`, with one new dual-channel-focused integration suite likely warranted. [VERIFIED: `tests/retrieval_cli.rs`; VERIFIED: `tests/lexical_search.rs`]

## Anti-Patterns To Avoid

- **Do not hand-maintain three unrelated config fixtures** if they can be derived from one parsed `config.toml` shape.
- **Do not let embedding-only results lose authority/citation identity.** Vector rows are substrate, not explanation objects.
- **Do not bury channel contribution inside one opaque fused score.** Traceability is a locked project constraint.
- **Do not create a semantic-only search service used by agent-search while ordinary retrieval uses something else.** Ordinary retrieval remains the shared seam.

## Recommended Plan Shape

Phase 9 fits three plans:

1. **Plan 09-01:** add retrieval-relevant config parsing + generated config matrix tests for lexical-only / embedding-only / hybrid.
2. **Plan 09-02:** implement embedding recall and lexical-first fusion/dedupe inside ordinary retrieval.
3. **Plan 09-03:** extend result trace/rerank/report contracts so dual-channel contribution stays explainable and regression-tested.

This plan shape directly matches both the roadmap and the user’s new requirement about config-derived test coverage. [VERIFIED: `.planning/ROADMAP.md`; VERIFIED: `.planning/phases/09-dual-channel-retrieval-fusion/09-CONTEXT.md`]
