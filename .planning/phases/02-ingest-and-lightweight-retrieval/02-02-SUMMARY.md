---
phase: 02-ingest-and-lightweight-retrieval
plan: 02
subsystem: search
tags: [rust, sqlite, fts5, libsimple, lexical-search, scoring]
requires:
  - phase: 01-foundation-kernel
    provides: retrieval mode contract, database lifecycle, and status/doctor scaffolding
  - phase: 02-ingest-and-lightweight-retrieval
    provides: authority-backed chunk metadata and synchronous ingest persistence
provides:
  - `libsimple`-bootstrapped external-content FTS5 sidecar on `memory_records`
  - library search service with lexical recall and typed score breakdowns
  - truthful Phase 2 lexical readiness while semantic modes stay deferred
affects: [phase-2, retrieval, ordinary-search, status, lexical-first]
tech-stack:
  added: [libsimple]
  patterns:
    - external-content FTS sidecar over the authority store
    - parameterized lexical recall plus Rust-side score composition
    - mode-aware readiness reporting that preserves deferred semantic modes
key-files:
  created:
    - migrations/0003_lexical_sidecar.sql
    - src/search/mod.rs
    - src/search/lexical.rs
    - src/search/score.rs
    - tests/lexical_search.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - src/core/db.rs
    - src/core/migrations.rs
    - src/core/status.rs
    - src/lib.rs
    - tests/foundation_schema.rs
    - tests/status_cli.rs
key-decisions:
  - "Bootstrapped `libsimple` once per process and applied `set_jieba_dict` per SQLite connection so lexical capability becomes real without changing the single-binary local-first shape."
  - "Kept lexical readiness truthful for `lexical_only` and the hybrid lexical baseline while leaving `embedding_only` / hybrid semantic paths explicitly deferred instead of adding hidden fallbacks."
patterns-established:
  - "Authority-store lexical indexing: `memory_records` stays canonical while FTS sidecar triggers and rebuilds stay additive."
  - "Recall first, score second: SQLite FTS supplies bounded candidates and Rust composes the inspectable final score."
requirements-completed: [ING-03, RET-01, RET-02]
duration: 13min
completed: 2026-04-15
---

# Phase 2 Plan 2: Lexical Recall Summary

**`libsimple`-backed lexical recall with additive FTS sidecar schema, deterministic Rust score breakdowns, and truthful Phase 2 readiness reporting**

## Performance

- **Duration:** 13min
- **Started:** 2026-04-15T12:12:32Z
- **Completed:** 2026-04-15T12:26:01Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- Added schema version 3 with a `libsimple`-powered external-content FTS5 sidecar, sync triggers, and rebuild support layered on top of `memory_records`.
- Implemented a `search` library module that performs parameterized lexical recall over Chinese and PinYin-oriented queries and hydrates authority rows back into typed records.
- Added inspectable Rust score composition so lexical base score remains primary while keyword, importance, and recency bonuses perturb ranking deterministically.
- Updated the canonical status contract so lexical capability is reported as real in Phase 2 without changing the deferred semantic-mode semantics from Phase 1.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add `libsimple` bootstrap and additive lexical sidecar schema** - `080ae91`, `dc170ea` (test, feat)
2. **Task 2: Add lexical recall, Rust scoring, and canonical status readiness assertions** - `7a81015`, `7505d40` (test, feat)

**Plan metadata:** Recorded in the final docs commit for this plan.

## Files Created/Modified

- `Cargo.toml` / `Cargo.lock` - adds and resolves `libsimple` for lexical tokenizer support.
- `migrations/0003_lexical_sidecar.sql` - creates the external-content FTS5 sidecar, sync triggers, and rebuild hook.
- `src/core/db.rs` - bootstraps `libsimple` once and configures the jieba dictionary for each connection.
- `src/core/migrations.rs` - registers schema version 3 as a separate lexical seam after ingest foundation.
- `src/core/status.rs` - inspects lexical sidecar readiness and reports real lexical capability while preserving deferred semantic states.
- `src/lib.rs` - exports the `search` module for library callers.
- `src/search/mod.rs` - defines request/result/search-service types.
- `src/search/lexical.rs` - executes bounded parameterized FTS recall and hydrates authority-backed records.
- `src/search/score.rs` - composes `ScoreBreakdown` values and deterministic final ranking.
- `tests/foundation_schema.rs` - verifies schema version 3, sidecar objects, trigger sync, and rebuild behavior.
- `tests/lexical_search.rs` - verifies mixed-script lexical recall and deterministic score composition.
- `tests/status_cli.rs` - asserts the canonical Phase 2 readiness contract.

## Decisions Made

- Kept the lexical index as an additive sidecar instead of moving retrieval authority away from `memory_records`, so Phase 2 stays explainable and later semantic extensions can remain secondary.
- Used two explicit parameterized SQL strategies, `jieba_query` and `simple_query`, then merged candidates in Rust so Chinese and PinYin recall stay inspectable without raw SQL interpolation.
- Normalized the lexical term before applying bonuses because raw FTS BM25 magnitudes were too small to remain the dominant score term on their own.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `cargo clippy --all-targets -- -D warnings` surfaced a pre-existing unused `NormalizedSource` import in `tests/ingest_pipeline.rs:12`. This file was outside Plan `02-02` scope, so it was logged in `.planning/phases/02-ingest-and-lightweight-retrieval/deferred-items.md` instead of being changed here.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan `02-03` can build citations, filters, and ordinary retrieval APIs on top of the existing `SearchService`, `ScoreBreakdown`, and lexical sidecar.
- The lexical-first contract is now real at the library/status layer, while semantic retrieval remains explicitly deferred for later work.

## Self-Check: PASSED

- Verified `.planning/phases/02-ingest-and-lightweight-retrieval/02-02-SUMMARY.md` exists on disk.
- Verified task commits `080ae91`, `dc170ea`, `7a81015`, and `7505d40` exist in git history.
- Confirmed plan verification results: `cargo test` passed; `cargo clippy --all-targets -- -D warnings` remains blocked only by the pre-existing deferred item logged for `tests/ingest_pipeline.rs`.
