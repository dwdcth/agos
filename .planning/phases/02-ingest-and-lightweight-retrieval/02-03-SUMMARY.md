---
phase: 02-ingest-and-lightweight-retrieval
plan: 03
subsystem: search
tags: [rust, clap, sqlite, lexical-search, citations, filtering, cli]
requires:
  - phase: 01-foundation-kernel
    provides: clap CLI bootstrap, config loading, and SQLite lifecycle management
  - phase: 02-ingest-and-lightweight-retrieval
    provides: authority-backed ingest rows, lexical sidecar recall, and score breakdown primitives
provides:
  - structured ordinary-retrieval responses with typed citations, filter traces, and validity metadata
  - CLI `ingest` and `search` commands over the library services with JSON and text output modes
  - SQL-level scope, record-type, truth-layer, and validity filtering ahead of lexical rerank
affects: [phase-2, retrieval, ordinary-search, cli, citations, filtering]
tech-stack:
  added: []
  patterns:
    - filter-first lexical recall with explicit applied-filter traces
    - citation-as-data derived only from persisted chunk metadata
    - thin CLI wrappers over ingest and search services
key-files:
  created:
    - src/search/filter.rs
    - src/search/citation.rs
    - src/search/rerank.rs
    - tests/retrieval_cli.rs
  modified:
    - src/search/mod.rs
    - src/search/lexical.rs
    - src/search/score.rs
    - src/interfaces/cli.rs
    - src/ingest/detect.rs
    - src/ingest/normalize.rs
    - src/ingest/mod.rs
    - tests/lexical_search.rs
key-decisions:
  - "Returned a structured `SearchResponse` with per-result trace and citation data instead of extending the old bare `Vec<SearchResult>` shape piecemeal."
  - "Applied scope, record type, truth layer, and validity filters inside the lexical SQL recall step so filtering remains auditable and does not degrade into post-hoc Rust-side dropping."
  - "Kept CLI ingest/search as synchronous wrappers over `IngestService` and `SearchService`, preserving the no-Rig, no-LLM ordinary retrieval contract."
patterns-established:
  - "Explainable ordinary retrieval: each result carries persisted chunk citation data, typed score breakdowns, and the exact applied filters."
  - "CLI/library parity: JSON and text rendering happen after service calls, with no duplicate retrieval logic in the command layer."
requirements-completed: [RET-03, RET-04, RET-05, AGT-01]
duration: 9min
completed: 2026-04-15
---

# Phase 2 Plan 3: Ordinary Retrieval Surface Summary

**Explainable lexical-first retrieval with SQL-backed filters, persisted chunk citations, and thin `ingest` / `search` CLI commands**

## Performance

- **Duration:** 9min
- **Started:** 2026-04-15T12:35:23Z
- **Completed:** 2026-04-15T12:44:25Z
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments

- Extended the ordinary retrieval library contract to return structured citations, applied-filter traces, validity metadata, and stable score breakdowns for every ranked result.
- Added SQL-level filtering for scope, record type, truth layer, validity, and recorded-at windows so lexical recall stays deterministic and auditable before rerank.
- Wired `agent-memos ingest` and `agent-memos search` as thin wrappers over the existing library services, with both JSON and text output modes and no Rig or LLM dependency.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add filtering, citations, and explainable ranked result contracts** - `259c7ee`, `fdb787e` (test, feat)
2. **Task 2: Wire thin CLI commands for ingest and ordinary retrieval** - `f379df1` (feat)

**Plan metadata:** Recorded in the final docs commit for this plan.

## Files Created/Modified

- `src/search/filter.rs` - defines typed ordinary-retrieval filters and applied-filter traces.
- `src/search/citation.rs` - builds citations only from persisted source/chunk metadata.
- `src/search/rerank.rs` - shapes scored candidates into stable search responses with per-result trace data.
- `src/search/mod.rs` - exports the structured search request/response contracts and service entrypoint.
- `src/search/lexical.rs` - applies typed filters during FTS recall and preserves lexical-first candidate ordering.
- `src/search/score.rs` - keeps inspectable score breakdowns while separating scoring from final result shaping.
- `src/interfaces/cli.rs` - adds `ingest` and `search` commands with typed flag parsing and JSON/text rendering.
- `src/ingest/detect.rs`, `src/ingest/normalize.rs`, `src/ingest/mod.rs` - derive serialization for CLI JSON output without changing ingest behavior.
- `tests/retrieval_cli.rs` - covers library retrieval contracts and CLI ingest/search end-to-end flows.
- `tests/lexical_search.rs` - updates lexical-search assertions to the new structured response shape.

## Decisions Made

- Promoted filter traces to first-class response data so operators can see exactly which scope, type, truth, and validity constraints shaped a result.
- Treated missing chunk metadata as a citation error instead of silently synthesizing anchors from snippets, preserving the explainability contract.
- Kept CLI text rendering intentionally thin and human-readable while emitting the full structured response in JSON mode.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Collapsed lexical filter bindings and retrieval test helper arity to satisfy the verification gate**
- **Found during:** plan-level verification after Task 2
- **Issue:** `cargo clippy --all-targets -- -D warnings` failed on the new lexical recall helper and retrieval test helper because both exceeded the configured argument-count lint threshold.
- **Fix:** Replaced the repeated SQL filter arguments with a typed `RecallFilters` helper in `src/search/lexical.rs` and converted the retrieval fixture helper in `tests/retrieval_cli.rs` into a struct-backed input.
- **Files modified:** `src/search/lexical.rs`, `tests/retrieval_cli.rs`
- **Verification:** `cargo test` remained green; `cargo clippy --all-targets -- -D warnings` then failed only on the previously deferred `tests/ingest_pipeline.rs:12` unused import.
- **Committed in:** `080786e`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The fix was verification-driven and internal. It did not change the plan scope or public behavior.

## Issues Encountered

- `cargo clippy --all-targets -- -D warnings` still fails on `tests/ingest_pipeline.rs:12` because of a pre-existing unused `NormalizedSource` import already recorded in `.planning/phases/02-ingest-and-lightweight-retrieval/deferred-items.md`. Per executor scope rules, that file was left untouched in this plan.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 2 ordinary retrieval is now complete at both the library and CLI surfaces, with citations, filters, and explainable score traces available without agent orchestration.
- Phase 3 can build truth-layer governance on top of the existing `truth_layer` filter contract and persisted citation/provenance metadata.
- The only remaining verification debt in the phase directory is the pre-existing `tests/ingest_pipeline.rs` clippy issue.

## Self-Check: PASSED

- Verified `.planning/phases/02-ingest-and-lightweight-retrieval/02-03-SUMMARY.md` exists on disk.
- Verified task/deviation commits `259c7ee`, `fdb787e`, `f379df1`, and `080786e` exist in git history.
- Confirmed plan verification results: `cargo test --test retrieval_cli library_search_returns_citations_and_filter_trace -- --nocapture` passed, `cargo test --test retrieval_cli -- --nocapture` passed, and `cargo test` passed.
- Confirmed `cargo clippy --all-targets -- -D warnings` is blocked only by the pre-existing deferred item at `tests/ingest_pipeline.rs:12`.
- Scanned files touched by this plan for placeholder/stub patterns and found no functional stubs.
